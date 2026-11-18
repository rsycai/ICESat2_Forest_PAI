use std::{fmt::Display, process::ExitCode};

pub fn exit_result(result: std::result::Result<(), impl Display>) -> ExitCode {
    match result {
        Ok(()) => ExitCode::SUCCESS,
        Err(error) => {
            crate::logging::report_error(error);
            ExitCode::FAILURE
        }
    }
}
