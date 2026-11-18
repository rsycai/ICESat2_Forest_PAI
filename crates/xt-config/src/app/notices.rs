use std::{path::PathBuf, sync::Mutex};

static MISSING_DEFAULT: Mutex<Option<PathBuf>> = Mutex::new(None);

pub(super) fn remember_missing_default(path: PathBuf) {
    *MISSING_DEFAULT
        .lock()
        .unwrap_or_else(|error| error.into_inner()) = Some(path);
}

pub fn take_missing_default() -> Option<PathBuf> {
    MISSING_DEFAULT
        .lock()
        .unwrap_or_else(|error| error.into_inner())
        .take()
}
