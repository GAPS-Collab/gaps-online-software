///
/// Monitoring 
/// thread + classes
///
///
///
///

use crate::errors::SerializationError;

#[derive(Debug, Copy, Clone, PartialEq)]
pub enum Encoding<T> {
  IsNativeU64(u64),
  IsNativeU32(u32),
  IsNativeU16(u16),
  IsNativeU8(u8),
  IsF64(f64),
  IsF32(f32),
  Unknown(T)
}

#[derive(Debug, Clone)]
pub struct MonitoringPacket {
  pub label         : String,
  pub label_size    : u8,
  pub payload       : Vec<u8>,
  pub payload_size  : u8
}

impl MonitoringPacket {

  const HEAD : u16 = 0xAAAA;
  const TAIL : u16 = 0x5555;

  pub fn new(label : String, payload : Vec<u8>) -> MonitoringPacket {
    // we don't like long labels
    let label_len = label.len();
    if label_len > 255 {
      panic!("The label is too long and has more than 255 characters! label {}, Please restrict yourself to shorter labels", label); 
    } 

    if payload.len() > 255 {
      panic!("The payload is too long and has more than 255 characters!" ); 
    }

    let payload_len = payload.len() as u8;
    // we disect the value in bytes here
    MonitoringPacket  { 
      label         : label,
      label_size    : label_len as u8,
      payload       : payload,
      payload_size  : payload_len
    }
  }

  pub fn to_bytestream(&self) -> Vec<u8> { 
    let mut bytestream = Vec::<u8>::with_capacity(256);
    bytestream.extend_from_slice(&MonitoringPacket::HEAD.to_le_bytes());
    bytestream.push(self.label_size);
    bytestream.extend_from_slice(self.label.as_bytes());
    bytestream.push(self.payload_size);
    bytestream.extend_from_slice(self.payload.as_slice());
    bytestream.extend_from_slice(&MonitoringPacket::TAIL.to_le_bytes());
    bytestream
  }
  
  pub fn from_bytestream(bytestream : Vec<u8>, start_pos : usize) 
    -> Result<MonitoringPacket, SerializationError> {
    
    let mut two_bytes : [u8;2];
    two_bytes = [bytestream[start_pos],
                 bytestream[start_pos+2]];
        

    if MonitoringPacket::HEAD != u16::from_le_bytes(two_bytes) {
      warn!("Packet does not start with HEAD signature");
      return Err(SerializationError::HeadInvalid {});
    }
    
    let mut payload  = Vec::<u8>::with_capacity(256);
    let label_size   = bytestream[2];
    let label        = String::from_utf8((&bytestream[3..=label_size as usize + 3]).to_vec()).unwrap();
    let payload_size = bytestream[2+label_size as usize];
    payload.extend_from_slice(&bytestream[4 + label_size as usize..payload_size as usize + label_size as usize +4]);

    two_bytes = [bytestream[payload_size as usize + label_size as usize + 4],
                 bytestream[payload_size as usize + label_size as usize + 5]];

    if MonitoringPacket::TAIL != u16::from_le_bytes(two_bytes) {
      warn!("Packet does not end with TAIL signature");
      return Err(SerializationError::TailInvalid {});
    }
    let mp = MonitoringPacket::new(label, payload);
    Ok(mp)
  }



