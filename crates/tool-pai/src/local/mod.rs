use crate::input::Granule;
use anyhow::{Context, Result};
use hdf5_pure::File;
use std::path::{Path, PathBuf};
use xt_config::paths::DataPaths;

pub struct LocalInput {
    pub source: PathBuf,
    pub manifest: PathBuf,
    products: crate::output::paths::ProductPaths,
}

pub(crate) fn is_matched(file: &File) -> Result<bool> {
    Ok(file
        .group("/")?
        .groups()?
        .iter()
        .any(|name| name == "ATL03_08"))
}

pub fn acquisition_date(path: &Path) -> Option<String> {
    Granule::parse(path.file_stem()?.to_str()?).map(|g| g.date.to_string())
}

impl LocalInput {
    pub fn inspect(source: &Path, paths: &DataPaths) -> Result<Option<Self>> {
        let Some(extension) = source.extension() else {
            return Ok(None);
        };
        let legacy = extension.eq_ignore_ascii_case("br");
        if !legacy {
            if !extension.eq_ignore_ascii_case("h5") {
                return Ok(None);
            }
            let file = File::open_streaming(source).context("Failed to read HDF5 directory")?;
            if !is_matched(&file)? {
                return Ok(None);
            }
        }
        let stem = source
            .file_stem()
            .and_then(|s| s.to_str())
            .context("Invalid input name")?;
        let (id, date) = Granule::parse(stem)
            .map(|g| (g.id, g.date.to_string()))
            .unwrap_or_else(|| (format!("local_{stem}"), "undated".into()));
        let products = crate::output::paths::ProductPaths::archived(&paths.products, &date, stem);
        Ok(Some(Self {
            source: source.into(),
            manifest: paths
                .manifests
                .join(date.replace('-', "/"))
                .join(format!("{id}.xml")),
            products,
        }))
    }

    pub fn outputs(&self, stage: crate::Stage) -> Vec<PathBuf> {
        self.products.selected(stage)
    }
}



