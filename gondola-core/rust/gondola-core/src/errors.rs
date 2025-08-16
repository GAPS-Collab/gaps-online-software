//! The following file is part of gaps-online-software and published 
//! under the GPLv3 license

use crate::prelude::*;

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
  Disconnected,
  ObjectNotFound,
  UnsupportedVersion
}

impl SerializationError { 
  pub fn as_str(&self) -> &str {
    match self {
      SerializationError::TailInvalid              => "TailInvalid", 
      SerializationError::HeadInvalid              => "HeadInvalid",     
      SerializationError::TrackerDelimiterInvalid  => "TrackerDelimiterInvalid",
      SerializationError::TofDelimiterInvalid      => "TofDelimiterInvalid",
      SerializationError::StreamTooShort           => "StreamTooLong",
      SerializationError::StreamTooLong            => "StreamTooLong",
      SerializationError::ValueNotFound            => "ValueNotFound",
      SerializationError::EventFragment            => "EventFragment",
      SerializationError::UnknownPayload           => "UnknownPayload",
      SerializationError::IncorrectPacketType      => "IncorrectPacketType",
      SerializationError::WrongByteSize            => "WrongByteSize",
      SerializationError::JsonDecodingError        => "JsonDecodingError",
      SerializationError::TomlDecodingError        => "TomlDecodingError",
      SerializationError::Disconnected             => "Disconnected",
      SerializationError::ObjectNotFound           => "ObjectNotFound",
      SerializationError::UnsupportedVersion       => "UnsupportedVersion",
    }
  }
}

impl fmt::Display for SerializationError {
  fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
    write!(f, "<SerializationError : {}>", self.to_string())
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

//------------------------------------------------------------------------

#[derive(Debug,Copy,Clone)]
#[repr(u8)]
pub enum UserError {
  IneligibleChannelLabel,
  NoChannel9Data,
}

impl fmt::Display for UserError {
  fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
    let disp : &str;
    match self {
      UserError::IneligibleChannelLabel => {
        disp = "IneligibleChannelLabel";
      },
      UserError::NoChannel9Data => {
        disp = "NoChannel9Data";
      },
    }
    write!(f, "<UserError : {}>", disp)
  }
}

impl Error for UserError {

//------------------------------------------------------------------------
}

#[derive(Debug,Copy,Clone)]
#[repr(u8)]
pub enum AnalysisError {
  MissingChannel,
  NoChannel9,
  InputBroken,
  DataMangling
}

impl fmt::Display for AnalysisError {
  fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
    let disp : &str;
    match self {
      AnalysisError::MissingChannel => {disp = "MissingChannel"},
      AnalysisError::NoChannel9     => {disp = "NoChannel9"},
      AnalysisError::InputBroken    => {disp = "InputBroken"},
      AnalysisError::DataMangling   => {disp = "DataMangling"}
    
    }
    write!(f, "<AnalysisError : {}>", disp)
  }
}

impl Error for AnalysisError {
}

