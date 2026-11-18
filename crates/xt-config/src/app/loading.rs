use crate::app::discovery::default_path;
use crate::app::notices::remember_missing_default;

pub fn ensure_loaded() -> crate::Result<()> {
    if crate::registry::is_initialized() || crate::registry::is_loading() {
        return Ok(());
    }
    crate::registry::initialize(
        || {
            let explicit = std::env::var_os("XT_CONFIG_PATH");
            let path = explicit
                .clone()
                .map(std::path::PathBuf::from)
                .map(Ok)
                .unwrap_or_else(default_path)?;
            let values = if path.try_exists().map_err(crate::Error::from)? {
                crate::EnvConfig::from_path(&path, "XT")?
                    .variables()
                    .clone()
            } else {
                if explicit.is_some() {
                    return Err(crate::Error::new(
                        xt_error::ErrorKind::Configuration,
                        "Configuration path specified by XT_CONFIG_PATH does not exist",
                    ));
                }
                remember_missing_default(path);
                std::collections::BTreeMap::new()
            };
            crate::env_load::install_file_defaults(values)
        },
        false,
    )
}
