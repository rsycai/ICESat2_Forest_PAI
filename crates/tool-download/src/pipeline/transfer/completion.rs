use std::time::Duration;
use xt_error::Result;
use xt_transfer::DownloadOutcome;

pub(super) struct Completion {
    pub index: usize,
    pub result: Result<DownloadOutcome>,
    pub remote_size: Option<u64>,
    pub elapsed: Duration,
}
