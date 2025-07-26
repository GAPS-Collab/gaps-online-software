//! The following file is part of gaps-online-software and published 
//! under the GPLv3 license

use std::fmt;

use crate::monitoring::MoniData;

#[cfg(feature = "random")]
use crate::random::FromRandom;
#[cfg(feature = "random")]
use rand::Rng;

use crate::io::serialization::Serialization;

use crate::packets::TofPackable;

use crate::io::parsers::{
  parse_u8,
  parse_u16,
  parse_u32,
  parse_f32
};

use crate::packets::TofPacketType;
use crate::errors::SerializationError;

#[cfg(feature="pybindings")]
use pyo3::prelude::*;

#[cfg(feature="pybindings")]
use pyo3::exceptions::{
  PyKeyError,
  PyIOError
};

#[cfg(feature="pybindings")]
use crate::packets::TofPacket;

#[cfg(feature="pybindings")]
use crate::pythonize_packable;

#[derive(Debug, Copy, Clone, PartialEq)]
#[cfg_attr(feature="pybindings", pyclass)]
pub struct CPUMoniData {
  pub uptime     : u32,
  pub disk_usage : u8,
  pub cpu_freq   : [u32; 4],
  pub cpu_temp   : f32,
  pub cpu0_temp  : f32,
  pub cpu1_temp  : f32,
  pub mb_temp    : f32,
}

impl CPUMoniData {
  pub fn new() -> Self {
    Self {
      uptime     : u32::MAX,
      disk_usage : u8::MAX,
      cpu_freq   : [u32::MAX; 4],
      cpu_temp   : f32::MAX,
      cpu0_temp  : f32::MAX,
      cpu1_temp  : f32::MAX,
      mb_temp    : f32::MAX,
    }
  }

  pub fn get_temps(&self) -> Vec<f32> {
    vec![self.cpu0_temp, self.cpu1_temp, self.cpu_temp, self.mb_temp]
  }

  //#[cfg(feature = "tofcontrol")]
  //pub fn add_temps(&mut self, cpu_temps : &CPUTempDebug) {
  //  self.cpu_temp   = cpu_temps.cpu_temp;
  //  self.cpu0_temp  = cpu_temps.cpu0_temp;
  //  self.cpu1_temp  = cpu_temps.cpu1_temp;
  //  self.mb_temp    = cpu_temps.mb_temp;
  //}

  //#[cfg(feature = "tofcontrol")]
  //pub fn add_info(&mut self, cpu_info : &CPUInfoDebug) {
  //  self.uptime = cpu_info.uptime;
  //  self.disk_usage = cpu_info.disk_usage;
  //  self.cpu_freq   = cpu_info.cpu_freq;
  //}
}

impl Default for CPUMoniData {
  fn default() -> Self {
    Self::new()
  }
}

impl fmt::Display for CPUMoniData {
  fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
    write!(f, "<CPUMoniData:
  core0   temp [\u{00B0}C] : {:.2} 
  core1   temp [\u{00B0}C] : {:.2} 
  CPU     temp [\u{00B0}C] : {:.2} 
  MB      temp [\u{00B0}C] : {:.2} 
  CPU (4) freq [Hz] : {} | {} | {} | {} 
  Disc usage   [%]  : {} 
  Uptime       [s]  : {}>",
           self.cpu0_temp,
           self.cpu1_temp,
           self.cpu_temp,
           self.mb_temp,
           self.cpu_freq[0],
           self.cpu_freq[1],
           self.cpu_freq[2],
           self.cpu_freq[3],
           self.disk_usage,
           self.uptime)
  }
}

impl TofPackable for CPUMoniData {
  const TOF_PACKET_TYPE : TofPacketType = TofPacketType::CPUMoniData;
}

impl Serialization for CPUMoniData {
  
  const SIZE : usize = 41;
  const HEAD : u16   = 0xAAAA;
  const TAIL : u16   = 0x5555;

  fn to_bytestream(&self) -> Vec<u8> {
    let mut stream = Vec::<u8>::with_capacity(Self::SIZE);
    stream.extend_from_slice(&Self::HEAD.to_le_bytes());
    stream.extend_from_slice(&self.uptime  .to_le_bytes());
    stream.extend_from_slice(&self.disk_usage  .to_le_bytes());
    for k in 0..4 {
      stream.extend_from_slice(&self.cpu_freq[k].to_le_bytes());
    }
    stream.extend_from_slice(&self.cpu_temp .to_le_bytes());
    stream.extend_from_slice(&self.cpu0_temp.to_le_bytes());
    stream.extend_from_slice(&self.cpu1_temp.to_le_bytes());
    stream.extend_from_slice(&self.mb_temp  .to_le_bytes());
    stream.extend_from_slice(&Self::TAIL.to_le_bytes());
    stream
  }

  fn from_bytestream(stream : &Vec<u8>, pos : &mut usize)
    -> Result<Self, SerializationError> {
    Self::verify_fixed(stream, pos)?;
    let mut moni = CPUMoniData::new();
    moni.uptime     = parse_u32(stream, pos); 
    moni.disk_usage = parse_u8(stream, pos); 
    for k in 0..4 {
      moni.cpu_freq[k] = parse_u32(stream, pos);
    }
    moni.cpu_temp   = parse_f32(stream, pos);
    moni.cpu0_temp  = parse_f32(stream, pos);
    moni.cpu1_temp  = parse_f32(stream, pos);
    moni.mb_temp    = parse_f32(stream, pos);
    *pos += 2;
    Ok(moni)
  }
}

impl MoniData for CPUMoniData {

  fn get_board_id(&self) -> u8 {
    return 0;
  }
  
  fn get(&self, varname : &str) -> Option<f32> {
    match varname {
      "uptime"     =>    Some(self.uptime as f32),
      "disk_usage" =>    Some(self.disk_usage as f32),
      "cpu_freq0"  =>    Some(self.cpu_freq[0] as f32),
      "cpu_freq1"  =>    Some(self.cpu_freq[1] as f32),
      "cpu_freq2"  =>    Some(self.cpu_freq[2] as f32),
      "cpu_freq3"  =>    Some(self.cpu_freq[0] as f32),
      "cpu_temp"   =>    Some(self.cpu_temp),
      "cpu0_temp"  =>    Some(self.cpu0_temp),
      "cpu1_temp"  =>    Some(self.cpu1_temp),
      "mb_temp"    =>    Some(self.mb_temp),
      _            =>    None
    }
  }
  
  fn keys() -> Vec<&'static str> {
    vec![
      "uptime"     ,
      "disk_usage" ,
      "cpu_freq0"  ,
      "cpu_freq1"  ,
      "cpu_freq2"  ,
      "cpu_freq3"  ,
      "cpu_temp"   ,
      "cpu0_temp"  ,
      "cpu1_temp"  ,
      "mb_temp"]
  }
}

#[cfg(feature = "random")]
impl FromRandom for CPUMoniData {
    
  fn from_random() -> Self {
    let mut moni    = Self::new();
    let mut rng     = rand::thread_rng();
    moni.uptime     = rng.gen::<u32>();
    moni.disk_usage = rng.gen::<u8>();
    for k in 0..4 {
      moni.cpu_freq[k] = rng.gen::<u32>();
    }
    moni.cpu_temp   = rng.gen::<f32>();
    moni.cpu0_temp  = rng.gen::<f32>();
    moni.cpu1_temp  = rng.gen::<f32>();
    moni.mb_temp    = rng.gen::<f32>();
    moni
  }
}

#[cfg(feature="pybindings")]
#[pymethods]
impl CPUMoniData {

  #[new]
  fn new_py() -> Self {
    Self::new()
  }

  #[getter]
  fn get_uptime(&self) -> u32 {
    self.uptime
  }

  #[getter]
  fn get_disk_usage(&self) -> u8 {
    self.disk_usage
  }

  #[getter]
  fn get_cpu_freq(&self) -> [u32; 4] {
    self.cpu_freq
  }

  #[getter]
  fn get_cpu_temp(&self) -> f32 {
    self.cpu_temp
  }

  #[getter]
  fn get_cpu0_temp(&self) -> f32 {
    self.cpu0_temp
  }

  #[getter]
  fn get_cpu1_temp(&self) -> f32 {
    self.cpu1_temp
  }

  #[getter]
  fn get_mb_temp(&self) -> f32 {
    self.mb_temp
  }
  
  #[staticmethod]
  #[pyo3(name="keys")]
  fn keys_py() -> Vec<&'static str> {
    Self::keys()
  }

  /// Access the (data) members by name
  #[pyo3(name="get")]
  fn get_py(&self, varname : &str) -> PyResult<f32> {
    match self.get(varname) {
      None => {
        let err_msg = format!("CPUMoniData does not have a key with name {}! See CPUmoniData.keys() for a list of available keys!", varname);
        return Err(PyKeyError::new_err(err_msg));
      }
      Some(val) => {
        return Ok(val)
      }
    }
  }

  #[getter]
  fn board_id      (&self)  -> u8  {
    self.get_board_id()
  }
}


#[cfg(attr="pybindings")]
pythonize_packable!(CPUMoniData);

//----------------------------------------

#[test]
#[cfg(feature = "random")]
fn monidata_cpumonidata() {
  let data = CPUMoniData::from_random();
  for k in CPUMoniData::keys() {
    assert!(data.get(k).is_some());
  }
  assert_eq!(data.get_board_id(), 0u8);
}

