use crate::cli::params::arguments::SourceId;
use crate::domain::{RunOptions, SearchQuery};
use std::time::Duration;
use xt_error::Result;
use xt_transfer::TransferSettings;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DownloadMode {
    Plan,
    Search,
    Fetch,
}

#[derive(Clone)]
pub struct DownloadParams {
    pub mode: DownloadMode,
    pub source: SourceId,
    pub product: Option<String>,
    pub release: Option<String>,
    pub query: SearchQuery,
    pub transfer: TransferSettings,
    pub requests: crate::services::transport::RequestSettings,
    pub options: RunOptions,
}

impl DownloadParams {
    pub fn validate(&self) -> Result<()> {
        self.options.validate()?;
        self.transfer.validate()?;
        self.requests.validate()?;
        if self.query.page_size == 0 || self.transfer.timeout > Duration::from_secs(86400) {
            return Err(crate::domain::invalid("Invalid page size, timeout, or retry delay"));
        }
        Ok(())
    }
}
