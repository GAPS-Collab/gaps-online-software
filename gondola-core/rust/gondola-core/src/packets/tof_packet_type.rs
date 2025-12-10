//! Collection of all possible items that can be stored in a TofPacket
// This file is part of gaps-online-software and published 
// under the GPLv3 license

use crate::prelude::*;

/// Types of serializable data structures used
/// throughout the TOF system. 
#[derive(Debug, Hash, Eq, PartialEq, Clone, Copy, FromRepr, AsRefStr, EnumIter)]
#[cfg_attr(feature = "pybindings", pyclass(eq, eq_int))]
#[repr(u8)]
pub enum TofPacketType {
  Unknown               = 0u8, 
  RBEvent               = 20u8,
  // v0.11 TofEvent -> TofEventDeprecated
  TofEventDeprecated    = 21u8,
  RBWaveform            = 22u8,
  // v0.11 TofEventSummary -> TofEvent
  TofEvent              = 23u8,
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
  LiftofSettings        = 205u8,
  LiftofSettingsDiff    = 206u8,
  RBCalibrationFlightV  = 210u8,
  RBCalibrationFlightT  = 211u8,
  /// A klude which allows us to send bfsw ack packets
  /// through the TOF system
  BfswAckPacket         = 212u8,
  /// Send out a panic message before death 
  PanicPacket           = 222u8,
  /// a MultiPacket consists of other TofPackets
  MultiPacket           = 255u8,
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

expand_and_test_enum!(TofPacketType, test_tofpackettype_repr);


