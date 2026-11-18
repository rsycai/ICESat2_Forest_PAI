use std::path::PathBuf;
use xt_lock::FileLock;

pub struct DayLock {
    pub(super) _guard: FileLock,
    pub(super) day: PathBuf,
}
