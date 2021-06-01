use crate::errors::Error;
use crate::headers::{File, RecordType};
use crate::parsers::ptu;
use crate::tttr_tools::colored_circular_buffer::CCircularBuffer;
use crate::{Click, TTTRFile, TTTRStream};
use std::fmt::Debug;

use ndarray::Array2;

const MAX_BUFFER_SIZE: usize = 4096;

struct G3<P: TTTRStream + Iterator> {
    pub click_stream: P,
    pub params: G3Params,
}

/// Result from the g3 algorithm
pub struct G3Result {
    pub t: Vec<f64>,
    pub hist: Array2<u64>,
}

/// Parameters for the g3 algorithm
///
/// # Parameters
///    - channel_1: The number of the first input channel into the TCSPC
///    - channel_2: The number of the second input channel into the TCSPC
///    - channel_3: The number of the third input channel into the TCSPC
///    - correlation_window: Length of the correlation window of interest in seconds
///    - resolution: Resolution of the g3 histogram in seconds
#[derive(Debug, Copy, Clone)]
pub struct G3Params {
    pub channel_1: i32,
    pub channel_2: i32,
    pub channel_3: i32,
    pub correlation_window: f64,
    pub resolution: f64,
    pub start_record: Option<usize>,
    pub stop_record: Option<usize>,
}

impl<P: TTTRStream + Iterator> G3<P> {
    fn compute(self) -> G3Result
    where
        <P as Iterator>::Item: Debug + Click,
    {
        let real_resolution = self.params.resolution.clone();
        let n_bins = (self.params.correlation_window / self.params.resolution) as u64;
        let correlation_window =
            self.params.correlation_window / (self.click_stream.time_resolution());

        let resolution = (correlation_window / (n_bins as f64)) as u64;
        let correlation_window = n_bins * resolution;
        let n_bins = n_bins * 2;

        let central_bin = n_bins / 2;
        let mut histogram = Array2::<u64>::zeros((n_bins as usize, n_bins as usize));

        let mut click_buffer = CCircularBuffer::new(MAX_BUFFER_SIZE);

        let relevant_channels: Vec<i32> =
            vec![self.params.channel_1, self.params.channel_2, self.params.channel_3];

        for click_1 in self.click_stream.into_iter() {
            let (&tof1, &chn1) = (click_1.tof(), click_1.channel());
            if !relevant_channels.contains(&chn1) {
                continue;
            }

            for click_2 in click_buffer.iter() {
                let &(tof2, chn2) = click_2;
                let delta12 = tof1 - tof2;
                if delta12 > correlation_window {
                    break;
                }

                for click_3 in click_buffer.iter() {
                    let &(tof3, chn3) = click_3;
                    // time ordering is broken here because we are going
                    // through the same click buffer
                    if tof3 >= tof2 {
                        continue;
                    }
                    let delta23 = tof2 - tof3;
                    let delta13 = tof1 - tof3;

                    // The if nesting happens in inverse order to the photon arrival
                    // times. The reason is that older photons (deeper in the nesting)
                    // happened before.
                    //
                    // tau_1 is defined as the delay registered between clicks on the
                    // channel we designate as ch1 and ch2. tau_2 is defined as the
                    // delay registed between clicks on the channel we designate as ch1
                    // and ch3.
                    //
                    // The nomenclature for the deltas (deltaXY) references the delay
                    // between click X and click Y within these nested loops. It is not
                    // the delay between channel X and Y. We only know what channels does
                    // clicks correspond once we are inside the 3IFs. For example the first
                    // nested IFs below correspond to an arrival of photons at channels
                    // 3 -> 2 -> 1. Since the last photon to be registed is the one at ch3
                    // it corresponds to tof3. Therefore delta13 corresponds to delay
                    // between the most recent click at `tof1` (ch1 here) and the click on
                    // ch3. That is tau2. Graphically, if we say channel_1 = 1, channel_2 =2
                    // and channel_3 = 3.
                    //
                    //     tau2; ch3 before ch1 => tau2 < 0
                    //  ┌─────────┐
                    //  ▼         ▼
                    //  3 -> 2 -> 1
                    //  ▲    ▲    ▲
                    //  │    │    │
                    // tof3 tof2 tof1
                    //       ▲    ▲
                    //       └────┘
                    //        tau1; ch2 before ch1 => tau1 < 0
                    //
                    // Another example is below
                    if chn1 == self.params.channel_1 {
                        if chn2 == self.params.channel_2 {
                            if chn3 == self.params.channel_3 {
                                // (321) tau_1 < 0, tau_2 < 0
                                let tau1 = delta12;
                                let tau2 = delta13;
                                if tau1 < correlation_window && tau2 < correlation_window {
                                    let idx1 = central_bin - tau1 / resolution - 1;
                                    let idx2 = central_bin - tau2 / resolution - 1;
                                    histogram[[idx1 as usize, idx2 as usize]] += 1;
                                } else {
                                    break;
                                }
                            }
                        } 
                        else if chn2 == self.params.channel_3 {
                            if chn3 == self.params.channel_2 {
                                // (231) tau_1 < 0, tau_2 < 0
                                let tau1 = delta13;
                                let tau2 = delta12;
                                if tau1 < correlation_window && tau2 < correlation_window {
                                    let idx1 = central_bin - tau1 / resolution - 1;
                                    let idx2 = central_bin - tau2 / resolution - 1;
                                    histogram[[idx1 as usize, idx2 as usize]] += 1;
                                } else {
                                    break;
                                }
                            }
                        }
                    } else if chn1 == self.params.channel_2 {
                        if chn2 == self.params.channel_1 {
                            if chn3 == self.params.channel_3 {
                                // (312) tau_1 > 0, tau_2 < 0
                                //        tau1; ch1 before ch2 => tau1 > 0
                                //       ┌────┐
                                //       ▼    ▼
                                //  3 -> 1 -> 2
                                //  ▲    ▲    ▲
                                //  │    │    │
                                // tof3 tof2 tof1
                                //   ▲    ▲
                                //   └────┘
                                //    tau2; ch3 before ch1 => tau2 < 0
                                let tau1 = delta12;
                                let tau2 = delta23;
                                if tau1 < correlation_window && tau2 < correlation_window {
                                    let idx1 = central_bin + tau1 / resolution;
                                    let idx2 = central_bin - tau2 / resolution - 1;
                                    histogram[[idx1 as usize, idx2 as usize]] += 1;
                                } else {
                                    break;
                                }
                            }
                        } else if chn2 == self.params.channel_3 {
                            if chn3 == self.params.channel_1 {
                                // (132) tau_1 > 0, tau_2 > 0
                                let tau1 = delta13;
                                let tau2 = delta23;
                                if tau1 < correlation_window && tau2 < correlation_window {
                                    let idx1 = central_bin + tau1 / resolution;
                                    let idx2 = central_bin + tau2 / resolution;
                                    histogram[[idx1 as usize, idx2 as usize]] += 1;
                                } else {
                                    break;
                                }
                            }
                        }
                    }
                    else if chn1 == self.params.channel_3 {
                        if chn2 == self.params.channel_1 {
                            if chn3 == self.params.channel_2 {
                                // (213) tau_1 < 0, tau_2 > 0
                                let tau1 = delta23;
                                let tau2 = delta12;
                                if tau1 < correlation_window && tau2 < correlation_window {
                                    let idx1 = central_bin - tau1 / resolution - 1;
                                    let idx2 = central_bin + tau2 / resolution;
                                    histogram[[idx1 as usize, idx2 as usize]] += 1;
                                } else {
                                    break;
                                }
                            }
                        } else if chn2 == self.params.channel_2 {
                            if chn3 == self.params.channel_1 {
                                // (123) tau_1 > 0, tau_2 > 0
                                let tau1 = delta23;
                                let tau2 = delta13;
                                if tau1 < correlation_window && tau2 < correlation_window {
                                    let idx1 = central_bin + tau1 / resolution;
                                    let idx2 = central_bin + tau2 / resolution;
                                    histogram[[idx1 as usize, idx2 as usize]] += 1;
                                } else {
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
            .map(|i| ((i as f64) - (central_bin as f64)) * real_resolution)
            .collect::<Vec<f64>>();
        G3Result {
            t: t,
            hist: histogram,
        }
    }
}

/// Computes the second order autocorrelation (g3) between two channels on a TCSPC module.
/// $tau_1$ and $tau_2$ are measured relative to the channel we designate as channel 1.
///
/// ## Parameters
///
/// The parameters to the algorithm are passed via a `G3Params` struct that contains
/// the following:
///    - channel_1: The number of the first input channel into the TCSPC,
///    - channel_2: The number of the second input channel into the TCSPC,
///    - channel_3: The number of the third input channel into the TCSPC,
///    - correlation_window: Length of the correlation window of interest in seconds,
///    - resolution: Resolution of the g3 histogram in seconds,
///
/// ## Return
/// A square matrix with the (center_idx, center_idx) index being the (t1=0, t2=0) delays
/// grow down and to the right. First index is tau1 and second index is tau2.
///
/// ## g^3: Third Order Coincidences
///
/// The g3 algorithm is a generalization of the g2 algorithm to third order coincidences.
/// This tra
///
/// <img src="https://raw.githubusercontent.com/GCBallesteros/tttr-toolbox/master/images/g3_orderings" alt="third order click orderings" >
///
/// Past clicks on each channel are pushed into circular buffers that keep the last N photons
/// that arrived at each of them. A circular buffer allows to always have time ordered arrival
/// times if we look from the head position of the buffer backwards.
///
/// ## Finite buffer artifacts
/// As with the g2 algorithm, the size of the buffers to store past clicks will determine
/// the importance and the point at which artifacts appear on the histogram. The same
/// consideration apply. See the [second order autocorrelation documentation](tttr_tools/g2/fn.g2.html).
///
pub fn g3(f: &File, params: &G3Params) -> Result<G3Result, Error> {
    let start_record = params.start_record;
    let stop_record = params.stop_record;
    match f {
        File::PTU(x) => match x.record_type().unwrap() {
            RecordType::PHT2 => {
                let stream = ptu::streamers::PHT2Stream::new(x, start_record, stop_record)?;
                let tt = G3 {
                    click_stream: stream,
                    params: *params,
                };
                Ok(tt.compute())
            }
            RecordType::HHT2_HH1 => {
                let stream = ptu::streamers::HHT2_HH1Stream::new(x, start_record, stop_record)?;
                let tt = G3 {
                    click_stream: stream,
                    params: *params,
                };
                Ok(tt.compute())
            }
            RecordType::HHT2_HH2 => {
                let stream = ptu::streamers::HHT2_HH2Stream::new(x, start_record, stop_record)?;
                let tt = G3 {
                    click_stream: stream,
                    params: *params,
                };
                Ok(tt.compute())
            }
            RecordType::HHT3_HH2 => {
                let stream = ptu::streamers::HHT3_HH2Stream::new(x, start_record, stop_record)?;
                let tt = G3 {
                    click_stream: stream,
                    params: *params,
                };
                Ok(tt.compute())
            }
            RecordType::NotImplemented => panic! {"Record type not implemented"},
        },
    }
}
