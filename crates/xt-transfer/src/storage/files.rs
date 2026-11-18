use std::fs;
use std::path::{Path, PathBuf};
use xt_error::{Error, ErrorKind, Result};

pub(crate) fn check_file(path: &Path) -> Result<bool> {
    match fs::symlink_metadata(path) {
        Ok(meta) if meta.file_type().is_file() => Ok(true),
        Ok(_) => Err(Error::new(
            ErrorKind::InvalidArgument,
            "Download target or temporary file is not a regular file",
        )),
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => Ok(false),
        Err(e) => Err(e.into()),
    }
}

pub(crate) fn remove_if_exists(path: &Path) -> Result<()> {
    if check_file(path)? {
        fs::remove_file(path)?;
    }
    Ok(())
}

pub fn append_suffix(path: &Path, suffix: &str) -> PathBuf {
    let mut name = path.as_os_str().to_os_string();
    name.push(suffix);
    name.into()
}
