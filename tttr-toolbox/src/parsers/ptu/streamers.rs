const BUFFER_SIZE: usize = 1024 * 16;

use std::io::{BufReader, Seek, SeekFrom};

use crate::errors::Error;
use crate::parsers::ptu;
use crate::parsers::ptu::{PTUTag, TAG_NUM_RECORDS};
use crate::{TTTRFile, TTTRRecord, TTTRStream};

use byteorder::{NativeEndian, ReadBytesExt};

use tttr_toolbox_proc_macros::make_ptu_stream;
use tttr_toolbox_proc_macros::read_ptu_tag;

// - - - - - - - - - - //
// PHT2 Record Stream //
// - - - - - - - - - - //
#[make_ptu_stream(PHT2)]
fn parse_record(&mut self, record: Self::RecordSize) -> TTTRRecord {
    const T2WRAPAROUND: u64 = 210698240;

    let ch = ((record & 0b11110000000000000000000000000000) >> 28) as i32;
    let tm = (record & 0b00001111111111111111111111111111) as u64;

    let tof;
    let channel;

    if ch == 0xF {
        // we have a special record
        let markers = tm & 0xF;
        if markers == 0 {
            // overflow record
            tof = 0;
            channel = -1;
            self.overflow_correction += T2WRAPAROUND; // unwrap the time tag overflow
        } else {
            // marker
            // Strictly, in case of a marker, the lower 4 bits of time are invalid
            // because they carry the marker bits. So one could zero them out.
            // However, the marker resolution is only a few tens of nanoseconds anyway,
            // so we can just ignore the few picoseconds of error.
            tof = self.overflow_correction + tm;
            channel = -2;
        }
    } else {
        tof = self.overflow_correction + tm;
        channel = ch;
    }

    TTTRRecord {
        channel: channel as i32,
        tof,
    }
}

// - - - - - - - - - - - -//
// HHT2_HH1 Record Stream //
// - - - - - - - - - - - -//
#[make_ptu_stream(HHT2_HH1)]
fn parse_record(&mut self, record: Self::RecordSize) -> TTTRRecord {
    const T2WRAPAROUND: u64 = 33552000;

    let sp = (((record & 0b10000000000000000000000000000000) >> 31) == 1) as i32;
    let ch = ((record & 0b01111110000000000000000000000000) >> 25) as i32;
    let tm = (record & 0b00000001111111111111111111111111) as u64;

    let tof;
    let channel;

    self.overflow_correction += T2WRAPAROUND * (sp as u64) * ((ch == 0x3F) as u64);
    channel = (1 - sp) * (ch + 1) - sp * ch;
    tof = self.overflow_correction + tm;

    //println!("channel: {:?}, ch: {:?}, sp: {:?}", channel, ch, sp);

    TTTRRecord { channel, tof }
}

// - - - - - - - - - - - -//
// HHT2_HH2 Record Stream //
// - - - - - - - - - - - -//
#[make_ptu_stream(HHT2_HH2)]
fn parse_record(&mut self, record: Self::RecordSize) -> TTTRRecord {
    const T2WRAPAROUND: u64 = 33554432;

    let sp = (((record & 0b10000000000000000000000000000000) >> 31) == 1) as i32;
    let ch = ((record & 0b01111110000000000000000000000000) >> 25) as i32;
    let tm = (record & 0b00000001111111111111111111111111) as u64;

    let tof;
    let channel;

    self.overflow_correction += T2WRAPAROUND * tm * ((ch == 0x3F) as u64);
    channel = (1 - sp) * (ch + 1) - sp * ch; // ch +1 - sp ch -sp - sp ch
    tof = self.overflow_correction + tm;

    TTTRRecord { channel, tof }
}

// - - - - - - - - - - - -//
// HHT3_HH2 Record Stream //
// - - - - - - - - - - - -//

// T3 mode records require to carry around metadata so the macro we were using for T2
// doesn't work.

#[allow(non_camel_case_types)]
pub struct HHT3_HH2Stream {
    // todo: make it just with a trait that implements readbuf
    source: BufReader<std::fs::File>,
    click_buffer: [u32; BUFFER_SIZE],
    num_records: usize,
    time_resolution: f64,
    photons_in_buffer: i32,
    click_count: usize,
    nsync: u64,
    pub sync_period: u64,
    dtime_res: u64,
}

impl HHT3_HH2Stream {
    pub fn new(
        ptu_file: &ptu::PTUFile,
        start_record: Option<usize>,
        stop_record: Option<usize>,
    ) -> Result<Self, Error> {
        let header = &ptu_file.header;
        let number_of_records: i64 = read_ptu_tag!(header[TAG_NUM_RECORDS] as Int8);
        let data_offset: i64 = read_ptu_tag!(header["DataOffset"] as Int8);

        let mut buffered =
            BufReader::with_capacity(8 * 1024, std::fs::File::open(ptu_file.path.clone())?);

        let record_offset = if let Some(offset) = start_record {
            offset as i64
        } else {
            0 as i64
        };

        let last_record = if let Some(last) = stop_record {
            last as i64
        } else {
            number_of_records as i64
        };

        // 4 bytes per record
        buffered.seek(SeekFrom::Start(
            (data_offset as u64) + (4 * record_offset) as u64,
        ))?;

        let header = &ptu_file.header;

        let sync_period: Result<f64, Error> =
            Ok(read_ptu_tag!(header["MeasDesc_GlobalResolution"] as Float8));
        let dtime_res: Result<f64, Error> =
            Ok(read_ptu_tag!(header["MeasDesc_Resolution"] as Float8));

        Ok(Self {
            source: buffered,
            click_buffer: [0; BUFFER_SIZE],
            num_records: (last_record - record_offset) as usize,
            time_resolution: 1e-12,
            photons_in_buffer: 0,
            click_count: 0,
            nsync: 0,
            sync_period: (sync_period? * 1e12) as u64,
            dtime_res: (dtime_res? * 1e12) as u64,
        })
    }
}

impl TTTRStream for HHT3_HH2Stream {
    type RecordSize = u32;
    #[inline(always)]
    fn parse_record(&mut self, record: Self::RecordSize) -> TTTRRecord {
        const T3WRAPAROUND: u64 = 1024;

        //  TimeTag: Raw TimeTag from Record * Globalresolution = Real Time arrival of Photon
        //  DTime: Arrival time of Photon after last Sync event (T3 only) DTime * Resolution = Real time arrival of Photon after last Sync event
        //  Channel: Channel the Photon arrived (0 = Sync channel for T2 measurements)
        let sp = (((record & 0b10000000000000000000000000000000) >> 31) == 1) as i32;
        let ch = ((record & 0b01111110000000000000000000000000) >> 25) as i32;
        let dtime = ((record & 0b00000001111111111111110000000000) >> 10) as u64;
        let nsync = (record & 0b00000000000000000000001111111111) as u64;

        let tof;
        let channel;

        if sp == 1 {
            if ch == 0x3F {
                if nsync == 0 {
                    //if it is zero or old version it is an old style single overflow
                    self.nsync += T3WRAPAROUND;
                } else {
                    self.nsync += T3WRAPAROUND * nsync;
                }
                tof = self.nsync * self.sync_period;
                channel = 0;
            } else if (ch >= 1) && (ch <= 15) {
                // markers
                tof = self.nsync * self.sync_period; // wrong look at picoquant for correct value
                channel = -1;
            } else {
                tof = 0;
                channel = -1;
            }
            // At the current time we ignore markers. This is signalled by returnig a
            //negative channel number.
        } else {
            let truensync = self.nsync + nsync;
            //the nsync time unit depends on sync period which can be obtained from the file header
            //the dtime unit depends on the resolution and can also be obtained from the file header
            tof = (truensync * self.sync_period + dtime * self.dtime_res) as u64;

            channel = ch + 1;
        }
        //println!("channel: {:?}, ch: {:?}, sp: {:?}", channel, ch, sp);
        TTTRRecord { channel, tof }
    }

    fn time_resolution(&self) -> f64 {
        self.time_resolution
    }
}

impl Iterator for HHT3_HH2Stream {
    type Item = TTTRRecord;

    #[inline(always)]
    fn next(&mut self) -> Option<Self::Item> {
        if self.photons_in_buffer == 0 {
            let read_res = self
                .source
                .read_u32_into::<NativeEndian>(&mut self.click_buffer[..]);
            if let Err(_x) = read_res {
                //if self.click_count < self.num_records {
                //println!("Missed {}", self.num_records - self.click_count);
                //}
                return None;
            };
            if self.click_count >= self.num_records {
                return None;
            };
            self.photons_in_buffer = BUFFER_SIZE as i32;
        }

        let current_photon = ((BUFFER_SIZE as i32) - self.photons_in_buffer) as usize;
        self.photons_in_buffer -= 1;
        self.click_count += 1;
        Some(self.parse_record(self.click_buffer[current_photon]))
    }
}
