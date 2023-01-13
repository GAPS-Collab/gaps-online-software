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
use crate::serialization::{Serialization, SerializationError};
use crate::packets::{TofPacket,
                     CommandPacket,
                     PacketType};


//pub const CMD_PON                : &'static str = "CMD::PON";       
//pub const CMD_POFF               : &'static str = "CMD::POFF";        
//pub const CMD_PCYCLE             : &'static str = "CMD::PCYCLE";        
//pub const CMD_RBSETUP            : &'static str = "CMD::RBSETUP";         
//pub const CMD_SETTHRESHOLD       : &'static str = "CMD::SETTHR";         
//pub const CMD_SETMTCONFIG        : &'static str = "CMD::SETMTCONF";        
//pub const CMD_STARTVALIDATIONRUN : &'static str = "CMD::STARTVRUN";         
//pub const CMD_GETFULLWAVEFORMS   : &'static str = "CMD::GETWF";      
//pub const CMD_DATARUNSTART       : &'static str = "CMD::DATARUNSTART";    
//pub const CMD_REQEUESTEVENT      : &'static str = "CMD::REQUESTEVENT";      
//pub const CMD_DATARUNSTOP        : &'static str = "CMD::DATARUNSTOP";  
//pub const CMD_VCALIB             : &'static str = "CMD::VCALIB";       
//pub const CMD_TCALIB             : &'static str = "CMD::TCALIB";      
//pub const CMD_CREATECALIBF       : &'static str = "CMD::CREATECFILE";   



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

// FIXME - these commands need to be implemented
/// NEEDTOIMPLEMENT: command code for "Send the whole event cache over the wire"
pub const CMD_UNSPOOL_EVENT_CACHE : u8 = 44;
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


/// General command class for ALL commands to the 
/// tof C&C instance and readout boards
///
/// Each command can carry a 32bit field with further
/// instructionns
///
#[derive(Debug, PartialEq)]
pub enum TofCommand {
  PowerOn(u32),
  PowerOff(u32),
  PowerCycle(u32),
  RBSetup(u32), 
  SetThresholds(u32),
  SetMtConfig(u32),
  StartValidationRun,
  RequestWaveforms(u32),
  /// Start a new run, the argument being the number 
  /// of events.
  DataRunStart(u32), 
  DataRunEnd   ,
  VoltageCalibration,
  TimingCalibration,
  CreateCalibrationFile,
  /// Request event data for a specific event being sent
  /// over the data wire. The argument being the event id.
  RequestEvent(u32),
  RequestMoni ,
  Unknown
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

  fn from_bytestream(stream    : &Vec<u8>, 
                     start_pos : usize) 
    -> Result<TofResponse, SerializationError>{
  
    let mut pos      = start_pos; 
    let mut two_bytes : [u8;2];
    let four_bytes    : [u8;4];
    two_bytes = [stream[pos],
                 stream[pos+1]];
    pos += 2;
    if TofResponse::HEAD != u16::from_le_bytes(two_bytes) {
      warn!("Packet does not start with HEAD signature");
      return Err(SerializationError::HeadInvalid {});
    }
   
    let cc   = stream[pos];
    pos += 1;
    four_bytes = [stream[pos],
                  stream[pos+1],
                  stream[pos+2],
                  stream[pos+3]];
    pos += 4;
    let value = u32::from_le_bytes(four_bytes);
    two_bytes = [stream[pos],
                 stream[pos+1]];
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

impl TofCommand {
  
  const HEAD : u16 = 0xAAAA;
  const TAIL : u16 = 0x5555;
  
  pub fn from_command_code(cc : u8, value : u32) -> TofCommand {
    match cc {
      CMD_POFF               => TofCommand::PowerOff             (value) ,        
      CMD_PON                => TofCommand::PowerOn              (value) ,       
      CMD_PCYCLE             => TofCommand::PowerCycle           (value) ,        
      CMD_RBSETUP            => TofCommand::RBSetup              (value) ,         
      CMD_SETTHRESHOLD       => TofCommand::SetThresholds        (value) ,         
      CMD_SETMTCONFIG        => TofCommand::SetMtConfig          (value) ,        
      CMD_DATARUNSTART       => TofCommand::DataRunStart         (value) ,  
      CMD_DATARUNSTOP        => TofCommand::DataRunEnd            ,    
      CMD_STARTVALIDATIONRUN => TofCommand::StartValidationRun    ,         
      CMD_GETFULLWAVEFORMS   => TofCommand::RequestWaveforms     (value) ,      
      CMD_REQEUESTEVENT      => TofCommand::RequestEvent         (value) , 
      CMS_REQUESTMONI        => TofCommand::RequestMoni           ,
      CMD_VCALIB             => TofCommand::VoltageCalibration    ,       
      CMD_TCALIB             => TofCommand::TimingCalibration     ,      
      CMD_CREATECALIBF       => TofCommand::CreateCalibrationFile ,   
      _                      => TofCommand::Unknown               , 
    }
  }
    
  pub fn to_command_code(cmd : &TofCommand) -> Option<u8> {
    match cmd {
      TofCommand::PowerOff      (_)        => Some(CMD_POFF              ),        
      TofCommand::PowerOn       (_)        => Some(CMD_PON               ),       
      TofCommand::PowerCycle    (_)        => Some(CMD_PCYCLE            ),        
      TofCommand::RBSetup       (_)        => Some(CMD_RBSETUP           ),         
      TofCommand::SetThresholds (_)        => Some(CMD_SETTHRESHOLD      ),         
      TofCommand::SetMtConfig   (_)        => Some(CMD_SETMTCONFIG       ),        
      TofCommand::DataRunStart  (_)        => Some(CMD_DATARUNSTART       ),  
      TofCommand::DataRunEnd               => Some(CMD_DATARUNSTOP      ),    
      TofCommand::StartValidationRun       => Some(CMD_STARTVALIDATIONRUN),         
      TofCommand::RequestWaveforms (_)     => Some(CMD_GETFULLWAVEFORMS  ),      
      TofCommand::RequestEvent     (_)     => Some(CMD_REQEUESTEVENT     ), 
      TofCommand::RequestMoni              => Some(CMS_REQUESTMONI       ), 
      TofCommand::VoltageCalibration       => Some(CMD_VCALIB            ),       
      TofCommand::TimingCalibration        => Some(CMD_TCALIB            ),      
      TofCommand::CreateCalibrationFile    => Some(CMD_CREATECALIBF      ),   
      TofCommand::Unknown                  => None                  , 
    }
  }

  pub fn from_tof_packet(packet : &TofPacket) 
    -> Option<TofCommand> {
    match packet.packet_type {
      PacketType::Command => (),
      _ => {
        debug!("Packet has not packet type CMD");
        return None;
        }
    } // end match
    let cmd_pk = CommandPacket::from_bytestream(&packet.payload, 0); 
    match cmd_pk {
      Err(err) => {
        debug!("Could not decode CMD packet, err {:?}", err);
        return None;
      }
      Ok(cmd) => {
        Some(cmd.command) 
      }
    } // end match
  }
} // end impl TofCommand

impl From<(u8, u32)> for TofCommand {
  fn from(pair : (u8, u32)) -> TofCommand {
    let (input, value) = pair;
    trace!("Got in input {:?}", pair);
    match input {
      CMD_POFF               => TofCommand::PowerOff             (value) ,        
      CMD_PON                => TofCommand::PowerOn              (value) ,       
      CMD_PCYCLE             => TofCommand::PowerCycle           (value) ,        
      CMD_RBSETUP            => TofCommand::RBSetup              (value) ,         
      CMD_SETTHRESHOLD       => TofCommand::SetThresholds        (value) ,         
      CMD_SETMTCONFIG        => TofCommand::SetMtConfig          (value) ,        
      CMD_DATARUNSTOP        => TofCommand::DataRunEnd            ,  
      CMD_DATARUNSTART       => TofCommand::DataRunStart         (value) ,    
      CMD_STARTVALIDATIONRUN => TofCommand::StartValidationRun    ,         
      CMD_GETFULLWAVEFORMS   => TofCommand::RequestWaveforms     (value) ,      
      CMD_REQEUESTEVENT      => TofCommand::RequestEvent         (value) , 
      CMS_REQUESTMONI        => TofCommand::RequestMoni           ,
      CMD_VCALIB             => TofCommand::VoltageCalibration    ,       
      CMD_TCALIB             => TofCommand::TimingCalibration     ,      
      CMD_CREATECALIBF       => TofCommand::CreateCalibrationFile ,   
      _                      => TofCommand::Unknown               , 
    }
  }
}

impl Serialization for TofCommand {

  fn from_bytestream(stream    : &Vec<u8>, 
                     start_pos : usize) 
    -> Result<TofCommand, SerializationError>{
  
    let mut pos      = start_pos; 
    let mut two_bytes : [u8;2];
    let four_bytes    : [u8;4];
    two_bytes = [stream[pos],
                 stream[pos+1]];
    pos += 2;
    if TofCommand::HEAD != u16::from_le_bytes(two_bytes) {
      warn!("Packet does not start with HEAD signature");
      return Err(SerializationError::HeadInvalid {});
    }
   
    let cc   = stream[pos];
    pos += 1;
    four_bytes = [stream[pos],
                  stream[pos+1],
                  stream[pos+2],
                  stream[pos+3]];
    pos += 4;
    let value = u32::from_le_bytes(four_bytes);
    two_bytes = [stream[pos],
                 stream[pos+1]];
    let pair    = (cc, value);
    let command = TofCommand::from(pair);
    if TofCommand::TAIL != u16::from_le_bytes(two_bytes) {
      warn!("Packet does not end with TAIL signature");
      return Err(SerializationError::TailInvalid {});
    }
    Ok(command)
  }
}

