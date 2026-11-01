// This file is part of gaps-online-software and published 
// under the GPLv3 license

use crate::prelude::*;

use argmin::core::{
  CostFunction,
  Executor,
  State,
  TerminationReason,
};
use argmin::solver::neldermead::NelderMead;


#[derive(Debug, Copy, Clone, PartialEq,FromRepr, AsRefStr, EnumIter)]
#[repr(u8)]
#[cfg_attr(feature = "pybindings", pyclass(eq, eq_int))]
pub enum FitStatus {
  Unknown        = 0u8,
  DidNotConverge = 10u8,
  Success        = 42u8,
}

expand_and_test_enum!(FitStatus, test_fitstatus_repr);


/// Describe line depending on z since that is our
/// best constrained value
///
/// This model has 6 free parameters, 3 for 
/// the anchor point and 3 for the direction
pub fn line3d(z : f32, x_a : f32, y_a : f32, z_a : f32, dx : f32, dy : f32, dz : f32) -> (f32, f32, f32) {
  // avoid zero division error 
  //let dx_nz : f32 = dx if dx !=0 {dx} else { 1e-5 };
  //let dy_nz : f32 = dy if dy !=0 {dy} else { 1e-5 };
  let dz_nz : f32 = if dz != 0.0 {dz} else { 1e-5 };
  let x = x_a + ((dx/dz_nz) * (z - z_a));
  let y = y_a + ((dy/dz_nz) * (z - z_a));
  (x,y,z)
}

//------------------------------------------------------------

/// Default error on the x/y coordinates of a hit, in case
/// no errors are given (same as in the python version)
pub const LINEFIT_DEFAULT_ERR_XY : f32 = 10.0;

/// A straight line through a pointcloud, parametrized
/// in z, since that is our best constrained value
///
/// x(z) = x0 + tx * (z - z0)
/// y(z) = y0 + ty * (z - z0)
///
/// The anchor z0 is fixed to the mean z of the hits, so
/// the line has 4 free parameters (x0, y0, tx, ty). The
/// 6 parameter model of line3d is degenerate (the anchor
/// can slide along the line and the direction vector can
/// be scaled), which only introduces flat directions
/// for a minimizer.
///
/// For the standard chi2 (sum of squared, normalized
/// residuals) the fit decouples into two independent
/// weighted linear regressions x(z) and y(z), which
/// can be solved analytically (LineFit::fit).
///
/// The python version minimizes the sum of the normalized
/// distances instead (sum of sqrt(rx^2/sx^2 + ry^2/sy^2)),
/// which is less sensitive to outliers, but has no closed
/// form solution. This is available through
/// LineFit::fit_sum_of_distances, which uses the analytic
/// solution as a seed for a Nelder-Mead minimization.
#[derive(Debug, Copy, Clone, PartialEq)]
#[cfg_attr(feature="pybindings", pyclass)]
pub struct LineFit {
  /// anchor point
  pub x0       : f32,
  pub y0       : f32,
  pub z0       : f32,
  /// slopes dx/dz and dy/dz
  pub tx       : f32,
  pub ty       : f32,
  /// parameter errors (from the analytic chi2 fit)
  pub x0_err   : f32,
  pub y0_err   : f32,
  pub tx_err   : f32,
  pub ty_err   : f32,
  /// sum of the squared, normalized residuals
  pub chi2     : f32,
  /// value of the minimized cost function. For the
  /// analytic fit this is the same as chi2
  pub cost     : f32,
  /// 2*npoints - 4
  pub ndof     : u32,
  pub npoints  : u32,
  pub status   : FitStatus,
}

impl LineFit {
  pub fn new() -> Self {
    Self {
      x0       : 0.0,
      y0       : 0.0,
      z0       : 0.0,
      tx       : 0.0,
      ty       : 0.0,
      x0_err   : 0.0,
      y0_err   : 0.0,
      tx_err   : 0.0,
      ty_err   : 0.0,
      chi2     : 0.0,
      cost     : 0.0,
      ndof     : 0,
      npoints  : 0,
      status   : FitStatus::Unknown,
    }
  }

  /// Analytic, weighted least squares fit of a straight line
  /// to the given points.
  ///
  /// Returns None if there are less than 2 points, or all
  /// points have the same z (the line can not be described
  /// as a function of z)
  ///
  /// # Arguments:
  ///   * errs_x, errs_y : errors on x and y for each point.
  ///                      If None, LINEFIT_DEFAULT_ERR_XY
  ///                      will be used
  pub fn fit(xs     : &[f32],
             ys     : &[f32],
             zs     : &[f32],
             errs_x : Option<&[f32]>,
             errs_y : Option<&[f32]>) -> Option<Self> {
    let points = LineFitPoints::new(xs, ys, zs, errs_x, errs_y)?;
    let (x0, tx, x0_var, tx_var) = points.regression(&points.xs, &points.wx)?;
    let (y0, ty, y0_var, ty_var) = points.regression(&points.ys, &points.wy)?;
    let mut lf  = Self::new();
    lf.z0       = points.z0 as f32;
    lf.x0       = x0 as f32;
    lf.y0       = y0 as f32;
    lf.tx       = tx as f32;
    lf.ty       = ty as f32;
    lf.x0_err   = x0_var.sqrt() as f32;
    lf.y0_err   = y0_var.sqrt() as f32;
    lf.tx_err   = tx_var.sqrt() as f32;
    lf.ty_err   = ty_var.sqrt() as f32;
    lf.chi2     = points.chi2(&[x0, y0, tx, ty]) as f32;
    lf.cost     = lf.chi2;
    lf.npoints  = xs.len() as u32;
    lf.ndof     = 2*lf.npoints - 4;
    lf.status   = FitStatus::Success;
    Some(lf)
  }

  /// Fit a straight line to the given points by minimizing
  /// the sum of the normalized distances, as done in the
  /// python version of the linefit.
  ///
  /// The analytic chi2 fit is used as a seed for the
  /// minimization with the Nelder-Mead algorithm. The parameter
  /// errors are the ones of the seed.
  pub fn fit_sum_of_distances(xs     : &[f32],
                              ys     : &[f32],
                              zs     : &[f32],
                              errs_x : Option<&[f32]>,
                              errs_y : Option<&[f32]>) -> Option<Self> {
    let mut lf  = Self::fit(xs, ys, zs, errs_x, errs_y)?;
    let points  = LineFitPoints::new(xs, ys, zs, errs_x, errs_y)?;
    let seed    = vec![lf.x0 as f64, lf.y0 as f64, lf.tx as f64, lf.ty as f64];
    // initial simplex - step sizes from the seed errors, so
    // that they come with the right scale for each parameter
    let steps   = [lf.x0_err as f64, lf.y0_err as f64, lf.tx_err as f64, lf.ty_err as f64];
    let mut simplex = vec![seed.clone()];
    for k in 0..seed.len() {
      let mut vertex = seed.clone();
      vertex[k] += if steps[k] > 0.0 { steps[k] } else { 1e-3 };
      simplex.push(vertex);
    }
    let solver = match NelderMead::new(simplex).with_sd_tolerance(1e-8) {
      Err(err) => {
        error!("Unable to set up Nelder-Mead solver! {err}");
        lf.status = FitStatus::DidNotConverge;
        return Some(lf);
      }
      Ok(solver) => solver
    };
    let result = Executor::new(points.clone(), solver)
      .configure(|state| state.max_iters(2000))
      .run();
    match result {
      Err(err) => {
        error!("Minimization failed! {err}");
        lf.status = FitStatus::DidNotConverge;
      }
      Ok(res) => {
        let state = res.state();
        if let Some(best) = state.get_best_param() {
          lf.x0   = best[0] as f32;
          lf.y0   = best[1] as f32;
          lf.tx   = best[2] as f32;
          lf.ty   = best[3] as f32;
          lf.chi2 = points.chi2(best) as f32;
          lf.cost = state.get_best_cost() as f32;
        }
        lf.status = match state.get_termination_reason() {
          Some(TerminationReason::SolverConverged) => FitStatus::Success,
          _                                        => FitStatus::DidNotConverge,
        };
      }
    }
    Some(lf)
  }

  /// Evaluate the line at a given z
  pub fn eval(&self, z : f32) -> (f32, f32, f32) {
    (self.x0 + self.tx*(z - self.z0), self.y0 + self.ty*(z - self.z0), z)
  }

  /// Unit direction vector of the line, pointing
  /// towards +z
  pub fn direction(&self) -> (f32, f32, f32) {
    let norm = (self.tx*self.tx + self.ty*self.ty + 1.0).sqrt();
    (self.tx/norm, self.ty/norm, 1.0/norm)
  }

  /// Cosine of the zenith angle of the line
  pub fn cos_theta(&self) -> f32 {
    self.direction().2
  }

  /// The parameters of the line in the form as expected
  /// by line3d (x_a, y_a, z_a, dx, dy, dz)
  pub fn line3d_params(&self) -> (f32, f32, f32, f32, f32, f32) {
    (self.x0, self.y0, self.z0, self.tx, self.ty, 1.0)
  }
}

impl fmt::Display for LineFit {
  fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
    let mut repr = String::from("<LineFit");
    repr += &(format!("\n  status {}", self.status));
    repr += &(format!("\n  anchor x0 {:.2} +- {:.2} y0 {:.2} +- {:.2} z0 {:.2}",
                      self.x0, self.x0_err, self.y0, self.y0_err, self.z0));
    repr += &(format!("\n  slopes tx {:.4} +- {:.4} ty {:.4} +- {:.4}",
                      self.tx, self.tx_err, self.ty, self.ty_err));
    repr += &(format!("\n  chi2 {:.2} / ndof {} (cost {:.2}, npoints {})>",
                      self.chi2, self.ndof, self.cost, self.npoints));
    write!(f,"{}", repr)
  }
}

//------------------------------------------------------------

/// The points (and their weights) a LineFit is performed on.
/// Also serves as the cost function for the minimization.
#[derive(Debug, Clone)]
struct LineFitPoints {
  xs : Vec<f64>,
  ys : Vec<f64>,
  zs : Vec<f64>,
  /// weights 1/err^2
  wx : Vec<f64>,
  wy : Vec<f64>,
  /// common anchor z for x(z) and y(z)
  z0 : f64,
}

impl LineFitPoints {
  fn new(xs     : &[f32],
         ys     : &[f32],
         zs     : &[f32],
         errs_x : Option<&[f32]>,
         errs_y : Option<&[f32]>) -> Option<Self> {
    if xs.len() != ys.len() || xs.len() != zs.len() {
      error!("Got {} x, {} y and {} z values! They need to be of the same length!", xs.len(), ys.len(), zs.len());
      return None;
    }
    if xs.len() < 2 {
      error!("Not enough points! Need at least 2, got {}", xs.len());
      return None;
    }
    let weights = |errs : Option<&[f32]>| -> Option<Vec<f64>> {
      match errs {
        None    => Some(vec![1.0/(LINEFIT_DEFAULT_ERR_XY as f64).powi(2); xs.len()]),
        Some(e) => {
          if e.len() != xs.len() {
            error!("Got {} errors for {} points!", e.len(), xs.len());
            return None;
          }
          Some(e.iter().map(|k| 1.0/(*k as f64).powi(2)).collect())
        }
      }
    };
    let zs : Vec<f64> = zs.iter().map(|k| *k as f64).collect();
    let z0 = zs.iter().sum::<f64>()/zs.len() as f64;
    Some(Self {
      xs : xs.iter().map(|k| *k as f64).collect(),
      ys : ys.iter().map(|k| *k as f64).collect(),
      zs,
      wx : weights(errs_x)?,
      wy : weights(errs_y)?,
      z0,
    })
  }

  /// Weighted linear regression v(z) = a + b*(z - z0)
  ///
  /// Returns a, b, var(a), var(b) or None if all z
  /// are the same
  fn regression(&self, vs : &[f64], ws : &[f64]) -> Option<(f64, f64, f64, f64)> {
    let sw   : f64 = ws.iter().sum();
    let zbar : f64 = ws.iter().zip(&self.zs).map(|(w,z)| w*z).sum::<f64>()/sw;
    let vbar : f64 = ws.iter().zip(vs).map(|(w,v)| w*v).sum::<f64>()/sw;
    let mut szz = 0.0f64;
    let mut szv = 0.0f64;
    for k in 0..vs.len() {
      szz += ws[k]*(self.zs[k] - zbar).powi(2);
      szv += ws[k]*(self.zs[k] - zbar)*(vs[k] - vbar);
    }
    if szz <= f64::EPSILON*sw {
      error!("All points have the same z, can not fit a line as a function of z!");
      return None;
    }
    let b     = szv/szz;
    // shift the anchor from the weighted mean zbar to z0
    let a     = vbar + b*(self.z0 - zbar);
    let var_b = 1.0/szz;
    let var_a = 1.0/sw + (self.z0 - zbar).powi(2)/szz;
    Some((a, b, var_a, var_b))
  }

  /// Squared, normalized residuals in x and y for each
  /// point, for parameters x0, y0, tx, ty
  fn residuals2<'a>(&'a self, p : &'a [f64]) -> impl Iterator<Item = (f64, f64)> + 'a {
    (0..self.xs.len()).map(move |k| {
      let dz = self.zs[k] - self.z0;
      let rx = self.xs[k] - (p[0] + p[2]*dz);
      let ry = self.ys[k] - (p[1] + p[3]*dz);
      (rx*rx*self.wx[k], ry*ry*self.wy[k])
    })
  }

  fn chi2(&self, p : &[f64]) -> f64 {
    self.residuals2(p).map(|(rx2, ry2)| rx2 + ry2).sum()
  }

  fn sum_of_distances(&self, p : &[f64]) -> f64 {
    self.residuals2(p).map(|(rx2, ry2)| (rx2 + ry2).sqrt()).sum()
  }
}

impl CostFunction for LineFitPoints {
  type Param  = Vec<f64>;
  type Output = f64;

  fn cost(&self, p : &Self::Param) -> Result<Self::Output, argmin::core::Error> {
    Ok(self.sum_of_distances(p))
  }
}

//------------------------------------------------------------

#[cfg(test)]
mod tests {
  use super::*;

  /// points on the line through (1,2,0) with slopes
  /// tx = 0.5, ty = -0.25
  fn line_points() -> (Vec<f32>, Vec<f32>, Vec<f32>) {
    let zs : Vec<f32> = vec![-100.0, -50.0, 0.0, 30.0, 80.0, 120.0];
    let xs = zs.iter().map(|z| 1.0 + 0.5*z).collect();
    let ys = zs.iter().map(|z| 2.0 - 0.25*z).collect();
    (xs, ys, zs)
  }

  #[test]
  fn linefit_exact_line() {
    let (xs, ys, zs) = line_points();
    let lf = LineFit::fit(&xs, &ys, &zs, None, None).unwrap();
    assert_eq!(lf.status, FitStatus::Success);
    assert_eq!(lf.ndof, 8);
    assert!((lf.tx - 0.5).abs()  < 1e-5);
    assert!((lf.ty + 0.25).abs() < 1e-5);
    let (x, y, _) = lf.eval(0.0);
    assert!((x - 1.0).abs() < 1e-4);
    assert!((y - 2.0).abs() < 1e-4);
    assert!(lf.chi2 < 1e-6);
  }

  #[test]
  fn linefit_not_enough_points() {
    assert!(LineFit::fit(&[1.0], &[1.0], &[1.0], None, None).is_none());
    // all points at the same z
    assert!(LineFit::fit(&[1.0, 2.0], &[1.0, 2.0], &[5.0, 5.0], None, None).is_none());
  }

  #[test]
  fn linefit_chi2_is_minimal() {
    let (mut xs, mut ys, zs) = line_points();
    xs[1] += 7.0;
    ys[3] -= 4.0;
    let errs = vec![3.0f32; xs.len()];
    let lf   = LineFit::fit(&xs, &ys, &zs, Some(&errs), Some(&errs)).unwrap();
    let pts  = LineFitPoints::new(&xs, &ys, &zs, Some(&errs), Some(&errs)).unwrap();
    let best = [lf.x0 as f64, lf.y0 as f64, lf.tx as f64, lf.ty as f64];
    let chi2 = pts.chi2(&best);
    for k in 0..4 {
      for delta in [-1e-2, 1e-2] {
        let mut p = best;
        p[k] += delta;
        assert!(pts.chi2(&p) > chi2);
      }
    }
  }

  #[test]
  fn linefit_sum_of_distances_outlier() {
    let (mut xs, ys, zs) = line_points();
    // one large outlier, which pulls the chi2 fit
    // considerably more than the sum of distances
    xs[2] += 200.0;
    let lf_chi2 = LineFit::fit(&xs, &ys, &zs, None, None).unwrap();
    let lf_dist = LineFit::fit_sum_of_distances(&xs, &ys, &zs, None, None).unwrap();
    assert_eq!(lf_dist.status, FitStatus::Success);
    assert!(lf_dist.cost <= LineFitPoints::new(&xs, &ys, &zs, None, None).unwrap()
                            .sum_of_distances(&[lf_chi2.x0 as f64, lf_chi2.y0 as f64,
                                                lf_chi2.tx as f64, lf_chi2.ty as f64]) as f32);
    assert!((lf_dist.tx - 0.5).abs() < (lf_chi2.tx - 0.5).abs() + 1e-6);
    assert!((lf_dist.eval(0.0).0 - 1.0).abs() < (lf_chi2.eval(0.0).0 - 1.0).abs());
  }
}
