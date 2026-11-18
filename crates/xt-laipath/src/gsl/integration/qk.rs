use crate::gsl::err::rescale_error;
use crate::gsl::function::GslFunction;
use crate::gsl_fn_eval;

#[allow(clippy::excessive_precision)]
const XGK: [f64; 11] = [
    0.995657163025808080735527280689003,
    0.973906528517171720077964012084452,
    0.930157491355708226001207180059508,
    0.865063366688984510732096688423493,
    0.780817726586416897063717578345042,
    0.679409568299024406234327365114874,
    0.562757134668604683339000099272694,
    0.433395394129247190799265943165784,
    0.294392862701460198131126603103866,
    0.148874338981631210884826001129720,
    0.000000000000000000000000000000000,
];

#[allow(clippy::excessive_precision)]
const WG: [f64; 5] = [
    0.066671344308688137593568809893332,
    0.149451349150580593145776339657697,
    0.219086362515982043995534934228163,
    0.269266719309996355091226921569469,
    0.295524224714752870173892994651338,
];

#[allow(clippy::excessive_precision)]
const WGK: [f64; 11] = [
    0.011694638867371874278064396062192,
    0.032558162307964727478818972459390,
    0.054755896574351996031381300244580,
    0.075039674810919952767043140916190,
    0.093125454583697605535065465083366,
    0.109387158802297641899210590325805,
    0.123491976262065851077958109831074,
    0.134709217311473325928054001771707,
    0.142775938577060080797094273138717,
    0.147739104901338491374841515972068,
    0.149445554002916905664936468389821,
];

pub type GslIntegrationRule<T> = fn(GslFunction<T>, f64, f64) -> (f64, f64, f64, f64);

#[inline]
pub fn gsl_integration_qk21<T: Clone>(f: GslFunction<T>, a: f64, b: f64) -> (f64, f64, f64, f64) {
    let mut fv1: [f64; 11] = [0_f64; 11];
    let mut fv2: [f64; 11] = [0_f64; 11];
    gsl_integration_qk(11, &XGK, &WG, &WGK, &mut fv1, &mut fv2, f, a, b)
}

fn gsl_integration_qk<T: Clone>(
    n: usize,
    xgk: &[f64],
    wg: &[f64],
    wgk: &[f64],
    fv1: &mut [f64],
    fv2: &mut [f64],
    f: GslFunction<T>,
    a: f64,
    b: f64,
) -> (f64, f64, f64, f64) {
    let center = 0.5 * a + 0.5 * b;
    let half_length = 0.5 * b - 0.5 * a;
    let abs_half_length = half_length.abs();
    let f_center = gsl_fn_eval!(f.clone(), center);

    let mut result_gauss = 0.0;
    let mut result_kronrod = f_center * wgk[n - 1];

    let mut result_abs = result_kronrod.abs();
    let mut result_asc;

    if n % 2 == 0 {
        result_gauss = f_center * wg[n / 2 - 1];
    }

    for (j, &gauss_weight) in wg.iter().take((n - 1) / 2).enumerate() {
        let jtw = j * 2 + 1; // j=1,2,3 -> jtw=2,4,6
        let abscissa = half_length * xgk[jtw];
        let fval1 = gsl_fn_eval!(f.clone(), center - abscissa);
        let fval2 = gsl_fn_eval!(f.clone(), center + abscissa);
        let fsum = fval1 + fval2;
        fv1[jtw] = fval1;
        fv2[jtw] = fval2;
        result_gauss += gauss_weight * fsum;
        result_kronrod += wgk[jtw] * fsum;
        result_abs += wgk[jtw] * (fval1.abs() + fval2.abs());
    }

    for j in 0..n / 2 {
        let jtwm1 = j * 2;
        let abscissa = half_length * xgk[jtwm1];
        let fval1 = gsl_fn_eval!(f.clone(), center - abscissa);
        let fval2 = gsl_fn_eval!(f.clone(), center + abscissa);
        fv1[jtwm1] = fval1;
        fv2[jtwm1] = fval2;
        result_kronrod += wgk[jtwm1] * (fval1 + fval2);
        result_abs += wgk[jtwm1] * (fval1.abs() + fval2.abs());
    }

    let mean = result_kronrod * 0.5;

    result_asc = wgk[n - 1] * (f_center - mean).abs();

    for j in 0..(n - 1) {
        result_asc += wgk[j] * ((fv1[j] - mean).abs() + (fv2[j] - mean).abs());
    }

    let err = (result_kronrod - result_gauss) * half_length;

    result_kronrod *= half_length;
    result_abs *= abs_half_length;
    result_asc *= abs_half_length;

    (
        result_kronrod,
        rescale_error(err, result_abs, result_asc),
        result_abs,
        result_asc,
    )
}
