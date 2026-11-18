use crate::error::invalid;
use xt_error::Result;

///
#[derive(Debug, Clone)]
pub struct Histogram {
    pub(super) bin: Vec<f64>,
    pub(super) range: Vec<f64>,
}

impl Histogram {
    pub fn new(mut bin: Vec<f64>, range: Vec<f64>) -> Result<Self> {
        if bin.is_empty() || bin.len().checked_add(1) != Some(range.len()) {
            return Err(invalid(
                "Histogram requires at least one bin and one more boundary than density values",
            ));
        }
        if range.iter().any(|v| !v.is_finite() || *v < 0.0)
            || range.windows(2).any(|p| p[1] <= p[0])
        {
            return Err(invalid("Path length bounds must be finite, nonnegative, and strictly increasing"));
        }
        if bin.iter().any(|v| !v.is_finite() || *v < 0.0) {
            return Err(invalid("Probability density weights must be finite and nonnegative"));
        }
        let mass: f64 = bin
            .iter()
            .zip(range.windows(2))
            .map(|(d, r)| d * (r[1] - r[0]))
            .sum();
        if !mass.is_finite() || mass <= 0.0 {
            return Err(invalid("Histogram integral mass must be finite and positive"));
        }
        for density in &mut bin {
            *density /= mass;
        }
        if bin.iter().any(|v| !v.is_finite()) {
            return Err(invalid("Interval is too narrow; normalized probability density overflowed"));
        }
        Ok(Self { bin, range })
    }
    pub fn bin(&self) -> &[f64] {
        &self.bin
    }
    pub fn range(&self) -> &[f64] {
        &self.range
    }
    ///
    pub fn probability(&self, value: f64) -> Result<f64> {
        if !value.is_finite() {
            return Err(invalid("Query path length must be finite"));
        }
        Ok(self.density(value))
    }
    pub(crate) fn density(&self, value: f64) -> f64 {
        let last = self.range[self.bin.len()];
        if value < self.range[0] || value > last {
            return 0.0;
        }
        if value == last {
            return self.bin[self.bin.len() - 1];
        }
        let index = self.range.partition_point(|&edge| edge <= value) - 1;
        self.bin[index]
    }
}
