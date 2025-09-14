// This file is part of gaps-online-software and published 
// under the GPLv3 license

use crate::prelude::*;
use colored::Colorize;

//use std::fmt;
//#[cfg(feature = "random")]  
//use crate::random::FromRandom;
//#[cfg(feature = "random")]  
//use rand::Rng;
//
//
//use crate::version::ProtocolVersion;
//use crate::io::serialization::Serialization;
//use crate::io::parsers::{
//  parse_u8,
//  parse_u16,
//  parse_u64,
//  parse_f32,
//};
//
//use crate::errors::SerializationError;
//
//use crate::packets::{
//  TofPackable,
//  TofPacketType,
//};
//
//use crate::monitoring::MoniData;
//use crate::moniseries;
//
//#[cfg(feature="pybindings")]
//use pyo3::prelude::*;
//
//#[cfg(feature="pybindings")]
//use pyo3::exceptions::{
//  PyIOError,
//  PyKeyError
//};
//
//#[cfg(feature="pybindings")]
//use crate::packets::TofPacket;
//
//#[cfg(feature="pybindings")]
//use crate::pythonize_packable;
//
//#[cfg(feature="pybindings")]
//use crate::pythonize_monidata;

#[derive(Debug, Copy, Clone, PartialEq)]
#[cfg_attr(feature="pybindings", pyclass)]
pub struct MasterTriggerHB {
  pub version             : ProtocolVersion, 
  pub total_elapsed       : u64, //aka met (mission elapsed time)
  pub n_events            : u64,
  pub evq_num_events_last : u64,
  pub evq_num_events_avg  : u64,
  pub n_ev_unsent         : u64,
  pub n_ev_missed         : u64,
  pub trate               : u64,
  pub lost_trate          : u64,
  // these will be available for ProtocolVersion::V1
  pub prescale_track      : f32,
  pub prescale_gaps       : f32,
}

impl MasterTriggerHB {
  pub fn new() -> Self {
    Self {
      version             : ProtocolVersion::Unknown,
      total_elapsed       : 0,
      n_events            : 0,
      evq_num_events_last : 0,
      evq_num_events_avg  : 0,
      n_ev_unsent         : 0,
      n_ev_missed         : 0,
      trate               : 0,
      lost_trate          : 0,
      // available for protocol version V1 and larger
      prescale_track      : 0.0,
      prescale_gaps       : 0.0,
    }
  }

  pub fn get_sent_packet_rate(&self) -> f64 {
    if self.total_elapsed > 0 {
      return self.n_events as f64 / self.total_elapsed as f64;
    }
    0.0
  }

  // get the prescale for the secondary trigger
  pub fn get_prescale_track(&self) -> f64 {
    if self.version == ProtocolVersion::Unknown {
      error!("Prescale not available for protocol version < V1!");
      return 0.0;
    }
    return self.prescale_track as f64
  }
  
  // get the prescale for the secondary trigger
  pub fn get_prescale_gaps(&self) -> f64 {
    if self.version == ProtocolVersion::Unknown {
      error!("Prescale not available for protocol version < V1!");
      return 0.0;
    }
    return self.prescale_gaps as f64
  }


  pub fn pretty_print(&self) -> String {
    let mut repr = format!("<MasterTriggerHBs (version : {})", self.version);
    repr += &(format!("\n \u{1FA90} \u{1FA90} \u{1FA90} \u{1FA90} \u{1FA90} MTB HEARTBEAT \u{1FA90} \u{1FA90} \u{1FA90} \u{1FA90} \u{1FA90} "));
    repr += &(format!("\n MET (Mission Elapsed Time)  : {:.1} sec", self.total_elapsed));
    repr += &(format!("\n Num. recorded Events        : {}", self.n_events));
    repr += &(format!("\n Last MTB EVQ size           : {}", self.evq_num_events_last));
    repr += &(format!("\n Avg. MTB EVQ size (per 30s ): {:.2}", self.evq_num_events_avg));
    repr += &(format!("\n trigger rate, recorded:     : {:.2} Hz", self.n_events as f64 / self.total_elapsed as f64));
    repr += &(format!("\n trigger rate, from register : {:.2} Hz", self.trate));
    repr += &(format!("\n lost trg rate, from register: {:.2} Hz", self.lost_trate));
    if self.n_ev_unsent > 0 {
        repr += &(format!("\n Num. sent errors        : {}", self.n_ev_unsent).bold());
    }
    if self.n_ev_missed > 0 {
        repr += &(format!("\n Num. missed events      : {}", self.n_ev_missed).bold());
    }
    if self.version != ProtocolVersion::Unknown {
        repr += &(format!("\n Prescale, prim. ('GAPS') trg : {:.4}", self.prescale_gaps));
        repr += &(format!("\n Prescale  sec. ('Track') trg : {:.4}", self.prescale_track));
    }
    repr += &(format!("\n \u{1FA90} \u{1FA90} \u{1FA90} \u{1FA90} \u{1FA90} END HEARTBEAT \u{1FA90} \u{1FA90} \u{1FA90} \u{1FA90} \u{1FA90} "));
    repr
  }
}
  
impl Default for MasterTriggerHB {
  fn default () -> Self {
    Self::new()
  }
}

impl TofPackable for MasterTriggerHB {
  const TOF_PACKET_TYPE : TofPacketType = TofPacketType::MasterTriggerHB;
}

impl Serialization for MasterTriggerHB {
  const HEAD : u16 = 0xAAAA;
  const TAIL : u16 = 0x5555;
  const SIZE : usize = 68;

  fn from_bytestream(stream    :&Vec<u8>,
                     pos       :&mut usize)
  -> Result<Self, SerializationError>{
    Self::verify_fixed(stream, pos)?;
    let mut hb = MasterTriggerHB::new();
    hb.total_elapsed          = parse_u64(stream, pos);
    hb.n_events               = parse_u64(stream, pos);
    hb.evq_num_events_last    = parse_u64(stream, pos);
    hb.evq_num_events_avg     = parse_u64(stream, pos);
    hb.n_ev_unsent            = parse_u64(stream, pos);
    hb.n_ev_missed            = parse_u64(stream, pos);
    // we use only 48bit here to carve out space for the 
    // protocol version and the prescales
    // this is a hack, but since we are expeting rates
    // < 65kHz, we should be fine with only 16bit for the 
    // rate and can use the rest for the prescale
    //let version_ps_rate       = parse_u64(stream, pos);
    //let version               = version_ps_rate & 0xff00000000000000;
    //let prescale_track        = version_ps_rate & 0x00ffffffff000000;
    //let trate                 = version_ps_rate & 0x0000000000ffffff;
    //hb.version                = ProtocolVersion::from((version >> 56) as u8); 
    hb.version                = ProtocolVersion::from(parse_u8(stream, pos) as u8);
    *pos += 1;
    hb.trate                  = parse_u16(stream, pos) as u64;
    hb.prescale_track         = parse_f32(stream, pos);
    *pos += 2;
    hb.lost_trate             = parse_u16(stream, pos) as u64;
    hb.prescale_gaps          = parse_f32(stream, pos);
    if hb.version == ProtocolVersion::Unknown {
      hb.prescale_gaps  = 0.0;
      hb.prescale_track = 0.0
    }
    *pos += 2;
    Ok(hb)
  }

  fn to_bytestream(&self) -> Vec<u8> {
    let mut bs = Vec::<u8>::with_capacity(Self::SIZE);
    bs.extend_from_slice(&Self::HEAD.to_le_bytes());
    bs.extend_from_slice(&self.total_elapsed.to_le_bytes());
    bs.extend_from_slice(&self.n_events.to_le_bytes());
    bs.extend_from_slice(&self.evq_num_events_last.to_le_bytes());
    bs.extend_from_slice(&self.evq_num_events_avg.to_le_bytes());
    bs.extend_from_slice(&self.n_ev_unsent.to_le_bytes());
    bs.extend_from_slice(&self.n_ev_missed.to_le_bytes());
    bs.push(self.version as u8);
    bs.push(0u8);
    let short_trate = (self.trate & 0x0000000000ffffff) as u16;
    bs.extend_from_slice(&short_trate.to_le_bytes());
    bs.extend_from_slice(&self.prescale_track.to_le_bytes());
    let short_lrate = (self.lost_trate & 0x0000000000ffffff) as u16;
    // FIXME - not needed, just filler
    bs.extend_from_slice(&short_lrate.to_le_bytes());
    bs.extend_from_slice(&short_lrate.to_le_bytes());
    bs.extend_from_slice(&self.prescale_gaps.to_le_bytes());
    //let rate_n_prescale_track =
    //    (((self.version as u8) as u64) << 56)
    //  | (self.prescale_track as u64) << 24 
    //  | (self.trate & 0x0000000000ffffff);
    //bs.extend_from_slice(&rate_n_prescale_track.to_le_bytes());
    //let rate_n_prescale_gaps = 
    // ((self.prescale_gaps as f64)   << 24)
    // | (self.lost_trate & 0x0000000000ffffff);
    //bs.extend_from_slice(&rate_n_prescale_gaps.to_le_bytes());
    bs.extend_from_slice(&Self::TAIL.to_le_bytes());
    bs
  }
}

#[cfg(feature = "random")]
impl FromRandom for MasterTriggerHB {
  fn from_random() -> Self {
    let mut hb = Self::new();
    let mut rng            = rand::rng();
    hb.total_elapsed       = rng.random::<u64>();
    hb.n_events            = rng.random::<u64>();
    hb.evq_num_events_last = rng.random::<u64>();
    hb.evq_num_events_avg  = rng.random::<u64>();
    hb.n_ev_unsent         = rng.random::<u64>();
    hb.n_ev_missed         = rng.random::<u64>();
    hb.trate               = rng.random::<u16>() as u64;
    hb.lost_trate          = rng.random::<u16>() as u64;
    hb.version             = ProtocolVersion::from_random();
    if hb.version != ProtocolVersion::Unknown {
      hb.prescale_gaps       = rng.random::<f32>();
      hb.prescale_track      = rng.random::<f32>();
    }
    hb
  }
}

impl fmt::Display for MasterTriggerHB {
  fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
    let repr = self.pretty_print();
    write!(f, "{}", repr)
  }
} 

impl MoniData for MasterTriggerHB {
  fn get_board_id(&self) -> u8 {
    0
  }
  
  /// Access the (data) members by name 
  fn get(&self, varname : &str) -> Option<f32> {
    match varname {
      "total_elapsed"       => Some(self.total_elapsed as f32),
      "n_events"            => Some(self.n_events as f32),
      "evq_num_events_last" => Some(self.evq_num_events_last as f32),
      "evq_num_events_avg"  => Some(self.evq_num_events_avg as f32),
      "n_ev_unsent"         => Some(self.n_ev_unsent as f32),
      "n_ev_missed"         => Some(self.n_ev_missed as f32),
      "trate"               => Some(self.trate as f32), 
      "lost_trate"          => Some(self.lost_trate as f32),
      "prescale_track"      => Some(self.prescale_track as f32),
      "prescale_gaps"       => Some(self.prescale_gaps as f32),
      _                     => None
    }
  }

  /// A list of the variables in this MoniData
  fn keys() -> Vec<&'static str> {
    vec!["board_id", "total_elapsed", "n_events",
         "evq_num_events_last", "evq_num_events_avg", "n_ev_unsent",
         "n_ev_missed", "trate", "lost_trate", "prescale_track",
         "prescale_gaps"]
  }
}

moniseries!(MasterTriggerHBSeries, MasterTriggerHB);

//-----------------------------------------------------

#[cfg(feature="pybindings")]
#[pymethods]
impl MasterTriggerHB {

  //    version             
  #[getter]
  fn get_total_elapsed(&self) -> u64 {
    self.total_elapsed
  }

  #[getter]
  fn get_evq_mum_events_last(&self) -> u64 {
    self.evq_num_events_last
  }

  #[getter]
  fn get_evq_num_events_avg(&self) -> u64 {
    self.evq_num_events_avg
  }
  
  #[getter]
  fn get_n_ev_unsent(&self) -> u64 {
    self.n_ev_unsent
  }

  #[getter]
  fn get_n_ev_missed(&self) -> u64 {
    self.n_ev_missed
  }
  
  #[getter]
  fn get_trate(&self) -> u64 {
    self.trate
  }

  #[getter]
  fn get_lost_trate(&self) -> u64 {
    self.lost_trate
  }

  #[getter]
  #[pyo3(name="get_prescale_track")]
  fn get_prescale_track_py(&self) -> f32 {
    self.prescale_track
  }

  #[getter]
  #[pyo3(name="get_prescale_gaps")]
  fn get_prescale_gaps_py(&self) -> f32 {
    self.prescale_gaps
  }
}

#[cfg(feature="pybindings")]
pythonize_packable!(MasterTriggerHB);
#[cfg(feature="pybindings")]
pythonize_monidata!(MasterTriggerHB);

//-----------------------------------------------------

#[cfg(feature="random")]
#[test]
fn pack_master_trigger_hb() {
  for _ in 0..100 {
    let hb = MasterTriggerHB::from_random();
    let test : MasterTriggerHB = hb.pack().unpack().unwrap();
    assert_eq!(hb, test);
  }
} 


