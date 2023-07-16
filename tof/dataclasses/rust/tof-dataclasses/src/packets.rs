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


pub mod paddle_packet;

// re-imports
pub use paddle_packet::PaddlePacket;

use std::fmt;
pub use crate::monitoring::{RBMoniData,
                            TofCmpMoniData,
                            MtbMoniData};
use crate::serialization::{Serialization, 
                           parse_u8,
                           parse_u32};
use crate::errors::SerializationError;
use crate::events::{RBEventPayload,
                    RBEventHeader,
                    MasterTriggerEvent};

pub enum PacketQuality {
  Perfect,
  Good,
  NotSoGood,
  Bad,
  Rubbish, 
  UtterRubish
}


pub const PACKET_TYPE_UNKNOWN        : u8 =  0;
pub const PACKET_TYPE_COMMAND        : u8 = 10;
pub const PACKET_TYPE_RBEVENT        : u8 = 20;
pub const PACKET_TYPE_TOFEVENT       : u8 = 21;
// not specific enough, deprecated. Use the packet
// types for monitor packets below
pub const PACKET_TYPE_MONITOR        : u8 = 30;
pub const PACKET_TYPE_HEARTBEAT      : u8 = 40;
pub const PACKET_TYPE_SCALAR         : u8 = 50;
pub const PACKET_TYPE_MT             : u8 = 60;
pub const PACKET_TYPE_RBHEADER       : u8 = 70;
// monitoring packets
pub const PACKET_TYPE_TOFCMP_MONI    : u8 = 80;
pub const PACKET_TYPE_MTB_MONI       : u8 = 90;
pub const PACKET_TYPE_RB_MONI        : u8 = 100;
pub const PACKET_TYPE_RBEVENTPAYLOAD : u8 = 110;

//// Each packet is send from somewhere.
////
//// Encode the sender in the packet, since
//// the streaming might be asynchronous.
////
//// Have a specific sender id per RB as 
//// well as one for the TofComputer.
////#[derive(Debug, Copy, Clone, PartialEq)]
////pub enum SenderId {
////  RB1,
////  RB2,
////  RB3,
////  RB4,
////  RB5,
////  RB6,
////  RB7,
////  RB8,
////  RB9,
////  RB10,
////  RB11,
////  RB12,
////  RB13,
////  RB14,
////  RB15,
////  RB1

/// Types of serializable data structures used
/// throughout the tof system
#[derive(Debug, PartialEq, Clone)]
//#[repr(u8)]
pub enum PacketType {
  Unknown       , 
  Command       ,
  RBEvent       ,
  TofEvent      ,
  Monitor       ,
  MasterTrigger , 
  HeartBeat     ,
  Scalar        ,
  RBHeader      ,
  RBEventPayload,
  MonitorRb     ,
  MonitorTofCmp ,
  MonitorMtb    ,
}

impl PacketType {
  pub fn as_u8(packet_type : &PacketType)   -> u8 {
    match packet_type {
      PacketType::Unknown        => PACKET_TYPE_UNKNOWN,
      PacketType::Command        => PACKET_TYPE_COMMAND,
      PacketType::RBEvent        => PACKET_TYPE_RBEVENT,
      PacketType::RBEventPayload => PACKET_TYPE_RBEVENTPAYLOAD, 
      PacketType::RBHeader       => PACKET_TYPE_RBHEADER,
      PacketType::TofEvent       => PACKET_TYPE_TOFEVENT,
      PacketType::Monitor        => PACKET_TYPE_MONITOR,
      PacketType::HeartBeat      => PACKET_TYPE_HEARTBEAT,
      PacketType::MasterTrigger  => PACKET_TYPE_MT,
      PacketType::Scalar         => PACKET_TYPE_SCALAR,
      PacketType::MonitorRb      => PACKET_TYPE_RB_MONI,
      PacketType::MonitorTofCmp => PACKET_TYPE_TOFCMP_MONI,
      PacketType::MonitorMtb     => PACKET_TYPE_MTB_MONI
    }
  }

  pub fn from_u8(value : u8) -> Option<PacketType> {
    match value {
      PACKET_TYPE_UNKNOWN        => Some(PacketType::Unknown),  
      PACKET_TYPE_COMMAND        => Some(PacketType::Command), 
      PACKET_TYPE_RBEVENT        => Some(PacketType::RBEvent), 
      PACKET_TYPE_TOFEVENT       => Some(PacketType::TofEvent),
      PACKET_TYPE_MONITOR        => Some(PacketType::Monitor), 
      PACKET_TYPE_HEARTBEAT      => Some(PacketType::HeartBeat),
      PACKET_TYPE_MT             => Some(PacketType::MasterTrigger),
      PACKET_TYPE_SCALAR         => Some(PacketType::Scalar),
      PACKET_TYPE_RBHEADER       => Some(PacketType::RBHeader),
      PACKET_TYPE_RBEVENTPAYLOAD => Some(PacketType::RBEventPayload),
      PACKET_TYPE_RB_MONI        => Some(PacketType::MonitorRb),
      PACKET_TYPE_MTB_MONI       => Some(PacketType::MonitorMtb),
      PACKET_TYPE_TOFCMP_MONI    => Some(PacketType::MonitorTofCmp),
      _   => None,
    }
  }
}

impl fmt::Display for PacketType {
  fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
    
    let repr : String;
    match self {
      PacketType::Unknown        => { repr = String::from("UNKNOWN")     },
      PacketType::Command        => { repr = String::from("COMMAND")     },
      PacketType::RBEvent        => { repr = String::from("RBEVENT")     },
      PacketType::RBEventPayload => { repr = String::from("RBEVENTPAYLOAD") },
      PacketType::TofEvent       => { repr = String::from("TOFEVENT")    },
      PacketType::Monitor        => { repr = String::from("MONITOR")     },
      PacketType::HeartBeat      => { repr = String::from("HEARTBEAT")   },
      PacketType::MasterTrigger  => { repr = String::from("MT")          },
      PacketType::Scalar         => { repr = String::from("SCALAR")      },
      PacketType::RBHeader       => { repr = String::from("RBHEADER")    },
      PacketType::MonitorRb      => { repr = String::from("RB_MONI")     },
      PacketType::MonitorTofCmp  => { repr = String::from("TOFCMP_MONI") },
      PacketType::MonitorMtb     => { repr = String::from("MTB_MONI")    },
    }
    write!(f, "<PacketType {}>", repr)
  }
}

/// The most basic of all packets
///  
/// A type and a payload. This wraps
/// all other packets.
///
/// Format when in bytestream
/// HEAD : u16
/// TYPE : u8
/// PAYLOAD_SIZE : u32
/// PYALOAD : [u8;PAYLOAD_SIZE]
/// TAIL : u16
///
/// => Fixed size is 13
///
#[derive(Debug, PartialEq, Clone)]
pub struct TofPacket {
  pub packet_type     : PacketType,
  pub payload         : Vec<u8>,
  // FUTURE EXTENSION: Be able to send
  // packets which contain multiple of the same packets
  pub is_multi_packet : bool,
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
  fn default() -> TofPacket {
    TofPacket {
      packet_type     : PacketType::Unknown,
      payload         : Vec::<u8>::new(),
      is_multi_packet : false
    }
  }
}

impl TofPacket {

  pub const PRELUDE_SIZE : usize = 7; 

  pub fn new() -> Self {
    Self {
      packet_type     : PacketType::Unknown,
      payload         : Vec::<u8>::new(),
      is_multi_packet : false
    }
  }
 
  pub fn get_n_packets(&self) -> u32 {
    if !self.is_multi_packet {
      return 1; 
    }
    todo!("Can not deal with multipackets right now!");
    return 1;
  }

  pub fn to_bytestream(&self) 
    -> Vec<u8> {
    if self.is_multi_packet {
      todo!("Can not deal with multipackets right now!");
    }
    let mut bytestream = Vec::<u8>::with_capacity(6 + self.payload.len());
    bytestream.extend_from_slice(&TofPacket::HEAD.to_le_bytes());
    let p_type = PacketType::as_u8(&self.packet_type);
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

  //impl from_bytes(stream : &[u8], start_pos : usize) {
  //  -> Result<TofPacket, SerializationError> {
  //    let (input, status) = le_u16(input)?;


  //  }

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
    tp.packet_type = PacketType::MonitorRb;
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

impl From<&RBEventPayload> for TofPacket {
  fn from(ev_payload : &RBEventPayload) -> TofPacket {
    let mut tp = TofPacket::new();
    tp.packet_type = PacketType::RBEventPayload;
    tp.payload = ev_payload.payload.clone();
    tp
  }
}

impl From<&RBEventHeader> for TofPacket {
  fn from(ev_header : &RBEventHeader) -> TofPacket {
    let mut tp     = TofPacket::new();
    tp.packet_type = PacketType::RBHeader;
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
    let mut two_bytes : [u8;2];
    two_bytes = [stream[*pos],
                 stream[*pos+1]];
        
    *pos += 2;
    if Self::HEAD != u16::from_le_bytes(two_bytes) {
      warn!("Packet does not start with HEAD signature");
      return Err(SerializationError::HeadInvalid {});
    }
    let packet_type_enc = stream[*pos];
    let packet_type : PacketType;
    *pos += 1;
    match PacketType::from_u8(packet_type_enc) {
      Some(pt) => packet_type = pt,
      None => {return Err(SerializationError::UnknownPayload);}
    }
    let payload_size = parse_u32(stream, pos);
    two_bytes = [stream[*pos + payload_size as usize], stream[*pos + 1 + payload_size as usize]];
    if Self::TAIL != u16::from_le_bytes(two_bytes) {
      warn!("Packet does not end with TAIL signature");
      return Err(SerializationError::TailInvalid {});
    }
    let mut payload = Vec::<u8>::with_capacity(payload_size as usize);
    payload.extend_from_slice(&stream[*pos..*pos+payload_size as usize]);
    //println!("PAYLOAD: {payload:?}");
    //trace!("TofPacket with Payload {payload:?}"
    let mut tp = TofPacket::new();
    tp.packet_type = packet_type;
    tp.payload = payload;
    Ok(tp) 
  }
}

#[test]
fn test_serialize_tofpacket() ->Result<(), SerializationError> {
  let mut pk     = TofPacket::new();
  pk.packet_type = PacketType::Command;
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
