use crate::config::catalog::SourceDefinition;
use crate::domain::{SourceAdapter, SourceContext};
use crate::interfaces::SourceProvider;
use crate::providers::nasa::icesat2::pair::Icesat2PairSource;
use crate::providers::nasa::icesat2::single::Icesat2Source;
use crate::providers::nasa::icesat2::storage::PairedStorage;
use std::sync::Arc;
use xt_error::Result;

pub struct Icesat2Provider;
impl SourceProvider for Icesat2Provider {
    const ID: &'static str = "icesat2";
    const PROVIDER: &'static str = "nasa";
    fn validate(definition: &SourceDefinition) -> Result<()> {
        crate::providers::nasa::validation::validate_definition(definition, "ATL")
    }
    fn create(context: SourceContext<'_>) -> Result<SourceAdapter> {
        if let Some(products) = context.product.filter(|value| value.contains(',')) {
            let mut names: Vec<_> = products
                .split(',')
                .map(|p| p.trim().to_ascii_uppercase())
                .collect();
            names.sort();
            if names != ["ATL03", "ATL08"] {
                return Err(crate::domain::invalid("ICESat-2 pairing supports ATL03 and ATL08 only"));
            }
            Ok(SourceAdapter {
                id: context.definition.id.clone(),
                discovery: Box::new(Icesat2PairSource::from_definition(
                    context.definition,
                    context.release,
                )?),
                storage: Arc::new(PairedStorage),
                authentication: context.definition.authentication.clone(),
            })
        } else {
            let source = Icesat2Source::from_definition(
                context.definition,
                context.product,
                context.release,
            )?;
            Ok(SourceAdapter::single(
                context.definition,
                source,
                crate::providers::nasa::storage::NasaGranuleStorage::default(),
            ))
        }
    }
}
crate::register_source!(Icesat2Provider);
