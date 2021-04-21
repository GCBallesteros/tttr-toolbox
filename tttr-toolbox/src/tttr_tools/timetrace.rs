use crate::errors::Error;
use crate::headers::{File, RecordType};
use crate::parsers::ptu;
use crate::{Click, TTTRFile, TTTRStream};
use std::fmt::Debug;

struct TimeTrace<P: TTTRStream + Iterator> {
    pub click_stream: P,
    pub params: TimeTraceParams,
}

/// Results for the timetrace algorithm
///
/// It stores both the intensity trace and the record number of the last click for
/// each bin in the intensity trace. This makes is possible to implement algorithms
/// like photon post-selection.
pub struct TimeTraceResult {
    pub intensity: Vec<u64>,
    pub recnum_trace: Vec<u64>,
}

/// Parameters for the timetrace algorithm
///
/// ## Parameters
///   1. resolution: The resolution in seconds of the intensity time trace.
///   2. channel: Optional channel we want to monitor. If None is passed then all
///      all channels are summed together.
#[derive(Debug, Copy, Clone)]
pub struct TimeTraceParams {
    pub resolution: f64,
    pub channel: Option<i32>,
}

impl<P: TTTRStream + Iterator> TimeTrace<P> {
    fn compute(self) -> TimeTraceResult
    where
        <P as Iterator>::Item: Debug + Click,
    {
        let blips_per_bin = (self.params.resolution / self.click_stream.time_resolution()) as u64;
        let mut trace: Vec<u64> = vec![];
        let mut recnum_trace: Vec<u64> = vec![];

        let mut counter = 0;
        let mut end_of_bin = blips_per_bin;

        for (idx, rec) in self.click_stream.into_iter().enumerate() {
            if let Some(ch) = self.params.channel {
                counter += if *rec.channel() == ch { 1 } else { 0 }
            } else {
                counter += if *rec.channel() >= 0 { 1 } else { 0 };
            };

            if *rec.tof() > end_of_bin {
                trace.push(counter);
                recnum_trace.push(idx as u64);
                counter = 0;
                end_of_bin += blips_per_bin;
            };
        }
        TimeTraceResult {
            intensity: trace,
            recnum_trace: recnum_trace,
        }
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
pub fn timetrace(f: &File, params: &TimeTraceParams) -> Result<TimeTraceResult, Error> {
    let start_record = None;
    let stop_record = None;
    match f {
        File::PTU(x) => match x.record_type().unwrap() {
            RecordType::PHT2 => {
                let stream = ptu::streamers::PHT2Stream::new(x, start_record, stop_record)?;
                let tt = TimeTrace {
                    click_stream: stream,
                    params: *params,
                };
                Ok(tt.compute())
            }
            RecordType::HHT2_HH1 => {
                let stream = ptu::streamers::HHT2_HH1Stream::new(x, start_record, stop_record)?;
                let tt = TimeTrace {
                    click_stream: stream,
                    params: *params,
                };
                Ok(tt.compute())
            }
            RecordType::HHT2_HH2 => {
                let stream = ptu::streamers::HHT2_HH2Stream::new(x, start_record, stop_record)?;
                let tt = TimeTrace {
                    click_stream: stream,
                    params: *params,
                };
                Ok(tt.compute())
            }
            RecordType::NotImplemented => panic! {"Record type not implemented"},
        },
    }
}
