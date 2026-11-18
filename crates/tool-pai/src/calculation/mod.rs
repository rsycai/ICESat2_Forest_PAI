pub(crate) mod projection;

use crate::model::{FILL, Location, Row};
use std::collections::{BTreeMap, BTreeSet};

pub(crate) fn segment(
    loc: Location,
    samples: impl Iterator<Item = (f64, f32)>,
    g0: Option<f32>,
) -> Row {
    let points: Vec<_> = samples
        .filter(|(x, h)| x.is_finite() && h.is_finite() && h.abs() < 1e30 && x.abs() < 1e30)
        .collect();
    let mut result = Row {
        loc,
        pai: FILL,
        effective: FILL,
        ci: FILL,
        gap: FILL,
        qa: -128,
        status: 1,
    };
    if points.is_empty() {
        return result;
    }
    let origin = points.iter().map(|p| p.0).fold(f64::INFINITY, f64::min);
    let mut maxima = BTreeMap::<u64, f64>::new();
    let mut ground_windows = BTreeSet::new();
    let mut ground = 0;
    let mut span = 0.0f64;
    for &(x, h) in &points {
        let dist = x - origin;
        span = span.max(dist);
        let top = maxima.entry(dist.floor() as u64).or_default();
        *top = top.max(f64::from(h));
        if h < 2.0 {
            ground += 1;
            ground_windows.insert((dist / 10.0) as u64);
        }
    }
    let last = (span / 10.0) as u64;
    if last <= 11 {
        result.qa = ((last + 1).max(10) - ground_windows.len() as u64) as i8;
    }
    let gap = ground as f64 / points.len() as f64;
    result.gap = gap as f32;
    if ground < 10 || points.len() - ground < 10 || span < 75.0 {
        result.status = 2;
        return result;
    }
    let g0 = f64::from(g0.unwrap_or(0.5));
    let hist = xt_laipath::histogram_stats(maxima.into_values().collect(), 25, 0.0, 1.0);
    let estimate = hist.and_then(|hist| {
        Ok((
            xt_laipath::calc_eff_pai(gap, 0.0, g0)?,
            xt_laipath::calc_pai_path(&hist, gap, 0.0, g0)?,
        ))
    });
    result.status = 3;
    if let Ok((effective, pai)) = estimate
        && pai.is_finite()
        && pai > 0.0
    {
        result.pai = pai as f32;
        result.effective = effective as f32;
        result.ci = (effective / pai) as f32;
        result.status = 0;
    }
    result
}
