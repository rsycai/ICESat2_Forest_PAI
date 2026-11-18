use crate::PaiConfig;
use sha2::{Digest, Sha256};
use std::{fs, path::Path, time::UNIX_EPOCH};

pub(super) fn modified(path: &Path) -> Option<String> {
    Some(
        fs::metadata(path)
            .ok()?
            .modified()
            .ok()?
            .duration_since(UNIX_EPOCH)
            .ok()?
            .as_nanos()
            .to_string(),
    )
}

pub(super) fn signature(config: &PaiConfig, op: &str, input: &Path, date: &str) -> Option<String> {
    let mut hash = Sha256::new();
    hash.update(b"pai-workflow-v2-photon-only");
    let mut add_file = |path: &Path| -> Option<()> {
        let path = path.canonicalize().ok()?;
        hash.update(path.as_os_str().as_encoded_bytes());
        hash.update(fs::metadata(&path).ok()?.len().to_le_bytes());
        hash.update(modified(&path)?);
        Some(())
    };
    add_file(input)?;
    add_file(&config.g0_raster)?;
    if op == "pai_v2" {
        let raster = config.landcover.replace("{year}", date.get(..4)?);
        add_file(Path::new(&raster))?;
        hash.update(&config.landcover_classes);
        hash.update([config.qa_min as u8, config.qa_max as u8]);
    }
    hash.update(op.as_bytes());
    Some(format!("{:x}", hash.finalize()))
}
