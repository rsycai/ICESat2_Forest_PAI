use crate::fs::paths::bounded_path;
use std::io;
use std::path::Path;

pub fn remove_file_and_empty_parents(
    path: impl AsRef<Path>,
    root: impl AsRef<Path>,
) -> io::Result<usize> {
    let (path, root) = bounded_path(path.as_ref(), root.as_ref())?;
    if path == root {
        return Err(io::Error::new(
            io::ErrorKind::InvalidInput,
            "Cannot delete the root directory",
        ));
    }
    match std::fs::remove_file(&path) {
        Ok(()) => (),
        Err(error) if error.kind() == io::ErrorKind::NotFound => (),
        Err(error) => return Err(error),
    }
    remove_empty_parents(path.parent().expect("File within boundary has a parent directory"), &root)
}

pub fn remove_empty_parents(
    directory: impl AsRef<Path>,
    root: impl AsRef<Path>,
) -> io::Result<usize> {
    let (mut directory, root) = bounded_path(directory.as_ref(), root.as_ref())?;
    let mut removed = 0;
    while directory != root {
        match std::fs::remove_dir(&directory) {
            Ok(()) => removed += 1,
            Err(error) if error.kind() == io::ErrorKind::NotFound => (),
            Err(error) if error.kind() == io::ErrorKind::DirectoryNotEmpty => break,
            Err(error) => return Err(error),
        }
        if !directory.pop() {
            break;
        }
    }
    Ok(removed)
}
