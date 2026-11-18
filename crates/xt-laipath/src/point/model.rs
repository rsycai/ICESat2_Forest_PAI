use crate::error::{invalid, numerical};
use crate::gsl::GSL_EMAXITER;
use xt_error::Result;

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Point {
    pub x: f64,
    pub y: f64,
}

impl Point {
    pub fn new(x: f64, y: f64) -> Result<Self> {
        if !x.is_finite() || !y.is_finite() {
            return Err(invalid("Coordinates must be finite"));
        }
        Ok(Self { x, y })
    }
    pub fn euclidean(&self, rhs: &Self) -> Result<f64> {
        let value = (self.x - rhs.x).hypot(self.y - rhs.y);
        if value.is_finite() {
            Ok(value)
        } else {
            Err(invalid("Planar distance is not finite"))
        }
    }
    pub(super) fn check_geographic(&self) -> Result<()> {
        if !self.x.is_finite()
            || !self.y.is_finite()
            || !(-180.0..=180.0).contains(&self.x)
            || !(-90.0..=90.0).contains(&self.y)
        {
            return Err(invalid(
                "Geographic distance requires finite longitude in [-180,180] and latitude in [-90,90]",
            ));
        }
        Ok(())
    }
    pub fn haversine(&self, rhs: &Self) -> Result<f64> {
        self.check_geographic()?;
        rhs.check_geographic()?;
        let lat1 = self.y.to_radians();
        let lat2 = rhs.y.to_radians();
        let dlat = (self.y - rhs.y).to_radians();
        let dlon = (self.x - rhs.x).to_radians();
        let a = ((dlat / 2.0).sin().powi(2) + lat1.cos() * lat2.cos() * (dlon / 2.0).sin().powi(2))
            .clamp(0.0, 1.0);
        Ok(2.0 * 6371000.0 * a.sqrt().atan2((1.0 - a).sqrt()))
    }
    pub fn vincenty(&self, rhs: &Self) -> Result<f64> {
        self.check_geographic()?;
        rhs.check_geographic()?;
        const A: f64 = 6378137.0;
        const F: f64 = 1.0 / 298.257223563;
        const B: f64 = A * (1.0 - F);
        const ITER_COUNTS: i64 = 1000;

        let phi1 = self.y.to_radians();
        let phi2 = rhs.y.to_radians();
        let lambda1 = self.x.to_radians();
        let lambda2 = rhs.x.to_radians();

        let u1 = ((1.0 - F) * phi1.tan()).atan();
        let u2 = ((1.0 - F) * phi2.tan()).atan();
        let l = (lambda2 - lambda1 + std::f64::consts::PI).rem_euclid(std::f64::consts::TAU)
            - std::f64::consts::PI;

        let mut lambda = l;
        let mut lambda_prev;
        let mut iter_count = 0;
        let mut sin_sigma;
        let mut cos_sigma;
        let mut sigma;
        let mut sin_alpha;
        let mut cos2_alpha;
        let mut cos2_sigma_m;
        let mut c;

        loop {
            let sin_lambda = lambda.sin();
            let cos_lambda = lambda.cos();

            sin_sigma = ((u2.cos() * sin_lambda).powi(2)
                + (u1.cos() * u2.sin() - u1.sin() * u2.cos() * cos_lambda).powi(2))
            .sqrt();

            if sin_sigma == 0.0 {
                return Ok(0.0);
            }

            cos_sigma = u1.sin() * u2.sin() + u1.cos() * u2.cos() * cos_lambda;
            sigma = sin_sigma.atan2(cos_sigma);
            sin_alpha = u1.cos() * u2.cos() * sin_lambda / sin_sigma;
            cos2_alpha = 1.0 - sin_alpha.powi(2);

            if cos2_alpha == 0.0 {
                cos2_sigma_m = 0.0;
            } else {
                cos2_sigma_m = cos_sigma - 2.0 * u1.sin() * u2.sin() / cos2_alpha;
            }

            c = F / 16.0 * cos2_alpha * (4.0 + F * (4.0 - 3.0 * cos2_alpha));
            lambda_prev = lambda;
            lambda = l
                + (1.0 - c)
                    * F
                    * sin_alpha
                    * (sigma
                        + c * sin_sigma
                            * (cos2_sigma_m + c * cos_sigma * (-1.0 + 2.0 * cos2_sigma_m.powi(2))));

            iter_count += 1;
            if (lambda - lambda_prev).abs() < 1e-12 || iter_count >= ITER_COUNTS {
                break;
            }
        }

        if iter_count >= ITER_COUNTS && (lambda - lambda_prev).abs() >= 1e-12 {
            return Err(numerical(GSL_EMAXITER, "Vincenty distance iteration"));
        }

        let u2 = cos2_alpha * (A * A - B * B) / (B * B);
        let big_a = 1.0 + u2 / 16384.0 * (4096.0 + u2 * (-768.0 + u2 * (320.0 - 175.0 * u2)));
        let big_b = u2 / 1024.0 * (256.0 + u2 * (-128.0 + u2 * (74.0 - 47.0 * u2)));
        let delta_sigma = big_b
            * sin_sigma
            * (cos2_sigma_m
                + big_b / 4.0
                    * (cos_sigma * (-1.0 + 2.0 * cos2_sigma_m.powi(2))
                        - big_b / 6.0
                            * cos2_sigma_m
                            * (-3.0 + 4.0 * sin_sigma.powi(2))
                            * (-3.0 + 4.0 * cos2_sigma_m.powi(2))));

        let s = B * big_a * (sigma - delta_sigma);
        if s.is_finite() && s >= 0.0 {
            Ok(s)
        } else {
            Err(invalid("Ellipsoidal distance is not finite and nonnegative"))
        }
    }
}
