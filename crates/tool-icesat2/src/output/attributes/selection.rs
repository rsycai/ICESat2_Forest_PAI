use crate::output::attributes::error::{bad, h5};
use crate::output::attributes::rows::Rows;
use hdf5_pure::{DType, Dataset, DatasetBuilder, Group, GroupBuilder};
use xt_error::Result;

pub(super) fn selected<T: Rows>(
    src: &Dataset,
    dst: &mut DatasetBuilder,
    indices: &[u64],
    shape: &[u64],
    window: u64,
) -> Result<()> {
    let width = shape[1..]
        .iter()
        .try_fold(1usize, |n, &d| n.checked_mul(d as usize))
        .ok_or_else(|| bad("Attribute column width overflow"))?;
    let capacity = indices
        .len()
        .checked_mul(width)
        .ok_or_else(|| bad("Attribute column size overflow"))?;
    let mut values = Vec::with_capacity(capacity);
    let mut cached_start = u64::MAX;
    let mut block = Vec::new();
    for &row in indices {
        let start = row / window * window;
        if start != cached_start {
            let end = start.saturating_add(window).min(shape[0]);
            block = T::rows(src, start, end - start)?;
            if block.len() != (end - start) as usize * width {
                return Err(bad("Inconsistent attribute read window lengths"));
            }
            cached_start = start;
        }
        let i = (row - start) as usize * width;
        values.extend_from_slice(&block[i..i + width]);
    }
    let mut output_shape = shape.to_vec();
    output_shape[0] = indices.len() as u64;
    dst.with_data(&values).with_shape(&output_shape);
    if !indices.is_empty() {
        let mut chunks = output_shape;
        chunks[0] = chunks[0].min(8192);
        dst.with_chunks(&chunks).with_shuffle().with_deflate(6);
    }
    Ok(())
}

pub(super) fn numeric(
    src: &Dataset,
    g: &mut GroupBuilder,
    name: &str,
    indices: &[u64],
    expected: u64,
    window: u64,
) -> Result<()> {
    let shape = src.shape().map_err(h5)?;
    if shape.is_empty() || shape[0] != expected || indices.iter().any(|&i| i >= shape[0]) {
        return Err(bad(format!("{name}: invalid attribute dimensions or source row index")));
    }
    let dtype = src.dtype().map_err(h5)?;
    let dst = g.create_dataset(name);
    match dtype {
        DType::F32 => selected::<f32>(src, dst, indices, &shape, window)?,
        DType::F64 => selected::<f64>(src, dst, indices, &shape, window)?,
        DType::I8 => selected::<i8>(src, dst, indices, &shape, window)?,
        DType::I16 => selected::<i16>(src, dst, indices, &shape, window)?,
        DType::I32 => selected::<i32>(src, dst, indices, &shape, window)?,
        DType::I64 => selected::<i64>(src, dst, indices, &shape, window)?,
        DType::U8 => selected::<u8>(src, dst, indices, &shape, window)?,
        DType::U16 => selected::<u16>(src, dst, indices, &shape, window)?,
        DType::U32 => selected::<u32>(src, dst, indices, &shape, window)?,
        DType::U64 => selected::<u64>(src, dst, indices, &shape, window)?,
        _ => return Err(bad(format!("{name} is not a supported numeric attribute"))),
    }
    Ok(())
}

pub(super) fn segment_group(
    src: &Group,
    dst: &mut GroupBuilder,
    indices: &[u64],
    expected: u64,
    window: u64,
    depth: usize,
) -> Result<()> {
    if depth > 8 {
        return Err(bad("Segment attribute group depth exceeds the limit"));
    }
    for name in src.datasets().map_err(h5)? {
        numeric(
            &src.dataset(&name).map_err(h5)?,
            dst,
            &name,
            indices,
            expected,
            window,
        )?;
    }
    for name in src.groups().map_err(h5)? {
        let mut child = dst.create_group(&name);
        segment_group(
            &src.group(&name).map_err(h5)?,
            &mut child,
            indices,
            expected,
            window,
            depth + 1,
        )?;
        dst.add_group(child.finish());
    }
    Ok(())
}
