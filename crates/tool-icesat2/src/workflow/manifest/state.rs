use crate::workflow::manifest::model::MatchDetails;
use std::path::{Path, PathBuf};
use xt_error::{Error, ErrorKind, Result};
use xt_manifest::store::completed_file;
use xt_manifest::{DataEntry, FileRecord, ManifestStore, OperationStatus, TaskState, XmlValue};

pub(crate) struct MatchJournal {
    store: ManifestStore<XmlValue>,
    identity: DataEntry<XmlValue>,
    pub(crate) path: PathBuf,
    output: PathBuf,
    target: String,
}

impl MatchJournal {
    pub(crate) fn begin(atl03: &Path, atl08: &Path, output: &Path, path: PathBuf) -> Result<Self> {
        let invalid = |message| Error::new(ErrorKind::Configuration, message);
        let name = |path: &Path| {
            path.file_name()
                .and_then(|v| v.to_str())
                .map(str::to_owned)
                .ok_or_else(|| invalid("Filename must be UTF-8"))
        };
        let a = name(atl03)?;
        let b = name(atl08)?;
        let stem = xt_icesat2_meta::manifest::pair_stem(&a, &b)?;
        let stamp = a.split('_').nth(1).ok_or_else(|| invalid("Missing acquisition time"))?;
        let date = chrono::NaiveDateTime::parse_from_str(stamp, "%Y%m%d%H%M%S")
            .map_err(|_| invalid("Invalid acquisition time"))?
            .date()
            .to_string();
        let store = ManifestStore::<XmlValue>::new(&path);
        let parent = output
            .parent()
            .filter(|p| !p.as_os_str().is_empty())
            .unwrap_or(Path::new("."));
        let local = xt_manifest::display_path(
            &std::fs::canonicalize(parent)?.join(
                output
                    .file_name()
                    .ok_or_else(|| invalid("Invalid result filename"))?,
            ),
        );
        let existing = store.load()?;
        let previous = existing.as_ref().and_then(|record| {
            record.data.files.iter().find(|f| {
                f.local_path == local
                    && f.tool == "xt-icesat2"
                    && f.operation.as_deref() == Some("match")
            })
        });
        let target = previous.map_or_else(|| "ATL03_08".to_owned(), |f| f.product.clone());
        let files = if previous.is_some() {
            vec![]
        } else {
            vec![FileRecord {
                product: target.clone(),
                tool: "xt-icesat2".into(),
                operation: Some("match".into()),
                updated_at: None,
                name: name(output)?,
                local_path: local.clone(),
                size_bytes: None,
                details: XmlValue::Fields(vec![]),
            }]
        };
        let mut identity = DataEntry {
            id: stem,
            paired: true,
            data_type: "ATL03_08".into(),
            date,
            status: vec![OperationStatus::new("match", TaskState::Running, &target)],
            files,
        };
        let record = store.update(identity.clone())?;
        let target = record
            .data
            .files
            .iter()
            .find(|f| {
                f.tool == "xt-icesat2"
                    && f.operation.as_deref() == Some("match")
                    && f.local_path == local
            })
            .ok_or_else(|| invalid("Matched result file reference is missing"))?
            .product
            .clone();
        identity.files.clear();
        identity.status.clear();
        Ok(Self {
            store,
            identity,
            path,
            output: output.to_path_buf(),
            target,
        })
    }

    pub(crate) fn complete(
        &self,
        photons: u64,
        segments: u64,
        signature: String,
        output_sha256: String,
    ) -> Result<()> {
        let mut data = self.identity.clone();
        data.status = vec![OperationStatus::new(
            "match",
            TaskState::Completed,
            &self.target,
        )];
        self.store.upsert(
            data,
            completed_file(
                &self.target,
                "xt-icesat2",
                "match",
                &self.output,
                XmlValue::encode(&MatchDetails {
                    photons,
                    segments,
                    workflow_signature: Some(signature),
                    output_sha256: Some(output_sha256),
                })?,
            )?,
        )?;
        Ok(())
    }

    pub(crate) fn fail(&self, error: &Error) -> Result<()> {
        let mut data = self.identity.clone();
        let mut status = OperationStatus::new(
            "match",
            if error.kind() == ErrorKind::Cancelled {
                TaskState::Cancelled
            } else {
                TaskState::Failed
            },
            &self.target,
        );
        status.error_code = Some(error.code().into());
        data.status = vec![status];
        self.store.update(data)?;
        Ok(())
    }
}
