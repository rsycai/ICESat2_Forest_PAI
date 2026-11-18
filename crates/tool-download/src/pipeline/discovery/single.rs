use crate::domain::{DiscoveryResult, RunOptions, SearchQuery};
use crate::interfaces::{DataSource, Discovery};
use crate::services::transport::HttpTransport as CatalogTransport;
use xt_error::Result;
use xt_transfer::{CancellationToken, DownloadFuture};

pub struct SingleDiscovery<S> {
    source: S,
}
impl<S: DataSource> SingleDiscovery<S> {
    pub fn new(source: S) -> Self {
        Self { source }
    }
}
impl<S: DataSource> Discovery for SingleDiscovery<S> {
    fn plan(&self, query: &SearchQuery) -> Result<serde_json::Value> {
        serde_json::to_value(self.source.search_request(query, None)?)
            .map_err(|_| crate::domain::invalid("Cannot serialize search plan"))
    }
    fn discover<'a>(
        &'a self,
        transport: &'a dyn CatalogTransport,
        query: &'a SearchQuery,
        options: &'a RunOptions,
        cancel: &'a CancellationToken,
    ) -> DownloadFuture<'a, DiscoveryResult> {
        Box::pin(crate::pipeline::discovery::pages::discover_pages(
            &self.source,
            transport,
            query,
            options,
            cancel,
        ))
    }
}
