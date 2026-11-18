use crate::error::{invalid, numerical};
use crate::gsl::GSL_EBADFUNC;
use xt_error::Result;

pub(super) fn geometry(zenith: f64, g: f64) -> Result<f64> {
    if !zenith.is_finite() || !(0.0..90.0).contains(&zenith) {
        return Err(invalid("Zenith angle must be finite and in [0,90) degrees"));
    }
    if !(g.is_finite() && 0.0 < g && g <= 1.0) {
        return Err(invalid("Leaf projection coefficient G must be finite and in (0,1]"));
    }
    Ok(zenith.to_radians().cos() / g)
}

pub(super) fn gap_check(gap: f64) -> Result<()> {
    if !(gap.is_finite() && 0.0 < gap && gap <= 1.0) {
        return Err(invalid(
            "Gap fraction must be finite and in (0,1]; zero gap fraction implies infinite LAI",
        ));
    }
    Ok(())
}

pub(super) fn finish(value: f64) -> Result<f64> {
    if value.is_finite() && value >= 0.0 {
        Ok(value)
    } else {
        Err(numerical(GSL_EBADFUNC, "LAI result validation"))
    }
}
