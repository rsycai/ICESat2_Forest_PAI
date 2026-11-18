use xt_error::{Error, ErrorKind};

pub(super) fn cancelled() -> Error {
    Error::new(ErrorKind::Cancelled, "Download cancelled; manifest and resumable files were retained")
}
