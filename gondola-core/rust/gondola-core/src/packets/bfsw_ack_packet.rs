// This file is part of gaps-online-software and published 
// under the GPLv3 license

use crate::prelude::*;

/// The acknowledgement packet used within the 
/// bfsw code
#[cfg_attr(feature="pybindings", pyclass)]
pub struct AckBfsw {
  pub header    : TelemetryPacketHeader,
  pub ack_type  : u8,
  pub ret_code1 : u8,
  pub ret_code2 : u8,
  pub body      : Vec<u8>
}

impl AckBfsw {
  pub fn new() -> Self {
    //let mut header = TelemetryHeader::new(),

    Self {
      header    : TelemetryPacketHeader::new(),
      ack_type  : 1,
      ret_code1 : 0,
      ret_code2 : 0,
      body      : Vec::<u8>::new()
    }
  }
  
  //pub fn to_bytestream(&self) -> Vec<u8> {
  //  let mut stream = Vec::<u8>::new();
  //  let mut s_head = self.header.to_bytestream();
  //  stream.append(&mut s_head);
  //  stream.extend_from_slice(self.payload.as_slice());
  //  //stream.append(&mut self.payload);
  //  stream
  //}
}

impl Serialization for AckBfsw {
  
  const HEAD : u16 = 0x90eb;
  const TAIL : u16 = 0x0000; // there is no tail for telemetry packets
  const SIZE : usize = 13; 

  fn from_bytestream(stream : &Vec<u8>,
                     pos    : &mut usize)
    -> Result<Self, SerializationError> {
    if stream.len() < *pos + 3 {
      return Err(SerializationError::StreamTooShort);
    }
    let mut ack   = AckBfsw::new();
    ack.ack_type  = parse_u8(stream, pos);
    ack.ret_code1 = parse_u8(stream, pos);
    ack.ret_code2 = parse_u8(stream, pos);
    Ok(ack)
  }
  
  fn to_bytestream(&self) -> Vec<u8> {
    let mut stream = Vec::<u8>::new();
    stream.push(self.ack_type);
    stream.push(self.ret_code1);
    stream.push(self.ret_code2);
    stream
  }
}

impl TofPackable for AckBfsw {
  const TOF_PACKET_TYPE : TofPacketType = TofPacketType::BfswAckPacket;
}

impl fmt::Display for AckBfsw {
  fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
    let mut repr = String::from("<AckBfsw:");
    //repr += &(format!("\n  Header      : {}" ,self.sync));
    //repr += &(format!("\n  Packet Type : {}" ,self.ptype));
    //repr += &(format!("\n  Timestamp   : {}" ,self.timestamp));
    repr += &(format!("\n  Ack Type    : {}" ,self.ack_type));
    repr += &(format!("\n  Ret Code1   : {}" ,self.ret_code1));
    repr += &(format!("\n  Ret Code2   : {}>",self.ret_code2));
    write!(f, "{}", repr)
  }
}

//-----------------------------------------------------

#[cfg(feature="pybindings")]
#[pymethods]
impl AckBfsw {
  #[getter]
  fn get_header(&self)    -> TelemetryPacketHeader {
    self.header.clone()
  }
  
  #[getter]
  fn get_ack_type(&self)  -> u8 {
    self.ack_type
  }

  #[getter]
  fn get_ret_code1(&self) -> u8 {
    self.ret_code1
  }

  #[getter]
  fn get_ret_code2(&self) -> u8 {
    self.ret_code2
  }
 
  #[getter]
  fn get_body(&self)      -> Vec<u8> {
    self.body.clone()
  }
}

#[cfg(feature="pybindings")]
pythonize!(AckBfsw);

