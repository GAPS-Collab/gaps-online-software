use std::error::Error;
use std::fmt;

/// Indicate issues with (de)serialization
#[derive(Debug, Copy, Clone)]
#[repr(u8)]
pub enum CRSerializationError {
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
  IncorrectScleriteType,
  WrongByteSize,
  JsonDecodingError,
  TomlDecodingError,
  Disconnected,
  UnknownError
}

impl CRSerializationError {
  pub fn to_string(&self) -> String {
    match self {
      CRSerializationError::TailInvalid              => {return String::from("TailInvalid");},
      CRSerializationError::HeadInvalid              => {return String::from("HeadInvalid");}, 
      CRSerializationError::TrackerDelimiterInvalid  => {return String::from("TrackerDelimiterInvalid");},
      CRSerializationError::TofDelimiterInvalid      => {return String::from("TofDelimiterInvalid");}, 
      CRSerializationError::StreamTooShort           => {return String::from("StreamTooShort");}, 
      CRSerializationError::StreamTooLong            => {return String::from("StreamTooLong");},  
      CRSerializationError::ValueNotFound            => {return String::from("ValueNotFound");},
      CRSerializationError::EventFragment            => {return String::from("EventFragment");}
      CRSerializationError::UnknownPayload           => {return String::from("UnknownPayload");},
      CRSerializationError::IncorrectPacketType      => {return String::from("IncorrectPacketType");},  
      CRSerializationError::IncorrectScleriteType    => {return String::from("IncorrectScleriteType");},    
      CRSerializationError::WrongByteSize            => {return String::from("WrongByteSize");}, 
      CRSerializationError::JsonDecodingError        => {return String::from("JsonDecodingError");},   
      CRSerializationError::TomlDecodingError        => {return String::from("TomlDecodingError");},  
      CRSerializationError::Disconnected             => {return String::from("Disconnected");}
      CRSerializationError::UnknownError             => {return String::from("UnknownError");}
    }
  }
}

impl fmt::Display for CRSerializationError {
  fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
    let repr = self.to_string();
    write!(f, "<Serialization Error : {}>", repr)
  }
}

impl Error for CRSerializationError {
}

