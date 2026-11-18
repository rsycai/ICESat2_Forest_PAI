mod arguments;
#[cfg(windows)]
mod logging;
pub(crate) use arguments::Cli;
#[cfg(windows)]
pub(crate) use logging::ExampleLogConfig;
