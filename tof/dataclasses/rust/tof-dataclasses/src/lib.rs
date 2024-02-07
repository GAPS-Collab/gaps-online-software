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
//!
//! # features:
//!
//! * random - allow random number generated data classes for 
//!            testing
//!
//! * database - access the SQLite data base for advanced paddle
//!              mapping
//!

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
pub mod analysis;

#[macro_use] extern crate log;

use std::collections::HashMap;

/// A type for the master trigger mappings
/// Dsi -> J -> (RBID,RBCH)
pub type DsiLtbRBMapping      = HashMap<u8,HashMap<u8,HashMap<u8,(u8,u8)>>>;

/// A type for the mappings of RB channels - paddle edn ids
/// Paddle end ids are the paddle id + 1000 for A and 
/// + 2000 for B
/// <div class="warning">In this map RB Channels start from 1! This is consistent with the database</div>
pub type RBChannelPaddleEndIDMap = HashMap<u8,u16>;

/// Create structures filled with random 
/// number to be used for testing and 
/// benchmarking
#[cfg(feature = "random")]
pub trait FromRandom {
  fn from_random() -> Self;
}


