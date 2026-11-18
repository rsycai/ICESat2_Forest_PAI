use crate::processing::error::{bad, h5};
use hdf5_pure::{Dataset, File, H5Element};
use xt_error::Result;

pub(super) fn dataset(file: &File, path: &str) -> Result<Dataset> {
    file.dataset(path).map_err(|e| h5(e).context(path))
}

pub(super) fn read1<T: H5Element>(file: &File, path: &str) -> Result<Vec<T>> {
    let ds = dataset(file, path)?;
    if ds.shape().map_err(h5)?.len() != 1 {
        return Err(bad(format!("{path} must be a one-dimensional dataset")));
    }
    ds.read().map_err(|e| h5(e).context(path))
}

pub(super) fn same_length(expected: usize, actual: usize, name: &str) -> Result<()> {
    if expected != actual {
        return Err(bad(format!(
            "{name}: length mismatch: expected {expected}, got {actual}"
        )));
    }
    Ok(())
}
