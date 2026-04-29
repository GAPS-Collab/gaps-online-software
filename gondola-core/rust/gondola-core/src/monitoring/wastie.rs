// This file is part of gaps-online-software and published 
// under the GPLv3 license

use crate::prelude::*;

// Sip (CSBF provided Gps position data
#[derive(Debug, Copy, Clone, PartialEq)]
#[cfg_attr(feature="pybindings", pyclass)] 
pub struct WastieMoniData {
  elapsed_time     : u16,
  threshold_mode   : u8 ,
  threshold_value  : u32,
  hits_processed   : u32,
  hits_removed     : u32,
  events_processed : u32,
  spectrum         : [u32;5],
  // not serialized 
  timestamp        : u64,
  board_id         : u8, 
}

impl WastieMoniData {

  pub fn new() -> Self {
    Self { 
      elapsed_time     : 0,
      threshold_mode   : 0,
      threshold_value  : 0,
      hits_processed   : 0,
      hits_removed     : 0,
      events_processed : 0,
      spectrum         : [0;5],
      timestamp        : 0,
      board_id         : 0,
    } 
  }
}

#[cfg(feature = "random")]
impl FromRandom for WastieMoniData {    
  fn from_random() -> Self {
    let mut moni          = Self::new();
    let mut rng           = rand::rng();
    moni.elapsed_time     = rng.random::<u16>();
    moni.threshold_mode   = rng.random::<u8>();
    moni.threshold_value  = rng.random::<u32>();
    moni.hits_processed   = rng.random::<u32>();
    moni.hits_removed     = rng.random::<u32>();
    moni.events_processed = rng.random::<u32>();
    for k in 0..5 {
      moni.spectrum[k]    = rng.random::<u32>();
    }
    //moni.spectrum         : [0;5]
    moni.timestamp        = rng.random::<u64>();
    moni.board_id         = rng.random::<u8>();
    moni
  } 
} 

impl Default for WastieMoniData {
  fn default() -> Self {
    Self::new()
  }
}

impl Serialization for WastieMoniData { 
  // FIXME - add size field

  fn from_bytestream(stream : &Vec<u8>,
                     pos    : &mut usize)
    -> Result<Self, SerializationError> {
    if stream.len() < Self::SIZE {
      return Err(SerializationError::StreamTooShort);
    }
    let mut moni    = Self::new();
    moni.elapsed_time     = parse_u16(stream, pos); 
    moni.threshold_mode   = parse_u8(stream, pos); 
    moni.threshold_value  = parse_u32(stream, pos); 
    moni.hits_processed   = parse_u32(stream, pos); 
    moni.hits_removed     = parse_u32(stream, pos); 
    moni.events_processed = parse_u32(stream, pos); 
    for k in 0..5 {
      moni.spectrum[k] = parse_u32(stream, pos); 
    }
    Ok(moni)
  }
}

impl MoniData for WastieMoniData {
  
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
    vec!["board_id",
         "elapsed_time",
         "threshold_mode",
         "threshold_value",
         "hits_processed",
         "hits_removed",
         "events_processed",
         "spectrum0",
         "spectrum1",
         "spectrum2",
         "spectrum3",
         "spectrum4",
         "timestamp"]
  }

  fn get(&self, varname : &str) -> Option<f32> {
    match varname {
      "board_id"          => Some(self.board_id as f32),
      "timestamp"         => Some(self.timestamp as f32),
      "elapsed_time"      => Some(self.elapsed_time as f32),
      "threshold_mode"    => Some(self.threshold_mode as f32),
      "threshold_value"   => Some(self.threshold_value as f32),
      "hits_processed"    => Some(self.hits_processed as f32),
      "hits_removed"      => Some(self.hits_removed as f32),
      "events_processed"  => Some(self.events_processed as f32),
      "spectrum0"         => Some(self.spectrum[0] as f32),
      "spectrum1"         => Some(self.spectrum[1] as f32),
      "spectrum2"         => Some(self.spectrum[2] as f32),
      "spectrum3"         => Some(self.spectrum[3] as f32),
      "spectrum4"         => Some(self.spectrum[4] as f32),
      _            => None
    }
  }  
}

impl TelemetryPackable for WastieMoniData {
  const TEL_PACKET_TYPE : TelemetryPacketType = TelemetryPacketType::WastieHK;
}

impl fmt::Display for WastieMoniData {
  fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
    let mut repr = String::from("<WastieMoniData: ");
    repr        += &(format!("\n elapsed_time     : {} ", self.elapsed_time    ));
    repr        += &(format!("\n threshold_mode   : {} ", self.threshold_mode  ));
    repr        += &(format!("\n threshold_value  : {} ", self.threshold_value ));
    repr        += &(format!("\n hits_processed   : {} ", self.hits_processed  ));
    repr        += &(format!("\n hits_removed     : {} ", self.hits_removed    ));
    repr        += &(format!("\n events_processed : {} ", self.events_processed));
    repr        += &(format!("\n spectrum0        : {} ", self.spectrum[0]     ));
    repr        += &(format!("\n spectrum1        : {} ", self.spectrum[1]     ));
    repr        += &(format!("\n spectrum2        : {} ", self.spectrum[2]     ));
    repr        += &(format!("\n spectrum3        : {} ", self.spectrum[3]     ));
    repr        += &(format!("\n spectrum4        : {}>", self.spectrum[4]     ));
    write!(f, "{}", repr)
  }
}

moniseries_telemetry!(WastieMoniDataSeries, WastieMoniData);

#[cfg(feature="pybindings")]
pythonize_monidata!(WastieMoniData);

#[cfg(feature="pybindings")]
pythonize_telemetry_only!(WastieMoniData);

