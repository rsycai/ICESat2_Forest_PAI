use crate::config::{DownloadConfig, FileDownloadConfig};
use crate::domain::{Asset, DownloadOptions, DownloadOutcome, DownloadTask};
use crate::interfaces::FileDownloader;
use crate::interfaces::download::SilentObserver;
use crate::runtime::{HyaDownloader, validate_url};
use std::path::Path;
use xt_error::Result;

///
///
/// ```no_run
/// # async fn example() -> xt_transfer::Result<()> {
/// xt_transfer::download_file("https://example.com/data.h5", "data/data.h5").await?;
/// # Ok(())
/// # }
/// ```
pub async fn download_file(
    url: impl AsRef<str>,
    destination: impl AsRef<Path>,
) -> Result<DownloadOutcome> {
    download_file_with(url, destination, DownloadConfig::default()).await
}

///
///
/// ```no_run
/// # async fn example() -> xt_transfer::Result<()> {
/// use xt_transfer::{DownloadConfig, ExistingFilePolicy, TransferSettings, download_file_with};
/// let config = DownloadConfig {
///     existing: ExistingFilePolicy::Skip,
///     transfer: TransferSettings {
///         proxy: Some("http://127.0.0.1:7890".into()),
///         ..Default::default()
///     },
///     ..Default::default()
/// };
/// download_file_with("https://example.com/data.h5", "data/data.h5", config).await?;
/// # Ok(())
/// # }
/// ```
pub async fn download_file_with(
    url: impl AsRef<str>,
    destination: impl AsRef<Path>,
    config: DownloadConfig,
) -> Result<DownloadOutcome> {
    let url = validate_url(url.as_ref(), config.transfer.allow_http_loopback)?;
    let authorization = config.authorization.resolve(&url)?;
    let downloader = HyaDownloader::new(
        config.transfer,
        authorization,
        config.cancellation.unwrap_or_default(),
    )?;
    download_file_with_client(
        &downloader,
        url.as_str(),
        destination,
        FileDownloadConfig {
            options: DownloadOptions {
                existing: config.existing,
                resume: config.resume,
            },
            size_bytes: config.size_bytes,
            checksum: config.checksum,
            observer: config.observer.as_deref(),
        },
    )
    .await
}

///
///
/// ```no_run
/// # async fn example(client: &xt_transfer::HyaDownloader, url: &str) -> xt_transfer::Result<()> {
/// xt_transfer::download_file_with_client(
///     client, url, "data/example.h5", xt_transfer::FileDownloadConfig::default(),
/// ).await?;
/// # Ok(())
/// # }
/// ```
pub async fn download_file_with_client(
    downloader: &HyaDownloader,
    url: impl AsRef<str>,
    destination: impl AsRef<Path>,
    config: FileDownloadConfig<'_>,
) -> Result<DownloadOutcome> {
    let url = downloader.url(url.as_ref())?;
    let task = DownloadTask {
        asset: Asset {
            url: url.to_string(),
            size_bytes: config.size_bytes,
            checksum: config.checksum,
        },
        destination: destination.as_ref().to_path_buf(),
    };
    downloader
        .download(
            &task,
            config.options,
            config.observer.unwrap_or(&SilentObserver),
        )
        .await
}
