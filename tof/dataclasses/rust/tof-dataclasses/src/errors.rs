//! Specific error types
//!
//!
//!

use std::error::Error;
use std::fmt;

extern crate serde;
extern crate serde_json;

////////////////////////////////////////

#[derive(Debug, Copy, Clone, serde::Deserialize, serde::Serialize)]
pub enum CalibrationError {
  EmptyInputData,
  CanNotConnectToMyOwnZMQSocket  
}

impl fmt::Display for CalibrationError {
  fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
    let disp = serde_json::to_string(self).unwrap_or(
      String::from("Error: cannot unwrap this CalibrationError"));
    write!(f, "<CalibrationError : {}>", disp)
  }
}

impl Error for CalibrationError {
}

////////////////////////////////////////

#[derive(Debug, Copy, Clone)]
pub enum EventError {
    EventIdMismatch
}

impl fmt::Display for EventError {
  fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
    write!(f,"<EventError>")
  }
}

impl Error for EventError {
}

////////////////////////////////////////

/// This error shall be thrown whenver there
/// is an issue in the de(serialization),
/// e.g. the from_bytestream methods.
#[derive(Debug, Copy, Clone, serde::Deserialize, serde::Serialize)]
pub enum SerializationError {
  //HeaderNotFound,
  TailInvalid,
  HeadInvalid,
  StreamTooShort,
  StreamTooLong,
  ValueNotFound,
  EventFragment,
  UnknownPayload,
  WrongByteSize,
  JsonDecodingError
}

impl fmt::Display for SerializationError {
  fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
    let disp = serde_json::to_string(self).unwrap_or(
      String::from("Error: cannot unwrap this SerializationError"));
    write!(f, "<Serialization Error : {}>", disp)
  }
}

impl Error for SerializationError {
}

////////////////////////////////////////

#[derive(Debug, Copy, Clone, serde::Deserialize, serde::Serialize)]
pub enum DecodingError {
  //HeaderNotFound,
  ChannelOutOfBounds,
  UnknownType
}

impl fmt::Display for DecodingError {
  fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
    let disp = serde_json::to_string(self).unwrap_or(
      String::from("Error: cannot unwrap this DecodingError"));
    write!(f, "<DecodingError Error : {}>", disp)
  }
}

impl Error for DecodingError {
}

////////////////////////////////////////

/// Error to be used for issues with 
/// the communication to the MTB.
#[derive(Debug, Copy, Clone, serde::Deserialize, serde::Serialize)]
pub enum MasterTriggerError {
  QueueEmpty,
  MaskTooLarge,
  BrokenPackage,
  DAQNotAvailable
}

impl fmt::Display for MasterTriggerError {
  fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
    let disp = serde_json::to_string(self).unwrap_or(
      String::from("Error: cannot unwrap this MasterTriggerError"));
    write!(f, "<MasterTriggerError : {}>", disp)
  }
}

impl Error for MasterTriggerError {
}

////////////////////////////////////////


/// Problems in waveform analysis
#[derive(Debug, Copy, Clone, serde::Deserialize, serde::Serialize)]
pub enum WaveformError {
  TimeIndexOutOfBounds,
  TimesTooSmall,
  NegativeLowerBound,
  OutOfRangeUpperBound,
  OutOfRangeLowerBound,
  DidNotCrossThreshold,
  TooSpiky,
}

impl fmt::Display for WaveformError {
  fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
    let disp = serde_json::to_string(self).unwrap_or(
      String::from("Error: cannot unwrap this WaveformError"));
    write!(f, "<WaveformError: {}>", disp)
  }
}

// TODO is this needed?
// DONE - Yes, we talked about it. I think you need 
// it if you want to returnt Box<Error>
impl Error for WaveformError {
}

////////////////////////////////////////


/// IPBus provides a package format for
/// sending UDP packets with a header.
/// This is used by the MTB to send its
/// packets over UDP
// TODO Isnt it too overkill to have an enum for this?
#[derive(Debug, Copy, Clone, serde::Deserialize, serde::Serialize)]
pub enum IPBusError {
  DecodingFailed,
}

impl fmt::Display for IPBusError {
  fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
    let disp = serde_json::to_string(self).unwrap_or(
      String::from("Error: cannot unwrap this IPBusError"));
    write!(f, "<IPBusError Error : {}>", disp)
  }
}

impl Error for IPBusError {
}

////////////////////////////////////////

/// Errors when converting events to PaddlePackets
/// or more general, when doing any kind of analysis
///
#[derive(Debug,Copy,Clone, serde::Deserialize, serde::Serialize)]
pub enum AnalysisError {
  MissingChannel,
  InputBroken,
}

impl fmt::Display for AnalysisError {
  fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
    let disp = serde_json::to_string(self).unwrap_or(
      String::from("Error: cannot unwrap this AnalysisError"));
    write!(f, "<AnalysisError : {}>", disp)
  }
}

impl Error for AnalysisError {
}

////////////////////////////////////////

#[derive(Debug,Copy,Clone, serde::Deserialize, serde::Serialize)]
pub enum UserError {
  IneligibleChannelLabel,
}

impl fmt::Display for UserError {
  fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
    let disp = serde_json::to_string(self).unwrap_or(
      String::from("Error: cannot unwrap this UserError"));
    write!(f, "<UserError : {}>", disp)
  }
}

impl Error for UserError {
}

