use ndarray::{Array1, Array, ArrayBase, Data, Ix1};
use ndarray_linalg::solve::Inverse;
use rustfft::{FftPlanner, num_complex::Complex};
use std::f64::consts::PI;
use statrs::distribution::ChiSquared;
use ndarray_rand::RandomExt;
use ndarray_rand::rand_distr::Uniform;
use argmin::prelude::*;
//use argmin::ArgminOp;
// use argmin::core::Executor;
// use argmin::core::Error;
use argmin::solver::neldermead::NelderMead;
use statrs::distribution::ContinuousCDF;

struct SinFunc<'a> {
    tt: &'a Array1<f64>,
    yy: &'a Array1<f64>,
}

impl ArgminOp for SinFunc<'_> {
    type Param = Vec<f64>;
    type Output = f64;
    type Hessian = ();
    type Jacobian = ();
    type Float = f64;

    fn apply(&self, p: &Self::Param) -> Result<Self::Output, Error> {
        let (A, w, ph, c) = (p[0], p[1], p[2], p[3]);
        let sinfunc = |t: f64| A * (w * t + ph).sin() + c;
        let residuals: f64 = self.tt.iter().zip(self.yy.iter())
            .map(|(&t, &y)| (y - sinfunc(t)).powi(2))
            .sum();
        Ok(residuals)
    }
}

pub fn fit_sine(nanoseconds: &Array1<f64>, volts: &Array1<f64>) -> Result<(f64, f64, f64, f64, f64, f64), Box<dyn std::error::Error>> {
    let tt = nanoseconds;
    let yy = volts;

    // FFT
    let mut planner = FftPlanner::new();
    let fft = planner.plan_fft_forward(tt.len());
    let mut signal: Vec<Complex<f64>> = yy.iter().map(|&y| Complex::new(y, 0.0)).collect();
    fft.process(&mut signal);

    let mut magnitudes: Vec<f64> = signal.iter().map(|c| c.norm()).collect();
    magnitudes[0] = 0.0; // exclude zero frequency peak

    let guess_freq = (magnitudes.iter().cloned().enumerate().max_by(|a, b| a.1.partial_cmp(&b.1).unwrap()).unwrap().0 as f64) / tt.len() as f64;
    let guess_amp = yy.std(0.0) * 2f64.sqrt();
    let guess_offset = yy.mean().unwrap();
    let guess = vec![guess_amp, 2.0 * PI * guess_freq, 0.0, guess_offset];

    // Optimization
    let problem = SinFunc { tt, yy };
    let solver = NelderMead::new();
    let res = Executor::new(problem, solver, guess)
        .max_iters(1000)
        .run()?;

    let result = &res.state().best_param;
    let (A, w, p, c) = (result[0], result[1], result[2], result[3]);
    let phase_multiple_pi = p / PI;

    let fitfunc = |t: f64| -> f64 {
        A * (w * t + p).sin() + c
    };

    let residuals: Vec<f64> = tt.iter().zip(yy.iter()).map(|(&t, &y)| y - fitfunc(t)).collect();
    let ss_res = residuals.iter().map(|&r| r.powi(2)).sum::<f64>();
    let ss_tot = yy.iter().map(|&y| (y - yy.mean().unwrap()).powi(2)).sum::<f64>();
    //let r_squared = 1.0 - (ss_res / ss_tot);

    // Chi-squared calculation
    let expected_values: Vec<f64> = tt.iter().map(|&t| fitfunc(t)).collect();
    let observed_values = yy.to_vec();
    let chi_squared_stat: f64 = observed_values.iter().zip(expected_values.iter())
        .map(|(&o, &e)| (o - e).powi(2) / e).sum();

    let df = tt.len() as f64 - result.len() as f64;
    let reduced_chi_squared = chi_squared_stat / df;

    let chi_squared = ChiSquared::new(df).unwrap();
    let p_value = 1.0 - chi_squared.cdf(chi_squared_stat);

    let f = w / (2.0 * PI);
    let period = 1.0 / f;


    Ok((A, f, p, phase_multiple_pi, c, reduced_chi_squared))
    // println!("amp: {:.2}", A);
    // println!("omega: {:.2}", w);
    // println!("phase: {:.2}", p);
    // println!("phase_formatted: {:.2}π", phase_multiple_pi);
    // println!("offset: {:.2}", c);
    // println!("freq: {:.2}", f);
    // println!("period: {:.2}", period);
    // println!("r_squared: {:.2}", r_squared);
    // println!("chi_squared_stat: {:.2}", chi_squared_stat);
    // println!("p_value: {:.2}", p_value);
    // println!("reduced_chi_squared: {:.2}", reduced_chi_squared);

}