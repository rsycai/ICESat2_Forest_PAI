use clap::ValueEnum;
use std::str::FromStr;

#[derive(Debug, Clone, Copy, PartialEq, Eq, ValueEnum)]
pub enum Stage {
    V1,
    V2,
    Both,
}

impl FromStr for Stage {
    type Err = String;
    fn from_str(value: &str) -> Result<Self, Self::Err> {
        <Self as ValueEnum>::from_str(value, false)
    }
}

#[derive(Debug, Clone, xt_config::EnvLoad)]
#[env(prefix = "XT", section = "pai")]
pub struct PaiConfig {
    #[env(default = Stage::V1)]
    pub stage: Stage,
    #[env(os, default = "data/LIA/G0_500m.tif")]
    pub g0_raster: std::path::PathBuf,
    #[env(default = "tests/data/landcover.tif")]
    pub landcover: String,
    #[env(json, default = vec![1, 2, 3, 4, 5])]
    pub landcover_classes: Vec<u8>,
    #[env(default = 0)]
    pub qa_min: i8,
    #[env(default = 5)]
    pub qa_max: i8,
}
