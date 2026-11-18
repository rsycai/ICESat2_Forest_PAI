use crate::error::context::Error;
use crate::error::result::Result;

pub trait ResultExt<T> {
    fn context(self, context: impl Into<String>) -> Result<T>;
    fn with_context(self, context: impl FnOnce() -> String) -> Result<T>;
}

impl<T, E: Into<Error>> ResultExt<T> for std::result::Result<T, E> {
    fn context(self, context: impl Into<String>) -> Result<T> {
        self.map_err(|error| error.into().context(context))
    }

    fn with_context(self, context: impl FnOnce() -> String) -> Result<T> {
        self.map_err(|error| error.into().context(context()))
    }
}
