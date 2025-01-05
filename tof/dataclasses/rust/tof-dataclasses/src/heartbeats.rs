//! Heartbeats - regularly (or on demand) sent 
//! software monitoring data
//!
//! This includes heartbeats for different threads

// use std::collections::btree_map::Range;
use std::fmt;
use colored::*;
use crate::serialization::{
  Serialization,
  SerializationError,
  Packable,
  parse_u8,
  parse_u16,
  parse_f32,
  parse_u64,
  parse_usize,
};

use crate::packets::PacketType;
use crate::version::ProtocolVersion;
// use std::collections::HashMap;

#[cfg(feature="random")]
use crate::FromRandom;
#[cfg(feature="random")]
use rand::Rng;

/// A very general and concise way 
/// to report RB activity
#[derive(Debug, Copy, Clone, PartialEq)]
pub struct RBPing {
  /// RB identifier
  pub rb_id  : u8,
  /// runtime of liftof-rb
  pub uptime : u64,
}

impl RBPing {
  pub fn new() -> Self {
    Self {
      rb_id  : 0,
      uptime : 0,
    }
  }
}

impl Packable for RBPing {
  const PACKET_TYPE : PacketType = PacketType::RBPing;
}

impl Serialization for RBPing {
  
  const HEAD : u16 = 0xAAAA;
  const TAIL : u16 = 0x5555;
  const SIZE : usize = 13; 
  
  fn from_bytestream(stream    : &Vec<u8>, 
                     pos       : &mut usize) 
    -> Result<Self, SerializationError>{
    Self::verify_fixed(stream, pos)?;  
    let mut rb_ping = RBPing::new();
    rb_ping.rb_id   = parse_u8(stream, pos);
    rb_ping.uptime  = parse_u64(stream, pos);
    *pos += 2;
    Ok(rb_ping)
  }
  
  fn to_bytestream(&self) -> Vec<u8> {
    let mut bs = Vec::<u8>::with_capacity(Self::SIZE);
    bs.extend_from_slice(&Self::HEAD.to_le_bytes());
    bs.extend_from_slice(&self.rb_id.to_le_bytes());
    bs.extend_from_slice(&self.uptime.to_le_bytes());
    bs.extend_from_slice(&Self::TAIL.to_le_bytes());
    bs
  }
}

impl Default for RBPing {
  fn default() -> Self {
    Self::new()
  }
}

#[cfg(feature = "random")]
impl FromRandom for RBPing {
  fn from_random() -> Self {
    let mut rng      = rand::thread_rng();
    let rb_id  = rng.gen::<u8>();
    let uptime = rng.gen::<u64>();
    Self {
      rb_id,
      uptime
    }
  }
}

impl fmt::Display for RBPing {
  fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
    //let cc = RBCommand::command_code_to_string(self.command_code);
    let mut repr = String::from("<RBPing");
    repr += &(format!("\n  RB ID      : {}", self.rb_id)); 
    repr += &(format!("\n  Uptime [s] : {}", self.uptime)); 
    write!(f, "{}", repr)
  }
}

#[derive(Debug, Copy, Clone, PartialEq)]
pub struct HeartBeatDataSink {

  /// mission elapsed time in seconds
  pub met                : u64,
  pub n_packets_sent     : u64,
  pub n_packets_incoming : u64,
  /// bytes written to disk
  pub n_bytes_written    : u64,
  /// event id check - missing event ids
  pub n_evid_missing     : u64,
  /// event id check - chunksize
  pub n_evid_chunksize   : u64,
  /// length of incoming buffer for 
  /// the thread
  /// check for missing event ids
  pub evid_missing       : u64,
  /// probe size for missing event id check
  pub evid_check_len     : u64,
  /// number of packets written to disk
  pub n_pack_write_disk  : u64,
  /// length of the incoming channel, which 
  /// is basically packets queued to be sent
  pub incoming_ch_len    : u64,
}

impl HeartBeatDataSink {

  pub fn new() -> Self {
    Self {
      met                : 0,
      n_packets_sent     : 0,
      n_packets_incoming : 0,
      n_bytes_written    : 0,
      n_evid_missing     : 0,
      n_evid_chunksize   : 0,
      evid_missing       : 0,
      evid_check_len     : 0,
      n_pack_write_disk  : 0,
      incoming_ch_len    : 0,
    }
  }

  pub fn get_sent_packet_rate(&self) -> f64 {
    self.n_packets_sent as f64 /  self.met as f64
  }

  pub fn get_mbytes_to_disk_per_sec(&self) -> f64 {
    self.n_bytes_written as f64/(1e6 * self.met as f64)
  }

  pub fn to_string(&self) -> String {
    let mut repr = String::from("<HearBeatDataSink");
    repr += &(format!("\n \u{1F98B} \u{1F98B} \u{1F98B} \u{1F98B} \u{1F98B} DATA SENDER HEARTBEAT \u{1F98B} \u{1F98B} \u{1F98B} \u{1F98B} \u{1F98B}"));
    repr += &(format!("\n Sent {} TofPackets! (packet rate {:.2}/s)", self.n_packets_sent , self.get_sent_packet_rate()));
    repr += &(format!("\n Writing events to disk: {} packets written, data write rate {:.2} MB/sec", self.n_pack_write_disk, self.get_mbytes_to_disk_per_sec()));
    repr += &(format!("\n Missing evid analysis:  {} of {} a chunk of events missing ({:.2}%)", self.evid_missing, self.evid_check_len, 100.0*(self.evid_missing as f64/self.evid_check_len as f64)));
    repr += &(format!("\n Incoming channel length: {}", self.incoming_ch_len));
    repr += &(format!("\n \u{1F98B} \u{1F98B} \u{1F98B} \u{1F98B} \u{1F98B} END HEARTBEAT \u{1F98B} \u{1F98B} \u{1F98B} \u{1F98B} \u{1F98B}"));
    repr 
  }
}

impl Default for HeartBeatDataSink {
  fn default() -> Self {
    Self::new()
  }
}

impl Packable for HeartBeatDataSink {
  const PACKET_TYPE : PacketType = PacketType::HeartBeatDataSink;
}

impl Serialization for HeartBeatDataSink {
  
  const HEAD : u16 = 0xAAAA;
  const TAIL : u16 = 0x5555;
  const SIZE : usize = 84; 
  
  fn from_bytestream(stream    : &Vec<u8>, 
                     pos       : &mut usize) 
    -> Result<Self, SerializationError>{
    Self::verify_fixed(stream, pos)?;  
    let mut hb = HeartBeatDataSink::new();
    hb.met                = parse_u64(stream, pos);
    hb.n_packets_sent     = parse_u64(stream, pos);
    hb.n_packets_incoming = parse_u64(stream, pos);
    hb.n_bytes_written    = parse_u64(stream, pos);
    hb.n_evid_missing     = parse_u64(stream, pos);
    hb.n_evid_chunksize   = parse_u64(stream, pos);
    hb.evid_missing       = parse_u64(stream, pos);
    hb.evid_check_len     = parse_u64(stream, pos);
    hb.n_pack_write_disk  = parse_u64(stream, pos);
    hb.incoming_ch_len    = parse_u64(stream, pos);
    *pos += 2;
    Ok(hb)
  }
  
  fn to_bytestream(&self) -> Vec<u8> {
    let mut bs = Vec::<u8>::with_capacity(Self::SIZE);
    bs.extend_from_slice(&Self::HEAD.to_le_bytes());
    bs.extend_from_slice(&self.met.to_le_bytes());
    bs.extend_from_slice(&self.n_packets_sent.to_le_bytes());
    bs.extend_from_slice(&self.n_packets_incoming.to_le_bytes());
    bs.extend_from_slice(&self.n_bytes_written.to_le_bytes());
    bs.extend_from_slice(&self.n_evid_missing.to_le_bytes());
    bs.extend_from_slice(&self.n_evid_chunksize.to_le_bytes());
    bs.extend_from_slice(&self.evid_missing     .to_le_bytes() );
    bs.extend_from_slice(&self.evid_check_len   .to_le_bytes() );
    bs.extend_from_slice(&self.n_pack_write_disk.to_le_bytes() );
    bs.extend_from_slice(&self.incoming_ch_len.to_le_bytes());
    bs.extend_from_slice(&Self::TAIL.to_le_bytes());
    bs
  }
}

#[cfg(feature = "random")]
impl FromRandom for HeartBeatDataSink {
  fn from_random() -> Self {
    let mut rng            = rand::thread_rng();
    let met                = rng.gen::<u64>();
    let n_packets_sent     = rng.gen::<u64>();
    let n_packets_incoming = rng.gen::<u64>();
    let n_bytes_written    = rng.gen::<u64>();
    let n_evid_missing     = rng.gen::<u64>();
    let n_evid_chunksize   = rng.gen::<u64>();
    let evid_missing       = rng.gen::<u64>();
    let evid_check_len     = rng.gen::<u64>();
    let n_pack_write_disk  = rng.gen::<u64>();
    let incoming_ch_len    = rng.gen::<u64>();
    Self {
      met,
      n_packets_sent,
      n_packets_incoming,
      n_bytes_written,
      n_evid_missing,
      n_evid_chunksize,
      evid_missing,
      evid_check_len,
      n_pack_write_disk,
      incoming_ch_len
    }
  }
}

impl fmt::Display for HeartBeatDataSink {
  fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
    let repr = self.to_string();
    write!(f, "{}", repr)
  }
}
#[cfg(feature = "random")]
#[test]
fn pack_rbping() {
  for _ in 0..100 {
    let ping = RBPing::from_random();
    let test : RBPing = ping.pack().unpack().unwrap();
    assert_eq!(ping, test);
  }
}

#[cfg(feature = "random")]
#[test]
fn pack_heartbeatdatasink() {
  for _ in 0..100 {
    let hb = HeartBeatDataSink::from_random();
    let test : HeartBeatDataSink = hb.pack().unpack().unwrap();
    assert_eq!(hb, test);
  }
}

#[derive(Debug, Copy, Clone, PartialEq)]
pub struct MTBHeartbeat {
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

impl MTBHeartbeat {
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
    self.n_events as f64 / self.total_elapsed as f64
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


  pub fn to_string(&self) -> String {
    let mut repr = format!("<MTBHeartbeats (version : {})", self.version);
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
  

impl Default for MTBHeartbeat {
  fn default () -> Self {
    Self::new()
  }
}

impl Packable for MTBHeartbeat {
  const PACKET_TYPE : PacketType = PacketType::MTBHeartbeat;
}

impl Serialization for MTBHeartbeat {
  const HEAD : u16 = 0xAAAA;
  const TAIL : u16 = 0x5555;
  const SIZE : usize = 68;

  fn from_bytestream(stream    :&Vec<u8>,
                     pos       :&mut usize)
  -> Result<Self, SerializationError>{
    Self::verify_fixed(stream, pos)?;
    let mut hb = MTBHeartbeat::new();
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
impl FromRandom for MTBHeartbeat {
  fn from_random() -> Self {
    let mut hb = Self::new();
    let mut rng             = rand::thread_rng();
    hb.total_elapsed       = rng.gen::<u64>();
    hb.n_events            = rng.gen::<u64>();
    hb.evq_num_events_last = rng.gen::<u64>();
    hb.evq_num_events_avg  = rng.gen::<u64>();
    hb.n_ev_unsent         = rng.gen::<u64>();
    hb.n_ev_missed         = rng.gen::<u64>();
    hb.trate               = rng.gen::<u16>() as u64;
    hb.lost_trate          = rng.gen::<u16>() as u64;
    hb.version             = ProtocolVersion::from_random();
    if hb.version != ProtocolVersion::Unknown {
      hb.prescale_gaps       = rng.gen::<f32>();
      hb.prescale_track      = rng.gen::<f32>();
    }
    hb
  }
}

impl fmt::Display for MTBHeartbeat {
  fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
    let repr = self.to_string();
    write!(f, "{}", repr)
  }
} 

#[cfg(feature="random")]
#[test]
fn pack_mtbheartbeat() {
  for _ in 0..100 {
    let hb = MTBHeartbeat::from_random();
    let test : MTBHeartbeat = hb.pack().unpack().unwrap();
    assert_eq!(hb, test);
  }
} 

#[derive(Debug, Copy, Clone, PartialEq)]
pub struct EVTBLDRHeartbeat {
  /// Mission elapsed time in seconds
  pub met_seconds           : usize,
  /// Total number of received MasterTriggerEvents (from MTB)
  pub n_mte_received_tot    : usize,
  /// Total number of received RBEvents (from all RB)
  pub n_rbe_received_tot    : usize,
  /// Average number of RBEvents per each MTEvent
  pub n_rbe_per_te          : usize,
  /// Total number of discarded RBEvents (accross all boards)
  pub n_rbe_discarded_tot   : usize,
  /// TOtal number of missed MTEvents. "Skipped means" gaps in 
  /// consecutive rising event ids
  pub n_mte_skipped         : usize,
  /// Total number of events that timed out, which means they 
  /// got send before all RBEvents could be associated with 
  /// this event
  pub n_timed_out           : usize,
  /// Total number of events passed on to the gloabl data sink 
  /// thread
  pub n_sent                : usize,
  /// ?
  pub delta_mte_rbe         : usize,
  /// The total size of the current event cache in number of events
  pub event_cache_size      : usize,
  /// In paralel to the event_cache, the event_id cache holds event ids.
  /// This should be perfectly aligned to the event_cache by design.
  pub event_id_cache_size   : usize, 
  /// The total number of hits which we lost due to the DRS being busy
  /// (this is on the Readoutboards)
  pub drs_bsy_lost_hg_hits  : usize,
  /// The total number of RBEvents which do not have a MasterTriggerEvent
  pub rbe_wo_mte            : usize,
  /// The current length of the channel which we use to send events from 
  /// the MasterTrigger thread to the event builder
  pub mte_receiver_cbc_len  : usize,
  /// The current length of the channel whcih we use for all readoutboard
  /// threads to send their events to the event builder
  pub rbe_receiver_cbc_len  : usize,
  /// the current length of the channel which we use to send built events 
  /// to the global data sink thread
  pub tp_sender_cbc_len     : usize,
  /// The total number of RBEvents which have an event id which is SMALLER
  /// than the smallest event id in the event cache. 
  pub n_rbe_from_past       : usize,
  pub n_rbe_orphan          : usize,
  // let's deprecate this!
  pub n_rbe_per_loop          : usize,
  /// The totabl number of events with the "AnyDataMangling" flag set
  pub data_mangled_ev       : usize,
  // pub seen_rbevents         : HashMap<u8, usize>,
}

impl EVTBLDRHeartbeat {
  // pub fn new() -> Self {
  //   let mut seen_rbevents = HashMap::<u8, usize>::new();
  //   for k in 1..47 {
  //     if k == 10 || k ==12 || k == 37 || k == 38 || k == 43 || k == 45 {
  //       continue;
  //     } else {
  //       seen_rbevents.insert(k as u8, 0);
  //     }
  //   }
  pub fn new() -> Self {
    Self {
      met_seconds          : 0,
      n_mte_received_tot   : 0,
      n_rbe_received_tot   : 0,
      n_rbe_per_te         : 0,
      n_rbe_discarded_tot  : 0,
      n_mte_skipped        : 0,
      n_timed_out          : 0,
      n_sent               : 0,
      delta_mte_rbe        : 0,
      event_cache_size     : 0,
      event_id_cache_size  : 0,
      drs_bsy_lost_hg_hits : 0,
      rbe_wo_mte           : 0,
      mte_receiver_cbc_len : 0,
      rbe_receiver_cbc_len : 0,
      tp_sender_cbc_len    : 0,
      n_rbe_per_loop         : 0,
      n_rbe_orphan         : 0,
      n_rbe_from_past      : 0,
      data_mangled_ev      : 0,
      // seen_rbevents        : seen_rbevents, 
    }
  }

  pub fn get_average_rbe_te(&self) -> f64 {
   if self.n_sent > 0 {
     return self.n_rbe_per_te as f64 / self.n_sent as f64;
   }
   0.0
  }

  pub fn get_timed_out_frac(&self) -> f64 {
    if self.n_sent > 0 {
      return self.n_timed_out as f64 / self.n_sent as f64;
    }
    0.0
  }

  // pub fn add_rbevent(&mut self, rb_id : u8) {
  //   *self.seen_rbevents.get_mut(&rb_id).unwrap() += 1;
  // }
  
  pub fn get_incoming_vs_outgoing_mte(&self) -> f64 {
    if self.n_sent > 0 {
      return self.n_mte_received_tot as f64 /  self.n_sent as f64;
    }
    0.0
  }

  pub fn get_nrbe_discarded_frac(&self) -> f64 {
    if self.n_rbe_received_tot > 0 {
     return self.n_rbe_discarded_tot as f64 / self.n_rbe_received_tot as f64;
   }
   0.0
  }
  
  pub fn get_mangled_frac(&self) -> f64 {
    if self.n_mte_received_tot > 0 {
     return self.data_mangled_ev as f64 / self.n_mte_received_tot as f64;
   }
   0.0
  }

  pub fn get_drs_lost_frac(&self) -> f64 {
    if self.n_rbe_received_tot > 0 {
      return self.drs_bsy_lost_hg_hits as f64 / self.n_rbe_received_tot as f64;
    }
    0.0
  }

  pub fn to_string(&self) -> String {
    let mut repr = String::from("");
    repr += &(format!("\n \u{2B50} \u{2B50} \u{2B50} \u{2B50} \u{2B50} EVENTBUILDER HEARTBTEAT \u{2B50} \u{2B50} \u{2B50} \u{2B50} \u{2B50} "));
    repr += &(format!("\n Mission elapsed time (MET) [s]      : {}", self.met_seconds).bright_purple());
    repr += &(format!("\n Num. events sent                    : {}", self.n_sent).bright_purple());
    repr += &(format!("\n Size of event cache                 : {}", self.event_cache_size).bright_purple());
    //repr += &(format!("\n Size of event ID cache              : {}", self.event_id_cache_size).bright_purple());
    repr += &(format!("\n Num. events timed out               : {}", self.n_timed_out).bright_purple());
    repr += &(format!("\n Percent events timed out            : {:.2}%", self.get_timed_out_frac()*(100 as f64)).bright_purple());
    //if self.n_sent > 0 && self.n_rbe_per_loop > 0 {
    //  repr += &(format!("\n Percent events w/out event ID : {:.2}%", (((self.n_rbe_per_loop / self.n_sent) as f64)*(100 as f64))).bright_purple());
    //} else if self.n_rbe_per_loop > 0 { 
    //  repr += &(format!("\n Percent events w/out event ID : N/A").bright_purple());
    //}
    if self.n_mte_received_tot > 0{ 
      repr += &(format!("\n \u{2504} \u{2504} \u{2504} \u{2504} \u{2504} \u{2504} \u{2504} \u{2504} \u{2504} \u{2504} \u{2504} \u{2504} \u{2504} \u{2504} \u{2504} \u{2504} \u{2504} \u{2504} \u{2504} \u{2504} \u{2504} \u{2504} \u{2504} \u{2504} \u{2504} \u{2504} \u{2504} \u{2504}"));
      repr += &(format!("\n Num. evts with ANY data mangling  : {}"     , self.data_mangled_ev));
      repr += &(format!("\n Per. evts with ANY data mangling  : {:.2}%" , self.get_mangled_frac()*(100 as f64)));
    }
    else {repr += &(format!("\n Percent events with data mangling: unable to calculate"));}
    repr += &(format!("\n \u{2504} \u{2504} \u{2504} \u{2504} \u{2504} \u{2504} \u{2504} \u{2504} \u{2504} \u{2504} \u{2504} \u{2504} \u{2504} \u{2504} \u{2504} \u{2504} \u{2504} \u{2504} \u{2504} \u{2504} \u{2504} \u{2504} \u{2504} \u{2504} \u{2504} \u{2504} \u{2504} \u{2504}"));
    repr += &(format!("\n Received MTEvents                   : {}", self.n_mte_received_tot).bright_purple());
    repr += &(format!("\n Skipped MTEvents                    : {}", self.n_mte_skipped).bright_purple());
    repr += &(format!("\n Incoming/outgoing MTEvents fraction : {:.2}", self.get_incoming_vs_outgoing_mte()).bright_purple());
    repr += &(format!("\n \u{2504} \u{2504} \u{2504} \u{2504} \u{2504} \u{2504} \u{2504} \u{2504} \u{2504} \u{2504} \u{2504} \u{2504} \u{2504} \u{2504} \u{2504} \u{2504} \u{2504} \u{2504} \u{2504} \u{2504} \u{2504} \u{2504} \u{2504} \u{2504} \u{2504} \u{2504} \u{2504} \u{2504}"));
    repr += &(format!("\n Received RBEvents                   : {}", self.n_rbe_received_tot).bright_purple());
    repr += &(format!("\n RBEvents Discarded                  : {}", self.n_rbe_discarded_tot).bright_purple());
    repr += &(format!("\n Percent RBEvents discarded          : {:.2}%", self.get_nrbe_discarded_frac()*(100 as f64)).bright_purple());
    repr += &(format!("\n DRS4 busy lost hits                 : {}", self.drs_bsy_lost_hg_hits).bright_purple());
    repr += &(format!("\n RDS4 busy lost hits fraction        : {:.2}%", self.get_drs_lost_frac()*(100.0 as f64)).bright_purple());
    repr += &(format!("\n \u{2504} \u{2504} \u{2504} \u{2504} \u{2504} \u{2504} \u{2504} \u{2504} \u{2504} \u{2504} \u{2504} \u{2504} \u{2504} \u{2504} \u{2504} \u{2504} \u{2504} \u{2504} \u{2504} \u{2504} \u{2504} \u{2504} \u{2504} \u{2504} \u{2504} \u{2504} \u{2504} \u{2504}"));
    if self.n_sent > 0 && self.n_mte_received_tot > 0 {
        repr += &(format!("\n RBEvent/Evts sent               : {:.2}", (self.n_rbe_received_tot as f64/ self.n_sent as f64)).bright_purple());
        repr += &(format!("\n RBEvent/MTEvents                : {:.2}", (self.n_rbe_received_tot as f64 / self.n_mte_received_tot as f64)).bright_purple()); }
    repr += &(format!("\n Current RBevents / iteration        : {:.2}", self.n_rbe_per_loop).bright_purple());
    repr += &(format!("\n Num. RBEvents with evid from past   : {}",  self.n_rbe_from_past).bright_purple());
    repr += &(format!("\n Num. orphan RBEvents                : {}",  self.n_rbe_orphan).bright_purple());
    repr += &(format!("\n\n Getting MTE from cache for RBEvent failed {} times :(", self.rbe_wo_mte).bright_blue());
    repr += &(format!("\n \u{2504} \u{2504} \u{2504} \u{2504} \u{2504} \u{2504} \u{2504} \u{2504} \u{2504} \u{2504} \u{2504} \u{2504} \u{2504} \u{2504} \u{2504} \u{2504} \u{2504} \u{2504} \u{2504} \u{2504} \u{2504} \u{2504} \u{2504} \u{2504} \u{2504} \u{2504} \u{2504} \u{2504}"));
    repr += &(format!("\n Ch. len MTE Receiver                : {}", self.mte_receiver_cbc_len).bright_purple());
    repr += &(format!("\n Ch. len RBE Reveiver                : {}", self.rbe_receiver_cbc_len).bright_purple());
    repr += &(format!("\n Ch. len TP Sender                   : {}", self.tp_sender_cbc_len).bright_purple());
    repr += &(format!("\n \u{2B50} \u{2B50} \u{2B50} \u{2B50} \u{2B50} END EVENTBUILDER HEARTBTEAT \u{2B50} \u{2B50} \u{2B50} \u{2B50} \u{2B50}"));
    repr
  }
}

impl Default for EVTBLDRHeartbeat {
  fn default () -> Self {
    Self::new()
  }
}

impl Packable for EVTBLDRHeartbeat {
  const PACKET_TYPE : PacketType = PacketType::EVTBLDRHeartbeat;
}

impl Serialization for EVTBLDRHeartbeat {
  const HEAD : u16 = 0xAAAA;
  const TAIL : u16 = 0x5555;
  const SIZE : usize = 156; //

  fn from_bytestream(stream : &Vec<u8>, 
                     pos        : &mut usize)
    -> Result<Self, SerializationError>{
    Self::verify_fixed(stream,pos)?;
    let mut hb = EVTBLDRHeartbeat::new();
    hb.met_seconds          = parse_usize(stream,pos);
    hb.n_mte_received_tot   = parse_usize(stream,pos);
    hb.n_rbe_received_tot   = parse_usize(stream,pos);
    hb.n_rbe_per_te         = parse_usize(stream,pos);
    hb.n_rbe_discarded_tot  = parse_usize(stream,pos);
    hb.n_mte_skipped        = parse_usize(stream,pos);
    hb.n_timed_out          = parse_usize(stream,pos);
    hb.n_sent               = parse_usize(stream,pos);
    hb.delta_mte_rbe        = parse_usize(stream,pos);
    hb.event_cache_size     = parse_usize(stream,pos);
    //hb.event_id_cache_size  = parse_usize(stream,pos);
    hb.drs_bsy_lost_hg_hits = parse_usize(stream,pos);
    hb.rbe_wo_mte           = parse_usize(stream,pos);
    hb.mte_receiver_cbc_len = parse_usize(stream,pos);
    hb.rbe_receiver_cbc_len = parse_usize(stream,pos);
    hb.tp_sender_cbc_len    = parse_usize(stream,pos);
    hb.n_rbe_per_loop         = parse_usize(stream,pos);
    hb.n_rbe_from_past      = parse_usize(stream,pos);
    hb.n_rbe_orphan         = parse_usize(stream,pos);
    hb.data_mangled_ev      = parse_usize(stream,pos);
    // hb.seen_rbevents        = HashMap::from(parse_u8(stream, pos));
    *pos += 2;
    Ok(hb)
  }
    
  fn to_bytestream(&self) -> Vec<u8> {
    let mut bs = Vec::<u8>::with_capacity(Self::SIZE);
    bs.extend_from_slice(&Self::HEAD.to_le_bytes());
    bs.extend_from_slice(&self.met_seconds.to_le_bytes());
    bs.extend_from_slice(&self.n_mte_received_tot.to_le_bytes());
    bs.extend_from_slice(&self.n_rbe_received_tot.to_le_bytes());
    bs.extend_from_slice(&self.n_rbe_per_te.to_le_bytes());
    bs.extend_from_slice(&self.n_rbe_discarded_tot.to_le_bytes());
    bs.extend_from_slice(&self.n_mte_skipped.to_le_bytes());
    bs.extend_from_slice(&self.n_timed_out.to_le_bytes());
    bs.extend_from_slice(&self.n_sent.to_le_bytes());
    bs.extend_from_slice(&self.delta_mte_rbe.to_le_bytes());
    bs.extend_from_slice(&self.event_cache_size.to_le_bytes());
    //bs.extend_from_slice(&self.event_id_cache_size.to_le_bytes());
    bs.extend_from_slice(&self.drs_bsy_lost_hg_hits.to_le_bytes());
    bs.extend_from_slice(&self.rbe_wo_mte.to_le_bytes());
    bs.extend_from_slice(&self.mte_receiver_cbc_len.to_le_bytes());
    bs.extend_from_slice(&self.rbe_receiver_cbc_len.to_le_bytes());
    bs.extend_from_slice(&self.tp_sender_cbc_len.to_le_bytes());
    bs.extend_from_slice(&self.n_rbe_per_loop.to_le_bytes());
    bs.extend_from_slice(&self.n_rbe_from_past.to_le_bytes());
    bs.extend_from_slice(&self.n_rbe_orphan.to_le_bytes());
    bs.extend_from_slice(&self.data_mangled_ev.to_le_bytes());
    // bs.push(self.seen_rbevents.to_u8());
    bs.extend_from_slice(&Self::TAIL.to_le_bytes());
    bs
  }
}

#[cfg(feature="random")]
impl FromRandom for EVTBLDRHeartbeat {
  fn from_random() -> Self {
    let mut rng       = rand::thread_rng();
    let met_seconds   = rng.gen::<usize>();
    let n_mte_received_tot = rng.gen::<usize>();
    let n_rbe_received_tot = rng.gen::<usize>();
    let n_rbe_per_te = rng.gen::<usize>();
    let n_rbe_discarded_tot = rng.gen::<usize>();
    let n_mte_skipped = rng.gen::<usize>();
    let n_timed_out = rng.gen::<usize>();
    let n_sent = rng.gen::<usize>();
    let delta_mte_rbe = rng.gen::<usize>();
    let event_cache_size = rng.gen::<usize>();
    //let event_id_cache_size = rng.gen::<usize>();
    let drs_bsy_lost_hg_hits  = rng.gen::<usize>();
    let rbe_wo_mte = rng.gen::<usize>();
    let mte_receiver_cbc_len = rng.gen::<usize>();
    let rbe_receiver_cbc_len = rng.gen::<usize>();
    let tp_sender_cbc_len = rng.gen::<usize>();
    let n_rbe_per_loop = rng.gen::<usize>();
    let n_rbe_from_past = rng.gen::<usize>();
    let n_rbe_orphan = rng.gen::<usize>();
    let data_mangled_ev = rng.gen::<usize>();
    Self {
      met_seconds,
      n_rbe_received_tot,
      n_rbe_per_te,
      n_rbe_discarded_tot,
      n_mte_skipped,
      n_timed_out,
      n_sent,
      delta_mte_rbe,
      event_cache_size,
      // don't randomize this, since it 
      // won't get serialized
      event_id_cache_size : 0,
      drs_bsy_lost_hg_hits,
      rbe_wo_mte,
      mte_receiver_cbc_len,
      rbe_receiver_cbc_len,
      tp_sender_cbc_len,
      n_mte_received_tot,
      n_rbe_per_loop,
      n_rbe_from_past,
      n_rbe_orphan,
      data_mangled_ev
    }
  }
} 

impl fmt::Display for EVTBLDRHeartbeat {
  fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
    let mut repr = String::from("<EVTBLDRHearbeat:   ");
    repr += &self.to_string();
    write!(f, "{}>", repr)
  }
}  

#[cfg(feature="random")]
#[test]
fn pack_evtbldrheartbeat() {
  for _ in 0..100 {
    let hb = EVTBLDRHeartbeat::from_random();
    let test : EVTBLDRHeartbeat = hb.pack().unpack().unwrap();
    assert_eq!(hb, test);
  }
}
