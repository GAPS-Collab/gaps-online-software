//! Secondary tracks and timing for the vertex of searchlight
//!
//! A RANSAC track finder with the anisotropic errors of the
//! tracker strips: a strip hit measures one coordinate well
//! (across the strip) and the other only to within the length of
//! the strip. Track candidates start from two hits in different
//! layers, take all free hits within the tolerances, are fitted
//! with per-axis errors, and the best ones are taken greedily.
//!
//! The timing score: every secondary TOF hit gives an estimate of
//! the stopping time of the primary, t - |hit - vertex|/c, and a
//! good vertex has several of them agreeing within the bounds the
//! primary allows.
// This file is part of gaps-online-software and published
// under the GPLv3 license

use std::collections::HashSet;

use glam::DVec3;

use crate::prelude::*;

use super::*;

/// Settings of the secondary track finder
#[derive(Debug, Clone)]
pub struct SearchlightTrackSettings {
  /// error across the strip (mm)
  pub sigma_strip  : f64,
  /// added to all hits for the inlier windows (scattering,
  /// alignment, mm)
  pub sigma_ms     : f64,
  /// error of a TOF hit in both coordinates (mm)
  pub tof_err      : f64,
  /// inlier window, measured coordinate and TOF, in units of
  /// the error
  pub n_sigma      : f64,
  /// along the strip: half its length plus this margin (mm)
  pub along_margin : f64,
  /// the first inlier search, around the seed line, is wider by
  /// this factor
  pub loose_factor : f64,
  /// seed hits at least this far apart in z (mm)
  pub min_dz       : f64,
  pub min_hits     : usize,
  /// tracker hits in at least this many layers
  pub min_trk      : usize,
  /// quality of a track: tracker hits count 1, TOF hits this
  pub tof_weight   : f64,
  pub max_chi2_ndf : f64,
  /// TOF pairs at beta ~ 1 as seeds too
  pub tof_pairs    : bool,
  /// scattering of the secondaries, added to the slope errors of
  /// the track (rad)
  pub theta_ms     : f64,
  /// smallest error of a track at its hits (mm)
  pub sigma0_min   : f64,
}

impl Default for SearchlightTrackSettings {
  fn default() -> Self {
    Self {
      sigma_strip  : 3.5,
      sigma_ms     : 5.0,
      tof_err      : 30.0,
      n_sigma      : 3.0,
      along_margin : 10.0,
      loose_factor : 2.5,
      min_dz       : 50.0,
      min_hits     : 3,
      min_trk      : 2,
      tof_weight   : 0.7,
      max_chi2_ndf : 10.0,
      tof_pairs    : true,
      theta_ms     : 0.1,
      sigma0_min   : 10.0,
    }
  }
}

/// Settings of the timing score
#[derive(Debug, Clone)]
pub struct SearchlightTimingSettings {
  /// agreement of the stopping time estimates (ns)
  pub window   : f64,
  /// assumed velocity of the secondaries
  pub beta_sec : f64,
  /// slowest primary
  pub beta_min : f64,
  /// margin on the bounds from the primary (ns)
  pub margin   : f64,
  /// weight of the timing score in the choice of the vertex
  pub w_time   : f64,
}

impl Default for SearchlightTimingSettings {
  fn default() -> Self {
    Self {
      window   : 0.5,
      beta_sec : 0.95,
      beta_min : 0.1,
      margin   : 0.5,
      w_time   : 1.0,
    }
  }
}

/// Per hit: fit errors in x and y, inlier windows in x and y
struct HitWindows {
  ex : Vec<f64>,
  ey : Vec<f64>,
  wx : Vec<f64>,
  wy : Vec<f64>,
}

fn hit_windows(hits : &SearchlightHits, s : &SearchlightTrackSettings) -> HitWindows {
  let n  = hits.len();
  let wt = s.n_sigma*s.tof_err.hypot(s.sigma_ms);
  let mut w = HitWindows { ex : vec![s.tof_err; n], ey : vec![s.tof_err; n], wx : vec![wt; n], wy : vec![wt; n] };
  for k in 0..n {
    if hits.measured[k] < 0 {
      continue;
    }
    let e_m = s.sigma_strip;
    let e_a = 2.0*hits.half_chord[k]/12f64.sqrt();
    let w_m = s.n_sigma*e_m.hypot(s.sigma_ms);
    let w_a = hits.half_chord[k] + s.along_margin;
    if hits.measured[k] == 0 {
      (w.ex[k], w.ey[k], w.wx[k], w.wy[k]) = (e_m, e_a, w_m, w_a);
    } else {
      (w.ex[k], w.ey[k], w.wx[k], w.wy[k]) = (e_a, e_m, w_a, w_m);
    }
  }
  w
}

/// The hits of idx within the windows (times factor) around the
/// line, with the residuals evaluated at the z of the hits
fn inliers(hits : &SearchlightHits, a : DVec3, u : DVec3, w : &HitWindows, idx : &[usize], factor : f64) -> Vec<usize> {
  if u.z.abs() < 1e-6 {
    return vec![];
  }
  idx.iter().copied().filter(|&k| {
    let p = hits.pos[k];
    let t = (p.z - a.z)/u.z;
    (p.x - (a.x + t*u.x)).abs() <= factor*w.wx[k] && (p.y - (a.y + t*u.y)).abs() <= factor*w.wy[k]
  }).collect()
}

/// The fit with per-axis errors: the fit, a point and the direction
fn fit(hits : &SearchlightHits, w : &HitWindows, idx : &[usize]) -> Option<(LineFit, DVec3, DVec3)> {
  let xs : Vec<f32> = idx.iter().map(|&k| hits.pos[k].x as f32).collect();
  let ys : Vec<f32> = idx.iter().map(|&k| hits.pos[k].y as f32).collect();
  let zs : Vec<f32> = idx.iter().map(|&k| hits.pos[k].z as f32).collect();
  let ex : Vec<f32> = idx.iter().map(|&k| w.ex[k] as f32).collect();
  let ey : Vec<f32> = idx.iter().map(|&k| w.ey[k] as f32).collect();
  let f = LineFit::fit(&xs, &ys, &zs, Some(&ex), Some(&ey))?;
  if ![f.x0, f.y0, f.tx, f.ty].iter().all(|x| x.is_finite()) {
    return None;
  }
  let a = DVec3::new(f.x0 as f64, f.y0 as f64, f.z0 as f64);
  let u = DVec3::new(f.tx as f64, f.ty as f64, 1.0).normalize();
  Some((f, a, u))
}

/// A secondary track
#[derive(Debug, Clone)]
pub struct SearchlightTrack {
  pub line  : SearchlightLine,
  pub hits  : Vec<usize>,
  pub ntrk  : usize,
  pub ntof  : usize,
  pub chi2n : f64,
}

fn track_candidate(hits : &SearchlightHits, w : &HitWindows, free : &[usize], seed : (usize, usize), s : &SearchlightTrackSettings)
  -> Option<(LineFit, Vec<usize>, usize, usize, f64)> {
  let d = hits.pos[seed.1] - hits.pos[seed.0];
  if d.z.abs() < s.min_dz {
    return None;
  }
  let mut idx = inliers(hits, hits.pos[seed.0], d.normalize(), w, free, s.loose_factor);
  for _ in 0..3 {
    if idx.len() < s.min_hits {
      return None;
    }
    let (_, a, u) = fit(hits, w, &idx)?;
    let new = inliers(hits, a, u, w, free, 1.0);
    if new.iter().collect::<HashSet<_>>() == idx.iter().collect::<HashSet<_>>() {
      break;
    }
    idx = new;
  }
  if idx.len() < s.min_hits {
    return None;
  }
  let trk : Vec<usize> = idx.iter().copied().filter(|&k| !hits.is_tof[k]).collect();
  let layers : HashSet<i64> = trk.iter().map(|&k| hits.pos[k].z.round() as i64).collect();
  if layers.len() < s.min_trk {
    return None;
  }
  let (f, _, _) = fit(hits, w, &idx)?;
  let chi2n = f.chi2 as f64/(f.ndof.max(1) as f64);
  if chi2n > s.max_chi2_ndf {
    return None;
  }
  let ntrk = trk.len();
  let ntof = idx.len() - ntrk;
  Some((f, idx, ntrk, ntof, chi2n))
}

/// The track as a line for the vertex fit, with its error at its
/// hits and the growth of the error from the fit
fn track_line(f : &LineFit, hits : Vec<usize>, s : &SearchlightTrackSettings) -> SearchlightLine {
  let a      = DVec3::new(f.x0 as f64, f.y0 as f64, f.z0 as f64);
  let u      = DVec3::new(f.tx as f64, f.ty as f64, 1.0).normalize();
  let theta  = ((f.tx_err as f64).hypot(f.ty_err as f64)/2f64.sqrt()).hypot(s.theta_ms);
  let sigma0 = s.sigma0_min.max((f.x0_err as f64).hypot(f.y0_err as f64)/2f64.sqrt());
  SearchlightLine { a, u, hits, sigma0, theta }
}

/// The secondary tracks among the free hits: the best candidate
/// (most hits, tracker hits weigh more, then the smallest chi2)
/// is taken, its hits are removed, and again
///
/// # Arguments:
///   * free    : the hits which can be used
///   * exclude : TOF hits which can not be part of a TOF seed pair
pub(crate) fn find_tracks(hits    : &SearchlightHits,
                          free    : &[usize],
                          s       : &SearchlightTrackSettings,
                          vs      : &SearchlightVertexSettings,
                          exclude : &[usize]) -> Vec<SearchlightTrack> {
  let w = hit_windows(hits, s);
  let mut free : Vec<usize> = free.to_vec();
  free.sort();
  let pairs = if s.tof_pairs { secondary_pairs(hits, vs, exclude) } else { vec![] };
  let mut tracks = Vec::<SearchlightTrack>::new();
  while free.len() >= s.min_hits {
    let fs : HashSet<usize> = free.iter().copied().collect();
    let trk : Vec<usize> = free.iter().copied().filter(|&k| !hits.is_tof[k]).collect();
    let tof : Vec<usize> = free.iter().copied().filter(|&k| hits.is_tof[k]).collect();
    let mut seeds = Vec::<(usize, usize)>::new();
    for x in 0..trk.len() {
      for y in x+1..trk.len() {
        seeds.push((trk[x], trk[y]));
      }
      for &k in &tof {
        seeds.push((k, trk[x]));
      }
    }
    for &(pi, po) in &pairs {
      if fs.contains(&pi) && fs.contains(&po) {
        seeds.push((po, pi));
      }
    }
    let mut best : Option<((f64, f64), (LineFit, Vec<usize>, usize, usize, f64))> = None;
    for sd in seeds {
      let Some(c) = track_candidate(hits, &w, &free, sd, s) else {
        continue;
      };
      let key = (c.2 as f64 + s.tof_weight*c.3 as f64, -c.4);
      let better = match &best {
        None         => true,
        Some((k, _)) => key.0 > k.0 || (key.0 == k.0 && key.1 > k.1),
      };
      if better {
        best = Some((key, c));
      }
    }
    let Some((_, (f, idx, ntrk, ntof, chi2n))) = best else {
      break;
    };
    let used : HashSet<usize> = idx.iter().copied().collect();
    free.retain(|k| !used.contains(k));
    tracks.push(SearchlightTrack { line : track_line(&f, idx.clone(), s), hits : idx, ntrk, ntof, chi2n });
  }
  tracks
}

/// The timing score of a vertex candidate v: the number of
/// secondary TOF hits which agree on one stopping time within
/// the bounds from the primary, and that stopping time.
///
/// # Arguments:
///   * anchor  : the last TOF hit of the primary
///   * exclude : the hits of the primary
pub(crate) fn timing_score(v : DVec3, hits : &SearchlightHits, anchor : usize, exclude : &HashSet<usize>, s : &SearchlightTimingSettings) -> (usize, f64) {
  let ta = hits.t[anchor];
  let da = (v - hits.pos[anchor]).length();
  let lo = ta + da/C_LIGHT_MM_NS - s.margin;
  let hi = ta + da/(s.beta_min*C_LIGHT_MM_NS) + s.margin;
  let mut est = Vec::<f64>::new();
  for k in 0..hits.ntof {
    if exclude.contains(&k) || !hits.t[k].is_finite() || hits.t[k] <= ta {
      continue;
    }
    let tv = hits.t[k] - (hits.pos[k] - v).length()/(s.beta_sec*C_LIGHT_MM_NS);
    if lo <= tv && tv <= hi {
      est.push(tv);
    }
  }
  if est.is_empty() {
    return (0, f64::NAN);
  }
  est.sort_by(|a, b| a.total_cmp(b));
  // the largest group within the window
  let (mut best, mut t0, mut j) = (0usize, f64::NAN, 0usize);
  for k in 0..est.len() {
    while est[k] - est[j] > s.window {
      j += 1;
    }
    if k - j + 1 > best {
      best = k - j + 1;
      let g = &est[j..=k];
      let mut g = g.to_vec();
      g.sort_by(|a, b| a.total_cmp(b));
      let m = g.len();
      t0 = if m % 2 == 1 { g[m/2] } else { 0.5*(g[m/2 - 1] + g[m/2]) };
    }
  }
  (best, t0)
}

/// The quality of a vertex: (TOF hits agreeing on the stopping
/// time, compatible measurements, from tracks (1) or only the
/// high-energy tracker hit (0), number of secondary tracks)
pub type SearchlightVertexQuality = [f64; 4];

/// A vertex which does not use the primary line: from the
/// secondary tracks and the highest-energy tracker hit, chosen
/// among the candidates of the adaptive fit by the timing score,
/// else the highest-energy tracker hit itself.
///
/// Only for seeds which pass the flags (both seed hits with
/// edep_min, outer earlier) and with a primary seed fit, otherwise
/// None (then the simple vertex is used). The inner Option is None
/// if no vertex is found.
pub(crate) fn independent_vertex(hits : &SearchlightHits, s : &SearchlightSettings, seed : (usize, usize))
  -> Option<Option<(DVec3, SearchlightVertexQuality)>> {
  let (o, i) = seed;
  let vs = &s.vertex;
  let seed_ok = hits.e[o] >= vs.seed_edep_min && hits.e[i] >= vs.seed_edep_min && hits.t[o] < hits.t[i];
  let all : Vec<usize> = (0..hits.len()).collect();
  let (_, _, prim) = fit_line(hits, &all, (o, i), &s.cleaning)?;
  if !seed_ok {
    return None;
  }
  let used : HashSet<usize> = prim.iter().copied().collect();
  let tracks = if s.use_tracks {
    let free : Vec<usize> = all.iter().copied().filter(|k| !used.contains(k) && *k != o).collect();
    find_tracks(hits, &free, &s.tracks, vs, &[o])
  } else {
    vec![]
  };
  let points = e_point(hits, vs);
  let lines : Vec<SearchlightLine> = tracks.iter().map(|t| t.line.clone()).collect();
  let ntracks = tracks.len() as f64;
  let timing = |v : DVec3| if s.use_timing { timing_score(v, hits, i, &used, &s.timing).0 } else { 0 };
  let w_time = if s.use_timing { s.timing.w_time } else { 0.0 };
  let mut best : Option<((f64, f64), DVec3, usize, usize)> = None;
  for (v, ncomp, chi2) in vertex_candidates(&lines, &points, hits, vs) {
    let nt  = timing(v);
    let key = (ncomp as f64 + w_time*nt as f64, -chi2);
    let better = match &best {
      None               => true,
      Some((k, _, _, _)) => key.0 > k.0 || (key.0 == k.0 && key.1 > k.1),
    };
    if better {
      best = Some((key, v, ncomp, nt));
    }
  }
  if let Some((_, v, ncomp, nt)) = best {
    return Some(Some((v, [nt as f64, ncomp as f64, 1.0, ntracks])));
  }
  if let Some(&(p, _)) = points.first() {
    return Some(Some((p, [timing(p) as f64, 1.0, 0.0, ntracks])));
  }
  Some(None)
}
