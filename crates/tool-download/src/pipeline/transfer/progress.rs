use xt_transfer::{CancellationToken, DownloadObserver};

pub(super) struct Progress {
    pub(super) label: String,
    pub(super) remote_size: std::sync::Mutex<Option<u64>>,
    pub(super) enabled: bool,
    pub(super) cancel: CancellationToken,
    pub(super) last: std::sync::Mutex<std::time::Instant>,
}

impl DownloadObserver for Progress {
    fn is_cancelled(&self) -> bool {
        self.cancel.is_cancelled()
    }
    fn remote_size(&self, size: Option<u64>) {
        if let Ok(mut remote) = self.remote_size.lock() {
            *remote = size;
        }
    }
    fn progress(&self, bytes: u64, total: Option<u64>) {
        if !self.enabled {
            return;
        }
        if let Ok(mut last) = self.last.lock()
            && last.elapsed().as_secs() >= 2
        {
            tracing::info!(file = %self.label, bytes, total, "Download progress");
            *last = std::time::Instant::now();
        }
    }
}
