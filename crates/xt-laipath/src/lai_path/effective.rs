use crate::lai_path::validation::{finish, gap_check, geometry};
use xt_error::Result;

pub fn calc_eff_lai(gap_fraction: f64, zenith: f64, g: f64) -> Result<f64> {
    gap_check(gap_fraction)?;
    let scale = geometry(zenith, g)?;
    finish(-gap_fraction.ln() * scale)
}
