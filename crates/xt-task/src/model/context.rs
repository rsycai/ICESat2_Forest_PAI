use tokio_util::sync::CancellationToken;
use xt_error::{Error, ErrorKind, Result};

#[derive(Clone, Default)]
pub struct ProcessContext {
    pub cancel: CancellationToken,
}

impl ProcessContext {
    pub fn check_cancelled(&self) -> Result<()> {
        if self.cancel.is_cancelled() {
            return Err(Error::new(ErrorKind::Cancelled, "Processing cancelled"));
        }
        Ok(())
    }
}
