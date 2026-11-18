use std::io;
use xt_error::{DownloadError, Error, ErrorKind};

pub(crate) fn transfer_error(error: io::Error) -> Error {
    let message = error.to_string();
    for (prefix, suffix) in [
        ("unexpected status ", " for a range request"),
        ("server ignored the Range request and answered ", ""),
    ] {
        if let Some(status) = message
            .strip_prefix(prefix)
            .and_then(|s| s.strip_suffix(suffix))
            .and_then(|s| s.parse::<u16>().ok())
            .filter(|s| (400..600).contains(s))
        {
            return Error::new(
                ErrorKind::Download(DownloadError::HttpStatus(status)),
                format!("Hya transfer returned HTTP {status}"),
            );
        }
    }
    let kind = match error.kind() {
        io::ErrorKind::TimedOut => ErrorKind::Download(DownloadError::Timeout),
        io::ErrorKind::InvalidInput => ErrorKind::InvalidArgument,
        io::ErrorKind::InvalidData | io::ErrorKind::UnexpectedEof => {
            ErrorKind::Download(DownloadError::Incomplete)
        }
        _ => ErrorKind::Download(DownloadError::Interrupted),
    };
    let reason = if message.starts_with("server ignored Range")
        || message.starts_with("server ignored the Range")
    {
        "Server did not honor the Range request"
    } else if message.starts_with("206 Content-Range")
        || message == "206 without a parsable Content-Range"
    {
        "Missing server Content-Range or mismatched start offset"
    } else if message.starts_with("server answered with Content-Encoding:") {
        "Server returned compressed encoding; cannot download original byte ranges"
    } else if message.starts_with("server redirected mid-transfer")
        || message.starts_with("redirect:")
    {
        "Redirect during transfer; obtain a new download URL"
    } else if message.starts_with("redirect loop:")
        || message.starts_with("redirect budget exhausted")
        || message.starts_with("unusable redirect target:")
    {
        "Redirect loop, limit exceeded, or invalid URL"
    } else if message.contains("no header terminator") {
        "Server returned incomplete HTTP response headers"
    } else if message.starts_with("bad chunk size") || message == "chunk not terminated by CRLF" {
        "Invalid HTTP chunked response"
    } else if error.kind() == io::ErrorKind::UnexpectedEof {
        "Connection closed before the full response was received"
    } else if error.kind() == io::ErrorKind::InvalidData {
        "Response or protocol validation failed; this is not an existing-file condition"
    } else {
        "Network or file I/O failed"
    };
    Error::new(
        kind,
        format!("Hya transfer failed; category: {:?}; reason: {reason}", error.kind()),
    )
}
