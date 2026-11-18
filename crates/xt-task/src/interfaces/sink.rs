use crate::model::DataGroup;
use std::{future::Future, pin::Pin};
use xt_error::Result;

pub trait GroupSink: Send + Sync {
    fn can_skip(
        &self,
        _group: DataGroup,
    ) -> Pin<Box<dyn Future<Output = Result<bool>> + Send + '_>> {
        Box::pin(async { Ok(false) })
    }
    fn accepts_unverified(&self) -> bool {
        false
    }
    fn submit(&self, group: DataGroup) -> Pin<Box<dyn Future<Output = Result<()>> + Send + '_>>;
}
