// This file is part of gaps-online-software and published 
// under the GPLv3 license

use crate::prelude::*;


// Sip (CSBF provided GPS position data
#[derive(Debug, Copy, Clone, PartialEq)]
#[cfg_attr(feature="pybindings", pyclass)] 
pub struct SipPosMoniData {
  pub longitude  : f32,
  pub latitude   : f32,
  pub altitude   : f32,
  pub status1    : u8,
  pub status2    : u8,
  pub sip_id     : u8,
  // make it compatible with other moni data
  pub timestamp  : u64,
  pub board_id   : u8,
}

impl SipPosMoniData {

  pub fn new() -> Self {
    Self {
      longitude  : 0.0,
      latitude   : 0.0,
      altitude   : 0.0,
      status1    : 0,
      status2    : 0,
      sip_id     : 0,
      timestamp  : 0,
      board_id   : 0,
    }
  }
}

#[cfg(feature = "random")]
impl FromRandom for SipPosMoniData {
    
  fn from_random() -> Self {
    let mut moni  = Self::new();
    let mut rng   = rand::rng();
    moni.longitude = rng.random::<f32>();
    moni.latitude  = rng.random::<f32>();
    moni.altitude  = rng.random::<f32>();
    moni.status1   = rng.random::<u8>();
    moni.status2   = rng.random::<u8>();
    moni.sip_id    = rng.random::<u8>(); 
    moni.timestamp = rng.random::<u64>();
    moni.board_id  = rng.random::<u8>();
    moni
  }
}

impl Default for SipPosMoniData {
  fn default() -> Self {
    Self::new()
  }
}

impl Serialization for SipPosMoniData { 

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
       error!("-2");
       return Err(SerializationError::Unknown);
    } 
    let sip_type = parse_u8(stream, pos);
    if sip_type != 0x10 {
      error!("Wrong SIP type - seeing {} instead of {}", sip_type, 0x10);
      return Err(SerializationError::Unknown);
    }
    moni.longitude    = parse_f32(stream, pos);
    moni.latitude     = parse_f32(stream, pos);
    moni.altitude     = parse_f32(stream, pos);
    moni.status1      = parse_u8(stream, pos);
    moni.status2      = parse_u8(stream, pos);
    let etx           = parse_u8(stream, pos);
    if etx != 0x03 {
       // return -4
       error!("-4");
       return Err(SerializationError::Unknown);
    }  
    Ok(moni)
  } 
}

impl MoniData for SipPosMoniData {
  
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
         "longitude", "latitude", "altitude",
         "status1", "status2",
         "timestamp"]
  }

  fn get(&self, varname : &str) -> Option<f32> {
    match varname {
      "board_id"  => Some(self.board_id as f32),
      "sip_id"    => Some(self.sip_id as f32),
      "longitude" => Some(self.longitude as f32), 
      "latitude"  => Some(self.latitude as f32),
      "altitude"  => Some(self.altitude as f32),
      "status1"   => Some(self.status1 as f32),
      "status2"   => Some(self.status2 as f32),
      "timestamp" => Some(self.timestamp as f32),
      _           => None
    }
  }  
}

impl TelemetryPackable for SipPosMoniData {
  const TEL_PACKET_TYPE : TelemetryPacketType = TelemetryPacketType::SipGpsPosition;
}

moniseries_telemetry!(SipPosMoniDataSeries, SipPosMoniData);

#[cfg(feature="pybindings")]
pythonize_monidata!(SipPosMoniData);

