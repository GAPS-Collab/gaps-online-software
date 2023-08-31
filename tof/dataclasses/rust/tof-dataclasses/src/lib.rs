//! Tof dataclasses
//!     
//! This crate provides tof related dataclasses for 
//!
//! * events
//!
//! * network i/o wrappers ("packets") for classes
//!
//! * function related constants
//!
//! * calibration
//!
//! * commands/responses
//!
//! * TODO: alerts

pub mod events;
pub mod packets;
pub mod errors;
pub mod serialization;
pub mod constants;
pub mod calibrations;
pub mod threading;
pub mod commands;
pub mod monitoring;
pub mod manifest;
pub mod run;
pub mod io;

extern crate pretty_env_logger;
#[macro_use] extern crate log;

/// Create structures filled with random 
/// number to be used for testing and 
/// benchmarking
#[cfg(feature = "random")]
pub trait FromRandom {
  fn from_random() -> Self;
}

/// Representation of 32 bit mask 
pub struct BitMask32 {
}

impl BitMask32 {
  
  /// A boolean array representation of the Bitmask
  pub fn decode(bitmask : u32) -> [bool;32] {
    let mut decoded_mask = [false;32];
    // FIXME this implicitly asserts that the fields for non available LTBs 
    // will be 0 and all the fields will be in order 
    let mut index = 32 - 1;
    for n in 0..32 {
      let mask = 1 << n;
      let bit_is_set = (mask & bitmask) > 0;
      decoded_mask[index] = bit_is_set;
      if index != 0 {
          index -= 1;
      }
    }
    decoded_mask
  }
}
