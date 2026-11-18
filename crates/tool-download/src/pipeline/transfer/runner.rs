use crate::domain::{PipelineSource, RunOptions, SearchQuery};
use crate::pipeline::transfer::scheduling::run_selected;
use crate::pipeline::transfer::summary::DownloadSummary;
use crate::services::transport::HttpTransport as CatalogTransport;
use std::io::Write;
use std::sync::Arc;
use xt_error::Result;
use xt_transfer::{CancellationToken, FileDownloader};

pub async fn execute<T: CatalogTransport + FileDownloader + 'static>(
    source: PipelineSource<'_>,
    query: &SearchQuery,
    downloader: T,
    options: RunOptions,
    cancel: CancellationToken,
    output: &mut (dyn Write + Send),
) -> Result<DownloadSummary> {
    execute_with_sink(source, query, downloader, options, cancel, output, None).await
}

#[allow(clippy::too_many_arguments)]
pub async fn execute_with_sink<T: CatalogTransport + FileDownloader + 'static>(
    source: PipelineSource<'_>,
    query: &SearchQuery,
    downloader: T,
    options: RunOptions,
    cancel: CancellationToken,
    output: &mut (dyn Write + Send),
    sink: Option<&dyn xt_task::interfaces::GroupSink>,
) -> Result<DownloadSummary> {
    options.validate()?;
    source.discovery.validate_options(&options)?;
    tracing::info!(source = source.id, "Starting data discovery");
    let downloader = Arc::new(downloader);
    let result = source
        .discovery
        .discover(downloader.as_ref(), query, &options, &cancel)
        .await?;
    run_selected(
        source.storage,
        query,
        downloader,
        options,
        cancel,
        result,
        output,
        sink,
    )
    .await
}
