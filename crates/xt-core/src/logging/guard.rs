use tracing_appender::non_blocking::WorkerGuard;

#[must_use = "Keep the log guard in main scope to flush logs before exit"]
pub struct LogGuard {
    pub(super) _worker: WorkerGuard,
}
