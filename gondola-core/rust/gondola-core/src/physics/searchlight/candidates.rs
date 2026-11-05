//! Candidate lines for the primary, their features and the
//! selection
// This file is part of gaps-online-software and published
// under the GPLv3 license

use glam::DVec3;

use crate::prelude::*;

use super::{fit_line, SearchlightHits, SearchlightSettings, SearchlightVertexQuality, C_LIGHT_MM_NS};

/// The kind of a candidate line for the primary: through which
/// two points its cleaned fit is seeded
#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash, FromRepr, AsRefStr, EnumIter)]
#[repr(u8)]
#[cfg_attr(feature = "pybindings", pyclass(eq, eq_int, hash, frozen))]
pub enum SearchlightCandidate {
  /// the earliest outer and the earliest inner TOF hit
  Today        = 0u8,
  /// the outer seed hit and the vertex
  OuterVertex  = 1u8,
  /// the inner seed hit and the vertex
  InnerVertex  = 2u8,
  /// the outer seed hit and a high-energy tracker hit
  OuterTracker = 3u8,
  /// the inner seed hit and a high-energy tracker hit
  InnerTracker = 4u8,
  /// the outer seed hit and a triggered inner paddle without hit
  OuterTrigger = 5u8,
  /// the inner seed hit and a triggered outer paddle without hit
  InnerTrigger = 6u8,
}

impl SearchlightCandidate {
  /// Number of kinds (for the one-hot features)
  pub const N : usize = 7;
}

impl fmt::Display for SearchlightCandidate {
  fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
    write!(f, "<SearchlightCandidate: {}>", self.as_ref())
  }
}

/// A candidate line for the primary
#[derive(Debug, Clone)]
pub struct SearchlightCandidateFit {
  pub kind     : SearchlightCandidate,
  /// a point on the line (mm)
  pub a        : DVec3,
  /// unit vector along the line, pointing inwards (from the
  /// outer-most point of the candidate to the other one)
  pub u        : DVec3,
  /// accepted hits (indices into SearchlightHits; an extra point
  /// such as the vertex has the index SearchlightHits::len())
  pub hits     : Vec<usize>,
  /// where the candidate starts and where it points to, for its
  /// orientation
  pub start    : DVec3,
  pub target   : DVec3,
  pub features : Vec<f64>,
  pub prob     : f64,
  /// if the candidate took part in the selection (it keeps the
  /// good seed hit; if no candidate does, the first one)
  pub allowed  : bool,
}

/// The fit seeded through the hits seed (indices into hits),
/// cleaned with cleaning, optionally with an extra point with
/// its own error (it gets the index hits.len())
pub(crate) fn candidate(hits     : &SearchlightHits,
                        seed     : (usize, usize),
                        kind     : SearchlightCandidate,
                        cleaning : &LineFitCleaning,
                        extra    : Option<(DVec3, f64)>,
                        start    : DVec3,
                        target   : DVec3) -> Option<SearchlightCandidateFit> {
  let with;
  let h = match extra {
    None             => hits,
    Some((p, err))   => {
      with = hits.with_point(p, err);
      &with
    }
  };
  let all : Vec<usize> = (0..h.len()).collect();
  let (a, u, acc) = fit_line(h, &all, seed, cleaning)?;
  Some(SearchlightCandidateFit { kind, a, u, hits : acc, start, target, features : vec![], prob : 0.0, allowed : false })
}

/// Energy-weighted inliers: primary-like hits count 1, low
/// energy ones 0.2
pub(crate) fn score(hits : &SearchlightHits, acc : &[usize], settings : &SearchlightSettings) -> f64 {
  acc.iter().filter(|&&k| k < hits.len()).map(|&k| {
    let min = if hits.is_tof[k] { settings.edep_min } else { settings.trk_e_min };
    if hits.e[k] >= min { 1.0 } else { 0.2 }
  }).sum()
}

/// Distance of the point p from the line through a along u
fn dist(a : DVec3, u : DVec3, p : DVec3) -> f64 {
  let w = p - a;
  (w - w.dot(u)*u).length()
}

/// The features of a candidate for the selection (u has to be
/// oriented already):
///
/// 1, primary-like TOF inliers, primary-like tracker inliers,
/// other inliers, log(1 + tracker energy), mean residual / 50 mm,
/// beta outside (0.05, 0.95), beta not measurable, distance to the
/// vertex / 50 mm (at most 5), no vertex, one-hot candidate kind,
/// and for the vertex candidates the quality of the independent
/// vertex (zeros otherwise)
pub(crate) fn features(hits     : &SearchlightHits,
                       c        : &SearchlightCandidateFit,
                       settings : &SearchlightSettings,
                       vertex   : Option<DVec3>,
                       quality  : Option<&SearchlightVertexQuality>) -> Vec<f64> {
  let acc : Vec<usize> = c.hits.iter().copied().filter(|&k| k < hits.len()).collect();
  let tof : Vec<usize> = acc.iter().copied().filter(|&k| hits.is_tof[k]).collect();
  let trk : Vec<usize> = acc.iter().copied().filter(|&k| !hits.is_tof[k]).collect();
  let n_hi_tof = tof.iter().filter(|&&k| hits.e[k] >= settings.edep_min).count();
  let n_hi_trk = trk.iter().filter(|&&k| hits.e[k] >= settings.trk_e_min).count();
  let n_lo     = acc.len() - n_hi_tof - n_hi_trk;
  let e_trk    = trk.iter().map(|&k| hits.e[k]).sum::<f64>().ln_1p();
  let res = if acc.is_empty() {
    1.0
  } else {
    acc.iter().map(|&k| dist(c.a, c.u, hits.pos[k])).sum::<f64>()/(acc.len() as f64)/50.0
  };
  // beta from the TOF hits along the line (inwards): the times should increase
  let (mut beta_bad, mut beta_none) = (0.0, 1.0);
  if tof.len() >= 2 {
    let s : Vec<f64> = tof.iter().map(|&k| (hits.pos[k] - c.start).dot(c.u)).collect();
    let t : Vec<f64> = tof.iter().map(|&k| hits.t[k]).collect();
    let smin = s.iter().copied().fold(f64::INFINITY, f64::min);
    let smax = s.iter().copied().fold(f64::NEG_INFINITY, f64::max);
    if smax - smin > 50.0 {
      let nf = s.len() as f64;
      let sm = s.iter().sum::<f64>()/nf;
      let tm = t.iter().sum::<f64>()/nf;
      let slope = s.iter().zip(&t).map(|(x, y)| (x - sm)*(y - tm)).sum::<f64>()
                / s.iter().map(|x| (x - sm).powi(2)).sum::<f64>();
      let beta = if slope != 0.0 { 1.0/(slope*C_LIGHT_MM_NS) } else { f64::INFINITY };
      beta_none = 0.0;
      beta_bad  = if beta > 0.05 && beta < 0.95 { 0.0 } else { 1.0 };
    }
  }
  let (dv, nov) = match vertex {
    Some(v) => ((dist(c.a, c.u, v)/50.0).min(5.0), 0.0),
    None    => (0.0, 1.0),
  };
  let mut f = vec![1.0, n_hi_tof as f64, n_hi_trk as f64, n_lo as f64, e_trk, res, beta_bad, beta_none, dv, nov];
  for k in 0..SearchlightCandidate::N {
    f.push(if c.kind as usize == k { 1.0 } else { 0.0 });
  }
  let is_vertex = matches!(c.kind, SearchlightCandidate::OuterVertex | SearchlightCandidate::InnerVertex);
  match quality {
    Some(q) if is_vertex => f.extend_from_slice(q),
    _                    => f.extend_from_slice(&[0.0; 4]),
  }
  f
}

/// Number of features of a candidate
pub const SEARCHLIGHT_N_FEATURES : usize = 10 + SearchlightCandidate::N + 4;
