//! Per-strip event information for the GAPS tracker
// This file is part of gaps-online-software and published 
// under the GPLv3 license


use crate::prelude::*;

/// Hit on a tracker strip
#[derive(Debug, Copy, Clone)]
#[cfg_attr(feature="pybindings", pyclass)]
pub struct TrackerHit {
  pub layer           : u16,
  pub row             : u16,
  pub module          : u16,
  pub channel         : u16,
  pub adc             : u16,
  pub oscillator      : u64,

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
    crate::events::strip_id(self.layer   as u8, 
                            self.row     as u8,
                            self.module  as u8,
                            self.channel as u8)
  }

 #[cfg(feature="database")]
 pub fn set_coordinates(&mut self, strip_map : &HashMap<u32, TrackerStrip>) {
   match strip_map.get(&self.get_stripid()) {
     None  => error!("Can not get strip for strip id {}" , self.get_stripid()),
     Some(strip) => { 
       self.x = strip.global_pos_x_l0;
       self.y = strip.global_pos_y_l0;
       self.z = strip.global_pos_z_l0;
       self.has_coordinates = true
     }
   }
 }
}

impl fmt::Display for TrackerHit {
  fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
    let mut repr = String::from("<TrackerHit:");
    repr += &(format!("\n  Layer, Row, Module, Channel : {} {} {} {}" ,self.layer, self.row, self.module, self.channel));
    repr += &(format!("\n  ADC           : {}" ,self.adc));
    repr += &(format!("\n  Oscillator    : {}",self.oscillator));
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
  fn get_layer(&self) -> u16 {
    self.layer
  }

  #[getter]
  fn get_row(&self) -> u16 {
    self.row
  }

  #[getter]
  fn get_module(&self) -> u16 {
    self.module
  }

  #[getter]
  fn get_channel(&self) -> u16 {
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
}

#[cfg(feature="pybindings")]
pythonize!(TrackerHit);

