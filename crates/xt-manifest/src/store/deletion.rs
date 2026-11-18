use crate::error::invalid;
use crate::store::status::put_status;
use crate::store::{ManifestStore, lock_day, persist_record};
use crate::{OperationStatus, TaskState, XmlValue};
use std::path::Path;
use xt_error::{Error, Result};

impl ManifestStore<XmlValue> {
    pub fn delete_local_file(&self, target: &str, expected: &Path, root: &Path) -> Result<()> {
        let (path, root) = xt_core::fs::bounded_path(expected, root)?;
        if path == root {
            return Err(invalid("Cannot delete the data root directory"));
        }
        let parent = path.parent().ok_or_else(|| invalid("File to delete has no parent directory"))?;
        let _file_lock = xt_lock::FileLock::try_for_file(&path, parent.join(".xt-locks"))?
            .with_cleanup_root(&root)?;
        let _day_lock = lock_day(self.day()?)?;
        let mut record = self
            .load()?
            .ok_or_else(|| invalid("A manifest must exist before deleting files"))?;
        let file = record
            .data
            .files
            .iter()
            .find(|f| f.product == target)
            .ok_or_else(|| invalid("Deletion target is not listed in the manifest"))?;
        let (recorded, _) = xt_core::fs::bounded_path(Path::new(&file.local_path), &root)?;
        if recorded != path {
            return Err(invalid("Deletion path does not match the manifest record"));
        }
        let mut status = OperationStatus::new("delete", TaskState::Running, target);
        status.touch();
        put_status(&mut record.data.status, status.clone());
        persist_record(&self.path, &record)?;
        let result = xt_core::fs::remove_file_and_empty_parents(&path, &root).map_err(Error::from);
        status.state = if std::fs::symlink_metadata(&path)
            .is_err_and(|error| error.kind() == std::io::ErrorKind::NotFound)
        {
            TaskState::Deleted
        } else {
            TaskState::Failed
        };
        status.error_code = result.as_ref().err().map(|e| e.code().to_owned());
        status.touch();
        if status.state == TaskState::Deleted {
            record
                .data
                .files
                .iter_mut()
                .find(|f| f.product == target)
                .expect("Validated file record")
                .touch();
        }
        put_status(&mut record.data.status, status);
        persist_record(&self.path, &record)?;
        result.map(|_| ())
    }
}
