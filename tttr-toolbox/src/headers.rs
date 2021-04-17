pub enum RecordType {
    PHT2,
    #[allow(non_camel_case_types)]
    HHT2_HH1,
    #[allow(non_camel_case_types)]
    HHT2_HH2,
    NotImplemented,
}

pub enum File {
    PTU(crate::parsers::ptu::PTUFile),
}
