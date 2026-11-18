use crate::processing::config::Icesat2Config;
use crate::workflow::manifest::MatchJournal;
use crate::model::{ProcessReport, ProcessRequest};
use crate::processing::paired_key;
use xt_error::{Error, ErrorKind, Result};

pub fn process_pair(request: &ProcessRequest, options: &Icesat2Config) -> Result<ProcessReport> {
    process_pair_with_context(request, options, &xt_task::model::ProcessContext::default())
}

pub fn process_pair_with_context(
    request: &ProcessRequest,
    options: &Icesat2Config,
    context: &xt_task::model::ProcessContext,
) -> Result<ProcessReport> {
    context.check_cancelled()?;
    options.validate()?;
    paired_key(&request.atl03, &request.atl08)?;
    let record_path = match &request.manifest {
        Some(path) => path.clone(),
        None => xt_icesat2_meta::manifest::record_path(&request.atl03, &request.atl08)?,
    };
    let lock_root = record_path
        .parent()
        .ok_or_else(|| Error::new(ErrorKind::Configuration, "Manifest path has no parent directory"))?
        .join(".locks");
    let _output_lock =
        xt_lock::FileLock::try_for_file(&request.output, lock_root).map_err(Error::from)?;
    if crate::workflow::manifest::recovery::completed_after_cleanup(request, options, context)? {
        tracing::info!(output=%xt_manifest::display_path(&request.output), "Data already processed and originals removed; skipping matching");
        return Ok(ProcessReport {
            skipped: true,
            pair_key: paired_key(&request.atl03, &request.atl08)?,
            output: request.output.clone(),
            manifest: record_path,
            beams: vec![],
        });
    }
    let path03 = std::fs::canonicalize(&request.atl03).map_err(Error::from)?;
    let path08 = std::fs::canonicalize(&request.atl08).map_err(Error::from)?;
    if request.output.exists() {
        let output = std::fs::canonicalize(&request.output).map_err(Error::from)?;
        if output == path03 || output == path08 {
            return Err(Error::new(
                ErrorKind::InvalidArgument,
                "Output path must not overwrite an input product",
            ));
        }
        if !request.overwrite {
            hdf5_reader::Hdf5File::open(&request.output).map_err(|error| {
                Error::new(
                    ErrorKind::Io(std::io::ErrorKind::InvalidData),
                    format!("Cannot read existing processed file: {error}"),
                )
            })?;
            tracing::info!(output=%xt_manifest::display_path(&request.output),
                "Processed file exists; skipping matching; enable overwrite to update");
            return Ok(ProcessReport {
                skipped: true,
                pair_key: paired_key(&request.atl03, &request.atl08)?,
                output: request.output.clone(),
                manifest: record_path,
                beams: vec![],
            });
        }
        tracing::info!(output=%xt_manifest::display_path(&request.output), "Processed file exists; matching again and updating result");
    }
    let signature =
        crate::workflow::manifest::recovery::signature(&path03, &path08, options, context, &request.beams)?;
    let journal = MatchJournal::begin(&path03, &path08, &request.output, record_path)?;
    let reports = match crate::processing::match_pair(request, options, context) {
        Ok(reports) => reports,
        Err(error) => {
            if let Err(record_error) = journal.fail(&error) {
                tracing::error!(code=record_error.code(),error=%record_error,"Failed to record matching failure status");
            }
            return Err(error);
        }
    };
    journal
        .complete(
            reports.iter().map(|b| b.stats.kept_photons).sum(),
            reports.iter().map(|b| b.stats.kept_segments).sum(),
            signature,
            crate::workflow::manifest::recovery::digest(
                &request.output,
                &xt_task::model::ProcessContext::default(),
            )?,
        )
        .map_err(|error| error.context("Result saved, but failed to create or update its manifest"))?;
    Ok(ProcessReport {
        skipped: false,
        pair_key: paired_key(&request.atl03, &request.atl08)?,
        output: request.output.clone(),
        manifest: journal.path,
        beams: reports,
    })
}
