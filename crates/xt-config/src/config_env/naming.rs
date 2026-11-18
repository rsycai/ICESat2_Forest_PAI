use crate::config_env::error::configuration_error;
use xt_error::Result;

pub(super) fn normalize(name: &str) -> Result<String> {
    if name.is_empty()
        || name.contains("__")
        || name.starts_with('_')
        || name.ends_with('_')
        || !name.bytes().all(|b| b.is_ascii_alphanumeric() || b == b'_')
    {
        return Err(configuration_error(format!(
            "Invalid configuration name {name}; use ASCII letters, digits, and single interior underscores"
        )));
    }
    Ok(name.to_ascii_uppercase())
}
