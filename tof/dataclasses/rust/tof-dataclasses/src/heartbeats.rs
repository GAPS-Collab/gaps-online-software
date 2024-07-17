//! Heartbeats - regularly (or on demand) sent 
//! software monitoring data
//!
//! This includes heartbeats for different threads

use std::fmt;
use colored::Colorize;
use crate::serialization::{
    Serialization,
    SerializationError,
    Packable,
    parse_u8,
    parse_u64
};

use crate::packets::PacketType;

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
  pub incoming_ch_len    : u64,
  /// check for missing event ids
  pub evid_missing       : u64,
  /// probe size for missing event id check
  pub evid_check_len     : u64,
  /// number of packets written to disk
  pub n_pack_write_disk  : u64,
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
      incoming_ch_len    : 0,
      evid_missing       : 0,
      evid_check_len     : 0,
      n_pack_write_disk  : 0,
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
    repr += &(format!("  {:<75}", ">> == == == == == == DATA SINK HEARTBEAT  == == == == == == <<".bright_cyan().bold()));
    repr += &(format!("  {:<75} <<", format!(">> ==> Sent {} TofPackets! (packet rate {:.2}/s)", self.n_packets_sent , self.get_sent_packet_rate()).bright_cyan()));
    repr += &(format!("  {:<75} <<", format!(">> ==> Incoming cb channel len {}", self.incoming_ch_len).bright_cyan()));
    repr += &(format!("  {:<75} <<", format!(">> ==> Writing events to disk: {} packets written, data write rate {:.2} MB/sec", self.n_pack_write_disk, self.get_mbytes_to_disk_per_sec()).bright_purple()));
    repr += &(format!("  {:<75} <<", format!(">> ==> Missing evid analysis:  {} of {} a chunk of events missing ({:.2}%)", self.evid_missing, self.evid_check_len, 100.0*(self.evid_missing as f64/self.evid_check_len as f64)).bright_purple()));
    repr += &(format!("  {:<75}", ">> == == == == == == == == == == == == == == == == == == == <<".bright_cyan().bold()));
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
    hb.incoming_ch_len    = parse_u64(stream, pos);
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
    bs.extend_from_slice(&self.incoming_ch_len  .to_le_bytes() );
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
    let incoming_ch_len    = rng.gen::<u64>();
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
      incoming_ch_len,
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



//println!("  {:<75}", ">> == == == == == == DATA SINK HEARTBEAT  == == == == == == <<".bright_cyan().bold());
//      println!("  {:<75} <<", format!(">> ==> Sent {} TofPackets! (packet rate {:.2}/s)", n_pack_sent ,packet_rate).bright_cyan());
//      println!("  {:<75} <<", format!(">> ==> Incoming cb channel len {}", incoming.len()).bright_cyan());
//      println!("  {:<75} <<", format!(">> ==> Writing events to disk: {} packets written, data write rate {:.2} MB/sec", n_pack_write_disk, bytes_sec_disk/(1e6*met_time_secs as f64)).bright_purple());
//      println!("  {:<75} <<", format!(">> ==> Missing evid analysis:  {} of {} a chunk of events missing ({:.2}%)", evid_missing, evid_check_len, 100.0*(evid_missing as f64/evid_check_len as f64)).bright_purple());
//
//      println!("  {:<75}", ">> == == == == == == == == == == == == == == == == == == == <<".bright_cyan().bold());
//      timer = Instant::now();
//}
//
//
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
  pub n_ev_unsent          : u64,
  pub n_ev_missed         : u64,
  pub trate               : u64,
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
      }
    }
    pub fn get_sent_packet_rate(&self) -> f64 {
      self.n_events as f64 / self.total_elapsed as f64
    }
    pub fn to_string(&self) -> String {
      let mut repr = String::from("<MTBHeartbeats");
      repr += &(format!("   {:<75}", ">> == == == == == == MTB HEARTBEAT == == == == == == <<".bright_cyan().bold()));
      repr += &(format!("   {:<75} <<", format!(">> ==> MET (Mission Elapsed Time) (sec) {:.1}",self.total_elapsed).bright_blue()));
      repr += &(format!("   {:<75} <<", format!(">> ==> Recorded Events                  {}", self.n_events).bright_blue()));
      repr += &(format!("   {:<75} <<", format!(">> ==> Last MTB EVQ size                {}", self.evq_num_events_last).bright_blue()));
      repr += &(format!("   {:<75} <<", format!(">> ==> Avg. MTB EVQ size (per 30s )     {:.2}", self.evq_num_events_avg).bright_blue()));
      repr += &(format!("   {:<75} <<", format!(">> ==> -- trigger rate, recorded  (Hz)  {:.2}", self.n_events as f64/self.total_elapsed as f64).bright_blue()));
      repr
    //   match TRIGGER_RATE.get(&mut bus) {
    //     Ok(trate) => {
    //       repr += &(format!("  {:<75}", ">> ==> -- trigger rate, from reg. (Hz)  {}", self.trate.bright_blue()));
    //     }
    //     Err(err) => {
    //       error!("Unable to query {}! {err}", TRIGGER_RATE);
    //       repr += &(format!("  {:<60}", ">> ==> -- trigger rate, from reg. (Hz)   N/A").bright_blue());
    //     }
    //   }
    //   match LOST_TRIGGER_RATE.get(&mut bus) {
    //     Ok(trate) => {
    //       repr += &(format!("  {:<75}", ">> ==> -- lost trg rate, from reg. (Hz)   {}", self.trate).bright_blue());
    //     }
    //     Err(err) => {
    //       error!("Unable to query {}! {err}", LOST_TRIGGER_RATE);
    //       repr += &(format!("  {:<75}", ">> ==> -- lost trigger rate, from reg. (Hz)   N/A").bright_blue());
    //     }
    //   }
    //   if n_ev_unsent > 0 {
    //     repr += &(format!("  {}{}{}", ">> ==> ".yellow().bold(),self.n_ev_unsent, " sent errors                       <<".yellow().bold()));
    //   }
    //   if n_ev_missed > 0 {
    //     repr += &(format!("  {}{}{}", ">> ==> ".yellow().bold(),self.n_events, " missed events                       <<".yellow().bold()));
    //   }
    //   repr += &(format!("  {:<75}", ">> == == == == == == ==  END HEARTBEAT = ==  == == == == ==".bright_blue().bold()))
    
    // }
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
    const SIZE : usize = 60;

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
      bs.extend_from_slice(&Self::TAIL.to_le_bytes());
      bs
    }
  }

  #[cfg(feature = "random")]
  impl FromRandom for MTBHeartbeat {
    fn from_random() -> Self {
    let mut_rng             = rand::thread_rng();
    let total_elapsed       = rng.gen::<u64>();
    let n_events            = rng.gen::<u64>();
    let evq_num_events_last = rng.gen::<u64>();
    let evq_num_events_avg  = rng.gen::<u64>();
    let n_ev_unsent      = rng.gen::<u64>();
    let n_ev_missed      = rng.gen::<u64>();
    let trate               = rng.gen::<u64>();
    Self {
      total_elapsed,       
        n_events,            
        evq_num_events_last,
        evq_num_events_avg,
        n_ev_unsent,
        n_ev_missed,
        trate,
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
