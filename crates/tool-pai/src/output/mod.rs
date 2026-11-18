mod metadata;
pub(crate) mod paths;

use crate::model::Row;
use anyhow::Result;
use hdf5_pure::{FileBuilder, LibVer};
use std::{collections::BTreeMap, path::Path};

pub(super) fn write(path: &Path, beams: &BTreeMap<String, Vec<Row>>) -> Result<()> {
    let mut builder = FileBuilder::new();
    builder.with_libver_bounds(LibVer::Earliest, LibVer::V18);
    for (beam, rows) in beams {
        let mut group = builder.create_group(beam);
        macro_rules! column {
            ($name:literal, $get:expr) => {{
                let values: Vec<_> = rows.iter().map($get).collect();
                let dataset = group.create_dataset($name);
                dataset.with_data(&values);
                metadata::apply(dataset, $name);
            }};
        }
        column!("longitude", |r| r.loc.lon);
        column!("latitude", |r| r.loc.lat);
        column!("segment_id_beg", |r| r.loc.begin);
        column!("segment_id_end", |r| r.loc.end);
        column!("delta_time", |r| r.loc.time);
        column!("PAI", |r| r.pai);
        column!("PAIe", |r| r.effective);
        column!("CI", |r| r.ci);
        column!("gap_f", |r| r.gap);
        column!("QA", |r| r.qa);
        column!("pai_status", |r| r.status);
        builder.add_group(group.finish());
    }
    publish(path, builder)
}

pub(crate) fn publish(path: &Path, builder: FileBuilder) -> Result<()> {
    let parent = path
        .parent()
        .filter(|p| !p.as_os_str().is_empty())
        .unwrap_or(Path::new("."));
    std::fs::create_dir_all(parent)?;
    let mut tmp = tempfile::NamedTempFile::new_in(parent)?;
    builder.finish_to(tmp.as_file_mut())?;
    tmp.persist_noclobber(path)?;
    Ok(())
}
