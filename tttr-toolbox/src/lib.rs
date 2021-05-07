//!
//! # TTTR Toolbox
//! The fastest streaming algorithms for your TTTR data.
//!
//! TTTR Toolbox can be used as a standalone Rust library. If you do most of your data
//! analysis in Python you may prefer to check Trattoria, a wrapper library for this
//! crate.
//!
//! ## Project Goals
//! - Single threaded performance
//! - Ease of extensibility
//!
//! ## Algorithms available
//! - [second order autocorrelation](tttr_tools/g2/fn.g2.html)
//! - [intensity time trace](tttr_tools/timetrace/fn.timetrace.html)
//! - [record number time trace](tttr_tools/timetrace/fn.timetrace.html)
//! - [zero delay finder](tttr_tools/zero_finder/fn.zerofinder.html)
//!
//! ## Supported file and record formats
//! - PicoQuant PTU
//!   - PHT2
//!   - HHT2_HH1
//!   - HHT2_HH2
//!   - HHT3_HH2
//!
//! If you want support for more record formats and file formats please ask for it.
//! At the very least we will need the file format specification and a file with some
//! discernible features to test the implementation.
//!
//! ## Examples
//! ```ignore
//! pub fn main() {
//!     let filename = PathBuf::from("/Users/garfield/Downloads/20191205_Xminus_0p1Ve-6_CW_HBT.ptu");
//!     let ptu_file = File::PTU(PTUFile::new(filename).unwrap());
//!     // Unwrap the file so we can print the header
//!     let File::PTU(f) = &ptu_file;
//!     println!("{}", f);
//!
//!     let params = G2Params {
//!         channel_1: 0,
//!         channel_2: 1,
//!         correlation_window: 50_000e-12,
//!         resolution: 600e-12,
//!         start_record: None,
//!         stop_record: None,
//!     };
//!     let g2_histogram = g2(&ptu_file, &params).unwrap();
//!     println!("{:?}", g2_histogram.hist);
//! }
//! ```

#[macro_use]
extern crate num_derive;
extern crate byteorder;

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
    fn channel(&self) -> &i32 {
        &self.channel
    }
    #[inline]
    fn tof(&self) -> &u64 {
        &self.tof
    }
}

/// The TTTRFile trait ensures that all files we support are aware of the time_resolution
/// and the what type of records they contain.
///
/// TTTR files don't usually represent time in seconds but rather as a multiple of them
/// that matches the equipment time resolution. This makes it possible to shave a few
/// bits per record.
pub trait TTTRFile {
    fn time_resolution(&self) -> Result<f64, errors::Error>;
    fn record_type(&self) -> Result<headers::RecordType, errors::Error>;
}
