use crate::errors::Error;
use crate::headers::{File, RecordType};
use crate::parsers::ptu;
use crate::tttr_tools::colored_circular_buffer::CCircularBuffer;
use crate::{Click, TTTRFile, TTTRStream};
use std::fmt::Debug;

use ndarray::Array2;

const MAX_BUFFER_SIZE: usize = 4096;

struct G3Sync<P: TTTRStream + Iterator> {
    pub click_stream: P,
    pub params: G3SyncParams,
    pub sync_period: u64,
}

/// Result from the g3 synced algorithm
pub struct G3SyncResult {
    pub t: Vec<f64>,
    pub hist: Array2<u64>,
}

/// Parameters for the synced g3 algorithm
///
/// # Parameters
///    - channel_sync: The number of the first input channel into the TCSPC
///    - channel_1: The number of the second input channel into the TCSPC
///    - channel_2: The number of the third input channel into the TCSPC
///    - resolution: Resolution of the g3 histogram in seconds
#[derive(Debug, Copy, Clone)]
pub struct G3SyncParams {
    pub channel_sync: i32,
    pub channel_1: i32,
    pub channel_2: i32,
    pub resolution: f64,
    pub start_record: Option<usize>,
    pub stop_record: Option<usize>,
}

impl<P: TTTRStream + Iterator> G3Sync<P> {
    fn compute(self) -> G3SyncResult
    where
        <P as Iterator>::Item: Debug + Click,
    {
        let real_resolution = self.params.resolution.clone();
        let correlation_window = (self.sync_period as f64) * 1e-12;

        let n_bins = (correlation_window / self.params.resolution) as u64;
        let resolution = self.sync_period / n_bins as u64;

        let mut histogram = Array2::<u64>::zeros((n_bins as usize, n_bins as usize));

        let mut click_buffer = CCircularBuffer::new(MAX_BUFFER_SIZE);

        let relevant_channels: Vec<i32> =
            vec![self.params.channel_sync, self.params.channel_1, self.params.channel_2];

        for click_1 in self.click_stream.into_iter() {
            let (&tof1, &chn1) = (click_1.tof(), click_1.channel());
            if !relevant_channels.contains(&chn1) {
                continue;
            }

            for click_2 in click_buffer.iter() {
                let &(tof2, chn2) = click_2;
                let delta12 = tof1 - tof2;
                if delta12 > self.sync_period {
                    break;
                }

                for click_3 in click_buffer.iter() {
                    let &(tof3, chn3) = click_3;
                    // time ordering is broken here because we are going
                    // through the same click buffer
                    if tof3 >= tof2 {
                        continue;
                    }
                    let delta13 = tof1 - tof3;
                    let delta23 = tof2 - tof3;

                    if chn1 == self.params.channel_1 {
                        if chn2 == self.params.channel_2 {
                            if chn3 == self.params.channel_sync {
                                // sync -> 2 -> 1
                                let tau1 = delta13;
                                let tau2 = delta23;

                                let idx1 = (tau1 % self.sync_period) / resolution;
                                let idx2 = (tau2 % self.sync_period) / resolution;
                                if idx1 < n_bins && idx2 < n_bins {
                                    histogram[[idx1 as usize, idx2 as usize]] += 1;
                                    break;
                                }
                            }
                        }
                    } else if chn1 == self.params.channel_2 {
                        if chn2 == self.params.channel_1 {
                            if chn3 == self.params.channel_sync {
                                // sync -> 1 -> 2
                                let tau1 = delta23;
                                let tau2 = delta13;

                                let idx1 = (tau1 % self.sync_period) / resolution;
                                let idx2 = (tau2 % self.sync_period) / resolution;
                                if idx1 < n_bins && idx2 < n_bins {
                                    histogram[[idx1 as usize, idx2 as usize]] += 1;
                                    break;
                                }
                            }
                        }
                    }

                }
            }

            // finish by adding the most recent click to the buffer
            click_buffer.push(tof1, chn1);
        }

        // Since we are using a square correlation window we only need one variable
        // to store the bin centers.
        let t = (0..n_bins)
            .map(|i| (i as f64) * real_resolution)
            .collect::<Vec<f64>>();
        G3SyncResult {
            t: t,
            hist: histogram,
        }
    }
}

/// Computes the third order autocorrelation (g3) of two channels relative
/// the a periodic sync channel.
///
/// ## Parameters
///
/// The parameters to the algorithm are passed via a `G3Params` struct that contains
/// the following:
///    - channel_sync: The number of the first input channel into the TCSPC,
///    - channel_1: The number of the second input channel into the TCSPC,
///    - channel_2: The number of the third input channel into the TCSPC,
///    - correlation_window: Length of the correlation window of interest in seconds,
///    - resolution: Resolution of the g3 histogram in seconds,
///
/// ## Return
/// A square matrix with the (0, 0) index being the (t1=0, t2=0) delays grow down and
/// to the right. First index is tau1 and second index is tau2.
pub fn g3_sync(f: &File, params: &G3SyncParams) -> Result<G3SyncResult, Error> {
    let start_record = params.start_record;
    let stop_record = params.stop_record;
    match f {
        File::PTU(x) => match x.record_type().unwrap() {
            RecordType::PHT2 => Err(Error::NotImplemented(String::from(
                "The synced g3 algorithm is only supported in T3 mode",
            ))),
            RecordType::HHT2_HH1 => Err(Error::NotImplemented(String::from(
                "The synced algorithm is only supported in T3 mode",
            ))),
            RecordType::HHT2_HH2 => Err(Error::NotImplemented(String::from(
                "The synced algorithm is only supported in T3 mode",
            ))),
            RecordType::HHT3_HH2 => {
                let stream = ptu::streamers::HHT3_HH2Stream::new(x, start_record, stop_record)?;
                let sync_period = stream.sync_period;
                let tt = G3Sync {
                    click_stream: stream,
                    params: *params,
                    sync_period,
                };
                Ok(tt.compute())
            }
            RecordType::NotImplemented => panic! {"Record type not implemented"},
        },
    }
}
