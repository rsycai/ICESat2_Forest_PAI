use super::GeoTiffSampler;
use std::{
    path::{Path, PathBuf, absolute},
    sync::{Arc, Mutex},
};
use xt_error::{Error, ErrorKind, Result};

#[derive(Default)]
pub struct GeoTiffCache {
    current: Mutex<Option<(PathBuf, Arc<GeoTiffSampler>)>>,
}

impl GeoTiffCache {
    pub fn open(&self, path: impl AsRef<Path>) -> Result<Arc<GeoTiffSampler>> {
        let path = absolute(path)?;
        let mut current = self
            .current
            .lock()
            .map_err(|_| Error::new(ErrorKind::Internal, "GeoTIFF file handle cache lock is poisoned"))?;
        if let Some((cached_path, sampler)) = current.as_ref()
            && *cached_path == path
        {
            return Ok(Arc::clone(sampler));
        }
        let sampler = Arc::new(GeoTiffSampler::open(&path)?);
        *current = Some((path, Arc::clone(&sampler)));
        Ok(sampler)
    }
}
