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

extern crate pretty_env_logger;
#[macro_use] extern crate log;

//pretty_env_logger::init();


