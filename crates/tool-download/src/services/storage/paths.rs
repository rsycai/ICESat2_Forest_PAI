use crate::Result;
use std::path::Path;

pub(crate) fn display(path: &Path) -> String {
    let value = path.to_string_lossy();
    #[cfg(windows)]
    {
        windows_display(&value)
    }
    #[cfg(not(windows))]
    {
        value.into_owned()
    }
}

#[cfg(any(windows, test))]
pub(super) fn windows_display(value: &str) -> String {
    let separator = char::from(92);
    let prefix = format!("{separator}{separator}?{separator}");
    if let Some(rest) = value.strip_prefix(&prefix) {
        if let Some(unc) = rest.strip_prefix(&format!("UNC{separator}")) {
            return format!("{separator}{separator}{unc}");
        }
        let bytes = rest.as_bytes();
        if bytes.len() >= 3 && bytes[0].is_ascii_alphabetic() && bytes[1] == b':' && bytes[2] == 92
        {
            return rest.to_owned();
        }
    }
    value.to_owned()
}

pub fn safe_component(value: &str) -> String {
    let result: String = value
        .chars()
        .take(80)
        .map(|ch| {
            if ch.is_ascii_alphanumeric() || ch == '-' || ch == '_' {
                ch
            } else {
                '_'
            }
        })
        .collect();
    if result.is_empty() {
        return "data".into();
    }
    let upper = result.to_ascii_uppercase();
    if [
        "CON", "PRN", "AUX", "NUL", "COM1", "COM2", "COM3", "COM4", "COM5", "COM6", "COM7", "COM8",
        "COM9", "LPT1", "LPT2", "LPT3", "LPT4", "LPT5", "LPT6", "LPT7", "LPT8", "LPT9",
    ]
    .contains(&upper.as_str())
    {
        format!("_{result}")
    } else {
        result
    }
}

pub fn data_directory_name(id: &str) -> Result<String> {
    if id.is_empty() {
        return Err(crate::domain::invalid("Data ID must not be empty"));
    }
    let mut name = String::new();
    for byte in id.bytes() {
        if byte.is_ascii_alphanumeric() || byte == b'-' || byte == b'_' {
            name.push(char::from(byte));
        } else {
            name.push_str(&format!("%{byte:02X}"));
        }
    }
    if safe_component(&name) == format!("_{name}") {
        name = format!("%{:02X}{}", name.as_bytes()[0], &name[1..]);
    }
    if name.len() > 240 {
        return Err(crate::domain::invalid(
            "Encoded data ID exceeds 240 bytes; cannot generate a stable directory name",
        ));
    }
    Ok(name)
}

pub fn original_file_name(url: &reqwest::Url) -> Result<&str> {
    let name = url
        .path_segments()
        .and_then(|mut parts| parts.next_back())
        .filter(|name| !name.is_empty())
        .ok_or_else(|| crate::domain::invalid("Download URL lacks the original filename"))?;
    let stem = name.split('.').next().unwrap_or_default();
    if name == "."
        || name == ".."
        || name.ends_with([' ', '.'])
        || name.chars().any(|ch| {
            ch.is_control() || matches!(ch, '/' | '\\' | '<' | '>' | ':' | '"' | '|' | '?' | '*')
        })
        || safe_component(stem) == format!("_{stem}")
    {
        return Err(crate::domain::invalid(
            "Invalid original filename or collision with local manifest name",
        ));
    }
    Ok(name)
}
