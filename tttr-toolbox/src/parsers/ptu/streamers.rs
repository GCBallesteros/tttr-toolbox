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
    let ch = ((record & 0b01111000000000000000000000000000) >> 27) as i32;
    let tm = (record & 0b00000111111111111111111111111111) as u64;

    let tof;
    let channel;

    self.overflow_correction += T2WRAPAROUND * ((ch == 0x3F) as u64);
    channel = (1 - sp) * (ch + 1) - sp * ch; // ch +1 - sp ch -sp - sp ch
    tof = self.overflow_correction + tm;

    TTTRRecord { channel, tof }
}

// - - - - - - - - - - - -//
// HHT2_HH2 Record Stream //
// - - - - - - - - - - - -//
#[make_ptu_stream(HHT2_HH2)]
fn parse_record(&mut self, record: Self::RecordSize) -> TTTRRecord {
    const T2WRAPAROUND: u64 = 33552000;

    let sp = (((record & 0b10000000000000000000000000000000) >> 31) == 1) as i32;
    let ch = ((record & 0b01111000000000000000000000000000) >> 27) as i32;
    let tm = (record & 0b00000111111111111111111111111111) as u64;

    let tof;
    let channel;

    self.overflow_correction += T2WRAPAROUND * tm * ((ch == 0x3F) as u64);
    channel = (1 - sp) * (ch + 1) - sp * ch; // ch +1 - sp ch -sp - sp ch
    tof = self.overflow_correction + tm;

    TTTRRecord { channel, tof }
}
