use crate::registry::guard::LoadingGuard;
use crate::registry::registration::Registration;
use crate::registry::state::LOADING;
use std::any::{Any, TypeId, type_name};
use std::collections::{BTreeSet, HashMap};
use xt_error::{Error, ErrorKind, Result};

#[derive(Default)]
pub struct ConfigRegistry {
    pub(super) values: HashMap<TypeId, Box<dyn Any + Send + Sync>>,
}

impl ConfigRegistry {
    pub fn load() -> Result<Self> {
        if LOADING.replace(true) {
            return Err(Error::new(
                ErrorKind::Configuration,
                "Cannot recursively load configuration while loading a subconfiguration",
            ));
        }
        let _loading = LoadingGuard;
        let mut entries: Vec<_> = inventory::iter::<Registration>.into_iter().collect();
        entries.sort_by_key(|entry| (entry.prefix, entry.section, entry.name));
        let mut sections = BTreeSet::new();
        let mut registry = Self::default();
        for entry in &entries {
            if !sections.insert((entry.prefix, entry.section)) {
                return Err(Error::new(
                    ErrorKind::Configuration,
                    format!("Duplicate subconfiguration section: {}__{}", entry.prefix, entry.section),
                ));
            }
        }
        for entry in entries {
            let value = (entry.load)()
                .map_err(|error| error.context(format!("Load subconfiguration {}", entry.name)))?;
            let id = value.as_ref().type_id();
            if registry.values.insert(id, value).is_some() {
                return Err(Error::new(
                    ErrorKind::Configuration,
                    format!("Duplicate subconfiguration type registration: {}", entry.name),
                ));
            }
        }
        Ok(registry)
    }

    pub fn get<T: Send + Sync + 'static>(&self) -> Result<&T> {
        self.values
            .get(&TypeId::of::<T>())
            .and_then(|value| value.downcast_ref())
            .ok_or_else(|| {
                Error::new(
                    ErrorKind::Configuration,
                    format!("Unregistered subconfiguration: {}", type_name::<T>()),
                )
            })
    }
}
