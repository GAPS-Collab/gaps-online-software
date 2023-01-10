///! Packets are a way to send data over the network.
///
///  Data gets serialized to a bytestream and then 
///  header and tail bytes are added to the front and
///  end of the stream.
///
///  A Packet has the following layout
///  HEAD    : u16 = 0xAAAA
///  TYPE    : u8  = PacketType
///  SIZE    : u64
///  PAYLOAD : [u8;6-SIZE]
///  TAIL    : u16 = 0x5555 
///
///  The total packet size is thus 13 + SIZE


pub mod paddle_packet;
pub mod generic_packet;
pub mod data_packet;

pub use crate::packets::generic_packet::GenericPacket;
pub use crate::packets::data_packet::{DataPacket,
                                      CommandPacket};

use crate::serialization::{Serialization};
use crate::errors::SerializationError;

//use nom::IResult;
//use nom::{error::ErrorKind, Err};
//use nom::number::complete::*;
//use nom::bytes::complete::{tag, take, take_until};

#[derive(Debug)]
//#[repr(u8)]
pub enum PacketType {
  Unknown   , 
  Command   ,
  RBEvent   ,
  Monitor   ,
  HeartBeat ,
}

impl PacketType {
  pub fn as_u8(packet_type : &PacketType)   -> u8 {
    match packet_type {
      PacketType::Unknown   => 0,
      PacketType::Command   => 10,
      PacketType::RBEvent   => 20,
      PacketType::Monitor   => 30,
      PacketType::HeartBeat => 40,
    }

  }

  pub fn from_u8(value : u8) -> Option<PacketType> {
    match value {
      0   => Some(PacketType::Unknown),  
      10  => Some(PacketType::Command), 
      20  => Some(PacketType::RBEvent), 
      30  => Some(PacketType::Monitor), 
      40  => Some(PacketType::HeartBeat), 
      _   => None,
    }
  }
}

///! The most basic of all packets
///  
///  A type and a payload. This wraps
///  all other packets.

pub struct TofPacket {
  pub packet_type : PacketType,
  pub payload     : Vec<u8>
}

//fn nom_deserialize_tp(

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
    let payload_len = self.payload.len() as u64;
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

//impl Default for TofPacket {
//}

impl Serialization for TofPacket {
  fn from_bytestream(stream : &Vec<u8>, start_pos : usize)
  -> Result<TofPacket, SerializationError> {
    let mut two_bytes : [u8;2];
    two_bytes = [stream[start_pos],
                 stream[start_pos+1]];
        
    let mut pos = start_pos + 2;
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
    let eight_bytes = [stream[pos],
                       stream[pos+1],
                       stream[pos+2],
                       stream[pos+3],
                       stream[pos+4],
                       stream[pos+5],
                       stream[pos+6],
                       stream[pos+7]];
    let payload_size = u64::from_le_bytes(eight_bytes);
    pos += 8;
    two_bytes = [stream[pos], stream[pos + payload_size as usize]];
    if TofPacket::TAIL != u16::from_le_bytes(two_bytes) {
      warn!("Packet does not end with TAIL signature");
      return Err(SerializationError::TailInvalid {});
    }
    let mut payload = Vec::<u8>::with_capacity(payload_size as usize - 13);
    payload.extend_from_slice(&stream[pos..=pos+payload_size as usize]);
    Ok(TofPacket {
      packet_type,
      payload
    })
  }
}


