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
use crate::errors::EventError;

use crate::packets::paddle_packet::PaddlePacket;
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


#[derive(Debug, Copy, Clone, PartialEq)]
pub enum CompressionLevel {
  Unknown,
  None,
}

impl fmt::Display for CompressionLevel {
  fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
    let r = self.string_repr();
    write!(f, "<EventQuality: {}>", r)
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
pub struct MasterTofEvent {

  pub compression_level : CompressionLevel,
  pub quality           : EventQuality,
  pub mt_event          : MasterTriggerEvent,
  pub rb_events         : Vec::<RBEvent>,
  pub missing_hits      : Vec::<RBMissingHit>, 
  pub rb_moni           : Vec::<RBMoniData>,
  
  // won't get serialized
  pub creation_time      : Instant,
}

impl fmt::Display for MasterTofEvent {
  fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
    write!(f, "<MasterTofEvent:
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

impl Default for MasterTofEvent {

  fn default() -> Self {
    Self::new()
  }
}

impl MasterTofEvent {

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

impl Serialization for MasterTofEvent {
  
  const HEAD               : u16   = 43690; //0xAAAA
  const TAIL               : u16   = 21845; //0x5555

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
impl FromRandom for MasterTofEvent {

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


/// The main event structure
#[derive(Debug, Clone, PartialEq)]
pub struct TofEvent  {
  
  pub event_id     : u32,
  // the timestamp sahll be comging from the master trigger
  pub timestamp_32 : u32,
  pub timestamp_16 : u16,

  // this field can be debated
  // the reason we have it is 
  // that for de/serialization, 
  // we need to know the length 
  // of the expected bytestream.
  pub n_paddles    : u8, // we don't have more than 
                         // 256 paddles.
                         // HOWEVER!! For future gaps
                         // flights, we might...
                         // This will then overflow 
                         // and cause problems.
  
  // this is private, paddles can only 
  // be added
  pub paddle_packets : Vec::<PaddlePacket>,

  // fields which won't get 
  // serialized
  //

  /// Comes from the master trigger. Number of paddles which 
  /// are over the threshold
  pub n_paddles_expected : u8,

  // for the event builder. 
  // if not using the master trigger,
  // we can look at the time the event has first
  // been seen and then it will be declared complete
  // after timeout microseconds
  // thus we are saving the time, this isntance has 
  // been created.
  pub creation_time      : Instant,

  pub valid              : bool,
}

impl Serialization for TofEvent {
  const HEAD               : u16  = 43690; //0xAAAA
  const TAIL               : u16  = 21845; //0x5555
  
  fn from_bytestream(bytestream : &Vec<u8>, pos : &mut usize)
     -> Result<Self, SerializationError> {
    let mut event = Self::new(0,0);
    //let mut pos = start_pos;
    *pos = search_for_u16(Self::HEAD, &bytestream, *pos)?;
   
    let mut raw_bytes_4  = [bytestream[*pos ],
                            bytestream[*pos + 1],
                            bytestream[*pos + 2],
                            bytestream[*pos + 3]];
    *pos   += 4; 
    event.event_id = u32::from_le_bytes(raw_bytes_4); 
    raw_bytes_4  = [bytestream[*pos ],
                    bytestream[*pos + 1],
                    bytestream[*pos + 2],
                    bytestream[*pos + 3]];
    *pos   += 4; 
    event.timestamp_32 = u32::from_le_bytes(raw_bytes_4);
    //let raw_bytes_2 = [bytestream[*pos],
    //                   bytestream[*pos + 1]];
    //pos += 2;
    event.timestamp_16 = parse_u16(bytestream, pos);
    event.n_paddles    = parse_u8(bytestream, pos);
    //event.timestamp_16 = u16::from_le_bytes(raw_bytes_2);
    //event.n_paddles      = bytestream[pos];
    //pos += 1; 
   
    for _ in 0..event.n_paddles {
      match PaddlePacket::from_bytestream(&bytestream, pos) {
        Err(err) => {
          error!("Unable to decode PaddlePacket, {err}");
          return Err(err);
        }
        Ok(pp)   => {
          event.paddle_packets.push(pp);
          *pos += PaddlePacket::SIZE;
        }
      }
    }
    Ok(event) 
  }
  
  fn to_bytestream(&self) -> Vec<u8> {

    let mut bytestream = Vec::<u8>::with_capacity(Self::PACKETSIZEFIXED + (self.n_paddles as usize)*PaddlePacket::SIZE as usize);

    bytestream.extend_from_slice(&Self::HEAD.to_le_bytes());
    //let mut evid = self.event_id.to_be_bytes();

    //evid  = [evid[1],
    //         evid[0],
    //         evid[3],
    //         evid[2]];
    
    //bytestream.extend_from_slice(&evid);
    bytestream.extend_from_slice(&self.event_id.to_le_bytes());
    bytestream.extend_from_slice(&self.timestamp_32.to_le_bytes());
    bytestream.extend_from_slice(&self.timestamp_16.to_le_bytes());
    bytestream.push(self.n_paddles);
    for n in 0..self.paddle_packets.len() as usize {
      let pp = self.paddle_packets[n];
      bytestream.extend_from_slice(&pp.to_bytestream());

    }
    bytestream.extend_from_slice(&Self::TAIL        .to_le_bytes()); 
    bytestream
  }
}

impl TofEvent {
  
  pub const PACKETSIZEFIXED    : usize = 24;
  pub const VERSION            : &'static str = "1.1";
  //pub const HEAD               : u16  = 43690; //0xAAAA
  //pub const TAIL               : u16  = 21845; //0x5555
  
  pub fn new(event_id : u32,
             n_paddles_expected : u8) -> Self {
    //let creation_time  = SystemTime::now()
    //                     .duration_since(SystemTime::UNIX_EPOCH)
    //                     .unwrap().as_micros();
    let creation_time = Instant::now();

    TofEvent { 
      event_id       : event_id,
      timestamp_32   : 0,
      timestamp_16   : 0,
      n_paddles      : 0,  
      paddle_packets : Vec::<PaddlePacket>::with_capacity(20),

      n_paddles_expected : n_paddles_expected,

      // This is strictly for when working
      // with event timeouts
      creation_time  : creation_time,

      valid          : true,
    }
  }


  /// Decode only the event id. 
  ///
  /// The bytestream must be sane, cannot fail
  pub fn get_evid_from_bytestream(bytestream : &Vec<u8>, start_pos : usize) 
    -> Result<u32, SerializationError> {
    if bytestream.len() < 6 {
      // something is utterly broken
      return Err(SerializationError::StreamTooShort);
    }
    let evid = u32::from_le_bytes([bytestream[start_pos],
                                   bytestream[start_pos + 1],
                                   bytestream[start_pos + 2],
                                   bytestream[start_pos + 3]]);
    //let evid = u32::from_le_bytes([bytestream[start_pos + 2],
    //                               bytestream[start_pos + 3],
    //                               bytestream[start_pos + 4],
    //                               bytestream[start_pos + 5]]);
    Ok(evid)
  }


  /// Add a paddle packet 
  ///  
  /// This makes sure the internal counter for 
  /// paddles is also incremented.
  pub fn add_paddle(&mut self, paddle : PaddlePacket) -> Result<(), EventError> {
    if self.event_id != paddle.event_id {
      error!("Tried to add paddle for event {} to event{}", self.event_id, paddle.event_id);
      return Err(EventError::EventIdMismatch);
    }
    self.n_paddles += 1;
    self.paddle_packets.push(paddle);
    Ok(())
  }


  /// Check if a certain time span has passed since event's creation
  ///
  ///
  ///
  pub fn has_timed_out(&self) -> bool {
    return self.age() > EVENT_TIMEOUT;
  }

  pub fn age(&self) -> u64 {
    self.creation_time.elapsed().as_secs()
  }

  pub fn is_complete(&self) -> bool {
    self.n_paddles == self.n_paddles_expected
  }

  /// This means that all analysis is 
  /// done, and it is fully assembled
  ///
  /// Alternatively, the timeout has 
  /// been passed
  ///
  pub fn is_ready_to_send(&self, use_timeout : bool)
    -> bool {
    //if self.n_paddles > 0 {
    //  println!("ready? {} {} {} {}", self.event_id, self.n_paddles, self.n_paddles_expected, self.age());
    //}
    // doing it like this will ensure that the events are ordered.
    // Otherwise, complete events will bypass
    if use_timeout {
      return self.has_timed_out();
    }
    self.is_complete() 
  }
}

impl Default for TofEvent {
  fn default() -> Self {
    TofEvent::new(0,0)
  }
}

impl From<&MasterTriggerEvent> for TofEvent {
  fn from(mte : &MasterTriggerEvent) -> Self {
    let mut te : TofEvent = Default::default();
    te.event_id              = mte.event_id;
    te.timestamp_32          = mte.timestamp;
    te.n_paddles_expected    = mte.get_hit_paddles();
    te
  }
}

impl From<&MasterTriggerEvent> for MasterTofEvent {
  fn from(mte : &MasterTriggerEvent) -> Self {
    let mut te : MasterTofEvent = Default::default();
    te.mt_event = *mte;
    te
  }
}


//
// TESTS
//
// ============================================

#[cfg(test)]
mod test_tofevents {
  use crate::serialization::Serialization;
  use crate::FromRandom;
  use crate::events::MasterTofEvent;

  #[test]
  fn mastertofevent_sizes_header() {
    let data = MasterTofEvent::from_random();
    let mask = data.construct_sizes_header();
    let size = MasterTofEvent::decode_size_header(&mask);
    assert_eq!(size.0, data.rb_events.len());
    assert_eq!(size.1, data.missing_hits.len());
    assert_eq!(size.2, data.rb_moni.len());
  }

  #[test]
  fn serialization_mastertofevent() {
    let data = MasterTofEvent::from_random();
    let test = MasterTofEvent::from_bytestream(&data.to_bytestream(), &mut 0).unwrap();
    assert_eq!(data, test);
    //println!("{}", data);
  }
}

