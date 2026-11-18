use crate::pipeline::manifest::{Journal, PlannedFile};
use xt_error::Result;
use xt_task::interfaces::GroupSink;
use xt_task::model::{DataGroup, LocalFile};

pub(super) async fn skip_completed(
    journal: &mut Journal,
    files: &[PlannedFile],
    sink: &dyn GroupSink,
) -> Result<u64> {
    let mut skipped = 0;
    for (index, entry) in journal.entries.iter().enumerate() {
        let local = files
            .iter()
            .filter(|file| file.group == index)
            .map(|file| {
                let record = &entry.files[file.file];
                LocalFile {
                    id: record.product.clone(),
                    role: record.product.clone(),
                    path: file.task.destination.clone(),
                    size_bytes: file.task.asset.size_bytes.unwrap_or(0),
                }
            })
            .collect();
        if sink
            .can_skip(DataGroup {
                id: entry.id.clone(),
                kind: entry.data_type.clone(),
                files: local,
            })
            .await?
        {
            journal.skipped_groups.insert(index);
            skipped += entry.files.len() as u64;
            tracing::info!(group=%entry.id, "Matched result verified; skipping original downloads");
        }
    }
    Ok(skipped)
}
