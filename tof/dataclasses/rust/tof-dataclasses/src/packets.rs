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
pub mod generic_packet;
pub mod data_packet;
//pub mod command_packet;


use std::fmt;
pub use crate::packets::generic_packet::GenericPacket;
pub use crate::packets::data_packet::DataPacket;
pub use crate::monitoring::RBMoniData;
//pub use crate::packets::command_packet::CommandPacket;
use crate::serialization::{Serialization};
use crate::errors::SerializationError;
use crate::events::RBEventPayload;
//use nom::IResult;
//use nom::{error::ErrorKind, Err};
//use nom::number::complete::*;
//use nom::bytes::complete::{tag, take, take_until};

pub enum PacketQuality {
  Perfect,
  Good,
  NotSoGood,
  Bad,
  Rubbish, 
  UtterRubish
}


pub const PACKET_TYPE_UNKNOWN   : u8 =  0;
pub const PACKET_TYPE_COMMAND   : u8 = 10;
pub const PACKET_TYPE_RBEVENT   : u8 = 20;
pub const PACKET_TYPE_TOFEVENT  : u8 = 21;
pub const PACKET_TYPE_MONITOR   : u8 = 30;
pub const PACKET_TYPE_HEARTBEAT : u8 = 40;
pub const PACKET_TYPE_SCALAR    : u8 = 50;
pub const PACKET_TYPE_MT        : u8 = 60;

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
}

impl PacketType {
  pub fn as_u8(packet_type : &PacketType)   -> u8 {
    match packet_type {
      PacketType::Unknown       => PACKET_TYPE_UNKNOWN,
      PacketType::Command       => PACKET_TYPE_COMMAND,
      PacketType::RBEvent       => PACKET_TYPE_RBEVENT,
      PacketType::TofEvent      => PACKET_TYPE_TOFEVENT,
      PacketType::Monitor       => PACKET_TYPE_MONITOR,
      PacketType::HeartBeat     => PACKET_TYPE_HEARTBEAT,
      PacketType::MasterTrigger => PACKET_TYPE_MT,
      PacketType::Scalar        => PACKET_TYPE_SCALAR
    }

  }

  pub fn from_u8(value : u8) -> Option<PacketType> {
    match value {
      PACKET_TYPE_UNKNOWN   => Some(PacketType::Unknown),  
      PACKET_TYPE_COMMAND   => Some(PacketType::Command), 
      PACKET_TYPE_RBEVENT   => Some(PacketType::RBEvent), 
      PACKET_TYPE_TOFEVENT  => Some(PacketType::TofEvent),
      PACKET_TYPE_MONITOR   => Some(PacketType::Monitor), 
      PACKET_TYPE_HEARTBEAT => Some(PacketType::HeartBeat),
      PACKET_TYPE_MT        => Some(PacketType::MasterTrigger),
      PACKET_TYPE_SCALAR    => Some(PacketType::Scalar),
      _   => None,
    }
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
/// PAYLOAD_SIZE : u64
/// PYALOAD : [u8;PAYLOAD_SIZE]
/// TAIL : u16
///
/// => Fixed size is 13
///
#[derive(Debug, PartialEq, Clone)]
pub struct TofPacket {
  pub packet_type : PacketType,
  pub payload     : Vec<u8>
}


impl fmt::Display for TofPacket {
  fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
    let p_len = self.payload.len();
    if p_len < 4 {
      write!(f, "<TofPacket: type {:?}, payload size {}>", self.packet_type, p_len)
    } else {
      write!(f, "<TofPacket: type {:?} payload [ {} {} {} {} .. {} {} {} {}] of size {} >",
             self.packet_type, self.payload[0], self.payload[1], self.payload[2], self.payload[3],
             self.payload[p_len-4], self.payload[p_len-3], self.payload[p_len - 2], self.payload[p_len-1], p_len ) 
    }
  }
}

impl Default for TofPacket {
  fn default() -> TofPacket {
    TofPacket {
      packet_type : PacketType::Unknown,
      payload     : Vec::<u8>::new(),
    }
  }
}

impl TofPacket {

  const HEAD : u16 = 0xaaaa;
  const TAIL : u16 = 0x5555;

  pub fn new() -> TofPacket {
    TofPacket {
      packet_type : PacketType::Unknown,
      payload     : Vec::<u8>::new()
    }
  }
  
  pub fn to_bytestream(&self) 
    -> Vec<u8> {
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

impl From<&RBMoniData> for TofPacket {
  fn from(moni : &RBMoniData) -> TofPacket {
    let mut tp = TofPacket::new();
    tp.packet_type = PacketType::Monitor;
    tp.payload = moni.to_bytestream();
    tp
  }
}

impl From<&RBEventPayload> for TofPacket {
  fn from(ev_payload : &RBEventPayload) -> TofPacket {
    let mut tp = TofPacket::new();
    tp.packet_type = PacketType::RBEvent;
    tp.payload = ev_payload.payload.clone();
    tp
  }
}


impl Serialization for TofPacket {
  fn from_bytestream(stream : &Vec<u8>, start_pos : usize)
  -> Result<TofPacket, SerializationError> {
    let mut pos = start_pos;
    let mut two_bytes : [u8;2];
    two_bytes = [stream[start_pos],
                 stream[start_pos+1]];
        
    pos += 2;
    if TofPacket::HEAD != u16::from_le_bytes(two_bytes) {
      warn!("Packet does not start with HEAD signature");
      return Err(SerializationError::HeadInvalid {});
    }
    let packet_type_enc = stream[pos];
    let packet_type : PacketType;
    pos += 1;
    match PacketType::from_u8(packet_type_enc) {
      Some(pt) => packet_type = pt,
      None => {return Err(SerializationError::UnknownPayload);}
    }
    //let eight_bytes = [stream[pos],
    //                   stream[pos+1],
    //                   stream[pos+2],
    //                   stream[pos+3],
    //                   stream[pos+4],
    //                   stream[pos+5],
    //                   stream[pos+6],
    //                   stream[pos+7]];
    let four_bytes = [stream[pos],
                      stream[pos + 1],
                      stream[pos + 2],
                      stream[pos + 3]];

    //println!("{eight_bytes:?}");
    //let payload_size = u64::from_le_bytes(eight_bytes);
    let payload_size = u32::from_le_bytes(four_bytes);
    //println!("{payload_size}");
    pos += 4;
    //println!("{pos}");
    two_bytes = [stream[pos + payload_size as usize], stream[pos + 1 + payload_size as usize]];
    if TofPacket::TAIL != u16::from_le_bytes(two_bytes) {
      warn!("Packet does not end with TAIL signature");
      return Err(SerializationError::TailInvalid {});
    }
    let mut payload = Vec::<u8>::with_capacity(payload_size as usize);
    payload.extend_from_slice(&stream[pos..pos+payload_size as usize]);
    //println!("PAYLOAD: {payload:?}");
    //trace!("TofPacket with Payload {payload:?}"
    Ok(TofPacket {
      packet_type,
      payload
    })
  }
}

#[test]
fn test_tof_packet_serialize_roundabout() ->Result<(), SerializationError> {
  let mut pk     = TofPacket::new();
  pk.packet_type = PacketType::Command;
  let mut pl     = Vec::<u8>::new();
  for n in 0..200000 {
    pl.push(n as u8);
  }
  pk.payload = pl;
  //pk.payload     = vec![1,2,3,4];
  let bs = pk.to_bytestream();
  println!("{bs:?}");
  let pk2 = TofPacket::from_bytestream(&bs, 0)?;
  
  assert_eq!(pk, pk2);
  Ok(())
}
