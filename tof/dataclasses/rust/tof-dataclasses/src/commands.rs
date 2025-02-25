//! Commmands which can be issued
//! to the various components of 
//! the tof system.
//!
//!
//!

pub mod factory;
pub mod config;

use std::fmt;

#[cfg(feature = "pybindings")]
use pyo3::pyclass;

use crate::serialization::{
    Serialization,
    Packable,
    SerializationError,
    parse_u8,
    parse_u16,
    parse_u32
};

use crate::packets::{
    TofPacket,
    PacketType
};

cfg_if::cfg_if! {
  if #[cfg(feature = "random")]  {
    use crate::FromRandom;
    extern crate rand;
    use rand::Rng;
  }
}

pub use factory::*;

#[derive(Debug, Copy, Clone, PartialEq, serde::Deserialize, serde::Serialize)]
#[cfg_attr(feature = "pybindings", pyclass(eq, eq_int))]
#[repr(u8)]
pub enum TofReturnCode {
  Unknown = 0,
  GeneralFail = 1,
  GarbledCommand = 2,
  Success = 200,
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
    let mut rng  = rand::thread_rng();
    let idx = rng.gen_range(0..choices.len());
    choices[idx]
  }
}

#[derive(Debug, Copy, Clone, PartialEq, serde::Deserialize, serde::Serialize)]
#[cfg_attr(feature = "pybindings", pyclass(eq, eq_int))]
#[repr(u8)]
pub enum TofCommandCode {
  Unknown                  = 0u8,
  /// en empty command just to check if stuff is online
  Ping                     = 1u8,
  /// command code for getting the monitoring data from the component
  Moni                     = 2u8,
  /// Kill myself
  Kill                     = 4u8, // Shi!
  /// Reload a default (to be defined) config file
  ResetConfigWDefault      = 5u8,
  /// Make the current editable config the active config
  SubmitConfig           = 6u8,
  /// command code to configure the data publisher thread
  SetDataPublisherConfig   = 20u8,
  /// command code for "Set LTB Thresholds"
  SetLTBThresholds         = 21u8,         
  /// command code for "Configure MTB"
  SetMTConfig              = 22u8,     
  /// command code for chaning general run parameters
  SetTofRunConfig          = 23u8,
  /// command code for changing RB parameters
  SetTofRBConfig           = 24u8,
  /// command code for AnalysisEngineConfig
  SetAnalysisEngineConfig  = 27u8,   
  /// command code for "Set preamp bias"
  SetPreampBias            = 28u8,         
  /// Change the settings of the event builder
  SetTOFEventBuilderConfig = 29u8,
  /// command code for "Stop Data taking"
  DataRunStop              = 30u8,  
  /// command code for "Start Data taking"
  DataRunStart             = 31u8,    
  /// command code for "Start validation run"
  StartValidationRun       = 32u8,         
  /// command code for "Get all waveforms"
  GetFullWaveforms         = 41u8,
  /// command code for "Send the whole event cache over the wire"
  UnspoolEventCache        = 44u8,
  /// command code for "Run full calibration"
  RBCalibration            = 53u8, 
  /// command code for restarting systemd
  RestartLiftofRBClients  = 60u8,
  /// command code for putting liftof-cc in listening mode
  Listen                  = 70u8,
  /// command code for putting liftof-cc in staging mode
  Staging                 = 71u8,
  /// lock the cmd dispatcher
  Lock                    = 80u8,
  /// unlock the cmd dispatcher
  Unlock                  = 81u8,
  /// Enable sending of TOF packets
  SendTofEvents           = 90u8,
  /// Diesable sending of TofEventPacket
  NoSendTofEvents         = 91u8,
  /// Enable sending of RBWaveform packets
  SendRBWaveforms         = 92u8,
  /// Disable sending of RBWaveform packets
  NoSendRBWaveforms       = 93u8,
  /// Enable RB Channel Masks
  SetRBChannelMask        = 99u8,
  /// Shutdown RB - send shutdown now to RB
  ShutdownRB              = 100u8,
  /// Change the config file for the next run
  ChangeNextRunConfig     = 101u8,
  /// Shutdown RAT - send shutdown command to 2RBs in the same RAT
  ShutdownRAT             = 102u8,
  /// Shutdown a pair of RATs (as always two of them are hooked up to the 
  /// same PDU channel)
  ShutdownRATPair         = 103u8,
  /// Shutdown the TOF CPU
  ShutdownCPU             = 104u8,
  /// Upload a new config file
  UploadConfig            = 105u8,
}

impl fmt::Display for TofCommandCode {
  fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
    let r = serde_json::to_string(self).unwrap_or(
      String::from("Error: cannot unwrap this TofCommandCode"));
    write!(f, "<TofCommandCode: {}>", r)
  }
}

impl From<u8> for TofCommandCode {
  fn from(value: u8) -> Self {
    match value {
      0   => TofCommandCode::Unknown,
      1   => TofCommandCode::Ping,
      2   => TofCommandCode::Moni,
      4   => TofCommandCode::Kill,
      5   => TofCommandCode::ResetConfigWDefault,
      6   => TofCommandCode::SubmitConfig,
      20  => TofCommandCode::SetDataPublisherConfig,
      21  => TofCommandCode::SetLTBThresholds,
      22  => TofCommandCode::SetMTConfig,
      23  => TofCommandCode::SetTofRunConfig,
      24  => TofCommandCode::SetTofRBConfig,
      28  => TofCommandCode::SetPreampBias,
      29  => TofCommandCode::SetTOFEventBuilderConfig,
      30  => TofCommandCode::DataRunStop,
      31  => TofCommandCode::DataRunStart,
      32  => TofCommandCode::StartValidationRun,
      41  => TofCommandCode::GetFullWaveforms,
      53  => TofCommandCode::RBCalibration,
      44  => TofCommandCode::UnspoolEventCache,
      60  => TofCommandCode::RestartLiftofRBClients,
      70  => TofCommandCode::Listen,
      71  => TofCommandCode::Staging,
      80  => TofCommandCode::Lock,
      81  => TofCommandCode::Unlock,
      90  => TofCommandCode::SendTofEvents,
      91  => TofCommandCode::NoSendTofEvents,
      92  => TofCommandCode::SendRBWaveforms,
      93  => TofCommandCode::NoSendRBWaveforms,
      100 => TofCommandCode::ShutdownRB,
      101 => TofCommandCode::ChangeNextRunConfig,
      102 => TofCommandCode::ShutdownRAT,
      103 => TofCommandCode::ShutdownRATPair,
      104 => TofCommandCode::ShutdownCPU,
      105 => TofCommandCode::UploadConfig,
      _   => TofCommandCode::Unknown
    }
  }
}

#[cfg(feature = "random")]
impl FromRandom for TofCommandCode {
  fn from_random() -> Self {
    let choices = [
      TofCommandCode::Unknown,
      TofCommandCode::Ping,
      TofCommandCode::Moni,
      TofCommandCode::ResetConfigWDefault,
      TofCommandCode::SubmitConfig,
      TofCommandCode::SetDataPublisherConfig,
      TofCommandCode::SetLTBThresholds,
      TofCommandCode::SetMTConfig,
      TofCommandCode::SetTofRunConfig,
      TofCommandCode::SetTofRBConfig,
      TofCommandCode::SetTOFEventBuilderConfig,
      TofCommandCode::SetPreampBias,
      TofCommandCode::DataRunStop,
      TofCommandCode::DataRunStart,
      TofCommandCode::StartValidationRun,
      TofCommandCode::GetFullWaveforms,
      TofCommandCode::RBCalibration,
      TofCommandCode::UnspoolEventCache,
      TofCommandCode::RestartLiftofRBClients,
      TofCommandCode::Listen,
      TofCommandCode::Staging,
      TofCommandCode::Lock,
      TofCommandCode::Unlock,
      TofCommandCode::SendTofEvents,
      TofCommandCode::NoSendTofEvents,
      TofCommandCode::SendRBWaveforms,
      TofCommandCode::NoSendRBWaveforms,
      TofCommandCode::Kill,
      TofCommandCode::ShutdownRB,
      TofCommandCode::ChangeNextRunConfig,
      TofCommandCode::ShutdownRAT,
      TofCommandCode::ShutdownRATPair,
      TofCommandCode::ShutdownCPU,
      TofCommandCode::UploadConfig,
    ];
    let mut rng  = rand::thread_rng();
    let idx = rng.gen_range(0..choices.len());
    choices[idx]
  }
}

///////////////////////////////////////////////////////////////////////////////

/// A general command class with an arbitrary payload
///
/// Since the commands should in general be small
/// the maixmal payload size is limited to 256 bytes
///
/// All commands will get broadcasted and the 
/// receiver has to figure out if they have 
/// to rect to that command
#[derive(Debug, Clone, PartialEq)]
pub struct TofCommandV2 {
  pub command_code : TofCommandCode,
  pub payload      : Vec<u8>,
}

impl TofCommandV2 {
 // BFSW command header 144, 235, 86, 248, 70, 41, 7, 15,
 pub fn new() -> Self {
    Self {
      command_code : TofCommandCode::Unknown,
      payload      : Vec::<u8>::new(),
    }
  }

  //pub fn from_config(cfg_file : String) -> Self {
  //  let mut cmd = TofCommandV2::new();
  //  cmd.command_code = TofCommandCode::UploadConfig:

  //}

  /// In case the command is supposed to change the next run configuration
  /// we can create it with this function
  ///
  /// # Arguments
  ///
  ///   * key_value :  a list of keys and a single value (last item of the 
  ///                  list
  pub fn forge_changerunconfig(key_value : &Vec<String>) -> Self {
    let mut cmd = TofCommandV2::new();
    cmd.command_code = TofCommandCode::ChangeNextRunConfig;
    if key_value.len() == 0 {
      error!("Empty command!");
      return cmd;
    }
    let mut payload_string = String::from("");
    for k in 0..key_value.len() - 1 {
      payload_string += &format!("{}::", key_value[k]);
    }
    payload_string += key_value.last().unwrap();
    let mut payload = Vec::<u8>::new();
    payload.extend_from_slice(payload_string.as_bytes());
    cmd.payload = payload;
    cmd
  }

  /// After the command has been unpackt, reconstruct the 
  /// key/value string
  pub fn extract_changerunconfig(&self) -> Option<Vec<String>> {
    if self.command_code != TofCommandCode::ChangeNextRunConfig {
      error!("Unable to extract configuration file changes from {}", self);
      return None;
    }
    let mut liftof_config = Vec::<String>::new();
    match String::from_utf8(self.payload.clone()) {
      Err(err) => {
        error!("Unable to extract the String payload! {err}");
      }
      Ok(concat_string) => {
        let foo = concat_string.split("::").collect::<Vec<&str>>().into_iter();
        for k in foo {
          liftof_config.push(String::from(k));
        }
      }
    }
    Some(liftof_config)
  }
}

impl Packable for TofCommandV2 {
  const PACKET_TYPE : PacketType = PacketType::TofCommandV2;
}

impl Serialization for TofCommandV2 {
  
  const HEAD : u16 = 0xAAAA;
  const TAIL : u16 = 0x5555;

  fn from_bytestream(stream    : &Vec<u8>, 
                     pos       : &mut usize) 
    -> Result<Self, SerializationError>{
    let mut command = TofCommandV2::new();
    if parse_u16(stream, pos) != Self::HEAD {
      error!("The given position {} does not point to a valid header signature of {}", pos, Self::HEAD);
      return Err(SerializationError::HeadInvalid {});
    }
    command.command_code = TofCommandCode::from(parse_u8(stream, pos));
    let payload_size     = parse_u8(stream, pos);
    let payload          = stream[*pos..*pos + payload_size as usize].to_vec();
    command.payload      = payload;
    *pos += payload_size as usize;
    let tail = parse_u16(stream, pos);
    if tail != Self::TAIL {
      error!("After parsing the event, we found an invalid tail signature {}", tail);
      return Err(SerializationError::TailInvalid);
    }
    Ok(command)
  }

  fn to_bytestream(&self) -> Vec<u8> {
    let mut stream = Vec::<u8>::with_capacity(9);
    stream.extend_from_slice(&Self::HEAD.to_le_bytes());
    stream.push(self.command_code as u8);
    stream.push(self.payload.len() as u8);
    stream.extend_from_slice(self.payload.as_slice());
    stream.extend_from_slice(&Self::TAIL.to_le_bytes());
    stream
  }
}

impl Default for TofCommandV2 {
  fn default() -> Self {
    Self::new()
  }
}

#[cfg(feature = "random")]
impl FromRandom for TofCommandV2 {
  fn from_random() -> Self {
    let mut rng      = rand::thread_rng();
    let command_code = TofCommandCode::from_random();
    let payload_size = rng.gen::<u8>();
    let mut payload  = Vec::<u8>::with_capacity(payload_size as usize);
    for _ in 0..payload_size {
      payload.push(rng.gen::<u8>());
    }
    Self {
      command_code : command_code,
      payload      : payload
    }
  }
}

impl fmt::Display for TofCommandV2 {
  fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
    //let cc = RBCommand::command_code_to_string(self.command_code);
    let mut repr = String::from("<TofCommandV2");
    repr += &(format!("\n  cmd code : {}", self.command_code)); 
    match self.command_code {
      TofCommandCode::ShutdownRB 
      | TofCommandCode::ShutdownRAT 
      | TofCommandCode::ShutdownRATPair => {
        repr += &(format!("\n Sending shutdown command to RBs {:?}>", self.payload));
      }
      _ => {
        repr += ">";
      }
    }
    write!(f, "{}", repr)
  }
}

// Specific response codes
// These are long (4 bytes) but 
// this allows to convey more information
// e.g. event id
#[derive(Debug, Copy, Clone, PartialEq, serde::Deserialize, serde::Serialize)]
#[repr(u32)]
pub enum TofResponseCode {
  Unknown                            = 0u32,
  /// response code for: Command can not be executed on the server side
  RespErrUnexecutable                = 500u32,
  RespErrAccessDenied                = 403u32,
  RespErrNotImplemented              = 404u32, 
  /// response code for: Something did not work quite right, 
  /// however, the problem has either fixed itself or it is 
  /// highly likely that if the command is issued again it 
  /// will succeed.
  RespErrLevelNoProblem              = 4000u32, 
  RespErrLevelMedium                 = 4010u32, 
  RespErrLevelSevere                 = 4020u32, 
  /// response code for: A critical condition. This might need a fix somehow and can 
  /// not be fixed automatically. Probably at least a power-cycle is necessary.
  RespErrLevelCritical               = 4030u32, 
  /// response code for: The severest error condition which can occur. This might
  /// still be fixable, but it is probably a good advice to get help. Currently, 
  /// the mission might be in a bad state.
  RespErrLevelMissionCritical        = 4040u32, 
  /// response code for: If you see this, it is probably reasonable to follow that advice..
  /// Something unexplicable, which should never have happened, did happen and there is probably
  /// no way to fix it. Call somebody if you see it, but probably the mission has failed.
  RespErrLevelRunFoolRun             = 99999u32, 
  /// response code for: The server has executed the command succesfully. 
  /// THIS DOES NOT GUARANTEE THAT SERVER IS ACTUALLY DOING 
  /// SOMETHING USEFUL, IT JUST ACKNOWLEDGES EXECUTION.
  RespSuccFingersCrossed             = 200u32,
  /// The command can't be executed since currently data taking is not active
  RespErrNoRunActive                 = 501u32,
  /// The command can't be executed since currently data taking is active
  RespErrRunActive                   = 502u32,
  /// The command got stuck somewhere and did not make it to the intended receiver
  RespErrCmdStuck                    = 503u32
}

impl fmt::Display for TofResponseCode {
  fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
    let r = serde_json::to_string(self).unwrap_or(
      String::from("Error: cannot unwrap this TofResponseCode"));
    write!(f, "<TofResponseCode: {}>", r)
  }
}

impl From<u32> for TofResponseCode {
  fn from(value: u32) -> Self {
    match value {
      500u32   => TofResponseCode::RespErrUnexecutable,
      403u32   => TofResponseCode::RespErrAccessDenied,
      404u32   => TofResponseCode::RespErrNotImplemented,
      4000u32  => TofResponseCode::RespErrLevelNoProblem,
      4010u32  => TofResponseCode::RespErrLevelMedium,
      4020u32  => TofResponseCode::RespErrLevelSevere,
      4030u32  => TofResponseCode::RespErrLevelCritical,
      4040u32  => TofResponseCode::RespErrLevelMissionCritical,
      99999u32 => TofResponseCode::RespErrLevelRunFoolRun,
      200u32   => TofResponseCode::RespSuccFingersCrossed,
      501u32   => TofResponseCode::RespErrNoRunActive,
      502u32   => TofResponseCode::RespErrRunActive,
      503u32   => TofResponseCode::RespErrCmdStuck,
      _        => TofResponseCode::Unknown
    }
  }
}

#[cfg(feature = "random")]
impl FromRandom for TofResponseCode {
  
  fn from_random() -> Self {
    let choices = [
      TofResponseCode::RespErrAccessDenied,
      TofResponseCode::RespErrUnexecutable,
      TofResponseCode::RespErrNotImplemented,
      TofResponseCode::RespErrLevelNoProblem,
      TofResponseCode::RespErrLevelMedium,
      TofResponseCode::RespErrLevelSevere,
      TofResponseCode::RespErrLevelCritical,
      TofResponseCode::RespErrLevelMissionCritical,
      TofResponseCode::RespErrLevelRunFoolRun,
      TofResponseCode::RespSuccFingersCrossed,
      TofResponseCode::RespErrNoRunActive,
      TofResponseCode::RespErrRunActive,
      TofResponseCode::RespErrCmdStuck
    ];
    let mut rng  = rand::thread_rng();
    let idx = rng.gen_range(0..choices.len());
    choices[idx]
  }
}

/// How to operate the readout Default mode is to request
/// events from the MasterTrigger. However, we can also stream
/// all the waveforms.
/// CAVEAT: For the whole tof, this will cap the rate at 
/// 112 Hz, because of the capacity of the switches.
#[derive(Debug, Copy, Clone, PartialEq, serde::Deserialize, serde::Serialize)]
#[repr(u8)]
pub enum TofOperationMode {
  Unknown          = 0u8,
  Default          = 1u8,
  //#[deprecated(since="0.8.3")] 
  //StreamAny        = 10u8,
  //#[deprecated(since="0.8.3")] 
  //RequestReply     = 20u8,
  /// Don't decode any of the event 
  /// data on the RB, just push it 
  /// onward
  RBHighThroughput = 30u8,
  RBCalcCRC32      = 40u8,
  RBWaveform       = 50u8,
}

impl fmt::Display for TofOperationMode {
  fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
    let r = serde_json::to_string(self).unwrap_or(
      String::from("Error: cannot unwrap this TofOperationMode"));
    write!(f, "<TofOperationMode: {}>", r)
  }
}

impl From<u8> for TofOperationMode {
  fn from(value: u8) -> Self {
    match value {
      0u8  => TofOperationMode::Unknown,
      1u8  => TofOperationMode::Default,
      //10u8 => TofOperationMode::StreamAny,
      //20u8 => TofOperationMode::RequestReply,
      30u8 => TofOperationMode::RBHighThroughput,
      40u8 => TofOperationMode::RBCalcCRC32,
      50u8 => TofOperationMode::RBWaveform,
      _    => TofOperationMode::Unknown
    }
  }
}

#[cfg(feature = "random")]
impl FromRandom for TofOperationMode {
  
  fn from_random() -> Self {
    let choices = [
      TofOperationMode::Unknown,
      TofOperationMode::Default,
      //TofOperationMode::RequestReply,
      //TofOperationMode::StreamAny,
      TofOperationMode::RBHighThroughput,
      TofOperationMode::RBCalcCRC32,
      TofOperationMode::RBWaveform,
      TofOperationMode::Unknown
    ];
    let mut rng  = rand::thread_rng();
    let idx = rng.gen_range(0..choices.len());
    choices[idx]
  }
}

//
//
//  pub fn command_code_to_string(cc : u8) -> String {
//    match cc {
//      Self::REQUEST_EVENT => {
//        return String::from("GetReducedDataPacket");
//      }
//      _ => {
//        return String::from("Unknown");
//      }
//    }
//  }
//}
//

/// General command class for ALL commands to the 
/// tof C&C instance and readout boards
///
/// Each command can carry a 32bit field with further
/// instructionns
///
#[derive(Debug, PartialEq, Copy, Clone, serde::Deserialize, serde::Serialize)]//, IntoEnumIterator)]
pub enum TofCommand {
  Unknown                 (u32),
  Ping                    (u32),
  Moni                    (u32),
  Power                   (u32),
  SetThresholds           (u32),
  SetMTConfig             (u32),
  SetPreampBias           (u32),
  DataRunStop             (u32),
  DataRunStart            (u32),
  StartValidationRun      (u32),
  GetFullWaveforms        (u32),
  NoiCalibration          (u32),
  VoltageCalibration      (u32),
  TimingCalibration       (u32),
  DefaultCalibration      (u32),
  UnspoolEventCache       (u32),
  TriggerModeForced       (u32),
  TriggerModeForcedMTB    (u32),
  SystemdReboot           (u32),
  Listen                  (u32),
  Lock                    (u32),
  Unlock                  (u32),
  Kill                    (u32),
}

impl Packable for TofCommand {
  const PACKET_TYPE : PacketType = PacketType::TofCommand;
}

impl fmt::Display for TofCommand {
  fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
    let r = serde_json::to_string(self).unwrap_or(
      String::from("Error: cannot unwrap this TofCommand"));
    write!(f, "<TofCommand: {}>", r)
  }
}

impl TofCommand { 

  // this can not fail
  pub fn get_value(&self) -> u32 {
    let value : u32;
    match self {
      TofCommand::Unknown                 (data) => { value = *data;}, 
      TofCommand::Ping                    (data) => { value = *data;},
      TofCommand::Moni                    (data) => { value = *data;},
      TofCommand::Power                   (data) => { value = *data;},
      TofCommand::SetThresholds           (data) => { value = *data;},
      TofCommand::SetMTConfig             (data) => { value = *data;},
      TofCommand::SetPreampBias           (data) => { value = *data;},
      TofCommand::DataRunStop             (data) => { value = *data;},
      TofCommand::DataRunStart            (data) => { value = *data;},
      TofCommand::StartValidationRun      (data) => { value = *data;},
      TofCommand::GetFullWaveforms        (data) => { value = *data;},
      TofCommand::NoiCalibration          (data) => { value = *data;},
      TofCommand::VoltageCalibration      (data) => { value = *data;},
      TofCommand::TimingCalibration       (data) => { value = *data;},
      TofCommand::DefaultCalibration      (data) => { value = *data;},
      TofCommand::UnspoolEventCache       (data) => { value = *data;},
      TofCommand::TriggerModeForced       (data) => { value = *data;},
      TofCommand::TriggerModeForcedMTB    (data) => { value = *data;},
      TofCommand::SystemdReboot           (data) => { value = *data;},
      TofCommand::Listen                  (data) => { value = *data;}
      TofCommand::Kill                    (data) => { value = *data;}
      TofCommand::Lock                    (data) => { value = *data;}
      TofCommand::Unlock                  (data) => { value = *data;}
    }
    value
  }  

  /// Generate a TofCommand from the specific bytecode
  /// representation
  pub fn from_command_code(cc : TofCommandCode, value : u32) -> TofCommand {
    match cc {
      TofCommandCode::Unknown                 => TofCommand::Unknown                 (value),
      TofCommandCode::Ping                    => TofCommand::Ping                    (value),
      TofCommandCode::Moni                    => TofCommand::Moni                    (value),
      TofCommandCode::SetLTBThresholds        => TofCommand::SetThresholds           (value),
      TofCommandCode::SetMTConfig             => TofCommand::SetMTConfig             (value),
      TofCommandCode::SetPreampBias           => TofCommand::SetPreampBias           (value),
      TofCommandCode::DataRunStop             => TofCommand::DataRunStop             (value),
      TofCommandCode::DataRunStart            => TofCommand::DataRunStart            (value),
      TofCommandCode::StartValidationRun      => TofCommand::StartValidationRun      (value),
      TofCommandCode::GetFullWaveforms        => TofCommand::GetFullWaveforms        (value),
      TofCommandCode::RBCalibration           => TofCommand::DefaultCalibration           (value),
      TofCommandCode::UnspoolEventCache       => TofCommand::UnspoolEventCache       (value),
      TofCommandCode::RestartLiftofRBClients  => TofCommand::SystemdReboot           (value),
      TofCommandCode::Listen                  => TofCommand::Listen                  (value),
      TofCommandCode::Kill                    => TofCommand::Kill                    (value),
      TofCommandCode::Lock                    => TofCommand::Lock                    (value),
      TofCommandCode::Unlock                  => TofCommand::Unlock                  (value),
      _                                          => TofCommand::Unknown                 (value),
    }
  }
    
  /// Translate a TofCommand into its specific byte representation
  pub fn to_command_code(cmd : &TofCommand) -> Option<TofCommandCode> {
    match cmd {
      TofCommand::Unknown                 (_) => Some(TofCommandCode::Unknown),
      TofCommand::Power                   (_) => Some(TofCommandCode::Unknown),
      TofCommand::NoiCalibration          (_) => Some(TofCommandCode::Unknown),
      TofCommand::VoltageCalibration      (_) => Some(TofCommandCode::Unknown),
      TofCommand::TimingCalibration       (_) => Some(TofCommandCode::Unknown),
      TofCommand::TriggerModeForced       (_) => Some(TofCommandCode::Unknown),
      TofCommand::TriggerModeForcedMTB    (_) => Some(TofCommandCode::Unknown),

      TofCommand::Ping                    (_) => Some(TofCommandCode::Ping),
      TofCommand::Moni                    (_) => Some(TofCommandCode::Moni),
      TofCommand::SetThresholds           (_) => Some(TofCommandCode::SetLTBThresholds),
      TofCommand::SetMTConfig             (_) => Some(TofCommandCode::SetMTConfig),
      TofCommand::SetPreampBias           (_) => Some(TofCommandCode::SetPreampBias),
      TofCommand::DataRunStop             (_) => Some(TofCommandCode::DataRunStop),
      TofCommand::DataRunStart            (_) => Some(TofCommandCode::DataRunStart),
      TofCommand::StartValidationRun      (_) => Some(TofCommandCode::StartValidationRun),
      TofCommand::GetFullWaveforms        (_) => Some(TofCommandCode::GetFullWaveforms),
      TofCommand::DefaultCalibration      (_) => Some(TofCommandCode::RBCalibration),
      TofCommand::UnspoolEventCache       (_) => Some(TofCommandCode::UnspoolEventCache),
      TofCommand::SystemdReboot           (_) => Some(TofCommandCode::RestartLiftofRBClients),
      TofCommand::Listen                  (_) => Some(TofCommandCode::Listen),
      TofCommand::Kill                    (_) => Some(TofCommandCode::Kill),
      TofCommand::Lock                    (_) => Some(TofCommandCode::Lock),
      TofCommand::Unlock                  (_) => Some(TofCommandCode::Unlock),
    }
  }

  pub fn from_tof_packet(packet : &TofPacket) 
    -> Option<TofCommand> {
    match packet.packet_type {
      PacketType::TofCommand => (),
      _ => {
        debug!("Packet doesn't have PacketType::TofCommand");
        return None;
        }
    } // end match
    let cmd_pk = TofCommand::from_bytestream(&packet.payload, &mut 0);
    match cmd_pk {
      Err(err) => {
        warn!("Could not decode CMD packet, err {:?}", err);
        return None;
      }
      Ok(cmd) => {
        Some(cmd) 
      }
    } // end match
  }
} // end impl TofCommand

//impl From<&TofPacket> for TofCommand {
//  fn from(tp : &TofPacket) -> Self {

impl From<(u8, u32)> for TofCommand {
  
  /// Generate a TofCommand from a pair of code, value
  ///
  /// The first argument must be the command code, the 
  /// second the specific value of the command.
  fn from(pair : (u8, u32)) -> TofCommand {
    let (input, value) = pair;
    trace!("Got in input {:?}", pair);
    TofCommand::from_command_code(TofCommandCode::try_from(input).unwrap(), value)
  }
}

#[cfg(feature = "random")]
impl FromRandom for TofCommand {
  
  fn from_random() -> Self {
    let mut rng  = rand::thread_rng();
    let val = rng.gen::<u32>();
    let choices = [
      TofCommand::Unknown                 (val),
      TofCommand::Ping                    (val),
      TofCommand::Moni                    (val),
      TofCommand::Power                   (val),
      TofCommand::SetThresholds           (val),
      TofCommand::SetMTConfig             (val),
      TofCommand::SetPreampBias           (val),
      TofCommand::DataRunStop             (val),
      TofCommand::DataRunStart            (val),
      TofCommand::StartValidationRun      (val),
      TofCommand::GetFullWaveforms        (val),
      TofCommand::NoiCalibration          (val),
      TofCommand::VoltageCalibration      (val),
      TofCommand::TimingCalibration       (val),
      TofCommand::DefaultCalibration      (val),
      TofCommand::UnspoolEventCache       (val),
      TofCommand::TriggerModeForced       (val),
      TofCommand::TriggerModeForcedMTB    (val),
      TofCommand::SystemdReboot           (val),
      TofCommand::Listen                  (val),
      TofCommand::Kill                    (val),
      TofCommand::Lock                    (val),
      TofCommand::Unlock                  (val),
    ];
    let idx = rng.gen_range(0..choices.len());
    choices[idx]
  }
}

impl Serialization for TofCommand {
  
  const HEAD : u16 = 0xAAAA;
  const TAIL : u16 = 0x5555;
  ///// The size of TofCommand when 
  ///// in byte representation is 
  ///// fixed:
  ///// it is 4 bytes (header/footer)
  ///// + 1 byte command code
  ///// + 4 bytes value
  ///// => 9 bytes
  const SIZE : usize = 9; 
  
  ///// Returns the serialized data stream
  ///// as byte array
  ///// 
  ///// Might be faster thant its sister
  ///// ::to_bytestream(), however is not
  ///// a trait, since the return type 
  ///// depends on the size. 
  ///// FIXME - can we somehow make this 
  ///// a trait? It seems we can not return 
  ///// &[u8] when we have the corresponding
  ///// array allocated in the function
  //pub fn to_bytearray(&self) -> [u8;TofCommand::SIZE] {

  //  let mut bytes = [0u8;TofCommand::SIZE];
  //  bytes[0] = 0xAA;
  //  bytes[1] = 0xAA;
  //  bytes[2] = TofCommand::to_command_code(&self)
  //    .expect("This can't fail, since this is implemented on MYSELF and I am a TofCommand!") as u8; 
  //  let value_bytes = self.get_value().to_le_bytes();
  // 
  //  for n in 0..4 {
  //    bytes[3+n] = value_bytes[n];
  //  }
  //  bytes[7] = 0x55;
  //  bytes[8] = 0x55;
  //  bytes
  //}
  
  fn to_bytestream(&self) -> Vec<u8> {
    //let mut stream = Vec::<u8>::with_capacity(TofCommand::SIZE);
    let mut stream : Vec::<u8> = vec![0,0,0,0,0,0,0,0,0];
    stream[0] = 0xAA;
    stream[1] = 0xAA;
    stream[2] = TofCommand::to_command_code(&self)
      .expect("This can't fail, since this is implemented on MYSELF and I am a TofCommand!") as u8; 
    let value_bytes = self.get_value().to_le_bytes();
   
    for n in 0..4 {
      stream[3+n] = value_bytes[n];
    }
    stream[7] = 0x55;
    stream[8] = 0x55;
    stream
  }


  fn from_bytestream(stream    : &Vec<u8>, 
                     pos       : &mut usize) 
    -> Result<Self, SerializationError>{
    Self::verify_fixed(stream, pos)?;  
    let cc      = parse_u8(stream, pos);
    let value   = parse_u32(stream, pos); 
    let pair    = (cc, value);
    let command = Self::from(pair);
    *pos += 2; // for the TAIL
    Ok(command)
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

impl Packable for TofResponse {
  const PACKET_TYPE : PacketType = PacketType::TofResponse;
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
    let mut rng  = rand::thread_rng();
    let val = rng.gen::<u32>();
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
    let idx = rng.gen_range(0..choices.len());
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
fn test_tofoperationmode() {
  let mut type_codes = Vec::<u8>::new();
  type_codes.push(TofOperationMode::Unknown as u8); 
  type_codes.push(TofOperationMode::Default as u8); 
  type_codes.push(TofOperationMode::RBHighThroughput as u8); 
  type_codes.push(TofOperationMode::RBWaveform as u8); 
  type_codes.push(TofOperationMode::RBCalcCRC32 as u8); 
  //type_codes.push(TofOperationMode::StreamAny as u8); 
  //type_codes.push(TofOperationMode::RequestReply as u8); 
  for tc in type_codes.iter() {
    assert_eq!(*tc,TofOperationMode::try_from(*tc).unwrap() as u8);  
  }
}

#[cfg(feature = "random")]
#[test]
fn pack_tofresponse() {
  let resp = TofResponse::from_random();
  let test : TofResponse = resp.pack().unpack().unwrap();
  assert_eq!(resp, test);
}

//#[cfg(feature = "random")]
//#[test]
//fn pack_tofcommand() {
//  for _ in 0..100 {
//    let cmd  = TofCommand::from_random();
//    let test : TofCommand = cmd.pack().unpack().unwrap();
//    assert_eq!(cmd, test);
//  }
//}

#[cfg(feature = "random")]
#[test]
fn pack_tofcommandv2() {
  for _ in 0..100 {
    let cmd  = TofCommandV2::from_random();
    let test : TofCommandV2 = cmd.pack().unpack().unwrap();
    assert_eq!(cmd, test);
  }
}

#[test]
fn forge_and_extract_tofcommandv2() {
  let input  = vec![String::from("foo"), String::from("bar"), String::from("baz")];
  let cmd    = TofCommandV2::forge_changerunconfig(&input);
  let output = cmd.extract_changerunconfig().unwrap();
  assert_eq!(input, output);
}
