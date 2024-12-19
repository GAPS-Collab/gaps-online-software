/// PacketType identifies the payload in TofPackets
///
/// This needs to be kept in sync with the C++ API
use std::fmt;

cfg_if::cfg_if! {
  if #[cfg(feature = "random")]  {
    use crate::FromRandom;
    extern crate rand;
    use rand::Rng;
  }
}

#[cfg(feature = "pybindings")]
use pyo3::pyclass;

/// Types of serializable data structures used
/// throughout the tof system
#[derive(Debug, PartialEq, Clone, Copy, serde::Deserialize, serde::Serialize)]
#[cfg_attr(feature = "pybindings", pyclass)]
#[repr(u8)]
pub enum PacketType {
  Unknown               = 0u8, 
  RBEvent               = 20u8,
  TofEvent              = 21u8,
  RBWaveform            = 22u8,
  TofEventSummary       = 23u8,
  HeartBeatDataSink     = 40u8,    
  MasterTrigger         = 60u8,    // needs to be renamed to either MasterTriggerEvent or MTEvent
  TriggerConfig         = 61u8,
  MTBHeartbeat          = 62u8, 
  EVTBLDRHeartbeat      = 63u8,
  RBChannelMaskConfig   = 64u8,
  TofRBConfig           = 68u8,
  AnalysisEngineConfig  = 69u8,
  RBEventHeader         = 70u8,    // needs to go away
  TOFEventBuilderConfig = 71u8,
  DataPublisherConfig   = 72u8,
  TofRunConfig          = 73u8,
  CPUMoniData           = 80u8,
  MonitorMtb            = 90u8,
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
  /// A klude which allows us to send bfsw ack packets
  /// through the TOF system
  BfswAckPacket         = 205u8,
  /// a MultiPacket consists of other TofPackets
  MultiPacket           = 255u8,
}

impl fmt::Display for PacketType {
  fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
    let r = serde_json::to_string(self).unwrap_or(
      String::from("Error - Don't understand packet type!"));
    write!(f, "<PacketType: {}>", r)
  }
}

impl From<u8> for PacketType {
  fn from(value: u8) -> Self {
    match value {
      0   => PacketType::Unknown,
      20  => PacketType::RBEvent,
      21  => PacketType::TofEvent,
      22  => PacketType::RBWaveform,
      23  => PacketType::TofEventSummary,
      40  => PacketType::HeartBeatDataSink,
      60  => PacketType::MasterTrigger,
      61  => PacketType::TriggerConfig,
      62  => PacketType::MTBHeartbeat,
      63  => PacketType::EVTBLDRHeartbeat,
      64  => PacketType::RBChannelMaskConfig,
      68  => PacketType::TofRBConfig,
      69  => PacketType::AnalysisEngineConfig,
      70  => PacketType::RBEventHeader,
      72  => PacketType::DataPublisherConfig,
      73  => PacketType::TofRunConfig,
      80  => PacketType::CPUMoniData,
      90  => PacketType::MonitorMtb,
      100 => PacketType::RBMoniData,
      101 => PacketType::PBMoniData   ,
      102 => PacketType::LTBMoniData  ,
      103 => PacketType::PAMoniData   ,
      120 => PacketType::RBEventMemoryView,
      130 => PacketType::RBCalibration,
      140 => PacketType::TofCommand,
      141 => PacketType::TofCommandV2,
      142 => PacketType::TofResponse,
      150 => PacketType::RBCommand,
      160 => PacketType::RBPing,
      161 => PacketType::PreampBiasConfig,
      162 => PacketType::RunConfig,
      163 => PacketType::LTBThresholdConfig,
      171 => PacketType::TofDetectorStatus,
      201 => PacketType::ConfigBinary,
      202 => PacketType::LiftofRBBinary,
      203 => PacketType::LiftofBinaryService,
      204 => PacketType::LiftofCCBinary,
      205 => PacketType::BfswAckPacket,
      255 => PacketType::MultiPacket,
      _     => PacketType::Unknown
    }
  }
}

#[cfg(feature = "random")]
impl FromRandom for PacketType {
  
  fn from_random() -> Self {
    let choices = [
      PacketType::Unknown,
      PacketType::TofEvent,
      PacketType::RBWaveform,
      PacketType::TofEventSummary,
      PacketType::MasterTrigger,
      PacketType::TriggerConfig, 
      PacketType::HeartBeatDataSink,
      PacketType::MTBHeartbeat,
      PacketType::EVTBLDRHeartbeat,
      PacketType::RBEventHeader,
      PacketType::RBEvent,
      PacketType::RBEventMemoryView,
      PacketType::TofCommand,
      PacketType::TofCommandV2,
      PacketType::TofResponse,
      PacketType::TofRBConfig,
      PacketType::RBChannelMaskConfig,
      PacketType::DataPublisherConfig,
      PacketType::TofRunConfig,
      PacketType::AnalysisEngineConfig,
      PacketType::RBCommand,
      PacketType::RBPing,
      PacketType::PreampBiasConfig,
      PacketType::RunConfig,
      PacketType::LTBThresholdConfig,
      PacketType::RBMoniData,
      PacketType::PBMoniData,
      PacketType::LTBMoniData,
      PacketType::PAMoniData,
      PacketType::CPUMoniData,
      PacketType::MonitorMtb,
      PacketType::RBCalibration,
      PacketType::TofDetectorStatus,
      PacketType::ConfigBinary,
      PacketType::LiftofRBBinary,
      PacketType::LiftofBinaryService,
      PacketType::LiftofCCBinary,
      PacketType::BfswAckPacket
    ];
    let mut rng  = rand::thread_rng();
    let idx = rng.gen_range(0..choices.len());
    choices[idx]
  }
}

#[test]
fn test_packet_types() {
  let mut type_codes = Vec::<u8>::new();
  type_codes.push(PacketType::Unknown as u8);
  type_codes.push(PacketType::TofEvent as u8);
  type_codes.push(PacketType::RBWaveform as u8);
  type_codes.push(PacketType::TofEventSummary as u8);
  type_codes.push(PacketType::TriggerConfig as u8);
  type_codes.push(PacketType::MasterTrigger as u8);
  type_codes.push(PacketType::HeartBeatDataSink as u8);
  type_codes.push(PacketType::MTBHeartbeat as u8);
  type_codes.push(PacketType::EVTBLDRHeartbeat as u8);
  type_codes.push(PacketType::RBEventHeader as u8);
  type_codes.push(PacketType::RBEvent as u8);
  type_codes.push(PacketType::TofRBConfig as u8);
  type_codes.push(PacketType::RBEventMemoryView as u8);
  type_codes.push(PacketType::TofCommand as u8);
  type_codes.push(PacketType::TofCommandV2 as u8);
  type_codes.push(PacketType::TofResponse as u8);
  type_codes.push(PacketType::RBCommand as u8);
  type_codes.push(PacketType::RBMoniData as u8);
  type_codes.push(PacketType::PBMoniData as u8);
  type_codes.push(PacketType::LTBMoniData as u8);
  type_codes.push(PacketType::PAMoniData as u8);
  type_codes.push(PacketType::CPUMoniData as u8);
  type_codes.push(PacketType::RBPing as u8);
  type_codes.push(PacketType::PreampBiasConfig as u8);
  type_codes.push(PacketType::RunConfig as u8);
  type_codes.push(PacketType::TofRunConfig as u8);
  type_codes.push(PacketType::LTBThresholdConfig as u8);
  type_codes.push(PacketType::AnalysisEngineConfig as u8);
  type_codes.push(PacketType::DataPublisherConfig as u8);
  type_codes.push(PacketType::RBChannelMaskConfig as u8);
  type_codes.push(PacketType::MonitorMtb as u8);
  type_codes.push(PacketType::RBCalibration as u8);
  type_codes.push(PacketType::TofDetectorStatus as u8);
  type_codes.push(PacketType::ConfigBinary as u8);
  type_codes.push(PacketType::LiftofCCBinary as u8);
  type_codes.push(PacketType::LiftofRBBinary as u8);
  type_codes.push(PacketType::LiftofBinaryService as u8);
  type_codes.push(PacketType::BfswAckPacket as u8);
  for tc in type_codes.iter() {
    assert_eq!(*tc,PacketType::try_from(*tc).unwrap() as u8);  
  }
}

