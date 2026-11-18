use crate::model::{DataGroup, ProcessContext};
use xt_error::Result;

pub trait Processor: Send + Sync {
    fn can_skip(&self, _group: &DataGroup, _context: &ProcessContext) -> Result<bool> {
        Ok(false)
    }
    fn id(&self) -> &str;
    fn validate(&self, group: &DataGroup) -> Result<()>;
    fn process(&self, group: DataGroup, context: &ProcessContext) -> Result<DataGroup>;
}
