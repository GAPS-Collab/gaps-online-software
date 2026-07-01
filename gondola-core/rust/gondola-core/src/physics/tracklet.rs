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


/// A "RecoHit" is truly just an assembly of coordinates, time 
/// and energy, allowing to carry an error on the position along. 
#[derive(Debug, Copy, Clone, PartialEq)]
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

impl Serialization for RecoHit {
  //FIXME - this has fixed size!!
  fn from_bytestream(stream : &Vec<u8>,
                     pos    : &mut usize) 
    -> Result<Self, SerializationError> {
    let mut hit = Self::new();
    let head    = parse_u16(stream, pos);
    if head != Self::HEAD {
      error!("Decoding of HEAD failed! Got {} instead!", head);
      return Err(SerializationError::HeadInvalid);
    }
    hit.x      = parse_f32(stream, pos);
    hit.x_err  = parse_f32(stream, pos);
    hit.y      = parse_f32(stream, pos);
    hit.y_err  = parse_f32(stream, pos);
    hit.z      = parse_f32(stream, pos);
    hit.z_err  = parse_f32(stream, pos);
    hit.time   = parse_f32(stream, pos);
    hit.energy = parse_f32(stream, pos);
    hit.volume = parse_u32(stream, pos);
    let tail   = parse_u16(stream, pos);
    if tail != Self::TAIL {
      //error!("Decoding of TAIL failed for version {}! Got {} instead!", version, tail);
      error!("Decoding of TAIL failed! Got {} instead!", tail);
      return Err(SerializationError::TailInvalid);
    }
    Ok(hit) 
  }
}

impl fmt::Display for RecoHit {
  fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
    let mut repr = String::from("<RecoHit");
    repr += &(format!("\n  x {:.2} y {:.2} z {:.2}", self.x, self.y, self.z));
    repr += &(format!("\n  energy {:.2}", self.energy));
    repr += &(format!("\n  time   {:.2}>", self.time));
    write!(f,"{}", repr)
  } 
}

// --------------------------------------------

/// This is semantics. However, in a vertex the 
/// energy will return the energy of the particle 
/// at that actual position, not the energy depositions
type Vertex = RecoHit;

// --------------------------------------------

/// A representation of a physics track, as a single, 
/// unbent, stright line
#[derive(Debug, Copy, Clone, PartialEq)]
#[cfg_attr(feature="pybindings", pyclass)] 
pub struct Tracklet {
  pub start         : Vertex,
  pub stop          : Vertex,
  pub is_infinite   : bool,
  pub vertex_mom_x  : f32,
  pub vertex_mom_y  : f32,
  pub vertex_mom_z  : f32,
  /// particle identifier
  pub pdg           : i32,
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
      pdg           : 0,
    }
  }

  pub fn get_vertex_pos(&self) -> (f32,f32,f32) {
    (self.start.x, self.start.y, self.start.z)
  }
  
  pub fn get_vertex_mom(&self) -> (f32,f32,f32) {
    (self.vertex_mom_x, self.vertex_mom_y, self.vertex_mom_z)
  }
}

impl Serialization for Tracklet {
  //FIXME - this has fixed size!!
  fn from_bytestream(stream : &Vec<u8>,
                     pos    : &mut usize) 
    -> Result<Self, SerializationError> {
    let mut tracklet = Self::new();
    let head    = parse_u16(stream, pos);
    if head != Self::HEAD {
      error!("Decoding of HEAD failed! Got {} instead!", head);
      return Err(SerializationError::HeadInvalid);
    }
    tracklet.start         = RecoHit::from_bytestream(stream, pos)?;
    tracklet.stop          = RecoHit::from_bytestream(stream, pos)?;
    tracklet.is_infinite   = parse_bool(stream, pos);
    tracklet.vertex_mom_x  = parse_f32(stream, pos);
    tracklet.vertex_mom_y  = parse_f32(stream, pos);
    tracklet.vertex_mom_z  = parse_f32(stream, pos);
    tracklet.pdg           = parse_i32(stream, pos);
    let tail   = parse_u16(stream, pos);
    if tail != Self::TAIL {
      //error!("Decoding of TAIL failed for version {}! Got {} instead!", version, tail);
      error!("Decoding of TAIL failed! Got {} instead!", tail);
      return Err(SerializationError::TailInvalid);
    }
    Ok(tracklet) 
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
    self.start.energy
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
    repr += &(format!("\n  vertex {:.2} {:.2} {:.2}", self.start.x, self.start.y, self.start.z));
    repr += &(format!("\n  pdg    {}>", self.pdg));
    write!(f,"{}", repr)
  } 
}

#[cfg(feature="pybindings")]
pythonize!(Tracklet);


pub struct Track {
  pub tracklets : Vec<Tracklet>
}

