//! The following file is part of gaps-online-software and published 
//! under the GPLv3 license

use std::fmt;

#[cfg(feature = "pybindings")]
use pyo3::{
  pyclass,
  pymethods
};


#[cfg(feature = "random")]
use crate::random::FromRandom;
#[cfg(feature = "random")]
use rand::Rng;

/// Describe the contents of a byte sequence ("packet") typicially 
/// crafted for telemetry. The numbers are defined by the 'bfsw'
/// software package.
#[derive(Debug, Copy, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
#[cfg_attr(feature = "pybindings", pyclass(eq, eq_int))]
#[repr(u8)]
pub enum TelemetryPacketType {
  Unknown            = 0,
  CardHKP            = 30,
  CoolingHK          = 40,
  PDUHK              = 50,
  Tracker            = 80,
  TrackerDAQCntr     = 81,
  GPS                = 82,
  TrkTempLeak        = 83,
  BoringEvent        = 90,
  RBWaveform         = 91,
  AnyTofHK           = 92,
  GcuEvtBldSettings  = 93,
  LabJackHK          = 100,
  MagHK              = 108,
  GcuMon             = 110,
  InterestingEvent   = 190,
  NoGapsTriggerEvent = 191,
  NoTofDataEvent     = 192,
  Ack                = 200,     
  AnyTrackerHK       = 255,
  // unknown/unused stuff
  TmP33              = 33,
  TmP34              = 34,
  TmP37              = 37,
  TmP38              = 38,
  TmP55              = 55,
  TmP64              = 64,
  TmP96              = 96,
  TmP214             = 214,
}

impl From<u8> for TelemetryPacketType {
  fn from(value: u8) -> Self {
    match value {
      0     => TelemetryPacketType::Unknown,
      30    => TelemetryPacketType::CardHKP,
      40    => TelemetryPacketType::CoolingHK,
      50    => TelemetryPacketType::PDUHK,
      80    => TelemetryPacketType::Tracker,
      81    => TelemetryPacketType::TrackerDAQCntr,
      82    => TelemetryPacketType::GPS,
      83    => TelemetryPacketType::TrkTempLeak,
      90    => TelemetryPacketType::BoringEvent,
      91    => TelemetryPacketType::RBWaveform,
      92    => TelemetryPacketType::AnyTofHK,
      93    => TelemetryPacketType::GcuEvtBldSettings,
      100   => TelemetryPacketType::LabJackHK,
      108   => TelemetryPacketType::MagHK,
      110   => TelemetryPacketType::GcuMon,
      190   => TelemetryPacketType::InterestingEvent,
      191   => TelemetryPacketType::NoGapsTriggerEvent,
      192   => TelemetryPacketType::NoTofDataEvent,
      200   => TelemetryPacketType::Ack,
      255   => TelemetryPacketType::AnyTrackerHK,
       33   => TelemetryPacketType::TmP33,
       34   => TelemetryPacketType::TmP34,
       37   => TelemetryPacketType::TmP37,
       38   => TelemetryPacketType::TmP38,
       55   => TelemetryPacketType::TmP55,
       64   => TelemetryPacketType::TmP64,
       96   => TelemetryPacketType::TmP96,
       214  => TelemetryPacketType::TmP214,
      _     => TelemetryPacketType::Unknown,
    }
  }
}

impl fmt::Display for TelemetryPacketType {
  fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
    let r = serde_json::to_string(self).unwrap_or(
      String::from("Error - Don't understand packet type!"));
    write!(f, "<TelemetryPacketType: {}>", r)
  }
}

#[cfg(feature = "random")]
impl FromRandom for TelemetryPacketType {  
  fn from_random() -> Self {
    let choices = [
      TelemetryPacketType::Unknown           ,
      TelemetryPacketType::CardHKP           ,
      TelemetryPacketType::CoolingHK         ,
      TelemetryPacketType::PDUHK             ,
      TelemetryPacketType::Tracker           ,
      TelemetryPacketType::TrackerDAQCntr    ,
      TelemetryPacketType::GPS               ,
      TelemetryPacketType::TrkTempLeak       ,
      TelemetryPacketType::BoringEvent       ,
      TelemetryPacketType::RBWaveform        ,
      TelemetryPacketType::AnyTofHK          ,
      TelemetryPacketType::GcuEvtBldSettings ,
      TelemetryPacketType::LabJackHK         ,
      TelemetryPacketType::MagHK             ,
      TelemetryPacketType::GcuMon            ,
      TelemetryPacketType::InterestingEvent  ,
      TelemetryPacketType::NoGapsTriggerEvent,
      TelemetryPacketType::NoTofDataEvent    ,
      TelemetryPacketType::Ack               ,     
      TelemetryPacketType::AnyTrackerHK      ,
      TelemetryPacketType::TmP33             ,
      TelemetryPacketType::TmP34             ,
      TelemetryPacketType::TmP37             ,
      TelemetryPacketType::TmP38             ,
      TelemetryPacketType::TmP55             ,
      TelemetryPacketType::TmP64             ,
      TelemetryPacketType::TmP96             ,
      TelemetryPacketType::TmP214            
    ];
    let mut rng  = rand::rng();
    let idx = rng.random_range(0..choices.len());
    choices[idx]
  }
}

//--------------------------------------------------------------

// in case we have pybindings for this type, 
// expand it so that it can be used as keys
// in dictionaries
#[cfg(feature = "pybindings")]
#[pymethods]
impl TelemetryPacketType {

  #[getter]
  fn __hash__(&self) -> usize {
    (*self as u8) as usize
  } 
}

//--------------------------------------------------------------

#[test]
fn telemetrypackettype_from_to_u8() {
  let mut type_codes = Vec::<u8>::new();
  #[cfg(feature = "random")]
  for _ in 0..100 {
    type_codes.push(TelemetryPacketType::from_random() as u8);
  }
  type_codes.push(TelemetryPacketType::Unknown            as u8);
  type_codes.push(TelemetryPacketType::CardHKP            as u8);
  type_codes.push(TelemetryPacketType::CoolingHK          as u8);
  type_codes.push(TelemetryPacketType::PDUHK              as u8);
  type_codes.push(TelemetryPacketType::Tracker            as u8);
  type_codes.push(TelemetryPacketType::TrackerDAQCntr     as u8);
  type_codes.push(TelemetryPacketType::GPS                as u8);
  type_codes.push(TelemetryPacketType::TrkTempLeak        as u8);
  type_codes.push(TelemetryPacketType::BoringEvent        as u8);
  type_codes.push(TelemetryPacketType::RBWaveform         as u8);
  type_codes.push(TelemetryPacketType::AnyTofHK           as u8);
  type_codes.push(TelemetryPacketType::GcuEvtBldSettings  as u8);
  type_codes.push(TelemetryPacketType::LabJackHK          as u8);
  type_codes.push(TelemetryPacketType::MagHK              as u8);
  type_codes.push(TelemetryPacketType::GcuMon             as u8);
  type_codes.push(TelemetryPacketType::InterestingEvent   as u8);
  type_codes.push(TelemetryPacketType::NoGapsTriggerEvent as u8);
  type_codes.push(TelemetryPacketType::NoTofDataEvent     as u8);
  type_codes.push(TelemetryPacketType::Ack                as u8);     
  type_codes.push(TelemetryPacketType::AnyTrackerHK       as u8);
  type_codes.push(TelemetryPacketType::TmP33              as u8);
  type_codes.push(TelemetryPacketType::TmP34              as u8);
  type_codes.push(TelemetryPacketType::TmP37              as u8);
  type_codes.push(TelemetryPacketType::TmP38              as u8);
  type_codes.push(TelemetryPacketType::TmP55              as u8);
  type_codes.push(TelemetryPacketType::TmP64              as u8);
  type_codes.push(TelemetryPacketType::TmP96              as u8);
  type_codes.push(TelemetryPacketType::TmP214             as u8);
  for tc in type_codes.iter() {
    assert_eq!(*tc,TelemetryPacketType::from(*tc) as u8);  
  }
}
