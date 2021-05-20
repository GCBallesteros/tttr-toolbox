use std::path::PathBuf;

extern crate tttr_toolbox_proc_macros;
extern crate clap;

use clap::{Arg, App, SubCommand};

use tttr_toolbox::tttr_tools::timetrace::{timetrace, TimeTraceParams};
use tttr_toolbox::tttr_tools::g2::{g2, G2Params};
use tttr_toolbox::tttr_tools::lifetime::{lifetime, LifetimeParams};
use tttr_toolbox::parsers::ptu::PTUFile;
use tttr_toolbox::headers::File;

// ToDo
// 2. Check magic number for PTU


pub fn main() {
    let matches = App::new("TTTR Toolbox")
        .version("0.3")
        .author("Guillem Ballesteros")
        .about("Apply streaming algorithms to TTTR data")
        .arg(
            Arg::with_name("with_header")
            .short("header")
            .takes_value(false)
            .global(true)
        )
        .subcommand(
            SubCommand::with_name("intensity")
            .about("Obtain intensity trace for one or all channels")
            .arg(
                Arg::with_name("input")
                .short("i")
                .help("Input file path")
                .takes_value(true)
                .required(true)
            )
            .arg(
                Arg::with_name("resolution")
                .short("r")
                .help("Time resolution of the intensity trace")
                .takes_value(true)
                .required(true)
            )
            .arg(
                Arg::with_name("channel")
                .short("c")
                .help("Channel number")
                .takes_value(true)
                .required(false)
            )
        )
        .subcommand(
            SubCommand::with_name("lifetime")
            .about("Compute the lifetime histogram from a pulsed excitation experiment. Only supports T3 mode.")
            .arg(
                Arg::with_name("input")
                .short("i")
                .help("Input file path")
                .takes_value(true)
                .required(true)
            )
            .arg(
                Arg::with_name("ch_sync")
                .help("Sync channel")
                .takes_value(true)
                .required(true)
            )
            .arg(
                Arg::with_name("ch_source")
                .help("Source channel")
                .takes_value(true)
                .required(true)
            )
            .arg(
                Arg::with_name("resolution")
                .short("r")
                .help("Time resolution of the lifetime histogram")
                .takes_value(true)
                .required(true)
            )
        )
        .subcommand(
            SubCommand::with_name("g2")
            .about("Compute second order coincidences between two channels")
            .arg(
                Arg::with_name("input")
                .short("i")
                .help("Input file path")
                .takes_value(true)
                .required(true)
            )
            .arg(
                Arg::with_name("channel1")
                .short("1")
                .help("First channel")
                .takes_value(true)
                .required(true)
            )
            .arg(
                Arg::with_name("channel2")
                .short("2")
                .help("Second channel")
                .takes_value(true)
                .required(true)
            )
            .arg(
                Arg::with_name("correlation_window")
                .short("w")
                .help("Length of the correlation window in seconds")
                .takes_value(true)
                .required(true)
            )
            .arg(
                Arg::with_name("resolution")
                .short("r")
                .help("Time resolution of the g2 histogram")
                .takes_value(true)
                .required(true)
            )
        )
        .get_matches();


    match matches.subcommand() {
        ("intensity", Some(intensity_matches)) => {
            let filename = PathBuf::from(intensity_matches.value_of("input").unwrap());
            let ptu_file = File::PTU(PTUFile::new(filename).unwrap());
            let params = TimeTraceParams {
                resolution: intensity_matches.value_of("resolution").unwrap().parse::<f64>().unwrap(),
                channel: intensity_matches.value_of("channel").map(|x| {x.parse::<i32>().unwrap()}),
            };
            let tt = timetrace(&ptu_file, &params).unwrap();
            println!("{:?}", tt.intensity);
            println!("{:?}", tt.recnum_trace);
        },
        ("g2", Some(g2_matches)) => {
            let filename = PathBuf::from(g2_matches.value_of("input").unwrap());
            let ptu_file = File::PTU(PTUFile::new(filename).unwrap());
            let params = G2Params {
                channel_1: g2_matches.value_of("1").unwrap().parse::<i32>().unwrap(),
                channel_2: g2_matches.value_of("2").unwrap().parse::<i32>().unwrap(),
                correlation_window: g2_matches.value_of("correlation_window").unwrap().parse::<f64>().unwrap(), //10e-9,
                resolution: g2_matches.value_of("resolution").unwrap().parse::<f64>().unwrap(),  //60e-12
                start_record: None,
                stop_record: None,
            };
            let g2_histogram = g2(&ptu_file, &params).unwrap();
            println!("{:?}", g2_histogram.hist);
            println!("{:?}", g2_histogram.t);
        },
        ("lifetime", Some(lifetime_matches)) => {
            let filename = PathBuf::from(lifetime_matches.value_of("input").unwrap());
            let ptu_file = File::PTU(PTUFile::new(filename).unwrap());
            let params = LifetimeParams {
                channel_sync: lifetime_matches.value_of("ch_sync").unwrap().parse::<i32>().unwrap(),
                channel_source: lifetime_matches.value_of("ch_source").unwrap().parse::<i32>().unwrap(), resolution: lifetime_matches.value_of("resolution").unwrap().parse::<f64>().unwrap(),  //60e-12
                start_record: None,
                stop_record: None,
            };
            let lifetime_histogram = lifetime(&ptu_file, &params).unwrap();
            println!("{:?}", lifetime_histogram.hist);
            println!("{:?}", lifetime_histogram.t);

        },
        (_, None) => println!("No subcommand was used"),
        _ => unreachable!(), // Assuming you've listed all direct children above, this is unreachable
    }
    //let filename = PathBuf::from("/Users/garfield/Downloads/20191205_Xminus_0p1Ve-6_CW_HBT.ptu");
    //let filename = PathBuf::from("/Users/garfield/Downloads/GUI_T3_10s.ptu");
    //let filename = PathBuf::from("/Users/garfield/Downloads/GUI_T2.ptu");
    //let File::PTU(f) = &ptu_file;
    //println!("{}", f);
}
