use crate::pipeline::manifest::{Journal, PlannedFile};
use crate::pipeline::transfer::error::cancelled;
use std::sync::Arc;
use xt_error::{Error, ErrorKind, Result};
use xt_transfer::{CancellationToken, FileDownloader};

pub(super) async fn inspect_sizes<T: FileDownloader + 'static>(
    downloader: &Arc<T>,
    files: &mut [PlannedFile],
    journal: &mut Journal,
    concurrency: usize,
    cancel: &CancellationToken,
) -> Result<()> {
    let mut jobs = tokio::task::JoinSet::new();
    for (index, file) in files.iter().enumerate() {
        if file.task.asset.size_bytes.is_some() {
            continue;
        }
        let downloader = downloader.clone();
        let asset = file.task.asset.clone();
        jobs.spawn(async move { (index, downloader.remote_size(&asset).await) });
        if jobs.len() >= concurrency {
            collect_size(&mut jobs, journal, files, cancel).await?;
        }
    }
    while !jobs.is_empty() {
        collect_size(&mut jobs, journal, files, cancel).await?;
    }
    for file in files {
        file.task.asset.size_bytes =
            journal.entries[file.group].files[file.file].expected_size_bytes;
    }
    Ok(())
}

pub(super) async fn collect_size(
    jobs: &mut tokio::task::JoinSet<(usize, Result<Option<u64>>)>,
    journal: &mut Journal,
    files: &[PlannedFile],
    cancel: &CancellationToken,
) -> Result<()> {
    let item = tokio::select! { biased; _ = cancel.cancelled() => return Err(cancelled()), item = jobs.join_next() => item };
    let (index, result) = item
        .ok_or_else(|| Error::new(ErrorKind::Internal, "Length probe queue is empty"))?
        .map_err(|_| Error::new(ErrorKind::Internal, "Length probe task terminated unexpectedly"))?;
    journal.set_size(&files[index], &result);
    Ok(())
}
