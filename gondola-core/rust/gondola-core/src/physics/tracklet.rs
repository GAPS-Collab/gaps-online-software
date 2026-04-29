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

//------------------------------------------------------------

#[derive(Debug, Copy, Clone)]
#[cfg_attr(feature="pybindings", pyclass)] 
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

impl RecoHit {
  pub fn new() -> Self {
    Self {
      x      : 0.0,
      x_err  : 0.0,
      y      : 0.0,
      y_err  : 0.0,
      z      : 0.0,
      z_err  : 0.0,
      time   : 0.0,
      energy : 0.0,
      volume : 0
    }
  }
}

/// A representation of a physics track, as a single, 
/// unbent, stright line
#[derive(Debug, Copy, Clone)]
#[cfg_attr(feature="pybindings", pyclass)] 
pub struct Tracklet {
  pub start         : RecoHit,
  pub stop          : RecoHit,
  pub is_infinite   : bool,
  pub vertex_mom_x  : f32,
  pub vertex_mom_y  : f32,
  pub vertex_mom_z  : f32,
  pub vertex_x      : f32,
  pub vertex_y      : f32,
  pub vertex_z      : f32,
  // initial kinetic energy in MeV
  pub vertex_energy : f32,
}

impl Tracklet {
  pub fn new() -> Self {
    Self {
      start         : RecoHit::new(),
      stop          : RecoHit::new(),
      is_infinite   : true,
      vertex_mom_x  : 0.0,
      vertex_mom_y  : 0.0,
      vertex_mom_z  : 0.0,
      vertex_x      : 0.0,
      vertex_y      : 0.0,
      vertex_z      : 0.0,
      vertex_energy : 0.0,
    }
  }

  pub fn get_vertex_pos(&self) -> (f32,f32,f32) {
    (self.vertex_x, self.vertex_y, self.vertex_z)
  }
  
  pub fn get_vertex_mom(&self) -> (f32,f32,f32) {
    (self.vertex_mom_x, self.vertex_mom_y, self.vertex_mom_z)
  }
}

#[cfg(feature="pybindings")]
#[pymethods]
impl Tracklet {
  
  #[getter]
  #[pyo3(name="vertex_mom")]
  fn get_vertex_mom_py(&self) -> (f32,f32,f32) {
    self.get_vertex_mom() 
  }

  #[getter] 
  #[pyo3(name="vertex_energy")]
  fn get_vertex_energy_py(&self) -> f32 {
    self.vertex_energy
  }

  #[getter]
  #[pyo3(name="vertex_pos")]
  fn get_vertex_pos_py(&self) -> (f32,f32,f32) {
    self.get_vertex_pos()
  }

}

impl fmt::Display for Tracklet {
  fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
    let mut repr = String::from("<Tracklet");
    repr += &(format!("\n  vertex {:.2} {:.2} {:.2}", self.vertex_x, self.vertex_y, self.vertex_z));

    write!(f,"{}", repr)
  } 
}

pub struct Track {
  pub tracklets : Vec<Tracklet>
}

