//! Specific error types
//!
//!
//!

use std::error::Error;
use std::fmt;

////////////////////////////////////////

#[derive(Debug, Copy, Clone, serde::Deserialize, serde::Serialize)]
#[repr(u8)]
pub enum PacketError {
  WrongPacketType,
  UnableToSendPacket,
}

impl fmt::Display for PacketError {
  fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
    let disp = serde_json::to_string(self).unwrap_or(
      String::from("Error: cannot unwrap this SensorError"));
    write!(f, "<PacketError : {}>", disp)
  }
}

impl Error for PacketError {
}

////////////////////////////////////////

#[derive(Debug, Copy, Clone, serde::Deserialize, serde::Serialize)]
#[repr(u8)]
pub enum SensorError {
  ReadoutError,
}

impl fmt::Display for SensorError {
  fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
    let disp = serde_json::to_string(self).unwrap_or(
      String::from("Error: cannot unwrap this SensorError"));
    write!(f, "<ReadoutError : {}>", disp)
  }
}

impl Error for SensorError {
}

////////////////////////////////////////

#[derive(Debug, Copy, Clone, serde::Deserialize, serde::Serialize)]
#[repr(u8)]
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

#[derive(Debug, Copy, Clone, serde::Deserialize, serde::Serialize)]
#[repr(u8)]
pub enum RunError {
  EmptyInputData,
  CanNotConnectToMyOwnZMQSocket  
}

impl fmt::Display for RunError {
  fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
    let disp = serde_json::to_string(self).unwrap_or(
      String::from("Error: cannot unwrap this RunError"));
    write!(f, "<RunError : {}>", disp)
  }
}

impl Error for RunError {
}

////////////////////////////////////////

#[derive(Debug, Copy, Clone, serde::Deserialize, serde::Serialize)]
#[repr(u8)]
pub enum EventError {
    EventIdMismatch
}

impl fmt::Display for EventError {
  fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
    let disp = serde_json::to_string(self).unwrap_or(
      String::from("Error: cannot unwrap this EventError"));
    write!(f, "<EventError : {}>", disp)
  }
}

impl Error for EventError {
}

////////////////////////////////////////

/// This error shall be thrown whenver there
/// is an issue in the de(serialization),
/// e.g. the from_bytestream methods.
#[derive(Debug, Copy, Clone, serde::Deserialize, serde::Serialize)]
#[repr(u8)]
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
  JsonDecodingError,
  TomlDecodingError,
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
#[repr(u8)]
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
#[derive(Debug, Copy, Clone, PartialEq, serde::Deserialize, serde::Serialize)]
#[repr(u8)]
pub enum MasterTriggerError {
  Unknown,
  QueueEmpty,
  MaskTooLarge,
  BrokenPackage,
  DAQNotAvailable,
  PackageFormatIncorrect,
  PackageHeaderIncorrect,
  PackageFooterIncorrect,
  FailedOperation,
  DataTooShort
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

// Implement the From trait to convert from Box<dyn StdError>
impl From<Box<dyn std::error::Error>> for MasterTriggerError {
  fn from(err: Box<dyn std::error::Error>) -> Self {
    error!("Converting {err} to MasterTriggerError! Exact error type might be incorrect!");
    MasterTriggerError::FailedOperation
  }
}

////////////////////////////////////////


/// Problems in waveform analysis
#[derive(Debug, Copy, Clone, serde::Deserialize, serde::Serialize)]
#[repr(u8)]
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
#[derive(Debug, Copy, Clone, PartialEq, serde::Deserialize, serde::Serialize)]
#[repr(u8)]
pub enum IPBusError {
  DecodingFailed,
  InvalidTransactionID,
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

#[derive(Debug,Copy,Clone, serde::Deserialize, serde::Serialize)]
#[repr(u8)]
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
#[repr(u8)]
pub enum UserError {
  IneligibleChannelLabel,
  NoChannel9Data,
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

////////////////////////////////////////

#[derive(Debug, Copy, Clone, serde::Deserialize, serde::Serialize)]
#[repr(u8)]
pub enum SetError {
  EmptyInputData,
  CanNotConnectToMyOwnZMQSocket  
}

impl fmt::Display for SetError {
  fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
    let disp = serde_json::to_string(self).unwrap_or(
      String::from("Error: cannot unwrap this SetError"));
    write!(f, "<SetError : {}>", disp)
  }
}

impl Error for SetError {
}
