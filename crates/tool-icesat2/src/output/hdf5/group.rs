use crate::output::hdf5::attribute::attribute;
use crate::output::hdf5::dataset::dataset;
use crate::output::hdf5::references::Context;
use crate::output::hdf5::result::Result;
use hdf5_pure::{DType, Group};
use hdf5_writer::{DatasetBuilder, Hdf5Builder};

pub(super) fn group(
    source: &Group,
    path: &str,
    mut target: Hdf5Builder,
    context: &Context,
) -> Result<Hdf5Builder> {
    let attrs = if let Some(attrs) = context.attributes(path, false)? {
        attrs
    } else {
        source
            .attrs()?
            .into_iter()
            .map(|(k, v)| attribute(&k, v))
            .collect::<Result<Vec<_>>>()?
    };
    for attr in attrs {
        target = if path.is_empty() {
            target.attribute(attr)
        } else {
            target.group_attribute(path, attr)
        };
    }
    for name in source.datasets()? {
        let src = source.dataset(&name)?;
        let full = if path.is_empty() {
            name
        } else {
            format!("{path}/{name}")
        };
        let mut ds = match src.dtype()? {
            DType::F32 => dataset::<f32>(&src, &full)?,
            DType::F64 => dataset::<f64>(&src, &full)?,
            DType::I8 => dataset::<i8>(&src, &full)?,
            DType::I16 => dataset::<i16>(&src, &full)?,
            DType::I32 => dataset::<i32>(&src, &full)?,
            DType::I64 => dataset::<i64>(&src, &full)?,
            DType::U8 => dataset::<u8>(&src, &full)?,
            DType::U16 => dataset::<u16>(&src, &full)?,
            DType::U32 => dataset::<u32>(&src, &full)?,
            DType::U64 => dataset::<u64>(&src, &full)?,
            DType::String => {
                let (original, old) = context.original(&full).ok_or("Text dataset lacks its original source")?;
                let original = original.file.dataset(&old)?;
                DatasetBuilder::new(&full, original.dtype().clone(), original.shape().to_vec())
                    .raw_data(original.read_raw_bytes()?)
            }
            other => return Err(format!("Unsupported HDF5 dataset for writing: {full}: {other:?}").into()),
        };
        let attrs = if let Some(attrs) = context.attributes(&full, true)? {
            attrs
        } else {
            src.attrs()?
                .into_iter()
                .map(|(k, v)| attribute(&k, v))
                .collect::<Result<Vec<_>>>()?
        };
        for attr in attrs {
            ds = ds.attribute(attr);
        }
        target = target.dataset(ds);
    }
    for name in source.groups()? {
        let full = if path.is_empty() {
            name.clone()
        } else {
            format!("{path}/{name}")
        };
        target = group(&source.group(&name)?, &full, target, context)?;
    }
    Ok(target)
}
