use crate::config::catalog::SourceDefinition;
use crate::{Discovery, StoragePolicy};
use std::sync::Arc;

pub struct SourceContext<'a> {
    pub definition: &'a SourceDefinition,
    pub product: Option<&'a str>,
    pub release: Option<&'a str>,
}

pub struct SourceAdapter {
    pub id: String,
    pub discovery: Box<dyn Discovery>,
    pub storage: Arc<dyn StoragePolicy>,
    pub authentication: Option<String>,
}

pub struct PipelineSource<'a> {
    pub id: &'a str,
    pub discovery: &'a dyn Discovery,
    pub storage: &'a dyn StoragePolicy,
}
impl SourceAdapter {
    pub fn single<S: crate::DataSource + 'static>(
        definition: &SourceDefinition,
        source: S,
        storage: impl StoragePolicy + 'static,
    ) -> Self {
        Self {
            id: definition.id.clone(),
            discovery: Box::new(crate::pipeline::discovery::SingleDiscovery::new(source)),
            storage: Arc::new(storage),
            authentication: definition.authentication.clone(),
        }
    }
    pub fn as_source(&self) -> PipelineSource<'_> {
        PipelineSource {
            id: &self.id,
            discovery: self.discovery.as_ref(),
            storage: self.storage.as_ref(),
        }
    }
}
