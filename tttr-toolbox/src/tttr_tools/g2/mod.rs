use crate::errors::Error;
use crate::headers::File;
use crate::array_stream::ArrayStream;

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

pub fn g2_from_arrays(
    timestamps_ps: &[u64],
    channels: &[i32],
    params: &G2Params,
    mode: G2Mode,
) -> Result<G2Result, Error> 
{

    // ---------------------------------------------------------
    // No record selection:
    // analyse the entire stream.
    // ---------------------------------------------------------

    if params.record_ranges.is_none() {

        let stream = ArrayStream::new(
            timestamps_ps,
            channels,
            None,
            None,
        )?;

        let result = match mode {
            G2Mode::Symmetric => {
                g2_symmetric::g2_from_stream(
                    stream,
                    params,
                    1e-12,
                )
            }

            G2Mode::Asymmetric => {
                g2_asymmetric::g2_from_stream(
                    stream,
                    params,
                    1e-12,
                )
            }
        };

        return Ok(result);
    }


    // ---------------------------------------------------------
    // Record-range selection
    // ---------------------------------------------------------

    let ranges = params
        .record_ranges
        .as_ref()
        .unwrap();

    let mut total_result: Option<G2Result> = None;

    for &(start_record, stop_record) in ranges {

        let stream = ArrayStream::new(
            timestamps_ps,
            channels,
            Some(start_record),
            Some(stop_record),
        )?;

        let result = match mode {
            G2Mode::Symmetric => {
                g2_symmetric::g2_from_stream(
                    stream,
                    params,
                    1e-12,
                )
            }

            G2Mode::Asymmetric => {
                g2_asymmetric::g2_from_stream(
                    stream,
                    params,
                    1e-12,
                )
            }
        };

        if let Some(total) = total_result.as_mut() {

            for (dst, src) in
                total.hist.iter_mut().zip(result.hist.iter())
            {
                *dst += *src;
            }

        } else {
            total_result = Some(result);
        }
    }


    // Empty range list: return an empty histogram of the
    // correct dimensions.
    if let Some(result) = total_result {
        Ok(result)
    } else {

        let stream = ArrayStream::new(
            timestamps_ps,
            channels,
            Some(0),
            Some(0),
        )?;

        Ok(match mode {
            G2Mode::Symmetric => {
                g2_symmetric::g2_from_stream(
                    stream,
                    params,
                    1e-12,
                )
            }

            G2Mode::Asymmetric => {
                g2_asymmetric::g2_from_stream(
                    stream,
                    params,
                    1e-12,
                )
            }
        })
    }
}