//! The following file is part of gaps-online-software and published 
//! under the GPLv3 license

use std::fmt;
use std::error::Error;

/// Indicate issues with (de)serialization
#[derive(Debug, Copy, Clone)]
#[repr(u8)]
pub enum SerializationError {
  //HeaderNotFound,
  TailInvalid,
  HeadInvalid,
  TrackerDelimiterInvalid,
  TofDelimiterInvalid,
  StreamTooShort,
  StreamTooLong,
  ValueNotFound,
  EventFragment,
  UnknownPayload,
  IncorrectPacketType,
  WrongByteSize,
  JsonDecodingError,
  TomlDecodingError,
  Disconnected
}

impl SerializationError { 
  pub fn to_string(&self) -> String {
    match self {
      SerializationError::TailInvalid              => String::from("TailInvalid"), 
      SerializationError::HeadInvalid              => String::from("HeadInvalid"),     
      SerializationError::TrackerDelimiterInvalid  => String::from("TrackerDelimiterInvalid"),
      SerializationError::TofDelimiterInvalid      => String::from("TofDelimiterInvalid"),
      SerializationError::StreamTooShort           => String::from("StreamTooLong"),
      SerializationError::StreamTooLong            => String::from("StreamTooLong"),
      SerializationError::ValueNotFound            => String::from("ValueNotFound"),
      SerializationError::EventFragment            => String::from("EventFragment"),
      SerializationError::UnknownPayload           => String::from("UnknownPayload"),
      SerializationError::IncorrectPacketType      => String::from("IncorrectPacketType"),
      SerializationError::WrongByteSize            => String::from("WrongByteSize"),
      SerializationError::JsonDecodingError        => String::from("JsonDecodingError"),
      SerializationError::TomlDecodingError        => String::from("TomlDecodingError"),
      SerializationError::Disconnected             => String::from("Disconnected"),
    }
  }
}

impl fmt::Display for SerializationError {
  fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
    write!(f, "<Serialization Error : {}>", self.to_string())
  }
}

impl Error for SerializationError {
}

//------------------------------------------------------------------------

/// IPBus provides a package format for
/// sending UDP packets with a header.
/// This is used by the MTB to send its
/// packets over UDP
#[derive(Debug, Copy, Clone, PartialEq, serde::Deserialize, serde::Serialize)]
#[repr(u8)]
pub enum IPBusError {
  DecodingFailed,
  InvalidTransactionID,
  InvalidPacketID,
  NotAStatusPacket,
  ConnectionTimeout,
  UdpSendFailed,
  UdpReceiveFailed
}

impl IPBusError {
  pub fn to_string(&self) -> String {
    match self {
      IPBusError::DecodingFailed       => String::from("DecodingFailed"),
      IPBusError::InvalidTransactionID => String::from("InvalidTransactionID"),
      IPBusError::InvalidPacketID      => String::from("InvalidPacketID"),
      IPBusError::NotAStatusPacket     => String::from("NotAStatusPacket"),
      IPBusError::ConnectionTimeout    => String::from("ConnectionTimeout"),
      IPBusError::UdpSendFailed        => String::from("UdpSendFailed"),
      IPBusError::UdpReceiveFailed     => String::from("UdpReceiveFailed"),
    }
  }
}

impl fmt::Display for IPBusError {
  fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
    write!(f, "<IPBusError: {}>", self.to_string())
  }
}

impl Error for IPBusError {
}

//------------------------------------------------------------------------

/// Issues occuring when doing waveform analysis
#[derive(Debug, Copy, Clone)]
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
    let disp : &str;
    match self {
      Self::TimeIndexOutOfBounds => { disp = "TimeIndexOutOfBounds"}
      Self::TimesTooSmall        => { disp = "TimesTooSmall"}  
      Self::NegativeLowerBound   => { disp = "NegativeLowerBound"}  
      Self::OutOfRangeUpperBound => { disp = "OutOfRangeUpperBound"}  
      Self::OutOfRangeLowerBound => { disp = "OutOfRangeLowerBound"}  ,
      Self::DidNotCrossThreshold => { disp = "DidNotCrossThreshold"}  
      Self::TooSpiky             => { disp = "TooSpiky"}  
    }
    write!(f, "<WaveformError: {}>", disp)
  }
}

impl Error for WaveformError {
}

//------------------------------------------------------------------------

#[derive(Debug, Copy, Clone)]
#[repr(u8)]
pub enum CalibrationError {
  EmptyInputData,
  CanNotConnectToMyOwnZMQSocket,
  CalibrationFailed,
  WrongBoardId,
  IncompatibleFlightCalibrations,
}

impl fmt::Display for CalibrationError {
  fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
    let repr : &str;
    match self {
      Self::EmptyInputData                => { repr = "EmptyInputData"},
      Self::CanNotConnectToMyOwnZMQSocket => { repr = "CanNotConnectToMyOwnZMQSocket"},
      Self::CalibrationFailed             => { repr = "CalibrationFailed"},
      Self::WrongBoardId                  => { repr = "WrongBoardId"},
      Self::IncompatibleFlightCalibrations => { repr = "IncompatibleFlightCalibrations"},
    }
    write!(f, "<CalibrationError : {}>", repr)
  }
}

impl Error for CalibrationError {
}

