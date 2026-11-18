mod product;

use crate::{PaiConfig, model::Row};
use anyhow::{Context, Result, ensure};
use std::{
    collections::BTreeMap,
    path::Path,
    sync::{Arc, LazyLock},
};
use xt_core::geotiff::{GeoTiffCache, GeoTiffSampler};

static LANDCOVER: LazyLock<GeoTiffCache> = LazyLock::new(GeoTiffCache::default);

pub(crate) struct Filter<'a> {
    raster: Arc<GeoTiffSampler>,
    config: &'a PaiConfig,
}

impl<'a> Filter<'a> {
    pub fn open(config: &'a PaiConfig, input: &Path) -> Result<Self> {
        ensure!(
            0 <= config.qa_min && config.qa_min <= config.qa_max && config.qa_max <= 9,
            "QA range must satisfy 0 <= qa_min <= qa_max <= 9"
        );
        ensure!(
            !config.landcover_classes.is_empty(),
            "landcover_classes must not be empty"
        );
        let mut path = config.landcover.clone();
        if path.contains("{year}") {
            let stem = input
                .file_stem()
                .and_then(|s| s.to_str())
                .context("Invalid input filename")?;
            let timestamp = stem
                .split('_')
                .find(|s| s.len() == 14 && s.bytes().all(|b| b.is_ascii_digit()))
                .context("Filename lacks a 14-digit acquisition timestamp; use --landcover to select the annual TIFF")?;
            path = path.replace("{year}", &timestamp[..4]);
        }
        let raster = LANDCOVER
            .open(&path)
            .with_context(|| format!("Open landcover raster {path}"))?;
        Ok(Self { raster, config })
    }

    fn indices(&self, rows: impl Iterator<Item = (i8, bool, f32, f32)>) -> Result<Vec<usize>> {
        let mut indices = Vec::new();
        let mut points = Vec::new();
        for (i, (qa, valid, lon, lat)) in rows.enumerate() {
            if valid
                && (self.config.qa_min..=self.config.qa_max).contains(&qa)
                && (-180.0..=180.0).contains(&lon)
                && (-90.0..=90.0).contains(&lat)
            {
                indices.push(i);
                points.push((f64::from(lon), f64::from(lat)));
            }
        }
        let classes = self.raster.sample_many::<u8>(&points)?;
        Ok(indices
            .into_iter()
            .zip(classes)
            .filter_map(|(i, c)| {
                c.filter(|id| self.config.landcover_classes.contains(id))
                    .map(|_| i)
            })
            .collect())
    }

    pub fn rows(&self, beams: &mut BTreeMap<String, Vec<Row>>) -> Result<()> {
        for rows in beams.values_mut() {
            let keep = self.indices(rows.iter().map(|r| {
                (
                    r.qa,
                    r.status == 0 && r.pai.is_finite() && r.pai > 0.0,
                    r.loc.lon,
                    r.loc.lat,
                )
            }))?;
            let mut keep = keep.into_iter().peekable();
            let mut i = 0;
            rows.retain(|_| {
                let retain = keep.next_if_eq(&i).is_some();
                i += 1;
                retain
            });
        }
        Ok(())
    }
}
