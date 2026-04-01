// This file is part of gaps-online-software and published 
// under the GPLv3 license

use crate::prelude::*;

// copy over the G4Process so that we do not need to
// include G4 here
/// Types of serializable data structures used
/// throughout the TOF system. 
#[derive(Debug, Hash, Eq, PartialEq, Clone, Copy, FromRepr, AsRefStr, EnumIter)]
#[cfg_attr(feature = "pybindings", pyclass(eq, eq_int))]
#[repr(u8)]
pub enum G4ProcessType {
   NotDefined        = 0u8,
   Transportation    = 10u8,
   Electromagnetic   = 20u8,
   Optical           = 30u8,
   Hadronic          = 40u8,
   PhotoLeptonHadron = 50u8,
   Decay             = 60u8,
   General           = 70u8,
   Parametrisation   = 80u8,
   UserDefined       = 90u8,
   Parallel          = 100u8,
   Phonon            = 110u8,
   Ucn               = 120u8,
   Unknown           = 255u8, // since 0 is already assigned
}

// in case we have pybindings for this type, 
// expand it so that it can be used as keys
// in dictionaries
#[cfg(feature = "pybindings")]
#[pymethods]
impl G4ProcessType {

  #[getter]
  fn __hash__(&self) -> usize {
    (*self as u8) as usize
  } 
}

expand_and_test_enum!(G4ProcessType, test_g4processtype_repr);

pub struct RecoHit {
  pub x      : f32,
  pub x_err  : f32,
  pub y      : f32,
  pub y_err  : f32,
  pub z      : f32,
  pub z_err  : f32,
  pub time   : f32,
  pub energy : f32,
  pub volume : u32
}

/// A representation of a physics track, as a single, 
/// unbent, stright line
pub struct Tracklet {
  pub start : RecoHit,
  pub stop  : RecoHit
}

pub struct Track {
  pub tracklets : Vec<Tracklet>
}

