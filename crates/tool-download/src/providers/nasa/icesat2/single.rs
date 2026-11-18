use crate::providers::nasa::cmr::{CmrSource, impl_source};

pub struct Icesat2Source(pub(super) CmrSource);

impl Icesat2Source {
    pub fn new(product: &str, version: &str) -> crate::Result<Self> {
        Ok(Self(CmrSource::new("icesat2", product, version)?))
    }

    pub fn from_definition(
        definition: &crate::config::catalog::SourceDefinition,
        product: Option<&str>,
        version: Option<&str>,
    ) -> crate::Result<Self> {
        if definition.adapter_id() != "icesat2" {
            return Err(crate::domain::invalid("ICESat-2 data source definition required"));
        }
        crate::providers::nasa::validation::validate_definition(definition, "ATL")?;
        Ok(Self(CmrSource::from_definition(
            definition, product, version,
        )?))
    }
}

impl_source!(Icesat2Source);
