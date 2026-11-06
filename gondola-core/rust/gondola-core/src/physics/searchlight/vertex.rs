//! The vertex (stopping point of the primary) for searchlight:
//! lines with an error growing with the distance from their
//! closest hit, and an adaptive fit to these lines and points
// This file is part of gaps-online-software and published
// under the GPLv3 license

use std::collections::HashSet;

use glam::{DMat3, DVec3};

use crate::prelude::*;

use super::{fit_line, SearchlightHits, C_LIGHT_MM_NS};

/// A line for the vertex fit, with an error which grows with
/// the distance from its closest hit:
///
/// sigma(s)^2 = sigma0^2 + (theta*s)^2
#[derive(Debug, Clone)]
pub struct SearchlightLine {
  /// a point on the line (mm)
  pub a      : DVec3,
  /// unit vector along the line
  pub u      : DVec3,
  /// the hits of the line (indices into SearchlightHits)
  pub hits   : Vec<usize>,
  /// error at the hits (mm)
  pub sigma0 : f64,
  /// growth of the error with the distance (rad)
  pub theta  : f64,
}

impl SearchlightLine {
  /// Distance of the point v from the line
  pub fn dist(&self, v : DVec3) -> f64 {
    let w = v - self.a;
    (w - w.dot(self.u)*self.u).length()
  }

  /// Error of the line at the point v
  pub fn sigma(&self, v : DVec3, hits : &SearchlightHits) -> f64 {
    let s = if self.hits.is_empty() {
      0.0
    } else {
      self.hits.iter().map(|&k| (hits.pos[k] - v).length()).fold(f64::INFINITY, f64::min)
    };
    (self.sigma0*self.sigma0 + (self.theta*s).powi(2)).sqrt()
  }
}

/// Settings for the vertex of searchlight
#[derive(Debug, Clone)]
pub struct SearchlightVertexSettings {
  /// secondary pairs (inner -> outer TOF hit at beta ~ 1):
  /// time resolution per hit (ns), position error of a TOF
  /// hit (mm), the window in units of the uncertainty and the
  /// smallest beta of a secondary
  pub pair_sigma_t     : f64,
  pub pair_sigma_pos   : f64,
  pub pair_n_sigma     : f64,
  pub pair_beta_min    : f64,
  /// tracker-only lines: through pairs of tracker hits at least
  /// trk_min_dz (mm) apart in z, collecting the hits within
  /// trk_radius (mm), at least trk_min_hits hits
  pub trk_lines        : bool,
  pub trk_radius       : f64,
  pub trk_min_hits     : usize,
  pub trk_min_dz       : f64,
  /// line errors (mm, rad): secondaries with tracker hits, with
  /// TOF hits only, and the primary
  pub sigma0_sec       : f64,
  pub sigma0_tof_only  : f64,
  pub theta_sec        : f64,
  pub sigma0_prim      : f64,
  pub theta_prim       : f64,
  /// the primary line takes part if both seed hits have at least
  /// this edep (MeV) and the outer one is earlier
  pub use_primary      : bool,
  pub seed_edep_min    : f64,
  /// the tracker hit with the largest energy as a point, if above
  /// e_point_min (MeV), with the error e_point_err (mm)
  pub use_e_point      : bool,
  pub e_point_min      : f64,
  pub e_point_err      : f64,
  /// adaptive fit: chi2 of the transition from inlier to outlier,
  /// and the temperatures of the annealing
  pub chi2_cut         : f64,
  pub temperatures     : Vec<f64>,
  /// the vertex has to be inside this box (mm)
  pub box_min          : DVec3,
  pub box_max          : DVec3,
}

impl Default for SearchlightVertexSettings {
  fn default() -> Self {
    Self {
      pair_sigma_t     : 0.17,
      pair_sigma_pos   : 50.0,
      pair_n_sigma     : 3.0,
      pair_beta_min    : 0.7,
      trk_lines        : true,
      trk_radius       : 30.0,
      trk_min_hits     : 3,
      trk_min_dz       : 50.0,
      sigma0_sec       : 20.0,
      sigma0_tof_only  : 50.0,
      theta_sec        : 0.05,
      sigma0_prim      : 20.0,
      theta_prim       : 0.05,
      use_primary      : true,
      seed_edep_min    : 3.0,
      use_e_point      : true,
      e_point_min      : 5.0,
      e_point_err      : 30.0,
      chi2_cut         : 9.0,
      temperatures     : vec![64.0, 16.0, 4.0, 1.0],
      box_min          : DVec3::new(-800.0, -800.0, 150.0),
      box_max          : DVec3::new(800.0, 800.0, 1200.0),
    }
  }
}

impl SearchlightVertexSettings {
  pub(crate) fn inside(&self, p : DVec3) -> bool {
    p.cmpge(self.box_min).all() && p.cmple(self.box_max).all()
  }
}

/// The points of closest approach of two lines (unit vectors
/// u1, u2), None for parallel lines
pub(crate) fn closest_points(a1 : DVec3, u1 : DVec3, a2 : DVec3, u2 : DVec3) -> Option<(DVec3, DVec3)> {
  let w = a1 - a2;
  let b = u1.dot(u2);
  let d = 1.0 - b*b;
  if d < 1e-9 {
    return None;
  }
  let s = (b*u2.dot(w) - u1.dot(w))/d;
  let t = (u2.dot(w) - b*u1.dot(w))/d;
  Some((a1 + s*u1, a2 + t*u2))
}

/// Pairs (inner, outer) of TOF hits which can be one particle
/// leaving the detector: the outer hit is later, at a velocity
/// close to the speed of light. Each hit is used at most once,
/// the best matching pairs first.
pub(crate) fn secondary_pairs(hits : &SearchlightHits, s : &SearchlightVertexSettings, exclude : &[usize]) -> Vec<(usize, usize)> {
  let sigma = (2.0*s.pair_sigma_t.powi(2) + 2.0*(s.pair_sigma_pos/C_LIGHT_MM_NS).powi(2)).sqrt();
  let ok = |k : usize| hits.t[k].is_finite() && !exclude.contains(&k);
  let mut cand = Vec::<(f64, usize, usize)>::new();
  for i in (0..hits.ntof).filter(|&k| hits.inner[k] && ok(k)) {
    for o in (0..hits.ntof).filter(|&k| !hits.inner[k] && ok(k)) {
      let dt = hits.t[o] - hits.t[i];
      let l  = (hits.pos[o] - hits.pos[i]).length();
      // dt - l/c is 0 for beta = 1 and l/c (1/beta - 1) for slower ones
      let r  = dt - l/C_LIGHT_MM_NS;
      let hi = s.pair_n_sigma*sigma + l/C_LIGHT_MM_NS*(1.0/s.pair_beta_min - 1.0);
      if -s.pair_n_sigma*sigma < r && r < hi {
        cand.push((r.abs()/sigma, i, o));
      }
    }
  }
  cand.sort_by(|x, y| x.0.total_cmp(&y.0).then(x.1.cmp(&y.1)).then(x.2.cmp(&y.2)));
  let mut used  = HashSet::<usize>::new();
  let mut pairs = Vec::<(usize, usize)>::new();
  for (_, i, o) in cand {
    if used.contains(&i) || used.contains(&o) {
      continue;
    }
    pairs.push((i, o));
    used.insert(i);
    used.insert(o);
  }
  pairs
}

/// Centroid and principal axis of points
fn pca_line(points : &[DVec3]) -> (DVec3, DVec3) {
  let c = points.iter().fold(DVec3::ZERO, |s, &p| s + p)/(points.len() as f64);
  let mut m = DMat3::ZERO;
  for &p in points {
    let d = p - c;
    m += DMat3::from_cols(d*d.x, d*d.y, d*d.z);
  }
  (c, principal_eigenvector(m))
}

/// The eigenvector of the largest eigenvalue of a symmetric 3x3
/// matrix (cyclic Jacobi rotations)
fn principal_eigenvector(m : DMat3) -> DVec3 {
  let mut a = m.to_cols_array_2d();
  let mut v = [[1.0, 0.0, 0.0], [0.0, 1.0, 0.0], [0.0, 0.0, 1.0]];
  for _ in 0..50 {
    let off = a[0][1].abs() + a[0][2].abs() + a[1][2].abs();
    if off < 1e-12*(a[0][0].abs() + a[1][1].abs() + a[2][2].abs() + 1e-300) {
      break;
    }
    for (p, q) in [(0usize, 1usize), (0, 2), (1, 2)] {
      if a[p][q].abs() < 1e-300 {
        continue;
      }
      let theta = 0.5*(a[q][q] - a[p][p])/a[p][q];
      let t = theta.signum()/(theta.abs() + (theta*theta + 1.0).sqrt());
      let t = if theta == 0.0 { 1.0 } else { t };
      let c = 1.0/(t*t + 1.0).sqrt();
      let s = t*c;
      for k in 0..3 {
        let (akp, akq) = (a[k][p], a[k][q]);
        a[k][p] = c*akp - s*akq;
        a[k][q] = s*akp + c*akq;
      }
      for k in 0..3 {
        let (apk, aqk) = (a[p][k], a[q][k]);
        a[p][k] = c*apk - s*aqk;
        a[q][k] = s*apk + c*aqk;
      }
      for k in 0..3 {
        let (vkp, vkq) = (v[k][p], v[k][q]);
        v[k][p] = c*vkp - s*vkq;
        v[k][q] = s*vkp + c*vkq;
      }
    }
  }
  let j = (0..3).max_by(|&x, &y| a[x][x].total_cmp(&a[y][y])).unwrap();
  DVec3::new(v[0][j], v[1][j], v[2][j]).normalize()
}

/// Secondaries seen only in the tracker: lines through pairs of
/// free tracker hits in different layers take all free hits
/// within trk_radius; the lines with the most hits are taken
/// greedily, each hit used once
pub(crate) fn tracker_lines(hits : &SearchlightHits, free : &[usize], s : &SearchlightVertexSettings) -> Vec<SearchlightLine> {
  let trk : Vec<usize> = free.iter().copied().filter(|&k| !hits.is_tof[k]).collect();
  let mut cand = Vec::<(usize, f64, Vec<usize>)>::new();
  for x in 0..trk.len() {
    for y in x+1..trk.len() {
      let (j, k) = (trk[x], trk[y]);
      let d = hits.pos[k] - hits.pos[j];
      if d.z.abs() < s.trk_min_dz {
        continue;
      }
      let seed = SearchlightLine { a : hits.pos[j], u : d.normalize(), hits : vec![], sigma0 : 0.0, theta : 0.0 };
      let near : Vec<usize> = free.iter().copied().filter(|&m| seed.dist(hits.pos[m]) < s.trk_radius).collect();
      if near.len() >= s.trk_min_hits {
        let pts : Vec<DVec3> = near.iter().map(|&m| hits.pos[m]).collect();
        let (a, u) = pca_line(&pts);
        let line = SearchlightLine { a, u, hits : vec![], sigma0 : 0.0, theta : 0.0 };
        let res  = near.iter().map(|&m| line.dist(hits.pos[m])).sum::<f64>()/(near.len() as f64);
        cand.push((near.len(), res, near));
      }
    }
  }
  // most hits first, then the smallest residual (a stable sort keeps the order otherwise)
  cand.sort_by(|x, y| y.0.cmp(&x.0).then(x.1.total_cmp(&y.1)));
  let mut used  = HashSet::<usize>::new();
  let mut lines = Vec::<SearchlightLine>::new();
  for (_, _, near) in cand {
    let near : Vec<usize> = near.into_iter().filter(|m| !used.contains(m)).collect();
    if near.len() < s.trk_min_hits || near.iter().filter(|&&m| !hits.is_tof[m]).count() < 2 {
      continue;
    }
    let pts : Vec<DVec3> = near.iter().map(|&m| hits.pos[m]).collect();
    let (a, u) = pca_line(&pts);
    used.extend(near.iter().copied());
    lines.push(SearchlightLine { a, u, hits : near, sigma0 : s.sigma0_sec, theta : s.theta_sec });
  }
  lines
}

/// Adaptive vertex fit (deterministic annealing) to lines and
/// points (position, error), starting at v0. The weight of a
/// measurement is
///
/// w = 1/(1 + exp((chi2 - chi2_cut)/(2T)))
///
/// Returns the vertex and the number of compatible measurements
pub(crate) fn adaptive_vertex(lines  : &[SearchlightLine],
                              points : &[(DVec3, f64)],
                              hits   : &SearchlightHits,
                              s      : &SearchlightVertexSettings,
                              v0     : DVec3) -> Option<(DVec3, usize)> {
  let mut v = v0;
  for &temp in &s.temperatures {
    for _ in 0..3 {
      let mut a    = DMat3::ZERO;
      let mut b    = DVec3::ZERO;
      let mut wsum = 0.0;
      for l in lines {
        let s2   = l.sigma(v, hits).powi(2);
        let chi2 = l.dist(v).powi(2)/s2;
        let w    = 1.0/(1.0 + ((chi2 - s.chi2_cut)/(2.0*temp)).exp());
        let proj = (DMat3::IDENTITY - DMat3::from_cols(l.u*l.u.x, l.u*l.u.y, l.u*l.u.z))*(w/s2);
        a += proj;
        b += proj*l.a;
        wsum += w;
      }
      for &(p, err) in points {
        let chi2 = (v - p).length_squared()/(err*err);
        let w    = 1.0/(1.0 + ((chi2 - s.chi2_cut)/(2.0*temp)).exp());
        a += DMat3::IDENTITY*(w/(err*err));
        b += p*(w/(err*err));
        wsum += w;
      }
      if wsum < 1e-6 {
        return None;
      }
      let det = a.determinant();
      if det == 0.0 || !det.is_finite() {
        return None;
      }
      v = a.inverse()*b;
    }
  }
  let ncomp = lines.iter().filter(|l| l.dist(v).powi(2)/l.sigma(v, hits).powi(2) < s.chi2_cut).count()
            + points.iter().filter(|(p, err)| (v - *p).length_squared()/(err*err) < s.chi2_cut).count();
  Some((v, ncomp))
}

/// The starting points of the vertex fit: the points, and the
/// midpoints of the closest approach of all pairs of lines
fn vertex_seeds(lines : &[SearchlightLine], points : &[(DVec3, f64)]) -> Vec<DVec3> {
  let mut seeds : Vec<DVec3> = points.iter().map(|&(p, _)| p).collect();
  for k in 0..lines.len() {
    for l in k+1..lines.len() {
      if let Some((p1, p2)) = closest_points(lines[k].a, lines[k].u, lines[l].a, lines[l].u) {
        seeds.push(0.5*(p1 + p2));
      }
    }
  }
  seeds
}

/// chi2 of a vertex, each measurement contributing at most chi2_cut
fn vertex_chi2(v : DVec3, lines : &[SearchlightLine], points : &[(DVec3, f64)], hits : &SearchlightHits, s : &SearchlightVertexSettings) -> f64 {
  lines.iter().map(|l| (l.dist(v).powi(2)/l.sigma(v, hits).powi(2)).min(s.chi2_cut)).sum::<f64>()
    + points.iter().map(|(p, err)| ((v - *p).length_squared()/(err*err)).min(s.chi2_cut)).sum::<f64>()
}

/// All vertex candidates of the adaptive fit, one per starting
/// point (candidates within 10 mm of an earlier one are dropped):
/// (vertex, compatible measurements, chi2)
pub(crate) fn vertex_candidates(lines  : &[SearchlightLine],
                                points : &[(DVec3, f64)],
                                hits   : &SearchlightHits,
                                s      : &SearchlightVertexSettings) -> Vec<(DVec3, usize, f64)> {
  let mut out = Vec::<(DVec3, usize, f64)>::new();
  for sd in vertex_seeds(lines, points) {
    if !s.inside(sd) {
      continue;
    }
    let Some((v, ncomp)) = adaptive_vertex(lines, points, hits, s, sd) else {
      continue;
    };
    if ncomp < 2 || !s.inside(v) || out.iter().any(|c| (v - c.0).length() < 10.0) {
      continue;
    }
    out.push((v, ncomp, vertex_chi2(v, lines, points, hits, s)));
  }
  out
}

/// The tracker hit with the largest energy as a point for the
/// vertex, if above e_point_min
pub(crate) fn e_point(hits : &SearchlightHits, s : &SearchlightVertexSettings) -> Vec<(DVec3, f64)> {
  if !s.use_e_point {
    return vec![];
  }
  let mut best : Option<usize> = None;
  for k in (0..hits.len()).filter(|&k| !hits.is_tof[k]) {
    if best.is_none_or(|b| hits.e[k] > hits.e[b]) {
      best = Some(k);
    }
  }
  match best {
    Some(k) if hits.e[k] > s.e_point_min => vec![(hits.pos[k], s.e_point_err)],
    _                                     => vec![],
  }
}

/// The best adaptive fit over all starting points (the points,
/// and the midpoints of the closest approach of all pairs of
/// lines): the most compatible measurements, then the smallest
/// chi2. Returns the vertex and the number of compatible
/// measurements.
pub(crate) fn find_vertex(lines  : &[SearchlightLine],
                          points : &[(DVec3, f64)],
                          hits   : &SearchlightHits,
                          s      : &SearchlightVertexSettings) -> Option<(DVec3, usize)> {
  let mut best : Option<(DVec3, usize, f64)> = None;
  for sd in vertex_seeds(lines, points) {
    if !s.inside(sd) {
      continue;
    }
    let Some((v, ncomp)) = adaptive_vertex(lines, points, hits, s, sd) else {
      continue;
    };
    if ncomp < 2 || !s.inside(v) {
      continue;
    }
    let chi2 = vertex_chi2(v, lines, points, hits, s);
    let better = match best {
      None                  => true,
      Some((_, n, c)) => ncomp > n || (ncomp == n && chi2 < c),
    };
    if better {
      best = Some((v, ncomp, chi2));
    }
  }
  best.map(|(v, n, _)| (v, n))
}

/// The vertex from the secondaries: pairs of TOF hits at beta ~ 1
/// and tracker-only lines, the highest-energy tracker hit, and
/// the primary seed line if its seed hits look right.
///
/// # Arguments:
///   * seed : (outer, inner) seed hit
pub(crate) fn simple_vertex(hits     : &SearchlightHits,
                            s        : &SearchlightVertexSettings,
                            cleaning : &LineFitCleaning,
                            seed     : (usize, usize)) -> Option<DVec3> {
  let (o, i) = seed;
  let n      = hits.len();
  let all : Vec<usize> = (0..n).collect();
  let mut used    = HashSet::<usize>::new();
  let mut primary = None;
  if let Some((a, u, acc)) = fit_line(hits, &all, (o, i), cleaning) {
    used.extend(acc.iter().copied());
    primary = Some(SearchlightLine { a, u, hits : acc, sigma0 : s.sigma0_prim, theta : s.theta_prim });
  }
  let mut lines    = Vec::<SearchlightLine>::new();
  let mut sec_used = HashSet::<usize>::new();
  for (pi, po) in secondary_pairs(hits, s, &[o]) {
    let free : Vec<usize> = (0..n).filter(|k| !sec_used.contains(k) && (!used.contains(k) || *k == pi || *k == po)).collect();
    let (Some(spo), Some(spi)) = (free.iter().position(|&k| k == po), free.iter().position(|&k| k == pi)) else {
      continue;
    };
    if let Some((a, u, acc)) = fit_line(hits, &free, (spo, spi), cleaning) {
      let sigma0 = if acc.iter().any(|&k| !hits.is_tof[k]) { s.sigma0_sec } else { s.sigma0_tof_only };
      sec_used.extend(acc.iter().copied());
      lines.push(SearchlightLine { a, u, hits : acc, sigma0, theta : s.theta_sec });
    }
  }
  if s.trk_lines {
    let free : Vec<usize> = (0..n).filter(|k| !used.contains(k) && !sec_used.contains(k)).collect();
    lines.extend(tracker_lines(hits, &free, s));
  }
  let points = e_point(hits, s);
  let seed_ok = hits.e[o] >= s.seed_edep_min && hits.e[i] >= s.seed_edep_min && hits.t[o] < hits.t[i];
  if s.use_primary && seed_ok {
    if let Some(p) = primary {
      lines.push(p);
    }
  }
  find_vertex(&lines, &points, hits, s).map(|(v, _)| v)
}
