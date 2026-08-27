use crate::errors::Error;
use crate::{TTTRRecord, TTTRStream};

pub(crate) struct ArrayStream<'a> {
    timestamps_ps: &'a [u64],
    channels: &'a [i32],

    current_record: usize,
    stop_record: usize,

    // We measure time relative to the first timestamp.
    origin_ps: u64,
}

impl<'a> ArrayStream<'a> {
    pub(crate) fn new(
        timestamps_ps: &'a [u64],
        channels: &'a [i32],
        start_record: Option<usize>,
        stop_record: Option<usize>,
    ) -> Result<Self, Error> {

        if timestamps_ps.len() != channels.len() {
            return Err(Error::InvalidInput(
                "timestamps and channels must have equal length".to_string()
            ));
        }

        if timestamps_ps.windows(2).any(|w| w[1] < w[0]) {
            return Err(Error::InvalidInput(
                "timestamps must be sorted in chronological order".to_string()
            ));
        }

        let n_records = timestamps_ps.len();

        let start = start_record.unwrap_or(0);
        let stop = stop_record.unwrap_or(n_records);

        if start > stop || stop > n_records {
            return Err(Error::InvalidInput(
                "invalid record range".to_string()
            ));
        }

        let origin_ps = timestamps_ps
            .first()
            .copied()
            .unwrap_or(0);

        Ok(Self {
            timestamps_ps,
            channels,
            current_record: start,
            stop_record: stop,
            origin_ps,
        })
    }
}


impl<'a> TTTRStream for ArrayStream<'a> {
    type RecordSize = (u64, i32);

    #[inline(always)]
    fn parse_record(
        &mut self,
        raw_record: Self::RecordSize,
    ) -> TTTRRecord {
        TTTRRecord {
            channel: raw_record.1,
            tof: raw_record.0 - self.origin_ps,
        }
    }

    #[inline(always)]
    fn time_resolution(&self) -> f64 {
        // timestamps are supplied in picoseconds
        1e-12
    }
}


impl<'a> Iterator for ArrayStream<'a> {
    type Item = TTTRRecord;

    #[inline(always)]
    fn next(&mut self) -> Option<Self::Item> {

        if self.current_record >= self.stop_record {
            return None;
        }

        let idx = self.current_record;

        let raw_record = (
            self.timestamps_ps[idx],
            self.channels[idx],
        );

        self.current_record += 1;

        Some(self.parse_record(raw_record))
    }
}