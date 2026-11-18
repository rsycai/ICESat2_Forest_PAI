use crate::domain::{DownloadOptions, DownloadOutcome, DownloadTask, ExistingFilePolicy};
use crate::interfaces::DownloadObserver;
use crate::runtime::HyaDownloader;
use crate::runtime::errors::cancelled;
use crate::storage::checksum::{validate_checksum, verify};
use crate::storage::files::{append_suffix, check_file, remove_if_exists};
use http::header::HeaderValue;
use std::fs;
use std::fs::OpenOptions;
use std::path::Path;
use xt_error::{DownloadError, Error, ErrorKind, Result};

impl HyaDownloader {
    pub(super) async fn file(
        &self,
        task: &DownloadTask,
        options: DownloadOptions,
        observer: &dyn DownloadObserver,
    ) -> Result<DownloadOutcome> {
        if self.cancel.is_cancelled() || observer.is_cancelled() {
            return Err(cancelled());
        }
        let destination = &task.destination;
        if destination.file_name().is_none() {
            return Err(Error::new(
                ErrorKind::InvalidArgument,
                "Download target must be a file path",
            ));
        }
        if let Some(parent) = destination.parent().filter(|p| !p.as_os_str().is_empty()) {
            fs::create_dir_all(parent)?;
        }
        let parent = destination
            .parent()
            .filter(|p| !p.as_os_str().is_empty())
            .unwrap_or_else(|| Path::new("."));
        let lock_directory = self
            .settings
            .lock_directory
            .clone()
            .unwrap_or_else(|| parent.join(".xt-locks"));
        let lock = std::sync::Arc::new(
            xt_lock::FileLock::try_for_file(destination, &lock_directory)
                .map_err(|error| Error::from(error).context("Acquire download file lock"))?,
        );
        if check_file(destination)? {
            match options.existing {
                ExistingFilePolicy::Skip => return Ok(DownloadOutcome::Skipped),
                ExistingFilePolicy::Fail => {
                    return Err(std::io::Error::new(
                        std::io::ErrorKind::AlreadyExists,
                        "Target file already exists",
                    )
                    .into());
                }
                ExistingFilePolicy::Overwrite => (),
            }
        }
        validate_checksum(task.asset.checksum.as_ref())?;
        let partial = append_suffix(destination, ".part");
        let sidecar = append_suffix(&partial, ".hya.json");
        for path in [&partial, &sidecar] {
            check_file(path)?;
        }
        let probe = self.probe(&task.asset.url).await?;
        observer.remote_size(probe.identity.size);
        if let (Some(expected), Some(actual)) = (task.asset.size_bytes, probe.identity.size)
            && expected != actual
        {
            return Err(Error::new(
                ErrorKind::Download(DownloadError::Incomplete),
                "Remote file size differs from asset metadata",
            ));
        }
        let mut headers = self.auth.headers(&probe.url)?;
        if let Some(etag) = &probe.identity.etag {
            headers.insert(
                http::header::IF_MATCH,
                HeaderValue::from_str(etag)
                    .map_err(|_| Error::new(ErrorKind::InvalidArgument, "Invalid remote ETag"))?,
            );
        } else if let Some(modified) = &probe.identity.modified {
            headers.insert(
                http::header::IF_UNMODIFIED_SINCE,
                HeaderValue::from_str(modified)
                    .map_err(|_| Error::new(ErrorKind::InvalidArgument, "Invalid remote modification time"))?,
            );
        }
        self.run_backend(
            &probe,
            partial.clone(),
            headers,
            options.resume && probe.ranges,
            observer,
            lock.clone(),
        )
        .await
        .map_err(|error| error.context("Transfer file contents"))?;
        if self.cancel.is_cancelled() || observer.is_cancelled() {
            return Err(cancelled());
        }
        let size = fs::metadata(&partial)?.len();
        if task
            .asset
            .size_bytes
            .or(probe.identity.size)
            .is_some_and(|expected| expected != size)
        {
            return Err(Error::new(
                ErrorKind::Download(DownloadError::Incomplete),
                "Downloaded file length verification failed",
            ));
        }
        let check_path = partial.clone();
        let checksum = task.asset.checksum.clone();
        if let Err(error) =
            tokio::task::spawn_blocking(move || verify(&check_path, checksum.as_ref()))
                .await
                .map_err(|_| Error::new(ErrorKind::Internal, "File verification task failed"))?
        {
            remove_if_exists(&partial)?;
            remove_if_exists(&sidecar)?;
            return Err(error);
        }
        if self.cancel.is_cancelled() || observer.is_cancelled() {
            return Err(cancelled());
        }
        OpenOptions::new().write(true).open(&partial)?.sync_all()?;
        match options.existing {
            ExistingFilePolicy::Overwrite => fs::rename(&partial, destination)?,
            _ => match fs::hard_link(&partial, destination) {
                Ok(()) => fs::remove_file(&partial)?,
                Err(e)
                    if e.kind() == std::io::ErrorKind::AlreadyExists
                        && matches!(options.existing, ExistingFilePolicy::Skip) =>
                {
                    return Ok(DownloadOutcome::Skipped);
                }
                Err(e) => return Err(e.into()),
            },
        }
        remove_if_exists(&sidecar)?;
        observer.progress(size, Some(size));
        Ok(DownloadOutcome::Completed { bytes: size })
    }
}
