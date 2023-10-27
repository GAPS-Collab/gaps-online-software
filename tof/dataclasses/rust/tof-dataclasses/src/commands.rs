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

use crate::serialization::{Serialization, SerializationError, parse_u8, parse_u32};
use crate::packets::{TofPacket,
                     PacketType};
#[cfg(feature = "random")] 
use crate::FromRandom;
#[cfg(feature = "random")]
extern crate rand;
#[cfg(feature = "random")]
use rand::Rng;

/// en empty command
pub const CMD_PING                : u8 = 1;
/// command code for "Power off"
pub const CMD_POFF                : u8 = 10;        
/// command code for "Power on"
pub const CMD_PON                 : u8 = 11;       
/// command code for "Power cycle"
pub const CMD_PCYCLE              : u8 = 12;        
/// command code for "Run RBSetup"
pub const CMD_RBSETUP             : u8 = 20;         
/// command code for "Set LTB Thresholds"
pub const CMD_SETTHRESHOLD        : u8 = 21;         
/// command code for "Configure MTB"
pub const CMD_SETMTCONFIG         : u8 = 22;        
/// command code for "Stop Data taking"
pub const CMD_DATARUNSTOP         : u8 = 30;  
/// command code for "Start Data taking"
pub const CMD_DATARUNSTART        : u8 = 31;    
/// command code for "Start validation run"
pub const CMD_STARTVALIDATIONRUN  : u8 = 32;         
/// command code for "Get all waveforms"
pub const CMD_GETFULLWAVEFORMS    : u8 = 41;      
/// command code for "Get waveforms/data for specific event"
pub const CMD_REQEUESTEVENT       : u8 = 42; 
/// command code for "Get monitoring data"
pub const CMS_REQUESTMONI         : u8 = 43;
/// command code for "Run voltage calibration"
pub const CMD_VCALIB              : u8 = 51;       
/// command code for "Run timing calibration"
pub const CMD_TCALIB              : u8 = 52;      
/// command code for "Create a new calibration file"
pub const CMD_CREATECALIBF        : u8 = 53;   

/// command code for "Send the whole event cache over the wire"
pub const CMD_UNSPOOL_EVENT_CACHE : u8 = 44;

/// command code for "Operate in a mode, where we stream any event 
/// (not only those which are requested)"
pub const CMD_STREAM_ANY_EVENT         : u8 = 45;
/// command code for "Stream only events which are explicitly requested"
pub const CMD_STREAM_ONLY_REQUESTED    : u8 = 46;
/// command code for setting the size of the rb buffers.
/// technically, this does not change the size, but sets 
/// a different value for trip
pub const CMD_SET_RB_DATABUF_SIZE      : u8 = 23;
/// command code for enable the forced trigger mode
/// on the RBs
pub const CMD_EN_TRIGGERMODE_FORCED    : u8 = 24;
/// command code to disable the forced trigger mode 
/// on the RBs
pub const CMD_DIS_TRIGGERMODE_FORCED   : u8 = 25;
/// Set forced trigger mode on MTB
pub const CMD_EN_TRIGGERMODE_FORCED_MTB : u8 = 26;
// Disable forced trigger mode on MTB
pub const CMD_DIS_TRIGGERMODE_FORCED_MTB : u8 = 27;

// Specific response codes
// These are long (4 bytes) but 
// this allows to convey more information
// e.g. event id

/// response code for: Command can not be executed on the server side
pub const RESP_ERR_UNEXECUTABLE              : u32 = 500;
pub const RESP_ERR_NOTIMPLEMENTED            : u32 = 404; 
/// response code for: Something did not work quite right, 
/// however, the problem has either fixed itself or it is 
/// highly likely that if the command is issued again it 
/// will succeed.
pub const RESP_ERR_LEVEL_NOPROBLEM           : u32 = 4000; 
pub const RESP_ERR_LEVEL_MEDIUM              : u32 = 4010; 
pub const RESP_ERR_LEVEL_SEVERE              : u32 = 4020; 
/// response code for: A critical condition. This might need a fix somehow and can 
/// not be fixed automatically. Probably at least a power-cycle is necessary.
pub const RESP_ERR_LEVEL_CRITICAL            : u32 = 4030; 
/// response code for: The severest error condition which can occur. This might
/// still be fixable, but it is probably a good advice to get help. Currently, 
/// the mission might be in a bad state.
pub const RESP_ERR_LEVEL_MISSION_CRITICAL    : u32 = 4040; 
/// response code for: If you see this, it is probably reasonable to follow that advice..
/// Something unexplicable, which should never have happened, did happen and there is probably
/// no way to fix it. Call somebody if you see it, but probably the mission has failed.
pub const RESP_ERR_LEVEL_RUN_FOOL_RUN        : u32 = 99999; 
/// response code for: The server has executed the command succesfully. 
/// THIS DOES NOT GUARANTEE THAT SERVER IS ACTUALLY DOING 
/// SOMETHING USEFUL, IT JUST ACKNOWLEDGES EXECUTION.
pub const RESP_SUCC_FINGERS_CROSSED          : u32 = 200;
/// The command can't be executed since currently data taking is not active
pub const RESP_ERR_NORUNACTIVE               : u32 = 501;
/// The command can't be executed since currently data taking is active
pub const RESP_ERR_RUNACTIVE                 : u32 = 502;
/// The command got stuck somewhere and did not make it to the intended receiver
pub const RESP_ERR_CMD_STUCK                 : u32 = 503;



/// How to operate the readout Default mode is to request
/// events from the MasterTrigger. However, we can also stream
/// all the waveforms.
/// CAVEAT: For the whole tof, this will cap the rate at 
/// 112 Hz, because of the capacity of the switches.
#[derive(Debug, Copy, Clone, PartialEq)]
pub enum TofOperationMode {
  RequestReply,
  StreamAny,
  Unknown
}

impl fmt::Display for TofOperationMode {
  fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
    let r = self.string_repr();
    write!(f, "<TofOperationMode: {}>", r)
  }
}

impl TofOperationMode {
  pub const UNKNOWN               : u8 = 0;
  pub const REQUESTREPLY          : u8 = 10;
  pub const STREAMANY             : u8 = 20;

  pub fn to_u8(&self) -> u8 {
    let result : u8;
    match self {
      TofOperationMode::Unknown => {
        result = TofOperationMode::UNKNOWN;
      }
      TofOperationMode::RequestReply => {
        result = TofOperationMode::REQUESTREPLY;
      }
      TofOperationMode::StreamAny => {
        result = TofOperationMode::STREAMANY;
      }
    }
    result
  }
  
  pub fn from_u8(code : &u8) -> Self {
    let mut result = TofOperationMode::Unknown;
    match *code {
      TofOperationMode::UNKNOWN => {
        result = TofOperationMode::Unknown;
      }
      TofOperationMode::REQUESTREPLY => {
        result = TofOperationMode::RequestReply;
      }
      TofOperationMode::STREAMANY => {
        result = TofOperationMode::StreamAny;
      }
      _ => {
        error!("Unknown TofOperationMode {}!", code);
      }
    }
    result
  }

  /// String representation of the TofOperationMode
  ///
  /// This is basically the enum type as 
  /// a string.
  pub fn string_repr(&self) -> String { 
    let repr : String;
    match self {
      TofOperationMode::Unknown => {
        repr = String::from("Unknown");
      }
      TofOperationMode::RequestReply => {
        repr = String::from("RequestReply");
      }
      TofOperationMode::StreamAny => {
        repr = String::from("StreamAny");
      }
    }
    repr
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

  pub fn command_code_to_string(cc : u8) -> String {
    match cc {
      Self::REQUEST_EVENT => {
        return String::from("RequestEvent");
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
    let mut command = RBCommand::new();
    command.rb_id        = parse_u8(stream, pos);
    command.command_code = parse_u8(stream, pos);
    command.channel_mask = parse_u8(stream, pos);
    command.payload = parse_u32(stream, pos);
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
#[derive(Debug, PartialEq, Copy, Clone)]//, IntoEnumIterator)]
pub enum TofCommand {
  Ping                    (u32),
  PowerOn                 (u32),
  PowerOff                (u32),
  PowerCycle              (u32),
  RBSetup                 (u32), 
  SetThresholds           (u32),
  SetMtConfig             (u32),
  StartValidationRun      (u32),
  RequestWaveforms        (u32),
  UnspoolEventCache       (u32),
  StreamAnyEvent          (u32),
  StreamOnlyRequested     (u32),
  /// Start a new run, the argument being the number 
  /// of events.
  DataRunStart            (u32), 
  DataRunEnd              (u32),
  VoltageCalibration      (u32),
  TimingCalibration       (u32),
  CreateCalibrationFile   (u32),
  /// Request event data for a specific event being sent
  /// over the data wire. The argument being the event id.
  RequestEvent            (u32),
  RequestMoni             (u32),
  /// Set RB buffer trip value
  SetRBBuffTrip           (u32),
  /// Switch forced trigger mode ON (RB)
  SetRBForcedTrigModeOn   (u32),
  /// Switch forced trigger mode OFF (RB)
  SetRBForcedTrigModeOff  (u32),
  SetMTBForcedTrigModeOn  (u32),
  /// Switch forced trigger mode OFF (RB)
  SetMTBForcedTrigModeOff (u32),
  Unknown                 (u32),
}

impl fmt::Display for TofCommand {
  fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
    let cmd = self.string_repr();
    //let arg = 
    write!(f, "<TofCommand {}>", cmd)
  }
}

impl Default for TofCommand {
  fn default() -> TofCommand {
    TofCommand::Unknown(0)
  }
}

impl TofCommand { 
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


  /// Returns the serialized data stream
  /// as byte array
  /// 
  /// Might be faster thant its sister
  /// ::to_bytestream(), however is not
  /// a trait, since the return type 
  /// depends on the size. 
  /// FIXME - can we somehow make this 
  /// a trait? It seems we can not return 
  /// &[u8] when we have the corresponding
  /// array allocated in the function
  pub fn to_bytearray(&self) -> [u8;TofCommand::SIZE] {

    let mut bytes = [0u8;TofCommand::SIZE];
    bytes[0] = 0xAA;
    bytes[1] = 0xAA;
    bytes[2] = TofCommand::to_command_code(&self).expect("This can't fail, since this is implemented on MYSELF and I am a TofCommand!"); 
    let value_bytes = self.get_value().to_le_bytes();
   
    for n in 0..4 {
      bytes[3+n] = value_bytes[n];
    }
    bytes[7] = 0x55;
    bytes[8] = 0x55;
    bytes
  }
  
  pub fn to_bytestream(&self) -> Vec<u8> {

    //let mut stream = Vec::<u8>::with_capacity(TofCommand::SIZE);
    let mut stream : Vec::<u8> = vec![0,0,0,0,0,0,0,0,0];
    stream[0] = 0xAA;
    stream[1] = 0xAA;
    stream[2] = TofCommand::to_command_code(&self).expect("This can't fail, since this is implemented on MYSELF and I am a TofCommand!"); 
    let value_bytes = self.get_value().to_le_bytes();
   
    for n in 0..4 {
      stream[3+n] = value_bytes[n];
    }
    stream[7] = 0x55;
    stream[8] = 0x55;
    stream
  }


  // this can not fail
  pub fn get_value(&self) -> u32 {
    let value : u32;
    match self {
      TofCommand::Ping                    (data) => { value = *data;},
      TofCommand::PowerOn                 (data) => { value = *data;}, 
      TofCommand::PowerOff                (data) => { value = *data;}, 
      TofCommand::PowerCycle              (data) => { value = *data;}, 
      TofCommand::RBSetup                 (data) => { value = *data;}, 
      TofCommand::SetThresholds           (data) => { value = *data;},
      TofCommand::SetMtConfig             (data) => { value = *data;},
      TofCommand::StartValidationRun      (data) => { value = *data;},
      TofCommand::RequestWaveforms        (data) => { value = *data;},
      TofCommand::UnspoolEventCache       (data) => { value = *data;},
      TofCommand::StreamAnyEvent          (data) => { value = *data;},
      TofCommand::StreamOnlyRequested     (data) => { value = *data;},
      TofCommand::DataRunStart            (data) => { value = *data;},
      TofCommand::DataRunEnd              (data) => { value = *data;},
      TofCommand::VoltageCalibration      (data) => { value = *data;},
      TofCommand::TimingCalibration       (data) => { value = *data;},
      TofCommand::CreateCalibrationFile   (data) => { value = *data;},
      TofCommand::RequestEvent            (data) => { value = *data;},
      TofCommand::RequestMoni             (data) => { value = *data;},
      TofCommand::SetRBBuffTrip           (data) => { value = *data;},
      TofCommand::SetRBForcedTrigModeOn   (data) => { value = *data;},
      TofCommand::SetRBForcedTrigModeOff  (data) => { value = *data;},
      TofCommand::SetMTBForcedTrigModeOn  (data) => { value = *data;},
      TofCommand::SetMTBForcedTrigModeOff (data) => { value = *data;},
      TofCommand::Unknown                 (data) => { value = *data;}, 
    }
    value
  }

  /// String representation of the enum
  ///
  /// This is basically the enum type as 
  /// a string.
  pub fn string_repr(&self) -> String { 
    match self {
      TofCommand::Ping                    (_) => {return String::from("Ping");},
      TofCommand::PowerOn                 (_) => {return String::from("PowerOn");},
      TofCommand::PowerOff                (_) => {return String::from("PowerOff");},
      TofCommand::PowerCycle              (_) => {return String::from("PowerCycle");},
      TofCommand::RBSetup                 (_) => {return String::from("RBSetup");}, 
      TofCommand::SetThresholds           (_) => {return String::from("SetThresholds");},
      TofCommand::SetMtConfig             (_) => {return String::from("SetMtConfig");},
      TofCommand::StartValidationRun      (_) => {return String::from("StartValidationRun");},
      TofCommand::RequestWaveforms        (_) => {return String::from("RequestWaveforms");},
      TofCommand::UnspoolEventCache       (_) => {return String::from("UnspoolEventCache");},
      TofCommand::StreamAnyEvent          (_) => {return String::from("StreamAnyEvent");},
      TofCommand::StreamOnlyRequested     (_) => {return String::from("StreamOnlyRequested");},
      TofCommand::DataRunStart            (_) => {return String::from("DataRunStart");}, 
      TofCommand::DataRunEnd              (_) => {return String::from("DataRunEnd");},
      TofCommand::VoltageCalibration      (_) => {return String::from("TimingCalibration");}, 
      TofCommand::TimingCalibration       (_) => {return String::from("TimingCalibration");},
      TofCommand::CreateCalibrationFile   (_) => {return String::from("CreateCalibrationFile");},
      TofCommand::RequestEvent            (_) => {return String::from("RequestEvent");},
      TofCommand::RequestMoni             (_) => {return String::from("RequestMoni");},
      TofCommand::SetRBBuffTrip           (_) => {return String::from("SetRBBuffTrip");},
      TofCommand::SetRBForcedTrigModeOn   (_) => {return String::from("SetRBForcedTrigModeOn");},
      TofCommand::SetRBForcedTrigModeOff  (_) => {return String::from("SetRBForcedTrigModeOff");}
      TofCommand::SetMTBForcedTrigModeOn  (_) => {return String::from("SetMTBForcedTrigModeOn");},
      TofCommand::SetMTBForcedTrigModeOff (_) => {return String::from("SetMTBForcedTrigModeOff");}
      TofCommand::Unknown                 (_) => {return String::from("Unknown");},
      //_                                      => {return String::from("_");}
    }
  }
  

  /// Generate a TofCommand from the specific bytecode
  /// representation
  pub fn from_command_code(cc : u8, value : u32) -> TofCommand {
    match cc {
      CMD_PING                   => TofCommand::Ping                 (value),
      CMD_POFF                   => TofCommand::PowerOff             (value),        
      CMD_PON                    => TofCommand::PowerOn              (value),       
      CMD_PCYCLE                 => TofCommand::PowerCycle           (value),        
      CMD_RBSETUP                => TofCommand::RBSetup              (value),         
      CMD_SETTHRESHOLD           => TofCommand::SetThresholds        (value),         
      CMD_SETMTCONFIG            => TofCommand::SetMtConfig          (value),        
      CMD_DATARUNSTART           => TofCommand::DataRunStart         (value),  
      CMD_DATARUNSTOP            => TofCommand::DataRunEnd           (value),    
      CMD_STARTVALIDATIONRUN     => TofCommand::StartValidationRun   (value),         
      CMD_GETFULLWAVEFORMS       => TofCommand::RequestWaveforms     (value),      
      CMD_REQEUESTEVENT          => TofCommand::RequestEvent         (value), 
      CMS_REQUESTMONI            => TofCommand::RequestMoni          (value),
      CMD_VCALIB                 => TofCommand::VoltageCalibration   (value),       
      CMD_TCALIB                 => TofCommand::TimingCalibration    (value),      
      CMD_CREATECALIBF           => TofCommand::CreateCalibrationFile(value),   
      CMD_UNSPOOL_EVENT_CACHE    => TofCommand::UnspoolEventCache    (value),
      CMD_STREAM_ANY_EVENT       => TofCommand::StreamAnyEvent       (value),
      CMD_STREAM_ONLY_REQUESTED  => TofCommand::StreamOnlyRequested  (value),
      CMD_SET_RB_DATABUF_SIZE    => TofCommand::SetRBBuffTrip        (value),
      CMD_EN_TRIGGERMODE_FORCED  => TofCommand::SetRBForcedTrigModeOn(value),
      CMD_DIS_TRIGGERMODE_FORCED => TofCommand::SetRBForcedTrigModeOff(value),
      CMD_EN_TRIGGERMODE_FORCED_MTB  => TofCommand::SetMTBForcedTrigModeOn(value),
      CMD_DIS_TRIGGERMODE_FORCED_MTB  => TofCommand::SetMTBForcedTrigModeOff(value),
      _                               => TofCommand::Unknown              (value), 
    }
  }
    
  /// Translate a TofCommand into its specific byte representation
  pub fn to_command_code(cmd : &TofCommand) -> Option<u8> {
    match cmd {
      TofCommand::Ping          (_)        => Some(CMD_PING              ),
      TofCommand::PowerOff      (_)        => Some(CMD_POFF              ),        
      TofCommand::PowerOn       (_)        => Some(CMD_PON               ),       
      TofCommand::PowerCycle    (_)        => Some(CMD_PCYCLE            ),        
      TofCommand::RBSetup       (_)        => Some(CMD_RBSETUP           ),         
      TofCommand::SetThresholds (_)        => Some(CMD_SETTHRESHOLD      ),         
      TofCommand::SetMtConfig   (_)        => Some(CMD_SETMTCONFIG       ),        
      TofCommand::DataRunStart  (_)        => Some(CMD_DATARUNSTART       ),  
      TofCommand::DataRunEnd    (_)        => Some(CMD_DATARUNSTOP        ),    
      TofCommand::StartValidationRun   (_) => Some(CMD_STARTVALIDATIONRUN),         
      TofCommand::RequestWaveforms (_)     => Some(CMD_GETFULLWAVEFORMS  ),      
      TofCommand::RequestEvent     (_)     => Some(CMD_REQEUESTEVENT     ), 
      TofCommand::RequestMoni      (_)     => Some(CMS_REQUESTMONI       ), 
      TofCommand::VoltageCalibration  (_)  => Some(CMD_VCALIB            ),       
      TofCommand::TimingCalibration   (_)  => Some(CMD_TCALIB            ),      
      TofCommand::CreateCalibrationFile  (_) => Some(CMD_CREATECALIBF      )    ,
      TofCommand::UnspoolEventCache      (_) => Some(CMD_UNSPOOL_EVENT_CACHE)   ,
      TofCommand::StreamAnyEvent         (_) => Some(CMD_STREAM_ANY_EVENT)      ,
      TofCommand::StreamOnlyRequested    (_) => Some(CMD_STREAM_ONLY_REQUESTED) ,
      TofCommand::SetRBBuffTrip          (_) => Some(CMD_SET_RB_DATABUF_SIZE)   ,
      TofCommand::SetRBForcedTrigModeOn  (_) => Some(CMD_EN_TRIGGERMODE_FORCED) ,
      TofCommand::SetRBForcedTrigModeOff (_) => Some(CMD_DIS_TRIGGERMODE_FORCED),
      TofCommand::SetMTBForcedTrigModeOn  (_) => Some(CMD_EN_TRIGGERMODE_FORCED_MTB) ,
      TofCommand::SetMTBForcedTrigModeOff (_) => Some(CMD_DIS_TRIGGERMODE_FORCED_MTB),
      TofCommand::Unknown                (_) => None                            , 
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

impl From<(u8, u32)> for TofCommand {
  
  /// Generate a TofCommand from a pair of code, value
  ///
  /// The first argument must be the command code, the 
  /// second the specific value of the command.
  fn from(pair : (u8, u32)) -> TofCommand {
    let (input, value) = pair;
    trace!("Got in input {:?}", pair);
    return TofCommand::from_command_code(input, value); 
    //match input {
    //  CMD_PING                => TofCommand::Ping                 (value) ,
    //  CMD_POFF                => TofCommand::PowerOff             (value) ,        
    //  CMD_PON                 => TofCommand::PowerOn              (value) ,       
    //  CMD_PCYCLE              => TofCommand::PowerCycle           (value) ,        
    //  CMD_RBSETUP             => TofCommand::RBSetup              (value) ,         
    //  CMD_SETTHRESHOLD        => TofCommand::SetThresholds        (value) ,         
    //  CMD_SETMTCONFIG         => TofCommand::SetMtConfig          (value) ,        
    //  CMD_DATARUNSTOP         => TofCommand::DataRunEnd            (value),  
    //  CMD_DATARUNSTART        => TofCommand::DataRunStart          (value) ,    
    //  CMD_STARTVALIDATIONRUN  => TofCommand::StartValidationRun    (value),         
    //  CMD_GETFULLWAVEFORMS    => TofCommand::RequestWaveforms      (value) ,      
    //  CMD_REQEUESTEVENT       => TofCommand::RequestEvent          (value) , 
    //  CMS_REQUESTMONI         => TofCommand::RequestMoni           (value),
    //  CMD_VCALIB              => TofCommand::VoltageCalibration    (value),       
    //  CMD_TCALIB              => TofCommand::TimingCalibration     (value),      
    //  CMD_CREATECALIBF        => TofCommand::CreateCalibrationFile (value),   
    //  CMD_UNSPOOL_EVENT_CACHE => TofCommand::UnspoolEventCache   (value) ,
    //  CMD_STREAM_ANY_EVENT    => TofCommand::StreamAnyEvent      (value) ,
    //  CMD_STREAM_ONLY_REQUESTED   => TofCommand::StreamOnlyRequested      (value) ,
    //  _                       => TofCommand::Unknown              (value) , 
    //}
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

  fn from_bytestream(stream    : &Vec<u8>, 
                     pos       : &mut usize) 
    -> Result<Self, SerializationError>{
  
    //let mut pos      = start_pos; 
    let mut two_bytes : [u8;2];
    let four_bytes    : [u8;4];
    two_bytes = [stream[*pos],
                 stream[*pos+1]];
    *pos += 2;
    if Self::HEAD != u16::from_le_bytes(two_bytes) {
      error!("Packet does not start with HEAD signature");
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
    let pair    = (cc, value);
    let command = Self::from(pair);
    if Self::TAIL != u16::from_le_bytes(two_bytes) {
      error!("Packet does not end with TAIL signature");
      return Err(SerializationError::TailInvalid {});
    }
    Ok(command)
  }
}

/// Each `TofCommand` triggers a `TofResponse` in reply
///
/// The responses are general classes, which carry a more
/// specific 32-bit response code.
#[derive(Debug, Copy, Clone, PartialEq)]
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
    let r = self.string_repr();
    //let arg = 
    write!(f, "<TofResponse {}>", r)
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

  pub fn string_repr(&self) -> String {
    let repr : String;
    match self {
      TofResponse::Success           (data) => {repr = "Success(".to_owned()           + &data.to_string() + ")";},
      TofResponse::GeneralFail       (data) => {repr = "GeneralFail(".to_owned()       + &data.to_string() + ")";},
      TofResponse::EventNotReady     (data) => {repr = "EventNotReady(".to_owned()     + &data.to_string() + ")";},
      TofResponse::SerializationIssue(data) => {repr = "SerializationIssue".to_owned() + &data.to_string() + ")";},
      TofResponse::ZMQProblem        (data) => {repr = "ZMQProblem(".to_owned()        + &data.to_string() + ")";},
      TofResponse::Unknown                  => {repr = "Unknown".to_owned();}, 
    }
  repr
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
  type_codes.push(TofOperationMode::UNKNOWN); 
  type_codes.push(TofOperationMode::STREAMANY); 
  type_codes.push(TofOperationMode::REQUESTREPLY); 
  for tc in type_codes.iter() {
    assert_eq!(*tc,TofOperationMode::to_u8(&TofOperationMode::from_u8(tc)));  
  }
}

#[cfg(feature = "random")]
#[test]
fn serialization_rbcommand() {
  let cmd  = RBCommand::from_random();
  let test = RBCommand::from_bytestream(&cmd.to_bytestream(), &mut 0).unwrap();
  assert_eq!(cmd, test);
}
