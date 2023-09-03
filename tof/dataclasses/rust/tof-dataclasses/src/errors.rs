//! Specific error types
//!
//!
//!

use std::error::Error;
use std::fmt;

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
#[derive(Debug, Copy, Clone)]
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
    let disp : String;
    match self {
      SerializationError::TailInvalid     => {disp = String::from("TailInvalid");},
      SerializationError::HeadInvalid     => {disp = String::from("HeadInvalid");},
      SerializationError::StreamTooShort  => {disp = String::from("StreamTooShort");},
      SerializationError::StreamTooLong   => {disp = String::from("StreamTooLong");},
      SerializationError::ValueNotFound   => {disp = String::from("ValueNotFound");},
      SerializationError::EventFragment   => {disp = String::from("EventFragment");},
      SerializationError::UnknownPayload  => {disp = String::from("UnknownPayload");},
      SerializationError::WrongByteSize   => {disp = String::from("WrongByteSize");},
      SerializationError::JsonDecodingError   => {disp = String::from("JsonDecodingError");},
    }
    write!(f, "<Serialization Error : {}>", disp)
  }
}

impl Error for SerializationError {
}

////////////////////////////////////////

#[derive(Debug, Copy, Clone)]
pub enum DecodingError {
  //HeaderNotFound,
  ChannelOutOfBounds,
  UnknownType
}

impl fmt::Display for DecodingError {
  fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
    let disp : String;
    match self {
      DecodingError::UnknownType  => {disp = String::from("UnknownType");},
      DecodingError::ChannelOutOfBounds => {disp = String::from("Remember channels start from 1, not 0");},
    }
    write!(f, "<DecodingError Error : {}>", disp)
  }
}

impl Error for DecodingError {
}

////////////////////////////////////////

/// Error to be used for issues with 
/// the communication to the MTB.
#[derive(Debug,Copy,Clone)]
pub enum MasterTriggerError {
  QueueEmpty,
  MaskTooLarge,
  BrokenPackage,
  DAQNotAvailable
}

impl fmt::Display for MasterTriggerError {
  fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
    let disp : String;
    match self {
      MasterTriggerError::QueueEmpty      => {disp = String::from("QueueEmpty");},
      MasterTriggerError::MaskTooLarge    => {disp = String::from("MaskTooLarge");},
      MasterTriggerError::BrokenPackage   => {disp = String::from("BrokenPackage");}
      MasterTriggerError::DAQNotAvailable => {disp = String::from("DAQNotAvaiable");}
    }
    write!(f, "<MasterTriggerError : {}>", disp)
  }
}

impl Error for MasterTriggerError {
}

////////////////////////////////////////


/// Problems in waveform analysis
#[derive(Debug,Copy,Clone)]
pub enum WaveformError {
  TimeIndexOutOfBounds,
  TimesTooSmall,
  NegativeLowerBound,
  OutOfRangeUpperBound,
  OutOfRangeLowerBound,
  DidNotCrossThreshold,
  TooSpiky,
}

impl WaveformError {
  pub fn to_string(&self) -> String {
    let disp : String;
    match self {
      WaveformError::TimeIndexOutOfBounds => {disp = String::from("TimeIndexOutOfBounds");},
      WaveformError::TimesTooSmall        => {disp = String::from("TimesTooSmall");},
      WaveformError::NegativeLowerBound   => {disp = String::from("NegativeLowerBound");},
      WaveformError::OutOfRangeUpperBound => {disp = String::from("OutOfRangeUpperBound");},
      WaveformError::OutOfRangeLowerBound => {disp = String::from("OutOfRangeLowerBound");},
      WaveformError::DidNotCrossThreshold => {disp = String::from("DidNotCrossThreshold");},
      WaveformError::TooSpiky             => {disp = String::from("TooSpiky");},
    }
    disp
  }
}

impl fmt::Display for WaveformError {
  fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
    let disp = self.to_string();
    write!(f, "<WaveformError: {}>", disp)
  }
}

impl Error for WaveformError {
}

////////////////////////////////////////


/// IPBus provides a package format for
/// sending UDP packets with a header.
/// This is used by the MTB to send its
/// packets over UDP
#[derive(Debug,Copy,Clone)]
pub enum IPBusError {
  DecodingFailed,
}

impl fmt::Display for IPBusError {
  fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
    let disp : String;
    match self {
      IPBusError::DecodingFailed => {disp = String::from("DecodingFailed");},
    }
    write!(f, "<IPBusError Error : {}>", disp)
  }
}

impl Error for IPBusError {
}

////////////////////////////////////////

/// Errors when converting events to PaddlePackets
/// or more general, when doing any kind of analysis
///
#[derive(Debug,Copy,Clone)]
pub enum AnalysisError {
  MissingChannel,
  InputBroken,
}

impl fmt::Display for AnalysisError {
  fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
    let disp : String;
    match self {
      AnalysisError::MissingChannel => {disp = String::from("MissingChannel");},
      AnalysisError::InputBroken    => {disp = String::from("InputBroken");},
    }
    write!(f, "<AnalysisError : {}>", disp)
  }
}

impl Error for AnalysisError {
}

