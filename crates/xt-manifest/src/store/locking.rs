use crate::error::invalid;
use crate::store::lock::DayLock;
use std::fs;
use std::path::Path;
use xt_error::Result;

pub fn lock_day(day: &Path) -> Result<DayLock> {
    fs::create_dir_all(day)?;
    let (lock_path, cleanup_root) = if let Some(root) = day
        .ancestors()
        .find(|p| p.file_name().is_some_and(|n| n == "manifests"))
    {
        let folder = root.join(".locks");
        fs::create_dir_all(&folder)?;
        let name = day
            .strip_prefix(root)
            .map_err(|_| invalid("Daily directory is outside the manifest directory"))?
            .components()
            .map(|c| c.as_os_str().to_string_lossy())
            .collect::<Vec<_>>()
            .join("-");
        (folder.join(format!("{name}.lock")), Some(root))
    } else {
        (day.join(".xt-manifest.lock"), None)
    };
    let guard = xt_lock::FileLock::acquire(lock_path)?;
    let guard = if let Some(root) = cleanup_root {
        guard.with_cleanup_root(root)?
    } else {
        guard
    };
    Ok(DayLock {
        _guard: guard,
        day: day.to_path_buf(),
    })
}
