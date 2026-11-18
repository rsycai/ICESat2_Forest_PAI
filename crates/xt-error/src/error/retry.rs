use std::time::Duration;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RetryAdvice {
    DoNotRetry,
    Backoff,
    After(Duration),
}
