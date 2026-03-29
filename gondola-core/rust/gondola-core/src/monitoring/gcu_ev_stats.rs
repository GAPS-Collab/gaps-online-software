// This file is part of gaps-online-software and published 
// under the GPLv3 license

use crate::prelude::*;

// Sip (CSBF provided gps time data
#[derive(Debug, Copy, Clone, PartialEq)]
#[cfg_attr(feature="pybindings", pyclass)] 
pub struct GcuEvBldStatsMoniData {
  pub num_bytes_190              : u32, 
  pub num_bytes_90               : u32, 
  pub num_bytes_191              : u32, 
  pub num_bytes_192              : u32, 
  pub num_events_in_queue        : u32, 
  pub num_tracker_packets_all    : u32, 
  pub num_tracker_packets_usable : u32, 
  pub num_tof_packets_all        : u32, 
  pub num_tof_packets_usable     : u32, 
  pub milliseconds               : u16, 
  pub num_events_90              : u16,  //uninteresting merged events
  pub num_events_190             : u16,  //interesting merged events
  pub num_events_191             : u16,  //track trigger merged events
  pub num_events_192             : u16,  //tracker-only merged events
  pub num_pack_fails             : u16, 
  pub num_events_tof_only        : u16, 
  pub num_events_tracker_only    : u16, 
  pub num_events_tracker_and_tof : u16, 
  pub version                    : u8, 
  pub pad1                       : u8, 

  // make it compatible with other moni data
  pub timestamp    : u64,
  pub board_id     : u8,
}

impl GcuEvBldStatsMoniData {

  pub fn new() -> Self {
    Self {
      num_bytes_190              : 0, 
      num_bytes_90               : 0, 
      num_bytes_191              : 0, 
      num_bytes_192              : 0, 
      num_events_in_queue        : 0, 
      num_tracker_packets_all    : 0, 
      num_tracker_packets_usable : 0, 
      num_tof_packets_all        : 0, 
      num_tof_packets_usable     : 0, 
      milliseconds               : 0, 
      num_events_90              : 0,  //uninteresting merged events
      num_events_190             : 0,  //interesting merged events
      num_events_191             : 0,  //track trigger merged events
      num_events_192             : 0,  //tracker-only merged events
      num_pack_fails             : 0, 
      num_events_tof_only        : 0, 
      num_events_tracker_only    : 0, 
      num_events_tracker_and_tof : 0, 
      version                    : 1, 
      pad1                       : 0, 
      timestamp                  : 0,
      board_id                   : 0
    }
  }
}

impl fmt::Display for GcuEvBldStatsMoniData {
  fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
    let mut repr = String::from("<GcuEvBldStatsMoniData:");
    repr += &(format!("\n  num_bytes_190               : {}", self.num_bytes_190             )); 
    repr += &(format!("\n  num_bytes_90                : {}", self.num_bytes_90              )); 
    repr += &(format!("\n  num_bytes_191               : {}", self.num_bytes_191             )); 
    repr += &(format!("\n  num_bytes_192               : {}", self.num_bytes_192             )); 
    repr += &(format!("\n  num_events_in_queue         : {}", self.num_events_in_queue       )); 
    repr += &(format!("\n  num_tracker_packets_all     : {}", self.num_tracker_packets_all   )); 
    repr += &(format!("\n  num_tracker_packets_usable  : {}", self.num_tracker_packets_usable)); 
    repr += &(format!("\n  num_tof_packets_all         : {}", self.num_tof_packets_all       )); 
    repr += &(format!("\n  num_tof_packets_usable      : {}", self.num_tof_packets_usable    )); 
    repr += &(format!("\n  milliseconds                : {}", self.milliseconds              )); 
    repr += &(format!("\n  num_events_90               : {}", self.num_events_90             ));  
    repr += &(format!("\n  num_events_190              : {}", self.num_events_190            ));  
    repr += &(format!("\n  num_events_191              : {}", self.num_events_191            ));  
    repr += &(format!("\n  num_events_192              : {}", self.num_events_192            ));  
    repr += &(format!("\n  num_pack_fails              : {}", self.num_pack_fails            )); 
    repr += &(format!("\n  num_events_tof_only         : {}", self.num_events_tof_only       )); 
    repr += &(format!("\n  num_events_tracker_only     : {}", self.num_events_tracker_only   )); 
    repr += &(format!("\n  num_events_tracker_and_tof  : {}", self.num_events_tracker_and_tof)); 
    repr += &(format!("\n  version                     : {}", self.version                   )); 
    repr += &(format!("\n  pad1                        : {}", self.pad1                      )); 
    write!(f, "{}", repr)
  }
}

#[cfg(feature = "random")]
impl FromRandom for GcuEvBldStatsMoniData {
    
  fn from_random() -> Self {
    let mut moni      = Self::new();
    let mut rng       = rand::rng();
    moni.num_bytes_190              = rng.random::<u32>();  
    moni.num_bytes_90               = rng.random::<u32>();  
    moni.num_bytes_191              = rng.random::<u32>();  
    moni.num_bytes_192              = rng.random::<u32>();  
    moni.num_events_in_queue        = rng.random::<u32>();  
    moni.num_tracker_packets_all    = rng.random::<u32>();  
    moni.num_tracker_packets_usable = rng.random::<u32>();  
    moni.num_tof_packets_all        = rng.random::<u32>();  
    moni.num_tof_packets_usable     = rng.random::<u32>();  
    moni.milliseconds               = rng.random::<u16>();  
    moni.num_events_90              = rng.random::<u16>();   //uninteresting merged events
    moni.num_events_190             = rng.random::<u16>();   //interesting merged events
    moni.num_events_191             = rng.random::<u16>();   //track trigger merged events
    moni.num_events_192             = rng.random::<u16>();   //tracker-only merged events
    moni.num_pack_fails             = rng.random::<u16>();  
    moni.num_events_tof_only        = rng.random::<u16>();  
    moni.num_events_tracker_only    = rng.random::<u16>();  
    moni.num_events_tracker_and_tof = rng.random::<u16>();  
    // don't change the version or the board id
    moni.pad1                       = rng.random::<u8>();  
    moni.timestamp                  = rng.random::<u64>(); 
    moni 
  }
}

impl Default for GcuEvBldStatsMoniData {
  fn default() -> Self {
    Self::new()
  }
}

impl Serialization for GcuEvBldStatsMoniData { 

  const SIZE : usize = 18;

  fn from_bytestream(stream : &Vec<u8>,
                     pos    : &mut usize)
    -> Result<Self, SerializationError> {
    if stream.len() < Self::SIZE {
      return Err(SerializationError::StreamTooShort);
    };
    let mut moni = Self::new();
    moni.num_bytes_190               = parse_u32(stream, pos); 
    moni.num_bytes_90                = parse_u32(stream, pos); 
    moni.num_bytes_191               = parse_u32(stream, pos); 
    moni.num_bytes_192               = parse_u32(stream, pos); 
    moni.num_events_in_queue         = parse_u32(stream, pos); 
    moni.num_tracker_packets_all     = parse_u32(stream, pos); 
    moni.num_tracker_packets_usable  = parse_u32(stream, pos); 
    moni.num_tof_packets_all         = parse_u32(stream, pos); 
    moni.num_tof_packets_usable      = parse_u32(stream, pos); 
    moni.milliseconds                = parse_u16(stream, pos); 
    moni.num_events_90               = parse_u16(stream, pos);  
    moni.num_events_190              = parse_u16(stream, pos);  
    moni.num_events_191              = parse_u16(stream, pos);  
    moni.num_events_192              = parse_u16(stream, pos);  
    moni.num_pack_fails              = parse_u16(stream, pos); 
    moni.num_events_tof_only         = parse_u16(stream, pos); 
    moni.num_events_tracker_only     = parse_u16(stream, pos); 
    moni.num_events_tracker_and_tof  = parse_u16(stream, pos); 
    moni.version                     = parse_u8(stream, pos); 
    moni.pad1                        = parse_u8(stream, pos); 
    Ok(moni)
  } 
}

impl MoniData for GcuEvBldStatsMoniData {
  
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
         "num_bytes_190"          , "num_bytes_90"               , "num_bytes_191"             , "num_bytes_192"      , 
         "num_events_in_queue"    , "num_tracker_packets_all"    , "num_tracker_packets_usable", "num_tof_packets_all", 
         "num_tof_packets_usable" , "milliseconds"               , "num_events_90"             , "num_events_190"     ,  
         "num_events_191"         , "num_events_192"             , "num_pack_fails"            , "num_events_tof_only", 
         "num_events_tracker_only", "num_events_tracker_and_tof" , "version"                   , "pad1"               , 
         "timestamp"]
  }





  fn get(&self, varname : &str) -> Option<f32> {
    match varname {
      "board_id"                   => Some(self.board_id as f32),
      "num_bytes_190"              => Some(self.num_bytes_190               as f32), 
      "num_bytes_90"               => Some(self.num_bytes_90                as f32), 
      "num_bytes_191"              => Some(self.num_bytes_191               as f32), 
      "num_bytes_192"              => Some(self.num_bytes_192               as f32), 
      "num_events_in_queue"        => Some(self.num_events_in_queue         as f32), 
      "num_tracker_packets_all"    => Some(self.num_tracker_packets_all     as f32), 
      "num_tracker_packets_usable" => Some(self.num_tracker_packets_usable  as f32), 
      "num_tof_packets_all"        => Some(self.num_tof_packets_all         as f32), 
      "num_tof_packets_usable"     => Some(self.num_tof_packets_usable      as f32), 
      "milliseconds"               => Some(self.milliseconds                as f32), 
      "num_events_90"              => Some(self.num_events_90               as f32),  
      "num_events_190"             => Some(self.num_events_190              as f32),  
      "num_events_191"             => Some(self.num_events_191              as f32),  
      "num_events_192"             => Some(self.num_events_192              as f32),  
      "num_pack_fails"             => Some(self.num_pack_fails              as f32), 
      "num_events_tof_only"        => Some(self.num_events_tof_only         as f32), 
      "num_events_tracker_only"    => Some(self.num_events_tracker_only     as f32), 
      "num_events_tracker_and_tof" => Some(self.num_events_tracker_and_tof  as f32), 
      "version"                    => Some(self.version                     as f32), 
      "pad1"                       => Some(self.pad1                        as f32), 
      "timestamp"    => Some(self.timestamp as f32),
      _              => None
    }
  }  
}

impl TelemetryPackable for GcuEvBldStatsMoniData {
  const TEL_PACKET_TYPE : TelemetryPacketType = TelemetryPacketType::GcuEvtBuilderStats;
}

moniseries_telemetry!(GcuEvBldStatsMoniDataSeries, GcuEvBldStatsMoniData);

#[cfg(feature="pybindings")]
pythonize_monidata!(GcuEvBldStatsMoniData);

#[cfg(feature="pybindings")]
pythonize_telemetry_only!(GcuEvBldStatsMoniData);

