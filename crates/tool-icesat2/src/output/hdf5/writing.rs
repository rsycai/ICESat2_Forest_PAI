use crate::output::hdf5::group::group;
use crate::output::hdf5::original::Original;
use crate::output::hdf5::references::Context;
use crate::output::hdf5::result::Result;
use hdf5_pure::{File, FileBuilder, Group};
use hdf5_writer::{Hdf5Builder, Hdf5Writer, WriteOptions};
use std::collections::BTreeSet;
use std::io::Write;
use std::path::Path;
use xt_error::{DataError, Error, ErrorKind};

pub(crate) fn write(
    builder: FileBuilder,
    sink: &mut std::fs::File,
    parent: &Path,
    source03: &Path,
    source08: &Path,
) -> xt_error::Result<()> {
    let result = (|| -> Result<()> {
        let mut staging = tempfile::NamedTempFile::new_in(parent)?;
        staging.write_all(&builder.finish()?)?;
        staging.flush()?;
        let source = File::open_streaming(staging.path())?;
        fn paths(group: &Group, prefix: &str, retained: &mut BTreeSet<String>) -> Result<()> {
            for name in group.datasets()? {
                retained.insert(format!("{prefix}{name}"));
            }
            for name in group.groups()? {
                paths(&group.group(&name)?, &format!("{prefix}{name}/"), retained)?;
            }
            Ok(())
        }
        let mut retained = BTreeSet::new();
        paths(&source.root(), "", &mut retained)?;
        let context = Context {
            originals: [
                Original::open("ATL03", source03)?,
                Original::open("ATL08", source08)?,
            ],
            retained,
        };
        let target = group(&source.root(), "", Hdf5Builder::new(), &context)?;
        let plan = target.into_plan()?;
        Hdf5Writer::new(sink, WriteOptions::default()).finish(plan)?;
        Ok(())
    })();
    result.map_err(|error| {
        Error::new(
            ErrorKind::Data(DataError::UnsupportedFormat),
            format!("Failed to write HDF5 file: {error}"),
        )
    })
}
