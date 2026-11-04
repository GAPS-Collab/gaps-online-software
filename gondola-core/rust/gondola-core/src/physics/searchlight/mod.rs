//! Searchlight - the reconstruction of the primary track
//!
//! The cleaned linefit is seeded through the earliest outer and
//! the earliest inner TOF hit. When the primary's paddle is not
//! read out, one of these seed hits belongs to a secondary, and
//! the fit goes astray. Searchlight flags such seed hits by their
//! timing and energy, and replaces them by other points: the
//! vertex (the stopping point of the primary, found from the
//! secondaries like a searchlight sweeping back to their origin),
//! high-energy tracker hits and triggered paddles without a hit.
//! Each candidate line is a cleaned fit, and a logistic
//! regression over a few features of the candidates picks one.
//!
//! All lengths are in mm, times in ns, energies in MeV.
// This file is part of gaps-online-software and published
// under the GPLv3 license

pub mod detector;
pub mod vertex;
pub mod candidates;
pub mod tracks;
#[cfg(feature="pybindings")]
pub mod python;

pub use detector::*;
pub use vertex::*;
pub use candidates::*;
pub use tracks::*;

use glam::DVec3;

use crate::prelude::*;

/// Speed of light (mm/ns)
pub(crate) const C_LIGHT_MM_NS : f64 = 299.792458;

//---------------------------------------------------------

/// The hits of an event, as searchlight uses them: TOF hits
/// first, then tracker hits
#[derive(Debug, Clone, Default)]
pub struct SearchlightHits {
  /// positions (mm)
  pub pos       : Vec<DVec3>,
  /// position errors in x and y for the fits (mm)
  pub err       : Vec<f64>,
  /// TOF: normalized hit times (ns), tracker: NaN
  pub t         : Vec<f64>,
  /// TOF: edep, tracker: calibrated energy (MeV)
  pub e         : Vec<f64>,
  pub is_tof    : Vec<bool>,
  /// inner TOF (cube) paddle
  pub inner     : Vec<bool>,
  /// TOF: paddle id, tracker: 0
  pub paddle_id : Vec<u8>,
  /// tracker: the coordinate the strip measures (0: x, 1: y),
  /// -1 for TOF hits and unknown strips
  pub measured  : Vec<i8>,
  /// tracker: half length of the strip (mm), TOF: 0
  pub half_chord : Vec<f64>,
  /// number of TOF hits
  pub ntof      : usize,
}

impl SearchlightHits {
  /// Position error of all hits in the fits (mm)
  pub const ERR : f64 = LINEFIT_DEFAULT_ERR_XY as f64;

  /// # Arguments:
  ///   * tof      : TOF hits (position in mm, normalized time,
  ///                edep, paddle id)
  ///   * tracker  : tracker hits (position in mm, energy in MeV,
  ///                strip id)
  ///   * geometry : if given, TOF hits in overlapping paddles at
  ///                the same time are moved to the middle of the
  ///                overlap (TofEvent::merge_overlapping_hits),
  ///                and the strips of the tracker hits are looked
  ///                up (for the secondary tracks)
  pub fn new(tof : &[(DVec3, f64, f64, u8)], tracker : &[(DVec3, f64, u32)], geometry : Option<&SearchlightGeometry>) -> Self {
    let overlaps = geometry.map(|g| &g.overlaps);
    let mut tof_pos : Vec<DVec3> = tof.iter().map(|h| h.0).collect();
    if let Some(ov) = overlaps {
      let mut ev = TofEvent::new();
      for h in tof {
        let mut hit = TofHit::new();
        hit.paddle_id = h.3;
        (hit.x, hit.y, hit.z) = (h.0.x as f32, h.0.y as f32, h.0.z as f32);
        hit.event_t0 = h.1 as f32;
        ev.hits.push(hit);
      }
      ev.hit_times_normalized = true;
      if ev.merge_overlapping_hits(ov, TofPaddleOverlaps::DEFAULT_MAX_DT) > 0 {
        for (p, h) in tof_pos.iter_mut().zip(&ev.hits) {
          // only the merged hits change, the others keep their full precision
          let moved = DVec3::new(h.x as f64, h.y as f64, h.z as f64);
          if (moved - *p).length() > 1e-3 {
            *p = moved;
          }
        }
      }
    }
    let n = tof.len() + tracker.len();
    let mut hits = Self {
      pos       : Vec::with_capacity(n),
      err       : vec![Self::ERR; n],
      t         : Vec::with_capacity(n),
      e         : Vec::with_capacity(n),
      is_tof    : Vec::with_capacity(n),
      inner     : Vec::with_capacity(n),
      paddle_id : Vec::with_capacity(n),
      measured  : vec![-1; n],
      half_chord : vec![0.0; n],
      ntof      : tof.len(),
    };
    for (h, p) in tof.iter().zip(tof_pos) {
      hits.pos.push(p);
      hits.t.push(h.1);
      hits.e.push(h.2);
      hits.is_tof.push(true);
      hits.inner.push(h.3 <= TOF_INNER_MAX_PADDLE_ID);
      hits.paddle_id.push(h.3);
    }
    for (k, h) in tracker.iter().enumerate() {
      hits.pos.push(h.0);
      hits.t.push(f64::NAN);
      hits.e.push(h.1);
      hits.is_tof.push(false);
      hits.inner.push(false);
      hits.paddle_id.push(0);
      if let Some(st) = geometry.and_then(|g| g.strips.get(&h.2)) {
        hits.measured[tof.len() + k]   = st.measured as i8;
        hits.half_chord[tof.len() + k] = st.half_chord;
      }
    }
    hits
  }

  /// The hits of a TelemetryEvent. The TOF hit times have to be
  /// normalized (TofEvent::normalize_hit_times), the tracker hits
  /// calibrated (energy in MeV) and with coordinates.
  pub fn from_event(event : &TelemetryEvent, geometry : Option<&SearchlightGeometry>) -> Self {
    let tof : Vec<(DVec3, f64, f64, u8)> = event.tof_event.hits.iter().map(|h| {
      (DVec3::new(h.x as f64, h.y as f64, h.z as f64), h.event_t0 as f64, h.get_edep() as f64, h.paddle_id)
    }).collect();
    // the tracker hit coordinates are in cm
    let trk : Vec<(DVec3, f64, u32)> = event.tracker_hits.iter().filter(|h| h.has_coordinates).map(|h| {
      (10.0*DVec3::new(h.x as f64, h.y as f64, h.z as f64), h.energy as f64, h.get_stripid())
    }).collect();
    Self::new(&tof, &trk, geometry)
  }

  pub fn len(&self) -> usize {
    self.pos.len()
  }

  pub fn is_empty(&self) -> bool {
    self.pos.is_empty()
  }

  /// The hits with an extra point (e.g. the vertex) appended, with
  /// its own error, no time and no energy
  pub fn with_point(&self, p : DVec3, err : f64) -> Self {
    let mut h = self.clone();
    h.pos.push(p);
    h.err.push(err);
    h.t.push(f64::NAN);
    h.e.push(0.0);
    h.is_tof.push(false);
    h.inner.push(false);
    h.paddle_id.push(0);
    h.measured.push(-1);
    h.half_chord.push(0.0);
    h
  }

  /// The earliest outer and the earliest inner TOF hit, if both exist
  pub fn seed(&self) -> Option<(usize, usize)> {
    let first = |inner : bool| {
      (0..self.ntof).filter(|&k| self.inner[k] == inner && self.t[k].is_finite())
                    .min_by(|&x, &y| self.t[x].total_cmp(&self.t[y]))
    };
    Some((first(false)?, first(true)?))
  }
}

/// The cleaned fit through the hits idx, seeded through seed
/// (indices into idx). Returns a point on the line, its direction
/// (unit vector) and the accepted hits (indices into hits).
pub(crate) fn fit_line(hits     : &SearchlightHits,
                       idx      : &[usize],
                       seed     : (usize, usize),
                       cleaning : &LineFitCleaning) -> Option<(DVec3, DVec3, Vec<usize>)> {
  let xs   : Vec<f32> = idx.iter().map(|&k| hits.pos[k].x as f32).collect();
  let ys   : Vec<f32> = idx.iter().map(|&k| hits.pos[k].y as f32).collect();
  let zs   : Vec<f32> = idx.iter().map(|&k| hits.pos[k].z as f32).collect();
  let errs : Vec<f32> = idx.iter().map(|&k| hits.err[k] as f32).collect();
  let r = LineFit::fit_cleaned_with_seed(&xs, &ys, &zs, Some(&errs), Some(&errs), seed, cleaning)?;
  let a = DVec3::new(r.fit.x0 as f64, r.fit.y0 as f64, r.fit.z0 as f64);
  let d = r.fit.direction();
  let u = DVec3::new(d.0 as f64, d.1 as f64, d.2 as f64).normalize();
  let acc = idx.iter().zip(&r.accepted).filter(|(_, ok)| **ok).map(|(&k, _)| k).collect();
  Some((a, u, acc))
}

//---------------------------------------------------------

/// Settings of searchlight
#[derive(Debug, Clone)]
#[cfg_attr(feature="pybindings", pyclass)]
pub struct SearchlightSettings {
  /// hit cleaning of all candidate fits
  pub cleaning      : LineFitCleaning,
  /// TOF hits below this edep are not primary-like (MeV); a seed
  /// hit below it is flagged
  pub edep_min      : f64,
  /// tracker hits above this energy are primary-like (MeV) and
  /// seed candidates
  pub trk_e_min     : f64,
  /// error of the vertex as a fit point (mm)
  pub vertex_err    : f64,
  pub use_vertex    : bool,
  pub use_tracker   : bool,
  /// triggered paddles without a hit: candidates through the point
  /// on the paddle closest to the best candidate, iterated
  /// trig_iter times, if the paddle is closer than trig_max_dist
  /// (mm), with the error trig_err (mm)
  pub use_trigger   : bool,
  pub trig_max_dist : f64,
  pub trig_err      : f64,
  pub trig_iter     : usize,
  pub vertex        : SearchlightVertexSettings,
  /// For seeds which pass the flags: a vertex without the primary
  /// line (independent_vertex), offered as an outer+vertex candidate
  /// if today's line has no primary-like tracker hits, with its
  /// quality as features. Otherwise the simple vertex is used.
  pub independent_vertex : bool,
  /// in the independent vertex: secondary tracks from the RANSAC
  /// track finder (needs the strips in the geometry), otherwise
  /// only the highest-energy tracker hit
  pub use_tracks    : bool,
  /// in the independent vertex: the timing score for the choice
  /// among the vertex candidates and as a quality feature
  pub use_timing    : bool,
  pub tracks        : SearchlightTrackSettings,
  pub timing        : SearchlightTimingSettings,
  /// weights of the logistic regression over the features of the
  /// candidates (see candidates::features). Empty: the candidate
  /// with the most energy-weighted inliers wins.
  pub weights       : Vec<f64>,
}

impl SearchlightSettings {
  /// Weights of the selection with all parts, trained on the
  /// antiproton MC (strip fix, 2026-11) with 3000 events: a
  /// candidate is good if it is within 4 degree of the true
  /// direction
  pub const DEFAULT_WEIGHTS : [f64; SEARCHLIGHT_N_FEATURES] = [
    -2.0259056637107813, 0.51971947247767, 0.13084705001546879, 0.006778196617624869, 0.5340678300859675,
    -0.5548880842904442, -1.6861963790174055, -0.2943184680426944, -0.19926527353671283, -0.2682748592678886,
     0.31781599292015067, 0.2635642103483618, -1.5670455931791791, 0.2969754389763969, -1.3270707031738658,
     0.4922798347333855, -0.9394941525994619, -0.09698240722520546, -0.43312250283369724, 0.485782776874773,
    -0.0018643037252919084];

  /// Weights of the selection without the independent vertex
  /// (phase A), trained in the same way. The quality features
  /// are not used.
  pub const PHASE_A_WEIGHTS : [f64; SEARCHLIGHT_N_FEATURES] = [
    -1.9597471282846652, 0.47967473408398714, 0.12602197075909988, -0.006924768098990917, 0.5338039876659284,
    -0.5706451950812257, -1.4351559632915485, -0.18620606160329897, -0.2766281152833499, -0.42066000833750733,
     0.2803266646748128, 0.4170075448442111, -1.597282633514432, 0.3642101986887803, -1.2901513177920285,
     0.5578027382599217, -0.9227856213910649, 0.0, 0.0, 0.0, 0.0];

  /// The settings without the independent vertex, the secondary
  /// tracks and the timing (phase A), with their weights
  pub fn phase_a() -> Self {
    Self {
      independent_vertex : false,
      use_tracks         : false,
      use_timing         : false,
      weights            : Self::PHASE_A_WEIGHTS.to_vec(),
      ..Self::default()
    }
  }
}

impl Default for SearchlightSettings {
  fn default() -> Self {
    let mut cleaning = LineFitCleaning::new(150.0);
    cleaning.final_radius = 75.0;
    Self {
      cleaning,
      edep_min      : 4.0,
      trk_e_min     : 3.0,
      vertex_err    : 30.0,
      use_vertex    : true,
      use_tracker   : true,
      use_trigger   : true,
      trig_max_dist : 400.0,
      trig_err      : 50.0,
      trig_iter     : 3,
      vertex        : SearchlightVertexSettings::default(),
      independent_vertex : true,
      use_tracks    : true,
      use_timing    : true,
      tracks        : SearchlightTrackSettings::default(),
      timing        : SearchlightTimingSettings::default(),
      weights       : Self::DEFAULT_WEIGHTS.to_vec(),
    }
  }
}

//---------------------------------------------------------

/// The result of searchlight for one event
#[derive(Debug, Clone, Default)]
#[cfg_attr(feature="pybindings", pyclass)]
pub struct SearchlightResult {
  /// (outer, inner) seed hit, indices into SearchlightHits
  pub seed       : Option<(usize, usize)>,
  /// (outer seed hit flagged, inner seed hit flagged)
  pub flags      : (bool, bool),
  /// the vertex used for the candidates
  pub vertex     : Option<DVec3>,
  /// its quality, if it is the independent vertex
  pub vertex_quality : Option<SearchlightVertexQuality>,
  /// all candidates, and the chosen one (index into candidates)
  pub candidates : Vec<SearchlightCandidateFit>,
  pub best       : Option<usize>,
}

impl SearchlightResult {
  /// The chosen candidate
  pub fn primary(&self) -> Option<&SearchlightCandidateFit> {
    self.best.map(|k| &self.candidates[k])
  }
}

impl fmt::Display for SearchlightResult {
  fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
    match self.primary() {
      None    => write!(f, "<SearchlightResult: no primary>"),
      Some(c) => write!(f, "<SearchlightResult: {}, direction ({:.3}, {:.3}, {:.3}), probability {:.3}, flags {:?}, {} candidates>",
                        c.kind.as_ref(), c.u.x, c.u.y, c.u.z, c.prob, self.flags, self.candidates.len()),
    }
  }
}

//---------------------------------------------------------

/// The searchlight reconstruction of the primary track
#[derive(Debug, Clone, Default)]
#[cfg_attr(feature="pybindings", pyclass)]
pub struct Searchlight {
  pub geometry : SearchlightGeometry,
  pub settings : SearchlightSettings,
}

impl Searchlight {
  pub fn new(geometry : SearchlightGeometry, settings : SearchlightSettings) -> Self {
    Self { geometry, settings }
  }

  /// Reconstruct the primary of a TelemetryEvent (normalized TOF
  /// hit times, calibrated tracker hits), see reconstruct_hits.
  pub fn reconstruct(&self, event : &TelemetryEvent, triggered : &[u8]) -> SearchlightResult {
    if !event.tof_event.hit_times_normalized {
      warn!("The TOF hit times are not normalized, searchlight can not reconstruct this event!");
      return SearchlightResult::default();
    }
    let hits = SearchlightHits::from_event(event, Some(&self.geometry));
    self.reconstruct_hits(&hits, triggered)
  }

  /// Reconstruct the primary
  ///
  /// # Arguments:
  ///   * hits      : the hits of the event
  ///   * triggered : paddle ids of triggered paddles without a
  ///                 hit (may be empty)
  pub fn reconstruct_hits(&self, hits : &SearchlightHits, triggered : &[u8]) -> SearchlightResult {
    use SearchlightCandidate as K;
    let s = &self.settings;
    let mut res = SearchlightResult::default();
    let Some((o, i)) = hits.seed() else {
      return res;
    };
    res.seed = Some((o, i));
    let outer_bad = hits.t[o] > hits.t[i] || hits.e[o] < s.edep_min;
    let inner_bad = hits.e[i] < s.edep_min;
    res.flags = (outer_bad, inner_bad);
    let p = &hits.pos;
    let n = hits.len();
    let mut cands = Vec::<SearchlightCandidateFit>::new();
    cands.extend(candidate(hits, (o, i), K::Today, s, None, p[o], p[i]));
    // the independent vertex for seeds which pass the flags, the
    // simple vertex otherwise
    let independent = if s.use_vertex && s.independent_vertex { independent_vertex(hits, s, (o, i)) } else { None };
    let (v, quality) = match independent {
      Some(Some((v, q))) => (Some(v), Some(q)),
      Some(None)         => (None, None),
      None               => (if s.use_vertex { simple_vertex(hits, &s.vertex, &s.cleaning, (o, i)) } else { None }, None),
    };
    res.vertex = v;
    res.vertex_quality = quality;
    if let Some(v) = v {
      if (v - p[o]).length() > 1.0 {
        cands.extend(candidate(hits, (o, n), K::OuterVertex, s, Some((v, s.vertex_err)), p[o], v));
      }
      if (v - p[i]).length() > 1.0 {
        cands.extend(candidate(hits, (i, n), K::InnerVertex, s, Some((v, s.vertex_err)), p[i], v));
      }
    }
    if s.use_tracker {
      for k in (0..n).filter(|&k| !hits.is_tof[k] && hits.e[k] >= s.trk_e_min) {
        for (seed, kind) in [(o, K::OuterTracker), (i, K::InnerTracker)] {
          if (p[k] - p[seed]).length() > 1.0 {
            cands.extend(candidate(hits, (seed, k), kind, s, None, p[seed], p[k]));
          }
        }
      }
    }
    // which candidates may replace the seed: those which keep the
    // good seed hit
    let (allowed, flagged_outer) : (Vec<K>, Option<bool>) = match (outer_bad, inner_bad) {
      (true, false) => (vec![K::InnerVertex, K::InnerTracker, K::InnerTrigger], Some(true)),
      (false, true) => (vec![K::OuterVertex, K::OuterTracker, K::OuterTrigger], Some(false)),
      _             => {
        // with the independent vertex: outer+vertex too, if today's
        // line has no primary-like tracker hits
        let mut allowed = vec![K::Today];
        if s.independent_vertex {
          let n_hi = cands.iter().find(|c| c.kind == K::Today).map(|c| {
            c.hits.iter().filter(|&&k| k < n && !hits.is_tof[k] && hits.e[k] >= s.trk_e_min).count()
          });
          if n_hi.is_none_or(|n_hi| n_hi == 0) {
            allowed.push(K::OuterVertex);
          }
        }
        (allowed, None)
      }
    };
    // triggered paddles without a hit on the flagged side: the line
    // from the good seed hit through the closest point on the paddle,
    // iterated
    if let (true, Some(outer)) = (s.use_trigger, flagged_outer) {
      let (keep, kind) = if outer { (i, K::InnerTrigger) } else { (o, K::OuterTrigger) };
      let base : Vec<&SearchlightCandidateFit> = {
        let b : Vec<_> = cands.iter().filter(|c| allowed.contains(&c.kind)).collect();
        if b.is_empty() { cands.iter().collect() } else { b }
      };
      let mut b0 : Option<&SearchlightCandidateFit> = None;
      for c in base {
        if b0.is_none_or(|b| score(hits, &c.hits, s) > score(hits, &b.hits, s)) {
          b0 = Some(c);
        }
      }
      if let Some(b0) = b0 {
        let (a0, u0) = (b0.a, b0.u);
        let mut new = Vec::<SearchlightCandidateFit>::new();
        for &pid in triggered {
          if (pid <= TOF_INNER_MAX_PADDLE_ID) == outer {
            continue;
          }
          let (mut a, mut u) = (a0, u0);
          let mut found : Option<SearchlightCandidateFit> = None;
          let mut point = DVec3::ZERO;
          for _ in 0..s.trig_iter {
            let Some((pt, d)) = self.geometry.closest_on_paddle(pid, a, u) else {
              found = None;
              break;
            };
            point = pt;
            if d > s.trig_max_dist {
              found = None;
              break;
            }
            let Some(c2) = candidate(hits, (keep, n), kind, s, Some((pt, s.trig_err)), DVec3::ZERO, DVec3::ZERO) else {
              break;
            };
            (a, u) = (c2.a, c2.u);
            found = Some(c2);
          }
          if let Some(mut c) = found {
            // for the inner seed and an outer paddle, the line starts at the paddle
            (c.start, c.target) = if outer { (point, p[keep]) } else { (p[keep], point) };
            new.push(c);
          }
        }
        cands.extend(new);
      }
    }
    if cands.is_empty() {
      return res;
    }
    for c in cands.iter_mut() {
      // orient inwards: from the outer-most point of the candidate to the other one
      if (c.target - c.start).dot(c.u) < 0.0 {
        c.u = -c.u;
      }
      c.features = features(hits, c, s, v, quality.as_ref());
      c.prob = if s.weights.is_empty() {
        score(hits, &c.hits, s)
      } else {
        let x : f64 = s.weights.iter().zip(&c.features).map(|(w, f)| w*f).sum();
        1.0/(1.0 + (-x).exp())
      };
    }
    // the most probable among the allowed candidates (the first
    // candidate if none is allowed)
    let mut best : Option<usize> = None;
    for c in cands.iter_mut() {
      c.allowed = allowed.contains(&c.kind);
    }
    if !cands.iter().any(|c| c.allowed) {
      cands[0].allowed = true;
    }
    for (k, c) in cands.iter().enumerate() {
      if c.allowed && best.is_none_or(|b| c.prob > cands[b].prob) {
        best = Some(k);
      }
    }
    res.best = best;
    res.candidates = cands;
    res
  }
}

//---------------------------------------------------------

#[cfg(test)]
mod tests {
  use super::*;
  use std::collections::HashSet;

  /// a horizontal paddle along y, 180 x 16 cm, centered at (x, 0, z) (cm)
  fn paddle(pid : u8, x : f32, z : f32) -> TofPaddle {
    let mut p = TofPaddle::new();
    p.paddle_id = pid as i16;
    p.width     = 16.0;
    p.length    = 180.0;
    p.height    = 0.635;
    p.normal_z  = 1.0;
    p.global_pos_x_l0   = x;
    p.global_pos_z_l0   = z;
    p.global_pos_x_l0_A = x;
    p.global_pos_y_l0_A = -90.0;
    p.global_pos_z_l0_A = z;
    p.global_pos_x_l0_B = x;
    p.global_pos_y_l0_B = 90.0;
    p.global_pos_z_l0_B = z;
    p
  }

  /// an outer paddle (id 100) at z = 2 m, an inner one (id 10) at
  /// z = 1.1 m, and an outer one (id 101) far away
  fn geometry() -> SearchlightGeometry {
    SearchlightGeometry::new(&HashMap::from([(100u8, paddle(100, 0.0, 200.0)), (10, paddle(10, 0.0, 110.0)),
                                             (101, paddle(101, 100.0, 200.0))]))
  }

  /// a primary from (20, 30, 2000) along u, at beta 0.4, with an
  /// outer and an inner TOF hit and tracker hits every 100 mm in z,
  /// stopping at z = 400 mm
  fn primary_event(outer_late : bool) -> (SearchlightHits, DVec3) {
    let u  = DVec3::new(0.05, -0.08, -1.0).normalize();
    let a  = DVec3::new(20.0, 30.0, 2000.0);
    let at = |z : f64| a + ((z - a.z)/u.z)*u;
    let t  = |p : DVec3| (p - a).length()/(0.4*C_LIGHT_MM_NS);
    let (po, pi) = (at(2000.0), at(1100.0));
    // with outer_late, the outer hit is a secondary leaving the detector after the inner one
    let to = if outer_late { t(pi) + 5.0 } else { t(po) };
    let tof = vec![(po, to, 12.0, 100u8), (pi, t(pi), 12.0, 10u8)];
    let trk : Vec<(DVec3, f64, u32)> = [982.0, 882.0, 782.0, 682.0, 582.0, 482.0].iter().map(|&z| (at(z), 8.0, 0)).collect();
    (SearchlightHits::new(&tof, &trk, None), u)
  }

  fn angle(a : DVec3, b : DVec3) -> f64 {
    a.normalize().dot(b.normalize()).abs().min(1.0).acos().to_degrees()
  }

  #[test]
  fn searchlight_closest_on_paddle() {
    let geo = geometry();
    // a vertical line through (100, 500, *): closest point on paddle 100 (along y at x = 0)
    let (p, d) = geo.closest_on_paddle(100, DVec3::new(100.0, 500.0, 0.0), DVec3::Z).unwrap();
    assert!((p - DVec3::new(0.0, 500.0, 2000.0)).length() < 1e-9);
    assert!((d - 100.0).abs() < 1e-9);
    // beyond the end of the paddle, the point is limited to its length
    let (p, _) = geo.closest_on_paddle(100, DVec3::new(0.0, 2000.0, 0.0), DVec3::Z).unwrap();
    assert!((p.y - 900.0).abs() < 1e-9);
    assert!(geo.closest_on_paddle(55, DVec3::ZERO, DVec3::Z).is_none());
  }

  #[test]
  fn searchlight_vertex_from_lines() {
    let v   = DVec3::new(100.0, -50.0, 600.0);
    let s   = SearchlightVertexSettings::default();
    let dir = [DVec3::new(1.0, 0.2, 0.3), DVec3::new(-0.4, 1.0, -0.2), DVec3::new(0.1, -0.3, 1.0)];
    let hits = SearchlightHits::new(&[], &dir.iter().map(|d| (v + 200.0*d.normalize(), 1.0, 0)).collect::<Vec<_>>(), None);
    let lines : Vec<SearchlightLine> = dir.iter().enumerate().map(|(k, d)| {
      SearchlightLine { a : v + 300.0*d.normalize(), u : d.normalize(), hits : vec![k], sigma0 : 20.0, theta : 0.05 }
    }).collect();
    let (found, ncomp) = find_vertex(&lines, &[], &hits, &s).unwrap();
    assert!((found - v).length() < 1e-6);
    assert_eq!(ncomp, 3);
    // an outlier line does not move the vertex much
    let mut with_outlier = lines.clone();
    with_outlier.push(SearchlightLine { a : v + DVec3::new(0.0, 400.0, 0.0), u : DVec3::X, hits : vec![0], sigma0 : 20.0, theta : 0.05 });
    let (found, _) = find_vertex(&with_outlier, &[], &hits, &s).unwrap();
    assert!((found - v).length() < 1.0);
  }

  #[test]
  fn searchlight_tracker_line() {
    // hits along a line: one tracker line through their centroid along
    // their principal axis
    let u    = DVec3::new(0.3, -0.5, 0.8).normalize();
    let pts  : Vec<(DVec3, f64, u32)> = (0..6).map(|k| (DVec3::new(10.0, 20.0, 30.0) + (k as f64*50.0)*u, 1.0, 0)).collect();
    let hits = SearchlightHits::new(&[], &pts, None);
    let s    = SearchlightVertexSettings { trk_min_dz : 10.0, ..Default::default() };
    let lines = tracker_lines(&hits, &(0..pts.len()).collect::<Vec<_>>(), &s);
    assert_eq!(lines.len(), 1);
    assert!((lines[0].a - DVec3::new(10.0, 20.0, 30.0) - 125.0*u).length() < 1e-9);
    assert!(angle(lines[0].u, u) < 1e-6);
    assert_eq!(lines[0].hits.len(), 6);
  }

  #[test]
  fn searchlight_secondary_pairs() {
    // an inner hit and an outer hit 1 m away 3.34 ns later (beta 1), and
    // one 10 ns later (beta 0.33, not a secondary)
    let tof = vec![(DVec3::new(0.0, 0.0, 1000.0), 0.0, 2.0, 10u8),
                   (DVec3::new(0.0, 0.0, 2000.0), 1000.0/C_LIGHT_MM_NS, 2.0, 100u8),
                   (DVec3::new(0.0, 1000.0, 1000.0), 10.0, 2.0, 101u8)];
    let hits = SearchlightHits::new(&tof, &[], None);
    let s    = SearchlightVertexSettings::default();
    assert_eq!(secondary_pairs(&hits, &s, &[]), vec![(0, 1)]);
    assert!(secondary_pairs(&hits, &s, &[1]).is_empty());
  }

  #[test]
  fn searchlight_good_seed() {
    let (hits, u) = primary_event(false);
    let sl  = Searchlight::new(geometry(), SearchlightSettings::default());
    let res = sl.reconstruct_hits(&hits, &[]);
    assert_eq!(res.seed, Some((0, 1)));
    assert_eq!(res.flags, (false, false));
    let p = res.primary().unwrap();
    assert_eq!(p.kind, SearchlightCandidate::Today);
    assert!(angle(p.u, u) < 0.01);
    // inwards: the direction points down, along the flight
    assert!(p.u.z < 0.0);
    assert_eq!(p.features.len(), SEARCHLIGHT_N_FEATURES);
    // 2 TOF and 6 tracker hits, beta 0.4 is measured and physical
    assert_eq!((p.features[1], p.features[2], p.features[6], p.features[7]), (2.0, 6.0, 0.0, 0.0));
  }

  #[test]
  fn searchlight_flagged_outer_seed() {
    // the outer hit comes after the inner one: it is a secondary, the
    // primary is reconstructed from the inner seed and the tracker hits
    let (hits, u) = primary_event(true);
    let sl  = Searchlight::new(geometry(), SearchlightSettings::default());
    let res = sl.reconstruct_hits(&hits, &[]);
    assert_eq!(res.flags, (true, false));
    let p = res.primary().unwrap();
    assert!(matches!(p.kind, SearchlightCandidate::InnerVertex | SearchlightCandidate::InnerTracker | SearchlightCandidate::InnerTrigger));
    assert!(angle(p.u, u) < 0.5);
    // only the candidates which keep the inner seed hit take part
    assert!(p.allowed);
    for c in &res.candidates {
      assert_eq!(c.allowed, matches!(c.kind, SearchlightCandidate::InnerVertex | SearchlightCandidate::InnerTracker | SearchlightCandidate::InnerTrigger));
    }
    // without any weights, the most energy-weighted inliers win
    let mut s = SearchlightSettings::default();
    s.weights.clear();
    let res = Searchlight::new(geometry(), s).reconstruct_hits(&hits, &[]);
    assert!(res.primary().unwrap().kind != SearchlightCandidate::Today);
  }

  /// two secondaries from v, through the tracker layers (strips
  /// alternating between measuring x and y) and an outer TOF paddle.
  /// They must not be back to back: then one line fits both.
  fn secondaries(v : DVec3) -> (SearchlightHits, [DVec3; 2]) {
    let dirs = [DVec3::new(0.6, 0.0, 1.0).normalize(), DVec3::new(-0.3, 0.5, 1.0).normalize()];
    let mut trk = Vec::<(DVec3, f64, u32)>::new();
    for d in dirs {
      for z in [982.0, 882.0, 782.0, 682.0, 582.0, 482.0, 382.0] {
        let s = (z - v.z)/d.z;
        if s > 30.0 {
          trk.push((v + s*d, 0.5, 0));
        }
      }
    }
    let tof = dirs.iter().enumerate().map(|(k, d)| {
      let s = (if d.z > 0.0 { 2000.0 } else { 0.0 } - v.z)/d.z;
      (v + s*d, 10.0 + s/(0.95*C_LIGHT_MM_NS), 2.0, 100 + k as u8)
    }).collect::<Vec<_>>();
    let mut hits = SearchlightHits::new(&tof, &trk, None);
    for k in hits.ntof..hits.len() {
      hits.measured[k]   = if (hits.pos[k].z.round() as i64/100) % 2 == 0 { 0 } else { 1 };
      hits.half_chord[k] = 40.0;
    }
    (hits, dirs)
  }

  #[test]
  fn searchlight_track_finder() {
    let v = DVec3::new(50.0, -80.0, 400.0);
    let (hits, dirs) = secondaries(v);
    let free : Vec<usize> = (0..hits.len()).collect();
    let tracks = find_tracks(&hits, &free, &SearchlightTrackSettings::default(), &SearchlightVertexSettings::default(), &[]);
    assert_eq!(tracks.len(), 2);
    for t in &tracks {
      // each track has the hits of one secondary, and passes the vertex
      let d = dirs.iter().map(|d| angle(t.line.u, *d)).fold(f64::INFINITY, f64::min);
      assert!(d < 1e-3);
      assert!(t.line.dist(v) < 1e-3);
      assert!(t.ntrk >= 2 && t.ntof == 1);
    }
    let all : HashSet<usize> = tracks.iter().flat_map(|t| t.hits.iter().copied()).collect();
    assert_eq!(all.len(), hits.len());
  }

  #[test]
  fn searchlight_timing_score() {
    // the primary's last TOF hit at t = 0, 600 mm before the vertex; the
    // secondaries leave the vertex at t = 10 ns
    let v = DVec3::new(50.0, -80.0, 400.0);
    let (mut hits, _) = secondaries(v);
    hits = SearchlightHits {
      pos       : [vec![v + DVec3::new(0.0, 0.0, 600.0)], hits.pos].concat(),
      err       : [vec![SearchlightHits::ERR], hits.err].concat(),
      t         : [vec![0.0], hits.t].concat(),
      e         : [vec![12.0], hits.e].concat(),
      is_tof    : [vec![true], hits.is_tof].concat(),
      inner     : [vec![true], hits.inner].concat(),
      paddle_id : [vec![10], hits.paddle_id].concat(),
      measured  : [vec![-1], hits.measured].concat(),
      half_chord : [vec![0.0], hits.half_chord].concat(),
      ntof      : hits.ntof + 1,
    };
    let s = SearchlightTimingSettings::default();
    let (n, t0) = timing_score(v, &hits, 0, &HashSet::from([0]), &s);
    assert_eq!(n, 2);
    assert!((t0 - 10.0).abs() < 1e-9);
    // 200 mm away, the two estimates do not agree anymore
    let (n, _) = timing_score(v + DVec3::new(200.0, 0.0, 0.0), &hits, 0, &HashSet::from([0]), &s);
    assert!(n < 2);
  }

  #[test]
  fn searchlight_settings_phases() {
    let a = SearchlightSettings::phase_a();
    assert!(!a.independent_vertex && !a.use_tracks && !a.use_timing);
    assert_eq!(a.weights.len(), SEARCHLIGHT_N_FEATURES);
    assert!(a.weights[SEARCHLIGHT_N_FEATURES - 4..].iter().all(|w| *w == 0.0));
    let b = SearchlightSettings::default();
    assert!(b.independent_vertex && b.use_tracks && b.use_timing);
    // a good seed with tracker hits: the same primary in both
    let (hits, u) = primary_event(false);
    for s in [a, b] {
      let res = Searchlight::new(geometry(), s).reconstruct_hits(&hits, &[]);
      let p = res.primary().unwrap();
      assert_eq!(p.kind, SearchlightCandidate::Today);
      assert!(angle(p.u, u) < 0.01);
      assert_eq!(p.features.len(), SEARCHLIGHT_N_FEATURES);
    }
  }

  #[test]
  fn searchlight_no_seed() {
    let hits = SearchlightHits::new(&[(DVec3::new(0.0, 0.0, 2000.0), 0.0, 10.0, 100u8)], &[], None);
    let res  = Searchlight::new(geometry(), SearchlightSettings::default()).reconstruct_hits(&hits, &[]);
    assert!(res.seed.is_none() && res.primary().is_none());
  }
}
