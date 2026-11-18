use crate::error::invalid;
use crate::model::FileRecord;
use crate::store::paths::display_path;
use std::fs;
use std::path::Path;
use xt_error::Result;

pub fn completed_file<T>(
    product: &str,
    tool: &str,
    operation: &str,
    path: &Path,
    details: T,
) -> Result<FileRecord<T>> {
    let path = fs::canonicalize(path)?;
    let metadata = fs::metadata(&path)?;
    if !metadata.is_file() {
        return Err(invalid("Processed result must be a regular file"));
    }
    Ok(FileRecord {
        product: product.into(),
        tool: tool.into(),
        operation: Some(operation.into()),
        updated_at: None,
        name: path
            .file_name()
            .and_then(|n| n.to_str())
            .ok_or_else(|| invalid("Result filename must be Unicode"))?
            .into(),
        local_path: display_path(&path),
        size_bytes: Some(metadata.len()),
        details,
    })
}

pub(super) fn put_file<T>(files: &mut Vec<FileRecord<T>>, mut file: FileRecord<T>) -> String {
    if let Some(previous) = files.iter_mut().find(|old| {
        old.tool == file.tool
            && old.operation == file.operation
            && old.local_path == file.local_path
    }) {
        file.product = previous.product.clone();
        let id = file.product.clone();
        *previous = file;
        return id;
    }
    if files.iter().any(|old| old.product == file.product) {
        let base = file.product.clone();
        file.product = format!("{}:{}", base, reference_token(&file.name));
        if files.iter().any(|old| old.product == file.product) {
            file.product = format!(
                "{}:{}:{}:{}",
                base,
                reference_token(&file.tool),
                reference_token(file.operation.as_deref().unwrap_or("file")),
                reference_token(&file.local_path)
            );
        }
    }
    let id = file.product.clone();
    files.push(file);
    id
}

pub fn reference_token(value: &str) -> String {
    let mut output = String::new();
    for byte in value.bytes() {
        if byte.is_ascii_alphanumeric() || b"_.-".contains(&byte) {
            output.push(byte as char);
        } else {
            output.push_str(&format!("%{byte:02X}"));
        }
    }
    output
}
