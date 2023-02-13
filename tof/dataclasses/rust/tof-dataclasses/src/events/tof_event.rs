//! Event strucutures for data reconrded byi the tof
//!
//! Compressed format containing analysis results of 
//! the waveforms for individual paddles.
//! Each paddle has a "paddle packet"
//!
//!

use std::time::{SystemTime,
                Instant};

use crate::constants::EVENT_TIMEOUT;
//use crate::errors::SerializationError;
use crate::errors::EventError;

use crate::packets::paddle_packet::PaddlePacket;
use crate::serialization::search_for_u16;
use crate::errors::SerializationError;

use crate::events::MasterTriggerEvent;

#[cfg(feature="random")]
use rand::Rng;



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
  pub n_paddles_expected : u8,

  // for the event builder. 
  // if not using the master trigger,
  // we can look at the time the event has first
  // been seen and then it will be declared complete
  // after timeout microseconds
  // thus we are saving the time, this isntance has 
  // been created.
  //pub creation_time      : u128,
  pub creation_time      : Instant,

  pub valid              : bool,
}


impl TofEvent {
  
  pub const PacketSizeFixed    : usize = 24;
  pub const Version            : &'static str = "1.1";
  pub const Head               : u16  = 43690; //0xAAAA
  pub const Tail               : u16  = 21845; //0x5555
  

  pub fn new(event_id : u32,
             n_paddles_expected : u8) -> TofEvent {
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
    let evid = u32::from_le_bytes([bytestream[start_pos + 2],
                                   bytestream[start_pos + 3],
                                   bytestream[start_pos + 4],
                                   bytestream[start_pos + 5]]);
    Ok(evid)
  }

  pub fn from_bytestream(bytestream : &Vec<u8>, start_pos : usize)
     -> Result<TofEvent, SerializationError> {
    let mut event = TofEvent::new(9,0);
    let mut pos = start_pos;

    pos = search_for_u16(TofEvent::Head, &bytestream, pos)?;
   
    let mut raw_bytes_4  = [bytestream[pos + 1],
                            bytestream[pos + 0],
                            bytestream[pos + 3],
                            bytestream[pos + 2]];
    pos   += 4; 
    event.event_id = u32::from_be_bytes(raw_bytes_4); 
    raw_bytes_4  = [bytestream[pos ],
                    bytestream[pos + 1],
                    bytestream[pos + 2],
                    bytestream[pos + 3]];
    event.timestamp_32 = u32::from_le_bytes(raw_bytes_4);
    let raw_bytes_2 = [bytestream[pos],
                       bytestream[pos + 1]];
    event.timestamp_16 = u16::from_le_bytes(raw_bytes_2);
    pos += 2;
    event.n_paddles      = bytestream[pos];
    pos += 1; 
   
    for n in 0..event.n_paddles {
      match PaddlePacket::from_bytestream(&bytestream, pos) {
        Err(err) => {
          return Err(err);
        }
        Ok(pp)   => {
          event.paddle_packets.push(pp);
          pos += PaddlePacket::PACKETSIZE;
        }
      }
    }
    Ok(event) 
  }
  
  pub fn to_bytestream(&self) -> Vec<u8> {

    let mut bytestream = Vec::<u8>::with_capacity(TofEvent::PacketSizeFixed + (self.n_paddles as usize)*PaddlePacket::PACKETSIZE as usize);

    bytestream.extend_from_slice(&TofEvent::Head.to_le_bytes());
    let mut evid = self.event_id.to_be_bytes();

    evid  = [evid[1],
             evid[0],
             evid[3],
             evid[2]];
    bytestream.extend_from_slice(&evid);
    bytestream.extend_from_slice(&self.timestamp_32.to_le_bytes());
    bytestream.extend_from_slice(&self.timestamp_16.to_le_bytes());
    bytestream.push(self.n_paddles);
    for n in 0..self.paddle_packets.len() as usize {
      let pp = self.paddle_packets[n];
      bytestream.extend_from_slice(&pp.to_bytestream());

    }
    bytestream.extend_from_slice(&TofEvent::Tail        .to_le_bytes()); 
    bytestream
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
    return self.is_complete() || (self.has_timed_out() && use_timeout);
  }
}

impl Default for TofEvent {
  fn default() -> TofEvent {
    TofEvent::new(0,0)
  }
}

impl From<&MasterTriggerEvent> for TofEvent {
  fn from(mte : &MasterTriggerEvent) -> TofEvent {
    let mut te : TofEvent = Default::default();
    te.event_id     = mte.event_id;
    te.timestamp_32 = mte.timestamp;
    te.n_paddles    = mte.get_hit_paddles();
    te
  }
}



///
/// TESTS
///
/// ============================================

#[test]
fn serialize_deserialize_pp_roundabout() {
  let mut pp = PaddlePacket::from_random();
  // a fresh packet is always valid
  assert!(pp.valid);
  // FIXME - as an idea. If we use
  // 4 byte emoji data, we can easily
  // check if a bytestream is that what
  // we expect visually 
   
  
  //let mut bytestream = Vec<u8>::new();
  let mut bytestream = pp.to_bytestream();
  match PaddlePacket::from_bytestream(&bytestream, 0) {
    Err(err) => {
      error!("Got deserialization error! {:?}", err);
    },
    Ok(new_pp)   => {
      assert_eq!(new_pp, pp);
    }
  }
}

#[test]
fn serialize_deserialize_tofevent_roundabout() {
  let mut event = TofEvent::new(0,0);

  // let's add 10 random paddles
  for n in 0..10 {
    let pp = PaddlePacket::from_random();
    event.paddle_packets.push(pp);
  }
  assert!(event.valid);

  //let mut bytestream = Vec<u8>::new();
  let mut bytestream = event.to_bytestream();
  match TofEvent::from_bytestream(&bytestream, 0) {
    Err(err) => {
      error!("Got deserialization error! {:?}", err);
    },
    Ok(new_event)   => {
      assert_eq!(new_event, event);
    }
  }
}

