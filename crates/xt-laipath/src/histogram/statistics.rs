use crate::error::invalid;
use crate::histogram::model::Histogram;
use xt_error::Result;

///
pub fn histogram_stats(data: Vec<f64>, num_bins: usize, a: f64, b: f64) -> Result<Histogram> {
    if data.is_empty() || num_bins == 0 || num_bins == usize::MAX {
        return Err(invalid("Samples must be nonempty; histogram bin count must be positive and representable"));
    }
    if !a.is_finite() || !b.is_finite() || a < 0.0 || b <= a {
        return Err(invalid("Histogram bounds must be finite and satisfy 0 <= a < b"));
    }
    if data.iter().any(|v| !v.is_finite() || *v < 0.0) {
        return Err(invalid("Path samples must be finite and nonnegative"));
    }
    let max = data.iter().copied().fold(0.0, f64::max);
    if max == 0.0 {
        return Err(invalid("All path lengths are zero; cannot normalize by the maximum"));
    }
    let width = (b - a) / num_bins as f64;
    if width == 0.0 {
        return Err(invalid("Histogram bin width is too small"));
    }
    let mut range: Vec<_> = (0..=num_bins)
        .map(|i| a + (b - a) * (i as f64 / num_bins as f64))
        .collect();
    range[num_bins] = b;
    let mut bin = vec![0.0; num_bins];
    for value in data {
        let index = ((value / max * num_bins as f64) as usize).min(num_bins - 1);
        bin[index] += 1.0;
    }
    Histogram::new(bin, range)
}
