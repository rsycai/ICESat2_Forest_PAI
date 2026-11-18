use crate::providers::nasa::cmr::{CmrSource, impl_source};

pub struct GediSource(pub(super) CmrSource);

impl GediSource {
    pub fn new(product: &str, version: &str) -> crate::Result<Self> {
        Ok(Self(CmrSource::new("gedi", product, version)?))
    }

    pub fn from_definition(
        definition: &crate::config::catalog::SourceDefinition,
        product: Option<&str>,
        version: Option<&str>,
    ) -> crate::Result<Self> {
        if definition.adapter_id() != "gedi" {
            return Err(crate::domain::invalid("GEDI data source definition required"));
        }
        crate::providers::nasa::validation::validate_definition(definition, "GEDI")?;
        Ok(Self(CmrSource::from_definition(
            definition, product, version,
        )?))
    }
}

impl_source!(GediSource);
