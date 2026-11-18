use anyhow::Result;
use std::path::Path;
pub(crate) fn open(_: &Path) -> Result<()> { Ok(()) }
pub(crate) fn sample(_: &(), coords: impl Iterator<Item=(f32,f32)>) -> Result<Vec<Option<f32>>> {
    Ok(coords.map(|_| Some(0.5)).collect())
}
