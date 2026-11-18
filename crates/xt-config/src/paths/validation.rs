pub(super) fn component(value: &str) -> crate::Result<()> {
    let stem = value.split('.').next().unwrap_or("").to_ascii_uppercase();
    let reserved = matches!(stem.as_str(), "CON" | "PRN" | "AUX" | "NUL")
        || (stem.len() == 4
            && (stem.starts_with("COM") || stem.starts_with("LPT"))
            && matches!(stem.as_bytes()[3], b'1'..=b'9'));
    if value.is_empty()
        || matches!(value, "." | "..")
        || value.ends_with(['.', ' '])
        || value
            .chars()
            .any(|c| c.is_control() || r#"/\:*?"<>|"#.contains(c))
        || reserved
    {
        return Err(crate::Error::new(
            crate::ErrorKind::Configuration,
            "Data name and storage subdirectories must be valid single-component directory names",
        ));
    }
    Ok(())
}
