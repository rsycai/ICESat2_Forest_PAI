//! Single final PAI product API.
mod calculation;
mod config;
mod filter;
mod input;
mod local;
mod manifest;
mod model;
mod output;
mod stages;
pub(crate) use config::{PaiConfig, Stage};
use std::path::{Path, PathBuf};
use anyhow::Result;

/// Final-product filtering options. G(0) is fixed at 0.5.
pub struct ProductConfig {
    pub landcover: String,
    pub landcover_classes: Vec<u8>,
    pub qa_min: i8,
    pub qa_max: i8,
}
impl ProductConfig {
    pub fn new() -> Result<Self> {
        let c = PaiConfig::new()?;
        Ok(Self { landcover:c.landcover, landcover_classes:c.landcover_classes, qa_min:c.qa_min, qa_max:c.qa_max })
    }
}
/// Calculate and filter directly into one final product.
pub fn run_product(input: &Path, output: &Path, config: &ProductConfig, manifest: &Path) -> Result<PathBuf> {
    let internal = PaiConfig { stage:Stage::V2, g0_raster:PathBuf::new(), landcover:config.landcover.clone(), landcover_classes:config.landcover_classes.clone(), qa_min:config.qa_min, qa_max:config.qa_max };
    stages::run_with_manifest(input, Some(output), &internal, Some(manifest))?;
    Ok(output.to_owned())
}
