use crate::errors::SerializationError;
use crate::serialization::Serialization;

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


///! A very simple packet, which 
///  can transport a single value
///  together with a string label
#[derive(Debug, Clone, PartialEq)]
pub struct ValuePacket {
  pub label         : String,
  pub label_size    : u8,
  pub payload       : Vec<u8>,
  pub payload_size  : u8
}

impl Serialization for ValuePacket {
  fn from_bytestream(bytestream : &Vec<u8>, start_pos : usize) 
    -> Result<ValuePacket, SerializationError> {
    
    let mut two_bytes : [u8;2];
    two_bytes = [bytestream[start_pos],
                 bytestream[start_pos+1]];
        

    if ValuePacket::HEAD != u16::from_le_bytes(two_bytes) {
      warn!("Packet does not start with HEAD signature");
      return Err(SerializationError::HeadInvalid {});
    }
   
    
    let mut payload  = Vec::<u8>::new();
    let label_size   = bytestream[2];
    let label        = String::from_utf8((&bytestream[3..label_size as usize + 3]).to_vec()).unwrap();
    let payload_size = bytestream[3+label_size as usize];
    println!("{} {} {}", label_size, label, payload_size);
    println!("{bytestream:?}");
    println!("{}", bytestream.len());
    if bytestream.len() <= label_size as usize + payload_size as usize + 4 {
      return Err(SerializationError::StreamTooShort {});
    }// head, tail, and the actual sizes 

    payload.extend_from_slice(&bytestream[4 + label_size as usize..payload_size as usize + label_size as usize +4]);

    two_bytes = [bytestream[payload_size as usize + label_size as usize + 4],
                 bytestream[payload_size as usize + label_size as usize + 5]];

    if ValuePacket::TAIL != u16::from_le_bytes(two_bytes) {
      warn!("Packet does not end with TAIL signature");
      return Err(SerializationError::TailInvalid {});
    }
    let mp = ValuePacket::new(label, payload);
    Ok(mp)
  }
}

impl ValuePacket {

  const HEAD : u16 = 0xAAAA;
  const TAIL : u16 = 0x5555;

  pub fn new(label : String, payload : Vec<u8>) -> ValuePacket {
    // we don't like long labels
    
    let label_bytes = label.as_bytes();
    let label_len   = label_bytes.len();
    if label_len > 255 {
      panic!("The label is too long and has more than 255 characters! label {}, Please restrict yourself to shorter labels", label); 
    } 

    if payload.len() > 255 {
      panic!("The payload is too long and has more than 255 characters!" ); 
    }

    let payload_len = payload.len() as u8;
    // we disect the value in bytes here
    ValuePacket  { 
      label         : label,
      label_size    : label_len as u8,
      payload       : payload,
      payload_size  : payload_len
    }
  }

  pub fn to_bytestream(&self) -> Vec<u8> { 
    let mut bytestream = Vec::<u8>::new();
    bytestream.extend_from_slice(&ValuePacket::HEAD.to_le_bytes());
    bytestream.push(self.label_size);
    bytestream.extend_from_slice(self.label.as_bytes());
    bytestream.push(self.payload_size);
    bytestream.extend_from_slice(self.payload.as_slice());
    bytestream.extend_from_slice(&ValuePacket::TAIL.to_le_bytes());
    bytestream
  }
  

  ///! Keep the label the same, but 
  ///  exchange the payload
  ///
  ///  The old payload gets deleted
  pub fn update_payload(&mut self, 
                        payload : Vec<u8>) 
    -> Result<(), SerializationError> {
    if payload.len() > 255 {
      return Err(SerializationError::StreamTooLong {});
    }
    let payload_size  = payload.len() as u8;
    self.payload      = payload;
    self.payload_size = payload_size;
    Ok(())
  }

}



//#[cfg(test)]
//mod test
//

#[test] 
fn serialize_deserialize_roundabout() {

  let label = String::from("foo");
  let value : u64 = 9876545321;
  let mut payload = Vec::<u8>::new();
  payload.extend_from_slice(&value.to_le_bytes());
  let vp = ValuePacket::new(label, payload);
  assert_eq!(vp.payload_size,8); // for 8 bytes for u64

  println!("{vp:?}");

  let bytestream = vp.to_bytestream();
  for k in 0..8 {
    println!("{}", bytestream[k]);
  }
  println!("{bytestream:?}");
  let vp_verify  = ValuePacket::from_bytestream(bytestream, 0).unwrap();
  assert_eq!(vp, vp_verify);
}


