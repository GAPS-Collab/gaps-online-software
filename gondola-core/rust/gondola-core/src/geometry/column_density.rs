//! Column density along a straight track through the
//! geometry, by ray casting the placed volumes.
//!
//! The volumes crossed by a segment A -> B are found with a
//! BVH over the world bounding boxes of all placements. For
//! each of them, the path length inside it is calculated
//! exactly (see raycast). Since a daughter volume replaces
//! the material of its mother, the column density is
//!
//!   X = rho_world * L_world + sum_v (rho_v - rho_mother(v)) * L_v
//!
//! with L_world the path length inside the world volume and
//! L_v the one inside volume v. Lengths are in mm, densities
//! in g/cm3 and column densities in g/cm2.
// This file is part of gaps-online-software and published
// under the GPLv3 license

use crate::prelude::*;

use std::sync::OnceLock;

use glam::DVec3;

use crate::geometry::bvh::{
  Aabb,
  Bvh,
};
use crate::geometry::model::{
  Geometry,
  VolumeId,
  WorldPlacement,
};
use crate::geometry::raycast::SolidRaycaster;
use crate::physics::Tracklet;

/// Crossings of the same volume closer than this (in mm)
/// do not count as overlaps
const OVERLAP_TOLERANCE : f64 = 1e-6;

//------------------------------------------------------------

/// Acceleration structures to cast rays through the placed
/// volumes of a geometry
#[derive(Debug, Clone)]
pub struct Raycaster {
  solids     : SolidRaycaster,
  /// all placements, the world is the first one
  placements : Vec<WorldPlacement>,
  /// world bounding box of every placement
  bounds     : Vec<Aabb>,
  /// BVH over the placements except the world. Its
  /// primitive i is the placement i + 1.
  bvh        : Bvh,
}

impl Raycaster {
  pub fn new(geo : &Geometry) -> Self {
    let solids     = SolidRaycaster::new(geo);
    let placements = geo.world_placements();
    let bounds : Vec<Aabb> = placements.iter()
      .map(|p| solids.bounds(geo.volumes[p.volume].solid).transformed(&p.transform))
      .collect();
    let bvh = Bvh::build(&bounds[1..]);
    Self { solids, placements, bounds, bvh }
  }

  /// The parts of the segment a -> b inside each volume
  pub fn trace(&self, geo : &Geometry, a : DVec3, b : DVec3) -> TrackPath {
    let length = (b - a).length();
    let mut path = TrackPath {
      length,
      crossings : Vec::new(),
      rejected  : Vec::new(),
    };
    if length == 0.0 {
      return path;
    }
    let dir = (b - a)/length;
    let mut add = |k : usize, placement : &WorldPlacement| {
      if self.bounds[k].clip_line(a, dir, 0.0, length).is_none() {
        return;
      }
      let t = &placement.transform;
      let solid = geo.volumes[placement.volume].solid;
      match self.solids.intervals(geo, solid, t.inverse_apply_point(a), t.inverse_apply_vector(dir), 0.0, length) {
        Some(iv) => {
          for (t0, t1) in iv {
            path.crossings.push(Crossing {
              copynumber : placement.copynumber,
              volume     : placement.volume,
              mother     : placement.mother,
              start      : t0,
              stop       : t1,
              density    : geo.density(placement.volume),
            });
          }
        }
        None => path.rejected.push(placement.copynumber),
      }
    };
    add(0, &self.placements[0]);
    self.bvh.traverse(a, dir, 0.0, length, |i| {
      let k = i as usize + 1;
      add(k, &self.placements[k]);
    });
    path.crossings.sort_by(|x, y| x.start.total_cmp(&y.start).then(x.stop.total_cmp(&y.stop)));
    path
  }
}

/// The raycaster of a geometry is built when it is first
/// needed. It does not take part in comparisons.
#[derive(Debug, Clone, Default)]
pub struct LazyRaycaster(OnceLock<Raycaster>);

impl PartialEq for LazyRaycaster {
  fn eq(&self, _other : &Self) -> bool {
    true
  }
}

//------------------------------------------------------------

/// The part of a track inside one placed volume
#[derive(Debug, Clone, PartialEq)]
#[cfg_attr(feature="pybindings", pyclass)]
pub struct Crossing {
  /// Geant4 copy number of the placement, None for the world
  pub copynumber : Option<i64>,
  /// the placed logical volume (index in Geometry.volumes)
  pub volume     : VolumeId,
  /// the logical volume it is placed in, None for the world
  pub mother     : Option<VolumeId>,
  /// distance from the start of the track where it enters
  /// the volume in mm
  pub start      : f64,
  /// distance from the start of the track where it leaves
  /// the volume in mm
  pub stop       : f64,
  /// density of the material of the volume in g/cm3
  pub density    : f64,
}

impl Crossing {
  /// Path length inside the volume in mm
  pub fn length(&self) -> f64 {
    self.stop - self.start
  }

  /// Column density inside the volume in g/cm2
  pub fn column_density(&self) -> f64 {
    self.density*self.length()/10.0
  }
}

impl fmt::Display for Crossing {
  fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
    write!(f, "<Crossing volume {} (copynumber {:?}) from {:.3} to {:.3} mm, {:.4} g/cm2>",
           self.volume, self.copynumber, self.start, self.stop, self.column_density())
  }
}

#[cfg(feature="pybindings")]
#[pymethods]
impl Crossing {
  /// Geant4 copy number of the placement, None for the world
  #[getter]
  fn get_copynumber(&self) -> Option<i64> {
    self.copynumber
  }

  /// The placed logical volume (index in Geometry.volumes)
  #[getter]
  fn get_volume(&self) -> VolumeId {
    self.volume
  }

  /// The logical volume it is placed in (index in
  /// Geometry.volumes), None for the world
  #[getter]
  fn get_mother(&self) -> Option<VolumeId> {
    self.mother
  }

  /// Distance from the start of the track where it enters
  /// the volume in mm
  #[getter]
  fn get_start(&self) -> f64 {
    self.start
  }

  /// Distance from the start of the track where it leaves
  /// the volume in mm
  #[getter]
  fn get_stop(&self) -> f64 {
    self.stop
  }

  /// Density of the material of the volume in g/cm3
  #[getter]
  fn get_density(&self) -> f64 {
    self.density
  }

  /// Path length inside the volume in mm
  #[getter]
  #[pyo3(name="length")]
  fn get_length_py(&self) -> f64 {
    self.length()
  }

  /// Column density inside the volume in g/cm2
  #[getter]
  #[pyo3(name="column_density")]
  fn get_column_density_py(&self) -> f64 {
    self.column_density()
  }
}

#[cfg(feature="pybindings")]
pythonize_display!(Crossing);

//------------------------------------------------------------

/// The volumes along a straight track
#[derive(Debug, Clone, PartialEq)]
#[cfg_attr(feature="pybindings", pyclass)]
pub struct TrackPath {
  /// length of the track in mm
  pub length    : f64,
  /// all parts of the track inside a volume (including the
  /// world), sorted by where they start
  pub crossings : Vec<Crossing>,
  /// copy numbers of volumes which have been left out,
  /// because the track crosses their (not closed) mesh
  /// an unequal number of times inwards and outwards
  pub rejected  : Vec<Option<i64>>,
}

impl TrackPath {
  /// Column density along the track in g/cm2
  pub fn column_density(&self) -> f64 {
    self.column_density_until(f64::INFINITY)
  }

  /// Column density from the start of the track up to a
  /// distance (in mm) along it in g/cm2
  pub fn column_density_until(&self, distance : f64) -> f64 {
    let mut x = 0.0;
    for c in &self.crossings {
      if c.start >= distance {
        break;
      }
      let mother_density = match c.mother {
        Some(m) => self.mother_density(m),
        None    => 0.0,
      };
      x += (c.density - mother_density)*(c.stop.min(distance) - c.start)/10.0;
    }
    x
  }

  fn mother_density(&self, mother : VolumeId) -> f64 {
    self.crossings.iter()
      .find(|c| c.volume == mother)
      .map(|c| c.density)
      .unwrap_or(0.0)
  }

  /// Pairs of crossings of volumes with the same mother
  /// which overlap along the track, as
  /// (copynumber, copynumber, overlap in mm). Overlapping
  /// volumes are an error in the geometry.
  pub fn overlaps(&self) -> Vec<(Option<i64>, Option<i64>, f64)> {
    let mut overlaps = Vec::new();
    for (i, a) in self.crossings.iter().enumerate() {
      for b in &self.crossings[i + 1..] {
        if b.start >= a.stop - OVERLAP_TOLERANCE {
          break;
        }
        if a.mother == b.mother && a.mother.is_some() {
          overlaps.push((a.copynumber, b.copynumber, a.stop.min(b.stop) - b.start));
        }
      }
    }
    overlaps
  }
}

impl fmt::Display for TrackPath {
  fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
    write!(f, "<TrackPath {:.3} mm, {} crossings, {} rejected, {:.4} g/cm2>",
           self.length, self.crossings.len(), self.rejected.len(), self.column_density())
  }
}

#[cfg(feature="pybindings")]
#[pymethods]
impl TrackPath {
  /// Length of the track in mm
  #[getter]
  fn get_length(&self) -> f64 {
    self.length
  }

  /// All parts of the track inside a volume (including the
  /// world), sorted by where they start
  #[getter]
  fn get_crossings(&self) -> Vec<Crossing> {
    self.crossings.clone()
  }

  /// Copy numbers of volumes which have been left out,
  /// because the track crosses their (not closed) mesh an
  /// unequal number of times inwards and outwards
  #[getter]
  fn get_rejected(&self) -> Vec<Option<i64>> {
    self.rejected.clone()
  }

  /// Column density along the track in g/cm2
  #[getter]
  #[pyo3(name="column_density")]
  fn get_column_density_py(&self) -> f64 {
    self.column_density()
  }

  /// Column density from the start of the track up to a
  /// distance (in mm) along it in g/cm2
  #[pyo3(name="column_density_until")]
  fn column_density_until_py(&self, distance : f64) -> f64 {
    self.column_density_until(distance)
  }

  /// Pairs of crossings of volumes with the same mother
  /// which overlap along the track, as
  /// (copynumber, copynumber, overlap in mm)
  #[pyo3(name="overlaps")]
  fn overlaps_py(&self) -> Vec<(Option<i64>, Option<i64>, f64)> {
    self.overlaps()
  }
}

#[cfg(feature="pybindings")]
pythonize_display!(TrackPath);

//------------------------------------------------------------

impl Geometry {
  /// The raycaster for this geometry, it is built the first
  /// time it is needed (this takes about half a second for
  /// the GAPS geometry)
  pub fn raycaster(&self) -> &Raycaster {
    self.raycaster.0.get_or_init(|| Raycaster::new(self))
  }

  /// The volumes along the straight track a -> b (in mm)
  pub fn trace(&self, a : DVec3, b : DVec3) -> TrackPath {
    self.raycaster().trace(self, a, b)
  }

  /// Column density along the straight track a -> b (in mm)
  /// in g/cm2, including the air of the world volume
  pub fn column_density(&self, a : DVec3, b : DVec3) -> f64 {
    self.trace(a, b).column_density()
  }

  /// Cumulative column density in g/cm2 at each point of a
  /// track made of straight segments between the points.
  /// The first value is always 0.
  pub fn depths(&self, points : &[DVec3]) -> Vec<f64> {
    let mut depths = Vec::with_capacity(points.len());
    let mut x = 0.0;
    for (k, p) in points.iter().enumerate() {
      if k > 0 {
        x += self.column_density(points[k - 1], *p);
      }
      depths.push(x);
    }
    depths
  }

  /// The start and stop point of a tracklet (in mm). For an
  /// infinite tracklet, the track goes from the start along
  /// the vertex momentum until it leaves the world volume.
  pub fn tracklet_endpoints(&self, tracklet : &Tracklet) -> Result<(DVec3, DVec3), GeometryError> {
    let start = DVec3::new(tracklet.start.x as f64, tracklet.start.y as f64, tracklet.start.z as f64);
    if !tracklet.is_infinite {
      let stop = DVec3::new(tracklet.stop.x as f64, tracklet.stop.y as f64, tracklet.stop.z as f64);
      return Ok((start, stop));
    }
    let mom = DVec3::new(tracklet.vertex_mom_x as f64, tracklet.vertex_mom_y as f64, tracklet.vertex_mom_z as f64);
    if mom.length() == 0.0 {
      return Err(GeometryError::InvalidTrack(String::from("Infinite tracklet without vertex momentum")));
    }
    let dir   = mom.normalize();
    let world = self.raycaster().bounds[0];
    let stop  = match world.clip_line(start, dir, 0.0, f64::INFINITY) {
      Some((_, t1)) => start + t1*dir,
      None          => start,
    };
    Ok((start, stop))
  }

  /// The volumes along a tracklet, see tracklet_endpoints
  pub fn trace_tracklet(&self, tracklet : &Tracklet) -> Result<TrackPath, GeometryError> {
    let (start, stop) = self.tracklet_endpoints(tracklet)?;
    Ok(self.trace(start, stop))
  }

  /// Column density along a tracklet in g/cm2, see
  /// tracklet_endpoints
  pub fn tracklet_column_density(&self, tracklet : &Tracklet) -> Result<f64, GeometryError> {
    Ok(self.trace_tracklet(tracklet)?.column_density())
  }
}

//------------------------------------------------------------

#[cfg(test)]
mod tests {
  use super::*;
  use crate::geometry::gdml::parse_gdml_str;

  /// A 100 mm aluminium cube at the origin and an aluminium
  /// ring at z = 200 mm, in an air world
  const GDML : &str = r#"<?xml version="1.0" encoding="UTF-8"?>
<gdml>
  <materials>
    <material name="Air"><D unit="g/cm3" value="0.001"/></material>
    <material name="Alu"><D unit="g/cm3" value="2.7"/></material>
  </materials>
  <solids>
    <box lunit="mm" name="World" x="1000" y="1000" z="1000"/>
    <box lunit="mm" name="Cube" x="100" y="100" z="100"/>
    <tube aunit="deg" deltaphi="360" lunit="mm" name="Ring" rmax="50" rmin="40" startphi="0" z="10"/>
  </solids>
  <structure>
    <volume name="CubeVol">
      <materialref ref="Alu"/>
      <solidref ref="Cube"/>
    </volume>
    <volume name="RingVol">
      <materialref ref="Alu"/>
      <solidref ref="Ring"/>
    </volume>
    <volume name="WorldVol">
      <materialref ref="Air"/>
      <solidref ref="World"/>
      <physvol copynumber="1" name="cube">
        <volumeref ref="CubeVol"/>
      </physvol>
      <physvol copynumber="2" name="ring">
        <volumeref ref="RingVol"/>
        <position name="ring_pos" unit="mm" x="0" y="0" z="200"/>
      </physvol>
    </volume>
  </structure>
  <setup name="Default" version="1.0">
    <world ref="WorldVol"/>
  </setup>
</gdml>
"#;

  #[test]
  fn column_density_through_boxes_and_tubes() {
    let geo = parse_gdml_str(GDML).unwrap();
    // along x through the cube: 100 mm alu, 300 mm air
    let a = DVec3::new(-200.0, 0.0, 0.0);
    let b = DVec3::new( 200.0, 0.0, 0.0);
    let path = geo.trace(a, b);
    assert_eq!(path.crossings.len(), 2);
    assert_eq!(path.crossings[1].copynumber, Some(1));
    assert!((path.crossings[1].length() - 100.0).abs() < 1e-9);
    let expected = (2.7*100.0 + 0.001*300.0)/10.0;
    assert!((geo.column_density(a, b) - expected).abs() < 1e-12);
    // along y through the ring at z = 200: 2 x 10 mm alu
    let a = DVec3::new(0.0, -200.0, 200.0);
    let b = DVec3::new(0.0,  200.0, 200.0);
    let path = geo.trace(a, b);
    assert_eq!(path.crossings.iter().filter(|c| c.copynumber == Some(2)).count(), 2);
    let expected = (2.7*20.0 + 0.001*380.0)/10.0;
    assert!((path.column_density() - expected).abs() < 1e-12);
    assert!((path.column_density_until(200.0) - expected/2.0).abs() < 1e-12);
    assert!(path.overlaps().is_empty());
  }

  #[test]
  fn tracks_leaving_the_world_and_depths() {
    let geo = parse_gdml_str(GDML).unwrap();
    // the world ends at |x| = 500, there is nothing beyond
    let a = DVec3::new(-2000.0, 0.0, 0.0);
    let b = DVec3::new( 2000.0, 0.0, 0.0);
    let expected = (2.7*100.0 + 0.001*900.0)/10.0;
    assert!((geo.column_density(a, b) - expected).abs() < 1e-12);
    // depths along a polyline
    let points = [DVec3::new(-200.0, 0.0, 0.0), DVec3::ZERO, DVec3::new(0.0, 0.0, 200.0)];
    let depths = geo.depths(&points);
    assert_eq!(depths[0], 0.0);
    assert!((depths[1] - (2.7*50.0 + 0.001*150.0)/10.0).abs() < 1e-12);
    // along the axis of the ring, through its hole
    assert!((depths[2] - depths[1] - (2.7*50.0 + 0.001*150.0)/10.0).abs() < 1e-12);
  }

  #[test]
  fn tracklets() {
    let geo = parse_gdml_str(GDML).unwrap();
    let mut t = Tracklet::new();
    t.start.x = -200.0;
    t.is_infinite = false;
    t.stop.x = 200.0;
    let expected = (2.7*100.0 + 0.001*300.0)/10.0;
    assert!((geo.tracklet_column_density(&t).unwrap() - expected).abs() < 1e-9);
    // an infinite tracklet goes until the end of the world
    t.is_infinite  = true;
    t.vertex_mom_x = 3.0;
    let (_, stop) = geo.tracklet_endpoints(&t).unwrap();
    assert!((stop - DVec3::new(500.0, 0.0, 0.0)).length() < 1e-9);
    let expected = (2.7*100.0 + 0.001*600.0)/10.0;
    assert!((geo.tracklet_column_density(&t).unwrap() - expected).abs() < 1e-9);
    t.vertex_mom_x = 0.0;
    assert!(matches!(geo.tracklet_column_density(&t), Err(GeometryError::InvalidTrack(_))));
  }
}
