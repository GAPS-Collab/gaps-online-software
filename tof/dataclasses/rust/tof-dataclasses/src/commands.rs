//! Commmands which can be issued
//! to the various components of 
//! the tof system.
//!
//!
//! Here is a comprehensive list (Sydney)
//! * Power on/off to PBs+RBs+LTBs+preamps (all at once) or MT
//! * Power on/off to LTB or preamp < 2/day Command to power on/off various components (to TOF -> to RB) 5 B:
//! * RBsetup ? Command to run rbsetup on a particular RB (to TOF -> to RBs) 8 B:
//! * Set Thresholds < 3/day Command to set a threshold level on all LTBs (to TOF -> to RBs) 8 B:
//! * Set MT Config 1/run, <10/day? Command to set MT trigger config (to TOF -> to MT) 4 B:
//! * Start Validation Run 1/run, <10/day? Command to take a small amount of data (some number E events, I
//! * 360xE full waveforms (from TOF)
//! 
//! * Start Data-Taking Run 1/run, <10/day? Command to take regular data (to TOF -> to RBs)
//! * Reduced data packet (from Flight computer)
//! * Stop Run < 1/run, < 10/day Command to stop a run (to TOF -> to RBs) 2 B = command name 6
//! 
//! * Voltage Calibration Runs 1/day Command to take 2 voltage calibration runs (to TOF -> to RBs) 12 B:
//! * Timing Calibration Run 1/day Command to take a timing calibration run (to TOF -> to RBs) 8 B:
//! * Create New Calibration File 1/day Command to create a new calibration file using data from the three
//! 
//! Each command will be answered by a specific response. The responses 
//! consists of a class, `TofResponse` together with a 32bit response code.
//!

use std::fmt;

use crate::serialization::{
    Serialization,
    SerializationError,
    parse_u8,
    parse_u32
};

use crate::packets::{TofPacket,
                     PacketType};

cfg_if::cfg_if! {
  if #[cfg(feature = "random")]  {
    use crate::FromRandom;
    extern crate rand;
    use rand::Rng;
  }
}

#[derive(Debug, Copy, Clone, PartialEq, serde::Deserialize, serde::Serialize)]
#[repr(u8)]
pub enum TofCommandCode {
  CmdUnknown                 = 0u8,
  /// en empty command just to check if stuff is online
  CmdPing                    = 1u8,
  /// command code for getting the monitoring data from the component
  CmdMoni                    = 2u8,
  /// command code for power management
  CmdPower                   = 10u8,
  /// command code for "Set LTB Thresholds"
  CmdSetThresholds           = 21u8,         
  /// command code for "Configure MTB"
  CmdSetMTConfig             = 22u8,        
  /// command code for "Set preamp bias"
  CmdSetPreampBias           = 28u8,         
  /// command code for "Stop Data taking"
  CmdDataRunStop             = 30u8,  
  /// command code for "Start Data taking"
  CmdDataRunStart            = 31u8,    
  /// command code for "Start validation run"
  CmdStartValidationRun      = 32u8,         
  /// command code for "Get all waveforms"
  CmdGetFullWaveforms        = 41u8,
  /// command code for "Run no input calibration"
  CmdNoiCalibration          = 50u8,       
  /// command code for "Run voltage calibration"
  CmdVoltageCalibration      = 51u8,       
  /// command code for "Run timing calibration"
  CmdTimingCalibration       = 52u8,
  /// command code for "Run full calibration"
  CmdDefaultCalibration      = 53u8, 

  /// command code for "Send the whole event cache over the wire"
  CmdUnspoolEventCache       = 44u8,

  /// command code for setting the size of the rb buffers.
  /// technically, this does not change the size, but sets 
  /// a different value for trip
  CmdSetRBDataBufSize        = 23u8,
  /// command code for enabling/disabling the forced trigger mode
  /// on the RBs
  CmdTriggerModeForced       = 24u8,
  /// command code for enabling/disabling the forced trigger mode
  /// on the MTB
  CmdTriggerModeForcedMTB    = 25u8,

  /// command code for restarting systemd
  CmdSystemdReboot           = 60u8,
  /// command code for putting liftof-cc in listening mode
  CmdListen                  = 70u8,
  /// command code for putting liftof-cc in staging mode
  CmdStaging                 = 71u8
}

impl fmt::Display for TofCommandCode {
  fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
    let r = serde_json::to_string(self).unwrap_or(
      String::from("Error: cannot unwrap this TofCommandCodes"));
    write!(f, "<TofCommandCodes: {}>", r)
  }
}

impl From<u8> for TofCommandCode {
  fn from(value: u8) -> Self {
    match value {
      0u8  => TofCommandCode::CmdUnknown,
      1u8  => TofCommandCode::CmdPing,
      2u8  => TofCommandCode::CmdMoni,
      10u8 => TofCommandCode::CmdPower,
      21u8 => TofCommandCode::CmdSetThresholds,
      22u8 => TofCommandCode::CmdSetMTConfig,
      28u8 => TofCommandCode::CmdSetPreampBias,
      30u8 => TofCommandCode::CmdDataRunStop,
      31u8 => TofCommandCode::CmdDataRunStart,
      32u8 => TofCommandCode::CmdStartValidationRun,
      41u8 => TofCommandCode::CmdGetFullWaveforms,
      50u8 => TofCommandCode::CmdNoiCalibration,
      51u8 => TofCommandCode::CmdVoltageCalibration,
      52u8 => TofCommandCode::CmdTimingCalibration,
      53u8 => TofCommandCode::CmdDefaultCalibration,
      44u8 => TofCommandCode::CmdUnspoolEventCache,
      23u8 => TofCommandCode::CmdSetRBDataBufSize,
      24u8 => TofCommandCode::CmdTriggerModeForced,
      25u8 => TofCommandCode::CmdTriggerModeForcedMTB,
      60u8 => TofCommandCode::CmdSystemdReboot,
      70u8 => TofCommandCode::CmdListen,
      71u8 => TofCommandCode::CmdStaging,
      _    => TofCommandCode::CmdUnknown
    }
  }
}

#[cfg(feature = "random")]
impl FromRandom for TofCommandCode {
  fn from_random() -> Self {
    let choices = [
      TofCommandCode::CmdUnknown,
      TofCommandCode::CmdPing,
      TofCommandCode::CmdMoni,
      TofCommandCode::CmdPower,
      TofCommandCode::CmdSetThresholds,
      TofCommandCode::CmdSetMTConfig,
      TofCommandCode::CmdSetPreampBias,
      TofCommandCode::CmdDataRunStop,
      TofCommandCode::CmdDataRunStart,
      TofCommandCode::CmdStartValidationRun,
      TofCommandCode::CmdGetFullWaveforms,
      TofCommandCode::CmdNoiCalibration,
      TofCommandCode::CmdVoltageCalibration,
      TofCommandCode::CmdTimingCalibration,
      TofCommandCode::CmdDefaultCalibration,
      TofCommandCode::CmdUnspoolEventCache,
      TofCommandCode::CmdSetRBDataBufSize,
      TofCommandCode::CmdTriggerModeForced,
      TofCommandCode::CmdTriggerModeForcedMTB,
      TofCommandCode::CmdSystemdReboot,
      TofCommandCode::CmdListen,
      TofCommandCode::CmdStaging
    ];
    let mut rng  = rand::thread_rng();
    let idx = rng.gen_range(0..choices.len());
    choices[idx]
  }
}

// Specific response codes
// These are long (4 bytes) but 
// this allows to convey more information
// e.g. event id
#[derive(Debug, Copy, Clone, PartialEq, serde::Deserialize, serde::Serialize)]
#[repr(u32)]
pub enum TofCommandResp {
  Unknown                            = 0u32,
  /// response code for: Command can not be executed on the server side
  RespErrUnexecutable                = 500u32,
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

impl fmt::Display for TofCommandResp {
  fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
    let r = serde_json::to_string(self).unwrap_or(
      String::from("Error: cannot unwrap this TofCommandResp"));
    write!(f, "<TofCommandResp: {}>", r)
  }
}

impl From<u32> for TofCommandResp {
  fn from(value: u32) -> Self {
    match value {
      500u32   => TofCommandResp::RespErrUnexecutable,
      404u32   => TofCommandResp::RespErrNotImplemented,
      4000u32  => TofCommandResp::RespErrLevelNoProblem,
      4010u32  => TofCommandResp::RespErrLevelMedium,
      4020u32  => TofCommandResp::RespErrLevelSevere,
      4030u32  => TofCommandResp::RespErrLevelCritical,
      4040u32  => TofCommandResp::RespErrLevelMissionCritical,
      99999u32 => TofCommandResp::RespErrLevelRunFoolRun,
      200u32   => TofCommandResp::RespSuccFingersCrossed,
      501u32   => TofCommandResp::RespErrNoRunActive,
      502u32   => TofCommandResp::RespErrRunActive,
      503u32   => TofCommandResp::RespErrCmdStuck,
      _        => TofCommandResp::Unknown
    }
  }
}

#[cfg(feature = "random")]
impl FromRandom for TofCommandResp {
  
  fn from_random() -> Self {
    let choices = [
      TofCommandResp::RespErrUnexecutable,
      TofCommandResp::RespErrNotImplemented,
      TofCommandResp::RespErrLevelNoProblem,
      TofCommandResp::RespErrLevelMedium,
      TofCommandResp::RespErrLevelSevere,
      TofCommandResp::RespErrLevelCritical,
      TofCommandResp::RespErrLevelMissionCritical,
      TofCommandResp::RespErrLevelRunFoolRun,
      TofCommandResp::RespSuccFingersCrossed,
      TofCommandResp::RespErrNoRunActive,
      TofCommandResp::RespErrRunActive,
      TofCommandResp::RespErrCmdStuck
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

/// Command class to control ReadoutBoards
#[derive(Debug, Copy, Clone, PartialEq)]
pub struct RBCommand {
  pub rb_id        : u8, // receipient
  pub command_code : u8,
  pub channel_mask : u8,
  pub payload      : u32,
}

impl RBCommand {
  pub const REQUEST_EVENT : u8 = 10; 
  pub fn new() -> Self {
    Self {
      rb_id        : 0,
      command_code : 0,
      channel_mask : 0,
      payload      : 0,
    }
  }

  pub fn get_payload_from_stream(stream : &Vec<u8>) -> u32 {
    parse_u32(stream, &mut 3)
  }

  pub fn command_code_to_string(cc : u8) -> String {
    match cc {
      Self::REQUEST_EVENT => {
        return String::from("GetReducedDataPacket");
      }
      _ => {
        return String::from("Unknown");
      }
    }
  }
}

impl From<&TofPacket> for RBCommand {
  fn from(pk : &TofPacket) -> Self {
    let mut cmd = RBCommand::new();
    if pk.packet_type == PacketType::RBCommand {
      match RBCommand::from_bytestream(&pk.payload, &mut 0) {
        Ok(_cmd) => {
          cmd = _cmd;
        },
        Err(err) => {
          error!("Can not get RBCommand from TofPacket, error {err}");
        }
      }
    }
    cmd
  }
}
impl fmt::Display for RBCommand {
  fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
    let cc = RBCommand::command_code_to_string(self.command_code);
    write!(f, "<RBCommand: {}; RB ID {}; CH MASK {}; PAYLOAD {}>", cc, self.rb_id, self.channel_mask, self.payload)
  }
}

impl Default for RBCommand {
  fn default() -> Self {
    RBCommand::new()
  }
}

impl Serialization for RBCommand {
  
  const HEAD : u16 = 0xAAAA;
  const TAIL : u16 = 0x5555;
  const SIZE : usize = 11; 

  fn from_bytestream(stream    : &Vec<u8>, 
                     pos       : &mut usize) 
    -> Result<Self, SerializationError>{
    Self::verify_fixed(stream, pos)?;
    let mut command      = RBCommand::new();
    command.rb_id        = parse_u8(stream, pos);
    command.command_code = parse_u8(stream, pos);
    command.channel_mask = parse_u8(stream, pos);
    command.payload      = parse_u32(stream, pos);
    *pos += 2;
    Ok(command)
  }

  fn to_bytestream(&self) -> Vec<u8> {
    let mut stream = Vec::<u8>::with_capacity(9);
    stream.extend_from_slice(&RBCommand::HEAD.to_le_bytes());
    stream.push(self.rb_id);
    stream.push(self.command_code);
    stream.push(self.channel_mask);
    stream.extend_from_slice(&self.payload.to_le_bytes());
    stream.extend_from_slice(&RBCommand::TAIL.to_le_bytes());
    stream
  }
}

#[cfg(feature = "random")]
impl FromRandom for RBCommand {    
  fn from_random() -> Self {
    let mut rng = rand::thread_rng();
    Self {
      rb_id        : rng.gen::<u8>(),
      command_code : rng.gen::<u8>(),
      channel_mask : rng.gen::<u8>(),
      payload      : rng.gen::<u32>(),
    }
  }
}

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
  SetRBDataBufSize        (u32),
  TriggerModeForced       (u32),
  TriggerModeForcedMTB    (u32),
  SystemdReboot           (u32),
  Listen                  (u32),
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
      TofCommand::SetRBDataBufSize        (data) => { value = *data;},
      TofCommand::TriggerModeForced       (data) => { value = *data;},
      TofCommand::TriggerModeForcedMTB    (data) => { value = *data;},
      TofCommand::SystemdReboot           (data) => { value = *data;},
      TofCommand::Listen                  (data) => { value = *data;}
    }
    value
  }  

  /// Generate a TofCommand from the specific bytecode
  /// representation
  pub fn from_command_code(cc : TofCommandCode, value : u32) -> TofCommand {
    match cc {
      TofCommandCode::CmdUnknown                 => TofCommand::Unknown                 (value),
      TofCommandCode::CmdPing                    => TofCommand::Ping                    (value),
      TofCommandCode::CmdMoni                    => TofCommand::Moni                    (value),
      TofCommandCode::CmdPower                   => TofCommand::Power                   (value),
      TofCommandCode::CmdSetThresholds           => TofCommand::SetThresholds           (value),
      TofCommandCode::CmdSetMTConfig             => TofCommand::SetMTConfig             (value),
      TofCommandCode::CmdSetPreampBias           => TofCommand::SetPreampBias           (value),
      TofCommandCode::CmdDataRunStop             => TofCommand::DataRunStop             (value),
      TofCommandCode::CmdDataRunStart            => TofCommand::DataRunStart            (value),
      TofCommandCode::CmdStartValidationRun      => TofCommand::StartValidationRun      (value),
      TofCommandCode::CmdGetFullWaveforms        => TofCommand::GetFullWaveforms        (value),
      TofCommandCode::CmdNoiCalibration          => TofCommand::NoiCalibration          (value),
      TofCommandCode::CmdVoltageCalibration      => TofCommand::VoltageCalibration      (value),
      TofCommandCode::CmdTimingCalibration       => TofCommand::TimingCalibration       (value),
      TofCommandCode::CmdDefaultCalibration      => TofCommand::DefaultCalibration      (value),
      TofCommandCode::CmdUnspoolEventCache       => TofCommand::UnspoolEventCache       (value),
      TofCommandCode::CmdSetRBDataBufSize        => TofCommand::SetRBDataBufSize        (value),
      TofCommandCode::CmdTriggerModeForced       => TofCommand::TriggerModeForced       (value),
      TofCommandCode::CmdTriggerModeForcedMTB    => TofCommand::TriggerModeForcedMTB    (value),
      TofCommandCode::CmdSystemdReboot           => TofCommand::SystemdReboot           (value),
      TofCommandCode::CmdListen                  => TofCommand::Listen                  (value),
      _                                          => TofCommand::Unknown                 (value),
    }
  }
    
  /// Translate a TofCommand into its specific byte representation
  pub fn to_command_code(cmd : &TofCommand) -> Option<TofCommandCode> {
    match cmd {
      TofCommand::Unknown                 (_) => Some(TofCommandCode::CmdUnknown),
      TofCommand::Ping                    (_) => Some(TofCommandCode::CmdPing),
      TofCommand::Moni                    (_) => Some(TofCommandCode::CmdMoni),
      TofCommand::Power                   (_) => Some(TofCommandCode::CmdPower),
      TofCommand::SetThresholds           (_) => Some(TofCommandCode::CmdSetThresholds),
      TofCommand::SetMTConfig             (_) => Some(TofCommandCode::CmdSetMTConfig),
      TofCommand::SetPreampBias           (_) => Some(TofCommandCode::CmdSetPreampBias),
      TofCommand::DataRunStop             (_) => Some(TofCommandCode::CmdDataRunStop),
      TofCommand::DataRunStart            (_) => Some(TofCommandCode::CmdDataRunStart),
      TofCommand::StartValidationRun      (_) => Some(TofCommandCode::CmdStartValidationRun),
      TofCommand::GetFullWaveforms        (_) => Some(TofCommandCode::CmdGetFullWaveforms),
      TofCommand::NoiCalibration          (_) => Some(TofCommandCode::CmdNoiCalibration),
      TofCommand::VoltageCalibration      (_) => Some(TofCommandCode::CmdVoltageCalibration),
      TofCommand::TimingCalibration       (_) => Some(TofCommandCode::CmdTimingCalibration),
      TofCommand::DefaultCalibration      (_) => Some(TofCommandCode::CmdDefaultCalibration),
      TofCommand::UnspoolEventCache       (_) => Some(TofCommandCode::CmdUnspoolEventCache),
      TofCommand::SetRBDataBufSize        (_) => Some(TofCommandCode::CmdSetRBDataBufSize),
      TofCommand::TriggerModeForced       (_) => Some(TofCommandCode::CmdTriggerModeForced),
      TofCommand::TriggerModeForcedMTB    (_) => Some(TofCommandCode::CmdTriggerModeForcedMTB),
      TofCommand::SystemdReboot           (_) => Some(TofCommandCode::CmdSystemdReboot),
      TofCommand::Listen                  (_) => Some(TofCommandCode::CmdListen),
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
      TofCommand::SetRBDataBufSize        (val),
      TofCommand::TriggerModeForced       (val),
      TofCommand::TriggerModeForcedMTB    (val),
      TofCommand::SystemdReboot           (val),
      TofCommand::Listen                  (val),
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
  Unknown
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
      TofResponse::Unknown,
    ];
    let idx = rng.gen_range(0..choices.len());
    choices[idx]
  }
}

impl TofResponse {
  const HEAD : u16 = 0xAAAA;
  const TAIL : u16 = 0x5555;

  pub fn to_bytestream(&self) -> Vec<u8> {
    let mut bytestream = Vec::<u8>::with_capacity(9);
    bytestream.extend_from_slice(&TofResponse::HEAD.to_le_bytes());
    let cc = u8::from(*self);
    bytestream.push(cc);
    let mut value : u32 = 0;
    match self {
      TofResponse::Success(data)            => value = *data,
      TofResponse::GeneralFail(data)        => value = *data,
      TofResponse::EventNotReady(data)      => value = *data,
      TofResponse::SerializationIssue(data) => value = *data,
      TofResponse::ZMQProblem(data)         => value = *data,
      TofResponse::Unknown => ()
    }
    bytestream.extend_from_slice(&value.to_le_bytes());
    bytestream.extend_from_slice(&TofResponse::TAIL.to_le_bytes());
    bytestream
  }
}

impl Serialization for TofResponse {
  const HEAD : u16 = 0xAAAA;
  const TAIL : u16 = 0x5555;
  const SIZE : usize = 0; //FIXME

  fn from_bytestream(stream    : &Vec<u8>, 
                     pos       : &mut usize) 
    -> Result<TofResponse, SerializationError>{
  
    let mut two_bytes : [u8;2];
    let four_bytes    : [u8;4];
    two_bytes = [stream[*pos],
                 stream[*pos+1]];
    *pos += 2;
    if TofResponse::HEAD != u16::from_le_bytes(two_bytes) {
      warn!("Packet does not start with HEAD signature");
      return Err(SerializationError::HeadInvalid {});
    }
   
    let cc   = stream[*pos];
    *pos += 1;
    four_bytes = [stream[*pos],
                  stream[*pos+1],
                  stream[*pos+2],
                  stream[*pos+3]];
    *pos += 4;
    let value = u32::from_le_bytes(four_bytes);
    two_bytes = [stream[*pos],
                 stream[*pos+1]];
    let pair = (cc, value);
    let response = TofResponse::from(pair);
    if TofResponse::TAIL != u16::from_le_bytes(two_bytes) {
      warn!("Packet does not end with TAIL signature");
      return Err(SerializationError::TailInvalid {});
    }
    Ok(response)
  }
}

impl From<TofResponse> for u8 {
  fn from(input : TofResponse) -> u8 {
    match input {
      TofResponse::Success(_)       => 1,
      TofResponse::GeneralFail(_)   => 2,
      TofResponse::EventNotReady(_) => 3,
      TofResponse::SerializationIssue(_) => 4,
      TofResponse::ZMQProblem(_) => 5,
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
fn serialization_rbcommand() {
  let cmd  = RBCommand::from_random();
  let test = RBCommand::from_bytestream(&cmd.to_bytestream(), &mut 0).unwrap();
  assert_eq!(cmd, test);
}
