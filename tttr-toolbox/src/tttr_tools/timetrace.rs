use crate::{TTTRStream, TTTRFile, Click};
use crate::headers::{RecordType, File};
use crate::errors::Error;
use crate::parsers::ptu;
use std::fmt::Debug;

use dict_derive::{FromPyObject, IntoPyObject};

struct TimeTrace<P: TTTRStream + Iterator> {
    pub click_stream: P,
    pub params: TimeTraceParams,
}

/// Parameters for the timetrace algorithm
///
/// ## Parameters
///   1. resolution: The resolution in seconds of the intensity time trace.
///   2. channel: Optional channel we want to monitor. If None is passed then all
///      all channels are summed together.
#[derive(Debug, Copy, Clone, FromPyObject, IntoPyObject)]
pub struct TimeTraceParams {
    pub resolution: f64,
    pub channel: Option<i32>,
}

impl<P: TTTRStream + Iterator> TimeTrace<P> {
    fn compute(self) -> Vec<u64> where <P as Iterator>::Item: Debug + Click {
        let blips_per_bin = (self.params.resolution  / self.click_stream.time_resolution()) as u64;
        let mut trace: Vec<u64> = vec![];

        let mut counter = 0;
        let mut end_of_bin = blips_per_bin;

        for rec in self.click_stream.into_iter() {
            if *rec.tof() < end_of_bin {
                if let Some(ch) = self.params.channel {
                    counter += if *rec.channel() == ch {1} else {0}
                } else {
                    counter += if *rec.channel() >= 0 {1} else {0};
                };
            } else {
                trace.push(counter);
                counter = 0;
                end_of_bin += blips_per_bin;
            };
        };
        trace
    }
}

/// Calculate the intensity timetrace of clicks on a TCSPC.
///
/// The intensity is computed by discretizing the duration of the experiment into
/// intervals of fixed duration and counting how many clicks occur on each of them. The
/// intensity during each interval is then the number of clicks divided by the length
/// of the interval.
///
///
/// ## Resolution/Variance tradeoff
/// Reducing the resolution value (finer discretization in time) makes it possible to
/// look at intensity dynamics on a finer timescale. This may be of interest if we are
/// for example studying blinking dynamics or resonance shifts. However, there is a
/// limit to how fine the time resolution can be. Finer resolutions lead to smaller numbers
/// of clicks per interval and therefore the relative error for the number of counts
/// grows as we make intervals finer.
pub fn timetrace(f: &File, params: &TimeTraceParams) -> Result<Vec<u64>, Error> {
    //let params = TimeTraceParams {resolution: 10, channel: Some(0)};
    match f {
        File::PTU(x) => {
            match x.record_type().unwrap() {
                RecordType::PHT2 => {
                    let stream = ptu::streamers::PHT2Stream::new(x)?;
                    let tt = TimeTrace {click_stream: stream, params: *params};
                    Ok(tt.compute())
                },
                RecordType::NotImplemented => panic!{"Record type not implemented"},
                _ => panic!{"Record type not implemented"},
            }
        },
    }
}

