use crate::error::invalid;
use crate::model::{DailyIndex, DataManifest, DataReference};
use crate::store::codec::{read_xml, write_xml};
use crate::store::paths::parent_dir;
use serde::Serialize;
use std::path::Path;
use xt_error::Result;

pub fn persist_record<T: Serialize>(path: &Path, record: &DataManifest<T>) -> Result<()> {
    crate::store::validation::validate_data(&record.data)?;
    let day = parent_dir(path);
    let index_path = day.join("index.xml");
    let mut index = if index_path.exists() {
        read_xml::<DailyIndex>(&index_path)?
    } else {
        DailyIndex {
            run_id: record.run_id.clone(),
            date: record.data.date.clone(),
            data_type: record.data.data_type.clone(),
            entries: vec![],
        }
    };
    if index.date != record.data.date || index.data_type != record.data.data_type {
        return Err(invalid("Daily index and detail data type or date mismatch"));
    }
    let reference = DataReference {
        id: record.data.id.clone(),
        paired: record.data.paired,
        path: path
            .file_name()
            .and_then(|s| s.to_str())
            .ok_or_else(|| invalid("XML filename must be UTF-8"))?
            .into(),
        files: record.data.files.len(),
        status: record.data.status.clone(),
        updated_at: record.latest_updated_at(),
    };
    let mut ids = std::collections::HashSet::new();
    let mut paths = std::collections::HashSet::new();
    for previous in &index.entries {
        if !ids.insert(&previous.id) || !paths.insert(previous.path.to_lowercase()) {
            return Err(invalid("Duplicate data IDs or detail paths in daily index"));
        }
        let same_path = previous.path.eq_ignore_ascii_case(&reference.path);
        if (previous.id == reference.id) != same_path {
            return Err(invalid("Daily index data ID conflicts with specified detail path"));
        }
    }
    if let Some(previous) = index.entries.iter_mut().find(|r| r.id == reference.id) {
        *previous = reference;
    } else {
        index.entries.push(reference);
    }
    index.entries.sort_by(|a, b| a.id.cmp(&b.id));
    write_xml(path, record)?;
    write_xml(&index_path, &index)
}
