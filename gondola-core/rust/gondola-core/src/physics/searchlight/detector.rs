//! The detector geometry searchlight needs: TOF paddles and their
//! overlaps
// This file is part of gaps-online-software and published
// under the GPLv3 license

use crate::prelude::*;

use glam::DVec3;

/// A TOF paddle, in mm
#[derive(Debug, Copy, Clone, PartialEq)]
pub struct SearchlightPaddle {
  pub center    : DVec3,
  /// unit vector along the paddle
  pub principal : DVec3,
  /// unit vector along the thickness of the paddle
  pub normal    : DVec3,
  pub width     : f64,
  pub length    : f64,
}

/// A tracker strip: which coordinate it measures, and the half
/// length of the strip (mm)
#[derive(Debug, Copy, Clone, PartialEq)]
pub struct SearchlightStrip {
  /// 0: x, 1: y
  pub measured   : u8,
  pub half_chord : f64,
}

/// The detector geometry for searchlight, built once from the
/// database
#[derive(Debug, Clone, Default)]
#[cfg_attr(feature="pybindings", pyclass)]
pub struct SearchlightGeometry {
  pub paddles  : HashMap<u8, SearchlightPaddle>,
  pub overlaps : TofPaddleOverlaps,
  /// strip id -> strip, needed for the secondary tracks
  pub strips   : HashMap<u32, SearchlightStrip>,
}

impl SearchlightGeometry {
  /// # Arguments:
  ///   * paddles : paddle id -> paddle, as from the database
  pub fn new(paddles : &HashMap<u8, TofPaddle>) -> Self {
    let vec = |v : (f32, f32, f32)| DVec3::new(v.0 as f64, v.1 as f64, v.2 as f64);
    let mut geo = HashMap::<u8, SearchlightPaddle>::new();
    for (&pid, p) in paddles {
      // the database has positions and sizes in cm
      geo.insert(pid, SearchlightPaddle {
        center    : 10.0*vec(p.center_pos()),
        principal : vec(p.principal()),
        normal    : vec(p.normal()),
        width     : 10.0*p.width as f64,
        length    : 10.0*p.length as f64,
      });
    }
    Self {
      paddles  : geo,
      overlaps : TofPaddleOverlaps::new(paddles),
      strips   : HashMap::new(),
    }
  }

  /// Radius of the Si(Li) detectors (mm)
  pub const R_DISK : f64 = 50.0;

  /// Add the tracker strips: a strip measures the coordinate in
  /// which its centre is offset from the centre of its detector,
  /// and its length is the chord of the detector disk there.
  ///
  /// # Arguments:
  ///   * strips : strip id -> strip, as from the database
  pub fn with_strips(mut self, strips : &HashMap<u32, TrackerStrip>) -> Self {
    for (&sid, st) in strips {
      // the database has positions in cm
      let ox = 10.0*st.global_pos_x_l0 as f64 - 10.0*st.global_pos_x_det_l0 as f64;
      let oy = 10.0*st.global_pos_y_l0 as f64 - 10.0*st.global_pos_y_det_l0 as f64;
      let (measured, o) = if ox.abs() > oy.abs() { (0u8, ox) } else { (1u8, oy) };
      let half_chord = (Self::R_DISK*Self::R_DISK - o*o).max(100.0).sqrt();
      self.strips.insert(sid, SearchlightStrip { measured, half_chord });
    }
    self
  }

  /// The point on the axis of the paddle which is closest to
  /// the line through a along u (unit vector), limited to the
  /// length of the paddle, and its distance from the line.
  /// None if the paddle is unknown.
  pub fn closest_on_paddle(&self, paddle_id : u8, a : DVec3, u : DVec3) -> Option<(DVec3, f64)> {
    let p  = self.paddles.get(&paddle_id)?;
    let h  = 0.5*p.length;
    let w  = p.center - a;
    let b  = p.principal.dot(u);
    let d  = 1.0 - b*b;
    let s  = if d < 1e-9 { 0.0 } else { (b*u.dot(w) - p.principal.dot(w))/d };
    let pt = p.center + s.clamp(-h, h)*p.principal;
    let r  = pt - a;
    Some((pt, (r - r.dot(u)*u).length()))
  }
}
