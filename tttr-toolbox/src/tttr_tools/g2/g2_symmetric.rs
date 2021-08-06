use crate::{
    errors::Error,
    headers::{File, RecordType},
    parsers::ptu,
    tttr_tools::{
        circular_buffer::CircularBuffer,
        g2::{G2Params, G2Result},
    },
    Click, TTTRFile, TTTRStream,
};
use std::fmt::Debug;

const MAX_BUFFER_SIZE: usize = 4096;

// ToDo
// Streamer params and G2Params should probably be different here

struct G2 {
    central_bin: u64,
    n_bins: u64,
    resolution: u64,
    correlation_window: u64,
    real_resolution: f64,
    channel_1: i32,
    channel_2: i32,
}

impl G2 {
    fn init(params: &G2Params, time_resolution: f64) -> Self {
        let real_resolution = params.resolution.clone();
        let n_bins = (params.correlation_window / params.resolution) as u64;
        let correlation_window = params.correlation_window / time_resolution;

        let resolution = (correlation_window / (n_bins as f64)) as u64;
        let correlation_window = n_bins * resolution;
        let n_bins = n_bins * 2;

        let central_bin = n_bins / 2;

        Self {
            central_bin,
            n_bins,
            resolution,
            correlation_window,
            real_resolution,
            channel_1: params.channel_1,
            channel_2: params.channel_2,
        }
    }

    fn compute<P: TTTRStream + Iterator>(
        &self,
        streamer: P,
        out_hist: &mut [u64],
        out_t: &mut [f64],
    ) where
        <P as Iterator>::Item: Debug + Click,
    {
        let mut buff_1 = CircularBuffer::new(MAX_BUFFER_SIZE);
        let mut buff_2 = CircularBuffer::new(MAX_BUFFER_SIZE);

        // Substractions between u64 below are safe from over/underflows due to
        // algorithm invariants.
        //   1. `rec.tof` is always the most recent click on the detector.
        //   2. The `if` guard on `delta`.
        for rec in streamer.into_iter() {
            let (tof, channel) = (*rec.tof(), *rec.channel());

            if channel == self.channel_1 {
                buff_1.push(tof);

                for click in buff_2.iter() {
                    let delta = tof - click;
                    if delta < self.correlation_window {
                        let hist_idx = self.central_bin - delta / self.resolution - 1;
                        out_hist[hist_idx as usize] += 1;
                    } else {
                        break;
                    }
                }
            } else if channel == self.channel_2 {
                buff_2.push(tof);

                for click in buff_1.iter() {
                    let delta = tof - click;
                    if delta < self.correlation_window {
                        let hist_idx = self.central_bin + delta / self.resolution;
                        out_hist[hist_idx as usize] += 1;
                    } else {
                        break;
                    }
                }
            }
        }

        for i in 0..self.n_bins {
            out_t[i as usize] = ((i as f64) - (self.central_bin as f64)) * self.real_resolution
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
/// <img src="https://raw.githubusercontent.com/GCBallesteros/tttr-toolbox/master/images/g2_orderings" alt="second order click orderings" >
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
pub(super) fn g2(f: &File, params: &G2Params) -> Result<G2Result, Error> {
    match f {
        File::PTU(x) => match x.record_type().unwrap() {
            RecordType::PHT2 => {
                let tt = G2::init(params, x.time_resolution()?);
                let mut g2_histogram = vec![0; tt.n_bins as usize];
                let mut t_histogram = vec![0.0; tt.n_bins as usize];

                if let Some(record_ranges) = &params.record_ranges {
                    for &(start_record, stop_record) in record_ranges {
                        let stream = ptu::streamers::PHT2Stream::new(
                            x,
                            Some(start_record),
                            Some(stop_record),
                        )?;
                        tt.compute(stream, &mut g2_histogram, &mut t_histogram);
                    }
                } else {
                    let stream = ptu::streamers::PHT2Stream::new(x, None, None)?;
                    tt.compute(stream, &mut g2_histogram, &mut t_histogram);
                };
                Ok(G2Result {
                    hist: g2_histogram,
                    t: t_histogram,
                })
            }
            RecordType::HHT2_HH1 => {
                let tt = G2::init(params, x.time_resolution()?);
                let mut g2_histogram = vec![0; tt.n_bins as usize];
                let mut t_histogram = vec![0.0; tt.n_bins as usize];

                if let Some(record_ranges) = &params.record_ranges {
                    for &(start_record, stop_record) in record_ranges {
                        let stream = ptu::streamers::HHT2_HH1Stream::new(
                            x,
                            Some(start_record),
                            Some(stop_record),
                        )?;
                        tt.compute(stream, &mut g2_histogram, &mut t_histogram);
                    }
                } else {
                    let stream = ptu::streamers::HHT2_HH1Stream::new(x, None, None)?;
                    tt.compute(stream, &mut g2_histogram, &mut t_histogram);
                };
                Ok(G2Result {
                    hist: g2_histogram,
                    t: t_histogram,
                })
            }
            RecordType::HHT2_HH2 => {
                let tt = G2::init(params, x.time_resolution()?);
                let mut g2_histogram = vec![0; tt.n_bins as usize];
                let mut t_histogram = vec![0.0; tt.n_bins as usize];

                if let Some(record_ranges) = &params.record_ranges {
                    for &(start_record, stop_record) in record_ranges {
                        let stream = ptu::streamers::HHT2_HH2Stream::new(
                            x,
                            Some(start_record),
                            Some(stop_record),
                        )?;
                        tt.compute(stream, &mut g2_histogram, &mut t_histogram);
                    }
                } else {
                    let stream = ptu::streamers::HHT2_HH2Stream::new(x, None, None)?;
                    tt.compute(stream, &mut g2_histogram, &mut t_histogram);
                };
                Ok(G2Result {
                    hist: g2_histogram,
                    t: t_histogram,
                })
            }
            RecordType::HHT3_HH2 => {
                let tt = G2::init(params, 1e-12);
                let mut g2_histogram = vec![0; tt.n_bins as usize];
                let mut t_histogram = vec![0.0; tt.n_bins as usize];

                if let Some(record_ranges) = &params.record_ranges {
                    for &(start_record, stop_record) in record_ranges {
                        let stream = ptu::streamers::HHT3_HH2Stream::new(
                            x,
                            Some(start_record),
                            Some(stop_record),
                        )?;
                        tt.compute(stream, &mut g2_histogram, &mut t_histogram);
                    }
                } else {
                    let stream = ptu::streamers::HHT3_HH2Stream::new(x, None, None)?;
                    tt.compute(stream, &mut g2_histogram, &mut t_histogram);
                };
                Ok(G2Result {
                    hist: g2_histogram,
                    t: t_histogram,
                })
            }
            RecordType::NotImplemented => panic! {"Record type not implemented"},
        },
    }
}
