use chrono::Utc;
use chrono_tz::Tz;

#[derive(Clone, Copy)]
pub(super) struct ZonedTimer(pub(super) Tz);

impl tracing_subscriber::fmt::time::FormatTime for ZonedTimer {
    fn format_time(
        &self,
        writer: &mut tracing_subscriber::fmt::format::Writer<'_>,
    ) -> std::fmt::Result {
        write!(
            writer,
            "{}",
            Utc::now()
                .with_timezone(&self.0)
                .to_rfc3339_opts(chrono::SecondsFormat::Millis, false)
        )
    }
}
