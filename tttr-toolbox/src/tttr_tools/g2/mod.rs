use crate::errors::Error;
use crate::headers::File;

pub mod g2_asymmetric;
pub mod g2_symmetric;

#[derive(Debug, Copy, Clone)]
pub enum G2Mode {
    Asymmetric,
    Symmetric,
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
#[derive(Debug, Clone)]
pub struct G2Params {
    pub channel_1: i32,
    pub channel_2: i32,
    pub correlation_window: f64,
    pub resolution: f64,
    pub record_ranges: Option<Vec<(usize, usize)>>,
}

pub fn g2(f: &File, params: &G2Params, mode: G2Mode) -> Result<G2Result, Error> {
    match mode {
        G2Mode::Symmetric => g2_symmetric::g2(f, params),
        G2Mode::Asymmetric => g2_asymmetric::g2(f, params),
    }
}
