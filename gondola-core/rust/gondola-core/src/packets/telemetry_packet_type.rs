//! Everything which can be stored within a telemetry packet
// The following file is part of gaps-online-software and published 
// under the GPLv3 license

use crate::prelude::*;

/// Describe the contents of a byte sequence ("packet") typicially 
/// crafted for telemetry. The numbers are defined by the 'bfsw'
/// software package.
#[derive(Debug, Hash, Eq, PartialEq, Clone, Copy, FromRepr, AsRefStr, EnumIter)]
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
  RPiHKP             = 89,
  BoringEvent        = 90,
  RBWaveform         = 91,
  AnyTofHK           = 92,
  GcuEvtBldSettings  = 93,
  GcuEvtBuilderStats = 94,
  TmP96              = 96,
  LabJackHK          = 100,
  LabjackSettings    = 101,
  HeatHVLVSettings   = 102,
  SurvivalPacket     = 114,
  GcuMonHKAddendum   = 120,
  PacketStats        = 111,
  TeleMainSettings   = 112,
  DecimationSettings = 113,
  MagHK              = 108,
  GcuMon             = 110,
  InterestingEvent   = 190,
  NoGapsTriggerEvent = 191,
  NoTofDataEvent     = 192,
  Ack                = 200,     
  RatePacket         = 219,
  AnyTrackerHK       = 255,
  // unknown/unused stuff
  TmP33              = 33,
  TmP34              = 34,
  TmP37              = 37,
  TmP38              = 38,
  TmP55              = 55,
  TmP64              = 64,
  TmP214             = 214,
}

expand_and_test_enum!(TelemetryPacketType, test_telemetrypackettype_repr);

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

