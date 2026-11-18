use crate::error::invalid;
use crate::model::{DataEntry, FileRecord};
use std::collections::BTreeSet;
use xt_error::Result;

pub(super) fn validate_file<T>(file: &FileRecord<T>) -> Result<()> {
    if [&file.product, &file.tool, &file.name, &file.local_path]
        .iter()
        .any(|v| v.trim().is_empty())
    {
        return Err(invalid("File record requires ID, tool, name, and path"));
    }
    if file
        .product
        .chars()
        .any(|c| c.is_whitespace() || c.is_control())
    {
        return Err(invalid("File ID must not contain whitespace or control characters"));
    }
    if file
        .operation
        .as_deref()
        .is_some_and(|v| v.trim().is_empty())
    {
        return Err(invalid("File operation name must not be empty"));
    }
    validate_time(file.updated_at.as_deref())
}

pub(super) fn validate_identity<T>(data: &DataEntry<T>) -> Result<()> {
    if [&data.id, &data.data_type, &data.date]
        .iter()
        .any(|v| v.trim().is_empty())
    {
        return Err(invalid("Manifest requires data ID, type, and date"));
    }
    Ok(())
}

pub(super) fn validate_data<T>(data: &DataEntry<T>) -> Result<()> {
    validate_identity(data)?;
    let mut ids = BTreeSet::new();
    for file in &data.files {
        validate_file(file)?;
        if !ids.insert(file.product.as_str()) {
            return Err(invalid("file.id must be unique within the data group"));
        }
    }
    for (index, status) in data.status.iter().enumerate() {
        if status.id.is_empty()
            || status
                .id
                .chars()
                .any(|c| c.is_whitespace() || c.is_control())
        {
            return Err(invalid("Operation name must be a nonempty identifier"));
        }
        let refs = status.targets();
        if refs.is_empty() || !refs.iter().all(|target| ids.contains(target)) {
            return Err(invalid("Operation target must reference an existing file.id in the data group"));
        }
        if data.status[..index].iter().any(|old| old.same_key(status)) {
            return Err(invalid("Duplicate operation and reference set"));
        }
        validate_time(status.updated_at.as_deref())?;
    }
    Ok(())
}

fn validate_time(value: Option<&str>) -> Result<()> {
    if value.is_some_and(|v| chrono::DateTime::parse_from_rfc3339(v).is_err()) {
        return Err(invalid("Update time must be RFC3339 with a timezone"));
    }
    Ok(())
}
