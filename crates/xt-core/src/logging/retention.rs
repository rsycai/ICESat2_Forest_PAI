use std::path::Path;
use std::{fs, io};
use time::macros::format_description;
use time::{Date, Duration};

pub(super) fn cleanup(directory: &Path, prefix: &str, days: u8, today: Date) -> io::Result<()> {
    let oldest = today - Duration::days(i64::from(days) - 1);
    let start = format!("{prefix}.");
    for entry in fs::read_dir(directory)? {
        let entry = entry?;
        if !entry.file_type()?.is_file() {
            continue;
        }
        let name = entry.file_name();
        let Some(name) = name.to_str() else {
            continue;
        };
        let Some(date) = name
            .strip_prefix(&start)
            .and_then(|s| s.strip_suffix(".log"))
        else {
            continue;
        };
        let Ok(date) = Date::parse(date, format_description!("[year]-[month]-[day]")) else {
            continue;
        };
        if date < oldest {
            match fs::remove_file(entry.path()) {
                Ok(()) => (),
                Err(e) if e.kind() == io::ErrorKind::NotFound => (),
                Err(e) => return Err(e),
            }
        }
    }
    Ok(())
}
