use crate::domain::{DataGroup, Granule, GroupIdentity};
use crate::interfaces::StoragePolicy;
use crate::services::storage::StandardStorage;
use crate::services::storage::paths::original_file_name;
use std::path::PathBuf;
use xt_error::Result;

pub struct PairedStorage;
impl StoragePolicy for PairedStorage {
    fn identity(&self, group: &DataGroup) -> Result<GroupIdentity> {
        if group.granules.len() != 2 {
            return Err(crate::domain::invalid("Pair manifest must contain both products"));
        }
        let original = |product: &str| -> Result<String> {
            let entries: Vec<_> = group
                .granules
                .iter()
                .filter(|g| g.product == product)
                .collect();
            if entries.len() != 1 || entries[0].assets.len() != 1 {
                return Err(crate::domain::invalid(
                    "Pairing requires one ATL03 file and one ATL08 file",
                ));
            }
            let url = reqwest::Url::parse(&entries[0].assets[0].url)
                .map_err(|_| crate::domain::invalid("Invalid paired file URL"))?;
            Ok(original_file_name(&url)?.to_owned())
        };
        Ok(GroupIdentity {
            id: xt_icesat2_meta::manifest::pair_stem(&original("ATL03")?, &original("ATL08")?)?,
            data_type: "ATL03_08".into(),
            acquired_at: group.acquired_at.clone().or_else(|| {
                group
                    .granules
                    .first()
                    .and_then(crate::providers::nasa::metadata::acquisition)
            }),
            paired: true,
        })
    }
    fn directory(&self, identity: &GroupIdentity) -> Result<PathBuf> {
        StandardStorage::default().directory(identity)
    }
    fn acquired_at(&self, granule: &Granule) -> Option<String> {
        crate::providers::nasa::metadata::acquisition(granule)
    }
}
