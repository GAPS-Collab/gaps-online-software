//! Structures for monitoring
//!
//! This is 
//! a) Monitoring the RBs
//! b) Monitoring the tof-computer/main C&C instance
//!
//!
//!

use crate::serialization::{Serialization,
                           SerializationError};

/// A collection of monitoring data
/// from the readoutboards
pub struct RBMoniData {

  pub rate : u32,

}

impl RBMoniData {

  pub const HEAD : u16 = 0xAAAA;
  pub const TAIL : u16 = 0x5555;
  /// The data size when serialized to a bytestream
  /// This needs to be updated when we change the 
  /// packet layout, e.g. add new members.
  /// HEAD + TAIL + sum(sizeof(m) for m in _all_members_))
  pub const SIZE : usize  = 6;

  pub fn new() -> RBMoniData {
    RBMoniData {
      rate : 0,  
    }
  }

  pub fn to_bytestream(&self) -> Vec<u8> {
    let mut bytestream = Vec::<u8>::with_capacity(RBMoniData::SIZE);
    bytestream.extend_from_slice(&RBMoniData::HEAD.to_le_bytes());
    bytestream.extend_from_slice(&self.rate.to_le_bytes());
    bytestream.extend_from_slice(&RBMoniData::TAIL.to_le_bytes());
    bytestream
  }
}

impl Default for RBMoniData {
  fn default() -> RBMoniData {
    RBMoniData::new()
  }
}

impl Serialization for RBMoniData {
  fn from_bytestream(stream    : &Vec<u8>, 
                     start_pos : usize) 
    -> Result<RBMoniData, SerializationError>{

    let mut pos      = start_pos; 
    let mut two_bytes : [u8;2];
    let four_bytes    : [u8;4];
    two_bytes = [stream[pos],
                 stream[pos+1]];
    pos += 2;
    if RBMoniData::HEAD != u16::from_le_bytes(two_bytes) {
      warn!("Packet does not start with HEAD signature");
      return Err(SerializationError::HeadInvalid {});
    }
    four_bytes = [stream[pos],
                  stream[pos+1],
                  stream[pos+2],
                  stream[pos+3]];
    pos += 4;
    two_bytes = [stream[pos],
                 stream[pos+1]];
    let mut moni_data  = RBMoniData::new();
    moni_data.rate = u32::from_le_bytes(four_bytes);  
    if RBMoniData::TAIL != u16::from_le_bytes(two_bytes) {
      warn!("Packet does not end with TAIL signature");
      return Err(SerializationError::TailInvalid {});
    }
    Ok(moni_data) 
  }
}

