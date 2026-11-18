use crate::{Icesat2Config, ProcessRequest};
use xt_error::{Error, ErrorKind, Result};
use xt_task::{
    interfaces::Processor,
    model::{DataGroup, LocalFile, ProcessContext},
};

pub struct Icesat2Processor {
    pub options: Icesat2Config,
    pub paths: xt_config::paths::DataPaths,
    pub overwrite: bool,
    pub delete_inputs_after_match: bool,
}

impl Icesat2Processor {
    fn request(&self, group: &DataGroup) -> Result<ProcessRequest> {
        self.validate(group)?;
        let path = |role| {
            &group
                .files
                .iter()
                .find(|f| f.role == role)
                .expect("Validated role")
                .path
        };
        let (a, b) = (path("ATL03"), path("ATL08"));
        Ok(ProcessRequest {
            atl03: a.clone(),
            atl08: b.clone(),
            output: crate::processing::dated_output_path(&self.paths.processed, a, b)?,
            manifest: Some(xt_icesat2_meta::manifest::record_path_in(
                &self.paths.manifests,
                a,
                b,
            )?),
            beams: vec![],
            overwrite: self.overwrite,
        })
    }
}

impl Processor for Icesat2Processor {
    fn can_skip(&self, group: &DataGroup, context: &ProcessContext) -> Result<bool> {
        let request = self.request(group)?;
        if request.overwrite || !request.output.try_exists()? {
            return Ok(false);
        }
        if !request.atl03.try_exists()? || !request.atl08.try_exists()? {
            return crate::workflow::manifest::recovery::completed_after_cleanup(
                &request,
                &self.options,
                context,
            );
        }
        let manifest = request.manifest.as_deref().expect("Parsed manifest");
        if !manifest.try_exists()? {
            return Ok(false);
        }
        let signature = crate::workflow::manifest::recovery::signature(
            &request.atl03,
            &request.atl08,
            &self.options,
            context,
            &[],
        )?;
        crate::workflow::manifest::recovery::reusable(
            manifest,
            &request.output,
            &signature,
            context,
        )
    }
    fn id(&self) -> &str {
        "match"
    }

    fn validate(&self, group: &DataGroup) -> Result<()> {
        for role in ["ATL03", "ATL08"] {
            if group.files.iter().filter(|f| f.role == role).count() != 1 {
                return Err(Error::new(
                    ErrorKind::InvalidArgument,
                    format!("Matching requires exactly one {role} file"),
                ));
            }
        }
        Ok(())
    }

    fn process(&self, mut group: DataGroup, context: &ProcessContext) -> Result<DataGroup> {
        let request = self.request(&group)?;
        let output = request.output.clone();
        let manifest = request.manifest.as_deref().expect("Parsed manifest");
        if crate::workflow::manifest::recovery::completed_after_cleanup(
            &request,
            &self.options,
            context,
        )? {
            tracing::info!(group=%group.id, "Data already processed and originals removed; skipping matching");
            group
                .files
                .retain(|file| !matches!(file.role.as_str(), "ATL03" | "ATL08" | "ATL03_08"));
            group.files.push(LocalFile {
                id: "ATL03_08".into(),
                role: "ATL03_08".into(),
                size_bytes: std::fs::metadata(&output)?.len(),
                path: output,
            });
            return Ok(group);
        }
        let signature = crate::workflow::manifest::recovery::signature(
            &request.atl03,
            &request.atl08,
            &self.options,
            context,
            &[],
        )?;
        if !self.overwrite
            && crate::workflow::manifest::recovery::reusable(
                manifest, &output, &signature, context,
            )?
        {
            tracing::info!(group=%group.id, "Existing processed file and inputs verified; reusing matched result");
        } else {
            let report =
                crate::workflow::pair::process_pair_with_context(&request, &self.options, context)?;
            if report.skipped {
                return Ok(group);
            }
        }
        if self.delete_inputs_after_match {
            crate::workflow::cleanup::delete_inputs(
                manifest,
                &output,
                &signature,
                &self.paths.downloads,
                &group.files,
                context,
            )?;
            group
                .files
                .retain(|file| !matches!(file.role.as_str(), "ATL03" | "ATL08"));
        }
        group.files.retain(|f| f.id != "ATL03_08");
        group.files.push(LocalFile {
            id: "ATL03_08".into(),
            role: "ATL03_08".into(),
            size_bytes: std::fs::metadata(&output)?.len(),
            path: output,
        });
        Ok(group)
    }
}
