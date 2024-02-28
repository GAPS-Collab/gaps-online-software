//! Event strucutures for data reconrded byi the tof
//!
//! Compressed format containing analysis results of 
//! the waveforms for individual paddles.
//! Each paddle has a "paddle packet"
//!
//!

use std::time::Instant;
use std::fmt;

cfg_if::cfg_if! {
  if #[cfg(feature = "random")]  {
    use crate::FromRandom;
    use rand::Rng;
  }
}

use crate::serialization::{Serialization,
                           parse_u8,
                           parse_u16,
                           parse_u32,
                           search_for_u16};
use crate::errors::SerializationError;

use crate::events::{
    MasterTriggerEvent,
    RBEvent,
    TofHit,
    RBWaveform,
    RBMissingHit
};

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
  pub missing_hits      : Vec::<RBMissingHit>, 
  
  // won't get serialized
  pub creation_time     : Instant,
  pub valid             : bool, 
}

impl fmt::Display for TofEvent {
  fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
    write!(f, 
"<TofEvent:
     quality        :  {}
     {} 
     {}
     n RBEvents      : {}
     n RBMissingHit  : {} >"
            ,self.quality,
            self.header,
            self.mt_event,
            self.rb_events.len(),
            self.missing_hits.len())
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
      mt_event          : MasterTriggerEvent::new(0,0),
      rb_events         : Vec::<RBEvent>::new(),
      missing_hits      : Vec::<RBMissingHit>::new(), 
      creation_time     : creation_time,
      valid             : true,
    }
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

  // Do we have all readoutboards which we are expecting 
  // in this event?
  //
  // This is useful from the perspective of the event builder.
  // Do we still have to wait for more rbs?
  // There might still be channels missing, but this is 
  // nothing the event builder can deal with right now.
  pub fn is_complete(&self) -> bool {
    self.mt_event.get_n_rbs_expected() as usize == self.rb_events.len()
  }

  /// Encode the sizes of the vectors holding the 
  /// into an u32
  ///
  /// We have one byte (256) max length per vector.
  pub fn construct_sizes_header(&self) -> u32 {
     let rb_event_len = self.rb_events.len() as u32;
     let miss_len     = self.missing_hits.len() as u32;
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
    + self.missing_hits.len() 
  }

  pub fn get_rbwaveforms(&self) -> Vec<RBWaveform> {
    let mut wf = Vec::<RBWaveform>::new();
    for ev in &self.rb_events {
      wf.extend_from_slice(&ev.get_rbwaveforms());
    }
    wf
  }

  pub fn get_summary(&self) -> TofEventSummary {
    let mut summary = TofEventSummary::new();
    //summary.status            = self.header.status;
    //summary.quality           = self.header.quality;
    //summary.trigger_setting   = self.;
    summary.n_trigger_paddles = self.mt_event.n_paddles;
    summary.event_id          = self.header.event_id;
    summary.timestamp32       = self.header.timestamp_32;
    summary.timestamp16       = self.header.timestamp_16;
    summary.primary_beta      = self.header.primary_beta; 
    summary.primary_charge    = self.header.primary_charge; 
    summary.hits              = Vec::<TofHit>::new();
    for ev in &self.rb_events {
      for hit in &ev.hits {
        summary.hits.push(hit.clone());
      }
    }
    summary
  }

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
    for k in 0..self.missing_hits.len() {
      stream.extend_from_slice(&self.missing_hits[k].to_bytestream());
    }
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
    for k in 0..v_sizes.1 {
      match RBMissingHit::from_bytestream(stream, pos) {
        Err(err) => error!("Expected RBMissingHit {} of {}, but got serialization error {}!", k,  v_sizes.1, err),
        Ok(miss) => {
          event.missing_hits.push(miss);
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
    let n_boards    = rng.gen::<u8>() as usize;
    //let n_paddles   = rng.gen::<u8>() as usize;
    let n_missing   = rng.gen::<u8>() as usize;
    for _ in 0..n_boards {
      event.rb_events.push(RBEvent::from_random());
    }
    for _ in 0..n_missing {
      event.missing_hits.push(RBMissingHit::from_random());
    }
    // for now, we do not randomize CompressionLevel and qualtiy
    //event.compression_level : CompressionLevel::,
    //event.quality           : EventQuality::Unknown,
    event
  }
}

impl From<&MasterTriggerEvent> for TofEvent {
  fn from(mte : &MasterTriggerEvent) -> Self {
    let mut te : TofEvent = Default::default();
    te.mt_event = *mte;
    te.header.event_id = te.mt_event.event_id;
    te
  }
}

/// The main event structure
#[derive(Debug, Clone, PartialEq)]
pub struct TofEventHeader  {

  pub run_id       : u32,
  pub event_id     : u32,
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

  pub nhit_outer_tof       : u8,  
  // no need to save this, can be 
  // rereated from paddle_info.size() - nhit_outer_tof
  pub nhit_inner_tof       : u8, 

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
    event.nhit_outer_tof      = parse_u8(stream, pos);
    event.nhit_inner_tof      = parse_u8(stream, pos);
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
    bytestream.extend_from_slice(&self.nhit_outer_tof            .to_le_bytes());
    bytestream.extend_from_slice(&self.nhit_inner_tof            .to_le_bytes());
    bytestream.extend_from_slice(&self.trigger_info              .to_le_bytes());
    bytestream.extend_from_slice(&self.ctr_etx                   .to_le_bytes());
    bytestream.extend_from_slice(&self.n_paddles                 .to_le_bytes());
    bytestream.extend_from_slice(&Self::TAIL        .to_le_bytes()); 
    bytestream
  }
}

impl TofEventHeader {
  
  pub const VERSION            : &'static str = "1.1";
  
  pub fn new() -> Self {
    Self {
      run_id               : 0,
      event_id             : 0,
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
      nhit_outer_tof       : 0,  
      nhit_inner_tof       : 0, 
      trigger_info         : 0,
      ctr_etx              : 0,
      n_paddles            : 0  
    }
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
    te.n_paddles             = mte.get_hit_paddles();
    te
  }
}

impl fmt::Display for TofEventHeader {
  fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
  let mut repr = String::from("<TofEventHeader"); 
  repr += &("\n  Run   ID          : ".to_owned() + &self.run_id              .to_string());
  repr += &("\n  Event ID          : ".to_owned() + &self.event_id            .to_string());
  repr += &("\n  Timestamp 32      : ".to_owned() + &self.timestamp_32        .to_string());
  repr += &("\n  Timestamp 16      : ".to_owned() + &self.timestamp_16        .to_string());
  repr += &("\n  Prim. Beta        : ".to_owned() + &self.primary_beta        .to_string());
  repr += &("\n  Prim. Beta Unc    : ".to_owned() + &self.primary_beta_unc    .to_string());
  repr += &("\n  Prim. Charge      : ".to_owned() + &self.primary_charge      .to_string());
  repr += &("\n  Prim. Charge unc  : ".to_owned() + &self.primary_charge_unc  .to_string());
  repr += &("\n  Prim. Outer Tof X : ".to_owned() + &self.primary_outer_tof_x .to_string());
  repr += &("\n  Prim. Outer Tof Y : ".to_owned() + &self.primary_outer_tof_y .to_string());
  repr += &("\n  Prim. Outer Tof Z : ".to_owned() + &self.primary_outer_tof_z .to_string());
  repr += &("\n  Prim. Inner Tof X : ".to_owned() + &self.primary_inner_tof_x .to_string());
  repr += &("\n  Prim. Inner Tof Y : ".to_owned() + &self.primary_inner_tof_y .to_string());
  repr += &("\n  Prim. Inner Tof Z : ".to_owned() + &self.primary_inner_tof_z .to_string());
  repr += &("\n  NHit  Outer Tof   : ".to_owned() + &self.nhit_outer_tof      .to_string());
  repr += &("\n  NHit  Inner Tof   : ".to_owned() + &self.nhit_inner_tof      .to_string());
  repr += &("\n  TriggerInfo       : ".to_owned() + &self.trigger_info        .to_string());
  repr += &("\n  Ctr ETX           : ".to_owned() + &self.ctr_etx             .to_string());
  repr += &("\n  NPaddles          : ".to_owned() + &self.n_paddles           .to_string());
  repr += ">";
  write!(f,"{}", repr)
  //run_id       : u32,
  //event_id     : u32,
  //timestamp_32 : u32,
  //timestamp_16 : u16, // -> 14 byres
  //primary_beta        : u16, 
  //primary_beta_unc    : u16, 
  //primary_charge      : u16, 
  //primary_charge_unc  : u16, 
  //primary_outer_tof_x : u16, 
  //primary_outer_tof_y : u16, 
  //primary_outer_tof_z : u16, 
  //primary_inner_tof_x : u16, 
  //primary_inner_tof_y : u16, 
  //primary_inner_tof_z : u16, //-> 20bytes primary 
  //nhit_outer_tof       : u8,  
  //nhit_inner_tof       : u8, 
  //trigger_info         : u8,
  //ctr_etx              : u8,
  //n_paddles           : u8, // we don't have more than 
  }
}

#[cfg(feature="random")]
impl FromRandom for TofEventHeader {

  fn from_random() -> Self {
    let mut rng     = rand::thread_rng();
    Self { 
      run_id               : rng.gen::<u32>(),
      event_id             : rng.gen::<u32>(),
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
      nhit_outer_tof       : rng.gen::<u8>(),  
      nhit_inner_tof       : rng.gen::<u8>(), 
      trigger_info         : rng.gen::<u8>(),
      ctr_etx              : rng.gen::<u8>(),
      n_paddles            : rng.gen::<u8>()  
    }
  }
}

/// Smaller packet for in-flight telemetry stream
#[derive(Debug, Clone, PartialEq)]
pub struct TofEventSummary {
  pub status            : u8,
  pub quality           : u8,
  pub trigger_setting   : u8,
  /// the number of triggered paddles coming
  /// from the MTB directly. This might NOT be
  /// the same as the number of hits!
  pub n_trigger_paddles : u8,
  pub event_id          : u32,
  pub timestamp32       : u32,
  pub timestamp16       : u16,
  /// reconstructed primary beta
  pub primary_beta      : u16, 
  /// reconstructed primary charge
  pub primary_charge    : u16, 
  pub hits : Vec<TofHit>,
}

impl TofEventSummary {

  pub fn new() -> Self {
    Self {
      status            : 0,
      quality           : 0,
      trigger_setting   : 0,
      n_trigger_paddles : 0,
      event_id          : 0,
      timestamp32       : 0,
      timestamp16       : 0,
      primary_beta      : 0, 
      primary_charge    : 0, 
      hits              : Vec::<TofHit>::new(),
    }
  }
  
  pub fn get_timestamp48(&self) -> u64 {
    ((self.timestamp16 as u64) << 32) | self.timestamp32 as u64
  }

  pub fn set_beta(&mut self, beta : f32) {
    // expecting beta in range of 0-1. If larger
    // than 1, we will save 1
    if beta > 1.0 {
      self.primary_beta = 1;
    }
    let pbeta = beta*(u16::MAX as f32).floor();
    // safe, bc of multiplication with u16::MAX
    self.primary_beta = pbeta as u16;
  }

  pub fn get_beta(&self) -> f32 {
    self.primary_beta as f32/u32::MAX as f32 
  }
}

impl Serialization for TofEventSummary {
  
  const HEAD               : u16   = 43690; //0xAAAA
  const TAIL               : u16   = 21845; //0x5555
  
  fn to_bytestream(&self) -> Vec<u8> {
    let mut stream = Vec::<u8>::new();
    stream.extend_from_slice(&Self::HEAD.to_le_bytes());
    stream.extend_from_slice(&self.status.to_le_bytes());
    stream.extend_from_slice(&self.quality.to_le_bytes());
    stream.extend_from_slice(&self.trigger_setting.to_le_bytes());
    stream.extend_from_slice(&self.n_trigger_paddles.to_le_bytes());
    stream.extend_from_slice(&self.event_id.to_le_bytes());
    stream.extend_from_slice(&self.timestamp32.to_le_bytes());
    stream.extend_from_slice(&self.timestamp16.to_le_bytes());
    stream.extend_from_slice(&self.primary_beta.to_le_bytes());
    stream.extend_from_slice(&self.primary_charge.to_le_bytes());
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
    summary.status            = parse_u8(stream, pos);
    summary.quality           = parse_u8(stream, pos);
    summary.trigger_setting   = parse_u8(stream, pos);
    summary.n_trigger_paddles = parse_u8(stream, pos);
    summary.event_id          = parse_u32(stream, pos);
    summary.timestamp32       = parse_u32(stream, pos);
    summary.timestamp16       = parse_u16(stream, pos);
    summary.primary_beta      = parse_u16(stream, pos); 
    summary.primary_charge    = parse_u16(stream, pos); 
    let nhits                 = parse_u16(stream, pos);
    for _ in 0..nhits {
      summary.hits.push(TofHit::from_bytestream(stream, pos)?);
    }
    let tail = parse_u16(stream, pos);
    if tail != Self::TAIL {
      error!("Decoding of TAIL failed! Got {} instead!", tail);
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
    let mut repr = String::from("<TofEventSummary");
    repr += &(format!("\n  Status           : {}", self.status));
    repr += &(format!("\n  TriggerSetting   : {}", self.trigger_setting));
    repr += &(format!("\n  NTrigPaddles     : {}", self.n_trigger_paddles));
    repr += &(format!("\n  EventID          : {}", self.event_id));
    repr += &(format!("\n  timestamp32      : {}", self.timestamp32)); 
    repr += &(format!("\n  timestamp16      : {}", self.timestamp16)); 
    repr += &(format!("\n   |-> timestamp48 : {}", self.get_timestamp48())); 
    repr += &(format!("\n  PrimaryBeta      : {}", self.get_beta())); 
    repr += &(format!("\n  PrimaryCharge    : {}", self.primary_charge));
    repr += &String::from("********* HITS *********");
    for h in &self.hits {
      repr += &(format!("\n  {}", h));
    }
    write!(f, "{}", repr)
  }
}

#[cfg(feature="random")]
impl FromRandom for TofEventSummary {

  fn from_random() -> Self {
    let mut summary = Self::new();
    let mut rng     = rand::thread_rng();

    summary.status            = rng.gen::<u8>();
    summary.quality           = rng.gen::<u8>();
    summary.trigger_setting   = rng.gen::<u8>();
    summary.n_trigger_paddles = rng.gen::<u8>();
    summary.event_id          = rng.gen::<u32>();
    summary.timestamp32       = rng.gen::<u32>();
    summary.timestamp16       = rng.gen::<u16>();
    summary.primary_beta      = rng.gen::<u16>(); 
    summary.primary_charge    = rng.gen::<u16>(); 
    let nhits                 = rng.gen::<u8>();
    for _ in 0..nhits {
      summary.hits.push(TofHit::from_random());
    }
    //hits : Vec<TofHit>,
    summary
  }
}

//
// TESTS
//
// ============================================

#[cfg(all(test,feature = "random"))]
mod test_tofevents {
  use crate::serialization::Serialization;
  use crate::FromRandom;
  use crate::events::{TofEvent,
                      TofEventHeader,
                      TofEventSummary};

  #[test]
  fn serialize_tofeventheader() {
    let data = TofEventHeader::from_random();
    let test = TofEventHeader::from_bytestream(&data.to_bytestream(), &mut 0).unwrap();
    assert_eq!(data, test);
  }

  #[test]
  fn tofevent_sizes_header() {
    for n in 0..100 {
      let data = TofEvent::from_random();
      let mask = data.construct_sizes_header();
      let size = TofEvent::decode_size_header(&mask);
      assert_eq!(size.0, data.rb_events.len());
      assert_eq!(size.1, data.missing_hits.len());
    }
  }

  #[test]
  fn serialization_tofevent() {
    for _ in 0..5 {
      let data = TofEvent::from_random();
      let test = TofEvent::from_bytestream(&data.to_bytestream(), &mut 0).unwrap();
      assert_eq!(data.header, test.header);
      assert_eq!(data.compression_level, test.compression_level);
      assert_eq!(data.quality, test.quality);
      assert_eq!(data.mt_event, test.mt_event);
      assert_eq!(data.rb_events.len(), test.rb_events.len());
      assert_eq!(data.missing_hits.len(), test.missing_hits.len());
      assert_eq!(data.missing_hits, test.missing_hits);
      assert_eq!(data.rb_events, test.rb_events);
      //assert_eq!(data, test);
      //println!("{}", data);
    }
  }
  
  #[test]
  fn serialization_tofeventsummary() {
    for _ in 0..100 {
      let data = TofEventSummary::from_random();
      let test = TofEventSummary::from_bytestream(&data.to_bytestream(), &mut 0).unwrap();
      assert_eq!(data, test);
    }
  }
}

