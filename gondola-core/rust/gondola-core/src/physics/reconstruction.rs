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

/// TOF paddles with ids up to this one form the inner TOF
/// (cube), the ones above the outer TOF (umbrella, cortina)
pub const TOF_INNER_MAX_PADDLE_ID : u8 = 60;

/// The cost function a LineFit minimizes
#[derive(Debug, Copy, Clone, PartialEq)]
#[cfg_attr(feature = "pybindings", pyclass(eq, eq_int))]
pub enum LineFitCost {
  /// sum of the squared, normalized residuals (LineFit::fit)
  Chi2,
  /// sum of the normalized distances (LineFit::fit_sum_of_distances)
  SumOfDistances,
}

/// Settings for the hit cleaning of LineFit::fit_cleaned
///
/// A hit is accepted if its distance from the line is at
/// most acceptance_radius(s), with s the distance along the
/// line beyond the inner seed hit:
///
/// r(s) = cylinder_radius + max(s, 0) * tan(cone_angle)
///
/// so hits before the inner seed hit (e.g. the outer TOF)
/// have to be within the cylinder, and beyond it (e.g. the
/// tracker) the acceptance widens like a cone, to allow for
/// the uncertainty of the direction of the line.
#[derive(Debug, Copy, Clone, PartialEq)]
#[cfg_attr(feature="pybindings", pyclass)]
pub struct LineFitCleaning {
  /// Only hits within a cylinder of this radius (in mm)
  /// around the line are used in the fit
  pub cylinder_radius : f32,
  /// Half opening angle (in rad) of the cone, opening from
  /// the inner seed hit away from the outer seed hit, which
  /// is added to the cylinder. 0 means cylinder only
  pub cone_angle      : f32,
  /// Maximum number of fits. The iteration stops earlier
  /// once the set of accepted hits does not change anymore
  pub max_iterations  : u32,
  /// The cost function for each fit
  pub cost            : LineFitCost,
}

impl LineFitCleaning {
  pub fn new(cylinder_radius : f32) -> Self {
    Self {
      cylinder_radius,
      cone_angle     : 0.0,
      max_iterations : 10,
      cost           : LineFitCost::Chi2,
    }
  }

  /// The radius (in mm) within which hits are accepted at a
  /// distance s (in mm) along the line beyond the inner seed
  /// hit (negative before it)
  pub fn acceptance_radius(&self, s : f32) -> f32 {
    self.cylinder_radius + s.max(0.0)*self.cone_angle.tan()
  }
}

/// The result of LineFit::fit_cleaned
#[derive(Debug, Clone, PartialEq)]
#[cfg_attr(feature="pybindings", pyclass)]
pub struct CleanedLineFit {
  /// the fit to the accepted hits of the last iteration
  pub fit        : LineFit,
  /// for each hit, if it is used in the fit
  pub accepted   : Vec<bool>,
  /// the indices of the two hits the first line was
  /// constructed through (for fit_cleaned_event: first the
  /// outer, then the inner TOF hit)
  pub seed       : (usize, usize),
  /// number of fits which have been performed
  pub iterations : u32,
  /// true if the set of accepted hits did not change in
  /// the last iteration, false if max_iterations was reached
  pub converged  : bool,
}

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

  /// Fit a straight line, using only hits within a cylinder
  /// around the line (LineFitCleaning::cylinder_radius), which
  /// widens to a cone beyond the inner seed hit
  /// (LineFitCleaning::cone_angle).
  ///
  /// The hits have to be ordered in time, with the TOF hits
  /// first. Tracker hits do not have a time (NaN).
  ///
  /// 1) The first line is constructed through the first two
  ///    hits with a time. These are not necessarily one outer
  ///    and one inner TOF hit, fit_cleaned_event makes sure
  ///    of that, using the paddle ids.
  /// 2) All hits within the cylinder (and cone) around the
  ///    line are accepted, the others rejected. The tip of the
  ///    cone is the inner (second) seed hit, projected on the
  ///    line, and it opens away from the outer (first) one.
  /// 3) The line is fitted to the accepted hits.
  /// 4) Steps 2) and 3) are repeated with the fitted line
  ///    until the accepted hits do not change anymore or
  ///    LineFitCleaning::max_iterations is reached.
  ///
  /// Returns None if there are less than 2 hits with a time,
  /// the first two of them are at the same position, or less
  /// than 2 hits are accepted (or they can not be fit).
  ///
  /// # Arguments:
  ///   * ts             : hit times, NaN for tracker hits
  ///   * errs_x, errs_y : errors on x and y for each point.
  ///                      If None, LINEFIT_DEFAULT_ERR_XY
  ///                      will be used
  pub fn fit_cleaned(xs       : &[f32],
                     ys       : &[f32],
                     zs       : &[f32],
                     ts       : &[f32],
                     errs_x   : Option<&[f32]>,
                     errs_y   : Option<&[f32]>,
                     cleaning : &LineFitCleaning) -> Option<CleanedLineFit> {
    if ts.len() != xs.len() {
      error!("Got {} t values for {} points!", ts.len(), xs.len());
      return None;
    }
    let mut timed = (0..ts.len()).filter(|k| !ts[*k].is_nan());
    let (Some(i), Some(j)) = (timed.next(), timed.next()) else {
      error!("Need at least 2 hits with a time (TOF hits) to construct the first line!");
      return None;
    };
    Self::fit_cleaned_with_seed(xs, ys, zs, errs_x, errs_y, (i, j), cleaning)
  }

  /// Fit a straight line to the hits of an event, using only
  /// hits within a cylinder around the line, see fit_cleaned.
  ///
  /// The first line is constructed through the first outer
  /// and the first inner TOF hit (in time), the inner TOF
  /// being the paddles up to TOF_INNER_MAX_PADDLE_ID.
  ///
  /// The hits are the TOF hits (in the order of
  /// event.tof_event.hits), followed by the tracker hits (in
  /// the order of event.tracker_hits), so for the n TOF hits,
  /// accepted[k] is the TOF hit k for k < n and the tracker
  /// hit k - n otherwise. Tracker hits without coordinates
  /// are never accepted.
  ///
  /// Returns None if the TOF paddle information has not been
  /// set for the event (TofEvent::set_paddles), there is no
  /// outer or no inner TOF hit with a time, or the fit fails
  /// (see fit_cleaned).
  pub fn fit_cleaned_event(event : &TelemetryEvent, cleaning : &LineFitCleaning) -> Option<CleanedLineFit> {
    // without paddle information the TOF hits have no
    // coordinates, get_pointcloud reports that
    let tof   = event.tof_event.get_pointcloud()?;
    let first = |inner : bool| -> Option<usize> {
      event.tof_event.hits.iter().enumerate()
        .filter(|(_, h)| (h.paddle_id <= TOF_INNER_MAX_PADDLE_ID) == inner && !h.event_t0.is_nan())
        .min_by(|(_, a), (_, b)| a.event_t0.total_cmp(&b.event_t0))
        .map(|(k, _)| k)
    };
    let (Some(outer), Some(inner)) = (first(false), first(true)) else {
      error!("Need at least one outer and one inner TOF hit with a time to construct the first line!");
      return None;
    };
    let points : Vec<(f32, f32, f32, f32, f32)> = tof.into_iter()
      .chain(event.get_tracker_pointcloud())
      .collect();
    let usable : Vec<bool> = event.tof_event.hits.iter().map(|_| true)
      .chain(event.tracker_hits.iter().map(|h| h.has_coordinates))
      .collect();
    let xs : Vec<f32> = points.iter().map(|p| p.0).collect();
    let ys : Vec<f32> = points.iter().map(|p| p.1).collect();
    let zs : Vec<f32> = points.iter().map(|p| p.2).collect();
    Self::fit_cleaned_core(&xs, &ys, &zs, None, None, &usable, (outer, inner), cleaning)
  }

  /// Fit a straight line, using only hits within a cylinder
  /// around the line, see fit_cleaned. The first line is
  /// constructed through the two hits with the indices
  /// given by seed.
  pub fn fit_cleaned_with_seed(xs       : &[f32],
                               ys       : &[f32],
                               zs       : &[f32],
                               errs_x   : Option<&[f32]>,
                               errs_y   : Option<&[f32]>,
                               seed     : (usize, usize),
                               cleaning : &LineFitCleaning) -> Option<CleanedLineFit> {
    Self::fit_cleaned_core(xs, ys, zs, errs_x, errs_y, &vec![true; xs.len()], seed, cleaning)
  }

  /// The iteration of fit_cleaned. Hits which are not usable
  /// are never accepted.
  fn fit_cleaned_core(xs       : &[f32],
                      ys       : &[f32],
                      zs       : &[f32],
                      errs_x   : Option<&[f32]>,
                      errs_y   : Option<&[f32]>,
                      usable   : &[bool],
                      seed     : (usize, usize),
                      cleaning : &LineFitCleaning) -> Option<CleanedLineFit> {
    let n = xs.len();
    if ys.len() != n || zs.len() != n || usable.len() != n {
      error!("Got {} x, {} y and {} z values! They need to be of the same length!", n, ys.len(), zs.len());
      return None;
    }
    for errs in [errs_x, errs_y].into_iter().flatten() {
      if errs.len() != n {
        error!("Got {} errors for {} points!", errs.len(), n);
        return None;
      }
    }
    let (i, j) = seed;
    if i >= n || j >= n {
      error!("Seed hits {:?} out of range for {} points!", seed, n);
      return None;
    }
    let anchor = (xs[i], ys[i], zs[i]);
    let dir    = (xs[j] - xs[i], ys[j] - ys[i], zs[j] - zs[i]);
    let norm   = (dir.0*dir.0 + dir.1*dir.1 + dir.2*dir.2).sqrt();
    if norm == 0.0 {
      error!("The two seed hits are at the same position, can not construct a line!");
      return None;
    }
    let inner        = (xs[j], ys[j], zs[j]);
    let mut line     = (anchor, (dir.0/norm, dir.1/norm, dir.2/norm));
    let mut accepted : Vec<bool> = Vec::new();
    let mut fit      = None;
    let mut converged = false;
    let mut iterations = 0;
    while iterations < cleaning.max_iterations {
      // the cone opens away from the outer seed hit,
      // independent of the orientation of the fitted line
      let d = line.1;
      let d = if d.0*dir.0 + d.1*dir.1 + d.2*dir.2 < 0.0 {(-d.0, -d.1, -d.2)} else {d};
      let along = |p : (f32, f32, f32)| -> f32 {
        (p.0 - line.0.0)*d.0 + (p.1 - line.0.1)*d.1 + (p.2 - line.0.2)*d.2
      };
      let s_inner = along(inner);
      let now : Vec<bool> = (0..n)
        .map(|k| {
          let p = (xs[k], ys[k], zs[k]);
          usable[k] && distance_to_line(p, line.0, d) <= cleaning.acceptance_radius(along(p) - s_inner)
        })
        .collect();
      if fit.is_some() && now == accepted {
        converged = true;
        break;
      }
      accepted = now;
      let select = |vs : &[f32]| -> Vec<f32> {
        vs.iter().zip(&accepted).filter(|(_, a)| **a).map(|(v, _)| *v).collect()
      };
      let (ex, ey) = (errs_x.map(select), errs_y.map(select));
      let (sx, sy, sz) = (select(xs), select(ys), select(zs));
      let lf = match cleaning.cost {
        LineFitCost::Chi2           => Self::fit(&sx, &sy, &sz, ex.as_deref(), ey.as_deref()),
        LineFitCost::SumOfDistances => Self::fit_sum_of_distances(&sx, &sy, &sz, ex.as_deref(), ey.as_deref()),
      }?;
      iterations += 1;
      line = ((lf.x0, lf.y0, lf.z0), lf.direction());
      fit  = Some(lf);
    }
    Some(CleanedLineFit {
      fit  : fit?,
      accepted,
      seed,
      iterations,
      converged,
    })
  }

  /// Distance of a point from the line (in the units of
  /// the coordinates)
  pub fn distance(&self, x : f32, y : f32, z : f32) -> f32 {
    distance_to_line((x, y, z), (self.x0, self.y0, self.z0), self.direction())
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

#[cfg(feature="pybindings")]
#[pymethods]
impl LineFit {
  /// Analytic, weighted least squares fit of a straight line
  /// to the given points. Returns None if there are less than
  /// 2 points, or all points have the same z.
  ///
  /// # Arguments:
  ///   * errs_x, errs_y : errors on x and y for each point.
  ///                      If None, LINEFIT_DEFAULT_ERR_XY
  ///                      will be used
  #[staticmethod]
  #[pyo3(name="fit", signature = (xs, ys, zs, errs_x = None, errs_y = None))]
  fn fit_py(xs     : Vec<f32>,
            ys     : Vec<f32>,
            zs     : Vec<f32>,
            errs_x : Option<Vec<f32>>,
            errs_y : Option<Vec<f32>>) -> Option<Self> {
    Self::fit(&xs, &ys, &zs, errs_x.as_deref(), errs_y.as_deref())
  }

  /// Fit a straight line by minimizing the sum of the normalized
  /// distances of the points from the line (as the python
  /// version line_fit), which is less sensitive to outliers.
  /// Returns None under the same conditions as fit.
  #[staticmethod]
  #[pyo3(name="fit_sum_of_distances", signature = (xs, ys, zs, errs_x = None, errs_y = None))]
  fn fit_sum_of_distances_py(xs     : Vec<f32>,
                             ys     : Vec<f32>,
                             zs     : Vec<f32>,
                             errs_x : Option<Vec<f32>>,
                             errs_y : Option<Vec<f32>>) -> Option<Self> {
    Self::fit_sum_of_distances(&xs, &ys, &zs, errs_x.as_deref(), errs_y.as_deref())
  }

  /// Fit a straight line, using only hits within a cylinder
  /// around the line (LineFitCleaning). The first line is
  /// constructed through the first two hits with a time
  /// (ts not NaN), which are not necessarily one outer and
  /// one inner TOF hit, see fit_cleaned_event for that.
  ///
  /// Returns a CleanedLineFit, or None if there are less than
  /// 2 hits with a time, or less than 2 hits are accepted.
  ///
  /// # Arguments:
  ///   * ts             : hit times, NaN for tracker hits
  ///   * cleaning       : the cylinder radius etc.
  ///   * errs_x, errs_y : errors on x and y for each point.
  ///                      If None, LINEFIT_DEFAULT_ERR_XY
  ///                      will be used
  #[staticmethod]
  #[pyo3(name="fit_cleaned", signature = (xs, ys, zs, ts, cleaning, errs_x = None, errs_y = None))]
  fn fit_cleaned_py(xs       : Vec<f32>,
                    ys       : Vec<f32>,
                    zs       : Vec<f32>,
                    ts       : Vec<f32>,
                    cleaning : &LineFitCleaning,
                    errs_x   : Option<Vec<f32>>,
                    errs_y   : Option<Vec<f32>>) -> Option<CleanedLineFit> {
    Self::fit_cleaned(&xs, &ys, &zs, &ts, errs_x.as_deref(), errs_y.as_deref(), cleaning)
  }

  /// Fit a straight line to the TOF and tracker hits of an
  /// event, using only hits within a cylinder around the line.
  /// The first line is constructed through the first outer
  /// and the first inner TOF hit (in time).
  ///
  /// accepted of the result has the TOF hits first (in the
  /// order of event.tof.hits), followed by the tracker hits
  /// (in the order of event.tracker). Tracker hits without
  /// coordinates are never accepted.
  ///
  /// Returns None if the TOF paddle information has not been
  /// set for the event (event.tof_set_paddles or hydrate),
  /// there is no outer or no inner TOF hit with a time, or
  /// the fit fails. The TOF hits only have a time (event_t0)
  /// after event.tof_normalize_hit_times (with or without
  /// timing offsets).
  #[staticmethod]
  #[pyo3(name="fit_cleaned_event")]
  fn fit_cleaned_event_py(event : &TelemetryEvent, cleaning : &LineFitCleaning) -> Option<CleanedLineFit> {
    Self::fit_cleaned_event(event, cleaning)
  }

  /// Fit a straight line, using only hits within a cylinder
  /// around the line, see fit_cleaned. The first line is
  /// constructed through the two hits with the indices
  /// given by seed.
  #[staticmethod]
  #[pyo3(name="fit_cleaned_with_seed", signature = (xs, ys, zs, seed, cleaning, errs_x = None, errs_y = None))]
  fn fit_cleaned_with_seed_py(xs       : Vec<f32>,
                              ys       : Vec<f32>,
                              zs       : Vec<f32>,
                              seed     : (usize, usize),
                              cleaning : &LineFitCleaning,
                              errs_x   : Option<Vec<f32>>,
                              errs_y   : Option<Vec<f32>>) -> Option<CleanedLineFit> {
    Self::fit_cleaned_with_seed(&xs, &ys, &zs, errs_x.as_deref(), errs_y.as_deref(), seed, cleaning)
  }

  /// Distance of a point from the line (in the units of
  /// the coordinates)
  #[pyo3(name="distance")]
  fn distance_py(&self, x : f32, y : f32, z : f32) -> f32 {
    self.distance(x, y, z)
  }

  /// Evaluate the line at a given z, returns (x, y, z)
  #[pyo3(name="eval")]
  fn eval_py(&self, z : f32) -> (f32, f32, f32) {
    self.eval(z)
  }

  /// Unit direction vector of the line, pointing
  /// towards +z
  #[getter]
  #[pyo3(name="direction")]
  fn get_direction_py(&self) -> (f32, f32, f32) {
    self.direction()
  }

  /// Cosine of the zenith angle of the line
  #[getter]
  #[pyo3(name="cos_theta")]
  fn get_cos_theta_py(&self) -> f32 {
    self.cos_theta()
  }

  /// The parameters of the line in the form as expected
  /// by line3d (x_a, y_a, z_a, dx, dy, dz)
  #[getter]
  #[pyo3(name="line3d_params")]
  fn get_line3d_params_py(&self) -> (f32, f32, f32, f32, f32, f32) {
    self.line3d_params()
  }

  /// Anchor point, x
  #[getter]
  fn get_x0(&self) -> f32 {
    self.x0
  }

  /// Anchor point, y
  #[getter]
  fn get_y0(&self) -> f32 {
    self.y0
  }

  /// Anchor point, z (fixed to the mean z of the hits)
  #[getter]
  fn get_z0(&self) -> f32 {
    self.z0
  }

  /// Slope dx/dz
  #[getter]
  fn get_tx(&self) -> f32 {
    self.tx
  }

  /// Slope dy/dz
  #[getter]
  fn get_ty(&self) -> f32 {
    self.ty
  }

  /// Error on x0 (from the analytic chi2 fit)
  #[getter]
  fn get_x0_err(&self) -> f32 {
    self.x0_err
  }

  /// Error on y0 (from the analytic chi2 fit)
  #[getter]
  fn get_y0_err(&self) -> f32 {
    self.y0_err
  }

  /// Error on tx (from the analytic chi2 fit)
  #[getter]
  fn get_tx_err(&self) -> f32 {
    self.tx_err
  }

  /// Error on ty (from the analytic chi2 fit)
  #[getter]
  fn get_ty_err(&self) -> f32 {
    self.ty_err
  }

  /// Sum of the squared, normalized residuals
  #[getter]
  fn get_chi2(&self) -> f32 {
    self.chi2
  }

  /// Value of the minimized cost function. For the
  /// analytic fit this is the same as chi2
  #[getter]
  fn get_cost(&self) -> f32 {
    self.cost
  }

  /// 2*npoints - 4
  #[getter]
  fn get_ndof(&self) -> u32 {
    self.ndof
  }

  /// Number of points used in the fit
  #[getter]
  fn get_npoints(&self) -> u32 {
    self.npoints
  }

  #[getter]
  fn get_status(&self) -> FitStatus {
    self.status
  }

  fn __repr__(&self) -> String {
    self.to_string()
  }
}

#[cfg(feature="pybindings")]
#[pymethods]
impl LineFitCleaning {
  #[new]
  #[pyo3(signature = (cylinder_radius, cone_angle = 0.0, max_iterations = 10, cost = LineFitCost::Chi2))]
  fn new_py(cylinder_radius : f32, cone_angle : f32, max_iterations : u32, cost : LineFitCost) -> Self {
    Self {
      cylinder_radius,
      cone_angle,
      max_iterations,
      cost,
    }
  }

  /// The radius (in mm) within which hits are accepted at a
  /// distance s (in mm) along the line beyond the inner seed
  /// hit (negative before it)
  #[pyo3(name="acceptance_radius")]
  fn acceptance_radius_py(&self, s : f32) -> f32 {
    self.acceptance_radius(s)
  }

  /// Only hits within a cylinder of this radius (in mm)
  /// around the line are used in the fit
  #[getter]
  fn get_cylinder_radius(&self) -> f32 {
    self.cylinder_radius
  }

  #[setter]
  fn set_cylinder_radius(&mut self, cylinder_radius : f32) {
    self.cylinder_radius = cylinder_radius;
  }

  /// Half opening angle (in rad) of the cone, opening from
  /// the inner seed hit away from the outer seed hit, which
  /// is added to the cylinder. 0 means cylinder only
  #[getter]
  fn get_cone_angle(&self) -> f32 {
    self.cone_angle
  }

  #[setter]
  fn set_cone_angle(&mut self, cone_angle : f32) {
    self.cone_angle = cone_angle;
  }

  /// Maximum number of fits. The iteration stops earlier
  /// once the set of accepted hits does not change anymore
  #[getter]
  fn get_max_iterations(&self) -> u32 {
    self.max_iterations
  }

  #[setter]
  fn set_max_iterations(&mut self, max_iterations : u32) {
    self.max_iterations = max_iterations;
  }

  /// The cost function for each fit
  #[getter]
  fn get_cost(&self) -> LineFitCost {
    self.cost
  }

  #[setter]
  fn set_cost(&mut self, cost : LineFitCost) {
    self.cost = cost;
  }

  fn __repr__(&self) -> String {
    format!("<LineFitCleaning: cylinder_radius {} mm, cone_angle {} rad, max_iterations {}, cost {:?}>",
            self.cylinder_radius, self.cone_angle, self.max_iterations, self.cost)
  }
}

#[cfg(feature="pybindings")]
#[pymethods]
impl CleanedLineFit {
  /// The fit to the accepted hits of the last iteration
  #[getter]
  fn get_fit(&self) -> LineFit {
    self.fit
  }

  /// For each hit, if it is used in the fit
  #[getter]
  fn get_accepted(&self) -> Vec<bool> {
    self.accepted.clone()
  }

  /// The indices of the two hits the first line was
  /// constructed through (for fit_cleaned_event: first the
  /// outer, then the inner TOF hit)
  #[getter]
  fn get_seed(&self) -> (usize, usize) {
    self.seed
  }

  /// Number of fits which have been performed
  #[getter]
  fn get_iterations(&self) -> u32 {
    self.iterations
  }

  /// True if the set of accepted hits did not change in
  /// the last iteration, False if max_iterations was reached
  #[getter]
  fn get_converged(&self) -> bool {
    self.converged
  }

  fn __repr__(&self) -> String {
    format!("<CleanedLineFit: {} of {} hits accepted, seed {:?}, {} iterations, converged {}\n{}>",
            self.accepted.iter().filter(|a| **a).count(), self.accepted.len(),
            self.seed, self.iterations, self.converged, self.fit)
  }
}

/// Distance of point p from the line through anchor a with
/// unit direction d
fn distance_to_line(p : (f32, f32, f32), a : (f32, f32, f32), d : (f32, f32, f32)) -> f32 {
  let v = (p.0 - a.0, p.1 - a.1, p.2 - a.2);
  let c = (v.1*d.2 - v.2*d.1, v.2*d.0 - v.0*d.2, v.0*d.1 - v.1*d.0);
  (c.0*c.0 + c.1*c.1 + c.2*c.2).sqrt()
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

  /// A vertical track along the z axis: two TOF hits, which
  /// are 30 mm off (TOF position resolution), followed by
  /// tracker hits on the track and one far away
  fn track_with_outlier() -> (Vec<f32>, Vec<f32>, Vec<f32>, Vec<f32>) {
    let mut xs = vec![30.0, -30.0];
    let mut zs = vec![1000.0, 0.0];
    let mut ts = vec![0.0, 3.5];
    for k in 1..10 {
      xs.push(0.0);
      zs.push(100.0*k as f32);
      ts.push(f32::NAN);
    }
    // outlier
    xs.push(100.0);
    zs.push(500.0);
    ts.push(f32::NAN);
    let ys = vec![0.0; xs.len()];
    (xs, ys, zs, ts)
  }

  #[test]
  fn linefit_distance() {
    let (xs, ys, zs) = line_points();
    let lf = LineFit::fit(&xs, &ys, &zs, None, None).unwrap();
    assert!(lf.distance(xs[2], ys[2], zs[2]) < 1e-4);
    // (1, 2, 0) is perpendicular to the direction (0.5, -0.25, 1)
    // of the line, which goes through (1, 2, 0)
    assert!((lf.distance(2.0, 4.0, 0.0) - 5.0f32.sqrt()).abs() < 1e-4);
    // a point on the z axis of a vertical line
    let lf = LineFit::fit(&[0.0, 0.0], &[0.0, 0.0], &[0.0, 1.0], None, None).unwrap();
    assert!((lf.distance(3.0, 4.0, 17.0) - 5.0).abs() < 1e-5);
  }

  #[test]
  fn linefit_cleaned_rejects_outlier() {
    let (xs, ys, zs, ts) = track_with_outlier();
    let cleaned = LineFit::fit_cleaned(&xs, &ys, &zs, &ts, None, None, &LineFitCleaning::new(20.0)).unwrap();
    assert_eq!(cleaned.seed, (0, 1));
    assert!(cleaned.converged);
    // the tracker hits at z = 100 and 900 are 24 mm from the
    // first line through the TOF hits and only get accepted
    // after the first fit
    assert!(cleaned.iterations >= 2);
    let mut expected = vec![true; xs.len()];
    *expected.last_mut().unwrap() = false;
    assert_eq!(cleaned.accepted, expected);
    assert_eq!(cleaned.fit.npoints, 11);
    // the track is vertical, only the TOF hits pull the
    // line away from it
    assert!(cleaned.fit.tx.abs() < 0.03);
    assert!(cleaned.fit.ty.abs() < 1e-6);
  }

  #[test]
  fn linefit_cleaned_large_radius_is_plain_fit() {
    let (xs, ys, zs, ts) = track_with_outlier();
    let cleaned = LineFit::fit_cleaned(&xs, &ys, &zs, &ts, None, None, &LineFitCleaning::new(1e6)).unwrap();
    let plain   = LineFit::fit(&xs, &ys, &zs, None, None).unwrap();
    assert!(cleaned.accepted.iter().all(|a| *a));
    assert_eq!(cleaned.iterations, 1);
    assert_eq!(cleaned.fit, plain);
  }

  #[test]
  fn linefit_cleaned_needs_two_tof_hits() {
    let (xs, ys, zs, mut ts) = track_with_outlier();
    ts[1] = f32::NAN;
    assert!(LineFit::fit_cleaned(&xs, &ys, &zs, &ts, None, None, &LineFitCleaning::new(20.0)).is_none());
    // the seed are the first two hits with a time
    ts[3] = 1.0;
    let cleaned = LineFit::fit_cleaned(&xs, &ys, &zs, &ts, None, None, &LineFitCleaning::new(20.0)).unwrap();
    assert_eq!(cleaned.seed, (0, 3));
  }

  #[test]
  fn linefit_cleaning_acceptance_radius() {
    let mut cleaning = LineFitCleaning::new(50.0);
    assert_eq!(cleaning.acceptance_radius(-100.0), 50.0);
    assert_eq!(cleaning.acceptance_radius(500.0), 50.0);
    cleaning.cone_angle = std::f32::consts::FRAC_PI_4;
    assert_eq!(cleaning.acceptance_radius(-100.0), 50.0);
    assert!((cleaning.acceptance_radius(100.0) - 150.0).abs() < 1e-3);
  }

  /// A vertical track (x = y = 0) through an outer TOF hit at
  /// z = 2000, an inner one at z = 1200 and 7 tracker layers,
  /// plus two hits 80 mm off the track: one before the inner
  /// TOF hit (z = 1600) and one 700 mm beyond it (z = 500)
  fn track_for_cone() -> (Vec<f32>, Vec<f32>, Vec<f32>, Vec<f32>) {
    let mut zs : Vec<f32> = vec![2000.0, 1200.0];
    zs.extend((0..7).map(|k| 400.0 + 100.0*k as f32));
    let mut xs = vec![0.0; zs.len()];
    let mut ts = vec![f32::NAN; zs.len()];
    ts[0] = 0.0;
    ts[1] = 3.0;
    xs.extend([80.0, 80.0]);
    zs.extend([1600.0, 500.0]);
    ts.extend([f32::NAN, f32::NAN]);
    let ys = vec![0.0; zs.len()];
    (xs, ys, zs, ts)
  }

  #[test]
  fn linefit_cleaned_cone() {
    let (xs, ys, zs, ts) = track_for_cone();
    let n = xs.len();
    // cylinder only, both hits off the track are rejected
    let mut cleaning = LineFitCleaning::new(50.0);
    let cleaned = LineFit::fit_cleaned(&xs, &ys, &zs, &ts, None, None, &cleaning).unwrap();
    assert!(cleaned.accepted[..n-2].iter().all(|a| *a));
    assert!(!cleaned.accepted[n-2] && !cleaned.accepted[n-1]);
    // a 5 deg cone gives r = 50 + 700*tan(5 deg) = 111 mm at
    // the hit beyond the inner TOF hit, but does not change
    // the acceptance before it
    cleaning.cone_angle = 5.0f32.to_radians();
    let cleaned = LineFit::fit_cleaned(&xs, &ys, &zs, &ts, None, None, &cleaning).unwrap();
    assert!(cleaned.accepted[..n-2].iter().all(|a| *a));
    assert!(!cleaned.accepted[n-2]);
    assert!(cleaned.accepted[n-1]);
    assert!(cleaned.converged);
  }

  #[test]
  fn linefit_cleaned_cone_opens_away_from_outer_hit() {
    // the same track mirrored in z (going upwards, outer TOF
    // hit at z = 400, inner at z = 1200), the cone has to
    // open towards +z now
    let (xs, ys, mut zs, ts) = track_for_cone();
    let n = xs.len();
    for z in zs.iter_mut() {
      *z = 2400.0 - *z;
    }
    let mut cleaning = LineFitCleaning::new(50.0);
    cleaning.cone_angle = 5.0f32.to_radians();
    let cleaned = LineFit::fit_cleaned(&xs, &ys, &zs, &ts, None, None, &cleaning).unwrap();
    // z = 800 is before the inner hit (with the cone opening
    // towards -z, r would be 50 + 400*tan(5 deg) = 85 mm)
    assert!(!cleaned.accepted[n-2]);
    // z = 1900 is 700 mm beyond the inner hit
    assert!(cleaned.accepted[n-1]);
  }

  fn tof_hit(paddle_id : u8, x : f32, z : f32, t : f32) -> TofHit {
    let mut h = TofHit::new();
    h.paddle_id = paddle_id;
    h.x         = x;
    h.y         = 0.0;
    h.z         = z;
    h.event_t0  = t;
    h
  }

  /// The vertical track of track_with_outlier as an event.
  /// The first two TOF hits in time are both outer: one on
  /// the track (umbrella) and one far off (cortina).
  fn event_with_two_outer_hits() -> TelemetryEvent {
    let mut ev = TelemetryEvent::new();
    ev.tof_event.hits = vec![tof_hit(70, 30.0, 1000.0, 0.0),
                             tof_hit(120, 500.0, 800.0, 1.0),
                             tof_hit(10, -30.0, 0.0, 3.5)];
    ev.tof_event.paddles_set = true;
    for k in 1..10 {
      let mut h = TrackerHit::new();
      // tracker coordinates are in cm
      h.z = 10.0*k as f32;
      h.has_coordinates = true;
      ev.tracker_hits.push(h);
    }
    // a tracker hit without coordinates (at the origin, which
    // is close to the track)
    ev.tracker_hits.push(TrackerHit::new());
    ev
  }

  #[test]
  fn linefit_cleaned_event_seeds_outer_and_inner() {
    let ev = event_with_two_outer_hits();
    let cleaned = LineFit::fit_cleaned_event(&ev, &LineFitCleaning::new(20.0)).unwrap();
    // the umbrella and the cube hit, not the two first in time
    assert_eq!(cleaned.seed, (0, 2));
    assert!(cleaned.converged);
    let mut expected = vec![true; 3 + 10];
    expected[1]  = false;
    expected[12] = false;
    assert_eq!(cleaned.accepted, expected);
    assert!(cleaned.fit.tx.abs() < 0.03);
    // seeding with the first two hits in time goes wrong:
    // the line through the umbrella and the cortina hit
    // misses all tracker hits
    let tof = ev.tof_event.get_pointcloud().unwrap();
    let pts : Vec<_> = tof.into_iter().chain(ev.get_tracker_pointcloud()).collect();
    let col = |f : fn(&(f32, f32, f32, f32, f32)) -> f32| -> Vec<f32> { pts.iter().map(f).collect() };
    let (xs, ys, zs, ts) = (col(|p| p.0), col(|p| p.1), col(|p| p.2), col(|p| p.3));
    let by_time = LineFit::fit_cleaned(&xs, &ys, &zs, &ts, None, None, &LineFitCleaning::new(20.0));
    let by_time = by_time.unwrap();
    assert_eq!(by_time.seed, (0, 1));
    assert_eq!(by_time.accepted.iter().filter(|a| **a).count(), 2);
  }

  #[test]
  fn linefit_cleaned_event_needs_outer_and_inner() {
    let mut ev = event_with_two_outer_hits();
    ev.tof_event.paddles_set = false;
    assert!(LineFit::fit_cleaned_event(&ev, &LineFitCleaning::new(20.0)).is_none());
    ev.tof_event.paddles_set = true;
    // no inner hit
    ev.tof_event.hits.pop();
    assert!(LineFit::fit_cleaned_event(&ev, &LineFitCleaning::new(20.0)).is_none());
  }
}
