use chrono::{Datelike, Utc};
use chrono_tz::Tz;
use std::path::Path;
use time::Date;
use tracing_appender::rolling::{RollingFileAppender, Rotation};

pub(super) fn local_date(now: chrono::DateTime<Utc>, timezone: Tz) -> Date {
    let local = now.with_timezone(&timezone);
    Date::from_ordinal_date(local.year(), local.ordinal() as u16)
        .expect("Current system time must be within the supported year range")
}

pub(super) fn daily_appender(
    directory: &Path,
    prefix: &str,
    date: Date,
) -> std::result::Result<RollingFileAppender, tracing_appender::rolling::InitError> {
    RollingFileAppender::builder()
        .rotation(Rotation::NEVER)
        .filename_prefix(format!("{prefix}.{date}"))
        .filename_suffix("log")
        .build(directory)
}
