//! Reporting catastrophic errors throughout the TOF system 
//! related to software crashes 
//!
// The following file is part of gaps-online-software and published 
// under the GPLv3 license

use crate::prelude::*;

/// To be sent right before a panic occurs
#[derive(Debug, Clone, PartialEq)]
#[cfg_attr(feature="pybindings", pyclass)]
pub struct PanicPacket {
  sender  : u8,
  message : String,
}

impl PanicPacket {

  pub fn new(sender : u8, message : String) -> Self {
    Self { 
      sender  : sender,
      message : message
    } 
  }

  /// Sent this over a newly created zmq socket.
  /// (If any data publisher thread is active, this 
  /// might itself cause a panic, so that this works
  /// all data publsher threads have to cease first 
  pub fn sent(&self) {
    let _bs = self.to_bytestream();
  }
}

impl Default for PanicPacket {

  fn default() -> Self {
    Self::new(0,String::from("Unknonwn panic!"))
  }
}

impl fmt::Display for PanicPacket {
  fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
    let mut repr = format!("<PanicPacket from {}\n  ", self.sender);
    repr += &self.message;
    repr += ">";
    write!(f, "{}", repr)
  }
}

impl TofPackable for PanicPacket {
  const TOF_PACKET_TYPE        : TofPacketType = TofPacketType::PanicPacket; 
}

impl Serialization for PanicPacket {
  const HEAD               : u16   = 43690; //0xAAAA
  const TAIL               : u16   = 21845; //0x5555
  
  fn to_bytestream(&self) -> Vec<u8> {
    let mut stream = Vec::<u8>::new();
    stream.extend_from_slice(&Self::HEAD.to_le_bytes());
    stream.push(self.sender);
    let mut payload = CRFrame::string_to_bytes(self.message.clone());
    let pl_len  = payload.len() as u16;
    stream.extend(&pl_len.to_le_bytes()); 
    stream.append(&mut payload);
    stream.extend_from_slice(&Self::TAIL.to_le_bytes());
    stream
  }
  
  fn from_bytestream(stream    : &Vec<u8>, 
                     pos       : &mut usize) 
    -> Result<Self, SerializationError>{
    
        
    let head      = parse_u16(stream, pos);
    if head != Self::HEAD {
      error!("Decoding of HEAD failed! Got {} instead!", head);
      return Err(SerializationError::HeadInvalid);
    }
    let sender    = parse_u8(stream, pos);
    let msg       = parse_string(stream, pos);
    let tail      = parse_u16(stream, pos);
    if tail != Self::HEAD {
      error!("Decoding of HEAD failed! Got {} instead!", head);
      return Err(SerializationError::HeadInvalid);
    }
    let panic = Self::new(sender, msg);
    Ok(panic)
  }
}
