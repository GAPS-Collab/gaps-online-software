//! Specific error types
//!
//!
//!

use std::error::Error;
use std::fmt;

#[derive(Debug)]
pub enum SerializationError {
  //HeaderNotFound,
  TailInvalid,
  HeadInvalid,
  StreamTooShort,
  StreamTooLong,
  ValueNotFound,
  EventFragment,
  UnknownPayload,
  WrongByteSize
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
    }
    write!(f, "<Serialization Error : {}>", disp)
  }
}

impl Error for SerializationError {
}

#[derive(Debug)]
pub enum DecodingError {
  //HeaderNotFound,
  UnknownType
}

impl fmt::Display for DecodingError {
  fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
    let disp : String;
    match self {
      DecodingError::UnknownType  => {disp = String::from("UnknownType");},
    }
    write!(f, "<DecodingError Error : {}>", disp)
  }
}

impl Error for DecodingError {
}


#[derive(Debug)]
pub enum MasterTriggerError {
  QueueEmpty,
  MaskTooLarge
}

impl fmt::Display for MasterTriggerError {
  fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
    let disp : String;
    match self {
      MasterTriggerError::QueueEmpty => {disp = String::from("QueueEmpty");},
      MasterTriggerError::MaskTooLarge => {disp = String::from("MaskTooLarge");},
    }
    write!(f, "<MasterTriggerError : {}>", disp)
  }
}

impl Error for MasterTriggerError {
}

#[derive(Debug)]
pub enum WaveformError {
  TimeIndexOutOfBounds,
  TimesTooSmall,
  NegativeLowerBound,
  OutOfRangeUpperBound
}

impl fmt::Display for WaveformError {
  fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
    let disp : String;
    match self {
      WaveformError::TimeIndexOutOfBounds => {disp = String::from("TimeIndexOutOfBounds");},
      WaveformError::TimesTooSmall        => {disp = String::from("TimesTooSmall");},
      WaveformError::NegativeLowerBound   => {disp = String::from("NegativeLowerBound");},
      WaveformError::OutOfRangeUpperBound => {disp = String::from("OutOfRangeUpperBound");},
    }
    write!(f, "<WaveformError Error : {}>", disp)
  }
}

impl Error for WaveformError {
}

#[derive(Debug)]
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
