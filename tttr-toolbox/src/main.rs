use std::path::PathBuf;
use std::time::Instant;

extern crate tttr_toolbox_proc_macros;
use tttr_toolbox::tttr_tools::timetrace::{timetrace, TimeTraceParams};
use tttr_toolbox::tttr_tools::g2::{g2, G2Params};
use tttr_toolbox::tttr_tools::lifetime::{lifetime, LifetimeParams};
use tttr_toolbox::parsers::ptu::PTUFile;
use tttr_toolbox::headers::File;

// ToDo
// 2. Check magic number for PTU


pub fn main() {
    //let filename = PathBuf::from("/Users/garfield/Downloads/20191205_Xminus_0p1Ve-6_CW_HBT.ptu");
    let filename = PathBuf::from("/Users/garfield/Downloads/GUI_T3_10s.ptu");
    //let filename = PathBuf::from("/Users/garfield/Downloads/GUI_T2.ptu");
    let ptu_file = File::PTU(PTUFile::new(filename).unwrap());
    let File::PTU(f) = &ptu_file;
    println!("{}", f);
    let start = Instant::now();
    //let params = TimeTraceParams {resolution: 0.1, channel: None};
    let params = LifetimeParams {
        channel_sync: 0,
        channel_source: 1,
        resolution: 60e-12,
        start_record: None,
        stop_record: None,
    };
    //let params = G2Params {
        //channel_1: 0,
        //channel_2: 1,
        //correlation_window: 10e-9,
        //resolution: 60e-12,
        //start_record: None,
        //stop_record: None,
    //};
    //let g2_histogram = g2(&ptu_file, &params).unwrap();
    //let tt = timetrace(&ptu_file, &params).unwrap();
    let lifetime_histogram = lifetime(&ptu_file, &params).unwrap();
    eprintln!("elapsed {:?}", start.elapsed());
    println!("{:?}", lifetime_histogram.hist);
    println!("{:?}", lifetime_histogram.t);
    //println!("{:?}", g2_histogram.hist);
    //println!("{:?}", g2_histogram.t);
    //println!("{:?}", tt.intensity);
}
