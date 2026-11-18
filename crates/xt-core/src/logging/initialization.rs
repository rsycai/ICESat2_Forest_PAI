use crate::logging::config::LogConfig;
use crate::logging::diagnostics::record_config_notices;
use crate::logging::formatter::LogFormat;
use crate::logging::guard::LogGuard;
use crate::logging::retention::cleanup;
use crate::logging::rotation::{daily_appender, local_date};
use crate::logging::timer::ZonedTimer;
use crate::logging::writer::RetainedWriter;
use chrono::Utc;
use std::{fs, io};
use tracing_appender::non_blocking::NonBlockingBuilder;
use tracing_subscriber::layer::SubscriberExt;
use tracing_subscriber::util::SubscriberInitExt;
use xt_error::{Error, ErrorKind, Result, ResultExt};

pub fn init_logging(config: &LogConfig) -> Result<LogGuard> {
    let filter = config.validate()?;
    fs::create_dir_all(&config.directory).context("Create log directory")?;
    let today = local_date(Utc::now(), config.timezone);
    cleanup(
        &config.directory,
        &config.prefix,
        config.retention_days,
        today,
    )
    .context("Remove expired logs")?;
    let appender = daily_appender(&config.directory, &config.prefix, today).map_err(|e| {
        Error::new(ErrorKind::Configuration, "Failed to create daily log file").with_source(e)
    })?;
    let writer = RetainedWriter {
        appender,
        directory: config.directory.clone(),
        prefix: config.prefix.clone(),
        days: config.retention_days,
        last_cleanup: today,
        current_date: today,
        timezone: config.timezone,
    };
    let (writer, worker) = NonBlockingBuilder::default().lossy(false).finish(writer);
    let file_layer = tracing_subscriber::fmt::layer()
        .with_ansi(false)
        .event_format(LogFormat(ZonedTimer(config.timezone)))
        .with_writer(writer);
    let console_layer = config.console.then(|| {
        tracing_subscriber::fmt::layer()
            .with_ansi(false)
            .event_format(LogFormat(ZonedTimer(config.timezone)))
            .with_writer(io::stderr)
    });
    tracing_subscriber::registry()
        .with(filter)
        .with(file_layer)
        .with(console_layer)
        .try_init()
        .map_err(|e| {
            Error::new(
                ErrorKind::Configuration,
                format!("Failed to initialize logging; a global subscriber may already exist: {e}"),
            )
        })?;
    record_config_notices();
    Ok(LogGuard { _worker: worker })
}
