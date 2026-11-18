use crate::config_env::discovery::{discover, format};
use crate::config_env::error::configuration_error;
use crate::config_env::flatten;
use crate::config_env::model::EnvConfig;
use crate::config_env::naming::normalize;
use serde_json::Value;
use std::collections::BTreeMap;
use std::path::{Path, PathBuf};
use xt_error::Result;

impl EnvConfig {
    pub fn from_path(path: impl AsRef<Path>, prefix: &str) -> Result<Self> {
        let prefix = normalize(prefix)?;
        let mut files = Vec::new();
        discover(path.as_ref(), &mut files)?;
        files.sort();
        if files.is_empty() {
            return Err(configuration_error("No supported configuration file found"));
        }
        let mut builder = config::Config::builder();
        for file in &files {
            builder = builder.add_source(
                config::File::from(file.clone()).format(format(file).expect("Filtered configuration extension")),
            );
        }
        let settings = builder
            .build()
            .map_err(|e| configuration_error("Failed to read configuration file").with_source(e))?;
        Self::from_config(&settings, &prefix).map(|mut result| {
            result.files = files;
            result
        })
    }
    pub fn from_config(settings: &config::Config, prefix: &str) -> Result<Self> {
        let prefix = normalize(prefix)?;
        let tree: Value = settings
            .clone()
            .try_deserialize()
            .map_err(|e| configuration_error("Failed to convert in-memory configuration tree").with_source(e))?;
        if !tree.is_object() {
            return Err(configuration_error("Configuration root must be an object"));
        }
        let mut variables = BTreeMap::new();
        flatten(&tree, &prefix, &mut variables)?;
        Ok(Self {
            variables,
            files: Vec::new(),
        })
    }
    pub fn variables(&self) -> &BTreeMap<String, String> {
        &self.variables
    }
    pub fn files(&self) -> &[PathBuf] {
        &self.files
    }
}
