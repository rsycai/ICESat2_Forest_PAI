use std::io;
use std::path::{Component, Path, PathBuf};

pub fn bounded_path(path: &Path, root: &Path) -> io::Result<(PathBuf, PathBuf)> {
    let root = std::fs::canonicalize(root)?;
    if !root.is_dir() {
        return Err(io::Error::new(
            io::ErrorKind::InvalidInput,
            "Deletion boundary must be a directory",
        ));
    }
    if path.components().any(|part| part == Component::ParentDir) {
        return Err(io::Error::new(
            io::ErrorKind::InvalidInput,
            "Deletion path must not contain parent traversal",
        ));
    }
    let absolute = std::path::absolute(path)?;
    let mut existing = absolute.as_path();
    let mut tail = Vec::new();
    loop {
        match std::fs::symlink_metadata(existing) {
            Ok(metadata) => {
                if metadata.file_type().is_symlink() {
                    return Err(io::Error::new(
                        io::ErrorKind::InvalidInput,
                        "Deletion does not accept symbolic links",
                    ));
                }
                break;
            }
            Err(error) if error.kind() == io::ErrorKind::NotFound => {
                tail.push(
                    existing
                        .file_name()
                        .ok_or_else(|| io::Error::new(io::ErrorKind::InvalidInput, "Invalid deletion path"))?
                        .to_os_string(),
                );
                existing = existing.parent().ok_or_else(|| {
                    io::Error::new(io::ErrorKind::InvalidInput, "Deletion path has no parent directory")
                })?;
            }
            Err(error) => return Err(error),
        }
    }
    let mut resolved = std::fs::canonicalize(existing)?;
    for name in tail.into_iter().rev() {
        resolved.push(name);
    }
    if !resolved.starts_with(&root) {
        return Err(io::Error::new(
            io::ErrorKind::InvalidInput,
            "Deletion path is outside the specified root",
        ));
    }
    Ok((resolved, root))
}
