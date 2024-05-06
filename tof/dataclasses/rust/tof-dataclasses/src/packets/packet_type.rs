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
  Unknown             = 0u8, 
  RBEvent             = 20u8,
  TofEvent            = 21u8,
  RBWaveform          = 22u8,
  TofEventSummary     = 23u8,
  //Monitor             = 30u8,    // needs to go away
  HeartBeat           = 40u8,    // might probably go away
  MasterTrigger       = 60u8,    // needs to be renamed to either MasterTriggerEvent or MTEvent
  RBEventHeader       = 70u8,    // needs to go away
  CPUMoniData         = 80u8,
  MonitorMtb          = 90u8,
  RBMoniData          = 100u8,
  PBMoniData          = 101u8,
  LTBMoniData         = 102u8,
  PAMoniData          = 103u8,
  RBEventMemoryView   = 120u8, // We'll keep it for now - indicates that the event
                               // still needs to be processed.
  RBCalibration       = 130u8,
  TofCommand          = 140u8,
  TofResponse         = 142u8,
  RBCommand           = 150u8,
  Ping                = 160u8,
  // use the > 200 values for transmitting
  // various binary files
  ConfigBinary        = 201u8,
  LiftofRBBinary      = 202u8,
  LiftofBinaryService = 203u8,
  LiftofCCBinary      = 204u8,

  /// a MultiPacket consists of other TofPackets
  MultiPacket         = 255u8,
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
      0u8   => PacketType::Unknown,
      20u8  => PacketType::RBEvent,
      21u8  => PacketType::TofEvent,
      22u8  => PacketType::RBWaveform,
      23u8  => PacketType::TofEventSummary,
      //30u8  => PacketType::Monitor,
      40u8  => PacketType::HeartBeat,
      60u8  => PacketType::MasterTrigger,
      70u8  => PacketType::RBEventHeader,
      80u8  => PacketType::CPUMoniData,
      90u8  => PacketType::MonitorMtb,
      100u8 => PacketType::RBMoniData,
      101u8 => PacketType::PBMoniData   ,
      102u8 => PacketType::LTBMoniData  ,
      103u8 => PacketType::PAMoniData   ,
      120u8 => PacketType::RBEventMemoryView,
      130u8 => PacketType::RBCalibration,
      140u8 => PacketType::TofCommand,
      142u8 => PacketType::TofResponse,
      150u8 => PacketType::RBCommand,
      160u8 => PacketType::Ping,
      201u8 => PacketType::ConfigBinary,
      202u8 => PacketType::LiftofRBBinary,
      203u8 => PacketType::LiftofBinaryService,
      204u8 => PacketType::LiftofCCBinary,
      255u8 => PacketType::MultiPacket,
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
      //PacketType::Monitor,
      PacketType::MasterTrigger,
      PacketType::HeartBeat,
      PacketType::RBEventHeader,
      PacketType::RBEvent,
      PacketType::RBEventMemoryView,
      PacketType::TofCommand,
      PacketType::TofResponse,
      PacketType::RBCommand,
      PacketType::RBMoniData,
      PacketType::PBMoniData,
      PacketType::LTBMoniData,
      PacketType::PAMoniData,
      PacketType::CPUMoniData,
      PacketType::MonitorMtb,
      PacketType::RBCalibration,
      PacketType::ConfigBinary,
      PacketType::LiftofRBBinary,
      PacketType::LiftofBinaryService,
      PacketType::LiftofCCBinary,
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
  //type_codes.push(PacketType::Monitor as u8);
  type_codes.push(PacketType::MasterTrigger as u8);
  type_codes.push(PacketType::HeartBeat as u8);
  type_codes.push(PacketType::RBEventHeader as u8);
  type_codes.push(PacketType::RBEvent as u8);
  type_codes.push(PacketType::RBEventMemoryView as u8);
  type_codes.push(PacketType::TofCommand as u8);
  type_codes.push(PacketType::TofResponse as u8);
  type_codes.push(PacketType::RBCommand as u8);
  type_codes.push(PacketType::RBMoniData as u8);
  type_codes.push(PacketType::PBMoniData as u8);
  type_codes.push(PacketType::LTBMoniData as u8);
  type_codes.push(PacketType::PAMoniData as u8);
  type_codes.push(PacketType::CPUMoniData as u8);
  type_codes.push(PacketType::MonitorMtb as u8);
  type_codes.push(PacketType::RBCalibration as u8);
  type_codes.push(PacketType::ConfigBinary as u8);
  type_codes.push(PacketType::LiftofCCBinary as u8);
  type_codes.push(PacketType::LiftofRBBinary as u8);
  type_codes.push(PacketType::LiftofBinaryService as u8);
  for tc in type_codes.iter() {
    assert_eq!(*tc,PacketType::try_from(*tc).unwrap() as u8);  
  }
}

