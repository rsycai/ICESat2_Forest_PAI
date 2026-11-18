use crate::domain::remote::Probe;
use crate::hya;
use crate::interfaces::DownloadObserver;
use crate::runtime::HyaDownloader;
use http::header::{COOKIE, HeaderMap};
use std::path::PathBuf;
use std::sync::Arc;
use std::time::Duration;
use xt_error::{Error, ErrorKind, Result};

impl HyaDownloader {
    pub(super) async fn run_backend(
        &self,
        probe: &Probe,
        partial: PathBuf,
        mut headers: HeaderMap,
        resume: bool,
        observer: &dyn DownloadObserver,
        lock: Arc<xt_lock::FileLock>,
    ) -> Result<()> {
        if !headers.contains_key(COOKIE)
            && let Some(cookie) = self.cookies.cookies(&probe.url)
        {
            headers.insert(COOKIE, cookie);
        }
        let transfer = hya::Transfer {
            settings: self.settings.clone(),
            client: self.hya_client.clone(),
            url: probe.url.clone(),
            headers,
            path: partial,
            size: probe.identity.size,
            ranges: probe.ranges,
            resume,
            asset_key: probe.identity.key.clone(),
            validator: probe
                .identity
                .etag
                .clone()
                .or_else(|| probe.identity.modified.clone()),
        };
        let cancel = self.cancel.child_token();
        let _cancel_on_drop = cancel.clone().drop_guard();
        let worker_cancel = cancel.clone();
        let (progress_tx, mut progress_rx) =
            tokio::sync::watch::channel((0u64, probe.identity.size));
        let mut worker = tokio::task::spawn_blocking(move || {
            let _lock = lock;
            let runtime = tokio::runtime::Builder::new_current_thread()
                .enable_all()
                .build()?;
            let result = runtime.block_on(hya::download(transfer, worker_cancel, progress_tx));
            drop(runtime);
            result
        });
        loop {
            tokio::select! {
                result = &mut worker => return result.map_err(|_| Error::new(ErrorKind::Internal, "Hya download worker failed"))?,
                _ = tokio::time::sleep(Duration::from_millis(100)) => {
                    if observer.is_cancelled() { cancel.cancel(); }
                    let (bytes, total) = *progress_rx.borrow_and_update();
                    observer.progress(bytes, total);
                }
            }
        }
    }
}
