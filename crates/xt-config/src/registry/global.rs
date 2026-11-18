use crate::registry::container::ConfigRegistry;
use crate::registry::state::is_loading;
use std::sync::{Mutex, OnceLock};
use xt_error::{Error, ErrorKind, Result};

pub(super) static GLOBAL: OnceLock<ConfigRegistry> = OnceLock::new();

pub(super) static INIT_LOCK: Mutex<()> = Mutex::new(());

pub(crate) fn is_initialized() -> bool {
    GLOBAL.get().is_some()
}

pub fn init_registered() -> Result<()> {
    initialize(|| Ok(()), false)
}

pub(crate) fn initialize(prepare: impl FnOnce() -> Result<()>, from_file: bool) -> Result<()> {
    if is_loading() {
        return Err(Error::new(
            ErrorKind::Configuration,
            "Cannot reinitialize global configuration while loading a subconfiguration",
        ));
    }
    if !from_file && GLOBAL.get().is_some() {
        return Ok(());
    }
    let _guard = INIT_LOCK
        .lock()
        .map_err(|_| Error::new(ErrorKind::Internal, "Configuration initialization lock is unavailable"))?;
    if GLOBAL.get().is_some() {
        if from_file {
            return Err(Error::new(
                ErrorKind::Configuration,
                "Global configuration is initialized; cannot reload a configuration file",
            ));
        }
    } else {
        prepare()?;
        let registry = ConfigRegistry::load()?;
        GLOBAL
            .set(registry)
            .map_err(|_| Error::new(ErrorKind::Internal, "Configuration snapshot installed more than once"))?;
    }
    Ok(())
}

pub fn get<T: Send + Sync + 'static>() -> Result<&'static T> {
    crate::ensure_loaded()?;
    GLOBAL
        .get()
        .ok_or_else(|| {
            Error::new(
                ErrorKind::Configuration,
                "Initialize configuration with envload! or AppConfig::new() first",
            )
        })?
        .get::<T>()
}
