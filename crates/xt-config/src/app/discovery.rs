pub const DEFAULT_CONFIG_PATH: &str = "config/config.yaml";

pub(super) fn default_path() -> crate::Result<std::path::PathBuf> {
    let base = std::path::Path::new(DEFAULT_CONFIG_PATH);
    for extension in ["yaml", "yml", "toml", "json", "json5", "ini", "ron"] {
        let path = base.with_extension(extension);
        if path.try_exists().map_err(crate::Error::from)? {
            return Ok(path);
        }
    }
    Ok(base.to_owned())
}
