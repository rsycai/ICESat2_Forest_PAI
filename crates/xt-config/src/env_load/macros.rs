#[macro_export]
macro_rules! envload {
    ($path:expr $(,)?) => {
        $crate::AppConfig::load_env($path, "XT")
    };
    ($path:expr, $prefix:expr $(,)?) => {
        $crate::AppConfig::load_env($path, $prefix)
    };
}
