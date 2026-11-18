use hdf5_pure::{AttrValue, GroupBuilder, H5Element};

pub(super) fn column<T: H5Element>(
    g: &mut GroupBuilder,
    name: &str,
    values: Vec<T>,
    units: Option<&str>,
) {
    let ds = g.create_dataset(name);
    ds.with_data(&values).with_shape(&[values.len() as u64]);
    if !values.is_empty() {
        ds.with_chunks(&[(values.len() as u64).min(8192)])
            .with_shuffle()
            .with_deflate(6);
    }
    if let Some(u) = units {
        ds.set_attr("units", AttrValue::AsciiString(u.into()));
    }
}
