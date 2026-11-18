use same_file::Handle;
use std::{
    fs, io,
    path::{Path, PathBuf},
};

pub struct FileLock {
    pub(super) handle: Option<Handle>,
    pub(super) path: PathBuf,
    pub(super) cleanup_root: Option<PathBuf>,
}

impl FileLock {
    pub fn with_cleanup_root(mut self, root: impl AsRef<Path>) -> io::Result<Self> {
        let root = fs::canonicalize(root)?;
        let parent = fs::canonicalize(self.path.parent().expect("Lock path has a parent directory"))?;
        if !parent.starts_with(&root) {
            return Err(io::Error::new(
                io::ErrorKind::InvalidInput,
                "Lock file is outside the cleanup root",
            ));
        }
        self.cleanup_root = Some(root);
        Ok(self)
    }
}

impl Drop for FileLock {
    fn drop(&mut self) {
        let Some(handle) = self.handle.take() else {
            return;
        };
        if Handle::from_path(&self.path).is_ok_and(|current| current == handle)
            && let Err(error) = fs::remove_file(&self.path)
            && error.kind() != std::io::ErrorKind::NotFound
        {
            tracing::warn!(path = %self.path.display(), error = %error, "Failed to remove lock file");
        }
        let _ = fs2::FileExt::unlock(handle.as_file());
        drop(handle);
        if let Some(root) = &self.cleanup_root
            && let Some(parent) = self.path.parent()
            && let Err(error) = xt_core::fs::remove_empty_parents(parent, root)
        {
            tracing::warn!(path = %parent.display(), error = %error, "Failed to remove empty lock directory");
        }
    }
}
