// This file is part of gaps-online-software and published 
// under the GPLv3 license

use crate::prelude::*;

//use std::fmt;
//
//use crate::monitoring::MoniData;
//
//#[cfg(feature = "random")]
//use crate::random::FromRandom;
//#[cfg(feature = "random")]
//use rand::Rng;
//
//use crate::io::serialization::Serialization;
//
//use crate::packets::TofPackable;
//
//use crate::io::parsers::{
//  parse_u8,
//  parse_f32
//};
//
//use crate::packets::TofPacketType;
//use crate::errors::SerializationError;
//
//#[cfg(feature="pybindings")]
//use pyo3::prelude::*;
//
//#[cfg(feature="pybindings")]
//use pyo3::exceptions::{
//  PyKeyError,
//  PyIOError
//};
//
//#[cfg(feature="pybindings")]
//use crate::packets::TofPacket;
//
//use crate::moniseries;
//
//#[cfg(feature="pybindings")]
//use crate::pythonize_packable;
//#[cfg(feature="pybindings")]
//use crate::pythonize_monidata;

#[cfg(feature="tofcontrol")]
use tof_control::helper::ltb_type::{
  LTBThreshold,
  LTBTemp
};

/// Sensors on the LTB
#[derive(Debug, Copy, Clone, PartialEq)]
#[cfg_attr(feature="pybindings", pyclass)]
pub struct LTBMoniData {
  pub board_id   : u8,
  pub trenz_temp : f32,
  pub ltb_temp   : f32,
  pub thresh     : [f32;3],
}

impl LTBMoniData {
  pub fn new() -> LTBMoniData {
    LTBMoniData {
      board_id   : 0,
      trenz_temp : f32::MAX,
      ltb_temp   : f32::MAX,
      thresh     : [f32::MAX,f32::MAX,f32::MAX],
    }
  }

  #[cfg(feature = "tofcontrol")]
  pub fn add_temps(&mut self, lt : &LTBTemp) {
    self.trenz_temp = lt.trenz_temp;
    self.ltb_temp   = lt.board_temp;
  }

  #[cfg(feature = "tofcontrol")]
  pub fn add_thresh(&mut self, lt : &LTBThreshold) {
    self.thresh = [lt.thresh_0, lt.thresh_1, lt.thresh_2];
  }
}

impl Default for LTBMoniData {
  fn default() -> Self {
    Self::new()
  }
}

impl fmt::Display for LTBMoniData {
  fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
    write!(f, "<LTBMoniData:
  Board ID  : {}
  ** Temperatures **
  TRENZ TMP : {:.2} [\u{00B0}C]
  LTB   TMP : {:.2} [\u{00B0}C]
  ** Threshold Voltages **
  THR HIT, THR BETA, THR VETO : {:.3} | {:.3} | {:.3} [mV]>",
  self.board_id,
  self.trenz_temp,
  self.ltb_temp,
  self.thresh[0],
  self.thresh[1],
  self.thresh[2])
  }
}

impl TofPackable for LTBMoniData {
  const TOF_PACKET_TYPE : TofPacketType = TofPacketType::LTBMoniData;
}

impl Serialization for LTBMoniData {
  
  const HEAD : u16 = 0xAAAA;
  const TAIL : u16 = 0x5555;
  /// The data size when serialized to a bytestream
  /// This needs to be updated when we change the 
  /// packet layout, e.g. add new members.
  /// HEAD + TAIL + sum(sizeof(m) for m in _all_members_))
  const SIZE : usize  = 4 + 1 + (4*5) ;
  
  fn to_bytestream(&self) -> Vec<u8> {
    let mut stream = Vec::<u8>::with_capacity(Self::SIZE);
    stream.extend_from_slice(&Self::HEAD.to_le_bytes());
    stream.extend_from_slice(&self.board_id          .to_le_bytes()); 
    stream.extend_from_slice(&self.trenz_temp. to_le_bytes());
    stream.extend_from_slice(&self.ltb_temp.   to_le_bytes());
    for k in 0..3 {
      stream.extend_from_slice(&self.thresh[k].to_le_bytes());
    }
    stream.extend_from_slice(&Self::TAIL.to_le_bytes());
    stream
  }
  
  fn from_bytestream(stream    : &Vec<u8>, 
                     pos       : &mut usize) 
    -> Result<Self, SerializationError>{
    let mut moni     = Self::new();
    Self::verify_fixed(stream, pos)?;
    moni.board_id    = parse_u8(stream, pos);
    moni.trenz_temp  = parse_f32(stream, pos);
    moni.ltb_temp    = parse_f32(stream, pos);
    for k in 0..3 {
      moni.thresh[k] = parse_f32(stream, pos);
    }
    *pos += 2;
    Ok(moni)
  }
}

#[cfg(feature = "random")]
impl FromRandom for LTBMoniData {
    
  fn from_random() -> LTBMoniData {
    let mut moni  = Self::new();
    let mut rng   = rand::rng();
    moni.board_id = rng.random::<u8>(); 
    moni.trenz_temp = rng.random::<f32>();
    moni.ltb_temp   = rng.random::<f32>();
    for k in 0..3 {
      moni.thresh[k] = rng.random::<f32>();
    }
    moni
  }
}

impl MoniData for LTBMoniData {
  fn get_board_id(&self) -> u8 {
    self.board_id
  }
  
  /// Access the (data) members by name 
  fn get(&self, varname : &str) -> Option<f32> {
    match varname {
    "board_id"    => Some(self.board_id as f32),
    "trenz_temp"  => Some(self.trenz_temp),
    "ltb_temp"    => Some(self.ltb_temp),
    "thresh0"     => Some(self.thresh[0]),
    "thresh1"     => Some(self.thresh[1]),
    "thresh2"     => Some(self.thresh[2]),
    _             => None
    }
  }

  /// A list of the variables in this MoniData
  fn keys() -> Vec<&'static str> {
    vec!["board_id", "trenz_temp", "ltb_temp",
         "thresh0", "thresh1", "thresh2"]
  }
}

#[cfg(feature="pybindings")]
#[pymethods]
impl LTBMoniData {
  
  #[getter]
  fn get_trenz_temp    (&self)  -> f32  {
    self.trenz_temp
  }

  #[getter]
  fn get_ltb_temp      (&self)  -> f32  {
    self.ltb_temp
  }
  #[getter]
  fn get_thresh0       (&self)  -> f32  {
    self.thresh[0]
  }
  #[getter]
  fn get_thresh1       (&self)  -> f32  {
    self.thresh[1]
  }
  #[getter]
  fn get_thresh2       (&self)  -> f32  {
    self.thresh[2]
  }
}

//----------------------------------------

moniseries!(LTBMoniDataSeries, LTBMoniData);

#[cfg(feature="pybindings")]
pythonize_packable!(LTBMoniData);

#[cfg(feature="pybindings")]
pythonize_monidata!(LTBMoniData);

//----------------------------------------

#[test]
#[cfg(feature = "random")]
fn pack_ltbmonidata() {
  for _ in 0..100 {
    let data = LTBMoniData::from_random();
    let test : LTBMoniData = data.pack().unpack().unwrap();
    assert_eq!(data, test);
  }
}


