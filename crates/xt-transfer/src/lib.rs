mod api;
mod authentication;
mod config;
mod domain;
mod hya;
mod interfaces;
mod runtime;
mod storage;
pub use api::{download_file, download_file_with, download_file_with_client};
pub use authentication::{Authentication, Session};
pub use config::{DownloadConfig, FileDownloadConfig, TransferSettings};
pub use domain::{
    Asset, Checksum, DownloadOptions, DownloadOutcome, DownloadTask, ExistingFilePolicy,
};
pub use interfaces::{
    DownloadFuture, DownloadObserver, FileDownloader, NoAuthorization, RequestAuthorization,
};
pub use runtime::{HyaDownloader, validate_url};
pub use storage::{append_suffix, asset_key};
pub use tokio_util::sync::CancellationToken;
pub use xt_error::{Error, ErrorKind, Result};
