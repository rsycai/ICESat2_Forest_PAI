#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[non_exhaustive]
pub enum DownloadError {
    Timeout,
    Connection,
    Dns,
    Tls,
    HttpStatus(u16),
    Interrupted,
    Incomplete,
    ChecksumMismatch,
    ResumeUnsupported,
}
