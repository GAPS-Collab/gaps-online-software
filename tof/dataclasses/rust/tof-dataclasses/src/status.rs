//! Detector status indicators
//!

use std::fmt;

cfg_if::cfg_if! {
  if #[cfg(feature = "random")]  {
    use crate::FromRandom;
    use rand::Rng;
  }
}

use crate::serialization::{
  Serialization,
  SerializationError,
  Packable,
  parse_u32
};

use crate::packets::PacketType;

/// Report dead channels/non-active detectors
/// for the TOF system
///
/// The reporting system is based on the 
/// "MTBChannel". This includes 10 masks
/// of u32, each reporting activity of 
/// as single channel by an indicator bit
#[derive(Debug, Copy, Clone, PartialEq)]
pub struct TofDetectorStatus {
  pub channels000_031 : u32,
  pub channels032_063 : u32,
  pub channels064_095 : u32,
  pub channels096_127 : u32,
  pub channels128_159 : u32,
  pub channels160_191 : u32,
  pub channels192_223 : u32,
  pub channels224_255 : u32,
  pub channels256_297 : u32,
  pub channels298_319 : u32,
}

impl TofDetectorStatus {
 
  pub fn new() -> Self {
    Self {
      channels000_031 : 0xFFFFFFFF,
      channels032_063 : 0xFFFFFFFF,
      channels064_095 : 0xFFFFFFFF,
      channels096_127 : 0xFFFFFFFF,
      channels128_159 : 0xFFFFFFFF,
      channels160_191 : 0xFFFFFFFF,
      channels192_223 : 0xFFFFFFFF,
      channels224_255 : 0xFFFFFFFF,
      channels256_297 : 0xFFFFFFFF,
      channels298_319 : 0xFFFFFFFF,
    }
  }

  //#[cfg(feature = "database")]
  //pub fn get_inactive_channels(&self) -> MTBChannel {
  //}

  //#[cfg(feature = "database")]
  //pub fn get_active_channels(&self) -> MTBChannel {
  //}

  /// Index of inactive channels in the range of 
  /// 0-319. These indizes are MTBChannel numbers
  pub fn get_inactive_channels_idx(&self) -> Vec<u16> {
    let mut channels = Vec::<u16>::new();
    for k in 0..10 {
      if (self.channels000_031 >> k & 0x1) == 1 {
        channels.push(k);
      }
    }
    for k in 0..10 {
      if (self.channels032_063 >> k & 0x1) == 1 {
        channels.push(k + 32);
      }
    }
    for k in 0..10 {
      if (self.channels064_095 >> k & 0x1) == 1 {
        channels.push(k + 64);
      }
    }
    for k in 0..10 {
      if (self.channels096_127 >> k & 0x1) == 1 {
        channels.push(k + 96);
      }
    }
    for k in 0..10 {
      if (self.channels128_159 >> k & 0x1) == 1 {
        channels.push(k + 128);
      }
    }
    for k in 0..10 {
      if (self.channels160_191 >> k & 0x1) == 1 {
        channels.push(k + 160);
      }
    }
    for k in 0..10 {
      if (self.channels192_223 >> k & 0x1) == 1 {
        channels.push(k + 192);
      }
    }
    for k in 0..10 {
      if (self.channels224_255 >> k & 0x1) == 1 {
        channels.push(k + 224);
      }
    }
    for k in 0..10 {
      if (self.channels256_297 >> k & 0x1) == 1 {
        channels.push(k + 256);
      }
    }
    for k in 0..10 {
      if (self.channels298_319 >> k & 0x1) == 1 {
        channels.push(k + 298);
      }
    }
    channels
  }

  /// Index of inactive channels in the range of 
  /// 0-319. These indizes are MTBChannel numbers
  pub fn get_active_channels_idx(&self) -> Vec<u16> {
    let inactive_channels   = self.get_inactive_channels_idx();
    let mut active_channels = Vec::<u16>::new();
    for ch in 0..329 {
      if !inactive_channels.contains(&ch) {
        active_channels.push(ch);
      }
    }
    active_channels
  }
}

impl Serialization for TofDetectorStatus {
  const HEAD : u16   = 0xAAAA;
  const TAIL : u16   = 0x5555;
  const SIZE : usize = 44; 
  
  fn from_bytestream(stream     : &Vec<u8>,
                     pos        : &mut usize)
    -> Result<Self, SerializationError>{
      Self::verify_fixed(stream, pos)?;
      let mut status = TofDetectorStatus::new();
      status.channels000_031 = parse_u32(stream, pos); 
      status.channels032_063 = parse_u32(stream, pos); 
      status.channels064_095 = parse_u32(stream, pos); 
      status.channels096_127 = parse_u32(stream, pos); 
      status.channels128_159 = parse_u32(stream, pos); 
      status.channels160_191 = parse_u32(stream, pos); 
      status.channels192_223 = parse_u32(stream, pos); 
      status.channels224_255 = parse_u32(stream, pos); 
      status.channels256_297 = parse_u32(stream, pos); 
      status.channels298_319 = parse_u32(stream, pos); 
      *pos += 2;
      Ok(status)
  } 
  
  fn to_bytestream(&self) -> Vec<u8> {
    let mut bs = Vec::<u8>::with_capacity(Self::SIZE);
    bs.extend_from_slice(&Self::HEAD.to_le_bytes());
    bs.extend_from_slice(&self.channels000_031.to_le_bytes());
    bs.extend_from_slice(&self.channels032_063.to_le_bytes());
    bs.extend_from_slice(&self.channels064_095.to_le_bytes());
    bs.extend_from_slice(&self.channels096_127.to_le_bytes());
    bs.extend_from_slice(&self.channels128_159.to_le_bytes());
    bs.extend_from_slice(&self.channels160_191.to_le_bytes());
    bs.extend_from_slice(&self.channels192_223.to_le_bytes());
    bs.extend_from_slice(&self.channels224_255.to_le_bytes());
    bs.extend_from_slice(&self.channels256_297.to_le_bytes());
    bs.extend_from_slice(&self.channels298_319.to_le_bytes());
    bs.extend_from_slice(&Self::TAIL.to_le_bytes());
    bs
  }
}

impl Packable for TofDetectorStatus {
  const PACKET_TYPE : PacketType = PacketType::TofDetectorStatus;
}

impl fmt::Display for TofDetectorStatus {
  fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
    let mut repr : String = String::from("<TofDetectorStatus");
    repr += &(format!("\n Ch 000 - 031 {:x}", &self.channels000_031));
    repr += &(format!("\n Ch 032 - 063 {:x}", &self.channels032_063));
    repr += &(format!("\n Ch 064 - 095 {:x}", &self.channels064_095));
    repr += &(format!("\n Ch 096 - 127 {:x}", &self.channels096_127));
    repr += &(format!("\n Ch 128 - 159 {:x}", &self.channels128_159));
    repr += &(format!("\n Ch 160 - 191 {:x}", &self.channels160_191));
    repr += &(format!("\n Ch 192 - 223 {:x}", &self.channels192_223));
    repr += &(format!("\n Ch 224 - 255 {:x}", &self.channels224_255));
    repr += &(format!("\n Ch 256 - 297 {:x}", &self.channels256_297));
    repr += &(format!("\n Ch 298 - 319 {:x}>", &self.channels298_319));
    write!(f, "{}", repr)
  }
}

#[cfg(feature = "random")]
impl FromRandom for TofDetectorStatus {
  fn from_random() -> Self {
    let mut status  = TofDetectorStatus::new();
    let mut rng     = rand::thread_rng();
    status.channels000_031 = rng.gen::<u32>();
    status.channels032_063 = rng.gen::<u32>();
    status.channels064_095 = rng.gen::<u32>();
    status.channels096_127 = rng.gen::<u32>();
    status.channels128_159 = rng.gen::<u32>();
    status.channels160_191 = rng.gen::<u32>();
    status.channels192_223 = rng.gen::<u32>();
    status.channels224_255 = rng.gen::<u32>();
    status.channels256_297 = rng.gen::<u32>();
    status.channels298_319 = rng.gen::<u32>();
    status
  }
}

#[cfg(feature = "random")]
#[test]
fn pack_tofdetectorstatus() {
  for _ in 0..100 {
    let status  = TofDetectorStatus::from_random();
    let test : TofDetectorStatus = status.pack().unpack().unwrap();
    assert_eq!(status, test);
  }
}

