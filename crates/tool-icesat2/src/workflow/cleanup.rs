use std::path::Path;
use xt_error::{Error, ErrorKind, Result};
use xt_manifest::{ManifestStore, XmlValue};
use xt_task::model::{LocalFile, ProcessContext};

pub(super) fn delete_inputs(
    manifest: &Path,
    output: &Path,
    signature: &str,
    root: &Path,
    files: &[LocalFile],
    context: &ProcessContext,
) -> Result<()> {
    context.check_cancelled()?;
    let lock_root = manifest
        .parent()
        .ok_or_else(|| Error::new(ErrorKind::Configuration, "Manifest has no parent directory"))?
        .join(".locks");
    let _output_lock = xt_lock::FileLock::try_for_file(output, lock_root)?;
    if !crate::workflow::manifest::recovery::reusable(manifest, output, signature, context)? {
        return Err(Error::new(
            ErrorKind::Configuration,
            "Matched result verification failed; retaining original files",
        ));
    }
    let store = ManifestStore::<XmlValue>::new(manifest);
    let record = store
        .load()?
        .ok_or_else(|| Error::new(ErrorKind::Configuration, "Matching manifest is missing"))?;
    let mut targets = Vec::new();
    for file in files
        .iter()
        .filter(|f| matches!(f.role.as_str(), "ATL03" | "ATL08"))
    {
        let canonical = std::fs::canonicalize(&file.path)?;
        let canonical_root = std::fs::canonicalize(root)?;
        if !canonical.starts_with(&canonical_root) || canonical == canonical_root {
            return Err(Error::new(
                ErrorKind::InvalidArgument,
                "Original file is outside the download directory; refusing automatic deletion",
            ));
        }
        let local = xt_manifest::display_path(&canonical);
        let target = record
            .data
            .files
            .iter()
            .find(|f| f.local_path == local)
            .ok_or_else(|| {
                Error::new(
                    ErrorKind::Configuration,
                    "Original file has no manifest record; refusing automatic deletion",
                )
            })?;
        targets.push((target.product.clone(), file.path.clone()));
    }
    for (target, path) in targets {
        context.check_cancelled()?;
        store.delete_local_file(&target, &path, root)?;
        tracing::info!(target, path=%xt_manifest::display_path(&path), "Matching completed; original files deleted and manifest updated");
    }
    Ok(())
}
