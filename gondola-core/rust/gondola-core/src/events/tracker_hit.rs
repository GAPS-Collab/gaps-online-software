//! Per-strip event information for the GAPS tracker
// This file is part of gaps-online-software and published 
// under the GPLv3 license

use crate::prelude::*;

#[cfg(feature="pybindings")]
use pyo3::basic::CompareOp;

/// Hit on a tracker strip
#[derive(Debug, Copy, Clone)]
#[cfg_attr(feature="pybindings", pyclass)]
pub struct TrackerHit {
  pub layer           : u8,
  pub row             : u8,
  pub module          : u8,
  pub channel         : u8,
  pub adc             : u16,
  pub oscillator      : u64,
  /// In BFSW, there are two versions of the tracker hit, 
  /// tracker_hit and tracker::hit. The latter has 
  /// an extra ASIC event code field. Let's unify those here
  pub asic_event_code : u8,

  // not getting serialized
  /// calibrated energy
  pub energy          : f32, 
  pub x               : f32,
  pub y               : f32,
  pub z               : f32,
  pub has_coordinates : bool,
  pub adc_pedestal    : u16,
}

impl TrackerHit {
  //const SIZE : usize = 18;
  
  pub fn new() -> Self {
    Self {
      layer           : 0,
      row             : 0,
      module          : 0,
      channel         : 0,
      adc             : 0,
      oscillator      : 0,
      asic_event_code : 0,
      energy          : 0.0,
      x               : 0.0,
      y               : 0.0,
      z               : 0.0,
      has_coordinates : false,
      adc_pedestal    : 0,
    }
  }
 
  /// Calculate the strip id from layer, module, row and channel
  pub fn get_stripid(&self) -> u32 {
    crate::events::strip_id(self.layer  , 
                            self.row    ,
                            self.module ,
                            self.channel)
  }

 #[cfg(feature="database")]
 pub fn set_coordinates(&mut self, strip_map : &HashMap<u32, TrackerStrip>) {
   match strip_map.get(&self.get_stripid()) {
     None  => debug!("Can not get strip for strip id {}" , self.get_stripid()),
     Some(strip) => { 
       self.x = strip.global_pos_x_l0;
       self.y = strip.global_pos_y_l0;
       self.z = strip.global_pos_z_l0;
       self.has_coordinates = true
     }
   }
 }
}

impl PartialEq for TrackerHit {
  fn eq(&self, other: &TrackerHit) -> bool {
    // we can only compare fields here which 
    // are always set, merged events do 
    // not have the asic event code 
    // populated
    self.layer              ==  other.layer           
    && self.row             ==  other.row            
    && self.module          ==  other.module         
    && self.channel         ==  other.channel        
    && self.adc             ==  other.adc            
    && self.oscillator      ==  other.oscillator     
    //&& self.asic_event_code ==  other.asic_event_code
  }
}


impl fmt::Display for TrackerHit {
  fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
    let mut repr = String::from("<TrackerHit:");
    repr += &(format!("\n  Layer, Row, Module, Channel : {} {} {} {}" ,self.layer, self.row, self.module, self.channel));
    repr += &(format!("\n  ADC           : {}" ,self.adc));
    repr += &(format!("\n  Oscillator    : {}" ,self.oscillator));
    repr += &(format!("\n  ASIC Evt code : {}" ,self.asic_event_code)); 
    if self.has_coordinates {
      repr += &(format!("\n -- coordinates x : {} , y : {} , z {}", self.x, self.y, self.z));
    } else {
      repr += "\n -- [no coordinates set]";
    }
    repr += &(format!("\n  Cali. energy  : {}>", self.energy));
    write!(f, "{}", repr)
  }
}

#[cfg(feature="pybindings")]
#[pymethods]
impl TrackerHit {

  /// Change the ADC value, e.g. if the 
  /// pedestal should be subtracted
  fn subtract_pedestal(&mut self, pedestal : u16) {
    self.adc -= pedestal;
  }

  #[getter]
  fn get_strip_id(&self) -> u32 {
    self.get_stripid()
  }

  #[getter]
  fn get_layer(&self) -> u8 {
    self.layer
  }

  #[getter]
  fn get_row(&self) -> u8 {
    self.row
  }

  #[getter]
  fn get_module(&self) -> u8 {
    self.module
  }

  #[getter]
  fn get_channel(&self) -> u8 {
    self.channel
  }

  #[getter]
  fn get_adc(&self) -> u16 {
    self.adc
  }

  #[getter]
  fn get_oscillator(&self) -> u64 {
    self.oscillator
  }
 
  #[getter] 
  fn get_asic_event_code(&self) -> u8 {
    self.asic_event_code
  }

  #[getter]
  fn get_energy(&self) -> f32 {
    self.energy
  }

  #[getter]
  fn get_x(&self) -> f32 {
    self.x
  }
  
  #[getter]
  fn get_y(&self) -> f32 {
    self.y
  }
  
  #[getter]
  fn get_z(&self) -> f32 {
    self.z
  }
    
  // This handles Python's == and != (and others if you wish)
  fn __richcmp__(&self, other: &Self, op: CompareOp) -> PyResult<bool> {
    match op {
      CompareOp::Eq => Ok(self == other),
      CompareOp::Ne => Ok(self != other),
      _ => Ok(false), // Or return an error for unsupported ops like < or >
    }
  }
}

#[cfg(feature="random")]
impl FromRandom for TrackerHit {
  fn from_random() -> Self {
    let mut rng       = rand::rng();
    Self {
      layer           : rng.random_range(0..9),
      row             : rng.random_range(0..6),
      module          : rng.random_range(0..6),
      channel         : rng.random_range(0..32),
      adc             : rng.random::<u16>() & 0x7ff,
      oscillator      : rng.random::<u64>(),
      asic_event_code : rng.random_range(0..4),
      energy          : 0.0, 
      x               : 0.0,
      y               : 0.0,
      z               : 0.0,
      has_coordinates : false,
      adc_pedestal    : 0,
    }
  }
}


#[cfg(feature="pybindings")]
pythonize!(TrackerHit);

