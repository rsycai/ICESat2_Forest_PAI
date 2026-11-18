use crate::processing::config::Icesat2Config;
use crate::model::{BeamReport, ProcessRequest};
use crate::processing::naming::paired_key;
use xt_error::{Error, ErrorKind, Result};
use xt_task::model::ProcessContext;

pub fn match_pair(
    request: &ProcessRequest,
    options: &Icesat2Config,
    context: &ProcessContext,
) -> Result<Vec<BeamReport>> {
    context.check_cancelled()?;
    options.validate()?;
    let key = paired_key(&request.atl03, &request.atl08)?;
    let a = std::fs::canonicalize(&request.atl03)?;
    let b = std::fs::canonicalize(&request.atl08)?;
    if request.output.exists() {
        let output = std::fs::canonicalize(&request.output)?;
        if output == a || output == b {
            return Err(Error::new(
                ErrorKind::InvalidArgument,
                "Output must not overwrite an input product",
            ));
        }
        if !request.overwrite {
            return Err(Error::new(
                ErrorKind::Io(std::io::ErrorKind::AlreadyExists),
                "Output already exists",
            ));
        }
    }
    crate::processing::execute::execute(request, options, &key, context)
}
