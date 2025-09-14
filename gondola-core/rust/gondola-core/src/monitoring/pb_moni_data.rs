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
use tof_control::helper::pb_type::{
  PBTemp,
  PBVcp
};

/// Sensors on the power boards (PB)
///
/// Each RAT has a single PB
#[derive(Debug, Copy, Clone, PartialEq)]
#[cfg_attr(feature="pybindings", pyclass)]
pub struct PBMoniData {
  pub board_id       : u8,
  pub p3v6_preamp_vcp: [f32; 3],
  pub n1v6_preamp_vcp: [f32; 3],
  pub p3v4f_ltb_vcp  : [f32; 3],
  pub p3v4d_ltb_vcp  : [f32; 3],
  pub p3v6_ltb_vcp   : [f32; 3],
  pub n1v6_ltb_vcp   : [f32; 3],
  pub pds_temp       : f32,
  pub pas_temp       : f32,
  pub nas_temp       : f32,
  pub shv_temp       : f32,
}

impl PBMoniData {
  pub fn new() -> Self {
    Self {
      board_id       : 0,
      p3v6_preamp_vcp: [f32::MAX, f32::MAX, f32::MAX],
      n1v6_preamp_vcp: [f32::MAX, f32::MAX, f32::MAX],
      p3v4f_ltb_vcp  : [f32::MAX, f32::MAX, f32::MAX],
      p3v4d_ltb_vcp  : [f32::MAX, f32::MAX, f32::MAX],
      p3v6_ltb_vcp   : [f32::MAX, f32::MAX, f32::MAX],
      n1v6_ltb_vcp   : [f32::MAX, f32::MAX, f32::MAX],
      pds_temp       : f32::MAX,
      pas_temp       : f32::MAX,
      nas_temp       : f32::MAX,
      shv_temp       : f32::MAX,
    }
  }
  
  #[cfg(feature = "tofcontrol")]
  pub fn add_temps(&mut self, pbtmp : &PBTemp) {
    self.pds_temp = pbtmp.pds_temp; 
    self.pas_temp = pbtmp.pas_temp; 
    self.nas_temp = pbtmp.nas_temp; 
    self.shv_temp = pbtmp.shv_temp; 
  }
  
  #[cfg(feature = "tofcontrol")]
  pub fn add_vcp(&mut self, pbvcp : &PBVcp) {
    self.p3v6_preamp_vcp = pbvcp.p3v6_pa_vcp; 
    self.n1v6_preamp_vcp = pbvcp.n1v6_pa_vcp;  
    self.p3v4f_ltb_vcp   = pbvcp.p3v4f_ltb_vcp;
    self.p3v4d_ltb_vcp   = pbvcp.p3v4d_ltb_vcp;
    self.p3v6_ltb_vcp    = pbvcp.p3v6_ltb_vcp;
    self.n1v6_ltb_vcp    = pbvcp.n1v6_ltb_vcp;
  }
}

#[cfg(feature="pybindings")]
#[pymethods]
impl PBMoniData {

  #[getter]
  fn get_board_id(&self) -> u8 {
    self.board_id
  }

  #[getter]
  fn get_p3v6_preamp_vcp(&self) -> [f32; 3] {
    self.p3v6_preamp_vcp
  }
  
  #[getter]
  fn get_n1v6_preamp_vcp(&self) -> [f32; 3] {
    self.n1v6_preamp_vcp
  }
  
  #[getter]
  fn get_p3v4f_ltb_vcp(&self) -> [f32; 3] {
    self.p3v4f_ltb_vcp
  }
  
  #[getter]
  fn get_p3v4d_ltb_vcp(&self) -> [f32; 3] {
    self.p3v4d_ltb_vcp
  }
  
  #[getter]
  fn get_p3v6_ltb_vcp(&self) -> [f32; 3] {
    self.p3v6_ltb_vcp
  }
  
  #[getter]
  fn get_n1v6_ltb_vcp(&self) -> [f32; 3] {
    self.n1v6_ltb_vcp
  }
  
  #[getter]
  fn get_pds_temp(&self) -> f32 {  
    self.pds_temp
  }
  #[getter]
  fn get_pas_temp(&self) -> f32 {
    self.pas_temp
  }
  #[getter]
  fn get_nas_temp(&self) -> f32 {
    self.nas_temp
  }

  #[getter]
  fn get_shv_temp(&self) -> f32 {
    self.shv_temp
  }
}

impl Default for PBMoniData {
  fn default() -> Self {
    Self::new()
  }
}

impl fmt::Display for PBMoniData {
  fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
    write!(f, "<PBMoniData:
  BOARD ID     :  {}
  ** Temperatures **
  PDS TMP      :  {:.2} [\u{00B0}C]
  PAS TMP      :  {:.2} [\u{00B0}C]
  NAS TMP      :  {:.2} [\u{00B0}C]
  SHV TMP      :  {:.2} [\u{00B0}C]
  ** Power **
  P3V6  Preamp :  {:.3}  [V] | {:.3} [A] | {:.3} [W]
  N1V6  Preamp : {:.3}  [V] | {:.3} [A] | {:.3} [W]
  P3V4f LTB    :  {:.3}  [V] | {:.3} [A] | {:.3} [W]
  P3V4d LTB    :  {:.3}  [V] | {:.3} [A] | {:.3} [W]
  P3V6  LTB    :  {:.3}  [V] | {:.3} [A] | {:.3} [W]
  N1V6  LTB    : {:.3}  [V] | {:.3} [A] | {:.3} [W]>",
           self.board_id       , 
           if self.pds_temp != f32::MAX {self.pds_temp.to_string()} else {String::from("f32::MAX (ERR)")},
           if self.pas_temp != f32::MAX {self.pas_temp.to_string()} else {String::from("f32::MAX (ERR)")},
           if self.nas_temp != f32::MAX {self.nas_temp.to_string()} else {String::from("f32::MAX (ERR)")},
           if self.shv_temp != f32::MAX {self.shv_temp.to_string()} else {String::from("f32::MAX (ERR)")},
           if self.p3v6_preamp_vcp[0] != f32::MAX {self.p3v6_preamp_vcp[0].to_string()} else {String::from("f32::MAX (ERR)")},
           if self.p3v6_preamp_vcp[1] != f32::MAX {self.p3v6_preamp_vcp[1].to_string()} else {String::from("f32::MAX (ERR)")},
           if self.p3v6_preamp_vcp[2] != f32::MAX {self.p3v6_preamp_vcp[2].to_string()} else {String::from("f32::MAX (ERR)")},
           if self.n1v6_preamp_vcp[0] != f32::MAX {self.n1v6_preamp_vcp[0].to_string()} else {String::from("f32::MAX (ERR)")},
           if self.n1v6_preamp_vcp[1] != f32::MAX {self.n1v6_preamp_vcp[1].to_string()} else {String::from("f32::MAX (ERR)")},
           if self.n1v6_preamp_vcp[2] != f32::MAX {self.n1v6_preamp_vcp[2].to_string()} else {String::from("f32::MAX (ERR)")},
           if self.p3v4f_ltb_vcp[0]   != f32::MAX {self.p3v4f_ltb_vcp[0].to_string()  } else {String::from("f32::MAX (ERR)")},
           if self.p3v4f_ltb_vcp[1]   != f32::MAX {self.p3v4f_ltb_vcp[1].to_string()  } else {String::from("f32::MAX (ERR)")},
           if self.p3v4f_ltb_vcp[2]   != f32::MAX {self.p3v4f_ltb_vcp[2].to_string()  } else {String::from("f32::MAX (ERR)")},
           if self.p3v4d_ltb_vcp[0]   != f32::MAX {self.p3v4d_ltb_vcp[0].to_string()  } else {String::from("f32::MAX (ERR)")},
           if self.p3v4d_ltb_vcp[1]   != f32::MAX {self.p3v4d_ltb_vcp[1].to_string()  } else {String::from("f32::MAX (ERR)")},
           if self.p3v4d_ltb_vcp[2]   != f32::MAX {self.p3v4d_ltb_vcp[2].to_string()  } else {String::from("f32::MAX (ERR)")},
           if self.p3v6_ltb_vcp[0]    != f32::MAX {self.p3v6_ltb_vcp[0].to_string()   } else {String::from("f32::MAX (ERR)")},
           if self.p3v6_ltb_vcp[1]    != f32::MAX {self.p3v6_ltb_vcp[1].to_string()   } else {String::from("f32::MAX (ERR)")},
           if self.p3v6_ltb_vcp[2]    != f32::MAX {self.p3v6_ltb_vcp[2].to_string()   } else {String::from("f32::MAX (ERR)")},
           if self.n1v6_ltb_vcp[0]    != f32::MAX {self.n1v6_ltb_vcp[0].to_string()   } else {String::from("f32::MAX (ERR)")},
           if self.n1v6_ltb_vcp[1]    != f32::MAX {self.n1v6_ltb_vcp[1].to_string()   } else {String::from("f32::MAX (ERR)")},
           if self.n1v6_ltb_vcp[2]    != f32::MAX {self.n1v6_ltb_vcp[2].to_string()   } else {String::from("f32::MAX (ERR)")})
  }
}

impl TofPackable for PBMoniData {
  const TOF_PACKET_TYPE : TofPacketType = TofPacketType::PBMoniData;
}

impl MoniData for PBMoniData {

  fn get_board_id(&self) -> u8 {
    self.board_id 
  }
  
  fn get(&self, varname : &str) -> Option<f32> {
    match varname {
      "board_id"      => Some(0.0f32),
      "p3v6_preamp_v" => Some(self.p3v6_preamp_vcp[0]), 
      "p3v6_preamp_c" => Some(self.p3v6_preamp_vcp[1]), 
      "p3v6_preamp_p" => Some(self.p3v6_preamp_vcp[2]), 
      "n1v6_preamp_v" => Some(self.n1v6_preamp_vcp[0]), 
      "n1v6_preamp_c" => Some(self.n1v6_preamp_vcp[1]), 
      "n1v6_preamp_p" => Some(self.n1v6_preamp_vcp[2]), 
      "p3v4f_ltb_v"   => Some(self.p3v4f_ltb_vcp[0]), 
      "p3v4f_ltb_c"   => Some(self.p3v4f_ltb_vcp[1]), 
      "p3v4f_ltb_p"   => Some(self.p3v4f_ltb_vcp[2]), 
      "p3v4d_ltb_v"   => Some(self.p3v4d_ltb_vcp[0]), 
      "p3v4d_ltb_c"   => Some(self.p3v4d_ltb_vcp[1]), 
      "p3v4d_ltb_p"   => Some(self.p3v4d_ltb_vcp[2]), 
      "p3v6_ltb_v"    => Some(self.p3v6_ltb_vcp[0]), 
      "p3v6_ltb_c"    => Some(self.p3v6_ltb_vcp[1]), 
      "p3v6_ltb_p"    => Some(self.p3v6_ltb_vcp[2]), 
      "n1v6_ltb_v"    => Some(self.n1v6_ltb_vcp[0]), 
      "n1v6_ltb_c"    => Some(self.n1v6_ltb_vcp[1]), 
      "n1v6_ltb_p"    => Some(self.n1v6_ltb_vcp[2]), 
      "pds_temp"      => Some(self.pds_temp),
      "pas_temp"      => Some(self.pas_temp),
      "nas_temp"      => Some(self.nas_temp),
      "shv_temp"      => Some(self.shv_temp),
      _               => None, 
    }
  }

  fn keys() -> Vec<&'static str> {
    vec!["board_id",
         "p3v6_preamp_v", "p3v6_preamp_c", "p3v6_preamp_p",
         "n1v6_preamp_v", "n1v6_preamp_c", "n1v6_preamp_p",
         "p3v4f_ltb_v", "p3v4f_ltb_c", "p3v4f_ltb_p",
         "p3v4d_ltb_v", "p3v4d_ltb_c", "p3v4d_ltb_p",
         "p3v6_ltb_v", "p3v6_ltb_c", "p3v6_ltb_p",
         "n1v6_ltb_v", "n1v6_ltb_c", "n1v6_ltb_p",
         "pds_temp", "pas_temp", "nas_temp", "shv_temp"]
  }
}


impl Serialization for PBMoniData {
  const HEAD : u16 = 0xAAAA;
  const TAIL : u16 = 0x5555;
  /// The data size when serialized to a bytestream
  /// This needs to be updated when we change the 
  /// packet layout, e.g. add new members.
  /// HEAD + TAIL + sum(sizeof(m) for m in _all_members_))
  const SIZE : usize  = 89 + 4; // 4 header + footer
  
  fn to_bytestream(&self) -> Vec<u8> {
    let mut stream = Vec::<u8>::with_capacity(Self::SIZE);
    stream.extend_from_slice(&Self::HEAD.to_le_bytes());
    stream.extend_from_slice(&self.board_id          .to_le_bytes());
    stream.extend_from_slice(&self.p3v6_preamp_vcp[0].to_le_bytes());
    stream.extend_from_slice(&self.p3v6_preamp_vcp[1].to_le_bytes());
    stream.extend_from_slice(&self.p3v6_preamp_vcp[2].to_le_bytes());
    stream.extend_from_slice(&self.n1v6_preamp_vcp[0].to_le_bytes());
    stream.extend_from_slice(&self.n1v6_preamp_vcp[1].to_le_bytes());
    stream.extend_from_slice(&self.n1v6_preamp_vcp[2].to_le_bytes());
    stream.extend_from_slice(&self.p3v4f_ltb_vcp[0]  .to_le_bytes());
    stream.extend_from_slice(&self.p3v4f_ltb_vcp[1]  .to_le_bytes());
    stream.extend_from_slice(&self.p3v4f_ltb_vcp[2]  .to_le_bytes());
    stream.extend_from_slice(&self.p3v4d_ltb_vcp[0]  .to_le_bytes());
    stream.extend_from_slice(&self.p3v4d_ltb_vcp[1]  .to_le_bytes());
    stream.extend_from_slice(&self.p3v4d_ltb_vcp[2]  .to_le_bytes());
    stream.extend_from_slice(&self.p3v6_ltb_vcp[0]   .to_le_bytes());
    stream.extend_from_slice(&self.p3v6_ltb_vcp[1]   .to_le_bytes());
    stream.extend_from_slice(&self.p3v6_ltb_vcp[2]   .to_le_bytes());
    stream.extend_from_slice(&self.n1v6_ltb_vcp[0]   .to_le_bytes());
    stream.extend_from_slice(&self.n1v6_ltb_vcp[1]   .to_le_bytes());
    stream.extend_from_slice(&self.n1v6_ltb_vcp[2]   .to_le_bytes());
    stream.extend_from_slice(&self.pds_temp          .to_le_bytes());
    stream.extend_from_slice(&self.pas_temp          .to_le_bytes());
    stream.extend_from_slice(&self.nas_temp          .to_le_bytes());
    stream.extend_from_slice(&self.shv_temp          .to_le_bytes());
    stream.extend_from_slice(&Self::TAIL.to_le_bytes());
    stream
  } 

  fn from_bytestream(stream    : &Vec<u8>, 
                     pos       : &mut usize) 
    -> Result<PBMoniData, SerializationError>{
    Self::verify_fixed(stream, pos)?;
    let mut moni            = PBMoniData::new();
    moni.board_id           = parse_u8(stream, pos) ; 
    moni.p3v6_preamp_vcp[0] = parse_f32(stream, pos);
    moni.p3v6_preamp_vcp[1] = parse_f32(stream, pos);
    moni.p3v6_preamp_vcp[2] = parse_f32(stream, pos);
    moni.n1v6_preamp_vcp[0] = parse_f32(stream, pos);
    moni.n1v6_preamp_vcp[1] = parse_f32(stream, pos);
    moni.n1v6_preamp_vcp[2] = parse_f32(stream, pos);
    moni.p3v4f_ltb_vcp[0]   = parse_f32(stream, pos);
    moni.p3v4f_ltb_vcp[1]   = parse_f32(stream, pos);
    moni.p3v4f_ltb_vcp[2]   = parse_f32(stream, pos);
    moni.p3v4d_ltb_vcp[0]   = parse_f32(stream, pos);
    moni.p3v4d_ltb_vcp[1]   = parse_f32(stream, pos);
    moni.p3v4d_ltb_vcp[2]   = parse_f32(stream, pos);
    moni.p3v6_ltb_vcp[0]    = parse_f32(stream, pos);
    moni.p3v6_ltb_vcp[1]    = parse_f32(stream, pos);
    moni.p3v6_ltb_vcp[2]    = parse_f32(stream, pos);
    moni.n1v6_ltb_vcp[0]    = parse_f32(stream, pos);
    moni.n1v6_ltb_vcp[1]    = parse_f32(stream, pos);
    moni.n1v6_ltb_vcp[2]    = parse_f32(stream, pos);
    moni.pds_temp           = parse_f32(stream, pos);
    moni.pas_temp           = parse_f32(stream, pos);
    moni.nas_temp           = parse_f32(stream, pos);
    moni.shv_temp           = parse_f32(stream, pos);
    *pos += 2;// account for tail
    Ok(moni)
  }
}

#[cfg(feature = "random")]
impl FromRandom for PBMoniData {
    
  fn from_random() -> PBMoniData {
    let mut moni = Self::new();
    let mut rng = rand::rng();
    moni.board_id           = rng.random::<u8>(); 
    for k in 0..3 {
      let foo = rng.random::<f32>();
      moni.p3v6_preamp_vcp[k] = foo;
    }
    for k in 0..3 {
      let foo = rng.random::<f32>();
      moni.n1v6_preamp_vcp[k] = foo;
    }
    for k in 0..3 {
      let foo = rng.random::<f32>();
      moni.p3v4f_ltb_vcp[k] = foo;
    }
    for k in 0..3 {
      let foo = rng.random::<f32>();
      moni.p3v4d_ltb_vcp[k] = foo;
    }
    for k in 0..3 {
      let foo = rng.random::<f32>();
      moni.p3v6_ltb_vcp[k] = foo;
    }
    for k in 0..3 {
      let foo = rng.random::<f32>();
      moni.n1v6_ltb_vcp[k] = foo;
    }
    moni.pds_temp = rng.random::<f32>(); 
    moni.pas_temp = rng.random::<f32>(); 
    moni.nas_temp = rng.random::<f32>(); 
    moni.shv_temp = rng.random::<f32>(); 
    moni
  }
}

//----------------------------------------

moniseries!(PBMoniDataSeries, PBMoniData);

#[cfg(feature="pybindings")]
pythonize_packable!(PBMoniData);

#[cfg(feature="pybindings")]
pythonize_monidata!(PBMoniData);

//----------------------------------------

#[test]
#[cfg(feature = "random")]
fn pack_pbmonidata() {
  for _ in 0..100 {
    let data = PBMoniData::from_random();
    let test : PBMoniData = data.pack().unpack().unwrap();
    assert_eq!(data, test);
  }
}

