//! Event strucutures for data reconrded byi the tof
//!
//! Compressed format containing analysis results of 
//! the waveforms for individual paddles.
//! Each paddle has a "paddle packet"
//!
//!

use std::time::Instant;
use std::fmt;

#[cfg(feature = "random")]
use crate::FromRandom;
#[cfg(feature = "random")]
use rand::Rng;

use crate::constants::EVENT_TIMEOUT;
use crate::serialization::{Serialization,
                           parse_u8,
                           parse_u16,
                           parse_u32,
                           search_for_u16};
use crate::errors::SerializationError;

use crate::events::{MasterTriggerEvent,
                    RBEvent,
                    RBMissingHit};

use crate::monitoring::RBMoniData;

// This looks like a TODO
#[derive(Debug, Copy, Clone, PartialEq)]
pub enum CompressionLevel {
  Unknown,
  None,
}

impl fmt::Display for CompressionLevel {
  fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
    let r = self.string_repr();
    write!(f, "<CompressionLevel: {}>", r)
  }
}

impl CompressionLevel {

  pub const UNKNOWN : u8 = 0;
  pub const NONE    : u8 = 10;

  pub fn to_u8(&self) -> u8 {
    let result : u8;
    match self {
      CompressionLevel::Unknown => {
        result = CompressionLevel::UNKNOWN;
      }
      CompressionLevel::None => {
        result = CompressionLevel::NONE;
      }
    }
    result
  }
  
  pub fn from_u8(code : &u8) -> Self {
    let mut result = CompressionLevel::Unknown;
    match *code {
      CompressionLevel::UNKNOWN => {
        result = CompressionLevel::Unknown;
      }
      CompressionLevel::NONE => {
        result = CompressionLevel::None;
      }
      _ => {
        error!("Unknown compression level {}!", code);
      }
    }
    result
  }

  /// String representation of the enum
  ///
  /// This is basically the enum type as 
  /// a string.
  pub fn string_repr(&self) -> String { 
    let repr : String;
    match self {
      CompressionLevel::Unknown => { 
        repr = String::from("Unknown");
      }
      CompressionLevel::None => {
        repr = String::from("None");
      }
    }
    repr
  }
}

#[derive(Debug, Copy, Clone, PartialEq)]
pub enum EventQuality {
  Unknown        =  0,
  Silver         = 10,
  Gold           = 20,
  Diamond        = 30,
  FourLeafClover = 40,
}

impl fmt::Display for EventQuality {
  fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
    let r = self.string_repr();
    write!(f, "<EventQuality: {}>", r)
  }
}

impl EventQuality {
  pub const UNKNOWN        : u8 =  0;
  pub const SILVER         : u8 = 10;
  pub const GOLD           : u8 = 20;
  pub const DIAMOND        : u8 = 30;
  pub const FOURLEAFCLOVER : u8 = 40;
  
  pub fn to_u8(&self) -> u8 {
    let result : u8;
    match self {
      EventQuality::Unknown => {
        result = EventQuality::UNKNOWN;
      }
      EventQuality::Silver => {
        result = EventQuality::SILVER;
      }
      EventQuality::Gold => {
        result = EventQuality::GOLD;
      }
      EventQuality::Diamond => {
        result = EventQuality::DIAMOND;
      }
      EventQuality::FourLeafClover => {
        result = EventQuality::FOURLEAFCLOVER;
      }
    }
    result
  }
  
  pub fn from_u8(code : &u8) -> Self {
    let mut result = EventQuality::Unknown;
    match *code {
      EventQuality::UNKNOWN => {
        result = EventQuality::Unknown;
      }
      EventQuality::SILVER => {
        result = EventQuality::Silver;
      }
      EventQuality::GOLD => {
        result = EventQuality::Gold;
      }
      EventQuality::DIAMOND => {
        result = EventQuality::Diamond;
      }
      EventQuality::FOURLEAFCLOVER => {
        result = EventQuality::FourLeafClover;
      }
      _ => {
        error!("Unknown event quality {}!", code);
      }
    }
    result
  }

  pub fn string_repr(&self) -> String { 
    let repr : String;
    match self {
      EventQuality::Unknown => { 
        repr = String::from("Unknown");
      }
      EventQuality::Silver => {
        repr = String::from("Silver");
      }
      EventQuality::Gold => {
        repr = String::from("Gold");
      }
      EventQuality::Diamond => {
        repr = String::from("Diamond");
      }
      EventQuality::FourLeafClover => {
        repr = String::from("FourLeafClover");
      }
    }
    repr
  }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TofEvent {

  pub compression_level : CompressionLevel,
  pub quality           : EventQuality,
  pub mt_event          : MasterTriggerEvent,
  pub rb_events         : Vec::<RBEvent>,
  pub missing_hits      : Vec::<RBMissingHit>, 
  pub rb_moni           : Vec::<RBMoniData>,
  
  // won't get serialized
  pub creation_time      : Instant,
}

impl fmt::Display for TofEvent {
  fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
    write!(f, "<TofEvent:
            \t event id  : {}
            \t quality   : {}
            \t n_boards  : {}
            \t miss_hits : {}
            \t n_moni    : {}>"
            ,self.mt_event.event_id,
            self.quality,
            self.rb_events.len(),
            self.missing_hits.len(),
            self.rb_moni.len())
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
      mt_event          : MasterTriggerEvent::new(0,0),
      rb_events         : Vec::<RBEvent>::new(),
      missing_hits      : Vec::<RBMissingHit>::new(), 
      rb_moni           : Vec::<RBMoniData>::new(),
      creation_time     : creation_time,
    }
  }
  
  /// Event can time out after specified time
  ///
  pub fn has_timed_out(&self) -> bool {
    return self.age() > EVENT_TIMEOUT;
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
     let moni_len     = self.rb_moni.len() as u32;
     let mut mask     = 0u32;
     mask = mask | rb_event_len;
     mask = mask | (miss_len << 8);
     mask = mask | (moni_len << 16);
     mask
  }

  pub fn decode_size_header(mask : &u32) 
    -> (usize, usize, usize) {
    let rb_event_len = (mask & 0xFF)        as usize;
    let miss_len     = ((mask & 0xFF00)     >> 8)  as usize;
    let moni_len     = ((mask & 0xFF0000)   >> 16) as usize;
    (rb_event_len, miss_len, moni_len)
  }
  
  pub fn get_combined_vector_sizes(&self) -> usize {
    self.rb_events.len() 
    + self.missing_hits.len() 
    + self.rb_moni.len()
  }
}

impl Serialization for TofEvent {
  
  const HEAD               : u16   = 43690; //0xAAAA
  const TAIL               : u16   = 21845; //0x5555

  // unify to_le_bytes and other in to_bytestream ? TODO
  fn to_bytestream(&self) -> Vec<u8> {
    let mut stream = Vec::<u8>::new();
    stream.extend_from_slice(&Self::HEAD.to_le_bytes());
    stream.extend_from_slice(&self.compression_level.to_u8().to_le_bytes());
    stream.extend_from_slice(&self.quality.to_u8().to_le_bytes());
    stream.extend_from_slice(&self.mt_event.to_bytestream());
    let sizes_header = self.construct_sizes_header();
    stream.extend_from_slice(&sizes_header.to_le_bytes());
    for k in 0..self.rb_events.len() {
      stream.extend_from_slice(&self.rb_events[k].to_bytestream());
    }
    for k in 0..self.missing_hits.len() {
      stream.extend_from_slice(&self.missing_hits[k].to_bytestream());
    }
    for k in 0..self.rb_moni.len() {
      stream.extend_from_slice(&self.rb_moni[k].to_bytestream());
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
    event.compression_level = CompressionLevel::from_u8(&parse_u8(stream, pos));
    event.quality           = EventQuality::from_u8(&parse_u8(stream, pos));
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
    for k in 0..v_sizes.2 {
      match RBMoniData::from_bytestream(stream, pos) {
        Err(err) => error!("Expected RBMoniPacket {} of {}, but got serialization error {}!", k,  v_sizes.2, err),
        Ok(moni) => {
          event.rb_moni.push(moni);
        }
      }
    }
    Ok(event)
  }
}

#[cfg(feature="random")]
impl FromRandom for TofEvent {

  fn from_random() -> Self {
    let mut event   = Self::new();
    event.mt_event  = MasterTriggerEvent::from_random();
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
    for _ in 0..n_boards {
      event.rb_moni.push(RBMoniData::from_random());
    }
    // for now, we do not randomize CompressionLevel and qualtiy
    //event.compression_level : CompressionLevel::,
    //event.quality           : EventQuality::Unknown,
    //  paddle_packets    : Vec::<PaddlePacket>::new(),
    event
  }
}

impl From<&MasterTriggerEvent> for TofEvent {
  fn from(mte : &MasterTriggerEvent) -> Self {
    let mut te : TofEvent = Default::default();
    te.mt_event = *mte;
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
    let mut event = Self::new();
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


//
// TESTS
//
// ============================================

#[cfg(all(test,feature = "random"))]
mod test_tofevents {
  use crate::serialization::Serialization;
  use crate::FromRandom;
  use crate::events::{TofEvent,
                      TofEventHeader};

  #[test]
  fn serialize_tofeventheader() {
    let data = TofEventHeader::from_random();
    let test = TofEventHeader::from_bytestream(&data.to_bytestream(), &mut 0).unwrap();
    assert_eq!(data, test);
  }

  #[test]
  fn mastertofevent_sizes_header() {
    let data = TofEvent::from_random();
    let mask = data.construct_sizes_header();
    let size = TofEvent::decode_size_header(&mask);
    assert_eq!(size.0, data.rb_events.len());
    assert_eq!(size.1, data.missing_hits.len());
    assert_eq!(size.2, data.rb_moni.len());
  }

  #[test]
  fn serialization_mastertofevent() {
    let data = TofEvent::from_random();
    let test = TofEvent::from_bytestream(&data.to_bytestream(), &mut 0).unwrap();
    assert_eq!(data, test);
    //println!("{}", data);
  }
}

