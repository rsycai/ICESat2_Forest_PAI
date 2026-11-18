use crate::pipeline::manifest::{Journal, PlannedFile};
use xt_error::Result;
use xt_task::model::{DataGroup, LocalFile};

pub(super) struct Readiness {
    remaining: Vec<usize>,
    failed: Vec<bool>,
    skipped: Vec<bool>,
}

impl Readiness {
    pub(super) fn new(files: &[PlannedFile]) -> Self {
        let count = files.iter().map(|f| f.group + 1).max().unwrap_or(0);
        let mut remaining = vec![0; count];
        for file in files {
            remaining[file.group] += 1;
        }
        Self {
            remaining,
            failed: vec![false; count],
            skipped: vec![false; count],
        }
    }

    pub(super) fn finish(
        &mut self,
        index: usize,
        success: Option<bool>,
        files: &[PlannedFile],
        journal: &Journal,
    ) -> Result<Option<DataGroup>> {
        let group = files[index].group;
        self.remaining[group] -= 1;
        self.failed[group] |= success == Some(false);
        self.skipped[group] |= success.is_none();
        if self.remaining[group] != 0 {
            return Ok(None);
        }
        if self.failed[group] {
            tracing::warn!(group=%journal.entries[group].id, "Data group has failed downloads; processing was not submitted");
            return Ok(None);
        }
        if self.skipped[group] {
            tracing::info!(group=%journal.entries[group].id, "Existing file download skipped; processing skipped because allow_unverified=false");
            return Ok(None);
        }
        let entry = &journal.entries[group];
        let local = files
            .iter()
            .filter(|f| f.group == group)
            .map(|f| {
                let record = &entry.files[f.file];
                Ok(LocalFile {
                    id: record.product.clone(),
                    role: record.product.clone(),
                    path: f.task.destination.clone(),
                    size_bytes: std::fs::metadata(&f.task.destination)?.len(),
                })
            })
            .collect::<Result<Vec<_>>>()?;
        Ok(Some(DataGroup {
            id: entry.id.clone(),
            kind: entry.data_type.clone(),
            files: local,
        }))
    }
}
