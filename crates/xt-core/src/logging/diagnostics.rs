use crate::logging::config::LogConfig;
use crate::logging::formatter::LogFormat;
use crate::logging::timer::ZonedTimer;
use chrono_tz::Tz;
use std::fmt::Display;
use std::io;

pub(super) fn with_stderr_diagnostics(timezone: Tz, operation: impl FnOnce()) {
    let subscriber = tracing_subscriber::fmt()
        .with_ansi(false)
        .event_format(LogFormat(ZonedTimer(timezone)))
        .with_writer(io::stderr)
        .finish();
    tracing::subscriber::with_default(subscriber, operation);
}

pub(super) fn record_config_notices() {
    if let Some(path) = xt_config::take_missing_default() {
        tracing::warn!(path = %path.display(), "No configuration file found; using defaults");
    }
}

pub(crate) fn report_error(error: impl Display) {
    let active = tracing::dispatcher::get_default(|dispatcher| {
        !dispatcher.is::<tracing::subscriber::NoSubscriber>()
    });
    if active {
        tracing::error!(error = %error, "Task failed");
    } else {
        with_stderr_diagnostics(LogConfig::default().timezone, || {
            record_config_notices();
            tracing::error!(error = %error, "Task failed");
        });
    }
}
