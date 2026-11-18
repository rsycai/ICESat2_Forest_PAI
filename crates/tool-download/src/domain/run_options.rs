use crate::domain::error::invalid;
use xt_error::Result;
use xt_transfer::DownloadOptions;

#[derive(Debug, Clone)]
pub struct RunOptions {
    pub paths: xt_config::paths::PathsConfig,
    pub concurrency: usize,
    pub progress: bool,
    pub download: DownloadOptions,
    pub search_only: bool,
}

impl RunOptions {
    pub fn validate(&self) -> Result<()> {
        self.paths.validate()?;
        if !(1..=64).contains(&self.concurrency) {
            return Err(invalid("File concurrency must be in 1..=64"));
        }
        Ok(())
    }
}
