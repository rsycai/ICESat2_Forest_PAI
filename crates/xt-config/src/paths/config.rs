use crate::paths::data::DataPaths;
use crate::paths::validation::component;
use std::path::PathBuf;

#[derive(Debug, Clone, crate::EnvLoad, crate::SubConfig)]
#[env(
    prefix = "XT",
    section = "paths",
    validate = "validate",
    crate_path = "crate"
)]
pub struct PathsConfig {
    #[env(default = "data", os)]
    pub root: PathBuf,
    #[env(default = "downloads")]
    pub downloads: String,
    #[env(default = "processed")]
    pub processed: String,
    #[env(default = "products")]
    pub products: String,
    #[env(default = "manifests")]
    pub manifests: String,
}

impl PathsConfig {
    pub fn validate(&self) -> crate::Result<()> {
        if self.root.as_os_str().is_empty() {
            return Err(crate::Error::new(
                crate::ErrorKind::Configuration,
                "paths.root must not be empty",
            ));
        }
        let mut names = std::collections::HashSet::new();
        for name in [
            &self.downloads,
            &self.processed,
            &self.products,
            &self.manifests,
        ] {
            component(name)?;
            if !names.insert(name.to_ascii_lowercase()) {
                return Err(crate::Error::new(
                    crate::ErrorKind::Configuration,
                    "Download, processed, product, and manifest directories must have distinct names",
                ));
            }
        }
        Ok(())
    }

    pub fn for_data(&self, name: &str) -> crate::Result<DataPaths> {
        self.validate()?;
        component(name)?;
        let root = self.root.join(name);
        Ok(DataPaths {
            downloads: root.join(&self.downloads),
            processed: root.join(&self.processed),
            products: root.join(&self.products),
            manifests: root.join(&self.manifests),
            root,
        })
    }
}
