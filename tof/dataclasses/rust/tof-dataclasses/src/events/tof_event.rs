//! Event strucutures for data reconrded by the tof
//!

use std::time::Instant;
use std::fmt;

cfg_if::cfg_if! {
  if #[cfg(feature = "random")]  {
    use crate::FromRandom;
    use rand::Rng;
  }
}

use crate::serialization::{
  Serialization,
  Packable,
  parse_u8,
  parse_u16,
  parse_u32,
  parse_u64,
  parse_f32,
  search_for_u16
};

use crate::packets::PacketType;
use crate::errors::SerializationError;

use crate::events::{
  MasterTriggerEvent,
  RBEvent,
  TofHit,
  RBWaveform,
  //RBMissingHit,
  TriggerType,
  EventStatus,
  transcode_trigger_sources,
};

use crate::events::master_trigger::{
  LTBThreshold,
  LTB_CHANNELS
};

use crate::ProtocolVersion;

cfg_if::cfg_if! {
  if #[cfg(feature = "database")]  {
    use crate::database::DsiJChPidMapping;
    use crate::database::Paddle;
    use std::collections::HashMap;
  }
}

// #[cfg(feature ="database")]
// use crate::database::Paddle;

// This looks like a TODO
#[derive(Debug, Copy, Clone, PartialEq, serde::Deserialize, serde::Serialize)]
#[repr(u8)]
pub enum CompressionLevel {
  Unknown = 0u8,
  None    = 10u8,
}

impl fmt::Display for CompressionLevel {
  fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
    let r = serde_json::to_string(self).unwrap_or(
      String::from("Error: cannot unwrap this CompressionLevel"));
    write!(f, "<CompressionLevel: {}>", r)
  }
}

impl From<u8> for CompressionLevel {
  fn from(value: u8) -> Self {
    match value {
      0u8  => CompressionLevel::Unknown,
      10u8 => CompressionLevel::None,
      _    => CompressionLevel::Unknown
    }
  }
}

#[derive(Debug, Copy, Clone, PartialEq, serde::Deserialize, serde::Serialize)]
#[repr(u8)]
pub enum EventQuality {
  Unknown        =  0u8,
  Silver         = 10u8,
  Gold           = 20u8,
  Diamond        = 30u8,
  FourLeafClover = 40u8,
}

impl fmt::Display for EventQuality {
  fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
    let r = serde_json::to_string(self).unwrap_or(
      String::from("Error: cannot unwrap this EventQuality"));
    write!(f, "<EventQuality: {}>", r)
  }
}

impl From<u8> for EventQuality {
  fn from(value: u8) -> Self {
    match value {
      0u8  => EventQuality::Unknown,
      10u8 => EventQuality::Silver,
      20u8 => EventQuality::Gold,
      30u8 => EventQuality::Diamond,
      40u8 => EventQuality::FourLeafClover,
      _    => EventQuality::Unknown
    }
  }
}

// FIXME - no PartialEq (or we have to implent it
// since the times will never be equal
#[derive(Debug, Clone)]
pub struct TofEvent {

  pub compression_level : CompressionLevel,
  pub quality           : EventQuality,
  pub header            : TofEventHeader,
  pub mt_event          : MasterTriggerEvent,
  pub rb_events         : Vec::<RBEvent>,
  //pub missing_hits      : Vec::<RBMissingHit>, 
  
  // won't get serialized
  pub creation_time     : Instant,
  pub write_to_disk     : bool, 
}

impl fmt::Display for TofEvent {
  fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
    write!(f, 
"<TofEvent:
     quality        :  {}
     {} 
     {}
     n RBEvents      : {}>"
            ,self.quality,
            self.header,
            self.mt_event,
            self.rb_events.len())
  }
}

impl Default for TofEvent {
  fn default() -> Self {
    Self::new()
  }
}

impl TofEvent {

  pub fn new() -> Self {
    let creation_time = Instant::now();
    Self {
      compression_level : CompressionLevel::Unknown,
      quality           : EventQuality::Unknown,
      header            : TofEventHeader::new(),
      mt_event          : MasterTriggerEvent::new(),
      rb_events         : Vec::<RBEvent>::new(),
      //missing_hits      : Vec::<RBMissingHit>::new(), 
      creation_time     : creation_time,
      write_to_disk     : true,
    }
  }

  /// Compare the MasterTriggerEvent::trigger_hits with 
  /// the actual hits to determine from which paddles
  /// we should have received HG hits (from waveforms)
  /// but we did not get them
  ///
  /// WARNING: The current implementation of this is 
  /// rather slow and not fit for production use
  /// FIXME - rewrite as a closure
  #[cfg(feature="database")]
  pub fn get_missing_paddles_hg(&self, pid_map : &DsiJChPidMapping) -> Vec<u8> {
    let mut missing = Vec::<u8>::new();
    for th in self.mt_event.get_trigger_hits() {
      let pid = pid_map.get(&th.0).unwrap().get(&th.1).unwrap().get(&th.2.0).unwrap().0;
      let mut found = false;
      for h in self.get_hits() {
        if h.paddle_id == pid {
          found = true;
          break
        }
      }
      if !found {
        missing.push(pid);
      }
    }
    missing
  }

  /// Get the triggered paddle ids
  ///
  /// Warning, this might be a bit slow
  #[cfg(feature="database")]
  pub fn get_triggered_paddles(&self, pid_map :   DsiJChPidMapping) -> Vec<u8> {
    let mut paddles = Vec::<u8>::new();
    for th in self.mt_event.get_trigger_hits() {
      let pid = pid_map.get(&th.0).unwrap().get(&th.1).unwrap().get(&th.2.0).unwrap().0;
      paddles.push(pid);
    }
    paddles
  }
  

  pub fn extract_event_id_from_stream(stream : &Vec<u8>) 
    -> Result<u32, SerializationError> {
    // 2 + 2 + 2 + 2 + 4
    let evid = parse_u32(stream, &mut 12);
    Ok(evid)
  }

  pub fn age(&self) -> u64 {
    self.creation_time.elapsed().as_secs()
  }

  /// Simple check if the event contains as much RBEvents 
  /// as expected from the provided boards masks by the MTB
  pub fn is_complete(&self) -> bool {
    self.mt_event.get_rb_link_ids().len() == self.rb_events.len()
  }
 
  /// A more advanced check, where events which are not in the 
  /// provided mtb_link_id list don't count for completion
  pub fn is_complete_masked(&self, mtb_link_ids_excluded : &Vec::<u8>) -> bool {
    let mut expected_events = 0usize;
    for k in &self.mt_event.get_rb_link_ids() {
      if !mtb_link_ids_excluded.contains(k) {
        expected_events += 1
      }
    }
    self.rb_events.len() == expected_events
  }

  /// Encode the sizes of the vectors holding the 
  /// into an u32
  ///
  /// We have one byte (256) max length per vector.
  pub fn construct_sizes_header(&self) -> u32 {
     let rb_event_len = self.rb_events.len() as u32;
     // disable missing hits
     //let miss_len     = self.missing_hits.len() as u32;
     let miss_len     = 0u32;
     let mut mask     = 0u32;
     mask = mask | rb_event_len;
     mask = mask | (miss_len << 8);
     mask
  }

  pub fn decode_size_header(mask : &u32) 
    -> (usize, usize) {
    let rb_event_len = (mask & 0xFF)        as usize;
    let miss_len     = ((mask & 0xFF00)     >> 8)  as usize;
    (rb_event_len, miss_len)
  }
  
  pub fn get_combined_vector_sizes(&self) -> usize {
    self.rb_events.len() 
    //+ self.missing_hits.len() 
  }

  /// Get all waveforms of all RBEvents in this event
  pub fn get_rbwaveforms(&self) -> Vec<RBWaveform> {
    let mut wf = Vec::<RBWaveform>::new();
    for ev in &self.rb_events {
      wf.extend_from_slice(&ev.get_rbwaveforms());
    }
    wf
  }

  /// Get all hits of all RBEvents in this event
  pub fn get_hits(&self) -> Vec<TofHit> {
    let mut hits = Vec::<TofHit>::new();
    for ev in &self.rb_events {
      for h in &ev.hits {
        hits.push(*h);
      }
    }
    hits
  }

  /// Check if th eassociated RBEvents have any of their
  /// mangling stati set
  pub fn has_any_mangling(&self) -> bool {
    for rbev in &self.rb_events {
      if rbev.status == EventStatus::CellAndChnSyncErrors 
      || rbev.status == EventStatus::CellSyncErrors 
      || rbev.status == EventStatus::ChnSyncErrors {
        return true;
      }
    }
    false
  }

  pub fn get_summary(&self) -> TofEventSummary {
    let mut summary         = TofEventSummary::new();
    // generate an aggregate status from all the different stati
    summary.status          = self.mt_event.event_status;
    if self.has_any_mangling() {
      summary.status = EventStatus::AnyDataMangling;
    }
    // FIXME - this is not trigger paddles, but trigger hits!
    summary.trigger_sources    = self.mt_event.trigger_source;
    summary.n_trigger_paddles  = self.mt_event.get_trigger_hits().len() as u8;
    summary.event_id           = self.header.event_id;
    // truncate the run id to u16
    summary.run_id             = (self.header.run_id & 0x0000ffff) as u16;
    // FIXME we set the protocol version here, but that should propably 
    // go elsewhere
    summary.version            = ProtocolVersion::V1;
    let mt_timestamp           = (self.mt_event.get_timestamp_abs48() as f64/1000.0).floor()  as u64; 
    summary.timestamp32        = (mt_timestamp  & 0x00000000ffffffff ) as u32;
    summary.timestamp16        = ((mt_timestamp & 0x0000ffff00000000 ) >> 32) as u16;
    //summary.primary_beta       = self.header.primary_beta; 
    //summary.primary_charge     = self.header.primary_charge; 
    summary.dsi_j_mask         = self.mt_event.dsi_j_mask;
    summary.channel_mask       = self.mt_event.channel_mask.clone();
    summary.mtb_link_mask      = self.mt_event.mtb_link_mask;
    summary.drs_dead_lost_hits = self.get_lost_hits();
    summary.hits               = Vec::<TofHit>::new();
    for ev in &self.rb_events {
      for hit in &ev.hits {
        let h = hit.clone();
        if summary.version == ProtocolVersion::V1 {
          if h.paddle_id <= 60 {
            summary.n_hits_cbe += 1;
            summary.tot_edep_cbe += h.get_edep();
          }
          else if h.paddle_id <= 108 && h.paddle_id > 60 {
            summary.n_hits_umb += 1;
            summary.tot_edep_umb += h.get_edep();
          }
          else {
            summary.n_hits_cor += 1;
            summary.tot_edep_cor += h.get_edep();
          }
        }
        summary.hits.push(h);
      }
    }
    summary
  }
  
  /// The number of hits we did not get 
  /// becaue of the DRS busy
  pub fn get_lost_hits(&self) -> u16 {
    let mut lost_hits = 0u16;
    for rbev in &self.rb_events {
      if rbev.header.drs_lost_trigger() {
        let mut nhits = rbev.header.get_nchan() as u16;
        if nhits > 0 {
          nhits -= 1;
        }
        lost_hits += nhits;
      }
    }
    lost_hits
  }
}

impl Packable for TofEvent {
  const PACKET_TYPE : PacketType = PacketType::TofEvent;
}

impl Serialization for TofEvent {
  
  const HEAD               : u16   = 43690; //0xAAAA
  const TAIL               : u16   = 21845; //0x5555

  // unify to_le_bytes and other in to_bytestream ? TODO
  fn to_bytestream(&self) -> Vec<u8> {
    let mut stream = Vec::<u8>::new();
    stream.extend_from_slice(&Self::HEAD.to_le_bytes());
    stream.extend_from_slice(&(self.compression_level as u8).to_le_bytes());
    stream.extend_from_slice(&(self.quality as u8).to_le_bytes());
    stream.extend_from_slice(&self.header.to_bytestream());
    stream.extend_from_slice(&self.mt_event.to_bytestream());
    let sizes_header = self.construct_sizes_header();
    stream.extend_from_slice(&sizes_header.to_le_bytes());
    for k in 0..self.rb_events.len() {
      stream.extend_from_slice(&self.rb_events[k].to_bytestream());
    }
    //for k in 0..self.missing_hits.len() {
    //  stream.extend_from_slice(&self.missing_hits[k].to_bytestream());
    //}
    stream.extend_from_slice(&Self::TAIL.to_le_bytes());
    stream
  }
  
  fn from_bytestream(stream    : &Vec<u8>, 
                     pos       : &mut usize) 
    -> Result<Self, SerializationError>{
    let mut event = Self::new();
    let head_pos = search_for_u16(Self::HEAD, stream, *pos)?; 
    *pos = head_pos + 2;
    event.compression_level = CompressionLevel::try_from(parse_u8(stream, pos)).unwrap();
    event.quality           = EventQuality::try_from(parse_u8(stream, pos)).unwrap();
    event.header            = TofEventHeader::from_bytestream(stream, pos)?;
    event.mt_event          = MasterTriggerEvent::from_bytestream(stream, pos)?;
    let v_sizes = Self::decode_size_header(&parse_u32(stream, pos));
    for k in 0..v_sizes.0 {
      match RBEvent::from_bytestream(stream, pos) {
        Err(err) => error!("Expected RBEvent {} of {}, but got serialization error {}!", k,  v_sizes.0, err),
        Ok(ev) => {
          event.rb_events.push(ev);
        }
      }
    }
    let tail = parse_u16(stream, pos);
    if tail != Self::TAIL {
      error!("Decoding of TAIL failed! Got {} instead!", tail);
    }
    Ok(event)
  }
}

#[cfg(feature="random")]
impl FromRandom for TofEvent {

  fn from_random() -> Self {
    let mut event   = Self::new();
    event.mt_event  = MasterTriggerEvent::from_random();
    event.header    = TofEventHeader::from_random();
    let mut rng     = rand::thread_rng();
    let n_boards    = rng.gen_range(1..41) as u8;
    //let n_boards    = rng.gen::<u8>() as usize;
    //let n_paddles   = rng.gen::<u8>() as usize;
    for _ in 0..n_boards {
      event.rb_events.push(RBEvent::from_random());
    }
    //for _ in 0..n_missing {
    //  event.missing_hits.push(RBMissingHit::from_random());
    //}
    // for now, we do not randomize CompressionLevel and qualtiy
    //event.compression_level : CompressionLevel::,
    //event.quality           : EventQuality::Unknown,
    event
  }
}

impl From<MasterTriggerEvent> for TofEvent {
  fn from(mte : MasterTriggerEvent) -> Self {
    let mut te : TofEvent = Default::default();
    te.mt_event = mte;
    te.header.event_id = te.mt_event.event_id;
    te
  }
}

/// The main event structure
#[derive(Debug, Clone, PartialEq)]
pub struct TofEventHeader  {

  pub run_id       : u32,
  pub event_id     : u32,
  // lost hits insead of n_hit outer and n_hit inner tof
  pub drs_dead_lost_hits  : u8,
  pub rsvd0              : u8,
  // the timestamp shall be comging from the master trigger
  pub timestamp_32 : u32,
  pub timestamp_16 : u16, // -> 14 byres
  

  // reconstructed quantities
  pub primary_beta        : u16, 
  pub primary_beta_unc    : u16, 
  pub primary_charge      : u16, 
  pub primary_charge_unc  : u16, 
  pub primary_outer_tof_x : u16, 
  pub primary_outer_tof_y : u16, 
  pub primary_outer_tof_z : u16, 
  pub primary_inner_tof_x : u16, 
  pub primary_inner_tof_y : u16, 
  pub primary_inner_tof_z : u16, //-> 20bytes primary 

  //pub nhit_outer_tof       : u8,  
  // no need to save this, can be 
  // rereated from paddle_info.size() - nhit_outer_tof
  //pub nhit_inner_tof       : u8, 

  pub trigger_info         : u8,
  pub ctr_etx              : u8,

  // this field can be debated
  // the reason we have it is 
  // that for de/serialization, 
  // we need to know the length 
  // of the expected bytestream.
  pub n_paddles           : u8, // we don't have more than 
                               // 256 paddles.
}

impl TofEventHeader {
  
  pub fn new() -> Self {
    Self {
      run_id               : 0,
      event_id             : 0,
      drs_dead_lost_hits   : 0,
      rsvd0               : 0,
      timestamp_32         : 0,
      timestamp_16         : 0,
      primary_beta         : 0, 
      primary_beta_unc     : 0, 
      primary_charge       : 0, 
      primary_charge_unc   : 0, 
      primary_outer_tof_x  : 0, 
      primary_outer_tof_y  : 0, 
      primary_outer_tof_z  : 0, 
      primary_inner_tof_x  : 0, 
      primary_inner_tof_y  : 0, 
      primary_inner_tof_z  : 0,  
      //nhit_outer_tof       : 0,  
      //nhit_inner_tof       : 0, 
      trigger_info         : 0,
      ctr_etx              : 0,
      n_paddles            : 0  
    }
  }
}

impl Serialization for TofEventHeader {
  const HEAD               : u16   = 0xAAAA;
  const TAIL               : u16   = 0x5555;
  const SIZE               : usize = 43; 

  fn from_bytestream(stream : &Vec<u8>, pos : &mut usize)
     -> Result<Self, SerializationError> {
    Self::verify_fixed(stream, pos)?;
    let mut event             = Self::new();
    event.run_id              = parse_u32(stream, pos);
    event.event_id            = parse_u32(stream, pos);
    event.timestamp_32        = parse_u32(stream, pos);
    event.timestamp_16        = parse_u16(stream, pos);
    event.primary_beta        = parse_u16(stream, pos);
    event.primary_beta_unc    = parse_u16(stream, pos);
    event.primary_charge      = parse_u16(stream, pos);
    event.primary_charge_unc  = parse_u16(stream, pos);
    event.primary_outer_tof_x = parse_u16(stream, pos);
    event.primary_outer_tof_y = parse_u16(stream, pos);
    event.primary_outer_tof_z = parse_u16(stream, pos);
    event.primary_inner_tof_x = parse_u16(stream, pos);
    event.primary_inner_tof_y = parse_u16(stream, pos);
    event.primary_inner_tof_z = parse_u16(stream, pos); 
    event.drs_dead_lost_hits  = parse_u8(stream, pos);
    event.rsvd0              = parse_u8(stream, pos);
    //event.nhit_outer_tof      = parse_u8(stream, pos);
    //event.nhit_inner_tof      = parse_u8(stream, pos);
    event.trigger_info        = parse_u8(stream, pos);
    event.ctr_etx             = parse_u8(stream, pos);
    event.n_paddles           = parse_u8(stream, pos); 
    *pos += 2; 
    Ok(event) 
  }
  
  fn to_bytestream(&self) -> Vec<u8> {
    let mut bytestream = Vec::<u8>::with_capacity(Self::SIZE);
    bytestream.extend_from_slice(&Self::HEAD                     .to_le_bytes());
    bytestream.extend_from_slice(&self.run_id                    .to_le_bytes());
    bytestream.extend_from_slice(&self.event_id                  .to_le_bytes());
    bytestream.extend_from_slice(&self.timestamp_32              .to_le_bytes());
    bytestream.extend_from_slice(&self.timestamp_16              .to_le_bytes());
    bytestream.extend_from_slice(&self.primary_beta              .to_le_bytes());
    bytestream.extend_from_slice(&self.primary_beta_unc          .to_le_bytes());
    bytestream.extend_from_slice(&self.primary_charge            .to_le_bytes());
    bytestream.extend_from_slice(&self.primary_charge_unc        .to_le_bytes());
    bytestream.extend_from_slice(&self.primary_outer_tof_x       .to_le_bytes());
    bytestream.extend_from_slice(&self.primary_outer_tof_y       .to_le_bytes());
    bytestream.extend_from_slice(&self.primary_outer_tof_z       .to_le_bytes());
    bytestream.extend_from_slice(&self.primary_inner_tof_x       .to_le_bytes());
    bytestream.extend_from_slice(&self.primary_inner_tof_y       .to_le_bytes());
    bytestream.extend_from_slice(&self.primary_inner_tof_z       .to_le_bytes());
    //bytestream.extend_from_slice(&self.rb_events
    //bytestream.extend_from_slice(&self.nhit_outer_tof            .to_le_bytes());
    //bytestream.extend_from_slice(&self.nhit_inner_tof            .to_le_bytes());
    bytestream.extend_from_slice(&self.drs_dead_lost_hits        .to_le_bytes());
    bytestream.extend_from_slice(&self.rsvd0                    .to_le_bytes());
    bytestream.extend_from_slice(&self.trigger_info              .to_le_bytes());
    bytestream.extend_from_slice(&self.ctr_etx                   .to_le_bytes());
    bytestream.extend_from_slice(&self.n_paddles                 .to_le_bytes());
    bytestream.extend_from_slice(&Self::TAIL        .to_le_bytes()); 
    bytestream
  }
}


impl Default for TofEventHeader {
  fn default() -> Self {
    Self::new()
  }
}

impl From<&MasterTriggerEvent> for TofEventHeader {
  fn from(mte : &MasterTriggerEvent) -> Self {
    let mut te               = Self::new();
    te.event_id              = mte.event_id;
    te.timestamp_32          = mte.timestamp;
    te.n_paddles             = mte.get_trigger_hits().len() as u8;
    te
  }
}

impl fmt::Display for TofEventHeader {
  fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
    let mut repr = String::from("<TofEventHeader"); 
    repr += &(format!("\n  Run   ID          : {}", self.run_id              ));
    repr += &(format!("\n  Event ID          : {}", self.event_id            ));
    repr += &(format!("\n  Timestamp 32      : {}", self.timestamp_32        ));
    repr += &(format!("\n  Timestamp 16      : {}", self.timestamp_16        ));
    repr += &(format!("\n  DRS LOST HITS     : {}", self.drs_dead_lost_hits  ));
    repr += &(format!("\n  Prim. Beta        : {}", self.primary_beta        ));
    repr += &(format!("\n  Prim. Beta Unc    : {}", self.primary_beta_unc    ));
    repr += &(format!("\n  Prim. Charge      : {}", self.primary_charge      ));
    repr += &(format!("\n  Prim. Charge unc  : {}", self.primary_charge_unc  ));
    repr += &(format!("\n  Prim. Outer Tof X : {}", self.primary_outer_tof_x ));
    repr += &(format!("\n  Prim. Outer Tof Y : {}", self.primary_outer_tof_y ));
    repr += &(format!("\n  Prim. Outer Tof Z : {}", self.primary_outer_tof_z ));
    repr += &(format!("\n  Prim. Inner Tof X : {}", self.primary_inner_tof_x ));
    repr += &(format!("\n  Prim. Inner Tof Y : {}", self.primary_inner_tof_y ));
    repr += &(format!("\n  Prim. Inner Tof Z : {}", self.primary_inner_tof_z ));
    //repr += &(format!("\n  NHit  Outer Tof   : {}", self.nhit_outer_tof      ));
    //repr += &(format!("\n  NHit  Inner Tof   : {}", self.nhit_inner_tof      ));
    repr += &(format!("\n  TriggerInfo       : {}", self.trigger_info        ));
    repr += &(format!("\n  Ctr ETX           : {}", self.ctr_etx             ));
    repr += &(format!("\n  NPaddles          : {}", self.n_paddles           ));
    repr += ">";
  write!(f,"{}", repr)
  }
}

#[cfg(feature="random")]
impl FromRandom for TofEventHeader {

  fn from_random() -> Self {
    let mut rng     = rand::thread_rng();
    Self { 
      run_id               : rng.gen::<u32>(),
      event_id             : rng.gen::<u32>(),
      drs_dead_lost_hits   : rng.gen::<u8>(),
      rsvd0               : rng.gen::<u8>(),
      timestamp_32         : rng.gen::<u32>(),
      timestamp_16         : rng.gen::<u16>(),
      primary_beta         : rng.gen::<u16>(), 
      primary_beta_unc     : rng.gen::<u16>(), 
      primary_charge       : rng.gen::<u16>(), 
      primary_charge_unc   : rng.gen::<u16>(), 
      primary_outer_tof_x  : rng.gen::<u16>(), 
      primary_outer_tof_y  : rng.gen::<u16>(), 
      primary_outer_tof_z  : rng.gen::<u16>(), 
      primary_inner_tof_x  : rng.gen::<u16>(), 
      primary_inner_tof_y  : rng.gen::<u16>(), 
      primary_inner_tof_z  : rng.gen::<u16>(),  
      trigger_info         : rng.gen::<u8>(),
      ctr_etx              : rng.gen::<u8>(),
      n_paddles            : rng.gen::<u8>()  
    }
  }
}

/// De-facto the main event class
///
/// TofEventSummary provides a list of extracted
/// hits from the ReadoutBoards as well as 
/// information about the trigger system.
#[derive(Debug, Clone, PartialEq)]
pub struct TofEventSummary {
  pub status            : EventStatus,
  pub version           : ProtocolVersion,
  pub quality           : u8,
  pub trigger_sources   : u16,

  /// the number of triggered paddles coming
  /// from the MTB directly. This might NOT be
  /// the same as the number of hits!
  pub n_trigger_paddles  : u8,
  pub event_id           : u32,
  /// NEW - uses the space for primary_beta,
  /// which we won't have anyway
  pub run_id             : u16,
  pub timestamp32        : u32,
  pub timestamp16        : u16,
  /// DEPRECATED, won't get serialized
  /// reconstructed primary beta
  pub primary_beta       : u16, 
  /// DEPRECATED, won't get serialized
  /// reconstructed primary charge
  pub primary_charge     : u16, 
  /// scalar number of hits missed in
  /// this event due to DRS on the RB
  /// being busy
  pub drs_dead_lost_hits : u16, 
  pub dsi_j_mask         : u32,
  pub channel_mask       : Vec<u16>,
  pub mtb_link_mask      : u64,
  pub hits               : Vec<TofHit>,
  // a bunch of calculated variablels, used 
  // for online interesting event search
  // these will be only available in ProtocolVersion 1
  pub n_hits_umb         : u8,
  pub n_hits_cbe         : u8,
  pub n_hits_cor         : u8,
  pub tot_edep_umb       : f32,
  pub tot_edep_cbe       : f32,
  pub tot_edep_cor       : f32,
  pub paddles_set        : bool,
}

impl TofEventSummary {

  pub fn new() -> Self {
    Self {
      status             : EventStatus::Unknown,
      version            : ProtocolVersion::Unknown,
      n_hits_umb         : 0,
      n_hits_cbe         : 0,
      n_hits_cor         : 0,
      tot_edep_umb       : 0.0,
      tot_edep_cbe       : 0.0,
      tot_edep_cor       : 0.0,
      quality            : 0,
      trigger_sources    : 0,
      n_trigger_paddles  : 0,
      event_id           : 0,
      run_id             : 0,
      timestamp32        : 0,
      timestamp16        : 0,
      primary_beta       : 0, 
      primary_charge     : 0, 
      drs_dead_lost_hits : 0,
      dsi_j_mask         : 0,
      channel_mask       : Vec::<u16>::new(),
      mtb_link_mask      : 0,
      hits               : Vec::<TofHit>::new(),
      paddles_set        : false,
    }
  }
 
  #[cfg(feature="database")]
  pub fn set_paddles(&mut self, paddles : &HashMap<u8, Paddle>) {
    let mut nerror = 0u8;
    for h in &mut self.hits {
      match paddles.get(&h.paddle_id) {
        None => {
          error!("Got paddle id {} which is not in given map!", h.paddle_id);
          nerror += 1;
          continue;
        }
        Some(pdl) => {
          h.set_paddle(pdl);
        }
      }
    }
    if nerror == 0 {
      self.paddles_set = true;
    }
  }

  /// Get the pointcloud of this event, sorted by time
  /// 
  /// # Returns
  ///   (f32, f32, f32, f32, f32) : (x,y,z,t,edep)
  pub fn get_pointcloud(&self) -> Option<Vec<(f32,f32,f32,f32,f32)>> {
    let mut pc = Vec::<(f32,f32,f32,f32,f32)>::new();
    if !self.paddles_set {
      error!("Before getting the pointcloud, paddle information needs to be set for this event. Call TofEventSummary;:set_paddle");
      return None;
    }
    for h in &self.hits {
      let result = (h.x, h.y, h.z, h.get_t0(), h.get_edep());
      pc.push(result);
    }
    Some(pc)
  }

  /// Compare the MasterTriggerEvent::trigger_hits with 
  /// the actual hits to determine from which paddles
  /// we should have received HG hits (from waveforms)
  /// but we did not get them
  ///
  /// WARNING: The current implementation of this is 
  /// rather slow and not fit for production use
  /// FIXME - rewrite as a closure
  #[cfg(feature="database")]
  pub fn get_missing_paddles_hg(&self, pid_map :   &DsiJChPidMapping) -> Vec<u8> {
    let mut missing = Vec::<u8>::new();
    for th in self.get_trigger_hits() {
      let pid = pid_map.get(&th.0).unwrap().get(&th.1).unwrap().get(&th.2.0).unwrap().0;
      let mut found = false;
      for h in &self.hits {
        if h.paddle_id == pid {
          found = true;
          break
        }
      }
      if !found {
        missing.push(pid);
      }
    }
    missing
  }
  
  /// Get the triggered paddle ids
  ///
  /// Warning, this might be a bit slow
  #[cfg(feature="database")]
  pub fn get_triggered_paddles(&self, pid_map :   DsiJChPidMapping) -> Vec<u8> {
    let mut paddles = Vec::<u8>::new();
    for th in self.get_trigger_hits() {
      let pid = pid_map.get(&th.0).unwrap().get(&th.1).unwrap().get(&th.2.0).unwrap().0;
      paddles.push(pid);
    }
    paddles
  }

  /// Get the RB link IDs according to the mask
  pub fn get_rb_link_ids(&self) -> Vec<u8> {
    let mut links = Vec::<u8>::new();
    for k in 0..64 {
      if (self.mtb_link_mask >> k) as u64 & 0x1 == 1 {
        links.push(k as u8);
      }
    }
    links
  }

  /// Get the combination of triggered DSI/J/CH on 
  /// the MTB which formed the trigger. This does 
  /// not include further hits which fall into the 
  /// integration window. For those, se rb_link_mask
  ///
  /// The returned values follow the TOF convention
  /// to start with 1, so that we can use them to 
  /// look up LTB ids in the db.
  ///
  /// # Returns
  ///
  ///   Vec<(hit)> where hit is (DSI, J, CH) 
  pub fn get_trigger_hits(&self) -> Vec<(u8, u8, (u8, u8), LTBThreshold)> {
    let mut hits = Vec::<(u8,u8,(u8,u8),LTBThreshold)>::with_capacity(5); 
    let physical_channels = [(1u8,  2u8), (3u8,4u8), (5u8, 6u8), (7u8, 8u8),
                             (9u8, 10u8), (11u8,12u8), (13u8, 14u8), (15u8, 16u8)];
    //let n_masks_needed = self.dsi_j_mask.count_ones() / 2 + self.dsi_j_mask.count_ones() % 2;
    let n_masks_needed = self.dsi_j_mask.count_ones();
    if self.channel_mask.len() < n_masks_needed as usize {
      error!("We need {} hit masks, but only have {}! This is bad!", n_masks_needed, self.channel_mask.len());
      return hits;
    }
    let mut n_mask = 0;
    trace!("Expecting {} hit masks", n_masks_needed);
    trace!("ltb channels {:?}", self.dsi_j_mask);
    trace!("hit masks {:?}", self.channel_mask); 
    //println!("We see LTB Channels {:?} with Hit masks {:?} for {} masks requested by us!", self.dsi_j_mask, self.channel_mask, n_masks_needed);
    
    // one k here is for one ltb
    for k in 0..32 {
      if (self.dsi_j_mask >> k) as u32 & 0x1 == 1 {
        let mut dsi = 0u8;
        let mut j   = 0u8;
        if k < 5 {
          dsi = 1;
          j   = k as u8 + 1;
        } else if k < 10 {
          dsi = 2;
          j   = k as u8 - 5 + 1;
        } else if k < 15 {
          dsi = 3;
          j   = k as u8- 10 + 1;
        } else if k < 20 {
          dsi = 4;
          j   = k as u8- 15 + 1;
        } else if k < 25 {
          dsi = 5;
          j   = k as u8 - 20 + 1;
        } 
        //let dsi = (k as f32 / 4.0).floor() as u8 + 1;       
        //let j   = (k % 5) as u8 + 1;
        //println!("n_mask {n_mask}");
        let channels = self.channel_mask[n_mask]; 
        for (i,ch) in LTB_CHANNELS.iter().enumerate() {
          //let chn = *ch as u8 + 1;
          let ph_chn = physical_channels[i];
          //let chn = i as u8 + 1;
          //println!("i,ch {}, {}", i, ch);
          let thresh_bits = ((channels & ch) >> (i*2)) as u8;
          //println!("thresh_bits {}", thresh_bits);
          if thresh_bits > 0 { // hit over threshold
            hits.push((dsi, j, ph_chn, LTBThreshold::from(thresh_bits)));
          }
        }
        n_mask += 1;
      } // next ltb
    }
    hits
  }
  
  /// Get the trigger sources from trigger source byte
  pub fn get_trigger_sources(&self) -> Vec<TriggerType> {
    transcode_trigger_sources(self.trigger_sources)
  }
  
  pub fn get_timestamp48(&self) -> u64 {
    ((self.timestamp16 as u64) << 32) | self.timestamp32 as u64
  }
  
  /// Ttotal energy depostion in the TOF - Umbrella
  ///
  /// Utilizes Philip's formula based on 
  /// peak height
  pub fn get_edep_umbrella(&self) -> f32 {
    let mut tot_edep = 0.0f32;
    for h in &self.hits {
      if h.paddle_id < 61 || h.paddle_id > 108 {
        continue;
      }
      tot_edep += h.get_edep();
    }
    tot_edep
  }
  
  /// Ttotal energy depostion in the TOF - Umbrella
  ///
  /// Utilizes Philip's formula based on 
  /// peak height
  pub fn get_edep_cube(&self) -> f32 {
    let mut tot_edep = 0.0f32;
    for h in &self.hits {
      if h.paddle_id > 60 {
        continue;
      }
      tot_edep += h.get_edep();
    }
    tot_edep
  }
  
  /// Ttotal energy depostion in the Cortina
  ///
  /// Utilizes Philip's formula based on 
  /// peak height
  pub fn get_edep_cortina(&self) -> f32 {
    let mut tot_edep = 0.0f32;
    for h in &self.hits {
      if h.paddle_id < 109 {
        continue;
      }
      tot_edep += h.get_edep();
    }
    tot_edep
  }
  
  /// Ttotal energy depostion in the complete TOF
  ///
  /// Utilizes Philip's formula based on 
  /// peak height
  pub fn get_edep(&self) -> f32 {
    let mut tot_edep = 0.0f32;
    for h in &self.hits {
      tot_edep += h.get_edep();
    }
    tot_edep
  }

  //pub fn set_beta(&mut self, beta : f32) {
  //  // expecting beta in range of 0-1. If larger
  //  // than 1, we will save 1
  //  if beta > 1.0 {
  //    self.primary_beta = 1;
  //  }
  //  let pbeta = beta*(u16::MAX as f32).floor();
  //  // safe, bc of multiplication with u16::MAX
  //  self.primary_beta = pbeta as u16;
  //}

  //pub fn get_beta(&self) -> f32 {
  //  self.primary_beta as f32/u32::MAX as f32 
  //}
}

impl Packable for TofEventSummary {
  const PACKET_TYPE        : PacketType = PacketType::TofEventSummary;
}

impl Serialization for TofEventSummary {
  
  const HEAD               : u16   = 43690; //0xAAAA
  const TAIL               : u16   = 21845; //0x5555
  
  fn to_bytestream(&self) -> Vec<u8> {
    let mut stream = Vec::<u8>::new();
    stream.extend_from_slice(&Self::HEAD.to_le_bytes());
    let status_version = self.status.to_u8() | self.version.to_u8();
    stream.push(status_version);
    stream.extend_from_slice(&self.trigger_sources.to_le_bytes());
    stream.extend_from_slice(&self.n_trigger_paddles.to_le_bytes());
    stream.extend_from_slice(&self.event_id.to_le_bytes());
    // depending on the version, we send the fc event packet
    if self.version == ProtocolVersion::V1 {
      stream.extend_from_slice(&self.n_hits_umb  .to_le_bytes()); 
      stream.extend_from_slice(&self.n_hits_cbe  .to_le_bytes()); 
      stream.extend_from_slice(&self.n_hits_cor  .to_le_bytes()); 
      stream.extend_from_slice(&self.tot_edep_umb.to_le_bytes()); 
      stream.extend_from_slice(&self.tot_edep_cbe.to_le_bytes()); 
      stream.extend_from_slice(&self.tot_edep_cor.to_le_bytes()); 
    }
    stream.extend_from_slice(&self.quality.to_le_bytes());
    stream.extend_from_slice(&self.timestamp32.to_le_bytes());
    stream.extend_from_slice(&self.timestamp16.to_le_bytes());
    //stream.extend_from_slice(&self.primary_beta.to_le_bytes());
    stream.extend_from_slice(&self.run_id.to_le_bytes());
    stream.extend_from_slice(&self.drs_dead_lost_hits.to_le_bytes());
    //stream.extend_from_slice(&self.primary_charge.to_le_bytes());
    stream.extend_from_slice(&self.dsi_j_mask.to_le_bytes());
    let n_channel_masks = self.channel_mask.len();
    stream.push(n_channel_masks as u8);
    for k in 0..n_channel_masks {
      stream.extend_from_slice(&self.channel_mask[k].to_le_bytes());
    }
    stream.extend_from_slice(&self.mtb_link_mask.to_le_bytes());
    let nhits = self.hits.len() as u16;
    stream.extend_from_slice(&nhits.to_le_bytes());
    for k in 0..self.hits.len() {
      stream.extend_from_slice(&self.hits[k].to_bytestream());
    }
    stream.extend_from_slice(&Self::TAIL.to_le_bytes());
    stream
  }
  
  fn from_bytestream(stream    : &Vec<u8>, 
                     pos       : &mut usize) 
    -> Result<Self, SerializationError>{
    let mut summary           = Self::new();
    let head = parse_u16(stream, pos);
    if head != Self::HEAD {
      error!("Decoding of HEAD failed! Got {} instead!", head);
      return Err(SerializationError::HeadInvalid);
    }
    let status_version_u8     = parse_u8(stream, pos);
    let status                = EventStatus::from(status_version_u8 & 0x3f);
    let version               = ProtocolVersion::from(status_version_u8 & 0xc0); 
    summary.status            = status;
    summary.version           = version;
    summary.trigger_sources   = parse_u16(stream, pos);
    summary.n_trigger_paddles = parse_u8(stream, pos);
    summary.event_id          = parse_u32(stream, pos);
    if summary.version == ProtocolVersion::V1 {
      summary.n_hits_umb      = parse_u8(stream, pos); 
      summary.n_hits_cbe      = parse_u8(stream, pos); 
      summary.n_hits_cor      = parse_u8(stream, pos); 
      summary.tot_edep_umb    = parse_f32(stream, pos); 
      summary.tot_edep_cbe    = parse_f32(stream, pos); 
      summary.tot_edep_cor    = parse_f32(stream, pos); 
    }
    summary.quality            = parse_u8(stream, pos);
    summary.timestamp32        = parse_u32(stream, pos);
    summary.timestamp16        = parse_u16(stream, pos);
    summary.run_id             = parse_u16(stream, pos);
    summary.drs_dead_lost_hits = parse_u16(stream, pos);
    summary.dsi_j_mask         = parse_u32(stream, pos);
    let n_channel_masks        = parse_u8(stream, pos);
    for _ in 0..n_channel_masks {
      summary.channel_mask.push(parse_u16(stream, pos));
    }
    summary.mtb_link_mask     = parse_u64(stream, pos);
    let nhits                 = parse_u16(stream, pos);
    for _ in 0..nhits {
      summary.hits.push(TofHit::from_bytestream(stream, pos)?);
    }
    let tail = parse_u16(stream, pos);
    if tail != Self::TAIL {
      error!("Decoding of TAIL failed for version {}! Got {} instead!", version, tail);
      return Err(SerializationError::TailInvalid);
    }
    Ok(summary)
  }
}
    
impl Default for TofEventSummary {
  fn default() -> Self {
    Self::new()
  }
}

impl fmt::Display for TofEventSummary {
  fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
    let mut repr = format!("<TofEventSummary (version {})", self.version);
    repr += &(format!("\n  EventID          : {}", self.event_id));
    repr += &(format!("\n  RunID            : {}", self.run_id));
    repr += &(format!("\n  EventStatus      : {}", self.status));
    repr += &(format!("\n  TriggerSources   : {:?}", self.get_trigger_sources()));
    repr += &(format!("\n  NTrigPaddles     : {}", self.n_trigger_paddles));
    repr += &(format!("\n  DRS dead hits    : {}", self.drs_dead_lost_hits));
    repr += &(format!("\n  timestamp32      : {}", self.timestamp32)); 
    repr += &(format!("\n  timestamp16      : {}", self.timestamp16)); 
    repr += &(format!("\n   |-> timestamp48 : {}", self.get_timestamp48())); 
    //repr += &(format!("\n  PrimaryBeta      : {}", self.get_beta())); 
    //repr += &(format!("\n  PrimaryCharge    : {}", self.primary_charge));
    repr += &(format!("\n  ** ** TRIGGER HITS (DSI/J/CH) [{} LTBS] ** **", self.dsi_j_mask.count_ones()));
    for k in self.get_trigger_hits() {
      repr += &(format!("\n  => {}/{}/({},{}) ({}) ", k.0, k.1, k.2.0, k.2.1, k.3));
    }
    repr += "\n  ** ** MTB LINK IDs ** **";
    let mut mtblink_str = String::from("\n  => ");
    for k in self.get_rb_link_ids() {
      mtblink_str += &(format!("{} ", k))
    }
    repr += &mtblink_str;
    repr += &(format!("\n  == Trigger hits {}, expected RBEvents {}",
            self.get_trigger_hits().len(),
            self.get_rb_link_ids().len()));
    repr += &String::from("\n  ** ** ** HITS ** ** **");
    for h in &self.hits {
      repr += &(format!("\n  {}", h));
    }
    write!(f, "{}", repr)
  }
}

#[cfg(feature="random")]
impl FromRandom for TofEventSummary {

  fn from_random() -> Self {
    let mut summary           = Self::new();
    let mut rng               = rand::thread_rng();
    let status                = EventStatus::from_random();
    let version               = ProtocolVersion::from_random();
    if version == ProtocolVersion::V1 {
      summary.n_hits_umb        = rng.gen::<u8>();
      summary.n_hits_cbe        = rng.gen::<u8>();
      summary.n_hits_cor        = rng.gen::<u8>();
      summary.tot_edep_umb      = rng.gen::<f32>();
      summary.tot_edep_cbe      = rng.gen::<f32>();
      summary.tot_edep_cor      = rng.gen::<f32>();
      summary.quality           = rng.gen::<u8>();
    }
    summary.status             = status;
    summary.version            = version;
    // variable packet for the FC
    summary.trigger_sources    = rng.gen::<u16>();
    summary.n_trigger_paddles  = rng.gen::<u8>();
    summary.event_id           = rng.gen::<u32>();
    summary.timestamp32        = rng.gen::<u32>();
    summary.timestamp16        = rng.gen::<u16>();
    summary.drs_dead_lost_hits = rng.gen::<u16>();
    summary.dsi_j_mask         = rng.gen::<u32>();
    let n_channel_masks        = rng.gen::<u8>();
    for _ in 0..n_channel_masks {
      summary.channel_mask.push(rng.gen::<u16>());
    }
    summary.mtb_link_mask      = rng.gen::<u64>();
    let nhits                  = rng.gen::<u8>();
    for _ in 0..nhits {
      summary.hits.push(TofHit::from_random());
    }
    summary
  }
}

//
// TESTS
//
// ============================================

#[test]
fn packable_tofeventsummary() {
  for _ in 0..100 {
    let data = TofEventSummary::from_random();
    let test : TofEventSummary = data.pack().unpack().unwrap();
    assert_eq!(data, test);
  }
}  

#[test]
fn emit_tofeventsummary() {
  for _ in 0..100 {
    let data = TofEvent::from_random();
    let summary = data.get_summary();
    let test : TofEventSummary = summary.pack().unpack().unwrap();
    assert_eq!(summary, test);
  }
}

#[test]
#[cfg(feature = "random")]
fn tofevent_sizes_header() {
  for _ in 0..100 {
    let data = TofEvent::from_random();
    let mask = data.construct_sizes_header();
    let size = TofEvent::decode_size_header(&mask);
    assert_eq!(size.0, data.rb_events.len());
    //assert_eq!(size.1, data.missing_hits.len());
  }
}

#[test]
#[cfg(feature = "random")]
fn packable_tofevent() {
  for _ in 0..5 {
    let data = TofEvent::from_random();
    let test : TofEvent = data.pack().unpack().unwrap();
    assert_eq!(data.header, test.header);
    assert_eq!(data.compression_level, test.compression_level);
    assert_eq!(data.quality, test.quality);
    assert_eq!(data.mt_event, test.mt_event);
    assert_eq!(data.rb_events.len(), test.rb_events.len());
    //assert_eq!(data.missing_hits.len(), test.missing_hits.len());
    //assert_eq!(data.missing_hits, test.missing_hits);
    assert_eq!(data.rb_events, test.rb_events);
    //assert_eq!(data, test);
    //println!("{}", data);
  }
}

#[test]
#[cfg(feature = "random")]
fn serialize_tofeventheader() {
  let data = TofEventHeader::from_random();
  let test = TofEventHeader::from_bytestream(&data.to_bytestream(), &mut 0).unwrap();
  assert_eq!(data, test);
}

