use crate::config::catalog::{AuthenticationProfile, DownloadCatalog, SourceDefinition};
use crate::domain::invalid;
use crate::services::registry::registrations::{AuthenticationRegistration, SourceRegistration};
use crate::{Authentication, Result, SourceAdapter, SourceContext};
use std::collections::BTreeMap;

pub fn registered_sources() -> Result<BTreeMap<&'static str, &'static SourceRegistration>> {
    let mut entries = BTreeMap::new();
    for entry in inventory::iter::<SourceRegistration> {
        if entries.insert(entry.id, entry).is_some() {
            return Err(invalid("Duplicate data source adapter registration"));
        }
    }
    Ok(entries)
}

fn source_registration(definition: &SourceDefinition) -> Result<&'static SourceRegistration> {
    registered_sources()?
        .get(definition.adapter_id())
        .copied()
        .ok_or_else(|| {
            invalid(&format!(
                "Unregistered data source adapter: {}",
                definition.adapter_id()
            ))
        })
}

pub fn validate_source(definition: &SourceDefinition) -> Result<()> {
    (source_registration(definition)?.validate)(definition)
}

pub fn create(
    catalog: &DownloadCatalog,
    id: &str,
    product: Option<&str>,
    release: Option<&str>,
) -> Result<SourceAdapter> {
    let definition = catalog.source(id)?;
    let entry = source_registration(definition)?;
    (entry.validate)(definition)?;
    let adapter = (entry.create)(SourceContext {
        definition,
        product,
        release,
    })?;
    if adapter.id != definition.id {
        return Err(invalid("Adapter data source ID does not match the catalog"));
    }
    if let Some(profile) = &adapter.authentication {
        catalog.authentication(profile)?;
    }
    Ok(adapter)
}

fn authentication_registration(id: &str) -> Result<&'static AuthenticationRegistration> {
    let mut entries = BTreeMap::new();
    for entry in inventory::iter::<AuthenticationRegistration> {
        if entries.insert(entry.id, entry).is_some() {
            return Err(invalid("Duplicate authentication adapter registration"));
        }
    }
    entries
        .get(id)
        .copied()
        .ok_or_else(|| invalid(&format!("Unregistered authentication adapter: {id}")))
}

pub fn validate_authentication(profile: &AuthenticationProfile) -> Result<()> {
    authentication_registration(&profile.adapter).map(|_| ())
}

pub fn authentication(profile: &AuthenticationProfile) -> Result<Box<dyn Authentication>> {
    (authentication_registration(&profile.adapter)?.create)(profile)
}
