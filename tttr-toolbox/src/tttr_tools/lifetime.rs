use crate::errors::Error;
use crate::headers::{File, RecordType};
use crate::parsers::ptu;
//use crate::tttr_tools::circular_buffer::CircularBuffer;
use crate::{Click, TTTRFile, TTTRStream};
use std::fmt::Debug;

struct Lifetime<P: TTTRStream + Iterator> {
    pub click_stream: P,
    pub params: LifetimeParams,
    pub sync_period: u64,
}

/// Result from the lifetime algorithm
pub struct LifetimeResult {
    pub t: Vec<f64>,
    pub hist: Vec<u64>,
}

/// Parameters for the lifetime algorithm
///
/// # Parameters
///    - channel_sync: The number of the sync channel TCSPC
///    - channel_source: The number of the channel your source is connected into the TCSPC
///    - correlation_window: Length of the correlation window of interest in seconds. If
///      it is longer than the sync pulse period you will get a tail of zero counts.
///    - resolution: Resolution of the lifetime histogram in seconds
#[derive(Debug, Copy, Clone)]
pub struct LifetimeParams {
    pub channel_sync: i32,
    pub channel_source: i32,
    pub resolution: f64,
    pub start_record: Option<usize>,
    pub stop_record: Option<usize>,
}

impl<P: TTTRStream + Iterator> Lifetime<P> {
    fn compute(self) -> LifetimeResult
    where
        <P as Iterator>::Item: Debug + Click,
    {
        let real_resolution = self.params.resolution.clone();
        let correlation_window = (self.sync_period as f64) * 1e-12;

        let n_bins = (correlation_window / self.params.resolution) as u64;
        let resolution = (correlation_window * 1e12 / (n_bins as f64)) as u64;

        let mut histogram = vec![0; n_bins as usize];
        let mut tof_sync = 0;

        for rec in self.click_stream.into_iter() {
            let (tof, channel) = (*rec.tof(), *rec.channel());

            if channel == self.params.channel_source {
                let delta = tof - tof_sync;
                let hist_idx = ((delta % self.sync_period) / resolution) as usize;
                if hist_idx < (n_bins as usize) {histogram[hist_idx] += 1;};
                
            } else if channel == self.params.channel_sync {
                tof_sync = tof;
            }
        }

        let t = (0..n_bins)
            .map(|i| (i as f64) * real_resolution)
            .collect::<Vec<f64>>();
        LifetimeResult {
            t: t,
            hist: histogram,
        }
    }
}

/// Lifetime algorithm for photon emitters.
///
/// ## Parameters
///
/// The parameters to the algorithm are passed via a `LifetimeParams` struct that contains
/// the following:
///    - channel_sync: The number of the sync channel into the TCSPC,
///    - channel_source: The number of the source input channel into the TCSPC,
///    - correlation_window: Length of the correlation window of interest in seconds,
///    - resolution: Resolution of the lifetime histogram in seconds,
///
/// ## Algorithm description
/// A lifetime algorithm is in essence equivalent to the g2 algorithm where we set
/// one of the channels to be the sync input. Doing it this way is suboptimal. In first,
/// place we get multiple replicas of the "lifetime" measurement. This means that our
/// counts are distributed in time leading to either lower statics or a more complex
/// postprocessing. The other major problem is that it is computationally more complex
/// to compute a full g2 than the simplified algorithm we are running here.
///
/// ## Measurement requirements
/// The input to the sync channel must consist on a train of deltas that is synchronized
/// with the  excitation source. Typically pulsed lasers used for lifetime measurements include
/// an RF output for this purpose.
pub fn lifetime(f: &File, params: &LifetimeParams) -> Result<LifetimeResult, Error> {
    let start_record = params.start_record;
    let stop_record = params.stop_record;
    match f {
        File::PTU(x) => match x.record_type().unwrap() {
            RecordType::PHT2 => {
                Err(Error::NotImplemented(String::from("The lifetime algorithm is only supported in T3 mode")))
            }
            RecordType::HHT2_HH1 => {
                Err(Error::NotImplemented(String::from("The lifetime algorithm is only supported in T3 mode")))
            }
            RecordType::HHT2_HH2 => {
                Err(Error::NotImplemented(String::from("The lifetime algorithm is only supported in T3 mode")))
            }
            RecordType::HHT3_HH2 => {
                let stream = ptu::streamers::HHT3_HH2Stream::new(x, start_record, stop_record)?;
                let sync_period = stream.sync_period;
                let tt = Lifetime {
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
