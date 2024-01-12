//! Tof housekeeping/monitoring
//!
//! Contains structs to hold monitoring
//! information for the different parts
//! of the TOF, e.g. RB,LTB,MTB
//!
//! An overview of the sensors in the 
//! GAPS TOF can be found in the 
//! [GAPS wiki](https://gaps1.astro.ucla.edu/wiki/gaps/index.php?title=TOF_environmental_sensors)

use std::fmt;

// Takeru's tof-control code
#[cfg(feature = "tof-control")]
use tof_control::helper::pb_type::{
    PBTemp,
    PBVcp,
};

#[cfg(feature = "tof-control")]
use tof_control::helper::preamp_type::{
    PreampTemp,
    PreampReadBias,
};

#[cfg(feature = "tof-control")]
use tof_control::helper::ltb_type::{
    LTBTemp,
    LTBThreshold,
};

#[cfg(feature = "tof-control")]
use tof_control::helper::rb_type::{
    RBTempDebug,
    RBMag,
    RBVcp,
    RBPh,
};

cfg_if::cfg_if! {
  if #[cfg(feature = "random")]  {
    use crate::FromRandom;
    extern crate rand;
    use rand::Rng;
  }
}
#[cfg(feature = "tof-control")]
use tof_control::helper::cpu_type::{
    CPUTempDebug,
    CPUInfo,
};


use crate::serialization::{
    Serialization,
    SerializationError,
    parse_u8,
    parse_u16,
    parse_u32,
    parse_f32
};


/// Sensors on the power boards (PB)
///
/// Each RAT has a single PB
#[derive(Debug, Copy, Clone, PartialEq)]
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
  
  #[cfg(feature = "tof-control")]
  pub fn add_temps(&mut self, pbtmp : &PBTemp) {
    self.pds_temp = pbtmp.pds_temp; 
    self.pas_temp = pbtmp.pas_temp; 
    self.nas_temp = pbtmp.nas_temp; 
    self.shv_temp = pbtmp.shv_temp; 
  }
  
  #[cfg(feature = "tof-control")]
  pub fn add_vcp(&mut self, pbvcp : &PBVcp) {
    self.p3v6_preamp_vcp = pbvcp.p3v6_preamp_vcp; 
    self.n1v6_preamp_vcp = pbvcp.n1v6_preamp_vcp;  
    self.p3v4f_ltb_vcp   = pbvcp.p3v4f_ltb_vcp;
    self.p3v4d_ltb_vcp   = pbvcp.p3v4d_ltb_vcp;
    self.p3v6_ltb_vcp    = pbvcp.p3v6_ltb_vcp;
    self.n1v6_ltb_vcp    = pbvcp.n1v6_ltb_vcp;
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

#[cfg(feature = "random")]
impl FromRandom for PBMoniData {
    
  fn from_random() -> PBMoniData {
    let mut moni = Self::new();
    let mut rng = rand::thread_rng();
    moni.board_id           = rng.gen::<u8>(); 
    for k in 0..3 {
      let foo = rng.gen::<f32>();
      moni.p3v6_preamp_vcp[k] = foo;
    }
    for k in 0..3 {
      let foo = rng.gen::<f32>();
      moni.n1v6_preamp_vcp[k] = foo;
    }
    for k in 0..3 {
      let foo = rng.gen::<f32>();
      moni.p3v4f_ltb_vcp[k] = foo;
    }
    for k in 0..3 {
      let foo = rng.gen::<f32>();
      moni.p3v4d_ltb_vcp[k] = foo;
    }
    for k in 0..3 {
      let foo = rng.gen::<f32>();
      moni.p3v6_ltb_vcp[k] = foo;
    }
    for k in 0..3 {
      let foo = rng.gen::<f32>();
      moni.n1v6_ltb_vcp[k] = foo;
    }
    moni.pds_temp = rng.gen::<f32>(); 
    moni.pas_temp = rng.gen::<f32>(); 
    moni.nas_temp = rng.gen::<f32>(); 
    moni.shv_temp = rng.gen::<f32>(); 
    moni
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

///////////////////////////////////////////////////////

/// Preamp temperature and bias data
#[derive(Debug, Copy, Clone, PartialEq)]
pub struct PAMoniData {
  pub board_id           : u8,
  pub temps              : [f32;16],
  pub biases             : [f32;16]
}

impl PAMoniData {

  pub fn new() -> Self {
    Self {
      board_id  : 0,
      temps     : [f32::MAX;16],
      biases    : [f32::MAX;16]
    }
  }

  #[cfg(feature = "tof-control")]
  pub fn add_temps(&mut self, pt : &PreampTemp ) {
    self.temps = pt.preamp_temps;
  }

  #[cfg(feature = "tof-control")]
  pub fn add_biases(&mut self, pb : &PreampReadBias) {
    self.biases = pb.read_biases;
  }
}

impl Default for PAMoniData {
  fn default() -> Self {
    Self::new()
  }
}

impl fmt::Display for PAMoniData {
  fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
    write!(f, "<PAMoniData:
  Board ID : {}
  **16 Temp values**
  T1   : {:.2} [\u{00B0}C]
  T2   : {:.2} [\u{00B0}C]
  T3   : {:.2} [\u{00B0}C]
  T4   : {:.2} [\u{00B0}C]
  T5   : {:.2} [\u{00B0}C]
  T6   : {:.2} [\u{00B0}C]
  T7   : {:.2} [\u{00B0}C]
  T8   : {:.2} [\u{00B0}C]
  T9   : {:.2} [\u{00B0}C]
  T10  : {:.2} [\u{00B0}C]
  T11  : {:.2} [\u{00B0}C]
  T12  : {:.2} [\u{00B0}C]
  T13  : {:.2} [\u{00B0}C]
  T14  : {:.2} [\u{00B0}C]
  T15  : {:.2} [\u{00B0}C]
  T16  : {:.2} [\u{00B0}C]
  **16 Bias voltages**
  V1   : {:.3} [V]
  V2   : {:.3} [V]
  V3   : {:.3} [V]
  V4   : {:.3} [V]
  V5   : {:.3} [V]
  V6   : {:.3} [V]
  V7   : {:.3} [V]
  V8   : {:.3} [V]
  V9   : {:.3} [V]
  V10  : {:.3} [V]
  V11  : {:.3} [V]
  V12  : {:.3} [V]
  V13  : {:.3} [V]
  V14  : {:.3} [V]
  V15  : {:.3} [V]
  V16  : {:.3} [V]>",
  self.board_id,
  if self.temps[0]  != f32::MAX {self.temps[0 ].to_string()} else {String::from("f32::MAX (ERR)")},
  if self.temps[1]  != f32::MAX {self.temps[1 ].to_string()} else {String::from("f32::MAX (ERR)")},
  if self.temps[2]  != f32::MAX {self.temps[2 ].to_string()} else {String::from("f32::MAX (ERR)")},
  if self.temps[3]  != f32::MAX {self.temps[3 ].to_string()} else {String::from("f32::MAX (ERR)")},
  if self.temps[4]  != f32::MAX {self.temps[4 ].to_string()} else {String::from("f32::MAX (ERR)")},
  if self.temps[5]  != f32::MAX {self.temps[5 ].to_string()} else {String::from("f32::MAX (ERR)")},
  if self.temps[6]  != f32::MAX {self.temps[6 ].to_string()} else {String::from("f32::MAX (ERR)")},
  if self.temps[7]  != f32::MAX {self.temps[7 ].to_string()} else {String::from("f32::MAX (ERR)")},
  if self.temps[8]  != f32::MAX {self.temps[8 ].to_string()} else {String::from("f32::MAX (ERR)")},
  if self.temps[9]  != f32::MAX {self.temps[9 ].to_string()} else {String::from("f32::MAX (ERR)")},
  if self.temps[10] != f32::MAX {self.temps[10].to_string()} else {String::from("f32::MAX (ERR)")},
  if self.temps[11] != f32::MAX {self.temps[11].to_string()} else {String::from("f32::MAX (ERR)")},
  if self.temps[12] != f32::MAX {self.temps[12].to_string()} else {String::from("f32::MAX (ERR)")},
  if self.temps[13] != f32::MAX {self.temps[13].to_string()} else {String::from("f32::MAX (ERR)")},
  if self.temps[14] != f32::MAX {self.temps[14].to_string()} else {String::from("f32::MAX (ERR)")},
  if self.temps[15] != f32::MAX {self.temps[15].to_string()} else {String::from("f32::MAX (ERR)")},
  self.biases[0],
  self.biases[1],
  self.biases[2],
  self.biases[3],
  self.biases[4],
  self.biases[5],
  self.biases[6],
  self.biases[7],
  self.biases[8],
  self.biases[9],
  self.biases[10],
  self.biases[11],
  self.biases[12],
  self.biases[13],
  self.biases[14],
  self.biases[15])
  }
}

#[cfg(feature = "random")]
impl FromRandom for PAMoniData {
    
  fn from_random() -> Self {
    let mut moni = Self::new();
    let mut rng = rand::thread_rng();
    moni.board_id     = rng.gen::<u8>(); 
    for k in 0..16 {
      moni.temps[k]   = rng.gen::<f32>(); 
    }
    for k in 0..16 {
      moni.biases[k]  = rng.gen::<f32>(); 
    }
    moni
  }
}

impl Serialization for PAMoniData {
  
  const HEAD : u16 = 0xAAAA;
  const TAIL : u16 = 0x5555;
  /// The data size when serialized to a bytestream
  /// This needs to be updated when we change the 
  /// packet layout, e.g. add new members.
  /// HEAD + TAIL + sum(sizeof(m) for m in _all_members_))
  const SIZE : usize  = 4 + 1 + (4*16*2);
  
  fn to_bytestream(&self) -> Vec<u8> {
    let mut stream = Vec::<u8>::with_capacity(Self::SIZE);
    stream.extend_from_slice(&Self::HEAD.to_le_bytes());
    stream.extend_from_slice(&self.board_id.to_le_bytes()); 
    for k in 0..16 {
      stream.extend_from_slice(&self.temps[k].to_le_bytes());
    }
    for k in 0..16 {
      stream.extend_from_slice(&self.biases[k].to_le_bytes());
    }
    stream.extend_from_slice(&Self::TAIL.to_le_bytes());
    stream
  }
  
  fn from_bytestream(stream : &Vec<u8>, pos : &mut usize)
    -> Result<Self, SerializationError> {
    let mut moni_data      = Self::new();
    Self::verify_fixed(stream, pos)?;
    moni_data.board_id = parse_u8(stream, pos);
    for k in 0..16 {
      moni_data.temps[k] = parse_f32(stream, pos);
    }
    for k in 0..16 {
      moni_data.biases[k] = parse_f32(stream, pos);
    }
    *pos += 2;
    Ok(moni_data)
  }
}

///////////////////////////////////////////////////////

/// Sensors on the LTB
#[derive(Debug, Copy, Clone, PartialEq)]
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

  #[cfg(feature = "tof-control")]
  pub fn add_temps(&mut self, lt : &LTBTemp) {
    self.trenz_temp = lt.trenz_temp;
    self.ltb_temp   = lt.board_temp;
  }

  #[cfg(feature = "tof-control")]
  pub fn add_thresh(&mut self, lt : &LTBThreshold) {
    self.thresh = lt.thresholds;
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
  THR1, THR2, THR3 : {:.3} | {:.3} | {:.3} [V]>",
  self.board_id,
  self.trenz_temp,
  self.ltb_temp,
  self.thresh[0],
  self.thresh[1],
  self.thresh[2])
  }
}

#[cfg(feature = "random")]
impl FromRandom for LTBMoniData {
    
  fn from_random() -> LTBMoniData {
    let mut moni  = Self::new();
    let mut rng   = rand::thread_rng();
    moni.board_id = rng.gen::<u8>(); 
    moni.trenz_temp = rng.gen::<f32>();
    moni.ltb_temp   = rng.gen::<f32>();
    for k in 0..3 {
      moni.thresh[k] = rng.gen::<f32>();
    }
    moni
  }
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

///////////////////////////////////////////////////////


/// Sensors on the individual RB
///  
/// This includes temperatures, power data,
/// pressure, humidity
/// as well as the magnetic sensors
#[derive(Debug, Copy, Clone, PartialEq)]
pub struct RBMoniData {
  pub board_id           : u8,
  pub rate               : u16,
  pub tmp_drs            : f32,
  pub tmp_clk            : f32,
  pub tmp_adc            : f32,
  pub tmp_zynq           : f32,
  pub tmp_lis3mdltr      : f32,
  pub tmp_bm280          : f32,
  pub pressure           : f32,
  pub humidity           : f32,
  pub mag_x              : f32,
  pub mag_y              : f32,
  pub mag_z              : f32,
  pub drs_dvdd_voltage   : f32, 
  pub drs_dvdd_current   : f32,
  pub drs_dvdd_power     : f32,
  pub p3v3_voltage       : f32,
  pub p3v3_current       : f32,
  pub p3v3_power         : f32,
  pub zynq_voltage       : f32,
  pub zynq_current       : f32,
  pub zynq_power         : f32,
  pub p3v5_voltage       : f32, 
  pub p3v5_current       : f32,
  pub p3v5_power         : f32,
  pub adc_dvdd_voltage   : f32,
  pub adc_dvdd_current   : f32,
  pub adc_dvdd_power     : f32,
  pub adc_avdd_voltage   : f32,
  pub adc_avdd_current   : f32,
  pub adc_avdd_power     : f32,
  pub drs_avdd_voltage   : f32, 
  pub drs_avdd_current   : f32,
  pub drs_avdd_power     : f32,
  pub n1v5_voltage       : f32,
  pub n1v5_current       : f32,
  pub n1v5_power         : f32,
}

impl RBMoniData {

  #[cfg(feature = "tof-control")]
  pub fn add_rbtemp(&mut self, rb_temp : &RBTempDebug) {
    self.tmp_drs         = rb_temp.drs_temp      ; 
    self.tmp_clk         = rb_temp.clk_temp      ; 
    self.tmp_adc         = rb_temp.adc_temp      ; 
    self.tmp_zynq        = rb_temp.zynq_temp     ; 
    //FIXME - this is on tof-control
    self.tmp_lis3mdltr   = rb_temp.lis3mdltr_temp; 
    self.tmp_bm280       = rb_temp.bme280_temp   ; 
  }

  #[cfg(feature = "tof-control")] 
  pub fn add_rbmag(&mut self, rb_mag   : &RBMag) {
    self.mag_x   = rb_mag.mag_xyz[0];
    self.mag_y   = rb_mag.mag_xyz[1];
    self.mag_z   = rb_mag.mag_xyz[2];
  }
 
  pub fn get_mag_tot(&self) -> f32 {
    (self.mag_x.powi(2) + self.mag_y.powi(2) + self.mag_z.powi(2)).sqrt()
  }


  #[cfg(feature = "tof-control")]
  pub fn add_rbvcp(&mut self, rb_vcp   : &RBVcp) {
    self.drs_dvdd_voltage = rb_vcp.drs_dvdd_vcp[0] ;
    self.drs_dvdd_current = rb_vcp.drs_dvdd_vcp[1] ;
    self.drs_dvdd_power   = rb_vcp.drs_dvdd_vcp[2] ;
    self.p3v3_voltage     = rb_vcp.p3v3_vcp[0]  ;
    self.p3v3_current     = rb_vcp.p3v3_vcp[1]  ;
    self.p3v3_power       = rb_vcp.p3v3_vcp[2]  ;
    self.zynq_voltage     = rb_vcp.zynq_vcp[0]  ;
    self.zynq_current     = rb_vcp.zynq_vcp[1]  ;
    self.zynq_power       = rb_vcp.zynq_vcp[2]  ;
    self.p3v5_voltage     = rb_vcp.p3v5_vcp[0]  ;
    self.p3v5_current     = rb_vcp.p3v5_vcp[1]  ;
    self.p3v5_power       = rb_vcp.p3v5_vcp[2]  ;
    self.adc_dvdd_voltage = rb_vcp.adc_dvdd_vcp[0] ;
    self.adc_dvdd_current = rb_vcp.adc_dvdd_vcp[1] ;
    self.adc_dvdd_power   = rb_vcp.adc_dvdd_vcp[2] ;
    self.adc_avdd_voltage = rb_vcp.adc_avdd_vcp[0]  ;
    self.adc_avdd_current = rb_vcp.adc_avdd_vcp[1]  ;
    self.adc_avdd_power   = rb_vcp.adc_avdd_vcp[2]  ;
    self.drs_avdd_voltage = rb_vcp.drs_avdd_vcp[0]  ;
    self.drs_avdd_current = rb_vcp.drs_avdd_vcp[1]  ;
    self.drs_avdd_power   = rb_vcp.drs_avdd_vcp[2]  ;
    self.n1v5_voltage     = rb_vcp.n1v5_vcp[0]      ;
    self.n1v5_current     = rb_vcp.n1v5_vcp[1]      ;
    self.n1v5_power       = rb_vcp.n1v5_vcp[2]      ;
  }
  
  #[cfg(feature = "tof-control")] 
  pub fn add_rbph(&mut self, rb_ph   : &RBPh) {
    self.pressure = rb_ph.pressure;
    self.humidity = rb_ph.humidity;
  }

  pub fn new() -> Self {
    Self {
      board_id           : 0, 
      rate               : 0,
      tmp_drs            : f32::MAX,
      tmp_clk            : f32::MAX,
      tmp_adc            : f32::MAX,
      tmp_zynq           : f32::MAX,
      tmp_lis3mdltr      : f32::MAX,
      tmp_bm280          : f32::MAX,
      pressure           : f32::MAX,
      humidity           : f32::MAX,
      mag_x              : f32::MAX,
      mag_y              : f32::MAX,
      mag_z              : f32::MAX,
      drs_dvdd_voltage   : f32::MAX, 
      drs_dvdd_current   : f32::MAX,
      drs_dvdd_power     : f32::MAX,
      p3v3_voltage       : f32::MAX,
      p3v3_current       : f32::MAX,
      p3v3_power         : f32::MAX,
      zynq_voltage       : f32::MAX,
      zynq_current       : f32::MAX,
      zynq_power         : f32::MAX,
      p3v5_voltage       : f32::MAX, 
      p3v5_current       : f32::MAX,
      p3v5_power         : f32::MAX,
      adc_dvdd_voltage   : f32::MAX,
      adc_dvdd_current   : f32::MAX,
      adc_dvdd_power     : f32::MAX,
      adc_avdd_voltage   : f32::MAX,
      adc_avdd_current   : f32::MAX,
      adc_avdd_power     : f32::MAX,
      drs_avdd_voltage   : f32::MAX, 
      drs_avdd_current   : f32::MAX,
      drs_avdd_power     : f32::MAX,
      n1v5_voltage       : f32::MAX,
      n1v5_current       : f32::MAX,
      n1v5_power         : f32::MAX,
    }
  }
}

impl Default for RBMoniData {
  fn default() -> Self {
    Self::new()
  }
}

impl fmt::Display for RBMoniData {
  fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
    write!(f, "<RBMoniData:
  BOARD ID       {}
  RATE           {}    [Hz] 
  ** Temperatures **
  DRS TMP        {:.3} [\u{00B0}C]
  CLK TMP        {:.3} [\u{00B0}C]
  ADC TMP        {:.3} [\u{00B0}C]
  ZYNQ TMP       {:.3} [\u{00B0}C]
  LIS3MDLTR TMP  {:.3} [\u{00B0}C]  
  BM280 TMP      {:.3} [\u{00B0}C]
  ** Ambience **
  PRESSURE       {:.3} [hPa]
  HUMIDITY       {:.3} [%]
  MAG_X , MAG_Y, MAG_Z, MAG_TOT:
   |->  {:.3} [G] | {:.3} [G] | {:.3} [G] | {:.3} [G]
  ** Power **
  ZYNQ 3.3V         Power:  {:.3}  [V] | {:.3} [A] | {:.3} [W]
  3.3V              Power:  {:.3}  [V] | {:.3} [A] | {:.3} [W]
  3.5V              Power:  {:.3}  [V] | {:.3} [A] | {:.3} [W]
  -1.5V             Power: {:.3}  [V] | {:.3} [A] | {:.3} [W]
  DRS4 Digital 2.5V Power:  {:.3}  [V] | {:.3} [A] | {:.3} [W]
  DRS4 Analog 2.5V  Power:  {:.3}  [V] | {:.3} [A] | {:.3} [W]
  ADC Digital 2.5V  Power:  {:.3}  [V] | {:.3} [A] | {:.3} [W]
  ADC Analog 3.0V   Power:  {:.3}  [V] | {:.3} [A] | {:.3} [W]>",
           self.board_id        , 
           self.rate            ,
           self.tmp_drs         ,
           self.tmp_clk         ,
           self.tmp_adc         ,
           self.tmp_zynq        ,
           self.tmp_lis3mdltr   ,
           self.tmp_bm280       ,
           self.pressure        ,
           self.humidity        ,
           self.mag_x           ,
           self.mag_y           ,
           self.mag_z           ,
           self.get_mag_tot()   ,
           self.zynq_voltage    ,
           self.zynq_current    ,
           self.zynq_power      ,
           self.p3v3_voltage    ,
           self.p3v3_current    ,
           self.p3v3_power      ,
           self.p3v5_voltage    , 
           self.p3v5_current    ,
           self.p3v5_power      ,
           self.n1v5_voltage    ,
           self.n1v5_current    ,
           self.n1v5_power      ,
           self.drs_dvdd_voltage, 
           self.drs_dvdd_current,
           self.drs_dvdd_power  ,
           self.drs_avdd_voltage, 
           self.drs_avdd_current,
           self.drs_avdd_power  ,
           self.adc_dvdd_voltage,
           self.adc_dvdd_current,
           self.adc_dvdd_power  ,
           self.adc_avdd_voltage,
           self.adc_avdd_current,
           self.adc_avdd_power  )
  }
}

#[cfg(feature = "random")]
impl FromRandom for RBMoniData {
    
  fn from_random() -> RBMoniData {
    let mut moni = RBMoniData::new();
    let mut rng = rand::thread_rng();
    moni.board_id           = rng.gen::<u8>(); 
    moni.rate               = rng.gen::<u16>();
    moni.tmp_drs            = rng.gen::<f32>();
    moni.tmp_clk            = rng.gen::<f32>();
    moni.tmp_adc            = rng.gen::<f32>();
    moni.tmp_zynq           = rng.gen::<f32>();
    moni.tmp_lis3mdltr      = rng.gen::<f32>();
    moni.tmp_bm280          = rng.gen::<f32>();
    moni.pressure           = rng.gen::<f32>();
    moni.humidity           = rng.gen::<f32>();
    moni.mag_x              = rng.gen::<f32>();
    moni.mag_y              = rng.gen::<f32>();
    moni.mag_z              = rng.gen::<f32>();
    moni.drs_dvdd_voltage   = rng.gen::<f32>(); 
    moni.drs_dvdd_current   = rng.gen::<f32>();
    moni.drs_dvdd_power     = rng.gen::<f32>();
    moni.p3v3_voltage       = rng.gen::<f32>();
    moni.p3v3_current       = rng.gen::<f32>();
    moni.p3v3_power         = rng.gen::<f32>();
    moni.zynq_voltage       = rng.gen::<f32>();
    moni.zynq_current       = rng.gen::<f32>();
    moni.zynq_power         = rng.gen::<f32>();
    moni.p3v5_voltage       = rng.gen::<f32>(); 
    moni.p3v5_current       = rng.gen::<f32>();
    moni.p3v5_power         = rng.gen::<f32>();
    moni.adc_dvdd_voltage   = rng.gen::<f32>();
    moni.adc_dvdd_current   = rng.gen::<f32>();
    moni.adc_dvdd_power     = rng.gen::<f32>();
    moni.adc_avdd_voltage   = rng.gen::<f32>();
    moni.adc_avdd_current   = rng.gen::<f32>();
    moni.adc_avdd_power     = rng.gen::<f32>();
    moni.drs_avdd_voltage   = rng.gen::<f32>(); 
    moni.drs_avdd_current   = rng.gen::<f32>();
    moni.drs_avdd_power     = rng.gen::<f32>();
    moni.n1v5_voltage       = rng.gen::<f32>();
    moni.n1v5_current       = rng.gen::<f32>();
    moni.n1v5_power         = rng.gen::<f32>();
    moni
  }
}


impl Serialization for RBMoniData {
  
  const HEAD : u16 = 0xAAAA;
  const TAIL : u16 = 0x5555;
  /// The data size when serialized to a bytestream
  /// This needs to be updated when we change the 
  /// packet layout, e.g. add new members.
  /// HEAD + TAIL + sum(sizeof(m) for m in _all_members_))
  const SIZE : usize  = 7 + (36*4) ;
  
  fn to_bytestream(&self) -> Vec<u8> {
    let mut stream = Vec::<u8>::with_capacity(Self::SIZE);
    stream.extend_from_slice(&Self::HEAD.to_le_bytes());
    stream.extend_from_slice(&self.board_id          .to_le_bytes()); 
    stream.extend_from_slice(&self.rate              .to_le_bytes()); 
    stream.extend_from_slice(&self.tmp_drs           .to_le_bytes()); 
    stream.extend_from_slice(&self.tmp_clk           .to_le_bytes()); 
    stream.extend_from_slice(&self.tmp_adc           .to_le_bytes()); 
    stream.extend_from_slice(&self.tmp_zynq          .to_le_bytes()); 
    stream.extend_from_slice(&self.tmp_lis3mdltr     .to_le_bytes()); 
    stream.extend_from_slice(&self.tmp_bm280         .to_le_bytes()); 
    stream.extend_from_slice(&self.pressure          .to_le_bytes()); 
    stream.extend_from_slice(&self.humidity          .to_le_bytes()); 
    stream.extend_from_slice(&self.mag_x             .to_le_bytes()); 
    stream.extend_from_slice(&self.mag_y             .to_le_bytes()); 
    stream.extend_from_slice(&self.mag_z             .to_le_bytes());
    // padding - just for compatibility
    stream.extend_from_slice(&0.0_f32                 .to_le_bytes());
    stream.extend_from_slice(&self.drs_dvdd_voltage   .to_le_bytes()); 
    stream.extend_from_slice(&self.drs_dvdd_current   .to_le_bytes()); 
    stream.extend_from_slice(&self.drs_dvdd_power     .to_le_bytes()); 
    stream.extend_from_slice(&self.p3v3_voltage       .to_le_bytes()); 
    stream.extend_from_slice(&self.p3v3_current       .to_le_bytes()); 
    stream.extend_from_slice(&self.p3v3_power         .to_le_bytes()); 
    stream.extend_from_slice(&self.zynq_voltage       .to_le_bytes()); 
    stream.extend_from_slice(&self.zynq_current       .to_le_bytes()); 
    stream.extend_from_slice(&self.zynq_power         .to_le_bytes()); 
    stream.extend_from_slice(&self.p3v5_voltage       .to_le_bytes()); 
    stream.extend_from_slice(&self.p3v5_current       .to_le_bytes()); 
    stream.extend_from_slice(&self.p3v5_power         .to_le_bytes()); 
    stream.extend_from_slice(&self.adc_dvdd_voltage   .to_le_bytes()); 
    stream.extend_from_slice(&self.adc_dvdd_current   .to_le_bytes()); 
    stream.extend_from_slice(&self.adc_dvdd_power     .to_le_bytes()); 
    stream.extend_from_slice(&self.adc_avdd_voltage   .to_le_bytes()); 
    stream.extend_from_slice(&self.adc_avdd_current   .to_le_bytes()); 
    stream.extend_from_slice(&self.adc_avdd_power     .to_le_bytes()); 
    stream.extend_from_slice(&self.drs_avdd_voltage   .to_le_bytes()); 
    stream.extend_from_slice(&self.drs_avdd_current   .to_le_bytes()); 
    stream.extend_from_slice(&self.drs_avdd_power     .to_le_bytes()); 
    stream.extend_from_slice(&self.n1v5_voltage       .to_le_bytes()); 
    stream.extend_from_slice(&self.n1v5_current       .to_le_bytes()); 
    stream.extend_from_slice(&self.n1v5_power         .to_le_bytes()); 
    stream.extend_from_slice(&Self::TAIL.to_le_bytes());
    stream
  }

  fn from_bytestream(stream    : &Vec<u8>, 
                     pos       : &mut usize) 
    -> Result<RBMoniData, SerializationError>{
    let mut moni_data = Self::new();
    Self::verify_fixed(stream, pos)?;
    moni_data.board_id           = parse_u8( stream, pos); 
    moni_data.rate               = parse_u16(stream, pos); 
    moni_data.tmp_drs            = parse_f32(stream, pos); 
    moni_data.tmp_clk            = parse_f32(stream, pos); 
    moni_data.tmp_adc            = parse_f32(stream, pos); 
    moni_data.tmp_zynq           = parse_f32(stream, pos); 
    moni_data.tmp_lis3mdltr      = parse_f32(stream, pos); 
    moni_data.tmp_bm280          = parse_f32(stream, pos); 
    moni_data.pressure           = parse_f32(stream, pos); 
    moni_data.humidity           = parse_f32(stream, pos); 
    moni_data.mag_x              = parse_f32(stream, pos); 
    moni_data.mag_y              = parse_f32(stream, pos); 
    moni_data.mag_z              = parse_f32(stream, pos); 
    // compatibility, no mag_tot anymore
    *pos += 4;
    moni_data.drs_dvdd_voltage   = parse_f32(stream, pos); 
    moni_data.drs_dvdd_current   = parse_f32(stream, pos); 
    moni_data.drs_dvdd_power     = parse_f32(stream, pos); 
    moni_data.p3v3_voltage       = parse_f32(stream, pos); 
    moni_data.p3v3_current       = parse_f32(stream, pos); 
    moni_data.p3v3_power         = parse_f32(stream, pos); 
    moni_data.zynq_voltage       = parse_f32(stream, pos); 
    moni_data.zynq_current       = parse_f32(stream, pos); 
    moni_data.zynq_power         = parse_f32(stream, pos); 
    moni_data.p3v5_voltage       = parse_f32(stream, pos); 
    moni_data.p3v5_current       = parse_f32(stream, pos); 
    moni_data.p3v5_power         = parse_f32(stream, pos); 
    moni_data.adc_dvdd_voltage   = parse_f32(stream, pos); 
    moni_data.adc_dvdd_current   = parse_f32(stream, pos); 
    moni_data.adc_dvdd_power     = parse_f32(stream, pos); 
    moni_data.adc_avdd_voltage   = parse_f32(stream, pos); 
    moni_data.adc_avdd_current   = parse_f32(stream, pos); 
    moni_data.adc_avdd_power     = parse_f32(stream, pos); 
    moni_data.drs_avdd_voltage   = parse_f32(stream, pos); 
    moni_data.drs_avdd_current   = parse_f32(stream, pos); 
    moni_data.drs_avdd_power     = parse_f32(stream, pos); 
    moni_data.n1v5_voltage       = parse_f32(stream, pos); 
    moni_data.n1v5_current       = parse_f32(stream, pos); 
    moni_data.n1v5_power         = parse_f32(stream, pos); 
    *pos += 2; // for tail
    Ok(moni_data) 
  }
}

///////////////////////////////////////////////////////

#[derive(Debug, Copy, Clone, PartialEq)]
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

  #[cfg(feature = "tof-control")]
  pub fn add_temps(&mut self, cpu_temps : &CPUTempDebug) {
    self.cpu_temp   = cpu_temps.cpu_temp;
    self.cpu0_temp  = cpu_temps.cpu0_temp;
    self.cpu1_temp  = cpu_temps.cpu1_temp;
    self.mb_temp    = cpu_temps.mb_temp;
  }

  #[cfg(feature = "tof-control")]
  pub fn add_info(&mut self, cpu_info : &CPUInfo) {
    self.uptime = cpu_info.uptime;
    self.disk_usage = cpu_info.disk_usage;
    self.cpu_freq   = cpu_info.cpu_freq;
  }
}

impl Default for CPUMoniData {
  fn default() -> Self {
    CPUMoniData::new()
  }
}

impl fmt::Display for CPUMoniData {
  fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
    write!(f, "<CPUMoniData:\n
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

///////////////////////////////////////////////////////

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

///////////////////////////////////////////////////////

/// Monitoring the MTB
#[derive(Debug, Copy, Clone, PartialEq)]
pub struct MtbMoniData {
  pub calibration : u16, 
  pub vccpint     : u16, 
  pub vccpaux     : u16, 
  pub vccoddr     : u16, 
  pub temp        : u16, 
  pub vccint      : u16, 
  pub vccaux      : u16, 
  pub vccbram     : u16, 
  pub rate        : u16, 
  pub lost_rate   : u16, 
}

impl MtbMoniData {
  
  pub fn new() -> Self {
    Self {
      calibration  : u16::MAX,
      vccpint      : u16::MAX,
      vccpaux      : u16::MAX,
      vccoddr      : u16::MAX,
      temp         : u16::MAX,
      vccint       : u16::MAX,
      vccaux       : u16::MAX,
      vccbram      : u16::MAX,
      rate         : u16::MAX,
      lost_rate    : u16::MAX
    }
  }

  /// Convert ADC temp from adc values to Celsius
  pub fn get_fpga_temp(&self) -> f32 {
    self.temp as f32 * 503.975 / 4096.0 - 273.15
  }
  
  // Convert ADC VCCINT from adc values to Voltage
  fn adc_vcc_conversion(data : u16) -> f32 {
    3.0 * data as f32 / (2_u32.pow(12-1)) as f32
  }

}

impl Default for MtbMoniData {
  fn default() -> Self {
    Self::new()
  }
}

impl fmt::Display for MtbMoniData {
  fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
    write!(f, "<MtbMoniData:
  MTB  EVT RATE   [Hz] {}
  LOST EVT RATE   [Hz] {}
  CALIBRATION   [ADC?] {}
  VCCPINT          [V] {:.2}
  VCCPAUX          [V] {:.2}
  VCCODDR          [V] {:.2}
  FPGA TEMP        [C] {:.2}
  VCCINT           [C] {:.2}
  VCCAUX           [V] {:.2}
  VCCBRAM          [V] {:.2}>",
           self.rate,
           self.lost_rate,
           self.calibration,
           MtbMoniData::adc_vcc_conversion(self.vccpint   ),
           MtbMoniData::adc_vcc_conversion(self.vccpaux   ),
           MtbMoniData::adc_vcc_conversion(self.vccoddr   ),
           self.get_fpga_temp(),
           MtbMoniData::adc_vcc_conversion(self.vccint    ),
           MtbMoniData::adc_vcc_conversion(self.vccaux    ),
           MtbMoniData::adc_vcc_conversion(self.vccbram   ),
           )
  }
}

impl Serialization for MtbMoniData {
  
  const SIZE : usize = 24;
  const HEAD : u16   = 0xAAAA;
  const TAIL : u16   = 0x5555;

  fn to_bytestream(&self) -> Vec<u8> {
    let mut stream = Vec::<u8>::with_capacity(Self::SIZE);
    stream.extend_from_slice(&Self::HEAD.to_le_bytes());
    stream.extend_from_slice(&self.calibration.to_le_bytes());
    stream.extend_from_slice(&self.vccpint    .to_le_bytes());
    stream.extend_from_slice(&self.vccpaux    .to_le_bytes());
    stream.extend_from_slice(&self.vccoddr    .to_le_bytes());
    stream.extend_from_slice(&self.temp       .to_le_bytes());
    stream.extend_from_slice(&self.vccint     .to_le_bytes()); 
    stream.extend_from_slice(&self.vccaux     .to_le_bytes()); 
    stream.extend_from_slice(&self.vccbram    .to_le_bytes()); 
    stream.extend_from_slice(&self.rate       .to_le_bytes()); 
    stream.extend_from_slice(&self.lost_rate  .to_le_bytes());
    stream.extend_from_slice(&Self::TAIL.to_le_bytes());
    stream
  }

  fn from_bytestream(stream : &Vec<u8>, pos : &mut usize)
    -> Result<Self, SerializationError> {
    let mut moni_data      = Self::new();
    Self::verify_fixed(stream, pos)?;
    moni_data.calibration  = parse_u16(&stream, pos);
    moni_data.vccpint      = parse_u16(&stream, pos);
    moni_data.vccpaux      = parse_u16(&stream, pos);
    moni_data.vccoddr      = parse_u16(&stream, pos);
    moni_data.temp         = parse_u16(&stream, pos);
    moni_data.vccint       = parse_u16(&stream, pos);
    moni_data.vccaux       = parse_u16(&stream, pos);
    moni_data.vccbram      = parse_u16(&stream, pos);
    moni_data.rate         = parse_u16(&stream, pos);
    moni_data.lost_rate    = parse_u16(&stream, pos);
    *pos += 2; // since we deserialized the tail earlier and 
              // didn't account for it
    Ok(moni_data)
  }
}

#[cfg(feature = "random")]
impl FromRandom for MtbMoniData {
  fn from_random() -> Self {
    let mut moni      = Self::new();
    let mut rng       = rand::thread_rng();
    moni.calibration  = rng.gen::<u16>();
    moni.vccpint      = rng.gen::<u16>();
    moni.vccpaux      = rng.gen::<u16>();
    moni.vccoddr      = rng.gen::<u16>();
    moni.temp         = rng.gen::<u16>();
    moni.vccint       = rng.gen::<u16>();
    moni.vccaux       = rng.gen::<u16>();
    moni.vccbram      = rng.gen::<u16>();
    moni.rate         = rng.gen::<u16>();
    moni.lost_rate    = rng.gen::<u16>();
    moni
  }
}

#[cfg(all(test,feature = "random"))]
mod test_monitoring {
  use crate::serialization::Serialization;
  use crate::FromRandom;
  use crate::monitoring::RBMoniData;
  use crate::monitoring::MtbMoniData;
  use crate::monitoring::PBMoniData;  
  use crate::monitoring::PAMoniData;
  use crate::monitoring::LTBMoniData;
  use crate::monitoring::CPUMoniData;

  #[test]
  fn serialization_ltbmonidata() {
    for k in 0..100 {
      let data = LTBMoniData::from_random();
      let test = LTBMoniData::from_bytestream(&data.to_bytestream(), &mut 0).unwrap();
      assert_eq!(data, test);
    }
  }

  #[test]
  fn serialization_pamonidata() {
    for k in 0..100 {
      let data = PAMoniData::from_random();
      let test = PAMoniData::from_bytestream(&data.to_bytestream(), &mut 0).unwrap();
      assert_eq!(data, test);
    }
  }

  #[test]
  fn serialization_pbmonidata() {
    for k in 0..100 {
      let data = PBMoniData::from_random();
      let test = PBMoniData::from_bytestream(&data.to_bytestream(), &mut 0).unwrap();
      assert_eq!(data, test);
    }
  }

  #[test]
  fn serialization_mtbmonidata() {
    let data = MtbMoniData::from_random();
    let test = MtbMoniData::from_bytestream(&data.to_bytestream(), &mut 0).unwrap();
    assert_eq!(data, test);
  }

  #[test]
  fn serialization_rbmonidata() {
    let data = RBMoniData::from_random();
    let test = RBMoniData::from_bytestream(&data.to_bytestream(), &mut 0).unwrap();
    assert_eq!(data, test);
  }
  
  #[test]
  fn serialization_cpumonidata() {
    for k in 0..100 {
      let data = CPUMoniData::from_random();
      let test = CPUMoniData::from_bytestream(&data.to_bytestream(), &mut 0).unwrap();
      assert_eq!(data, test);
    }
  }
}

