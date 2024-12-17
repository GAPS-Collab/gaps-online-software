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
//! * database - access a data base for advanced paddle
//!              mapping, readoutboard and ltb information etc.
//!
//! * caraspace - register TofPacket through the caraspace library
//!               This allows to write TofPackets to frames, which 
//!               will ultimatly allow them to write them to 
//!               caraspace files
//!
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
pub mod io;
pub mod analysis;
pub mod ipbus;
pub mod series;
pub mod heartbeats;
pub mod version;
pub mod status;
#[cfg(feature="database")]
pub mod database;
#[cfg(feature="caraspace-serial")]
pub mod caraspace;

pub use version::ProtocolVersion;

#[macro_use] extern crate log;

use std::collections::HashMap;

/// A type for the master trigger mappings
/// Dsi -> J -> (RBID,RBCH)
pub type DsiLtbRBMapping      = HashMap<u8,HashMap<u8,HashMap<u8,(u8,u8)>>>;

pub type RbChPidMapping      = HashMap<u8,HashMap<u8,u8>>;

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

