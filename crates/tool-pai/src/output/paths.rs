use crate::{Stage, input::Granule};
use anyhow::{Context, Result};
use std::path::{Path, PathBuf};

pub(crate) struct ProductPaths {
    pub v1: PathBuf,
    pub v2: PathBuf,
}

impl ProductPaths {
    pub fn archived(root: &Path, date: &str, stem: &str) -> Self {
        let release = Granule::parse(stem).map_or("unknown", |g| g.release);
        let root = root.join(release);
        let suffix = Granule::suffix(stem).unwrap_or(stem);
        let stem = format!("PAI_{suffix}");
        let day = date.replace('-', "/");
        Self {
            v1: root.join("v01").join(&day).join(format!("{stem}.h5")),
            v2: root.join("v02").join(day).join(v2_name(&stem)),
        }
    }

    pub fn explicit(path: &Path, stage: Stage) -> Result<Self> {
        let mut paths = Self {
            v1: path.into(),
            v2: path.into(),
        };
        if stage == Stage::Both {
            let stem = path
                .file_stem()
                .and_then(|s| s.to_str())
                .context("Invalid output filename")?;
            paths.v2 = path.with_file_name(v2_name(stem));
            let same_file = paths.v1 == paths.v2
                || (cfg!(windows)
                    && path
                        .file_name()
                        .zip(paths.v2.file_name())
                        .is_some_and(|(a, b)| a.eq_ignore_ascii_case(b)));
            anyhow::ensure!(
                !same_file,
                "Intermediate and final output paths collide; specify a different output name"
            );
        }
        Ok(paths)
    }

    pub fn selected(&self, stage: Stage) -> Vec<PathBuf> {
        match stage {
            Stage::V1 => vec![self.v1.clone()],
            Stage::V2 => vec![self.v2.clone()],
            Stage::Both => vec![self.v1.clone(), self.v2.clone()],
        }
    }
}

fn v2_name(stem: &str) -> String {
    let base = stem
        .rsplit_once('_')
        .filter(|(_, tail)| tail.len() == 2 && tail.bytes().all(|b| b.is_ascii_digit()))
        .map_or(stem, |(prefix, _)| prefix);
    format!("{base}_02.h5")
}
