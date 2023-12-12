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


use crate::constants::EVENT_TIMEOUT;

// re-imports
use std::time::Instant;
use std::fmt;
pub use crate::monitoring::{RBMoniData,
                            TofCmpMoniData,
                            MtbMoniData};
use crate::serialization::{
    Serialization, 
    parse_u8,
    parse_u16,
    parse_u32
};

use std::error::Error;
use crate::errors::{
    SerializationError,
    PacketError
};
use crate::events::{RBEventHeader,
                    RBEvent,
                    MasterTriggerEvent,
                    TofEvent};
use crate::commands::{TofCommand,
                      RBCommand};
use crate::calibrations::RBCalibrations;

pub mod packet_type;
pub use packet_type::PacketType;

pub enum PacketQuality {
  Perfect,
  Good,
  NotSoGood,
  Bad,
  Rubbish, 
  UtterRubish
}

/// The most basic of all packets
///  
/// A type and a payload. This wraps
/// all other packets.
///
/// Format when in bytestream
/// HEAD : u16

/// PAYLOAD_SIZE : u32
/// PYALOAD : [u8;PAYLOAD_SIZE]
/// TAIL : u16
///
/// => Fixed size is 13
///
#[derive(Debug, Clone)]
pub struct TofPacket {
  pub packet_type        : PacketType,
  pub payload            : Vec<u8>,
  // FUTURE EXTENSION: Be able to send
  /// packets which contain multiple of the same packets
  pub is_multi_packet    : bool,
  // fields which won't get serialized
  /// mark a packet as not eligible to be written to disk
  pub no_write_to_disk   : bool,
  /// mark a packet as not eligible to be sent over network 
  /// FIXME - future extension
  pub no_send_over_nw    : bool,
  /// creation_time for the instance
  pub creation_time      : Instant,
  pub valid              : bool, // will be always valid, unless invalidated
}

impl fmt::Display for TofPacket {
  fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
    let p_len = self.payload.len();
    if p_len < 4 {
      write!(f, "<TofPacket: type {:?}, payload size {}>", self.packet_type, p_len)
    } else {
      write!(f, "<TofPacket: type {:?}, multi {}, payload [ {} {} {} {} .. {} {} {} {}] of size {} >",
             self.packet_type,
             self.is_multi_packet,
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
    (self.is_multi_packet == other.is_multi_packet)   &&
    (self.no_write_to_disk == other.no_write_to_disk) &&
    (self.no_send_over_nw == other.no_send_over_nw)   &&
    (self.valid == other.valid)
  }
}

impl TofPacket {

  pub const PRELUDE_SIZE : usize = 7; 
 
  pub fn new() -> Self {
    let creation_time = Instant::now();
    Self {
      packet_type      : PacketType::Unknown,
      payload          : Vec::<u8>::new(),
      is_multi_packet  : false,
      no_write_to_disk : false,
      no_send_over_nw  : false,
      creation_time    : creation_time,
      valid            : true,
    }
  }

  /// Unpack possible content
  pub fn unpack_rbevent(&self) -> Result<RBEvent, Box<dyn Error>> {
    if self.packet_type != PacketType::RBEvent {
      error!("We expeckt packet type {}, but got packet type {} instead!", PacketType::RBEvent, self.packet_type);
      return Err(Box::new(PacketError::WrongPacketType));
    }
    Ok(RBEvent::from_bytestream(&self.payload, &mut 0)?)
  }

  /// Event can time out after specified time
  pub fn has_timed_out(&self) -> bool {
    return self.age() > EVENT_TIMEOUT;
  }

  pub fn age(&self) -> u64 {
    self.creation_time.elapsed().as_secs()
  }
 
  pub fn get_n_packets(&self) -> u32 {
    if !self.is_multi_packet {
      return 1; 
    }
    todo!("Can not deal with multipackets right now!");
    #[allow(unreachable_code)] {
      return 1;
    }
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

impl From<&RBCommand> for TofPacket {
  fn from(cmd : &RBCommand) -> Self {
    let mut tp = Self::new();
    tp.packet_type = PacketType::RBCommand;
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


impl From<&RBMoniData> for TofPacket {
  fn from(moni : &RBMoniData) -> TofPacket {
    let mut tp = TofPacket::new();
    tp.packet_type = PacketType::RBMoni;
    tp.payload = moni.to_bytestream();
    tp
  }
}

impl From<&MtbMoniData> for TofPacket {
  fn from(moni : &MtbMoniData) -> TofPacket {
    let mut tp = TofPacket::new();
    tp.packet_type = PacketType::MonitorMtb;
    tp.payload = moni.to_bytestream();
    tp
  }
}

impl From<&TofCmpMoniData> for TofPacket {
  fn from(moni : &TofCmpMoniData) -> TofPacket {
    let mut tp = TofPacket::new();
    tp.packet_type = PacketType::MonitorTofCmp;
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

impl Serialization for TofPacket {
  const HEAD : u16 = 0xaaaa;
  const TAIL : u16 = 0x5555;
  const SIZE : usize = 0; // FIXME - size/prelude_size 

  fn from_bytestream(stream : &Vec<u8>, pos : &mut usize)
  -> Result<Self, SerializationError> {
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
    if self.is_multi_packet {
      todo!("Can not deal with multipackets right now!");
    }
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



#[test]
fn test_serialize_tofpacket() ->Result<(), SerializationError> {
  let mut pk     = TofPacket::new();
  pk.packet_type = PacketType::TofEvent;
  let mut pl     = Vec::<u8>::new();
  for n in 0..200000 {
    pl.push(n as u8);
  }
  pk.payload = pl;
  //pk.payload     = vec![1,2,3,4];
  let bs = pk.to_bytestream();
  //println!("{bs:?}");
  let mut pos = 0usize;
  let pk2 = TofPacket::from_bytestream(&bs, &mut pos)?;
  
  assert_eq!(pk, pk2);
  Ok(())
}
