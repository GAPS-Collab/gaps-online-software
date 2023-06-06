//! Structures for monitoring
//!
//! This is 
//! a) Monitoring the RBs
//! b) Monitoring the tof-computer/main C&C instance
//! c) Monitoring the MTB
//!
//!

use std::fmt;
use crate::serialization::{Serialization,
                           SerializationError,
                           search_for_u16,
                           parse_u8,
                           parse_u16,
                           parse_u32,
                           parse_f32};

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
                     pos       : &mut usize) 
    -> Result<RBMoniData, SerializationError>{

    let mut two_bytes : [u8;2];
    let four_bytes    : [u8;4];
    two_bytes = [stream[*pos],
                 stream[*pos+1]];
    *pos += 2;
    if RBMoniData::HEAD != u16::from_le_bytes(two_bytes) {
      warn!("Packet does not start with HEAD signature");
      return Err(SerializationError::HeadInvalid {});
    }
    four_bytes = [stream[*pos],
                  stream[*pos+1],
                  stream[*pos+2],
                  stream[*pos+3]];
    *pos += 4;
    two_bytes = [stream[*pos],
                 stream[*pos+1]];
    let mut moni_data  = RBMoniData::new();
    moni_data.rate = u32::from_le_bytes(four_bytes);  
    if RBMoniData::TAIL != u16::from_le_bytes(two_bytes) {
      warn!("Packet does not end with TAIL signature");
      return Err(SerializationError::TailInvalid {});
    }
    Ok(moni_data) 
  }
}

/// Monitoring the main tof computer
pub struct TofCmpMoniData {
  pub core1_tmp : u8,
  pub core2_tmp : u8,
  pub pch_tmp   : u8
}

impl TofCmpMoniData {
  const SIZE : usize = 7;
  const HEAD : u16   = 0xAAAA;
  const TAIL : u16   = 0x5555;
  
  pub fn new() -> TofCmpMoniData {
    TofCmpMoniData {
      core1_tmp : 0,
      core2_tmp : 0,
      pch_tmp   : 0
    }
  }
}

impl Default for TofCmpMoniData {
  fn default() -> TofCmpMoniData {
    TofCmpMoniData::new()
  }
}

impl fmt::Display for TofCmpMoniData {
  fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
    write!(f, "<TofCmpMoniData:\n
           \t CORE1 TMP [C] {}\n
           \t CORE2 TMP [C] {}\n
           \t PCH TMP [C] {}>",
           self.core1_tmp, self.core2_tmp, self.pch_tmp)
  }
}

impl Serialization for TofCmpMoniData {

  fn to_bytestream(&self) -> Vec<u8> {
    let mut stream = Vec::<u8>::with_capacity(TofCmpMoniData::SIZE);
    stream.extend_from_slice(&TofCmpMoniData::HEAD.to_le_bytes());
    stream.extend_from_slice(&self.core1_tmp  .to_le_bytes());
    stream.extend_from_slice(&self.core2_tmp  .to_le_bytes());
    stream.extend_from_slice(&self.pch_tmp    .to_le_bytes());
    stream.extend_from_slice(&TofCmpMoniData::TAIL.to_le_bytes());
    stream
  }

  fn from_bytestream(stream : &Vec<u8>, pos : &mut usize)
    -> Result<TofCmpMoniData, SerializationError> {
    let mut moni_data = TofCmpMoniData::new();
    let head_pos = search_for_u16(TofCmpMoniData::HEAD, stream, *pos)?; 
    let tail_pos = search_for_u16(TofCmpMoniData::TAIL, stream, head_pos + TofCmpMoniData::SIZE-2)?;
    // At this state, this can be a header or a full event. Check here and
    // proceed depending on the options
    if tail_pos + 2 - head_pos != TofCmpMoniData::SIZE {
      error!("TofCmpMoniData incomplete. Seing {} bytes, but expecting {}", tail_pos + 2 - head_pos, TofCmpMoniData::SIZE);
      //error!("{:?}", &stream[head_pos + 18526..head_pos + 18540]);
      *pos = head_pos + 2; //start_pos += RBBinaryDump::SIZE;
      return Err(SerializationError::WrongByteSize);
    }
    *pos = head_pos + 2; 
    moni_data.core1_tmp  = parse_u8(&stream, pos);
    moni_data.core2_tmp  = parse_u8(&stream, pos);
    moni_data.pch_tmp    = parse_u8(&stream, pos);
    *pos += 2; // since we deserialized the tail earlier and 
              // didn't account for it
    Ok(moni_data)
  }
}

/// Monitoring the MTB
pub struct MtbMoniData {
  pub fpga_temp    : f32,
  pub fpga_vccint  : f32,
  pub fpga_vccaux  : f32,
  pub fpga_vccbram : f32,
  pub rate         : u16,
  pub lost_rate    : u16
}

impl MtbMoniData {
  const SIZE : usize = 24;
  const HEAD : u16   = 0xAAAA;
  const TAIL : u16   = 0x5555;
  
  pub fn new() -> MtbMoniData {
    MtbMoniData {
      fpga_temp    : -4242.42,
      fpga_vccint  : -4242.42,
      fpga_vccaux  : -4242.42,
      fpga_vccbram : -4242.42,
      rate         : u16::MAX,
      lost_rate    : u16::MAX
    }
  }
}

impl Default for MtbMoniData {
  fn default() -> MtbMoniData {
    MtbMoniData::new()
  }
}

impl fmt::Display for MtbMoniData {
  fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
    write!(f, "<MtbMoniData:\n
           \t MTB  EVT RATE {}\n
           \t LOST EVT RATE {}\n
           \t FPGA TMP     [C] {}\n
           \t FPGA VCCINT  [V] {}\n
           \t FPGA VCCAUX  [V] {}\n
           \t FPGA VCCBRAM [V] {}\n>",
           self.rate,
           self.lost_rate,
           self.fpga_vccint,
           self.fpga_vccaux,
           self.fpga_vccbram,
           self.fpga_temp)
  }
}

impl Serialization for MtbMoniData {

  fn to_bytestream(&self) -> Vec<u8> {
    let mut stream = Vec::<u8>::with_capacity(MtbMoniData::SIZE);
    stream.extend_from_slice(&MtbMoniData::HEAD.to_le_bytes());
    stream.extend_from_slice(&self.fpga_temp   .to_le_bytes());
    stream.extend_from_slice(&self.fpga_vccint .to_le_bytes());
    stream.extend_from_slice(&self.fpga_vccaux .to_le_bytes());
    stream.extend_from_slice(&self.fpga_vccbram.to_le_bytes());
    stream.extend_from_slice(&self.rate        .to_le_bytes());
    stream.extend_from_slice(&self.lost_rate   .to_le_bytes());
    stream.extend_from_slice(&MtbMoniData::TAIL.to_le_bytes());
    stream
  }

  fn from_bytestream(stream : &Vec<u8>, pos : &mut usize)
    -> Result<MtbMoniData, SerializationError> {
    let mut moni_data = MtbMoniData::new();
    let head_pos = search_for_u16(MtbMoniData::HEAD, stream, *pos)?; 
    let tail_pos = search_for_u16(MtbMoniData::TAIL, stream, head_pos + MtbMoniData::SIZE-2)?;
    // At this state, this can be a header or a full event. Check here and
    // proceed depending on the options
    if tail_pos + 2 - head_pos != MtbMoniData::SIZE {
      error!("MtbMoniData incomplete. Seing {} bytes, but expecting {}", tail_pos + 2 - head_pos, TofCmpMoniData::SIZE);
      //error!("{:?}", &stream[head_pos + 18526..head_pos + 18540]);
      *pos = head_pos + 2; //start_pos += RBBinaryDump::SIZE;
      return Err(SerializationError::WrongByteSize);
    }
    *pos = head_pos + 2; 
    moni_data.fpga_temp    = parse_f32(&stream, pos);
    moni_data.fpga_vccint  = parse_f32(&stream, pos);
    moni_data.fpga_vccaux  = parse_f32(&stream, pos);
    moni_data.fpga_vccbram = parse_f32(&stream, pos);
    moni_data.rate         = parse_u16(&stream, pos);
    moni_data.lost_rate    = parse_u16(&stream, pos);
    *pos += 2; // since we deserialized the tail earlier and 
              // didn't account for it
    Ok(moni_data)
  }
}

