//! GAPS detector geometry, read from the GDML file
//! (resources/geometry/geometry.vXX.YY.gdml in crane)
//!
//! The parser supports the subset of GDML which is used
//! by the GAPS geometry:
//!
//! * define    : position, rotation
//! * materials : material (only the density is kept)
//! * solids    : tessellated, box, tube, intersection, subtraction, union
//! * structure : volume, physvol
//! * setup     : world
//!
//! Anything else in define, solids or structure is an error,
//! so that a geometry which is not understood does not
//! silently produce wrong results.
//!
//! Since the GDML file is large (~500 MB) the parsed geometry
//! can be written to and read from a binary cache
//! (see Geometry::load).
// This file is part of gaps-online-software and published
// under the GPLv3 license

pub mod model;
pub use model::*;

pub mod gdml;
pub use gdml::{
  parse_gdml_file,
  parse_gdml_str,
};

pub mod bvh;
pub use bvh::Aabb;

pub mod raycast;
pub use raycast::{
  Intervals,
  SolidRaycaster,
};

pub mod column_density;
pub use column_density::{
  Crossing,
  Raycaster,
  TrackPath,
};
