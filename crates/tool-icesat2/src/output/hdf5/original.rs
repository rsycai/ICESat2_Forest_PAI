use crate::output::hdf5::result::Result;
use hdf5_reader::Hdf5File;
use std::collections::{BTreeMap, BTreeSet};
use std::path::Path;

pub(super) struct Original {
    pub(super) product: &'static str,
    pub(super) file: Hdf5File,
    pub(super) addresses: BTreeMap<u64, String>,
}

impl Original {
    pub(super) fn open(product: &'static str, path: &Path) -> Result<Self> {
        pub(super) fn walk(
            group: &hdf5_reader::group::Group,
            path: &str,
            result: &mut BTreeMap<u64, String>,
            visited: &mut BTreeSet<u64>,
        ) -> Result<()> {
            if !visited.insert(group.address()) {
                return Ok(());
            }
            result.insert(group.address(), path.into());
            let (groups, datasets) = group.member_names()?;
            for name in datasets {
                let object = group.dataset(&name)?;
                result
                    .entry(object.address())
                    .or_insert_with(|| format!("{path}{name}"));
            }
            for name in groups {
                walk(
                    &group.group(&name)?,
                    &format!("{path}{name}/"),
                    result,
                    visited,
                )?;
            }
            Ok(())
        }
        let file = Hdf5File::open(path)?;
        let mut addresses = BTreeMap::new();
        walk(
            &file.root_group()?,
            "/",
            &mut addresses,
            &mut BTreeSet::new(),
        )?;
        Ok(Self {
            product,
            file,
            addresses,
        })
    }
    pub(super) fn target(&self, reference: &[u8]) -> Result<String> {
        let address = hdf5_reader::reference::resolve_object_reference(
            reference,
            self.file.root_group()?.offset_size(),
        )?;
        self.addresses
            .get(&address)
            .cloned()
            .ok_or_else(|| format!("Cannot resolve original object reference {address} in {}", self.product).into())
    }
}
