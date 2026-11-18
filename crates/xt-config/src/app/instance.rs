use crate::app::loading::ensure_loaded;
use crate::app::model::{APP_CONFIG, AppConfig};

impl AppConfig {
    pub fn from_file(path: impl AsRef<std::path::Path>) -> crate::Result<Self> {
        crate::registry::initialize(
            || {
                let values = crate::EnvConfig::from_path(path, "XT")?.variables().clone();
                crate::env_load::install_file_defaults(values)
            },
            true,
        )?;
        Ok(Self {})
    }

    pub fn global() -> crate::Result<&'static Self> {
        ensure_loaded()?;
        Ok(&APP_CONFIG)
    }

    #[doc(hidden)]
    #[cfg(windows)]
    pub fn load_env(
        path: impl AsRef<std::path::Path>,
        prefix: &str,
    ) -> crate::Result<crate::ExportReport> {
        let mut report = None;
        crate::registry::initialize(
            || {
                report = Some(crate::init_config(path, prefix)?);
                Ok(())
            },
            true,
        )?;
        Ok(report.expect("Successful file initialization must produce export statistics"))
    }

    ///
    /// # Safety
    #[doc(hidden)]
    #[cfg(not(windows))]
    pub unsafe fn load_env(
        path: impl AsRef<std::path::Path>,
        prefix: &str,
    ) -> crate::Result<crate::ExportReport> {
        let mut report = None;
        crate::registry::initialize(
            || {
                report = Some(unsafe { crate::init_config(path, prefix)? });
                Ok(())
            },
            true,
        )?;
        Ok(report.expect("Successful file initialization must produce export statistics"))
    }
    #[cfg(windows)]
    pub fn from_path(path: impl AsRef<std::path::Path>) -> crate::Result<Self> {
        Self::from_path_with_prefix(path, "XT")
    }

    #[cfg(windows)]
    pub fn from_path_with_prefix(
        path: impl AsRef<std::path::Path>,
        prefix: &str,
    ) -> crate::Result<Self> {
        Self::load_env(path, prefix)?;
        Ok(Self {})
    }

    ///
    /// # Safety
    #[cfg(not(windows))]
    pub unsafe fn from_path(path: impl AsRef<std::path::Path>) -> crate::Result<Self> {
        unsafe { Self::from_path_with_prefix(path, "XT") }
    }

    ///
    /// # Safety
    #[cfg(not(windows))]
    pub unsafe fn from_path_with_prefix(
        path: impl AsRef<std::path::Path>,
        prefix: &str,
    ) -> crate::Result<Self> {
        unsafe {
            Self::load_env(path, prefix)?;
        }
        Ok(Self {})
    }
}
