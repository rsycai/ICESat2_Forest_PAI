use crate::services::transport::HttpTransport;
use crate::{DiscoveryResult, DownloadFuture, Result, RunOptions, SearchQuery};
use xt_transfer::CancellationToken;

pub trait Discovery: Send + Sync {
    fn plan(&self, query: &SearchQuery) -> Result<serde_json::Value>;
    fn validate_options(&self, _options: &RunOptions) -> Result<()> {
        Ok(())
    }
    fn discover<'a>(
        &'a self,
        transport: &'a dyn HttpTransport,
        query: &'a SearchQuery,
        options: &'a RunOptions,
        cancel: &'a CancellationToken,
    ) -> DownloadFuture<'a, DiscoveryResult>;
}
