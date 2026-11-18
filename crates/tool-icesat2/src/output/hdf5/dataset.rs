use crate::output::hdf5::result::Result;
use hdf5_pure::{Dataset, H5Element};
use hdf5_writer::{DatasetBuilder, FilterDescription, H5WriteElement};

pub(super) fn dataset<T: H5Element + H5WriteElement>(
    source: &Dataset,
    path: &str,
) -> Result<DatasetBuilder> {
    let shape = source.shape()?;
    let values = source.read::<T>()?;
    let mut ds = DatasetBuilder::typed_data(path, shape.clone(), &values)?;
    if !values.is_empty() && !shape.is_empty() {
        let mut chunks = shape;
        chunks[0] = chunks[0].min(8192);
        ds = ds
            .chunked(chunks)
            .filter(FilterDescription {
                id: 2,
                name: None,
                client_data: vec![],
            })
            .filter(FilterDescription {
                id: 1,
                name: None,
                client_data: vec![6],
            });
    }
    Ok(ds)
}
