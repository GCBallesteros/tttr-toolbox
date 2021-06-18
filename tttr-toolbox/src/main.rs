use anyhow::Result;

use ndarray::arr1;
use ndarray_npy::NpzWriter;

use std;
use std::path::PathBuf;

extern crate clap;
extern crate tttr_toolbox_proc_macros;

use clap::{App, Arg, SubCommand};

use tttr_toolbox::headers::File;
use tttr_toolbox::parsers::ptu::PTUFile;
use tttr_toolbox::tttr_tools::g2::{g2, G2Params};
use tttr_toolbox::tttr_tools::g3::{g3, G3Params};
use tttr_toolbox::tttr_tools::synced_g3::{g3_sync, G3SyncParams};
use tttr_toolbox::tttr_tools::lifetime::{lifetime, LifetimeParams};
use tttr_toolbox::tttr_tools::timetrace::{timetrace, TimeTraceParams};

// ToDo
// 1. Check magic number for PTU
// 2. Documentation for g3 and g2 symmetrizing algorithm

pub fn main() -> Result<()> {
    let matches = App::new("TTTR Toolbox")
        .version("0.4")
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
                Arg::with_name("output")
                .short("o")
                .help("Output Numpy npz file path")
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
                Arg::with_name("output")
                .short("o")
                .help("Output Numpy npz file path")
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
                Arg::with_name("output")
                .short("o")
                .help("Output Numpy npz file path")
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
        .subcommand(
            SubCommand::with_name("g3")
            .about("Compute third order coincidences between two channels")
            .arg(
                Arg::with_name("input")
                .short("i")
                .help("Input file path")
                .takes_value(true)
                .required(true)
            )
            .arg(
                Arg::with_name("output")
                .short("o")
                .help("Output Numpy npz file path")
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
                Arg::with_name("channel3")
                .short("3")
                .help("Third channel")
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
                .help("Time resolution of the g3 histogram")
                .takes_value(true)
                .required(true)
            )
        )
        .subcommand(
            SubCommand::with_name("g3sync")
            .about("Compute third order coincidences between channels with a regular sync")
            .arg(
                Arg::with_name("input")
                .short("i")
                .help("Input file path")
                .takes_value(true)
                .required(true)
            )
            .arg(
                Arg::with_name("output")
                .short("o")
                .help("Output Numpy npz file path")
                .takes_value(true)
                .required(true)
            )
            .arg(
                Arg::with_name("channelS")
                .short("s")
                .help("First channel")
                .takes_value(true)
                .required(true)
            )
            .arg(
                Arg::with_name("channel1")
                .short("1")
                .help("Second channel")
                .takes_value(true)
                .required(true)
            )
            .arg(
                Arg::with_name("channel2")
                .short("2")
                .help("Third channel")
                .takes_value(true)
                .required(true)
            )
            .arg(
                Arg::with_name("resolution")
                .short("r")
                .help("Time resolution of the g3 histogram")
                .takes_value(true)
                .required(true)
            )
        )
        .get_matches();

    match matches.subcommand() {
        ("intensity", Some(intensity_matches)) => {
            let filename = PathBuf::from(intensity_matches.value_of("input").unwrap());
            let ptu_file = File::PTU(PTUFile::new(filename)?);
            let params = TimeTraceParams {
                resolution: intensity_matches
                    .value_of("resolution")
                    .unwrap()
                    .parse::<f64>()?,
                channel: intensity_matches
                    .value_of("channel")
                    .map(|x| x.parse::<i32>().unwrap()),
            };
            let tt = timetrace(&ptu_file, &params)?;

            let mut npz = NpzWriter::new(std::fs::File::create(
                intensity_matches.value_of("output").unwrap(),
            )?);
            npz.add_array("intensity", &arr1(&tt.intensity))?;
            npz.add_array("recnum_trace", &arr1(&tt.recnum_trace))?;
            npz.finish()?;
        }
        ("g2", Some(g2_matches)) => {
            let filename = PathBuf::from(g2_matches.value_of("input").unwrap());
            let ptu_file = File::PTU(PTUFile::new(filename)?);
            let params = G2Params {
                channel_1: g2_matches.value_of("channel1").unwrap().parse::<i32>()?,
                channel_2: g2_matches.value_of("channel2").unwrap().parse::<i32>()?,
                correlation_window: g2_matches
                    .value_of("correlation_window")
                    .unwrap()
                    .parse::<f64>()?,
                resolution: g2_matches.value_of("resolution").unwrap().parse::<f64>()?,
                record_ranges: None,
            };
            let g2_histogram = g2(&ptu_file, &params)?;

            let mut npz = NpzWriter::new(std::fs::File::create(
                g2_matches.value_of("output").unwrap(),
            )?);
            npz.add_array("histogram", &arr1(&g2_histogram.hist))?;
            npz.add_array("t", &arr1(&g2_histogram.t))?;
            npz.finish()?;
        }
        ("g3", Some(g3_matches)) => {
            let filename = PathBuf::from(g3_matches.value_of("input").unwrap());
            let ptu_file = File::PTU(PTUFile::new(filename)?);
            let params = G3Params {
                channel_1: g3_matches.value_of("channel1").unwrap().parse::<i32>()?,
                channel_2: g3_matches.value_of("channel2").unwrap().parse::<i32>()?,
                channel_3: g3_matches.value_of("channel3").unwrap().parse::<i32>()?,
                correlation_window: g3_matches
                    .value_of("correlation_window")
                    .unwrap()
                    .parse::<f64>()?,
                resolution: g3_matches.value_of("resolution").unwrap().parse::<f64>()?,
                start_record: None,
                stop_record: None,
            };
            let g3_histogram = g3(&ptu_file, &params).unwrap();

            let mut npz = NpzWriter::new(std::fs::File::create(
                g3_matches.value_of("output").unwrap(),
            )?);
            npz.add_array("histogram", &g3_histogram.hist)?;
            npz.add_array("t", &arr1(&g3_histogram.t))?;
            npz.finish()?;
        }
        ("g3sync", Some(g3_matches)) => {
            let filename = PathBuf::from(g3_matches.value_of("input").unwrap());
            let ptu_file = File::PTU(PTUFile::new(filename)?);
            let params = G3SyncParams {
                channel_sync: g3_matches.value_of("channelS").unwrap().parse::<i32>()?,
                channel_1: g3_matches.value_of("channel1").unwrap().parse::<i32>()?,
                channel_2: g3_matches.value_of("channel2").unwrap().parse::<i32>()?,
                resolution: g3_matches.value_of("resolution").unwrap().parse::<f64>()?,
                start_record: None,
                stop_record: None,
            };
            let g3_histogram = g3_sync(&ptu_file, &params).unwrap();

            let mut npz = NpzWriter::new(std::fs::File::create(
                g3_matches.value_of("output").unwrap(),
            )?);
            npz.add_array("histogram", &g3_histogram.hist)?;
            npz.add_array("t", &arr1(&g3_histogram.t))?;
            npz.finish()?;
        }
        ("lifetime", Some(lifetime_matches)) => {
            let filename = PathBuf::from(lifetime_matches.value_of("input").unwrap());
            let ptu_file = File::PTU(PTUFile::new(filename)?);
            let params = LifetimeParams {
                channel_sync: lifetime_matches
                    .value_of("ch_sync")
                    .unwrap()
                    .parse::<i32>()?,
                channel_source: lifetime_matches
                    .value_of("ch_source")
                    .unwrap()
                    .parse::<i32>()?,
                resolution: lifetime_matches
                    .value_of("resolution")
                    .unwrap()
                    .parse::<f64>()?,
                start_record: None,
                stop_record: None,
            };
            let lifetime_histogram = lifetime(&ptu_file, &params)?;

            let mut npz = NpzWriter::new(std::fs::File::create(
                lifetime_matches.value_of("output").unwrap(),
            )?);
            npz.add_array("histogram", &arr1(&lifetime_histogram.hist))?;
            npz.add_array("t", &arr1(&lifetime_histogram.t))?;
            npz.finish()?;
        }
        (_, None) => println!("No subcommand was used"),
        _ => unreachable!(), // Assuming you've listed all direct children above, this is unreachable
    };
    //let filename = PathBuf::from("/Users/garfield/Downloads/20191205_Xminus_0p1Ve-6_CW_HBT.ptu");
    //let filename = PathBuf::from("/Users/garfield/Downloads/GUI_T3_10s.ptu");
    //let filename = PathBuf::from("/Users/garfield/Downloads/GUI_T2.ptu");
    //let File::PTU(f) = &ptu_file;
    //println!("{}", f);
    Ok(())
}
