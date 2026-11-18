use super::Details;
use anyhow::{Context, Result, ensure};
use std::path::Path;
use xt_manifest::{DataEntry, FileRecord, ManifestStore, OperationStatus, TaskState, XmlValue};

pub(crate) struct Journal {
    store: ManifestStore<XmlValue>,
    identity: DataEntry,
    reuse: Option<(crate::PaiConfig, bool)>,
}

impl Journal {
    pub(crate) fn new(path: &Path, mut identity: DataEntry) -> Self {
        identity.status.clear();
        identity.files.clear();
        Self {
            store: ManifestStore::new(path),
            identity,
            reuse: None,
        }
    }

    pub(crate) fn configure_reuse(&mut self, config: &crate::PaiConfig, overwrite: bool) {
        self.reuse = Some((config.clone(), overwrite));
    }

    pub(crate) fn stage(
        &self,
        op: &str,
        product: &str,
        input: &Path,
        output: &Path,
        calculate: impl FnOnce(&Path) -> Result<()>,
    ) -> Result<()> {
        let parent = output
            .parent()
            .filter(|p| !p.as_os_str().is_empty())
            .unwrap_or(Path::new("."));
        ensure!(
            !output
                .extension()
                .is_some_and(|s| s.eq_ignore_ascii_case("xml")),
            "PAI output must not be an XML manifest"
        );
        let _lock = xt_lock::FileLock::try_for_file(output, parent.join(".xt-locks"))?;
        let name = output
            .file_name()
            .and_then(|s| s.to_str())
            .context("Invalid result filename")?;
        let local = xt_manifest::display_path(&parent.canonicalize()?.join(name));
        let source = std::path::absolute(input)?;
        ensure!(
            input
                .canonicalize()
                .ok()
                .zip(output.canonicalize().ok())
                .is_none_or(|(a, b)| a != b),
            "Input and output must differ"
        );
        let signature = self.reuse.as_ref().and_then(|(config, _)| {
            super::signature::signature(config, op, input, &self.identity.date)
        });
        let mut details = Details {
            input_path: xt_manifest::display_path(&source),
            signature,
            output_modified: None,
        };
        let old = self.store.load()?;
        let previous = old.as_ref().and_then(|r| {
            r.data.files.iter().find(|f| {
                f.local_path == local && f.tool == "xt-pai" && f.operation.as_deref() == Some(op)
            })
        });
        let exists = output.try_exists()?;
        ensure!(
            !exists || previous.is_some(),
            "Existing output is not registered for this operation; choose another path: {}",
            output.display()
        );
        if exists
            && matches!(self.reuse, Some((_, false)))
            && let (Some(record), Some(file)) = (&old, previous)
            && record.data.status.iter().any(|s| {
                s.id == op
                    && s.state == TaskState::Completed
                    && s.targets().contains(file.product.as_str())
            })
            && !record.data.status.iter().any(|s| {
                s.state == TaskState::Deleted && s.targets().contains(file.product.as_str())
            })
            && file.size_bytes == Some(output.metadata()?.len())
            && let Ok(saved) = file.details.decode::<Details>()
            && details.signature.is_some()
            && saved.signature == details.signature
            && saved.output_modified.is_some()
            && saved.output_modified == super::signature::modified(output)
        {
            tracing::info!(operation=op, path=%output.display(), "Reusing completed PAI product");
            return Ok(());
        }
        let mut file = FileRecord {
            product: product.into(),
            tool: "xt-pai".into(),
            operation: Some(op.into()),
            name: name.into(),
            local_path: local,
            updated_at: None,
            size_bytes: previous.and_then(|f| f.size_bytes),
            details: XmlValue::encode(&details)?,
        };
        let mut data = self.identity.clone();
        data.status
            .push(OperationStatus::new(op, TaskState::Running, product));
        let record = self.store.upsert(data, file.clone())?;
        let target = record
            .data
            .files
            .iter()
            .find(|f| {
                f.local_path == file.local_path
                    && f.tool == file.tool
                    && f.operation == file.operation
            })
            .context("Manifest lacks the operation output")?
            .product
            .clone();
        let result = (|| -> Result<()> {
            if exists {
                let dir = tempfile::tempdir_in(parent)?;
                let tmp = dir.path().join("result.h5");
                calculate(&tmp)?;
                std::fs::rename(tmp, output)?;
            } else {
                calculate(output)?;
            }
            file.size_bytes = Some(output.metadata()?.len());
            if self.reuse.is_some() {
                details.output_modified = super::signature::modified(output);
                file.details = XmlValue::encode(&details)?;
            }
            Ok(())
        })();
        let mut data = self.identity.clone();
        if let Err(error) = result {
            let mut status = OperationStatus::new(op, TaskState::Failed, &target);
            status.error_code = Some("PAI_PROCESSING".into());
            data.status.push(status);
            self.store
                .update(data)
                .with_context(|| format!("Failed to record failure status; computation error: {error:#}"))?;
            return Err(error);
        }
        data.status
            .push(OperationStatus::new(op, TaskState::Completed, &target));
        file.product = target;
        self.store.upsert(data, file)?;
        Ok(())
    }
}
