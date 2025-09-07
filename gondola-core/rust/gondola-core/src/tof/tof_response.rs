// This file is part of gaps-online-software and published 
// under the GPLv3 license

use std::fmt;
use crate::packets::{
  TofPackable,
  TofPacketType
};
use crate::io::serialization::Serialization;
use crate::errors::SerializationError;
use crate::io::parsers::*;

#[cfg(feature="random")]
use crate::random::FromRandom;
#[cfg(feature="random")]
use rand::Rng;

#[cfg(feature="pybindings")]
use pyo3::prelude::*; 

//use crate::prelude::*;

#[derive(Debug, Copy, Clone, PartialEq, serde::Deserialize, serde::Serialize)]
#[cfg_attr(feature = "pybindings", pyclass(eq, eq_int))]
#[repr(u8)]
pub enum TofReturnCode {
  Unknown        = 0,
  GeneralFail    = 1,
  GarbledCommand = 2,
  Success        = 200,
}

impl fmt::Display for TofReturnCode {
  fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
    let r = serde_json::to_string(self).unwrap_or(
      String::from("Error: cannot unwrap this TofCommandCode"));
    write!(f, "<TofReturnCode: {}>", r)
  }
}

impl From<u8> for TofReturnCode {
  fn from(value: u8) -> Self {
    match value {
      0   => TofReturnCode::Unknown,
      1   => TofReturnCode::GeneralFail,
      2   => TofReturnCode::GarbledCommand,
      200 => TofReturnCode::Success,
      _   => {
        error!("Can not understand {}", value);
        TofReturnCode::Unknown
      }
    }
  }
}

#[cfg(feature = "random")]
impl FromRandom for TofReturnCode {
  fn from_random() -> Self {
    let choices = [
      TofReturnCode::Unknown,
      TofReturnCode::GarbledCommand,
      TofReturnCode::Success,
      TofReturnCode::GeneralFail,
    ];
    let mut rng  = rand::rng();
    let idx = rng.random_range(0..choices.len());
    choices[idx]
  }
}



/// Each `TofCommand` triggers a `TofResponse` in reply
///
/// The responses are general classes, which carry a more
/// specific 32-bit response code.
#[derive(Debug, Copy, Clone, PartialEq, serde::Deserialize, serde::Serialize)]
pub enum TofResponse {
  Success(u32),
  /// A unknown problem led to a non-execution
  /// of the command. The error code should tell
  /// more. A re-issue of the command might 
  /// solve the problem.
  GeneralFail(u32),
  /// The requested event is not ready yet. This 
  /// means, it is still lingering in the caches
  /// of the readout boards. If this problem 
  /// occurs many times, it might be helpful to 
  /// reduce the cache size of the readoutboards 
  /// to be more responsive.
  /// The response code is the specific event id
  /// we initially requested.
  EventNotReady(u32),
  /// Somehwere, a serialization error happened. 
  /// It might be worth trying to execute that 
  /// command again.
  SerializationIssue(u32),
  ZMQProblem(u32),
  TimeOut(u32),
  NotImplemented(u32),
  AccessDenied(u32),
  Unknown
}

impl TofPackable for TofResponse {
  const TOF_PACKET_TYPE : TofPacketType = TofPacketType::TofResponse;
}

impl fmt::Display for TofResponse {
  fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
    let r = serde_json::to_string(self).unwrap_or(
      String::from("Error: cannot unwrap this TofResponse"));
    write!(f, "<TofResponse: {}>", r)
  }
}

#[cfg(feature = "random")]
impl FromRandom for TofResponse {
  
  fn from_random() -> Self {
    let mut rng  = rand::rng();
    let val = rng.random::<u32>();
    let choices = [
      TofResponse::Success(val),
      TofResponse::GeneralFail(val),
      TofResponse::EventNotReady(val),
      TofResponse::SerializationIssue(val),
      TofResponse::ZMQProblem(val),
      TofResponse::TimeOut(val),
      TofResponse::NotImplemented(val),
      TofResponse::AccessDenied(val),
      TofResponse::Unknown,
    ];
    let idx = rng.random_range(0..choices.len());
    choices[idx]
  }
}


impl Serialization for TofResponse {
  const HEAD : u16   = 0xAAAA;
  const TAIL : u16   = 0x5555;
  const SIZE : usize = 9; //FIXME
  
  fn to_bytestream(&self) -> Vec<u8> {
    let mut bytestream = Vec::<u8>::with_capacity(9);
    bytestream.extend_from_slice(&Self::HEAD.to_le_bytes());
    let cc = u8::from(*self);
    bytestream.push(cc);
    let mut value : u32 = 0;
    match self {
      TofResponse::Success(data)            => value = *data,
      TofResponse::GeneralFail(data)        => value = *data,
      TofResponse::EventNotReady(data)      => value = *data,
      TofResponse::SerializationIssue(data) => value = *data,
      TofResponse::ZMQProblem(data)         => value = *data,
      TofResponse::TimeOut(data)            => value = *data,
      TofResponse::NotImplemented(data)     => value = *data,
      TofResponse::AccessDenied(data)       => value = *data,
      TofResponse::Unknown => ()
    }
    bytestream.extend_from_slice(&value.to_le_bytes());
    bytestream.extend_from_slice(&TofResponse::TAIL.to_le_bytes());
    bytestream
  }

  fn from_bytestream(stream    : &Vec<u8>, 
                     pos       : &mut usize) 
    -> Result<TofResponse, SerializationError>{
    Self::verify_fixed(stream, pos)?;  
    let cc       = parse_u8(stream, pos);
    let value    = parse_u32(stream, pos);
    let pair     = (cc, value);
    let response = TofResponse::from(pair);
    *pos += 2; // acccount for TAIL
    Ok(response)
  }
}

impl From<TofResponse> for u8 {
  fn from(input : TofResponse) -> u8 {
    match input {
      TofResponse::Success(_)            => 1,
      TofResponse::GeneralFail(_)        => 2,
      TofResponse::EventNotReady(_)      => 3,
      TofResponse::SerializationIssue(_) => 4,
      TofResponse::ZMQProblem(_)         => 5,
      TofResponse::TimeOut(_)            => 6,
      TofResponse::NotImplemented(_)     => 7,
      TofResponse::AccessDenied(_)       => 8,
      TofResponse::Unknown => 0
    }
  }
}

impl From<(u8, u32)> for TofResponse {
  fn from(pair : (u8, u32)) -> TofResponse {
    let (input, value) = pair;
    match input {
      1 => TofResponse::Success(value),
      2 => TofResponse::GeneralFail(value),
      3 => TofResponse::EventNotReady(value),
      4 => TofResponse::SerializationIssue(value),
      5 => TofResponse::ZMQProblem(value),
      6 => TofResponse::TimeOut(value),
      7 => TofResponse::NotImplemented(value),
      8 => TofResponse::AccessDenied(value),
      _ => TofResponse::Unknown
    }
  }
}

#[cfg(feature = "random")]
#[test]
fn pack_tofresponse() {
  let resp = TofResponse::from_random();
  let test : TofResponse = resp.pack().unpack().unwrap();
  assert_eq!(resp, test);
}

