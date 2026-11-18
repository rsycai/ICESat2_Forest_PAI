use crate::domain::{DiscoveryResult, RunOptions, SearchQuery};
use crate::pipeline::manifest::{Journal, PlannedFile};
use crate::pipeline::transfer::completion::Completion;
use crate::pipeline::transfer::error::cancelled;
use crate::pipeline::transfer::progress::Progress;
use crate::pipeline::transfer::sizes::inspect_sizes;
use crate::pipeline::transfer::summary::DownloadSummary;
use crate::services::storage::paths::display;
use std::collections::HashSet;
use std::io::Write;
use std::sync::Arc;
use xt_error::{Error, ErrorKind, Result};
use xt_transfer::{CancellationToken, DownloadOutcome, DownloadTask, FileDownloader};

#[allow(clippy::too_many_arguments)]
pub async fn run_selected<T: FileDownloader + 'static>(
    storage: &dyn crate::StoragePolicy,
    query: &SearchQuery,
    downloader: Arc<T>,
    options: RunOptions,
    cancel: CancellationToken,
    discovered: DiscoveryResult,
    output: &mut (dyn Write + Send),
    sink: Option<&dyn xt_task::interfaces::GroupSink>,
) -> Result<DownloadSummary> {
    let matched_groups = discovered.matched_groups;
    let (mut journal, mut files) = Journal::new(&options.paths, query, discovered.groups, storage)?;
    let skipped = if !options.search_only
        && !matches!(
            options.download.existing,
            xt_transfer::ExistingFilePolicy::Overwrite
        )
        && let Some(sink) = sink
    {
        crate::pipeline::transfer::preflight::skip_completed(&mut journal, &files, sink).await?
    } else {
        0
    };
    if skipped > 0 && skipped == files.len() as u64 {
        return Ok(DownloadSummary {
            matched_groups,
            granules: journal
                .entries
                .iter()
                .flat_map(|entry| entry.files.iter().map(|file| &file.granule_id))
                .collect::<HashSet<_>>()
                .len() as u64,
            files: skipped,
            skipped,
            manifests: journal.index_paths.values().cloned().collect(),
            ..Default::default()
        });
    }
    files.retain(|file| !journal.skipped_groups.contains(&file.group));
    journal.publish_catalog(crate::pipeline::manifest::CatalogState::Preparing)?;
    let inspected = if options.search_only {
        inspect_sizes(
            &downloader,
            &mut files,
            &mut journal,
            options.concurrency,
            &cancel,
        )
        .await
    } else {
        Ok(())
    };
    journal.publish_catalog(if inspected.is_ok() {
        crate::pipeline::manifest::CatalogState::Ready
    } else {
        crate::pipeline::manifest::CatalogState::Interrupted
    })?;
    inspected?;
    let granules = journal
        .entries
        .iter()
        .flat_map(|e| e.files.iter().map(|f| &f.granule_id))
        .collect::<HashSet<_>>()
        .len() as u64;
    let mut summary = DownloadSummary {
        matched_groups,
        granules,
        files: files.len() as u64 + skipped,
        skipped,
        manifests: journal.index_paths.values().cloned().collect(),
        ..Default::default()
    };
    for manifest in &summary.manifests {
        tracing::info!(manifest = %crate::services::storage::paths::display(manifest), "Search manifest generated");
    }
    if options.search_only {
        for file in &files {
            let record = &journal.entries[file.group].files[file.file];
            serde_json::to_writer(&mut *output, &serde_json::json!({ "granule": record.granule_id, "title": file.title, "path": display(&file.task.destination) }))
                .map_err(|_| Error::new(ErrorKind::Io(std::io::ErrorKind::BrokenPipe), "Failed to write search results"))?;
            writeln!(output)?;
        }
        if cancel.is_cancelled() {
            return Err(cancelled());
        }
        return Ok(summary);
    }
    journal.initialize_records()?;
    let mut ready = crate::pipeline::transfer::readiness::Readiness::new(&files);
    let mut jobs = tokio::task::JoinSet::new();
    for (index, file) in files.iter().enumerate() {
        if cancel.is_cancelled() {
            break;
        }
        let downloader = downloader.clone();
        let cancel = cancel.clone();
        let task = DownloadTask {
            asset: file.task.asset.clone(),
            destination: file.task.destination.clone(),
        };
        let download_options = options.download;
        let show_progress = options.progress;
        jobs.spawn(async move {
            let started = std::time::Instant::now();
            let observer = Progress {
                remote_size: std::sync::Mutex::new(None),
                enabled: show_progress,
                label: task
                    .destination
                    .file_name()
                    .unwrap_or_default()
                    .to_string_lossy()
                    .into_owned(),
                cancel,
                last: std::sync::Mutex::new(std::time::Instant::now()),
            };
            let result = downloader
                .download(&task, download_options, &observer)
                .await;
            let remote_size = observer.remote_size.lock().ok().and_then(|size| *size);
            Completion {
                index,
                result,
                remote_size,
                elapsed: started.elapsed(),
            }
        });
        if jobs.len() >= options.concurrency {
            let (index, success) = collect(
                &mut jobs,
                &files,
                &mut journal,
                &mut summary,
                sink.is_some_and(|s| s.accepts_unverified()),
            )
            .await?;
            if let Some(sink) = sink
                && let Some(group) = ready.finish(index, success, &files, &journal)?
            {
                sink.submit(group).await?;
            }
        }
    }
    while !jobs.is_empty() {
        let (index, success) = collect(
            &mut jobs,
            &files,
            &mut journal,
            &mut summary,
            sink.is_some_and(|s| s.accepts_unverified()),
        )
        .await?;
        if let Some(sink) = sink
            && let Some(group) = ready.finish(index, success, &files, &journal)?
        {
            sink.submit(group).await?;
        }
    }
    if cancel.is_cancelled() {
        journal.cancel_pending()?;
        return Err(cancelled());
    }
    Ok(summary)
}

pub(super) async fn collect(
    jobs: &mut tokio::task::JoinSet<Completion>,
    files: &[PlannedFile],
    journal: &mut Journal,
    summary: &mut DownloadSummary,
    allow_unverified: bool,
) -> Result<(usize, Option<bool>)> {
    let Completion {
        index,
        result,
        remote_size,
        elapsed,
    } = jobs
        .join_next()
        .await
        .ok_or_else(|| Error::new(ErrorKind::Internal, "Download queue is empty"))?
        .map_err(|_| Error::new(ErrorKind::Internal, "Download task terminated unexpectedly"))?;
    let file = &files[index];
    if remote_size.is_some() {
        journal.set_size(file, &Ok(remote_size));
    }
    journal.outcome(file, &result)?;
    let path = &file.task.destination;
    let success = match &result {
        Ok(DownloadOutcome::Completed { .. }) => Some(true),
        Ok(DownloadOutcome::Skipped) => allow_unverified.then_some(true),
        Err(_) => Some(false),
    };
    match result {
        Ok(DownloadOutcome::Completed { bytes }) => {
            summary.completed += 1;
            summary.bytes += bytes;
            tracing::info!(path = %crate::services::storage::paths::display(path), bytes, elapsed_seconds=elapsed.as_secs_f64(), average_mib_per_second=(bytes as f64 / 1048576.0 / elapsed.as_secs_f64().max(0.001)), "File download complete");
        }
        Ok(DownloadOutcome::Skipped) => {
            summary.skipped += 1;
            tracing::info!(path = %crate::services::storage::paths::display(path), "File already exists; download skipped");
        }
        Err(error) => {
            summary.failed += 1;
            tracing::error!(path = %crate::services::storage::paths::display(path), code = error.code(), error = %error, "File download failed");
        }
    }
    Ok((index, success))
}
