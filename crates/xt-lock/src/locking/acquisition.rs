use crate::locking::guard::FileLock;
use same_file::Handle;
use std::{
    fs::{self, OpenOptions},
    io,
    path::Path,
    time::Duration,
};

impl FileLock {
    pub fn try_for_file(path: impl AsRef<Path>, directory: impl AsRef<Path>) -> io::Result<Self> {
        use sha2::{Digest, Sha256};
        let path = path.as_ref();
        let parent = path
            .parent()
            .filter(|p| !p.as_os_str().is_empty())
            .unwrap_or(Path::new("."));
        fs::create_dir_all(parent)?;
        let absolute = fs::canonicalize(parent)?.join(
            path.file_name()
                .ok_or_else(|| io::Error::new(io::ErrorKind::InvalidInput, "Lock target must be a file"))?,
        );
        let identity = absolute.to_str().ok_or_else(|| {
            io::Error::new(io::ErrorKind::InvalidInput, "File lock path must be valid Unicode")
        })?;
        #[cfg(windows)]
        let identity = identity.to_lowercase();
        let key = Sha256::digest(identity.as_bytes())
            .iter()
            .map(|byte| format!("{byte:02x}"))
            .collect::<String>();
        let directory = directory.as_ref();
        let root = directory
            .parent()
            .filter(|p| !p.as_os_str().is_empty())
            .unwrap_or(Path::new("."));
        Self::try_acquire(directory.join(format!("{key}.xt.lock")))?.with_cleanup_root(root)
    }

    pub fn acquire(path: impl AsRef<Path>) -> io::Result<Self> {
        Self::open(path.as_ref(), true)
    }

    pub fn try_acquire(path: impl AsRef<Path>) -> io::Result<Self> {
        Self::open(path.as_ref(), false)
    }

    fn open(path: &Path, wait: bool) -> io::Result<Self> {
        let path = std::path::absolute(path)?;
        let path = path.as_path();
        for attempt in 0..128 {
            if let Some(parent) = path.parent() {
                match fs::create_dir_all(parent) {
                    Ok(()) => (),
                    Err(error)
                        if matches!(
                            error.kind(),
                            io::ErrorKind::NotFound
                                | io::ErrorKind::AlreadyExists
                                | io::ErrorKind::PermissionDenied
                        ) && attempt < 127 =>
                    {
                        std::thread::sleep(Duration::from_millis(1));
                        continue;
                    }
                    Err(error) => return Err(error),
                }
            }
            if fs::symlink_metadata(path).is_ok_and(|meta| !meta.file_type().is_file()) {
                return Err(io::Error::new(
                    io::ErrorKind::InvalidInput,
                    "Lock path must be a regular file",
                ));
            }
            let file = match OpenOptions::new()
                .create(true)
                .truncate(false)
                .read(true)
                .write(true)
                .open(path)
            {
                Ok(file) => file,
                Err(error)
                    if matches!(
                        error.kind(),
                        io::ErrorKind::PermissionDenied | io::ErrorKind::NotFound
                    ) && attempt < 127 =>
                {
                    std::thread::sleep(Duration::from_millis(1));
                    continue;
                }
                Err(error) => return Err(error),
            };
            if wait {
                fs2::FileExt::lock_exclusive(&file)?;
            } else {
                fs2::FileExt::try_lock_exclusive(&file).map_err(|error| {
                    if error.kind() == io::ErrorKind::WouldBlock
                        || error.raw_os_error() == fs2::lock_contended_error().raw_os_error()
                    {
                        io::Error::new(io::ErrorKind::WouldBlock, error)
                    } else {
                        error
                    }
                })?;
            }
            let handle = Handle::from_file(file)?;
            if Handle::from_path(path).is_ok_and(|current| current == handle) {
                return Ok(Self {
                    handle: Some(handle),
                    path: path.to_path_buf(),
                    cleanup_root: None,
                });
            }
            drop(handle);
        }
        Err(io::Error::new(
            io::ErrorKind::WouldBlock,
            "Lock file keeps changing; retry",
        ))
    }
}
