//! The following file is part of gaps-online-software and published 
//! under the GPLv3 license

#[cfg(feature="random")]
use crate::random::FromRandom;

#[cfg(feature="random")]
use rand::Rng;

use std::fmt;

#[cfg(feature = "pybindings")]
use pyo3::pyclass;

/// The Protocol version is designed in such 
/// a way that we can "hijack" an existing 
/// field, using the most significant digits.
///
/// It uses the 2 most significant bit of an u8,
/// so it should be possible to basically slap 
/// this on to anyting
#[derive(Debug, Copy, Clone, PartialEq, PartialOrd)]
#[repr(u8)]
#[cfg_attr(feature = "pybindings", pyclass(eq, eq_int))]
pub enum ProtocolVersion {
  Unknown  = 0u8,
  V1       = 64u8,
  V2       = 128u8,
  V3       = 192u8,
}

impl fmt::Display for ProtocolVersion {
  fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
    let r : &str;
    match self {
      ProtocolVersion::Unknown => {r = "Unknown"},
      ProtocolVersion::V1      => {r = "V1"},
      ProtocolVersion::V2      => {r = "V2"},
      ProtocolVersion::V3      => {r = "V3"},
    }
    write!(f, "<ProtocolVersion: {}>", r)
  }
}

impl ProtocolVersion {
  pub fn to_u8(&self) -> u8 {
    match self {
      ProtocolVersion::Unknown => {
        return 0;
      }
      ProtocolVersion::V1 => {
        return 64;
      }
      ProtocolVersion::V2 => {
        return 128;
      }
      ProtocolVersion::V3 => {
        return 192;
      }
    }
  }
}

impl From<u8> for ProtocolVersion {
  fn from(value: u8) -> Self {
    match value {
        0 => ProtocolVersion::Unknown,
       64 => ProtocolVersion::V1,
      128 => ProtocolVersion::V2,
      192 => ProtocolVersion::V3,
      _   => ProtocolVersion::Unknown
    }
  }
}

#[cfg(feature = "random")]
impl FromRandom for ProtocolVersion {
  
  fn from_random() -> Self {
    let choices = [
      ProtocolVersion::Unknown,
      ProtocolVersion::V1,
      ProtocolVersion::V2,
      ProtocolVersion::V3,
    ];
    let mut rng  = rand::rng();
    let idx = rng.random_range(0..choices.len());
    choices[idx]
  }
}

#[test]
#[cfg(feature = "random")]
fn test_protocol_version() {
  for _ in 0..100 {
    let pv    = ProtocolVersion::from_random();
    let pv_u8 = pv.to_u8();
    let u8_pv = ProtocolVersion::from(pv_u8);
    assert_eq!(pv, u8_pv);
  }
}

