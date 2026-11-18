use crate::config_env::model::{EnvConfig, ExportReport};
use std::path::Path;
use xt_error::Result;

#[cfg(windows)]
pub fn init_config(path: impl AsRef<Path>, prefix: &str) -> Result<ExportReport> {
    Ok(EnvConfig::from_path(path, prefix)?.apply())
}

///
/// # Safety
#[cfg(not(windows))]
pub unsafe fn init_config(path: impl AsRef<Path>, prefix: &str) -> Result<ExportReport> {
    let config = EnvConfig::from_path(path, prefix)?;
    Ok(unsafe { config.apply() })
}
