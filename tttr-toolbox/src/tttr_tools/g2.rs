use crate::errors::Error;
use crate::headers::{File, RecordType};
use crate::parsers::ptu;
use crate::tttr_tools::circular_buffer::CircularBuffer;
use crate::{Click, TTTRFile, TTTRStream};
use std::fmt::Debug;

const MAX_BUFFER_SIZE: usize = 4096;

struct G2<P: TTTRStream + Iterator> {
    pub click_stream: P,
    pub params: G2Params,
}

/// Result from the g2 algorithm
pub struct G2Result {
    pub t: Vec<f64>,
    pub hist: Vec<u64>,
}

/// Parameters for the g2 algorithm
///
/// # Parameters
///    - channel_1: The number of the first input channel into the TCSPC
///    - channel_2: The number of the second input channel into the TCSPC
///    - correlation_window: Length of the correlation window of interest in seconds
///    - resolution: Resolution of the g2 histogram in seconds
#[derive(Debug, Copy, Clone)]
pub struct G2Params {
    pub channel_1: i32,
    pub channel_2: i32,
    pub correlation_window: f64,
    pub resolution: f64,
    pub start_record: Option<usize>,
    pub stop_record: Option<usize>,
}

impl<P: TTTRStream + Iterator> G2<P> {
    fn compute(self) -> G2Result
    where
        <P as Iterator>::Item: Debug + Click,
    {
        let real_resolution = self.params.resolution.clone();
        let n_bins = (self.params.correlation_window / self.params.resolution) as u64;
        let correlation_window =
            self.params.correlation_window / self.click_stream.time_resolution();

        let resolution = (correlation_window / (n_bins as f64)) as u64;
        let correlation_window = n_bins * resolution;
        let n_bins = n_bins * 2;

        let central_bin = n_bins / 2;
        let mut histogram = vec![0; n_bins as usize];

        let mut buff_1 = CircularBuffer::new(MAX_BUFFER_SIZE);
        let mut buff_2 = CircularBuffer::new(MAX_BUFFER_SIZE);

        // Substractions between u64 below are safe from over/underflows due to
        // algorithm invariants.
        //   1. `rec.tof` is always the most recent click on the detector.
        //   2. The `if` guard on `delta`.
        for rec in self.click_stream.into_iter() {
            let (tof, channel) = (*rec.tof(), *rec.channel());

            if channel == self.params.channel_1 {
                buff_1.push(tof);

                for click in buff_2.iter() {
                    let delta = tof - click;
                    if delta < correlation_window {
                        let hist_idx = central_bin - delta / resolution - 1;
                        histogram[hist_idx as usize] += 1;
                    } else {
                        break;
                    }
                }
            } else if channel == self.params.channel_2 {
                buff_2.push(tof);

                for click in buff_1.iter() {
                    let delta = tof - click;
                    if delta < correlation_window {
                        let hist_idx = central_bin + delta / resolution;
                        histogram[hist_idx as usize] += 1;
                    } else {
                        break;
                    }
                }
            }
        }
        let t = (0..n_bins)
            .map(|i| ((i as f64) - (central_bin as f64)) * real_resolution)
            .collect::<Vec<f64>>();
        G2Result {
            t: t,
            hist: histogram,
        }
    }
}

/// Computes the second order autocorrelation (g2) between two channels on a TCSPC module.
///
/// ## Parameters
///
/// The parameters to the algorithm are passed via a `G2Params` struct that contains
/// the following:
///    - channel_1: The number of the first input channel into the TCSPC,
///    - channel_2: The number of the second input channel into the TCSPC,
///    - correlation_window: Length of the correlation window of interest in seconds,
///    - resolution: Resolution of the g2 histogram in seconds,
///
/// ## Algorithm description
///
/// The streaming g2 algorithm measures the time difference between a
/// photon arriving at a channel and all the photons that came before it and arrived
/// at the other channel. A histogram of time differences is then built and is the output
/// we are after.
///
/// Computational constraints force us to define beforehand the maximum and minimum
/// time differences, the correlation window, that can fit into the histogram. Having a finite
/// value for the time difference we can store implies that we don't need to measure every single
/// photon arriving at channel A against every previous photon on channel B. Instead, only
/// a finite number of clicks into the past need to be considered.
///
/// Past clicks on each channel are pushed into circular buffers that keep the last N photons
/// that arrived at each of them. A circular buffer allows to always have time ordered arrival
/// times if we look from the head position of the buffer backwards.
///
/// ## Finite buffer artifacts
/// Not being capable to look back to all photons that came before can be a potential
/// source of artifacts on the calculated g2 histograms. At it's most extreme if N=1
/// we only look into the immediately previous photon. This introduces an exponential
/// decay artifact on the g2.
///
/// The maximum size of the correlation window that is artifact free is dependent
/// on the click rate. A quick estimate can be obtained by multiplying the inverse
/// of the click rate times the size of the buffer. E.g. for a buffer of 4096 photons
/// and a click rate of 10e6 Hz we get an artifcat free window of 0.4 milliseconds.
/// Taking into consideration typical emitter lifetimes and collection optics efficiency
/// this should be more than enough to capture any relevant dynamics. If this is
/// not the case for you will need to modify the hard coded maximum buffer size
/// defined on `src/tttr_tools/g2.rs`.
pub fn g2(f: &File, params: &G2Params) -> Result<G2Result, Error> {
    let start_record = params.start_record;
    let stop_record = params.stop_record;
    match f {
        File::PTU(x) => match x.record_type().unwrap() {
            RecordType::PHT2 => {
                let stream = ptu::streamers::PHT2Stream::new(x, start_record, stop_record)?;
                let tt = G2 {
                    click_stream: stream,
                    params: *params,
                };
                Ok(tt.compute())
            }
            RecordType::HHT2_HH1 => {
                let stream = ptu::streamers::HHT2_HH1Stream::new(x, start_record, stop_record)?;
                let tt = G2 {
                    click_stream: stream,
                    params: *params,
                };
                Ok(tt.compute())
            }
            RecordType::HHT2_HH2 => {
                let stream = ptu::streamers::HHT2_HH2Stream::new(x, start_record, stop_record)?;
                let tt = G2 {
                    click_stream: stream,
                    params: *params,
                };
                Ok(tt.compute())
            }
            RecordType::NotImplemented => panic! {"Record type not implemented"},
        },
    }
}
