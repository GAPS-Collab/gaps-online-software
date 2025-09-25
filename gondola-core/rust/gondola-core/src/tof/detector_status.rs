//! Short notation for TOF detector status
//! 1 bit per working channel
//!
// This file is part of gaps-online-software and published 
// under the GPLv3 license

use crate::prelude::*;

/// Report dead channels/non-active detectors
/// for the TOF system
///
/// This is a very simple approach
/// A channels are the paddle_id - 1
/// while B channels are encoded as paddle_id - 159
///
/// Dead channels will be 0, active channels 
/// will be 1
#[derive(Debug, Copy, Clone, PartialEq)]
#[cfg_attr(feature="pybindings", pyclass)]
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
      // full hex notation to remind us what 
      // these numbers represent
      channels000_031 : 0x00000000,
      channels032_063 : 0x00000000,
      channels064_095 : 0x00000000,
      channels096_127 : 0x00000000,
      channels128_159 : 0x00000000,
      channels160_191 : 0x00000000,
      channels192_223 : 0x00000000,
      channels224_255 : 0x00000000,
      channels256_297 : 0x00000000,
      channels298_319 : 0x00000000,
    }
  }

  /// Update the dead channel list form a HashMap with 
  /// paddle information as it is created in the 
  /// RB communication threads of liftof-cc
  pub fn update_from_map(&mut self, paddles : HashMap<u16,bool>) {
    for k in 0..320 {
      if let Some(val) = paddles.get(&(&k + 1)) {
        if k < 32 && *val {
          self.channels000_031 = self.channels000_031 | (k as u32) ;
        } else if k < 64 && *val  {
          self.channels032_063 = self.channels032_063 | (k as u32) - 32;
        } else if k < 96 && *val  {
          self.channels064_095 = self.channels064_095 | (k as u32) - 64;
        } else if k < 128 && *val {
          self.channels096_127 = self.channels096_127 | (k as u32) - 96;
        } else if k < 160 && *val {
          self.channels128_159 = self.channels128_159 | (k as u32) - 125;
        } else if k < 192 && *val {
          self.channels160_191 = self.channels160_191 | (k as u32) - 160;
        } else if k < 224 && *val {
          self.channels192_223 = self.channels192_223 | (k as u32) - 192;
        } else if k < 256 && *val {
          self.channels224_255 = self.channels224_255 | (k as u32) - 224;
        } else if k < 298 && *val {
          self.channels256_297 = self.channels256_297 | (k as u32) - 256;
        } else if k < 320 && *val {
          self.channels298_319 = self.channels298_319 | (k as u32) - 298;
        }
      } else {
        error!("No entry in paddle status map for channel {}", k);
        continue;
      }
    }
  }

  /// Get all paddle ids which have dead 
  /// channels on the A-side
  pub fn get_dead_paddles_a(&self) -> Vec<u8> {
    let mut dead_a = Vec::<u8>::new();
    let inactive = self.get_inactive_channels_idx();
    for k in inactive.iter() {
      if *k < 160 {
        dead_a.push(*k as u8);
      }
    }
    dead_a
  }

  /// Get all paddle ids which have dead 
  /// channels on the B-side
  pub fn get_dead_paddles_b(&self) -> Vec<u8> {
    let mut dead_b = Vec::<u8>::new();
    let inactive = self.get_inactive_channels_idx();
    for k in inactive.iter() {
      if *k >= 160 {
        dead_b.push((*k-159) as u8);
      }
    }
    dead_b
  }

  /// Index of inactive channels in the range of 
  /// 0-319. These indizes are MTBChannel numbers
  fn get_inactive_channels_idx(&self) -> Vec<u16> {
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

  ///// Index of inactive channels in the range of 
  ///// 0-319. These indizes are MTBChannel numbers
  //fn get_active_channels_idx(&self) -> Vec<u16> {
  //  let inactive_channels   = self.get_inactive_channels_idx();
  //  let mut active_channels = Vec::<u16>::new();
  //  for ch in 0..329 {
  //    if !inactive_channels.contains(&ch) {
  //      active_channels.push(ch);
  //    }
  //  }
  //  active_channels
  //}
}

impl Default for TofDetectorStatus {
  fn default() -> Self {
    Self::new()
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

impl TofPackable for TofDetectorStatus {
  const TOF_PACKET_TYPE : TofPacketType = TofPacketType::TofDetectorStatus;
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

#[cfg(feature="pybindings")]
#[pymethods]
impl TofDetectorStatus {
  
  #[getter]
  fn channels000_031(&self) -> u32 {
    self.channels000_031
  }

  #[getter]
  fn channels032_063(&self) -> u32 { 
    self.channels032_063
  }

  #[getter]
  fn channels064_095(&self) -> u32 { 
    self.channels064_095
  }

  #[getter]
  fn channels096_127(&self) -> u32 { 
    self.channels096_127
  }  

  #[getter]
  fn channels128_159(&self) -> u32 { 
    self.channels128_159
  }

  #[getter]
  fn channels160_191(&self) -> u32 { 
    self.channels160_191
  }

  #[getter]
  fn channels192_223(&self) -> u32 { 
    self.channels192_223
  }

  #[getter]
  fn channels224_255(&self) -> u32 { 
    self.channels224_255
  }
  
  #[getter]
  fn channels256_297(&self) -> u32 { 
    self.channels256_297
  }

  #[getter]
  fn channels298_319(&self) -> u32 { 
    self.channels298_319
  }
}

#[cfg(feature="pybindings")]
pythonize_packable!(TofDetectorStatus);


#[cfg(feature = "random")]
impl FromRandom for TofDetectorStatus {
  fn from_random() -> Self {
    let mut status  = TofDetectorStatus::new();
    let mut rng     = rand::rng();
    status.channels000_031 = rng.random::<u32>();
    status.channels032_063 = rng.random::<u32>();
    status.channels064_095 = rng.random::<u32>();
    status.channels096_127 = rng.random::<u32>();
    status.channels128_159 = rng.random::<u32>();
    status.channels160_191 = rng.random::<u32>();
    status.channels192_223 = rng.random::<u32>();
    status.channels224_255 = rng.random::<u32>();
    status.channels256_297 = rng.random::<u32>();
    status.channels298_319 = rng.random::<u32>();
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

