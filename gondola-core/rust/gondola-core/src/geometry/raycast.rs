//! Ray casting through the solids of the geometry: for a
//! line origin + t*dir in the local frame of a solid, find
//! the ranges of t for which the line is inside the solid.
//!
//! * Box, tube : analytic
//! * Boolean   : the intervals of both operands are combined
//! * Tessellated : all crossings of the line with the mesh
//!   (watertight ray/triangle test, Woop et al., JCGT 2013)
//!   are classified as entering/leaving by the outward
//!   normal of the triangle, then turned into intervals.
//!   Crossings through shared edges and vertices are merged.
//!
//! If dir has unit length, t (and thus the length of the
//! intervals) is in mm.
// This file is part of gaps-online-software and published
// under the GPLv3 license

use std::f64::consts::TAU;

use glam::DVec3;

use crate::geometry::bvh::{
  Aabb,
  Bvh,
};
use crate::geometry::model::{
  BooleanOp,
  Geometry,
  Shape,
  SolidId,
};

/// Disjoint ranges [t0, t1] of the line parameter, sorted
/// in ascending order
pub type Intervals = Vec<(f64, f64)>;

/// Crossings of a mesh closer than this (in mm) are taken
/// as the same point (shared edges and vertices)
const CROSSING_TOLERANCE : f64 = 1e-7;

//------------------------------------------------------------

/// Bounding boxes and acceleration structures for all
/// solids of a geometry
#[derive(Debug, Clone, PartialEq)]
pub struct SolidRaycaster {
  /// bounding box of every solid, in its local frame
  bounds : Vec<Aabb>,
  /// BVH over the triangles of each tessellated solid
  bvhs   : Vec<Option<Bvh>>,
}

impl SolidRaycaster {
  pub fn new(geo : &Geometry) -> Self {
    let mut bounds = Vec::with_capacity(geo.solids.len());
    let mut bvhs   = Vec::with_capacity(geo.solids.len());
    for (k, solid) in geo.solids.iter().enumerate() {
      bounds.push(solid_bounds(geo, k));
      bvhs.push(match &solid.shape {
        Shape::Tessellated { vertices, triangles } => {
          let boxes : Vec<Aabb> = triangles.iter()
            .map(|tri| Aabb::from_points(tri.iter().map(|&i| vertices[i as usize])))
            .collect();
          Some(Bvh::build(&boxes))
        }
        _ => None,
      });
    }
    Self { bounds, bvhs }
  }

  /// Bounding box of a solid in its local frame
  pub fn bounds(&self, solid : SolidId) -> &Aabb {
    &self.bounds[solid]
  }

  /// The intervals of t in [tmin, tmax] (both finite) for
  /// which origin + t*dir is inside the solid.
  ///
  /// Returns None if the line crosses a tessellated solid
  /// an unequal number of times inwards and outwards, which
  /// happens for meshes which are not closed.
  pub fn intervals(&self, geo : &Geometry, solid : SolidId,
                   origin : DVec3, dir : DVec3,
                   tmin : f64, tmax : f64) -> Option<Intervals> {
    match &geo.solids[solid].shape {
      Shape::Box { half_lengths } => {
        let aabb = Aabb { min : -*half_lengths, max : *half_lengths };
        Some(aabb.clip_line(origin, dir, tmin, tmax).into_iter().collect())
      }
      Shape::Tube { rmin, rmax, half_z, start_phi, delta_phi } => {
        Some(tube_intervals(*rmin, *rmax, *half_z, *start_phi, *delta_phi, origin, dir, tmin, tmax))
      }
      Shape::Tessellated { vertices, triangles } => {
        let bvh = self.bvhs[solid].as_ref().expect("Tessellated solid without BVH");
        mesh_intervals(vertices, triangles, bvh, origin, dir, tmin, tmax)
      }
      Shape::Boolean { op, first, second, second_transform } => {
        let a = self.intervals(geo, *first, origin, dir, tmin, tmax)?;
        if a.is_empty() && *op != BooleanOp::Union {
          return Some(a);
        }
        let b = self.intervals(geo, *second,
                               second_transform.inverse_apply_point(origin),
                               second_transform.inverse_apply_vector(dir),
                               tmin, tmax)?;
        Some(match op {
          BooleanOp::Union        => combine(&a, &b, |x, y| x || y),
          BooleanOp::Intersection => combine(&a, &b, |x, y| x && y),
          BooleanOp::Subtraction  => combine(&a, &b, |x, y| x && !y),
        })
      }
    }
  }
}

/// Bounding box of a solid in its local frame
fn solid_bounds(geo : &Geometry, solid : SolidId) -> Aabb {
  match &geo.solids[solid].shape {
    Shape::Tessellated { vertices, .. } => Aabb::from_points(vertices.iter().copied()),
    Shape::Box { half_lengths } => Aabb { min : -*half_lengths, max : *half_lengths },
    Shape::Tube { rmax, half_z, .. } => Aabb {
      min : DVec3::new(-rmax, -rmax, -half_z),
      max : DVec3::new( *rmax, *rmax,  *half_z),
    },
    Shape::Boolean { op, first, second, second_transform } => {
      let a = solid_bounds(geo, *first);
      let b = solid_bounds(geo, *second).transformed(second_transform);
      match op {
        BooleanOp::Union        => { let mut u = a; u.merge(&b); u }
        BooleanOp::Intersection => a.intersection(&b),
        BooleanOp::Subtraction  => a,
      }
    }
  }
}

//------------------------------------------------------------

/// Combine two sets of intervals, keeping the parts for
/// which keep(in a, in b) is true
fn combine(a : &[(f64, f64)], b : &[(f64, f64)], keep : impl Fn(bool, bool) -> bool) -> Intervals {
  let mut edges : Vec<f64> = a.iter().chain(b).flat_map(|&(t0, t1)| [t0, t1]).collect();
  edges.sort_by(f64::total_cmp);
  edges.dedup();
  let contains = |iv : &[(f64, f64)], t : f64| iv.iter().any(|&(t0, t1)| t0 <= t && t <= t1);
  let mut out = Intervals::new();
  for w in edges.windows(2) {
    let t = 0.5*(w[0] + w[1]);
    if keep(contains(a, t), contains(b, t)) {
      push_merged(&mut out, w[0], w[1]);
    }
  }
  out
}

/// Append [t0, t1] to sorted intervals, merging it with
/// the last one if they touch
fn push_merged(iv : &mut Intervals, t0 : f64, t1 : f64) {
  if let Some(last) = iv.last_mut() {
    if last.1 >= t0 {
      last.1 = last.1.max(t1);
      return;
    }
  }
  iv.push((t0, t1));
}

//------------------------------------------------------------

fn tube_intervals(rmin : f64, rmax : f64, half_z : f64, start_phi : f64, delta_phi : f64,
                  o : DVec3, d : DVec3, tmin : f64, tmax : f64) -> Intervals {
  let slab = Aabb {
    min : DVec3::new(-rmax, -rmax, -half_z),
    max : DVec3::new( rmax,  rmax,  half_z),
  };
  let Some((t0, t1)) = slab.clip_line(o, d, tmin, tmax) else {
    return Intervals::new();
  };
  let mut iv = disk_interval(rmax, o, d, t0, t1);
  if rmin > 0.0 && !iv.is_empty() {
    iv = combine(&iv, &disk_interval(rmin, o, d, t0, t1), |x, y| x && !y);
  }
  if delta_phi < TAU && !iv.is_empty() {
    iv = combine(&iv, &wedge_intervals(start_phi, delta_phi, o, d, t0, t1), |x, y| x && y);
  }
  iv
}

/// Where the line is within radius r of the z axis
fn disk_interval(r : f64, o : DVec3, d : DVec3, t0 : f64, t1 : f64) -> Intervals {
  let a = d.x*d.x + d.y*d.y;
  let b = o.x*d.x + o.y*d.y;
  let c = o.x*o.x + o.y*o.y - r*r;
  if a == 0.0 {
    return if c <= 0.0 { vec![(t0, t1)] } else { Intervals::new() };
  }
  let disc = b*b - a*c;
  if disc < 0.0 {
    return Intervals::new();
  }
  let s = disc.sqrt();
  let (ta, tb) = (((-b - s)/a).max(t0), ((-b + s)/a).min(t1));
  if ta < tb { vec![(ta, tb)] } else { Intervals::new() }
}

/// Where the azimuth of the line is in
/// [start_phi, start_phi + delta_phi]
fn wedge_intervals(start_phi : f64, delta_phi : f64, o : DVec3, d : DVec3, t0 : f64, t1 : f64) -> Intervals {
  let inside = |t : f64| {
    let p = o + t*d;
    (p.y.atan2(p.x) - start_phi).rem_euclid(TAU) <= delta_phi
  };
  // the azimuth can only enter or leave the wedge where
  // the line crosses one of the two planes through the z
  // axis which contain the edges of the wedge
  let mut ts = vec![t0, t1];
  for phi in [start_phi, start_phi + delta_phi] {
    let n  = DVec3::new(-phi.sin(), phi.cos(), 0.0);
    let dn = d.dot(n);
    if dn != 0.0 {
      let t = -o.dot(n)/dn;
      if t > t0 && t < t1 {
        ts.push(t);
      }
    }
  }
  ts.sort_by(f64::total_cmp);
  let mut iv = Intervals::new();
  for w in ts.windows(2) {
    if w[1] > w[0] && inside(0.5*(w[0] + w[1])) {
      push_merged(&mut iv, w[0], w[1]);
    }
  }
  iv
}

//------------------------------------------------------------

/// A line prepared for the watertight ray/triangle test
/// (S. Woop, C. Benthin, I. Wald, "Watertight Ray/Triangle
/// Intersection", JCGT 2(1), 2013)
struct WatertightRay {
  origin : DVec3,
  dir    : DVec3,
  kx     : usize,
  ky     : usize,
  kz     : usize,
  sx     : f64,
  sy     : f64,
  sz     : f64,
}

impl WatertightRay {
  fn new(origin : DVec3, dir : DVec3) -> Self {
    let a  = dir.abs();
    let kz = if a.x >= a.y && a.x >= a.z { 0 } else if a.y >= a.z { 1 } else { 2 };
    let mut kx = (kz + 1) % 3;
    let mut ky = (kx + 1) % 3;
    // keep the winding of the triangles
    if dir[kz] < 0.0 {
      std::mem::swap(&mut kx, &mut ky);
    }
    Self {
      origin,
      dir,
      kx,
      ky,
      kz,
      sx : dir[kx]/dir[kz],
      sy : dir[ky]/dir[kz],
      sz : 1.0/dir[kz],
    }
  }

  /// The line parameter t where the line crosses the
  /// triangle and +1 if it enters the solid there (against
  /// the outward normal), -1 if it leaves
  fn intersect(&self, v0 : DVec3, v1 : DVec3, v2 : DVec3) -> Option<(f64, i32)> {
    let (kx, ky, kz) = (self.kx, self.ky, self.kz);
    let (a, b, c) = (v0 - self.origin, v1 - self.origin, v2 - self.origin);
    let (ax, ay) = (a[kx] - self.sx*a[kz], a[ky] - self.sy*a[kz]);
    let (bx, by) = (b[kx] - self.sx*b[kz], b[ky] - self.sy*b[kz]);
    let (cx, cy) = (c[kx] - self.sx*c[kz], c[ky] - self.sy*c[kz]);
    let u = cx*by - cy*bx;
    let v = ax*cy - ay*cx;
    let w = bx*ay - by*ax;
    if (u < 0.0 || v < 0.0 || w < 0.0) && (u > 0.0 || v > 0.0 || w > 0.0) {
      return None;
    }
    let det = u + v + w;
    if det == 0.0 {
      return None;
    }
    let t = (u*a[kz] + v*b[kz] + w*c[kz])*self.sz/det;
    let ndir = (v1 - v0).cross(v2 - v0).dot(self.dir);
    if ndir == 0.0 {
      return None;
    }
    Some((t, if ndir < 0.0 { 1 } else { -1 }))
  }
}

fn mesh_intervals(vertices : &[DVec3], triangles : &[[u32; 3]], bvh : &Bvh,
                  origin : DVec3, dir : DVec3, tmin : f64, tmax : f64) -> Option<Intervals> {
  // all crossings along the line through the bounding
  // box are needed to know if tmin is inside or not
  let Some((ta, tb)) = bvh.bounds().expanded(1e-6).clip_line(origin, dir, f64::NEG_INFINITY, f64::INFINITY) else {
    return Some(Intervals::new());
  };
  let ray = WatertightRay::new(origin, dir);
  let mut crossings = Vec::new();
  bvh.traverse(origin, dir, ta, tb, |i| {
    let [a, b, c] = triangles[i as usize];
    if let Some(crossing) = ray.intersect(vertices[a as usize], vertices[b as usize], vertices[c as usize]) {
      crossings.push(crossing);
    }
  });
  crossings.sort_by(|x, y| x.0.total_cmp(&y.0));

  let mut iv      = Intervals::new();
  let mut winding = 0;
  let mut start   = 0.0;
  let mut k       = 0;
  while k < crossings.len() {
    // a line through an edge or a vertex crosses several
    // triangles at the same point, which counts once.
    // Entering and leaving at the same point (grazing an
    // edge) cancels.
    let t = crossings[k].0;
    let mut net = 0;
    while k < crossings.len() && crossings[k].0 - t <= CROSSING_TOLERANCE {
      net += crossings[k].1;
      k   += 1;
    }
    let before = winding;
    winding += net.signum();
    if winding < 0 {
      return None;
    }
    if before == 0 && winding > 0 {
      start = t;
    } else if before > 0 && winding == 0 {
      let (t0, t1) = (start.max(tmin), t.min(tmax));
      if t0 < t1 {
        push_merged(&mut iv, t0, t1);
      }
    }
  }
  if winding != 0 {
    return None;
  }
  Some(iv)
}

//------------------------------------------------------------

#[cfg(test)]
mod tests {
  use super::*;
  use std::f64::consts::FRAC_PI_2;
  use crate::geometry::model::{
    Material,
    Solid,
    Transform,
    Volume,
  };

  /// A geometry with the given solids, the first one is
  /// used as the world
  fn geometry(shapes : Vec<Shape>) -> Geometry {
    let solids = shapes.into_iter().enumerate()
      .map(|(k, shape)| Solid { name : format!("s{k}"), shape })
      .collect();
    let world = Volume {
      name      : String::from("world"),
      material  : 0,
      solid     : 0,
      daughters : Vec::new(),
      auxiliary : Vec::new(),
    };
    Geometry::new(vec![Material { name : String::from("vacuum"), density : 0.0 }], solids, vec![world], 0).unwrap()
  }

  fn assert_intervals(iv : Option<Intervals>, expected : &[(f64, f64)]) {
    let iv = iv.expect("line crosses the solid inconsistently");
    assert_eq!(iv.len(), expected.len(), "{iv:?} != {expected:?}");
    for (a, b) in iv.iter().zip(expected) {
      assert!((a.0 - b.0).abs() < 1e-9 && (a.1 - b.1).abs() < 1e-9, "{iv:?} != {expected:?}");
    }
  }

  /// Cube [-1, 1]^3 as a mesh, with outward normals
  fn cube() -> Shape {
    let vertices : Vec<DVec3> = (0..8).map(|k| DVec3::new(
      if k & 1 == 0 { -1.0 } else { 1.0 },
      if k & 2 == 0 { -1.0 } else { 1.0 },
      if k & 4 == 0 { -1.0 } else { 1.0 },
    )).collect();
    let quads = [[0, 2, 6, 4], [1, 3, 7, 5], [0, 1, 5, 4], [2, 3, 7, 6], [0, 1, 3, 2], [4, 5, 7, 6]];
    let mut triangles = Vec::new();
    for q in quads {
      for mut tri in [[q[0], q[1], q[2]], [q[0], q[2], q[3]]] {
        let [a, b, c] = tri.map(|i : u32| vertices[i as usize]);
        if (b - a).cross(c - a).dot(a + b + c) < 0.0 {
          tri.swap(1, 2);
        }
        triangles.push(tri);
      }
    }
    Shape::Tessellated { vertices, triangles }
  }

  #[test]
  fn box_and_tube() {
    let geo = geometry(vec![
      Shape::Box  { half_lengths : DVec3::new(1.0, 2.0, 3.0) },
      Shape::Tube { rmin : 1.0, rmax : 2.0, half_z : 1.0, start_phi : 0.0, delta_phi : TAU },
      Shape::Tube { rmin : 0.0, rmax : 2.0, half_z : 1.0, start_phi : 0.0, delta_phi : FRAC_PI_2 },
    ]);
    let rc = SolidRaycaster::new(&geo);
    let x0 = DVec3::new(-5.0, 0.0, 0.0);
    assert_intervals(rc.intervals(&geo, 0, x0, DVec3::X, -10.0, 10.0), &[(4.0, 6.0)]);
    assert_intervals(rc.intervals(&geo, 0, x0, DVec3::X, 0.0, 5.0), &[(4.0, 5.0)]);
    assert_intervals(rc.intervals(&geo, 0, DVec3::new(0.0, 2.5, 0.0), DVec3::X, -10.0, 10.0), &[]);
    // through the hole of the tube
    assert_intervals(rc.intervals(&geo, 1, x0, DVec3::X, -10.0, 10.0), &[(3.0, 4.0), (6.0, 7.0)]);
    assert_intervals(rc.intervals(&geo, 1, DVec3::ZERO, DVec3::Z, -10.0, 10.0), &[]);
    assert_intervals(rc.intervals(&geo, 1, DVec3::new(1.5, 0.0, 0.0), DVec3::Z, -10.0, 10.0), &[(-1.0, 1.0)]);
    // quarter tube in the first quadrant
    let xq = (4.0f64 - 0.25).sqrt();
    assert_intervals(rc.intervals(&geo, 2, DVec3::new(-5.0, 0.5, 0.0), DVec3::X, -10.0, 10.0), &[(5.0, 5.0 + xq)]);
    assert_intervals(rc.intervals(&geo, 2, DVec3::new(-5.0, -0.5, 0.0), DVec3::X, -10.0, 10.0), &[]);
  }

  #[test]
  fn booleans() {
    let shift = Transform { rotation : glam::DMat3::IDENTITY, translation : DVec3::new(1.0, 0.0, 0.0) };
    let geo = geometry(vec![
      Shape::Box { half_lengths : DVec3::splat(2.0) },
      Shape::Box { half_lengths : DVec3::splat(1.0) },
      Shape::Boolean { op : BooleanOp::Subtraction,  first : 0, second : 1, second_transform : Transform::IDENTITY },
      Shape::Boolean { op : BooleanOp::Intersection, first : 0, second : 1, second_transform : shift },
      Shape::Boolean { op : BooleanOp::Union,        first : 1, second : 1, second_transform : shift },
    ]);
    let rc = SolidRaycaster::new(&geo);
    let x0 = DVec3::new(-5.0, 0.0, 0.0);
    assert_intervals(rc.intervals(&geo, 2, x0, DVec3::X, -10.0, 10.0), &[(3.0, 4.0), (6.0, 7.0)]);
    assert_intervals(rc.intervals(&geo, 3, x0, DVec3::X, -10.0, 10.0), &[(5.0, 7.0)]);
    assert_intervals(rc.intervals(&geo, 4, x0, DVec3::X, -10.0, 10.0), &[(4.0, 7.0)]);
    assert_eq!(*rc.bounds(3), Aabb { min : DVec3::new(0.0, -1.0, -1.0), max : DVec3::new(2.0, 1.0, 1.0) });
  }

  #[test]
  fn mesh_edges_and_vertices() {
    let geo = geometry(vec![cube()]);
    let rc  = SolidRaycaster::new(&geo);
    // through the faces
    assert_intervals(rc.intervals(&geo, 0, DVec3::new(-5.0, 0.3, 0.2), DVec3::X, -10.0, 10.0), &[(4.0, 6.0)]);
    // through the diagonal edge of two triangles of a face
    assert_intervals(rc.intervals(&geo, 0, DVec3::new(0.0, 0.0, -5.0), DVec3::Z, -10.0, 10.0), &[(4.0, 6.0)]);
    // through edges of the cube
    let d = DVec3::new(1.0, 1.0, 0.0).normalize();
    let s = 2.0f64.sqrt();
    assert_intervals(rc.intervals(&geo, 0, DVec3::new(-2.0, -2.0, 0.3), d, -10.0, 10.0), &[(s, 3.0*s)]);
    // through two opposite vertices
    let d = DVec3::ONE.normalize();
    let s = 3.0f64.sqrt();
    assert_intervals(rc.intervals(&geo, 0, DVec3::splat(-2.0), d, -10.0, 10.0), &[(s, 3.0*s)]);
    // starting inside
    assert_intervals(rc.intervals(&geo, 0, DVec3::ZERO, DVec3::X, 0.0, 10.0), &[(0.0, 1.0)]);
    // missing
    assert_intervals(rc.intervals(&geo, 0, DVec3::new(-5.0, 1.5, 0.0), DVec3::X, -10.0, 10.0), &[]);
  }

  #[test]
  fn open_mesh_is_detected() {
    let Shape::Tessellated { vertices, mut triangles } = cube() else { unreachable!() };
    // remove the two triangles of the face at x = +1
    triangles.retain(|tri| !tri.iter().all(|&i| vertices[i as usize].x > 0.0));
    let geo = geometry(vec![Shape::Tessellated { vertices, triangles }]);
    let rc  = SolidRaycaster::new(&geo);
    assert!(rc.intervals(&geo, 0, DVec3::new(-5.0, 0.3, 0.2), DVec3::X, -10.0, 10.0).is_none());
    // lines which do not pass the hole are fine
    assert_intervals(rc.intervals(&geo, 0, DVec3::new(0.3, -5.0, 0.2), DVec3::Y, -10.0, 10.0), &[(4.0, 6.0)]);
  }

  #[test]
  fn mesh_volume() {
    // integrating the path length over a grid of parallel
    // lines gives the volume of the solid
    let vertices  = vec![DVec3::ZERO, DVec3::new(3.0, 0.0, 0.0), DVec3::new(0.0, 2.0, 0.0), DVec3::new(0.0, 0.0, 1.0)];
    let triangles = vec![[0, 2, 1], [0, 1, 3], [0, 3, 2], [1, 2, 3]];
    let geo = geometry(vec![Shape::Tessellated { vertices, triangles }]);
    let rc  = SolidRaycaster::new(&geo);
    let n   = 500;
    let dir = DVec3::new(0.1, 0.2, 1.0).normalize();
    let mut volume = 0.0;
    for i in 0..n {
      for j in 0..n {
        // lines start on a plane perpendicular to dir
        let (u, v) = (-5.0 + 10.0*(i as f64 + 0.5)/n as f64, -5.0 + 10.0*(j as f64 + 0.5)/n as f64);
        let e1 = dir.any_orthonormal_vector();
        let e2 = dir.cross(e1);
        let o  = u*e1 + v*e2 - 10.0*dir;
        let iv = rc.intervals(&geo, 0, o, dir, -100.0, 100.0).unwrap();
        volume += iv.iter().map(|(t0, t1)| t1 - t0).sum::<f64>()*(10.0/n as f64).powi(2);
      }
    }
    assert!((volume - 1.0).abs() < 0.01, "volume {volume}");
  }
}
