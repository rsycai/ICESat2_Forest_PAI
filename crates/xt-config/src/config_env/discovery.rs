use std::fs;
use std::path::{Path, PathBuf};
use xt_error::{Result, ResultExt};

pub(super) fn format(path: &Path) -> Option<config::FileFormat> {
    match path.extension()?.to_str()?.to_ascii_lowercase().as_str() {
        "toml" => Some(config::FileFormat::Toml),
        "json" => Some(config::FileFormat::Json),
        "yaml" | "yml" => Some(config::FileFormat::Yaml),
        "ini" => Some(config::FileFormat::Ini),
        "ron" => Some(config::FileFormat::Ron),
        "json5" => Some(config::FileFormat::Json5),
        _ => None,
    }
}

pub(super) fn discover(path: &Path, files: &mut Vec<PathBuf>) -> Result<()> {
    let metadata =
        fs::symlink_metadata(path).with_context(|| format!("Check configuration path {}", path.display()))?;
    if metadata.file_type().is_symlink() {
        return Ok(());
    }
    if metadata.is_dir() {
        for entry in fs::read_dir(path).context("Scan configuration directory")? {
            discover(&entry.context("Read configuration directory entry")?.path(), files)?;
        }
    } else if metadata.is_file() && format(path).is_some() {
        files.push(path.to_owned());
    }
    Ok(())
}
