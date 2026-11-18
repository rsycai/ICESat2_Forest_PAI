use crate::config_env::model::{EnvConfig, ExportReport};

impl EnvConfig {
    #[cfg(windows)]
    pub fn apply(&self) -> ExportReport {
        unsafe { self.apply_inner() }
    }
    ///
    /// # Safety
    #[cfg(not(windows))]
    pub unsafe fn apply(&self) -> ExportReport {
        unsafe { self.apply_inner() }
    }

    unsafe fn apply_inner(&self) -> ExportReport {
        let mut report = ExportReport {
            files: self.files.len(),
            written: 0,
            preserved: 0,
        };
        for (key, value) in &self.variables {
            if std::env::var_os(key).is_some() {
                report.preserved += 1;
            } else {
                unsafe {
                    std::env::set_var(key, value);
                }
                report.written += 1;
            }
        }
        report
    }
}
