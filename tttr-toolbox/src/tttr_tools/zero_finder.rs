use crate::{TTTRStream, TTTRFile, Click};
use crate::headers::{RecordType, File};
use crate::errors::Error;
use crate::parsers::ptu;
use std::fmt::Debug;


struct ZeroFinder<P: TTTRStream + Iterator> {
    pub click_stream: P,
    pub params: ZeroFinderParams,
}

/// Result from the zero finder algorithm
pub struct ZeroFinderResult {
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
pub struct ZeroFinderParams {
    pub channel_1: i32,
    pub channel_2: i32,
    pub correlation_window: f64,
    pub resolution: f64,
}

impl<P: TTTRStream + Iterator> ZeroFinder<P> {
    fn compute(self) -> ZeroFinderResult where <P as Iterator>::Item: Debug + Click {
        let real_resolution = self.params.resolution.clone();
        let n_bins = (self.params.correlation_window / real_resolution) as u64;
        let correlation_window = self.params.correlation_window / self.click_stream.time_resolution();

        let resolution = (correlation_window / (n_bins as f64)) as u64;
        let correlation_window = n_bins * resolution;
        let n_bins = n_bins*2;

        let central_bin = n_bins / 2;
        let mut histogram = vec![0; n_bins as usize];

        // Substractions between u64 below are safe from over/underflows due to
        // algorithm invariants.
        //   1. `rec.tof` is always the most recent click on the detector.
        //   2. The `if` guard on `delta`.
        let mut prev_tof_channel_1 = 0;
        let mut prev_tof_channel_2 = 0;

        for rec  in self.click_stream.into_iter() {
            let (tof, channel) = (*rec.tof(), *rec.channel());

            if channel == self.params.channel_1 {
                prev_tof_channel_1 = tof;

                let delta = tof - prev_tof_channel_2;
                if delta < correlation_window {
                    let hist_idx = central_bin - delta / resolution - 1;
                    histogram[hist_idx as usize] += 1;
                }
            } else if channel == self.params.channel_2 {
                prev_tof_channel_2 = tof;

                let delta = tof - prev_tof_channel_1;
                if delta < correlation_window {
                    let hist_idx = central_bin + delta / resolution;
                    histogram[hist_idx as usize] +=  1;
                }
            }
        };
        let t = (0..n_bins)
            .map(|i| ((i as f64) - (central_bin as f64)) * real_resolution)
            .collect::<Vec<f64>>();
        ZeroFinderResult { t: t, hist: histogram}
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
/// See Finite Buffer Artifacts in the documentation for g2.
pub fn zerofinder(f: &File, params: &ZeroFinderParams) -> Result<ZeroFinderResult, Error> {
    let start_record = None;
    let stop_record = None;
    match f {
        File::PTU(x) => {
            match x.record_type().unwrap() {
                RecordType::PHT2 => {
                    let stream = ptu::streamers::PHT2Stream::new(x, start_record, stop_record)?;
                    let tt = ZeroFinder {click_stream: stream, params: *params};
                    Ok(tt.compute())
                },
                RecordType::HHT2_HH1 => {
                    let stream = ptu::streamers::HHT2_HH1Stream::new(x, start_record, stop_record)?;
                    let tt = ZeroFinder {click_stream: stream, params: *params};
                    Ok(tt.compute())
                }
                RecordType::HHT2_HH2 => {
                    let stream = ptu::streamers::HHT2_HH2Stream::new(x, start_record, stop_record)?;
                    let tt = ZeroFinder {click_stream: stream, params: *params};
                    Ok(tt.compute())
                }
                RecordType::NotImplemented => panic!{"Record type not implemented"},
            }
        },
    }
}

