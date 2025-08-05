//! The following file is part of gaps-online-software and published 
//! under the GPLv3 license

use std::fmt;

#[cfg(feature="random")]
use crate::random::FromRandom;
#[cfg(feature="random")]
use rand::Rng;

#[cfg(feature="pybindings")]
use pyo3::prelude::*;

/// Types of serializable data structures used
/// throughout the TOF system. 
#[derive(Debug, Hash, Eq, PartialEq, Clone, Copy, serde::Deserialize, serde::Serialize)]
#[cfg_attr(feature = "pybindings", pyclass(eq, eq_int))]
#[repr(u8)]
pub enum TofPacketType {
  Unknown               = 0u8, 
  RBEvent               = 20u8,
  TofEvent              = 21u8,
  RBWaveform            = 22u8,
  TofEventSummary       = 23u8,
  DataSinkHB            = 40u8,    
  MasterTrigger         = 60u8,    // needs to be renamed to either MasterTriggerEvent or MTEvent
  TriggerConfig         = 61u8,
  MasterTriggerHB       = 62u8, 
  EventBuilderHB        = 63u8,
  RBChannelMaskConfig   = 64u8,
  TofRBConfig           = 68u8,
  AnalysisEngineConfig  = 69u8,
  RBEventHeader         = 70u8,    // needs to go away
  TOFEventBuilderConfig = 71u8,
  DataPublisherConfig   = 72u8,
  TofRunConfig          = 73u8,
  CPUMoniData           = 80u8,
  MtbMoniData            = 90u8,
  RBMoniData            = 100u8,
  PBMoniData            = 101u8,
  LTBMoniData           = 102u8,
  PAMoniData            = 103u8,
  RBEventMemoryView     = 120u8, // We'll keep it for now - indicates that the event
                                 // still needs to be processed.
  RBCalibration         = 130u8,
  TofCommand            = 140u8,
  TofCommandV2          = 141u8,
  TofResponse           = 142u8,
  // needs to go away
  RBCommand             = 150u8,
  // > 160 configuration packets
  RBPing                = 160u8,
  PreampBiasConfig      = 161u8,
  RunConfig             = 162u8,
  LTBThresholdConfig    = 163u8,
  // avoid 170 since it is our 
  // delimiter
  // >= 171 detector status
  TofDetectorStatus     = 171u8,
  // use the > 200 values for transmitting
  // various binary files
  ConfigBinary          = 201u8,
  LiftofRBBinary        = 202u8,
  LiftofBinaryService   = 203u8,
  LiftofCCBinary        = 204u8,
  RBCalibrationFlightV  = 210u8,
  RBCalibrationFlightT  = 211u8,
  /// A klude which allows us to send bfsw ack packets
  /// through the TOF system
  BfswAckPacket         = 212u8,
  /// a MultiPacket consists of other TofPackets
  MultiPacket           = 255u8,
}

impl fmt::Display for TofPacketType {
  fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
    let r = serde_json::to_string(self).unwrap_or(
      String::from("Error - Don't understand packet type!"));
    write!(f, "<TofPacketType: {}>", r)
  }
}

impl From<u8> for TofPacketType {
  fn from(value: u8) -> Self {
    match value {
      0   => TofPacketType::Unknown,
      20  => TofPacketType::RBEvent,
      21  => TofPacketType::TofEvent,
      22  => TofPacketType::RBWaveform,
      23  => TofPacketType::TofEventSummary,
      40  => TofPacketType::DataSinkHB,
      60  => TofPacketType::MasterTrigger,
      61  => TofPacketType::TriggerConfig,
      62  => TofPacketType::MasterTriggerHB,
      63  => TofPacketType::EventBuilderHB,
      64  => TofPacketType::RBChannelMaskConfig,
      68  => TofPacketType::TofRBConfig,
      69  => TofPacketType::AnalysisEngineConfig,
      70  => TofPacketType::RBEventHeader,
      72  => TofPacketType::DataPublisherConfig,
      73  => TofPacketType::TofRunConfig,
      80  => TofPacketType::CPUMoniData,
      90  => TofPacketType::MtbMoniData,
      100 => TofPacketType::RBMoniData,
      101 => TofPacketType::PBMoniData   ,
      102 => TofPacketType::LTBMoniData  ,
      103 => TofPacketType::PAMoniData   ,
      120 => TofPacketType::RBEventMemoryView,
      130 => TofPacketType::RBCalibration,
      140 => TofPacketType::TofCommand,
      141 => TofPacketType::TofCommandV2,
      142 => TofPacketType::TofResponse,
      150 => TofPacketType::RBCommand,
      160 => TofPacketType::RBPing,
      161 => TofPacketType::PreampBiasConfig,
      162 => TofPacketType::RunConfig,
      163 => TofPacketType::LTBThresholdConfig,
      171 => TofPacketType::TofDetectorStatus,
      201 => TofPacketType::ConfigBinary,
      202 => TofPacketType::LiftofRBBinary,
      203 => TofPacketType::LiftofBinaryService,
      204 => TofPacketType::LiftofCCBinary,
      210 => TofPacketType::RBCalibrationFlightV,
      211 => TofPacketType::RBCalibrationFlightT,
      212 => TofPacketType::BfswAckPacket,
      255 => TofPacketType::MultiPacket,
      _   => TofPacketType::Unknown
    }
  }
}

// in case we have pybindings for this type, 
// expand it so that it can be used as keys
// in dictionaries
#[cfg(feature = "pybindings")]
#[pymethods]
impl TofPacketType {

  #[getter]
  fn __hash__(&self) -> usize {
    (*self as u8) as usize
  } 
}


#[cfg(feature = "random")]
impl FromRandom for TofPacketType { 
  fn from_random() -> Self {
    let choices = [
      TofPacketType::Unknown,
      TofPacketType::TofEvent,
      TofPacketType::RBWaveform,
      TofPacketType::TofEventSummary,
      TofPacketType::MasterTrigger,
      TofPacketType::TriggerConfig, 
      TofPacketType::DataSinkHB,
      TofPacketType::MasterTriggerHB,
      TofPacketType::EventBuilderHB,
      TofPacketType::RBEventHeader,
      TofPacketType::RBEvent,
      TofPacketType::RBEventMemoryView,
      TofPacketType::TofCommand,
      TofPacketType::TofCommandV2,
      TofPacketType::TofResponse,
      TofPacketType::TofRBConfig,
      TofPacketType::RBChannelMaskConfig,
      TofPacketType::DataPublisherConfig,
      TofPacketType::TofRunConfig,
      TofPacketType::AnalysisEngineConfig,
      TofPacketType::RBCommand,
      TofPacketType::RBPing,
      TofPacketType::PreampBiasConfig,
      TofPacketType::RunConfig,
      TofPacketType::LTBThresholdConfig,
      TofPacketType::RBMoniData,
      TofPacketType::PBMoniData,
      TofPacketType::LTBMoniData,
      TofPacketType::PAMoniData,
      TofPacketType::CPUMoniData,
      TofPacketType::MtbMoniData,
      TofPacketType::RBCalibration,
      TofPacketType::TofDetectorStatus,
      TofPacketType::ConfigBinary,
      TofPacketType::LiftofRBBinary,
      TofPacketType::LiftofBinaryService,
      TofPacketType::LiftofCCBinary,
      TofPacketType::BfswAckPacket,
      TofPacketType::RBCalibrationFlightV,
      TofPacketType::RBCalibrationFlightT,
    ];
    let mut rng  = rand::thread_rng();
    let idx = rng.random_range(0..choices.len());
    choices[idx]
  }
}

#[test]
fn test_packet_types() {
  let mut type_codes = Vec::<u8>::new();
  #[cfg(feature = "random")]
  for _ in 0..100 {
    type_codes.push(TofPacketType::from_random() as u8);
  }
  for tc in type_codes.iter() {
    assert_eq!(*tc,TofPacketType::try_from(*tc).unwrap() as u8);  
  }
}


