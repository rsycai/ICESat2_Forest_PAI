pub(crate) fn invalid(message: impl Into<String>) -> xt_error::Error {
    xt_error::Error::new(xt_error::ErrorKind::Configuration, message)
}
