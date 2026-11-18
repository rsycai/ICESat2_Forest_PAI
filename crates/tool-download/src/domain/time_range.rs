use crate::domain::error::invalid;
use chrono::{DateTime, SecondsFormat, Utc};
use xt_error::Result;

#[derive(Debug, Clone)]
pub struct TimeRange {
    start: DateTime<Utc>,
    end: DateTime<Utc>,
}

impl TimeRange {
    pub fn new(start: &str, end: &str) -> Result<Self> {
        let parse = |value: &str| {
            chrono::DateTime::parse_from_rfc3339(value)
                .map_err(|_| invalid("Time must be RFC3339 with a timezone"))
        };
        let start = parse(start)?;
        let end = parse(end)?;
        if start > end {
            return Err(invalid("Start time must not be later than end time"));
        }
        Ok(Self {
            start: start.to_utc(),
            end: end.to_utc(),
        })
    }

    pub fn days(&self) -> impl Iterator<Item = Self> {
        let mut next = Some(self.start);
        let end = self.end;
        std::iter::from_fn(move || {
            let start = next.take()?;
            let stop = start
                .date_naive()
                .succ_opt()
                .map(|d| d.and_hms_opt(0, 0, 0).unwrap().and_utc().min(end))
                .unwrap_or(end);
            next = (stop < end).then_some(stop);
            Some(Self { start, end: stop })
        })
    }
    pub fn parameter(&self) -> String {
        format!(
            "{},{}",
            self.start.to_rfc3339_opts(SecondsFormat::AutoSi, true),
            self.end.to_rfc3339_opts(SecondsFormat::AutoSi, true)
        )
    }
}
