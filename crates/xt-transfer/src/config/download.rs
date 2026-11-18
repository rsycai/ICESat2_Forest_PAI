use crate::authentication::Authentication;
use crate::domain::{Checksum, DownloadOptions, ExistingFilePolicy};
use crate::interfaces::DownloadObserver;
use std::sync::Arc;
use std::time::Duration;
use tokio_util::sync::CancellationToken;
use xt_error::{Error, ErrorKind, Result};

#[derive(Clone)]
pub struct DownloadConfig {
    pub transfer: TransferSettings,
    pub existing: ExistingFilePolicy,
    pub resume: bool,
    pub size_bytes: Option<u64>,
    pub checksum: Option<Checksum>,
    pub authorization: Authentication,
    pub cancellation: Option<CancellationToken>,
    pub observer: Option<Arc<dyn DownloadObserver>>,
}

impl Default for DownloadConfig {
    fn default() -> Self {
        Self {
            transfer: TransferSettings::default(),
            existing: ExistingFilePolicy::Fail,
            resume: true,
            size_bytes: None,
            checksum: None,
            authorization: Authentication::None,
            cancellation: None,
            observer: None,
        }
    }
}

#[derive(Clone)]
pub struct FileDownloadConfig<'a> {
    pub options: DownloadOptions,
    pub size_bytes: Option<u64>,
    pub checksum: Option<Checksum>,
    pub observer: Option<&'a dyn DownloadObserver>,
}

impl Default for FileDownloadConfig<'_> {
    fn default() -> Self {
        Self {
            options: DownloadOptions {
                resume: true,
                ..Default::default()
            },
            size_bytes: None,
            checksum: None,
            observer: None,
        }
    }
}

#[derive(Clone)]
pub struct TransferSettings {
    pub workers: u64,
    pub multipart_threshold_bytes: u64,
    pub timeout: Duration,
    pub proxy: Option<String>,
    pub speed_limit: Option<u64>,
    pub lock_directory: Option<std::path::PathBuf>,
    pub allow_http_loopback: bool,
}

impl Default for TransferSettings {
    fn default() -> Self {
        Self {
            workers: 4,
            multipart_threshold_bytes: 1_000_000_000,
            timeout: Duration::from_secs(60),
            proxy: None,
            speed_limit: None,
            lock_directory: None,
            allow_http_loopback: false,
        }
    }
}

impl TransferSettings {
    pub fn validate(&self) -> Result<()> {
        if self.workers == 0
            || self.workers > 64
            || self.multipart_threshold_bytes == 0
            || self.timeout.is_zero()
            || self
                .speed_limit
                .is_some_and(|n| n == 0 || n > u32::MAX as u64)
        {
            return Err(Error::new(
                ErrorKind::InvalidArgument,
                "Invalid transfer concurrency, chunk threshold, timeout, or rate limit",
            ));
        }
        Ok(())
    }

    pub(crate) fn workers_for(&self, size: Option<u64>, ranges: bool) -> u64 {
        if ranges && size.is_some_and(|size| size > self.multipart_threshold_bytes) {
            self.workers
        } else {
            1
        }
    }
}
