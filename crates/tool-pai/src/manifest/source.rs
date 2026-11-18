use super::Details;
use crate::Stage;
use crate::input::Granule;
use anyhow::{Context, Result, ensure};
use std::path::{Path, PathBuf};
use xt_manifest::{DataEntry, ManifestStore, TaskState, XmlValue};

fn lock_record(path: &Path) -> Result<xt_lock::FileLock> {
    Ok(xt_lock::FileLock::try_for_file(
        path,
        path.with_file_name(".pai-locks"),
    )?)
}

fn from_record(data: &DataEntry, stage: Stage) -> Result<PathBuf> {
    let completed = |op: &str| -> Result<Option<PathBuf>> {
        let mut files = data.files.iter().filter(|f| {
            f.operation.as_deref() == Some(op)
                && data.status.iter().any(|s| {
                    s.id == op
                        && s.state == TaskState::Completed
                        && s.targets().contains(f.product.as_str())
                })
                && !data.status.iter().any(|s| {
                    s.id == "delete"
                        && s.state == TaskState::Deleted
                        && s.targets().contains(f.product.as_str())
                })
                && Path::new(&f.local_path).is_file()
        });
        let Some(first) = files.next() else {
            return Ok(None);
        };
        let Some(second) = files.next() else {
            return Ok(Some(first.local_path.clone().into()));
        };
        if op == "pai_v1" {
            let dirs = xt_config::paths::PathsConfig::new()?.for_data("ATL03_08")?;
            let expected =
                crate::output::paths::ProductPaths::archived(&dirs.products, &data.date, &data.id)
                    .v1;
            if let Ok(expected) = expected.canonicalize() {
                for file in [first, second].into_iter().chain(files) {
                    if Path::new(&file.local_path)
                        .canonicalize()
                        .is_ok_and(|p| p == expected)
                    {
                        return Ok(Some(file.local_path.clone().into()));
                    }
                }
            }
        }
        anyhow::bail!("Manifest contains multiple {op} results; specify the input file and --manifest")
    };
    if stage == Stage::V2
        && let Some(path) = completed("pai_v1")?
    {
        return Ok(path);
    }
    if let Some(path) = completed("match")? {
        return Ok(path);
    }
    let mut sources = std::collections::BTreeSet::new();
    for file in data.files.iter().filter(|f| {
        f.tool == "xt-pai" && (stage == Stage::V2 || f.operation.as_deref() == Some("pai_v1"))
    }) {
        let details: Details = file.details.decode()?;
        let path = PathBuf::from(details.input_path);
        if path.is_file() {
            sources.insert(path);
        }
    }
    ensure!(
        sources.len() == 1,
        "Manifest does not uniquely identify the PAI input; specify a data file and --manifest"
    );
    Ok(sources.pop_first().unwrap())
}

pub(crate) fn resolve(
    input: &Path,
    explicit: Option<&Path>,
    stage: Stage,
) -> Result<(PathBuf, PathBuf, DataEntry, xt_lock::FileLock)> {
    let is_xml = input
        .extension()
        .is_some_and(|s| s.eq_ignore_ascii_case("xml"));
    if is_xml {
        ensure!(explicit.is_none(), "Input is already a manifest; do not specify --manifest");
        let lock = lock_record(input)?;
        let record = ManifestStore::<XmlValue>::new(input)
            .load()?
            .context("Input detail XML does not exist")?;
        return Ok((
            from_record(&record.data, stage)?,
            input.into(),
            record.data,
            lock,
        ));
    }
    let stem = input
        .file_stem()
        .and_then(|s| s.to_str())
        .context("Invalid input filename")?;
    let known = Granule::parse(stem);
    let (id, date) = known
        .as_ref()
        .map(|g| (g.id.clone(), g.date.to_string()))
        .unwrap_or_else(|| (stem.to_owned(), "undated".into()));
    let data_type = if known.is_some() { "ATL03_08" } else { "PAI" };
    let path = if let Some(p) = explicit {
        p.to_owned()
    } else if known.is_some() {
        let dirs = xt_config::paths::PathsConfig::new()?.for_data(data_type)?;
        dirs.manifests
            .join(date.replace('-', "/"))
            .join(format!("{id}.xml"))
    } else {
        input
            .with_file_name("manifests")
            .join("undated")
            .join(format!("{id}.xml"))
    };
    let lock = lock_record(&path)?;
    let data = if let Some(record) = ManifestStore::<XmlValue>::new(&path).load()? {
        ensure!(
            known.is_none()
                || (record.data.id == id
                    && record.data.date == date
                    && record.data.data_type == data_type),
            "Specified manifest does not match the input granule"
        );
        record.data
    } else {
        DataEntry {
            id,
            paired: known.is_some(),
            data_type: data_type.into(),
            date,
            status: vec![],
            files: vec![],
        }
    };
    Ok((input.into(), path, data, lock))
}
