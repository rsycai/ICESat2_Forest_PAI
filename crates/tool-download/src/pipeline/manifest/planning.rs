use crate::domain::{DataGroup, SearchQuery};
use crate::interfaces::StoragePolicy;
use crate::pipeline::manifest::naming::record_file_name;
use crate::pipeline::manifest::schema::DownloadDetails;
use crate::pipeline::manifest::state::{DataEntry, Journal, ManifestFile, PlannedFile};
use crate::services::storage::paths::{original_file_name, safe_component};
use std::collections::HashSet;
use std::fs;
use std::sync::atomic::{AtomicU64, Ordering};
use xt_error::Result;
use xt_transfer::DownloadTask;

impl Journal {
    pub fn new(
        paths: &xt_config::paths::PathsConfig,
        query: &SearchQuery,
        groups: Vec<DataGroup>,
        storage: &dyn StoragePolicy,
    ) -> Result<(Self, Vec<PlannedFile>)> {
        paths.validate()?;
        fs::create_dir_all(&paths.root)?;
        let paths = xt_config::paths::PathsConfig {
            root: fs::canonicalize(&paths.root)?,
            ..paths.clone()
        };
        static RUN: AtomicU64 = AtomicU64::new(0);
        let now = chrono::Utc::now();
        let run_id = format!(
            "{}-{}-{}",
            now.format("%Y%m%dT%H%M%S%.9fZ"),
            std::process::id(),
            RUN.fetch_add(1, Ordering::Relaxed)
        );
        let mut journal = Self {
            skipped_groups: HashSet::new(),
            index_paths: std::collections::BTreeMap::new(),
            paths,
            run_id,
            generated_at: now.to_rfc3339(),
            bbox: query.bbox.map(|b| b.parameter()),
            temporal: query.time.as_ref().map(|t| t.parameter()),
            entries: vec![],
            file_states: vec![],
            record_paths: vec![],
        };
        let mut planned = Vec::new();
        let mut seen = HashSet::new();
        let mut destinations = HashSet::new();
        for group in groups {
            let identity = storage.identity(&group)?;
            let relative = storage.directory(&identity)?;
            if relative.components().any(|component| match component {
                std::path::Component::Normal(name) => name
                    .to_string_lossy()
                    .chars()
                    .any(|c| c.is_control() || matches!(c, '\\' | ':')),
                _ => true,
            }) {
                return Err(crate::domain::invalid(
                    "Storage policy must return a normal relative path within the root",
                ));
            }
            if identity.id.is_empty()
                || identity.data_type.is_empty()
                || safe_component(&identity.data_type) != identity.data_type
            {
                return Err(crate::domain::invalid("Storage policy returned an invalid data identity"));
            }
            let expected_files: usize = group.granules.iter().map(|g| g.assets.len()).sum();
            let paired = identity.paired;
            let data_type = identity.data_type;
            let acquired = identity.acquired_at;
            let date = acquired
                .as_ref()
                .and_then(|v| chrono::DateTime::parse_from_rfc3339(v).ok())
                .map(|time| {
                    time.with_timezone(&chrono::Utc)
                        .format("%Y-%m-%d")
                        .to_string()
                })
                .unwrap_or_else(|| "unknown".into());
            let folder_date = if date == "unknown" {
                "undated".into()
            } else {
                date.replace('-', "/")
            };
            let id = identity.id;
            let directories = journal.paths.for_data(&data_type)?;
            let directory = directories.downloads.join(relative);
            journal.index_paths.insert(
                data_type.clone(),
                directories.manifests.join("manifest.xml"),
            );
            let group_index = journal.entries.len();
            let mut entry = DataEntry {
                id,
                paired,
                data_type,
                date,
                status: vec![],
                files: vec![],
            };
            let mut local_names = HashSet::new();
            let mut product_counts = std::collections::BTreeMap::<String, usize>::new();
            for granule in &group.granules {
                *product_counts.entry(granule.product.clone()).or_default() += granule.assets.len();
            }
            for granule in group.granules {
                let time = storage.acquired_at(&granule);
                for asset in granule.assets {
                    let key = xt_transfer::asset_key(&asset.url)?;
                    if !seen.insert(key.clone()) {
                        continue;
                    }
                    let url = reqwest::Url::parse(&asset.url)
                        .map_err(|_| crate::domain::invalid("Invalid asset URL"))?;
                    let original_name = original_file_name(&url)?;
                    let name = original_name.to_owned();
                    if !local_names.insert(name.to_ascii_lowercase()) {
                        return Err(crate::domain::invalid(
                            "Assets in the same directory share a filename and cannot be distinguished",
                        ));
                    }
                    let destination = directory.join(&name);
                    destination
                        .to_str()
                        .ok_or_else(|| crate::domain::invalid("Manifest does not support non-Unicode local paths"))?;
                    let local_path = crate::services::storage::paths::display(&destination);
                    if !destinations.insert(local_path.to_lowercase()) {
                        return Err(crate::domain::invalid(
                            "Assets in the same directory share a filename and cannot be distinguished",
                        ));
                    }
                    let record = ManifestFile {
                        product: {
                            let base = xt_manifest::store::reference_token(&granule.product);
                            let base = if base.is_empty() {
                                "file".to_owned()
                            } else {
                                base
                            };
                            if product_counts.get(&granule.product).copied().unwrap_or(0) > 1 {
                                format!("{}:{}", base, xt_manifest::store::reference_token(&name))
                            } else {
                                base
                            }
                        },
                        tool: "xt-download".into(),
                        operation: Some("download".into()),
                        updated_at: Some(chrono::Utc::now().to_rfc3339()),
                        name: original_name.to_owned(),
                        local_path,
                        size_bytes: None,
                        details: DownloadDetails {
                            granule_id: granule.id.clone(),
                            local_name: name,
                            acquired_at: time.clone(),
                            expected_size_bytes: asset.size_bytes,
                            catalog_size_mb: granule.catalog_size_mb.clone(),
                            size_status: if asset.size_bytes.is_some() {
                                "metadata"
                            } else {
                                "unknown"
                            }
                            .into(),
                            url: asset.url.clone(),
                            ..Default::default()
                        },
                    };
                    planned.push(PlannedFile {
                        task: DownloadTask { asset, destination },
                        group: group_index,
                        file: entry.files.len(),
                        title: granule.title.clone(),
                    });
                    entry.files.push(record);
                }
            }
            if entry.files.is_empty() {
                continue;
            }
            if paired && entry.files.len() != expected_files {
                return Err(crate::domain::invalid("Duplicate or missing files in matching group"));
            }
            let record_name = record_file_name(&entry)?;
            journal
                .record_paths
                .push(directories.manifests.join(&folder_date).join(record_name));
            entry.status.push(xt_manifest::OperationStatus::new(
                "download",
                xt_manifest::TaskState::Pending,
                entry
                    .files
                    .iter()
                    .map(|file| file.product.as_str())
                    .collect::<Vec<_>>()
                    .join(" "),
            ));
            journal
                .file_states
                .push(vec![xt_manifest::TaskState::Pending; entry.files.len()]);
            journal.entries.push(entry);
        }
        Ok((journal, planned))
    }
    pub fn initialize_records(&mut self) -> Result<()> {
        for group in 0..self.entries.len() {
            if self.skipped_groups.contains(&group) {
                continue;
            }
            for file in &mut self.entries[group].files {
                file.touch();
            }
            self.file_states[group].fill(xt_manifest::TaskState::Running);
            self.persist_group(group)?;
        }
        Ok(())
    }
}
