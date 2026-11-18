use crate::processing::config::Icesat2Config;
use crate::processing::matching::{build_match_plan, group_segments};
use crate::model::{BeamReport, ProcessRequest};
use crate::output;
use crate::processing::beams::select_beams;
use crate::processing::error::h5;
use crate::processing::orbit::verify_orbit;
use crate::processing::photon::PhotonSource;
use crate::processing::segments::load_segments;
use hdf5_pure::File;
use xt_error::Result;

pub(super) fn execute(
    request: &ProcessRequest,
    options: &Icesat2Config,
    key: &str,
    context: &xt_task::model::ProcessContext,
) -> Result<Vec<BeamReport>> {
    let a = File::open_streaming(&request.atl03).map_err(h5)?;
    let b = File::open_streaming(&request.atl08).map_err(h5)?;
    verify_orbit(&a, &b, key)?;
    let beams = select_beams(&a, &b, &request.beams)?;
    let mut builder = output::builder(request, options, key)?;
    output::global(&mut builder, &a, &b)?;
    let mut reports = Vec::new();
    for beam in beams {
        context.check_cancelled()?;
        let (geo, land, classified) = load_segments(&a, &b, &beam).map_err(|e| e.context(&beam))?;
        let source = PhotonSource::new(&a, &beam)?;
        let plan = build_match_plan(&geo, &land, &classified, source.count)?;
        let photons = source.read_matched(&plan, &geo, options)?;
        let result = group_segments(&land, photons, plan.stats, options)?;
        let counts = super::pulses::counts(&a, &beam, &geo, &result)?;
        output::add_beam(
            &mut builder,
            &beam,
            &result,
            counts.as_deref(),
            &a,
            &b,
            options,
        )?;
        reports.push(BeamReport {
            beam,
            stats: result.stats,
        });
    }
    context.check_cancelled()?;
    output::publish(builder, &request.output, request.overwrite)?;
    Ok(reports)
}
