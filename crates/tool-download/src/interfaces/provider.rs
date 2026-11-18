use crate::config::catalog::SourceDefinition;
use crate::{Result, SourceAdapter, SourceContext};

pub trait SourceProvider: Send + Sync + 'static {
    const ID: &'static str;
    const PROVIDER: &'static str;
    fn validate(_definition: &SourceDefinition) -> Result<()> {
        Ok(())
    }
    fn create(context: SourceContext<'_>) -> Result<SourceAdapter>;
}
