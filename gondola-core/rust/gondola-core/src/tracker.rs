// This file is part of gaps-online-software and published 
// under the GPLv3 license


pub mod strips;
pub use strips::*;

pub mod online_calibration;
pub use online_calibration::*;

use crate::prelude::*;

#[derive(Debug, Hash, Eq, PartialEq, Clone, Copy, FromRepr, AsRefStr, EnumIter)]
#[cfg_attr(feature = "pybindings", pyclass(eq, eq_int))]
#[repr(u8)]
pub enum TrackerHitSource {
  Unknown = 0,
  TelemetryEvent = 10,
  TrackerPacket  = 20,
}

// in case we have pybindings for this type, 
// expand it so that it can be used as keys
// in dictionaries
#[cfg(feature = "pybindings")]
#[pymethods]
impl TrackerHitSource {

  #[getter]
  fn __hash__(&self) -> usize {
    (*self as u8) as usize
  } 
}

expand_and_test_enum!(TrackerHitSource, test_trackerhitsource_repr);

