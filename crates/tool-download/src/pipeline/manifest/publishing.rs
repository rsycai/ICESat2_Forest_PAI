use crate::pipeline::manifest::naming::record_file_name;
use crate::pipeline::manifest::schema::{CatalogState, DayReference, ManifestIndex};
use crate::pipeline::manifest::state::{DataEntry, DataManifest, Journal};
use std::collections::{BTreeMap, HashSet};
use std::fs;
use std::path::{Path, PathBuf};
use xt_error::Result;
use xt_manifest::{DailyIndex, DataReference};

impl Journal {
    pub fn publish_catalog(&self, status: CatalogState) -> Result<()> {
        let _catalog_locks = self
            .index_paths
            .values()
            .map(|path| {
                let parent = path
                    .parent()
                    .ok_or_else(|| crate::domain::invalid("Master manifest has no parent directory"))?;
                xt_manifest::lock_day(parent)
            })
            .collect::<Result<Vec<_>>>()?;
        let mut days = BTreeMap::<(String, String), Vec<DataEntry>>::new();
        for (group, entry) in self.entries.iter().enumerate() {
            if self.skipped_groups.contains(&group) {
                continue;
            }
            days.entry((entry.data_type.clone(), entry.date.clone()))
                .or_default()
                .push(entry.clone());
        }
        let mut references = Vec::new();
        for ((data_type, date), entries) in days {
            let manifest_root = self.paths.for_data(&data_type)?.manifests;
            let folder_date = if date == "unknown" {
                "undated".into()
            } else {
                date.replace('-', "/")
            };
            let folder = PathBuf::from(folder_date);
            let relative = folder.join("index.xml");
            let day_lock = xt_manifest::lock_day(&manifest_root.join(&folder))?;
            let mut records = BTreeMap::<String, DataReference>::new();
            match fs::read_to_string(manifest_root.join(&relative)) {
                Ok(text) => {
                    let previous: DailyIndex = quick_xml::de::from_str(&text)
                        .map_err(|_| crate::domain::invalid("Invalid existing daily index"))?;
                    if previous.date != date || previous.data_type != data_type {
                        return Err(crate::domain::invalid("Existing daily index does not match the directory"));
                    }
                    for reference in previous.entries {
                        let path = Path::new(&reference.path);
                        if path.components().count() != 1
                            || !matches!(
                                path.components().next(),
                                Some(std::path::Component::Normal(_))
                            )
                            || !reference.path.ends_with(".xml")
                            || reference.path.eq_ignore_ascii_case("index.xml")
                            || reference.path.contains('/')
                            || reference.path.contains(char::from(92))
                        {
                            return Err(crate::domain::invalid("Invalid file reference in existing daily index"));
                        }
                        if !manifest_root.join(&folder).join(path).is_file() {
                            return Err(crate::domain::invalid("XML referenced by the existing daily index does not exist"));
                        }
                        if records.insert(reference.id.clone(), reference).is_some() {
                            return Err(crate::domain::invalid("Duplicate data IDs in existing daily index"));
                        }
                    }
                }
                Err(error) if error.kind() == std::io::ErrorKind::NotFound => {}
                Err(error) => return Err(error.into()),
            }
            let mut names = HashSet::new();
            let mut current = Vec::new();
            for entry in entries {
                let name = record_file_name(&entry)?;
                records.insert(
                    entry.id.clone(),
                    DataReference {
                        id: entry.id.clone(),
                        paired: entry.paired,
                        path: name.clone(),
                        files: entry.files.len(),
                        status: entry.status.clone(),
                        updated_at: None,
                    },
                );
                current.push((name, entry));
            }
            for reference in records.values() {
                if !names.insert(reference.path.to_ascii_lowercase()) {
                    return Err(crate::domain::invalid("XML filename collision for data ID"));
                }
            }
            for (name, data) in current {
                xt_manifest::ManifestStore::new(manifest_root.join(&folder).join(name))
                    .merge_locked(
                        &day_lock,
                        DataManifest {
                            run_id: self.run_id.clone(),
                            generated_at: self.generated_at.clone(),
                            data,
                        },
                    )?;
            }
            let saved_index: DailyIndex = xt_manifest::read_xml(&manifest_root.join(&relative))?;
            let entries = saved_index.entries;
            let files = entries.iter().map(|entry| entry.files).sum();
            references.push(DayReference {
                date: date.clone(),
                data_type: data_type.clone(),
                path: relative.to_string_lossy().into_owned(),
                entries: entries.len(),
                files,
            });
        }
        for (data_type, index_path) in &self.index_paths {
            let days: Vec<_> = references
                .iter()
                .filter(|day| &day.data_type == data_type)
                .cloned()
                .collect();
            crate::pipeline::manifest::catalog::merge_catalog(
                index_path,
                ManifestIndex {
                    schema_version: 2,
                    run_id: self.run_id.clone(),
                    generated_at: self.generated_at.clone(),
                    status,
                    files: days.iter().map(|day| day.files).sum(),
                    selected_files: self
                        .entries
                        .iter()
                        .filter(|entry| &entry.data_type == data_type)
                        .map(|entry| entry.files.len())
                        .sum(),
                    bounding_box: self.bbox.clone(),
                    temporal: self.temporal.clone(),
                    days,
                },
            )?;
        }
        Ok(())
    }
    pub(super) fn persist_group(&mut self, group: usize) -> Result<()> {
        let entry = &mut self.entries[group];
        use xt_manifest::TaskState;
        let states = &self.file_states[group];
        let state = if states.contains(&TaskState::Failed) {
            TaskState::Failed
        } else if states.contains(&TaskState::Cancelled) {
            TaskState::Cancelled
        } else if states.iter().all(|s| *s == TaskState::Completed) {
            TaskState::Completed
        } else if states
            .iter()
            .all(|s| matches!(s, TaskState::Completed | TaskState::Unverified))
        {
            TaskState::Unverified
        } else if states.iter().any(|s| *s != TaskState::Pending) {
            TaskState::Running
        } else {
            TaskState::Pending
        };
        let status = &mut entry.status[0];
        status.state = state;
        status.updated_at = entry
            .files
            .iter()
            .filter_map(|f| f.updated_at.clone())
            .max_by_key(|text| chrono::DateTime::parse_from_rfc3339(text).ok());
        xt_manifest::ManifestStore::new(&self.record_paths[group]).merge(DataManifest {
            run_id: self.run_id.clone(),
            generated_at: self.generated_at.clone(),
            data: entry.clone(),
        })?;
        Ok(())
    }
}
