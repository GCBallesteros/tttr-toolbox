use std::path::PathBuf;
use std::time::Instant;

extern crate tttr_toolbox_proc_macros;
//use tttr_toolbox::tttr_tools::timetrace::{timetrace, TimeTraceParams};
use tttr_toolbox::tttr_tools::g2::{g2, G2Params};
use tttr_toolbox::parsers::ptu::PTUFile;
use tttr_toolbox::headers::File;

// ToDo
// 2. Check magic number for PTU


pub fn main() {
    let filename = PathBuf::from("/Users/garfield/Downloads/20191205_Xminus_0p1Ve-6_CW_HBT.ptu");
    let ptu_file = File::PTU(PTUFile::new(filename).unwrap());
    let File::PTU(f) = &ptu_file;
    println!("{}", f);
    //println!("{}", answer());
    let start = Instant::now();
    //let params = TimeTraceParams {resolution: 10, channel: Some(0)};
    //let intensity_trace = timetrace(&ptu_file, &params);
    let params = G2Params {
        channel_1: 0,
        channel_2: 1,
        correlation_window: 50_000e-12,
        resolution: 600e-12,
        start_record: None,
        stop_record: None,
    };
    //g2_resolution = 600 * 1e-12  # picoseconds * 1e-12 to change to seconds
    //g2_window = 50000 * 1e-12
    let g2_histogram = g2(&ptu_file, &params).unwrap();
    eprintln!("elapsed {:?}", start.elapsed());
    println!("{:?}", g2_histogram.hist);
}
