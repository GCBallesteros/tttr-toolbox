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
    //let filename = PathBuf::from("/Users/garfield/Downloads/20191205_Xminus_0p1Ve-6_CW_HBT.ptu");
    let filename = PathBuf::from("/Users/garfield/Downloads/GUI_T3_10s.ptu");
    let ptu_file = File::PTU(PTUFile::new(filename).unwrap());
    let File::PTU(f) = &ptu_file;
    println!("{}", f);
    let start = Instant::now();
    //let params = TimeTraceParams {resolution: 0.1, channel: Some(1)};
    let params = G2Params {
        channel_1: 2,
        channel_2: 1,
        correlation_window: 30e-9,
        resolution: 30e-12,
        start_record: None,
        stop_record: None,
    };
    let g2_histogram = g2(&ptu_file, &params).unwrap();
    //let tt = timetrace(&ptu_file, &params).unwrap();
    eprintln!("elapsed {:?}", start.elapsed());
    println!("{:?}", g2_histogram.hist);
    println!("{:?}", g2_histogram.t);
    //println!("{:?}", tt.intensity);
}
