use crate::pipeline::manifest::state::{Journal, PlannedFile};
use std::fs;
use xt_error::Result;
use xt_transfer::DownloadOutcome;

impl Journal {
    pub fn set_size(&mut self, planned: &PlannedFile, result: &Result<Option<u64>>) {
        let record = &mut self.entries[planned.group].files[planned.file];
        match result {
            Ok(Some(size)) => {
                record.expected_size_bytes = Some(*size);
                record.size_status = "remote".into();
            }
            Ok(None) => {}
            Err(error) => record.size_error_code = Some(error.code().into()),
        }
        record.touch();
    }
    pub fn outcome(
        &mut self,
        planned: &PlannedFile,
        result: &Result<DownloadOutcome>,
    ) -> Result<()> {
        let file = &mut self.entries[planned.group].files[planned.file];
        file.size_bytes = fs::metadata(&planned.task.destination)
            .ok()
            .filter(|m| m.is_file())
            .map(|m| m.len());
        self.file_states[planned.group][planned.file] = match result {
            Ok(DownloadOutcome::Completed { bytes }) => {
                file.size_bytes = Some(*bytes);
                xt_manifest::TaskState::Completed
            }
            Ok(DownloadOutcome::Skipped) => xt_manifest::TaskState::Unverified,
            Err(error) => {
                if error.code() == "CANCELLED" {
                    xt_manifest::TaskState::Cancelled
                } else {
                    xt_manifest::TaskState::Failed
                }
            }
        };
        file.touch();
        if let Err(error) = result {
            self.entries[planned.group].status[0].error_code = Some(error.code().into());
        }
        self.persist_group(planned.group)
    }
    pub fn cancel_pending(&mut self) -> Result<()> {
        for index in 0..self.entries.len() {
            if self.skipped_groups.contains(&index) {
                continue;
            }
            let mut changed = false;
            for (file, state) in self.entries[index]
                .files
                .iter_mut()
                .zip(&mut self.file_states[index])
            {
                if matches!(
                    state,
                    xt_manifest::TaskState::Pending | xt_manifest::TaskState::Running
                ) {
                    *state = xt_manifest::TaskState::Cancelled;
                    file.touch();
                    changed = true;
                }
            }
            if changed {
                self.persist_group(index)?;
            }
        }
        Ok(())
    }
}
