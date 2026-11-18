use crate::services::transport::DownloadClient;
use xt_transfer::{
    Asset, DownloadFuture, DownloadObserver, DownloadOptions, DownloadOutcome, DownloadTask,
    FileDownloadConfig, FileDownloader, download_file_with_client,
};

impl FileDownloader for DownloadClient {
    fn remote_size<'a>(&'a self, asset: &'a Asset) -> DownloadFuture<'a, Option<u64>> {
        self.download.remote_size(asset)
    }
    fn download<'a>(
        &'a self,
        task: &'a DownloadTask,
        options: DownloadOptions,
        observer: &'a dyn DownloadObserver,
    ) -> DownloadFuture<'a, DownloadOutcome> {
        Box::pin(download_file_with_client(
            &self.download,
            &task.asset.url,
            &task.destination,
            FileDownloadConfig {
                options,
                size_bytes: task.asset.size_bytes,
                checksum: task.asset.checksum.clone(),
                observer: Some(observer),
            },
        ))
    }
}
