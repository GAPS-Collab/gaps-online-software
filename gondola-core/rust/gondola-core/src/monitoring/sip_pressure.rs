// This file is part of gaps-online-software and published 
// under the GPLv3 license

use crate::prelude::*;

// Sip (CSBF provided Pressure data
#[derive(Debug, Copy, Clone, PartialEq)]
#[cfg_attr(feature="pybindings", pyclass)] 
pub struct SipPresMoniData {
  pub sip_id     : u8,
  pub mks_high   : u16,
  pub mks_mid    : u16, 
  pub mks_lo     : u16,
  // make it compatible with other moni data
  pub timestamp  : u64,
  pub board_id   : u8,
  //pub timestamp  : u64,
}

impl SipPresMoniData {

  pub fn new() -> Self {
    Self {
      sip_id     : 0,
      mks_high   : 0,
      mks_mid    : 0, 
      mks_lo    : 0,
      timestamp  : 0,
      board_id   : 0,
    }
  }
}

impl fmt::Display for SipPresMoniData {
  fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
    write!(f, "<SipPresMoniData:
  SIP ID   : {}
  mks high : {}
  mks_mid  : {}
  mks_lo   : {} 
  timstamp : {}>",
  self.sip_id, 
  self.mks_high,
  self.mks_mid,
  self.mks_lo,
  self.timestamp)
  }
}

#[cfg(feature = "random")]
impl FromRandom for SipPresMoniData {
    
  fn from_random() -> Self {
    let mut moni   = Self::new();
    let mut rng    = rand::rng();
    moni.sip_id    = rng.random::<u8>();
    moni.mks_lo    = rng.random::<u16>();
    moni.mks_mid   = rng.random::<u16>();
    moni.mks_high  = rng.random::<u16>();
    moni.timestamp = rng.random::<u64>();
    moni.board_id  = rng.random::<u8>();
    moni 
  }
}

impl Default for SipPresMoniData {
  fn default() -> Self {
    Self::new()
  }
}

impl Serialization for SipPresMoniData { 
  
  // size without TelemetryPacketHeader
  const SIZE : usize = 10;

  fn from_bytestream(stream : &Vec<u8>,
                     pos    : &mut usize)
    -> Result<Self, SerializationError> {
    if stream.len() < Self::SIZE {
      return Err(SerializationError::StreamTooShort);
    }
    let mut moni = Self::new();
    moni.sip_id  = parse_u8(stream, pos);
    let     dle  = parse_u8(stream, pos);
    if dle != 0x10 {
       //return -2;
       return Err(SerializationError::Unknown);
    } 
    let sip_type = parse_u8(stream, pos);
    if sip_type != 0x12 {
      error!("Wrong SIP type - seeing {}", sip_type);
      return Err(SerializationError::Unknown);
    }
    moni.mks_high     = parse_u16(stream, pos);
    moni.mks_mid      = parse_u16(stream, pos);
    moni.mks_lo       = parse_u16(stream, pos);
    let etx           = parse_u8(stream, pos);
    if etx != 0x03 {
       // return -4
       return Err(SerializationError::Unknown);
    }  
    Ok(moni)
  } 
}

impl MoniData for SipPresMoniData {
  
  fn get_board_id(&self) -> u8 {
    return self.board_id;
  }

  fn get_timestamp(&self) -> u64 {
    self.timestamp 
  }

  fn set_timestamp(&mut self, ts: u64) {
    self.timestamp = ts;
  }

  fn keys() -> Vec<&'static str> {
    vec!["board_id", "sip_id", 
         "mks_lo", "mks_mid", "mks_high",
         "timestamp"]
  }

  fn get(&self, varname : &str) -> Option<f32> {
    match varname {
      "board_id"  => Some(self.board_id as f32),
      "sip_id"    => Some(self.sip_id as f32),
      "mks_lo"    => Some(self.mks_lo as f32), 
      "mks_mid"   => Some(self.mks_mid as f32),
      "mks_high"  => Some(self.mks_high as f32),
      "timestamp" => Some(self.timestamp as f32),
      _           => None
    }
  }  
}

impl TelemetryPackable for SipPresMoniData {
  const TEL_PACKET_TYPE : TelemetryPacketType = TelemetryPacketType::SipPressure;
}

moniseries_telemetry!(SipPresMoniDataSeries, SipPresMoniData);

