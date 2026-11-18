use crate::{Error, ErrorKind};

pub(crate) fn cancelled() -> Error {
    Error::new(ErrorKind::Cancelled, "Download cancelled; incomplete files were retained")
}
