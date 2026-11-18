use crate::output::model::OutputBuilder;
use std::path::Path;
use xt_error::{Error, Result};

pub(crate) fn publish(mut builder: OutputBuilder, path: &Path, overwrite: bool) -> Result<()> {
    let parent = path
        .parent()
        .filter(|p| !p.as_os_str().is_empty())
        .unwrap_or(Path::new("."));
    std::fs::create_dir_all(parent).map_err(Error::from)?;
    let mut temp = tempfile::NamedTempFile::new_in(parent).map_err(Error::from)?;
    builder.file.add_group(builder.atl03.finish());
    builder.file.add_group(builder.atl08.finish());
    builder.file.add_group(builder.matched.finish());
    crate::output::hdf5::write(
        builder.file,
        temp.as_file_mut(),
        parent,
        &builder.source03,
        &builder.source08,
    )?;
    temp.as_file().sync_all().map_err(Error::from)?;
    if overwrite {
        temp.persist(path)
    } else {
        temp.persist_noclobber(path)
    }
    .map_err(|e| Error::from(e.error))?;
    Ok(())
}
