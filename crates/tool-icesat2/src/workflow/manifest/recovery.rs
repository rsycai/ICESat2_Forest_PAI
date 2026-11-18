use crate::{Icesat2Config, workflow::manifest::MatchDetails};
use sha2::{Digest, Sha256};
use std::{io::Read, path::Path};
use xt_error::Result;
use xt_manifest::{ManifestStore, TaskState, XmlValue};
use xt_task::model::ProcessContext;

pub(crate) fn completed_after_cleanup(
    request: &crate::ProcessRequest,
    options: &Icesat2Config,
    context: &ProcessContext,
) -> Result<bool> {
    context.check_cancelled()?;
    if request.overwrite {
        return Ok(false);
    }
    let missing03 = !request.atl03.try_exists()?;
    let missing08 = !request.atl08.try_exists()?;
    if (!missing03 && !missing08) || !request.output.try_exists()? {
        return Ok(false);
    }
    let manifest = match &request.manifest {
        Some(path) => path.clone(),
        None => xt_icesat2_meta::manifest::record_path(&request.atl03, &request.atl08)?,
    };
    let Some(record) = ManifestStore::<XmlValue>::new(&manifest).load()? else {
        return Ok(false);
    };
    let expected_name = crate::processing::paired_output_name(&request.atl03, &request.atl08)?;
    if record.data.id != expected_name.trim_end_matches(".h5") {
        return Ok(false);
    }
    let has_status = |operation: &str, state: TaskState, target: &str| {
        record
            .data
            .status
            .iter()
            .any(|s| s.id == operation && s.state == state && s.targets().contains(target))
    };
    let local = xt_manifest::display_path(&std::fs::canonicalize(&request.output)?);
    let Some(file) = record.data.files.iter().find(|f| {
        f.product == "ATL03_08"
            && f.tool == "xt-icesat2"
            && f.operation.as_deref() == Some("match")
            && f.local_path == local
    }) else {
        return Ok(false);
    };
    if !has_status("match", TaskState::Completed, "ATL03_08") {
        return Ok(false);
    }
    let details: MatchDetails = file.details.decode()?;
    let Some(signature) = &details.workflow_signature else {
        return Ok(false);
    };
    let parts: Vec<_> = signature.splitn(5, ':').collect();
    if parts.len() != 5
        || parts[0] != "match-v1"
        || parts[1] != options.read_window_rows.to_string()
        || parts[4] != format!("{:?}", request.beams)
    {
        return Ok(false);
    }
    for (role, path, missing, hash) in [
        ("ATL03", &request.atl03, missing03, parts[2]),
        ("ATL08", &request.atl08, missing08, parts[3]),
    ] {
        if !record.data.files.iter().any(|f| {
            f.product == role && Some(f.name.as_str()) == path.file_name().and_then(|s| s.to_str())
        }) {
            return Ok(false);
        }
        if missing {
            if !has_status("delete", TaskState::Deleted, role) {
                return Ok(false);
            }
        } else if digest(path, context)? != hash {
            return Ok(false);
        }
    }
    Ok(details.output_sha256.as_deref() == Some(digest(&request.output, context)?.as_str()))
}

pub(crate) fn digest(path: &Path, context: &ProcessContext) -> Result<String> {
    let mut file = std::fs::File::open(path)?;
    let mut hash = Sha256::new();
    let mut buffer = vec![0; 1024 * 1024];
    loop {
        context.check_cancelled()?;
        let n = file.read(&mut buffer)?;
        if n == 0 {
            break;
        }
        hash.update(&buffer[..n]);
    }
    Ok(format!("{:x}", hash.finalize()))
}

pub(crate) fn signature(
    a: &Path,
    b: &Path,
    options: &Icesat2Config,
    context: &ProcessContext,
    beams: &[String],
) -> Result<String> {
    Ok(format!(
        "match-v1:{}:{}:{}:{:?}",
        options.read_window_rows,
        digest(a, context)?,
        digest(b, context)?,
        beams
    ))
}

pub(crate) fn reusable(
    manifest: &Path,
    output: &Path,
    signature: &str,
    context: &ProcessContext,
) -> Result<bool> {
    if !output.is_file() {
        return Ok(false);
    }
    let Some(record) = ManifestStore::<XmlValue>::new(manifest).load()? else {
        return Ok(false);
    };
    let local = xt_manifest::display_path(&std::fs::canonicalize(output)?);
    let Some(file) = record.data.files.iter().find(|f| {
        f.local_path == local && f.tool == "xt-icesat2" && f.operation.as_deref() == Some("match")
    }) else {
        return Ok(false);
    };
    if !record.data.status.iter().any(|s| {
        s.id == "match"
            && s.state == TaskState::Completed
            && s.targets().contains(file.product.as_str())
    }) {
        return Ok(false);
    }
    let details: MatchDetails = file.details.decode()?;
    if details.workflow_signature.as_deref() != Some(signature) {
        return Ok(false);
    }
    Ok(details.output_sha256.as_deref() == Some(digest(output, context)?.as_str()))
}
