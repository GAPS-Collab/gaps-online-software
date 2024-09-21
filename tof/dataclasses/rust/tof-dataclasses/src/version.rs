#[cfg(feature="random")]
use crate::FromRandom;

#[cfg(feature="random")]
use rand::Rng;

use std::fmt;

#[cfg(feature = "pybindings")]
use pyo3::pyclass;

/// Use the fisrt 3 bits (most significant) in 
/// the event status field for conveyilng versio
/// information
/// This means all numbers have to be > 64
#[derive(Debug, Copy, Clone, PartialEq, serde::Deserialize, serde::Serialize)]
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
    let r = serde_json::to_string(self).unwrap_or(
      String::from("Error: Unknown/Incompatible verison"));
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
      0u8   => ProtocolVersion::Unknown,
      64u8  => ProtocolVersion::V1,
      128u8 => ProtocolVersion::V2,
      192u8 => ProtocolVersion::V3,
      _     => ProtocolVersion::Unknown
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
    let mut rng  = rand::thread_rng();
    let idx = rng.gen_range(0..choices.len());
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

