use crate::output::model::OutputBuilder;
use xt_error::{Error, ErrorKind, Result};

pub(super) fn json<T: serde::Serialize>(value: &T) -> Result<String> {
    serde_json::to_string(value)
        .map_err(|e| Error::new(ErrorKind::Internal, "Failed to serialize processing parameters").with_source(e))
}

pub(crate) fn global(
    output: &mut OutputBuilder,
    a: &hdf5_pure::File,
    b: &hdf5_pure::File,
) -> Result<()> {
    crate::output::attributes::global(&mut output.atl03, a)?;
    crate::output::attributes::global(&mut output.atl08, b)
}
