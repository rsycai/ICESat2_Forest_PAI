use crate::domain::{Asset, DownloadOptions, DownloadOutcome, DownloadTask};
use std::future::Future;
use std::pin::Pin;
use xt_error::Result;

pub trait FileDownloader: Send + Sync {
    fn remote_size<'a>(&'a self, asset: &'a Asset) -> DownloadFuture<'a, Option<u64>> {
        Box::pin(async move { Ok(asset.size_bytes) })
    }

    fn download<'a>(
        &'a self,
        task: &'a DownloadTask,
        options: DownloadOptions,
        observer: &'a dyn DownloadObserver,
    ) -> DownloadFuture<'a, DownloadOutcome>;
}

pub type DownloadFuture<'a, T> = Pin<Box<dyn Future<Output = Result<T>> + Send + 'a>>;

pub trait DownloadObserver: Send + Sync {
    fn is_cancelled(&self) -> bool;
    fn remote_size(&self, _size: Option<u64>) {}
    fn progress(&self, transferred_bytes: u64, total_bytes: Option<u64>);
}

pub(crate) struct SilentObserver;

impl DownloadObserver for SilentObserver {
    fn is_cancelled(&self) -> bool {
        false
    }

    fn progress(&self, _transferred_bytes: u64, _total_bytes: Option<u64>) {}
}
