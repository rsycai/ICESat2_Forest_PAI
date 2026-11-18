use xt_error::{Error, ErrorKind, Result};

pub(super) static FILE_DEFAULTS: std::sync::RwLock<std::collections::BTreeMap<String, String>> =
    std::sync::RwLock::new(std::collections::BTreeMap::new());

thread_local! {
    pub(super) static USE_FILE_DEFAULTS: std::cell::Cell<bool> = const { std::cell::Cell::new(false) };
}

pub(crate) fn install_file_defaults(
    values: std::collections::BTreeMap<String, String>,
) -> Result<()> {
    *FILE_DEFAULTS
        .write()
        .map_err(|_| Error::new(ErrorKind::Internal, "Configuration source lock is unavailable"))? = values;
    Ok(())
}

#[doc(hidden)]
pub fn with_file_defaults<T>(load: impl FnOnce() -> Result<T>) -> Result<T> {
    struct Restore(bool);
    impl Drop for Restore {
        fn drop(&mut self) {
            USE_FILE_DEFAULTS.set(self.0);
        }
    }
    let _restore = Restore(USE_FILE_DEFAULTS.replace(true));
    load()
}
