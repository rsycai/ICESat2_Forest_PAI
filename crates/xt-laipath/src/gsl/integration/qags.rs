use crate::gsl::consts::{
    GSL_DBL_EPSILON, GSL_DBL_MAX, GSL_EBADFUNC, GSL_EBADTOL, GSL_EINVAL, GSL_EMAXITER, GSL_EROUND,
    GSL_SUCCESS,
};
use crate::gsl::err::fmt_err;
use crate::gsl::function::GslFunction;
use crate::gsl::integration::extrapolation_table::ExtrapolationTable;
use crate::gsl::integration::qk::{GslIntegrationRule, gsl_integration_qk21};
use crate::gsl::integration::workspace::GslIntegrationWorkspace;
use crate::gsl::test_positivity::test_positivity;
use crate::gsl::util::subinterval_too_small;
use crate::{gsl_error, gsl_max_dbl};

///
pub fn gsl_integration_qags<T: Clone>(
    f: &GslFunction<T>,
    a: f64,
    b: f64,
    epsabs: f64,
    epsrel: f64,
    limit: usize,
    result: &mut f64,
    abserr: &mut f64,
) -> i32 {
    qags(
        f,
        a,
        b,
        epsabs,
        epsrel,
        limit,
        result,
        abserr,
        gsl_integration_qk21,
    )
}

fn qags<T: Clone>(
    f: &GslFunction<T>,
    a: f64,
    b: f64,
    epsabs: f64,
    epsrel: f64,
    limit: usize,
    result: &mut f64,
    abserr: &mut f64,
    q: GslIntegrationRule<T>,
) -> i32 {
    *result = 0.0;
    *abserr = 0.0;
    if limit == 0 {
        return GSL_EINVAL;
    }
    if !epsabs.is_finite() || !epsrel.is_finite() || epsabs < 0.0 || epsrel < 0.0 {
        return GSL_EBADTOL;
    }
    if !a.is_finite() || !b.is_finite() {
        return GSL_EINVAL;
    }

    let (mut area, mut errsum): (f64, f64);
    let (mut res_ext, mut err_ext): (f64, f64);
    let mut tolerance: f64;

    let mut ertest = 0.0;
    let mut error_over_large_intervals = 0.0;
    let (mut reseps, mut abseps, mut correc) = (0.0, 0.0, 0.0);
    let mut ktmin = 0;
    let mut iteration;
    let (mut roundoff_type1, mut roundoff_type2, mut roundoff_type3) = (0, 0, 0);
    let (mut error_type, mut error_type2) = (0, 0);

    let mut extrapolate = 0;
    let mut disallow_extrapolation = 0;

    let mut table = ExtrapolationTable::new();

    /* Initialize outputs. */
    let mut workspace = GslIntegrationWorkspace::new(limit);
    workspace.initialise(a, b);

    *result = 0.;
    *abserr = 0.;

    /* Validate parameters. */
    if limit > workspace.limit {
        gsl_error!("iteration limit exceeds available workspace", GSL_EINVAL);
    }

    if epsabs <= 0. && (epsrel < 50. * GSL_DBL_EPSILON || epsrel < 0.5e-28) {
        gsl_error!(
            "tolerance cannot be achieved with given epsabs and epsrel",
            GSL_EBADTOL
        );
    }

    /* Perform the initial quadrature. */
    let (result0, abserr0, resabs0, resasc0) = q(f.clone(), a, b);
    if [result0, abserr0, resabs0, resasc0]
        .iter()
        .any(|v| !v.is_finite())
    {
        return GSL_EBADFUNC;
    }

    workspace.set_initial_result(result0, abserr0);

    tolerance = gsl_max_dbl!(epsabs, epsrel * result0.abs());

    if abserr0 <= 100.0 * GSL_DBL_EPSILON * resabs0 && abserr0 > tolerance {
        *result = result0;
        *abserr = abserr0;

        gsl_error!(
            "cannot reach tolerance because of roundoff error on first attempt",
            GSL_EROUND
        );
    } else if (abserr0 <= tolerance && abserr0 != resasc0) || abserr0 == 0.0 {
        *result = result0;
        *abserr = abserr0;
        return GSL_SUCCESS;
    } else if limit == 1 {
        *result = result0;
        *abserr = abserr0;
        gsl_error!("a maximum of one iteration was insufficient", GSL_EMAXITER);
    }

    table.append(result0);

    area = result0;
    errsum = abserr0;

    res_ext = result0;
    err_ext = GSL_DBL_MAX;

    let positive_integrand = test_positivity(result0, resabs0);
    iteration = 1;

    loop {
        let (a1, b1, a2, b2): (f64, f64, f64, f64);
        let (mut a_i, mut b_i, mut r_i, mut e_i) = (0.0, 0.0, 0.0, 0.0);

        /* Bisect the subinterval with the largest error. */
        workspace.retrieve(&mut a_i, &mut b_i, &mut r_i, &mut e_i);

        let current_level: usize = workspace.level[workspace.i] + 1;

        a1 = a_i;
        b1 = 0.5 * a_i + 0.5 * b_i;
        a2 = b1;
        b2 = b_i;

        iteration += 1;

        let (area1, error1, _resabs1, resasc1) = q(f.clone(), a1, b1);
        if [area1, error1, _resabs1, resasc1]
            .iter()
            .any(|v| !v.is_finite())
        {
            return GSL_EBADFUNC;
        }
        let (area2, error2, _resabs2, resasc2) = q(f.clone(), a2, b2);
        if [area2, error2, _resabs2, resasc2]
            .iter()
            .any(|v| !v.is_finite())
        {
            return GSL_EBADFUNC;
        }

        let area12 = area1 + area2;
        let error12 = error1 + error2;
        let last_e_i: f64 = e_i;


        errsum = errsum + error12 - e_i;
        area = area + area12 - r_i;

        tolerance = gsl_max_dbl!(epsabs, epsrel * area.abs());

        if resasc1 != error1 && resasc2 != error2 {
            let delta = r_i - area12;

            if delta.abs() <= 1.0e-5 * area12.abs() && error12 >= 0.99 * e_i {
                if extrapolate == 0 {
                    roundoff_type1 += 1;
                } else {
                    roundoff_type2 += 1;
                }
            }

            if iteration > 10 && error12 > e_i {
                roundoff_type3 += 1;
            }
        }

        /* Check roundoff error and update status. */
        if roundoff_type1 + roundoff_type2 >= 10 || roundoff_type3 >= 20 {
            error_type = 2; // round off error
        }

        if roundoff_type2 >= 5 {
            error_type2 = 1;
        }

        if subinterval_too_small(a1, a2, b2) != 0 {
            error_type = 4;
        }

        workspace.update(a1, b1, area1, error1, a2, b2, area2, error2);

        if errsum <= tolerance {
            *result = workspace.sum_results();
            *abserr = errsum;
            return fmt_err(error_type);
        }

        if error_type != 0 {
            break;
        }

        if iteration >= limit - 1 {
            error_type = 1;
            break;
        }

        if iteration == 2 {
            /* set up variables on first iteration */
            error_over_large_intervals = errsum;
            ertest = tolerance;
            table.append(area);
            continue;
        }

        if disallow_extrapolation != 0 {
            continue;
        }

        error_over_large_intervals += -last_e_i;

        if current_level < workspace.maximum_level {
            error_over_large_intervals += error12;
        }

        if extrapolate == 0 {
            // Test whether the interval to be bisected next is the smallest interval.
            if workspace.large_interval() != 0 {
                continue;
            }

            extrapolate = 1;
            workspace.nrmax = 1;
        }

        // The smallest interval has the largest error. Before bisecting, decrease the sum of the errors over the larger intervals and perform extrapolation.
        if error_type2 == 0
            && error_over_large_intervals > ertest
            && workspace.increase_nrmax() != 0
        {
            continue;
        }

        table.append(area);

        table.qelg(&mut reseps, &mut abseps);

        ktmin += 1;

        if ktmin > 5 && err_ext < 0.001 * errsum {
            error_type = 5;
        }

        if abseps < err_ext {
            ktmin = 0;
            err_ext = abseps;
            res_ext = reseps;
            correc = error_over_large_intervals;
            ertest = gsl_max_dbl!(epsabs, epsrel * reseps.abs());
            if err_ext <= ertest {
                break;
            }
        }

        /* Prepare to bisect the smallest subinterval. */
        if table.n == 1 {
            disallow_extrapolation = 1;
        }

        if error_type == 5 {
            break;
        }
        /* Continue with the interval having the largest error. */
        workspace.reset_nrmax();
        extrapolate = 0;
        error_over_large_intervals = errsum;

        if iteration >= limit {
            break;
        }
    }

    *result = res_ext;
    *abserr = err_ext;

    if err_ext == GSL_DBL_MAX {
        *result = workspace.sum_results();
        *abserr = errsum;
        return fmt_err(error_type);
    }

    if error_type != 0 || error_type2 != 0 {
        if error_type2 != 0 {
            err_ext += correc;
        }

        if error_type == 0 {
            error_type = 3;
        }

        if *result != 0.0 && area != 0.0 {
            if err_ext / res_ext.abs() > errsum / area.abs() {
                *result = workspace.sum_results();
                *abserr = errsum;
                return fmt_err(error_type);
            }
        } else if err_ext > errsum {
            *result = workspace.sum_results();
            *abserr = errsum;

            return fmt_err(error_type);
        } else if area == 0.0 {
            return fmt_err(error_type);
        }
    }

    /*  Check for divergence. */
    {
        {
            let max_area = gsl_max_dbl!(res_ext.abs(), area.abs());

            if positive_integrand == 0 && max_area < 0.01 * resabs0 {
                return fmt_err(error_type);
            }
        }
    }

    {
        let ratio = res_ext / area;

        if !(0.01..=100.0).contains(&ratio) || errsum > area.abs() {
            error_type = 6;
        }
    }

    // *result = workspace.sum_results();
    // *abserr = errsum;

    fmt_err(error_type)
}
