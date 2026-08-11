// This file is part of gaps-online-software and published 
// under the GPLv3 license

use crate::prelude::*;

// Sip (CSBF provided Gps position data
#[derive(Debug, Copy, Clone, PartialEq)]
#[cfg_attr(feature="pybindings", pyclass)] 
pub struct TrackerGpsMoniData {
  pub daq_header : TrackerHeader,
  pub utc_time   : u32,
  pub gps_info   : u8,
  // make it compatible with other moni data
  pub timestamp  : u64,
  pub board_id   : u8,
}

impl TrackerGpsMoniData {

  fn new() ->Self {
    Self {
      daq_header : TrackerHeader::new(),
      utc_time   : 0,
      gps_info   : 0,
      timestamp  : 0,
      board_id   : 0,
    }
  }
}

#[cfg(feature = "random")]
impl FromRandom for TrackerGpsMoniData {    
  fn from_random() -> Self {
    let mut moni    = Self::new();
    let mut rng     = rand::rng();
    moni.utc_time   = rng.random::<u32>();
    moni.gps_info   = rng.random::<u8>();
    moni.daq_header = TrackerHeader::from_random();
    moni.timestamp  = rng.random::<u64>();
    moni.board_id   = rng.random::<u8>();
    moni
  } 
} 

impl Default for TrackerGpsMoniData {
  fn default() -> Self {
    Self::new()
  }
}

impl Serialization for TrackerGpsMoniData { 

  fn from_bytestream(stream : &Vec<u8>,
                     pos    : &mut usize)
    -> Result<Self, SerializationError> {
    if stream.len() < Self::SIZE {
      return Err(SerializationError::StreamTooShort);
    }
    let mut moni = Self::new();
    moni.daq_header = TrackerHeader::from_bytestream(stream, pos)?;
    *pos += 1; // there is one empty byte
    moni.utc_time   = parse_u32(stream, pos);
    moni.gps_info   = parse_u8(stream, pos);
    Ok(moni)
  }
}

impl MoniData for TrackerGpsMoniData {
  
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
    vec!["board_id", "utc_time", 
         "gps_info",
         "timestamp"]
  }

  fn get(&self, varname : &str) -> Option<f32> {
    match varname {
      "board_id"  => Some(self.board_id as f32),
      "utc_time"   => Some(self.utc_time as f32),
      "gps_info"   => Some(self.gps_info as f32),
      "timestamp" => Some(self.timestamp as f32),
      _           => None
    }
  }  
}

impl TelemetryPackable for TrackerGpsMoniData {
  const TEL_PACKET_TYPE : TelemetryPacketType = TelemetryPacketType::TrackerGps;
}

impl fmt::Display for TrackerGpsMoniData {
  fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
    let mut repr = String::from("<TrackerGpsMoniData: ");
    repr        += &(format!("\n utc_time : {}", self.utc_time));
    repr        += &(format!("\n gps_info : {}>", self.gps_info));
    write!(f, "{}", repr)
  }
}


moniseries_telemetry!(TrackerGpsMoniDataSeries, TrackerGpsMoniData);

#[cfg(feature="pybindings")]
pythonize_monidata!(TrackerGpsMoniData);

#[cfg(feature="pybindings")]
pythonize_telemetry_only!(TrackerGpsMoniData);
