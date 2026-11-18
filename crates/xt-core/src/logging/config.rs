use chrono_tz::Tz;
use std::path::PathBuf;
use tracing_subscriber::EnvFilter;
use xt_error::{Error, ErrorKind, Result};

#[derive(Debug, Clone, xt_config::EnvLoad, xt_config::SubConfig)]
#[env(prefix = "XT", section = "logging", validate = "validate")]
pub struct LogConfig {
    #[env(default = "logs", os, alias = "XT_LOG_DIR")]
    pub directory: PathBuf,
    #[env(default = "XT")]
    pub prefix: String,
    #[env(default = false, alias = "XT_LOG_CONSOLE")]
    pub console: bool,
    #[env(default = "info", alias = "XT_LOG_FILTER")]
    pub filter: String,
    #[env(default = 30, alias = "XT_LOG_DAYS")]
    pub retention_days: u8,
    #[env(default = chrono_tz::Asia::Shanghai, alias = "XT_LOG_TIMEZONE")]
    pub timezone: Tz,
}

impl LogConfig {
    pub fn from_env(prefix: &str) -> Result<Self> {
        let mut config = Self::new()?;
        let configured_prefix = xt_config::env_load::with_file_defaults(|| {
            xt_config::env_load::read_os(&["XT__LOGGING__PREFIX"])
        })?;
        if configured_prefix.is_none() {
            config.prefix = prefix.into();
        }
        config.validate()?;
        Ok(config)
    }
    pub(super) fn validate(&self) -> Result<EnvFilter> {
        if self.prefix.is_empty()
            || !self
                .prefix
                .bytes()
                .all(|b| b.is_ascii_alphanumeric() || b == b'_' || b == b'-')
        {
            return Err(Error::new(
                ErrorKind::Configuration,
                "Log prefix allows ASCII letters, digits, underscores, and hyphens only",
            ));
        }
        if !(1..=30).contains(&self.retention_days) {
            return Err(Error::new(
                ErrorKind::Configuration,
                "Log retention must be in 1..=30 days",
            ));
        }
        EnvFilter::try_new(&self.filter)
            .map_err(|e| Error::new(ErrorKind::Configuration, "Invalid log filter expression").with_source(e))
    }
}
