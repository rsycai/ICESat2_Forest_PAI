use crate::cli::params::{DownloadMode, DownloadParams};
use crate::config::catalog::DownloadCatalog;
use crate::pipeline::command::authenticated_downloader;
use crate::pipeline::execute_with_sink;
use crate::services::registry;
use std::io;
use std::io::Write;
use xt_error::{DownloadError, Error, ErrorKind, Result, ResultExt};
use xt_transfer::CancellationToken;

pub async fn run(params: DownloadParams) -> Result<()> {
    run_with_sink(params, None).await
}

pub async fn run_with_sink(
    params: DownloadParams,
    sink: Option<&dyn xt_task::interfaces::GroupSink>,
) -> Result<()> {
    run_with_context(params, sink, CancellationToken::new()).await
}

pub async fn run_with_context(
    params: DownloadParams,
    sink: Option<&dyn xt_task::interfaces::GroupSink>,
    cancel: CancellationToken,
) -> Result<()> {
    params.validate()?;
    let catalog = DownloadCatalog::load_default()?;
    let source = registry::create(
        &catalog,
        params.source.as_str(),
        params.product.as_deref(),
        params.release.as_deref(),
    )?;
    source.discovery.validate_options(&params.options)?;
    if params.mode == DownloadMode::Plan {
        let request = source.discovery.plan(&params.query)?;
        serde_json::to_writer_pretty(io::stdout().lock(), &request)
            .map_err(|_| Error::new(ErrorKind::Internal, "Failed to write request plan"))?;
        return writeln!(io::stdout().lock()).context("Write output newline");
    }

    let parent_cancel = cancel;
    let cancel = parent_cancel.child_token();
    let _cancel_on_drop = cancel.clone().drop_guard();
    let operation = async {
        let downloader = authenticated_downloader(
            &catalog,
            &source,
            params.transfer,
            params.requests,
            cancel.clone(),
        )
        .await?;
        execute_with_sink(
            source.as_source(),
            &params.query,
            downloader,
            params.options,
            cancel.clone(),
            &mut io::stdout(),
            sink,
        )
        .await
    };
    tokio::pin!(operation);
    let summary = tokio::select! {
        result = &mut operation => result,
        signal = tokio::signal::ctrl_c() => {
            match signal {
                Ok(()) => parent_cancel.cancel(),
                Err(error) => tracing::warn!(error = %error, "Cannot listen for cancellation; continuing download"),
            }
            operation.await
        }
    }?;
    tracing::info!(
        matched_groups = summary.matched_groups,
        granules = summary.granules,
        files = summary.files,
        completed = summary.completed,
        skipped = summary.skipped,
        failed = summary.failed,
        bytes = summary.bytes,
        "Download summary"
    );
    if summary.failed > 0 {
        return Err(Error::new(
            ErrorKind::Download(DownloadError::Interrupted),
            format!("{} file downloads failed; rerun the command to resume", summary.failed),
        ));
    }
    Ok(())
}
