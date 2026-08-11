// This file is part of gaps-online-software and published 
// under the GPLv3 license

use crate::prelude::*;



// Sip (CSBF provided gps time data
#[derive(Debug, Copy, Clone, PartialEq)]
#[cfg_attr(feature="pybindings", pyclass)] 
pub struct SipTimeMoniData {
  pub sip_id       : u8,
  pub time_of_week : u32,
  pub week_number  : u16,
  pub time_offset  : u32,
  pub cpu_time     : u32,
  // make it compatible with other moni data
  pub timestamp    : u64,
  pub board_id     : u8,
}

impl SipTimeMoniData {

  pub fn new() -> Self {
    Self {
      sip_id       : 0,
      time_of_week : 0,
      week_number  : 0,
      time_offset  : 0,
      cpu_time     : 0,
      timestamp    : 0,
      board_id     : 0,
    }
  }
}

impl fmt::Display for SipTimeMoniData {
  fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
    write!(f, "<SipTimeMoniData:
  SIP ID   : {}
  time_of_week : {},
  week_number  : {},
  time_offset  : {},
  cpu_time     : {},
  timstamp     : {}>",
  self.sip_id, 
  self.time_of_week,
  self.week_number,
  self.time_offset,
  self.cpu_time,
  self.timestamp)
  }
}

#[cfg(feature = "random")]
impl FromRandom for SipTimeMoniData {
    
  fn from_random() -> Self {
    let mut moni      = Self::new();
    let mut rng       = rand::rng();
    moni.sip_id       = rng.random::<u8>();
    moni.time_of_week = rng.random::<u32>();
    moni.week_number  = rng.random::<u16>();
    moni.time_offset  = rng.random::<u32>();
    moni.cpu_time     = rng.random::<u32>();
    moni.board_id     = rng.random::<u8>();
    moni 
  }
}

impl Default for SipTimeMoniData {
  fn default() -> Self {
    Self::new()
  }
}

impl Serialization for SipTimeMoniData { 

  const SIZE : usize = 18;

  fn from_bytestream(stream : &Vec<u8>,
                     pos    : &mut usize)
    -> Result<Self, SerializationError> {
    if stream.len() < Self::SIZE {
      return Err(SerializationError::StreamTooShort);
    };
    let mut moni = Self::new();
    moni.sip_id = parse_u8(stream, pos);
    let     dle = parse_u8(stream, pos);
    if dle != 0x10 {
       //return -2;
       return Err(SerializationError::Unknown);
    } 
    let sip_type = parse_u8(stream, pos);
    if sip_type != 0x11 {
      // return -3 
      return Err(SerializationError::Unknown);
    }
    moni.time_of_week = parse_u32(stream, pos);
    moni.week_number  = parse_u16(stream, pos);
    moni.time_offset  = parse_u32(stream, pos);
    moni.cpu_time     = parse_u32(stream, pos);
    let etx           = parse_u8(stream, pos);
    if etx != 0x03 {
       // return -4
       return Err(SerializationError::Unknown);
    }  
    Ok(moni)
  } 
}

impl MoniData for SipTimeMoniData {
  
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
         "time_of_week", "week_number", "time_offset", "cpu_time",
         "timestamp"]
  }

  fn get(&self, varname : &str) -> Option<f32> {
    match varname {
      "board_id"     => Some(self.board_id as f32),
      "time_of_week" => Some(self.time_of_week as f32),
      "week_number"  => Some(self.week_number as f32), 
      "time_offset"  => Some(self.time_offset as f32),
      "cpu_time"     => Some(self.cpu_time as f32),
      "timestamp"    => Some(self.timestamp as f32),
      _              => None
    }
  }  
}

impl TelemetryPackable for SipTimeMoniData {
  const TEL_PACKET_TYPE : TelemetryPacketType = TelemetryPacketType::SipGpsTime;
}

moniseries_telemetry!(SipTimeMoniDataSeries, SipTimeMoniData);

#[cfg(feature="pybindings")]
pythonize_monidata!(SipTimeMoniData);

