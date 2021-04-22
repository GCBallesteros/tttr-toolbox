pub mod header;
pub mod streamers;

use std::collections::HashMap;
use std::path::PathBuf;

use num_traits::FromPrimitive;

use pyo3;

use crate::errors::Error;
use crate::headers;
use crate::TTTRFile;

pub type Header = HashMap<String, PTUTag>;

#[derive(Debug)]
pub enum PTUTag {
    Empty8,
    Bool8(bool),
    Int8(i64),
    BitSet64(i64),
    Color8(i64),
    Float8(f64),
    TDateTime(f64),
    Float8Array(Vec<f64>),
    AnsiString8(String),
    WideString(String),
    BinaryBlob(Vec<u8>),
}

impl pyo3::ToPyObject for PTUTag {
    fn to_object(&self, py: pyo3::Python) -> pyo3::PyObject {
        match self {
            PTUTag::Empty8 => (py.None(), "Empty8").to_object(py),
            PTUTag::Bool8(x) => (x, "Bool8").to_object(py),
            PTUTag::Int8(x) => (x, "Int8").to_object(py),
            PTUTag::BitSet64(x) => (x, "BitSet64").to_object(py),
            PTUTag::Color8(x) => (x, "Color8").to_object(py),
            PTUTag::Float8(x) => (x, "Float8").to_object(py),
            PTUTag::TDateTime(x) => (x, "TDateTime").to_object(py),
            PTUTag::Float8Array(x) => (x, "Float8Array").to_object(py),
            PTUTag::AnsiString8(x) => (x, "AnsiString8").to_object(py),
            PTUTag::WideString(x) => (x, "WideString").to_object(py),
            PTUTag::BinaryBlob(x) => (x, "BinaryBlob").to_object(py),
        }
    }
}

#[derive(FromPrimitive, ToPrimitive, Debug)]
enum PTUTagType {
    Empty8 = 0xFFFF0008,
    Bool8 = 0x00000008,
    Int8 = 0x10000008,
    BitSet64 = 0x11000008,
    Color8 = 0x12000008,
    Float8 = 0x20000008,
    TDateTime = 0x21000008,
    Float8Array = 0x2001FFFF,
    AnsiString8 = 0x4001FFFF,
    WideString = 0x4002FFFF,
    BinaryBlob = 0xFFFFFFFF,
}

#[derive(FromPrimitive, ToPrimitive, Debug)]
enum RecType {
    PicoHarpT3 = 0x00010303, // (SubID = $00 ,RecFmt: $01) (V1), T-Mode: $03 (T3), HW: $03 (PicoHarp)
    PicoHarpT2 = 0x00010203, // (SubID = $00 ,RecFmt: $01) (V1), T-Mode: $02 (T2), HW: $03 (PicoHarp)
    HydraHarpT3 = 0x00010304, // (SubID = $00 ,RecFmt: $01) (V1), T-Mode: $03 (T3), HW: $04 (HydraHarp)
    HydraHarpT2 = 0x00010204, // (SubID = $00 ,RecFmt: $01) (V1), T-Mode: $02 (T2), HW: $04 (HydraHarp)
    HydraHarp2T3 = 0x01010304, // (SubID = $01 ,RecFmt: $01) (V2), T-Mode: $03 (T3), HW: $04 (HydraHarp)
    HydraHarp2T2 = 0x01010204, // (SubID = $01 ,RecFmt: $01) (V2), T-Mode: $02 (T2), HW: $04 (HydraHarp)
    TimeHarp260NT3 = 0x00010305, // (SubID = $00 ,RecFmt: $01) (V1), T-Mode: $03 (T3), HW: $05 (TimeHarp260N)
    TimeHarp260NT2 = 0x00010205, // (SubID = $00 ,RecFmt: $01) (V1), T-Mode: $02 (T2), HW: $05 (TimeHarp260N)
    TimeHarp260PT3 = 0x00010306, // (SubID = $00 ,RecFmt: $01) (V1), T-Mode: $03 (T3), HW: $06 (TimeHarp260P)
    TimeHarp260PT2 = 0x00010206, // (SubID = $00 ,RecFmt: $01) (V1), T-Mode: $02 (T2), HW: $06 (TimeHarp260P)
}

const TAG_TTTR_REC_TYPE: &str = "TTResultFormat_TTTRRecType";
const TAG_NUM_RECORDS: &str = "TTResult_NumberOfRecords"; // Number of TTTR Records in the File;
const TAG_GLOB_RES: &str = "MeasDesc_GlobalResolution"; // Global Resolution of TimeTag(T2) /NSync (T3)
const FILE_TAG_END: &str = "Header_End"; // Always appended as last tag (BLOCKEND)
const _TAG_ACQUISITION_TIMETTTR: &str = "MeasDesc_AcquisitionTime";
const _TAG_RES: &str = "MeasDesc_Resolution"; // Resolution for the Dtime (T3 Only)

/// Metadata for a PTU file from PicoQuant
pub struct PTUFile {
    pub path: PathBuf,
    pub header: Header,
}

impl PTUFile {
    /// Create a PTUFile from its filepath.
    ///
    /// If the file does not exist a FileNotAvailable error will be returned.
    pub fn new(filename: PathBuf) -> Result<Self, Error> {
        // check if file in path exists
        if filename.exists() {
            let header = self::header::read_ptu_header(&filename)?;
            Ok(Self {
                path: filename,
                header,
            })
        } else {
            let filename_string = filename.display().to_string();
            Err(Error::FileNotAvailable(filename_string))
        }
    }
}

use tttr_toolbox_proc_macros::read_ptu_tag;

impl TTTRFile for PTUFile {
    fn time_resolution(&self) -> Result<f64, Error> {
        let header = &self.header;
        Ok(read_ptu_tag!(header[TAG_GLOB_RES] as Float8))
    }

    /// Returns the `record_type` used in the file. This is matched on each algorithm
    /// with a specific file parser.
    fn record_type(&self) -> Result<headers::RecordType, Error> {
        let header = &self.header;
        let record_type = FromPrimitive::from_i64(read_ptu_tag!(header[TAG_TTTR_REC_TYPE] as Int8));

        Ok(
            match record_type
                .ok_or_else(|| Error::InvalidHeader(String::from("Invalid RecordType type")))?
            {
                RecType::PicoHarpT3 => headers::RecordType::NotImplemented,
                RecType::PicoHarpT2 => headers::RecordType::PHT2,
                RecType::HydraHarpT3 => headers::RecordType::NotImplemented,
                RecType::HydraHarpT2 => headers::RecordType::HHT2_HH2,
                RecType::HydraHarp2T3 => headers::RecordType::NotImplemented,
                RecType::HydraHarp2T2 => headers::RecordType::HHT2_HH1,
                RecType::TimeHarp260NT3 => headers::RecordType::NotImplemented,
                RecType::TimeHarp260NT2 => headers::RecordType::HHT2_HH2,
                RecType::TimeHarp260PT3 => headers::RecordType::NotImplemented,
                RecType::TimeHarp260PT2 => headers::RecordType::HHT2_HH2,
            },
        )
    }
}

impl std::fmt::Display for PTUFile {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        let mut string = String::from("");
        for (key, value) in &self.header {
            string.push_str(&format!("{:<35}: {}\n", key, value));
        }
        write!(f, "{}", string)
    }
}
