//! Checks the ray casting through every tessellated solid of
//! a real GDML geometry: the path length integrated over a
//! grid of parallel lines has to give the volume enclosed by
//! the mesh.
//!
//! It needs the GDML file and is therefore ignored by default:
//!
//!   GAPS_GDML=/path/to/geometry.v26.08.gdml \
//!   GAPS_GDML_CACHE=/path/to/geometry.v26.08.bin \
//!   GAPS_VOLUME_CHECK_N=256 \
//!   cargo test --release --test geometry_volume_check -- --ignored --nocapture
//!
//! GAPS_GDML_CACHE (optional) is a binary cache of the parsed
//! geometry, GAPS_VOLUME_CHECK_N (default 64) the number of
//! lines per direction for every solid. With N = 256 the
//! median relative difference for geometry v26.08 is 6e-5.
//! Long thin solids need a finer grid.
// This file is part of gaps-online-software and published
// under the GPLv3 license

use std::path::PathBuf;
use std::time::Instant;

use glam::DVec3;

use gondola_core::geometry::{
  Geometry,
  Shape,
  SolidRaycaster,
};

#[test]
#[ignore]
fn geometry_volume_check() {
  let gdml  = PathBuf::from(std::env::var("GAPS_GDML").expect("Set GAPS_GDML to the GDML file"));
  let cache = std::env::var("GAPS_GDML_CACHE").ok().map(PathBuf::from);
  let n : usize = std::env::var("GAPS_VOLUME_CHECK_N").map(|s| s.parse().unwrap()).unwrap_or(64);
  let geo = Geometry::load(&gdml, cache.as_deref()).unwrap();

  let start = Instant::now();
  let rc = SolidRaycaster::new(&geo);
  println!("Raycaster built in {:.2} s", start.elapsed().as_secs_f64());

  let start = Instant::now();
  let (mut n_rays, mut n_rejected) = (0usize, 0usize);
  let mut errors : Vec<(f64, String, f64, f64)> = Vec::new();
  for (k, solid) in geo.solids.iter().enumerate() {
    let Shape::Tessellated { vertices, triangles } = &solid.shape else { continue };
    let volume : f64 = triangles.iter().map(|t| {
      let (a, b, c) = (vertices[t[0] as usize], vertices[t[1] as usize], vertices[t[2] as usize]);
      a.dot(b.cross(c))/6.0
    }).sum();
    // a different (pseudo random) direction for every solid
    let phi       = (k as f64*2.399963).rem_euclid(std::f64::consts::TAU);
    let cos_theta = ((k as f64*0.618034).fract()*2.0 - 1.0).clamp(-0.99, 0.99);
    let sin_theta = (1.0 - cos_theta*cos_theta).sqrt();
    let dir = DVec3::new(sin_theta*phi.cos(), sin_theta*phi.sin(), cos_theta);
    let e1  = dir.any_orthonormal_vector();
    let e2  = dir.cross(e1);
    // the lines start on a square perpendicular to dir which
    // covers the bounding sphere of the solid
    let bounds = rc.bounds(k);
    let center = bounds.center();
    let r = 0.5*(bounds.max - bounds.min).length();
    let h = 2.0*r/n as f64;
    let mut raycast  = 0.0;
    let mut rejected = 0;
    for i in 0..n {
      for j in 0..n {
        // jitter within the grid cell
        let ju = ((i*7919 + j*104729 + k) as f64*0.754877).fract();
        let jv = ((i*15485863 + j*32452843 + k) as f64*0.569840).fract();
        let origin = center + (-r + (i as f64 + ju)*h)*e1 + (-r + (j as f64 + jv)*h)*e2 - 2.0*r*dir;
        n_rays += 1;
        match rc.intervals(&geo, k, origin, dir, 0.0, 4.0*r) {
          Some(iv) => raycast += iv.iter().map(|(t0, t1)| t1 - t0).sum::<f64>()*h*h,
          None     => rejected += 1,
        }
      }
    }
    n_rejected += rejected;
    if rejected > 0 {
      println!("{} : {} of {} lines rejected", solid.name, rejected, n*n);
    } else {
      errors.push((((raycast - volume)/volume).abs(), solid.name.clone(), volume, raycast));
    }
  }
  let dt = start.elapsed().as_secs_f64();
  println!("{} lines in {:.2} s ({:.2} us per line), {} rejected", n_rays, dt, 1e6*dt/n_rays as f64, n_rejected);

  errors.sort_by(|a, b| a.0.total_cmp(&b.0));
  let quantile = |q : f64| errors[((errors.len() - 1) as f64*q) as usize].0;
  println!("|relative volume difference| of {} solids: median {:.2e}, 90% {:.2e}, 99% {:.2e}, max {:.2e}",
           errors.len(), quantile(0.5), quantile(0.9), quantile(0.99), quantile(1.0));
  for (e, name, volume, raycast) in errors.iter().rev().take(8) {
    println!("  {e:.3e} {name} : mesh {volume:.1} mm3, raycast {raycast:.1} mm3");
  }
  // loose enough for the default grid, any real problem with
  // the ray casting shows up as a much larger difference
  assert!(quantile(0.5) < 1e-2);
}
