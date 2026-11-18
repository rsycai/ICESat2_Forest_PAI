use crate::config::catalog::SourceDefinition;
use crate::domain::{SourceAdapter, SourceContext};
use crate::interfaces::SourceProvider;
use crate::providers::nasa::gedi::source::GediSource;
use crate::services::storage::DirectoryLayout;
use xt_error::Result;

pub struct GediProvider;
impl SourceProvider for GediProvider {
    const ID: &'static str = "gedi";
    const PROVIDER: &'static str = "nasa";
    fn validate(definition: &SourceDefinition) -> Result<()> {
        crate::providers::nasa::validation::validate_definition(definition, "GEDI")
    }
    fn create(context: SourceContext<'_>) -> Result<SourceAdapter> {
        let source =
            GediSource::from_definition(context.definition, context.product, context.release)?;
        Ok(SourceAdapter::single(
            context.definition,
            source,
            crate::providers::nasa::storage::NasaGranuleStorage {
                layout: DirectoryLayout::DateOnly,
            },
        ))
    }
}
crate::register_source!(GediProvider);
