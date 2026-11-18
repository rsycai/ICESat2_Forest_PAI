#![forbid(unsafe_code)]
pub mod cli;
pub mod model;
mod output;
pub mod processing;
pub mod workflow;

pub use cli::{Cli, params, run};
pub use cli::params::ProcessParams;
pub use model::{
    BEAMS, BeamReport, BeamResult, BeamStats, ClassifiedPhoton, GeoSegment, LandSegment, MatchPlan,
    MatchedIndex, Photon, ProcessReport, ProcessRequest, RetainedSegment,
};
pub use processing::{config, matching};
pub use processing::config::Icesat2Config;
pub use processing::matching::{build_match_plan, group_segments};
pub use workflow::{manifest, process_pair};
pub use workflow as integration;
pub use xt_error::{Error, ErrorKind, Result};
