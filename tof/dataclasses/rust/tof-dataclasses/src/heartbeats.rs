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
    parse_u64,
    parse_usize,
};

use crate::packets::PacketType;
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
  const SIZE : usize = 76; 
  
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
    Self {
      met,
      n_packets_sent,
      n_packets_incoming,
      n_bytes_written,
      n_evid_missing,
      n_evid_chunksize,
      evid_missing,
      evid_check_len,
      n_pack_write_disk
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
  pub total_elapsed       : u64, //aka met (mission elapsed time)
  pub n_events            : u64,
  pub evq_num_events_last : u64,
  pub evq_num_events_avg  : u64,
  pub n_ev_unsent         : u64,
  pub n_ev_missed         : u64,
  pub trate               : u64,
  pub lost_trate          : u64,
}

  impl MTBHeartbeat {
    pub fn new() -> Self {
      Self {
        total_elapsed       : 0,
        n_events            : 0,
        evq_num_events_last : 0,
        evq_num_events_avg  : 0,
        n_ev_unsent         : 0,
        n_ev_missed         : 0,
        trate               : 0,
        lost_trate          : 0,
      }
    }

    pub fn get_sent_packet_rate(&self) -> f64 {
      self.n_events as f64 / self.total_elapsed as f64
    }


pub fn to_string(&self) -> String {
    let mut repr = String::from("<MTBHeartbeats");
    repr += &(format!("\n \u{1FA90} \u{1FA90} \u{1FA90} \u{1FA90} \u{1FA90} MTB HEARTBEAT \u{1FA90} \u{1FA90} \u{1FA90} \u{1FA90} \u{1FA90} "));
    repr += &(format!("\n MET (Mission Elapsed Time): \t\t{:.1} sec", self.total_elapsed));
    repr += &(format!("\n Num. recorded Events: \t\t{}", self.n_events));
    repr += &(format!("\n Last MTB EVQ size \t\t\t{}", self.evq_num_events_last));
    repr += &(format!("\n Avg. MTB EVQ size (per 30s ): \t{:.2}", self.evq_num_events_avg));
    repr += &(format!("\n trigger rate, recorded: \t\t{:.2} Hz", self.n_events as f64 / self.total_elapsed as f64));
    repr += &(format!("\n trigger rate, from register: \t\t{:.2} Hz", self.trate));
    repr += &(format!("\n lost trg rate, from register: \t{:.2} Hz", self.lost_trate));
    if self.n_ev_unsent > 0 {
        repr += &(format!("\n Num. sent errors: \t\t{}", self.n_ev_unsent).bold());
    }
    if self.n_ev_missed > 0 {
        repr += &(format!("\n Num. missed events: \t\t{}", self.n_ev_missed).bold());
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
      hb.n_ev_unsent         = parse_u64(stream, pos);
      hb.n_ev_missed         = parse_u64(stream, pos);
      hb.trate                  = parse_u64(stream, pos);
      hb.lost_trate             = parse_u64(stream, pos);
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
      bs.extend_from_slice(&self.trate.to_le_bytes());
      bs.extend_from_slice(&self.lost_trate.to_le_bytes());
      bs.extend_from_slice(&Self::TAIL.to_le_bytes());
      bs
    }
  }

  #[cfg(feature = "random")]
  impl FromRandom for MTBHeartbeat {
    fn from_random() -> Self {
    let mut rng             = rand::thread_rng();
    let total_elapsed       = rng.gen::<u64>();
    let n_events            = rng.gen::<u64>();
    let evq_num_events_last = rng.gen::<u64>();
    let evq_num_events_avg  = rng.gen::<u64>();
    let n_ev_unsent      = rng.gen::<u64>();
    let n_ev_missed      = rng.gen::<u64>();
    let trate               = rng.gen::<u64>();
    let lost_trate          = rng.gen::<u64>();
    Self {
      total_elapsed,       
        n_events,            
        evq_num_events_last,
        evq_num_events_avg,
        n_ev_unsent,
        n_ev_missed,
        trate,
        lost_trate,
      }
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
  pub met_seconds           : usize,
  pub n_mte_received_tot    : usize,
  pub n_rbe_received_tot    : usize,
  pub n_rbe_per_te          : usize,
  pub n_rbe_discarded_tot   : usize,
  pub n_mte_skipped         : usize,
  pub n_timed_out           : usize,
  pub n_sent                : usize,
  pub delta_mte_rbe         : usize,
  pub event_cache_size      : usize,
  pub event_id_cache_size   : usize, 
  pub rbe_wo_mte            : usize,
  pub mte_receiver_cbc_len  : usize,
  pub rbe_receiver_cbc_len  : usize,
  pub tp_sender_cbc_len     : usize,
  pub n_rbe_from_past       : usize,
  pub n_rbe_orphan          : usize,
  pub n_ev_wo_evid          : usize,
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
      rbe_wo_mte           : 0,
      mte_receiver_cbc_len : 0,
      rbe_receiver_cbc_len : 0,
      tp_sender_cbc_len    : 0,
      n_ev_wo_evid         : 0,
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
 
pub fn to_string(&self) -> String {
  let mut repr = String::from("<EVTBLDRHearbeats");
  repr += &(format!("\n \u{2B50} \u{2B50} \u{2B50} \u{2B50} \u{2B50} EVENTBUILDER HEARTBTEAT \u{2B50} \u{2B50} \u{2B50} \u{2B50} \u{2B50} "));
  repr += &(format!("\n Num. events sent: \t\t\t{}", self.n_sent).bright_purple());
  repr += &(format!("\n Size of event cache: \t\t\t{}", self.event_cache_size).bright_purple());
  repr += &(format!("\n Size of event ID cache: \t\t{}", self.event_id_cache_size).bright_purple());
  repr += &(format!("\n Num. events timed out \t\t{}", self.n_timed_out).bright_purple());
  repr += &(format!("\n Percent events timed out: \t\t{:.2}%", self.get_timed_out_frac()*(100 as f64)).bright_purple());
  if self.n_sent > 0 {
    repr += &(format!("\n Percent events w/out event ID: \t{:.2}%", (((self.n_ev_wo_evid / self.n_sent) as f64)*(100 as f64))).bright_purple());
  } else { 
    repr += &(format!("\n Percent events w/out event ID: \tN/A").bright_purple());
  }
  if self.n_rbe_received_tot > 0{
    repr += &(format!("\n \u{2504} \u{2504} \u{2504} \u{2504} \u{2504} \u{2504} \u{2504} \u{2504} \u{2504} \u{2504} \u{2504} \u{2504} \u{2504} \u{2504} \u{2504} \u{2504} \u{2504} \u{2504} \u{2504} \u{2504} \u{2504} \u{2504} \u{2504} \u{2504} \u{2504} \u{2504} \u{2504} \u{2504}"));
    repr += &(format!("\n Num. evts with data mangling: \t\t{}", self.data_mangled_ev));
    repr += &(format!("\n Percent events with data mangling: \t\t\t {:.2}", ((self.data_mangled_ev as f64)/(self.n_rbe_received_tot as f64))*(100 as f64)));
  }
  else {repr += &(format!("\n Percent events with data mangling: unable to calculate"));}
  repr += &(format!("\n \u{2504} \u{2504} \u{2504} \u{2504} \u{2504} \u{2504} \u{2504} \u{2504} \u{2504} \u{2504} \u{2504} \u{2504} \u{2504} \u{2504} \u{2504} \u{2504} \u{2504} \u{2504} \u{2504} \u{2504} \u{2504} \u{2504} \u{2504} \u{2504} \u{2504} \u{2504} \u{2504} \u{2504}"));
  repr += &(format!("\n Received MTEvents: \t\t\t{}", self.n_mte_received_tot).bright_purple());
  repr += &(format!("\n Skipped MTEvents: \t\t\t{}", self.n_mte_skipped).bright_purple());
  repr += &(format!("\n Incoming/outgoing MTEvents fraction   {:.2}", self.get_incoming_vs_outgoing_mte()).bright_purple());
  repr += &(format!("\n \u{2504} \u{2504} \u{2504} \u{2504} \u{2504} \u{2504} \u{2504} \u{2504} \u{2504} \u{2504} \u{2504} \u{2504} \u{2504} \u{2504} \u{2504} \u{2504} \u{2504} \u{2504} \u{2504} \u{2504} \u{2504} \u{2504} \u{2504} \u{2504} \u{2504} \u{2504} \u{2504} \u{2504}"));
  repr += &(format!("\n Received RBEvents: \t\t\t{}", self.n_rbe_received_tot).bright_purple());
  repr += &(format!("\n RBEvents Discarded: \t\t\t{}", self.n_rbe_discarded_tot).bright_purple());
  repr += &(format!("\n Percent RBEvents discarded: \t\t{:.2}%", self.get_nrbe_discarded_frac()*(100 as f64)).bright_purple());
  repr += &(format!("\n \u{2504} \u{2504} \u{2504} \u{2504} \u{2504} \u{2504} \u{2504} \u{2504} \u{2504} \u{2504} \u{2504} \u{2504} \u{2504} \u{2504} \u{2504} \u{2504} \u{2504} \u{2504} \u{2504} \u{2504} \u{2504} \u{2504} \u{2504} \u{2504} \u{2504} \u{2504} \u{2504} \u{2504}"));
  if self.n_sent > 0 && self.n_mte_received_tot > 0 {
      repr += &(format!("\n RBEvent/Evts sent       \t\t{:.2}", (self.n_rbe_received_tot as f64/ self.n_sent as f64)).bright_purple());
      repr += &(format!("\n RBEvent/MTEvents       \t\t{:.2}", (self.n_rbe_received_tot as f64 / self.n_mte_received_tot as f64)).bright_purple()); }
  repr += &(format!("\n Num. RBEvents with evid from past:  \t{}", self.n_rbe_from_past).bright_purple());
  repr += &(format!("\n Num. orphan RBEvents: \t\t{}", self.n_rbe_orphan).bright_purple());
  repr += &(format!("\n\n Getting MTE from cache for RBEvent failed {} times :(", self.rbe_wo_mte).bright_blue());
  repr += &(format!("\n \u{2504} \u{2504} \u{2504} \u{2504} \u{2504} \u{2504} \u{2504} \u{2504} \u{2504} \u{2504} \u{2504} \u{2504} \u{2504} \u{2504} \u{2504} \u{2504} \u{2504} \u{2504} \u{2504} \u{2504} \u{2504} \u{2504} \u{2504} \u{2504} \u{2504} \u{2504} \u{2504} \u{2504}"));
  repr += &(format!("\n Ch. len MTE Receiver: \t\t{}", self.mte_receiver_cbc_len).bright_purple());
  repr += &(format!("\n Ch. len RBE Reveiver: \t\t{}", self.rbe_receiver_cbc_len).bright_purple());
  repr += &(format!("\n Ch. len TP Sender: \t\t\t{}", self.tp_sender_cbc_len).bright_purple());
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
        hb.event_id_cache_size  = parse_usize(stream,pos);
        hb.rbe_wo_mte           = parse_usize(stream,pos);
        hb.mte_receiver_cbc_len = parse_usize(stream,pos);
        hb.rbe_receiver_cbc_len = parse_usize(stream,pos);
        hb.tp_sender_cbc_len    = parse_usize(stream,pos);
        hb.n_ev_wo_evid         = parse_usize(stream,pos);
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
          bs.extend_from_slice(&self.event_id_cache_size.to_le_bytes());
          bs.extend_from_slice(&self.rbe_wo_mte.to_le_bytes());
          bs.extend_from_slice(&self.mte_receiver_cbc_len.to_le_bytes());
          bs.extend_from_slice(&self.rbe_receiver_cbc_len.to_le_bytes());
          bs.extend_from_slice(&self.tp_sender_cbc_len.to_le_bytes());
          bs.extend_from_slice(&self.n_ev_wo_evid.to_le_bytes());
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
    let event_id_cache_size = rng.gen::<usize>();
    let rbe_wo_mte = rng.gen::<usize>();
    let mte_receiver_cbc_len = rng.gen::<usize>();
    let rbe_receiver_cbc_len = rng.gen::<usize>();
    let tp_sender_cbc_len = rng.gen::<usize>();
    let n_ev_wo_evid = rng.gen::<usize>();
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
      event_id_cache_size,
      rbe_wo_mte,
      mte_receiver_cbc_len,
      rbe_receiver_cbc_len,
      tp_sender_cbc_len,
      n_mte_received_tot,
      n_ev_wo_evid,
      n_rbe_from_past,
      n_rbe_orphan,
      data_mangled_ev
    }
  }
} 

impl fmt::Display for EVTBLDRHeartbeat {
  fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
    let repr = self.to_string();
    write!(f, "{}", repr)
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
