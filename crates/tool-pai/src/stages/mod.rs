use crate::{PaiConfig, Stage, filter::Filter, input, output};
use anyhow::{Context, Result};
use hdf5_pure::File;
use std::{
    io::Read,
    path::{Path, PathBuf},
};

pub fn run_stages(input: &Path, output: Option<&Path>, config: &PaiConfig) -> Result<Vec<PathBuf>> {
    run_with_manifest(input, output, config, None)
}

pub fn run_with_manifest(
    input: &Path,
    output: Option<&Path>,
    config: &PaiConfig,
    manifest: Option<&Path>,
) -> Result<Vec<PathBuf>> {
    execute(input, output, config, manifest, None)
}

pub fn run_workflow(
    input: &Path,
    products_root: &Path,
    config: &PaiConfig,
    manifest: &Path,
    overwrite: bool,
) -> Result<Vec<PathBuf>> {
    execute(
        input,
        None,
        config,
        Some(manifest),
        Some((products_root, overwrite)),
    )
}

fn execute(
    input: &Path,
    output: Option<&Path>,
    config: &PaiConfig,
    manifest: Option<&Path>,
    workflow: Option<(&Path, bool)>,
) -> Result<Vec<PathBuf>> {
    let (source, path, identity, _run_lock) =
        crate::manifest::resolve(input, manifest, config.stage)?;
    let granule = identity.id.clone();
    let products = if let Some(path) = output {
        output::paths::ProductPaths::explicit(path, config.stage)?
    } else {
        let root = match workflow {
            Some((root, _)) => root.to_owned(),
            None => {
                xt_config::paths::PathsConfig::new()?
                    .for_data("ATL03_08")?
                    .products
            }
        };
        let stem = if input
            .extension()
            .is_some_and(|s| s.eq_ignore_ascii_case("xml"))
        {
            &identity.id
        } else {
            input
                .file_stem()
                .and_then(|s| s.to_str())
                .context("Invalid input filename")?
        };
        output::paths::ProductPaths::archived(&root, &identity.date, stem)
    };
    let mut journal = crate::manifest::Journal::new(&path, identity);
    if let Some((_, overwrite)) = workflow {
        journal.configure_reuse(config, overwrite);
    }
    let input = source.as_path();
    let mut beams = None;
    if config.stage != Stage::V2 {
        journal.stage("pai_v1", "PAI_V1", input, &products.v1, |dest| {
            let rows = match load(input, config)? {
                Source::Segments(rows) => rows,
                Source::Product(_) => anyhow::bail!("Existing segmented PAI HDF5 can only be used for final filtering"),
            };
            output::write(dest, &rows)?;
            beams = Some(rows);
            Ok(())
        })?;
    }
    if config.stage != Stage::V1 {
        journal.stage("pai_v2", "PAI_V2", input, &products.v2, |dest| {
            let filter = Filter::open(
                config,
                if granule.starts_with("ATL03_08_") {
                    Path::new(&granule)
                } else {
                    input
                },
            )?;
            let mut rows = match beams.take() {
                Some(rows) => rows,
                None => match load(input, config)? {
                    Source::Segments(rows) => rows,
                    Source::Product(file) => return filter.product(&file, dest),
                },
            };
            filter.rows(&mut rows)?;
            output::write(dest, &rows)
        })?;
    }
    Ok(products.selected(config.stage))
}

enum Source {
    Segments(std::collections::BTreeMap<String, Vec<crate::model::Row>>),
    Product(File),
}

fn load(input: &Path, config: &PaiConfig) -> Result<Source> {
    let mut magic = [0; 8];
    let mut reader =
        std::fs::File::open(input).with_context(|| format!("Open {}", input.display()))?;
    let is_h5 = reader.read(&mut magic)? == 8 && magic == *b"\x89HDF\r\n\x1a\n";
    if is_h5 {
        let file = File::open_streaming(input)?;
        if !crate::local::is_matched(&file)? {
            return Ok(Source::Product(file));
        }
    }
    let g0 = crate::calculation::projection::open(&config.g0_raster)?;
    let rows = if is_h5 {
        input::matched::read(input, &g0)
    } else {
        input::legacy::read(input, &g0)
    };
    Ok(Source::Segments(rows.context("Read segment input")?))
}
