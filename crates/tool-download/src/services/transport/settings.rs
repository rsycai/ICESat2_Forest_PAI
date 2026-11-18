use std::time::Duration;
use xt_error::{Error, ErrorKind, Result};

#[derive(Clone)]
pub struct RequestSettings {
    pub retries: u8,
    pub max_retry_delay: Duration,
}
impl Default for RequestSettings {
    fn default() -> Self {
        Self {
            retries: 3,
            max_retry_delay: Duration::from_secs(120),
        }
    }
}
impl RequestSettings {
    pub fn validate(&self) -> Result<()> {
        if self.retries > 20 || self.max_retry_delay > Duration::from_secs(86400) {
            return Err(Error::new(
                ErrorKind::InvalidArgument,
                "Search retry parameters are out of range",
            ));
        }
        Ok(())
    }
}
