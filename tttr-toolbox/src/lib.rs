#[macro_use]
extern crate num_derive;
extern crate byteorder;
//extern crate tttr_toolbox_proc_macros;

pub mod errors;
pub mod headers;
pub mod parsers;
pub mod tttr_tools;

pub(crate) trait TTTRStream {
    type RecordSize;
    fn parse_record(&mut self, raw_record: Self::RecordSize) -> TTTRRecord;
    fn time_resolution(&self) -> f64;
}

#[derive(Debug)]
pub struct TTTRRecord {
    channel: i32,
    tof: u64,
}

pub(crate) trait Click {
    fn channel(&self) -> &i32;
    fn tof(&self) -> &u64;
}

impl Click for TTTRRecord {
    #[inline]
    fn channel(&self) -> &i32 {&self.channel}
    #[inline]
    fn tof(&self) -> &u64 {&self.tof}

}

/// The TTTRFile trait ensures that all files we support are aware of the time_resolution
/// they support and the what type of records they contain. This is neccessary 
pub trait TTTRFile {
    fn time_resolution(&self) -> Result<f64, errors::Error>;
    fn record_type(&self) -> Result<headers::RecordType, errors::Error>;
}

