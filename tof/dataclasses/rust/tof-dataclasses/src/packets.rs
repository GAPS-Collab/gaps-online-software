//! Packets are a way to send data over the network.
//!
//! Data gets serialized to a bytestream and then 
//! header and tail bytes are added to the front and
//! end of the stream.
//!
//! A TofPacket has the following layout
//! HEAD    : u16 = 0xAAAA
//! TYPE    : u8  = PacketType
//! SIZE    : u32
//! PAYLOAD : [u8;6-SIZE]
//! TAIL    : u16 = 0x5555 
//!
//! The total packet size is thus 13 + SIZE

pub mod packet_type;
pub use packet_type::PacketType;

use std::time::Instant;
use std::fmt;
// re-exports
pub use crate::monitoring::{
    RBMoniData,
    PBMoniData,
    LTBMoniData,
    PAMoniData,
    MtbMoniData,
    CPUMoniData
};

use crate::serialization::{
    Serialization, 
    Packable,
    parse_u8,
    parse_u16,
    parse_u32
};


//use std::error::Error;
use crate::errors::{
    SerializationError,
    //PacketError
};

use crate::events::{
    RBEventHeader,
    RBEvent,
    MasterTriggerEvent,
    TofEvent,
    RBWaveform,
    TofEventSummary,
};

use crate::commands::{
    TofCommand,
};

use crate::calibrations::RBCalibrations;

#[cfg(feature = "random")]
use crate::FromRandom;
#[cfg(feature = "random")]
use rand::Rng;

/// The most basic of all packets
///  
/// A type and a payload. This wraps
/// all other packets.
///
/// Format when in bytestream
/// HEAD : u16

/// PAYLOAD_SIZE : u32
/// PAYLOAD      : \[u8;PAYLOAD_SIZE\]
/// TAIL         : u16
///
/// => Fixed size is 13
///
#[derive(Debug, Clone)]
pub struct TofPacket {
  pub packet_type        : PacketType,
  pub payload            : Vec<u8>,
  // fields which won't get serialized
  /// mark a packet as not eligible to be written to disk
  pub no_write_to_disk   : bool,
  /// mark a packet as not eligible to be sent over network 
  /// FIXME - future extension
  pub no_send_over_nw    : bool,
  /// creation_time for the instance
  pub creation_time    : Instant,
  pub valid            : bool, // will be always valid, unless invalidated
}

impl fmt::Display for TofPacket {
  fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
    let p_len = self.payload.len();
    if p_len < 4 {
      write!(f, "<TofPacket: type {:?}, payload size {}>", self.packet_type, p_len)
    } else {
      write!(f, "<TofPacket: type {:?}, payload [ {} {} {} {} .. {} {} {} {}] of size {} >",
             self.packet_type,
             self.payload[0], self.payload[1], self.payload[2], self.payload[3],
             self.payload[p_len-4], self.payload[p_len-3], self.payload[p_len - 2], self.payload[p_len-1], p_len ) 
    }
  }
}

impl Default for TofPacket {
  fn default() -> Self {
    Self::new()
  }
}

/// Implement because TofPacket saves the creation time, 
/// which never will be the same for 2 different instances
impl PartialEq for TofPacket {
  fn eq(&self, other: &Self) -> bool {
    (self.packet_type == other.packet_type)           &&
    (self.payload == other.payload)                   &&
    (self.no_write_to_disk == other.no_write_to_disk) &&
    (self.no_send_over_nw == other.no_send_over_nw)   &&
    (self.valid == other.valid)
  }
}

impl TofPacket {

  pub fn new() -> Self {
    let creation_time = Instant::now();
    Self {
      packet_type      : PacketType::Unknown,
      payload          : Vec::<u8>::new(),
      no_write_to_disk : false,
      no_send_over_nw  : false,
      creation_time    : creation_time,
      valid            : true,
    }
  }

  /// Generate a bytestream of self for ZMQ, prefixed with 
  /// BRCT so all RBs will see it
  pub fn zmq_payload_brdcast(&self) -> Vec<u8> {
    let mut payload     = String::from("BRCT").into_bytes(); 
    let mut stream  = self.to_bytestream();
    payload.append(&mut stream);
    payload
  }
  
  /// Generate a bytestream of self for ZMQ, prefixed with 
  /// RBX, to address only a certain board
  pub fn zmq_payload_rb(&self, rb_id : u8) -> Vec<u8> {
    let mut payload     = format!("RB{:02}", rb_id).into_bytes(); 
    let mut stream  = self.to_bytestream();
    payload.append(&mut stream);
    payload
  }

  /// Unpack the TofPacket and return its content
  pub fn unpack<T>(&self) -> Result<T, SerializationError>
    where T: Packable + Serialization {
    if T::PACKET_TYPE != self.packet_type {
      error!("This bytestream is not for a {} packet!", self.packet_type);
      return Err(SerializationError::IncorrectPacketType);
    }
    let unpacked : T = T::from_bytestream(&self.payload, &mut 0)?;
    Ok(unpacked)
  }
  
  pub fn age(&self) -> u64 {
    self.creation_time.elapsed().as_secs()
  }
}


#[cfg(feature="random")]
impl FromRandom for TofPacket {

  fn from_random() -> Self {
    // FIXME - this should be an actual, realistic
    // distribution
    let choices = [
      PacketType::TofEvent,
      PacketType::TofEvent,
      PacketType::TofEvent,
      PacketType::TofEvent,
      PacketType::TofEvent,
      PacketType::TofEvent,
      PacketType::TofEvent,
      PacketType::RBWaveform,
      PacketType::RBWaveform,
      PacketType::TofEventSummary,
      PacketType::TofEventSummary,
      PacketType::TofEventSummary,
      PacketType::TofEventSummary,
      PacketType::TofEventSummary,
      PacketType::TofEventSummary,
      PacketType::MasterTrigger,
      PacketType::MasterTrigger,
      PacketType::MasterTrigger,
      PacketType::RBMoniData,
      PacketType::PBMoniData,
      PacketType::LTBMoniData,
      PacketType::PAMoniData,
      PacketType::CPUMoniData,
      PacketType::MonitorMtb,
    ];
    let mut rng  = rand::thread_rng();
    let idx = rng.gen_range(0..choices.len());
    let packet_type = choices[idx];
    match packet_type {
      PacketType::TofEvent => {
        let te = TofEvent::from_random();
        return te.pack()
      }
      PacketType::TofEventSummary => {
        let te = TofEventSummary::from_random();
        return te.pack()
      }
      PacketType::RBWaveform => {
        let te = RBWaveform::from_random();
        return te.pack()
      }
      PacketType::MasterTrigger => {
        let te = MasterTriggerEvent::from_random();
        return te.pack()
      }
      PacketType::RBMoniData => {
        let te = RBMoniData::from_random();
        return te.pack()
      }
      PacketType::PAMoniData => {
        let te = PAMoniData::from_random();
        return te.pack()
      }
      PacketType::LTBMoniData => {
        let te = LTBMoniData::from_random();
        return te.pack()
      }
      PacketType::PBMoniData => {
        let te = PBMoniData::from_random();
        return te.pack()
      }
      PacketType::CPUMoniData => {
        let te = CPUMoniData::from_random();
        return te.pack()
      }
      PacketType::MonitorMtb  => {
        let te = MtbMoniData::from_random();
        return te.pack()
      }
      _ => {
        let te = TofEvent::from_random();
        return te.pack()
      }
    }
  }
}


/// FIXME - all these can go away now, because we have the
/// Packable trait! Amazing!
impl From<&RBWaveform> for TofPacket {
  fn from(rbwave : &RBWaveform) -> Self {
    let mut tp     = Self::new();
    tp.packet_type = PacketType::RBWaveform;
    tp.payload     = rbwave.to_bytestream();
    tp
  }
}

impl From<&TofEventSummary> for TofPacket {
  fn from(tsum : &TofEventSummary) -> Self {
    let mut tp     = Self::new();
    tp.packet_type = PacketType::TofEventSummary;
    tp.payload     = tsum.to_bytestream();
    tp
  }
}

impl From<&TofEvent> for TofPacket {
  fn from(event : &TofEvent) -> Self {
    let mut tp = Self::new();
    tp.packet_type = PacketType::TofEvent;
    tp.payload = event.to_bytestream();
    tp
  }
}

impl From<&mut TofEvent> for TofPacket {
  fn from(event : &mut TofEvent) -> Self {
    let mut tp     = Self::new();
    tp.packet_type = PacketType::TofEvent;
    tp.payload     = event.to_bytestream();
    tp
  }
}

impl From<&TofCommand> for TofPacket {
  fn from(cmd : &TofCommand) -> Self {
    let mut tp = Self::new();
    tp.packet_type = PacketType::TofCommand;
    tp.payload = cmd.to_bytestream();
    tp
  }
}


impl From<&RBCalibrations> for TofPacket {
  fn from(calib : &RBCalibrations) -> Self {
    let mut tp = Self::new();
    tp.packet_type = PacketType::RBCalibration;
    tp.payload = calib.to_bytestream();
    tp
  }
}

impl From<&CPUMoniData> for TofPacket {
  fn from(moni : &CPUMoniData) -> Self {
    let mut tp = Self::new();
    tp.packet_type = PacketType::CPUMoniData;
    tp.payload     = moni.to_bytestream();
    tp
  }
}

impl From<&mut RBCalibrations> for TofPacket {
  fn from(calib : &mut RBCalibrations) -> Self {
    let mut tp = Self::new();
    tp.packet_type = PacketType::RBCalibration;
    tp.payload = calib.to_bytestream();
    tp
  }
}

impl From<&RBEvent> for TofPacket {
  fn from(event : &RBEvent) -> Self {
    let mut tp = Self::new();
    tp.packet_type = PacketType::RBEvent;
    tp.payload = event.to_bytestream();
    tp
  }
}

impl From<&MasterTriggerEvent> for TofPacket {
  fn from(mt : &MasterTriggerEvent) -> TofPacket {
    let mut tp     = TofPacket::new();
    tp.packet_type = PacketType::MasterTrigger;
    tp.payload     = mt.to_bytestream();
    tp
  }
}


//impl From<&RBMoniData> for TofPacket {
//  fn from(moni : &RBMoniData) -> Self {
//    let mut tp     = Self::new();
//    tp.packet_type = PacketType::RBMoniData;
//    tp.payload     = moni.to_bytestream();
//    tp
//  }
//}
//
//impl From<&PBMoniData> for TofPacket {
//  fn from(moni : &PBMoniData) -> Self {
//    let mut tp     = Self::new();
//    tp.packet_type = PacketType::PBMoniData;
//    tp.payload     = moni.to_bytestream();
//    tp
//  }
//}
//impl From<&LTBMoniData> for TofPacket {
//  fn from(moni : &LTBMoniData) -> Self {
//    let mut tp     = Self::new();
//    tp.packet_type = PacketType::LTBMoniData;
//    tp.payload     = moni.to_bytestream();
//    tp
//  }
//}
//
//impl From<&PAMoniData> for TofPacket {
//  fn from(moni : &PAMoniData) -> Self {
//    let mut tp     = Self::new();
//    tp.packet_type = PacketType::PAMoniData;
//    tp.payload     = moni.to_bytestream();
//    tp
//  }
//}

impl From<&MtbMoniData> for TofPacket {
  fn from(moni : &MtbMoniData) -> TofPacket {
    let mut tp = TofPacket::new();
    tp.packet_type = PacketType::MonitorMtb;
    tp.payload = moni.to_bytestream();
    tp
  }
}

impl From<&RBEventHeader> for TofPacket {
  fn from(ev_header : &RBEventHeader) -> TofPacket {
    let mut tp     = TofPacket::new();
    tp.packet_type = PacketType::RBEventHeader;
    tp.payload     = ev_header.to_bytestream();
    tp
  }
}

// I would LOOVE to implement the Packable trait here and have 
// a matroshka doll for TofPackets. I just don't know why that 
// would be useful. It might be leading to a new approach 
// for multipackets

impl Serialization for TofPacket {
  const HEAD : u16 = 0xaaaa;
  const TAIL : u16 = 0x5555;
  const SIZE : usize = 0; // FIXME - size/prelude_size 

  fn from_bytestream(stream : &Vec<u8>, pos : &mut usize)
  -> Result<Self, SerializationError> {
    if stream.len() < 2 {
      return Err(SerializationError::HeadInvalid {});
    }
    let head = parse_u16(stream, pos);
    if Self::HEAD != head {
      error!("Packet does not start with HEAD signature");
      return Err(SerializationError::HeadInvalid {});
    }
    let packet_type : PacketType;
    let packet_type_enc = parse_u8(stream, pos);
    match PacketType::try_from(packet_type_enc) {
      Ok(pt) => packet_type = pt,
      Err(_) => {
        error!("Can not decode packet with packet type {}", packet_type_enc);
        return Err(SerializationError::UnknownPayload);}
    }
    let payload_size = parse_u32(stream, pos);
    *pos += payload_size as usize; 
    let tail = parse_u16(stream, pos);
    if Self::TAIL != tail {
      error!("Packet does not end with TAIL signature");
      return Err(SerializationError::TailInvalid {});
    }
    *pos -= 2; // for tail parsing
    *pos -= payload_size as usize;

    let mut tp = TofPacket::new();
    tp.packet_type = packet_type;
    tp.payload.extend_from_slice(&stream[*pos..*pos+payload_size as usize]);
    Ok(tp) 
  }
  
  fn to_bytestream(&self) 
    -> Vec<u8> {
    let mut bytestream = Vec::<u8>::with_capacity(6 + self.payload.len());
    bytestream.extend_from_slice(&TofPacket::HEAD.to_le_bytes());
    let p_type = self.packet_type as u8;
    bytestream.push(p_type);
    // payload size of 32 bit accomodates up to 4 GB packet
    // a 16 bit size would only hold 65k, which might be not
    // good enough if we sent multiple events in a batch in 
    // the same TofPacket (in case we do that)
    let payload_len = self.payload.len() as u32;
    //let foo = &payload_len.to_le_bytes();
    //debug!("TofPacket binary payload: {foo:?}");
    bytestream.extend_from_slice(&payload_len.to_le_bytes());
    bytestream.extend_from_slice(self.payload.as_slice());
    bytestream.extend_from_slice(&TofPacket::TAIL.to_le_bytes());
    bytestream
  }
}


