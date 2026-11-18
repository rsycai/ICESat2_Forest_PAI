use crate::gsl::consts::{GSL_DBL_EPSILON, GSL_DBL_MAX};
use crate::gsl_max_dbl;

pub struct ExtrapolationTable {
    pub(crate) n: usize,
    rlist2: [f64; 52],
    nres: usize,
    res3la: [f64; 3],
}

impl ExtrapolationTable {
    #[inline]
    pub fn new() -> Self {
        Self {
            n: 0,
            rlist2: [0.; 52],
            nres: 0,
            res3la: [0.; 3],
        }
    }

    #[inline]
    pub fn append(&mut self, y: f64) {
        let n = self.n;
        self.rlist2[n] = y; // Append y to the extrapolation table.
        self.n += 1;
    }

    pub fn qelg(&mut self, result: &mut f64, abserr: &mut f64) {
        let epstab = &mut self.rlist2;
        let res3la = &mut self.res3la;
        let n = self.n - 1;

        let current = epstab[n];
        let mut absolute = f64::MAX;
        let relative = 5.0 * f64::EPSILON * current.abs();

        let newelm = n / 2;
        let n_orig = n;
        let mut n_final = n;

        *result = current;
        *abserr = f64::MAX;

        if n < 2 {
            *abserr = absolute.max(relative);
            return;
        }

        epstab[n + 2] = current;
        epstab[n] = f64::MAX;

        for i in 0..newelm {
            let res = epstab[n - 2 * i + 2];
            let e0 = epstab[n - 2 * i - 2];
            let e1 = epstab[n - 2 * i - 1];
            let e2 = res;

            let e1abs = e1.abs();
            let delta2 = e2 - e1;
            let err2 = delta2.abs();
            let tol2 = gsl_max_dbl!(e2.abs(), e1abs) * GSL_DBL_EPSILON;
            let delta3 = e1 - e0;
            let err3 = delta3.abs();
            let tol3 = gsl_max_dbl!(e0.abs(), e1abs) * GSL_DBL_EPSILON;

            if err2 <= tol2 && err3 <= tol3 {
                *result = res;
                absolute = err2 + err3;
                *abserr = gsl_max_dbl!(absolute, 5.0 * GSL_DBL_EPSILON * res.abs());
                return;
            }

            let e3 = epstab[n - 2 * i];
            epstab[n - 2 * i] = e1;
            let delta1 = e1 - e3;
            let err1 = delta1.abs();
            let tol1 = gsl_max_dbl!(e3.abs(), e1abs) * GSL_DBL_EPSILON;

            if err1 <= tol1 || err2 <= tol2 || err3 <= tol3 {
                n_final = 2 * i;
                break;
            }

            let ss = (1.0 / delta1 + 1.0 / delta2) - 1.0 / delta3;

            if (ss * e1).abs() <= 0.0001 {
                n_final = 2 * i;
                break;
            }

            let res = e1 + 1.0 / ss;
            epstab[n - 2 * i] = res;

            let error = err2 + (res - e2).abs() + err3;
            if error <= *abserr {
                *abserr = error;
                *result = res;
            }
        }

        let limexp = 50 - 1;

        if n_final == limexp {
            n_final = 2 * (limexp / 2);
        }

        if n_orig % 2 == 1 {
            for i in 0..=newelm {
                epstab[1 + i * 2] = epstab[i * 2 + 3];
            }
        } else {
            for i in 0..=newelm {
                epstab[i * 2] = epstab[i * 2 + 2];
            }
        }

        if n_orig != n_final {
            for i in 0..=n_final {
                epstab[i] = epstab[n_orig - n_final + i];
            }
        }

        self.n = n_final + 1;

        if self.nres < 3 {
            res3la[self.nres] = *result;
            *abserr = GSL_DBL_MAX;
        } else {
            *abserr = (*result - res3la[2]).abs()
                + (*result - res3la[1]).abs()
                + (*result - res3la[0]).abs();

            res3la[0] = res3la[1];
            res3la[1] = res3la[2];
            res3la[2] = *result;
        }

        self.nres += 1;
        *abserr = abserr.max(5.0 * GSL_DBL_EPSILON * result.abs());
    }
}
