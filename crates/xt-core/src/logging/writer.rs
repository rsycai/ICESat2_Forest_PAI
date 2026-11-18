use crate::logging::diagnostics::with_stderr_diagnostics;
use crate::logging::retention::cleanup;
use crate::logging::rotation::{daily_appender, local_date};
use chrono::Utc;
use chrono_tz::Tz;
use std::io;
use std::io::Write;
use std::path::PathBuf;
use time::Date;
use tracing_appender::rolling::RollingFileAppender;

pub(super) struct RetainedWriter {
    pub(super) appender: RollingFileAppender,
    pub(super) directory: PathBuf,
    pub(super) prefix: String,
    pub(super) days: u8,
    pub(super) last_cleanup: Date,
    pub(super) current_date: Date,
    pub(super) timezone: Tz,
}

impl RetainedWriter {
    pub(super) fn prepare(&mut self, now: chrono::DateTime<Utc>) -> io::Result<()> {
        let today = local_date(now, self.timezone);
        if today != self.current_date {
            self.appender.flush()?;
            self.appender =
                daily_appender(&self.directory, &self.prefix, today).map_err(io::Error::other)?;
            self.current_date = today;
        }
        if today != self.last_cleanup {
            match cleanup(&self.directory, &self.prefix, self.days, today) {
                Ok(()) => self.last_cleanup = today,
                Err(error) => with_stderr_diagnostics(self.timezone, || {
                    tracing::warn!(error = %error, "Failed to remove expired logs");
                }),
            }
        }
        Ok(())
    }
}

impl Write for RetainedWriter {
    fn write(&mut self, buffer: &[u8]) -> io::Result<usize> {
        self.prepare(Utc::now())?;
        self.appender.write(buffer)
    }

    fn flush(&mut self) -> io::Result<()> {
        self.appender.flush()
    }
}
