// This file is part of gaps-online-software and published 
// under the GPLv3 license

use crate::prelude::*;


#[cfg(feature="tofcontrol")]
use tof_control::helper::pa_type::{
  PATemp,
  PAReadBias
};


/// Preamp temperature and bias data
#[derive(Debug, Copy, Clone, PartialEq)]
#[cfg_attr(feature="pybindings", pyclass)] 
pub struct PAMoniData {
  pub board_id           : u8,
  pub temps              : [f32;16],
  pub biases             : [f32;16],
}

impl PAMoniData {

  pub fn new() -> Self {
    Self {
      board_id  : 0,
      temps     : [f32::MAX;16],
      biases    : [f32::MAX;16],
    }
  }

  #[cfg(feature = "tofcontrol")]
  pub fn add_temps(&mut self, pt : &PATemp ) {
    self.temps = pt.pa_temps;
  }

  #[cfg(feature = "tofcontrol")]
  pub fn add_biases(&mut self, pb : &PAReadBias) {
    self.biases = pb.read_biases;
  }
}

#[cfg(feature="pybindings")]
#[pymethods]
impl PAMoniData {
  
  /// The temperature for the 16 preamp channels 
  #[getter]
  fn get_temps(&self) -> [f32;16] {
    self.temps
  }

  /// Pramp bias voltages (mV) for the 16 channels
  #[getter]
  fn get_biases(&self) -> [f32;16] {
    self.biases
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

impl TofPackable for PAMoniData {
  const TOF_PACKET_TYPE : TofPacketType = TofPacketType::PAMoniData;
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

impl MoniData for PAMoniData {
  
  fn get_board_id(&self) -> u8 {
    return self.board_id;
  }

  fn keys() -> Vec<&'static str> {
    vec!["board_id",
         "temps1"  , "temps2"  , "temps3"  , "temps4"  ,
         "temps5"  , "temps6"  , "temps7"  , "temps8"  , 
         "temps9"  , "temps10" , "temps11" , "temps12" ,
         "temps13" , "temps14" , "temps15" , "temps16" ,
         "biases1" , "biases2" , "biases3" , "biases4" , 
         "biases5" , "biases6" , "biases7" , "biases8" ,
         "biases9" , "biases10", "biases11", "biases12",
         "biases13", "biases14", "biases15", "biases16" ]
  }

  fn get(&self, varname : &str) -> Option<f32> {
    match varname {
      "board_id" =>  Some(self.board_id as f32),
      "temps1"   =>  Some(self.temps[0]  ),
      "temps2"   =>  Some(self.temps[1]  ),
      "temps3"   =>  Some(self.temps[2]  ),
      "temps4"   =>  Some(self.temps[3]  ),
      "temps5"   =>  Some(self.temps[4]  ),
      "temps6"   =>  Some(self.temps[5]  ),
      "temps7"   =>  Some(self.temps[6]  ),
      "temps8"   =>  Some(self.temps[7]  ),
      "temps9"   =>  Some(self.temps[8]  ),
      "temps10"  =>  Some(self.temps[9]  ),
      "temps11"  =>  Some(self.temps[10] ),
      "temps12"  =>  Some(self.temps[11] ),
      "temps13"  =>  Some(self.temps[12] ),
      "temps14"  =>  Some(self.temps[13] ),
      "temps15"  =>  Some(self.temps[14] ),
      "temps16"  =>  Some(self.temps[15] ),
      "biases1"  =>  Some(self.biases[0] ),
      "biases2"  =>  Some(self.biases[1] ),
      "biases3"  =>  Some(self.biases[2] ),
      "biases4"  =>  Some(self.biases[3] ),
      "biases5"  =>  Some(self.biases[4] ),
      "biases6"  =>  Some(self.biases[5] ),
      "biases7"  =>  Some(self.biases[6] ),
      "biases8"  =>  Some(self.biases[7] ),
      "biases9"  =>  Some(self.biases[8] ),
      "biases10" =>  Some(self.biases[9] ),
      "biases11" =>  Some(self.biases[10]),
      "biases12" =>  Some(self.biases[11]),
      "biases13" =>  Some(self.biases[12]),
      "biases14" =>  Some(self.biases[13]),
      "biases15" =>  Some(self.biases[14]),
      "biases16" =>  Some(self.biases[15]),
      _          =>  None
    }
  }  
}


#[cfg(feature = "random")]
impl FromRandom for PAMoniData {
    
  fn from_random() -> Self {
    let mut moni = Self::new();
    let mut rng  = rand::rng();
    moni.board_id     = rng.random::<u8>(); 
    for k in 0..16 {
      moni.temps[k]   = rng.random::<f32>(); 
    }
    for k in 0..16 {
      moni.biases[k]  = rng.random::<f32>(); 
    }
    moni
  }
}

//----------------------------------------

moniseries!(PAMoniDataSeries, PAMoniData);

#[cfg(feature="pybindings")]
pythonize_packable!(PAMoniData);

#[cfg(feature="pybindings")]
pythonize_monidata!(PAMoniData);

//----------------------------------------

#[test]
#[cfg(feature = "random")]
fn pack_pamonidata() {
  for _ in 0..100 {
    let data = PAMoniData::from_random();
    let test : PAMoniData = data.pack().unpack().unwrap();
    assert_eq!(data, test);
  }
}

