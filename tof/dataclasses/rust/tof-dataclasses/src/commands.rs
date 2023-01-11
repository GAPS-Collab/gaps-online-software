///! Commmands which can be issued
///  to the various components of 
///  the tof system.
///
///
///  Here is a comprehensive list (Sydney)
///  * Power on/off to PBs+RBs+LTBs+preamps (all at once) or MT
///  * Power on/off to LTB or preamp < 2/day Command to power on/off various components (to TOF -> to RB) 5 B:
///  * RBsetup ? Command to run rbsetup on a particular RB (to TOF -> to RBs) 8 B:
///  * Set Thresholds < 3/day Command to set a threshold level on all LTBs (to TOF -> to RBs) 8 B:
///  * Set MT Config 1/run, <10/day? Command to set MT trigger config (to TOF -> to MT) 4 B:
///  * Start Validation Run 1/run, <10/day? Command to take a small amount of data (some number E events, I
///  * 360xE full waveforms (from TOF)
///  
///  * Start Data-Taking Run 1/run, <10/day? Command to take regular data (to TOF -> to RBs)
///  * Reduced data packet (from Flight computer)
///  * Stop Run < 1/run, < 10/day Command to stop a run (to TOF -> to RBs) 2 B = command name 6
///  
///  * Voltage Calibration Runs 1/day Command to take 2 voltage calibration runs (to TOF -> to RBs) 12 B:
///  * Timing Calibration Run 1/day Command to take a timing calibration run (to TOF -> to RBs) 8 B:
///  * Create New Calibration File 1/day Command to create a new calibration file using data from the three
///  





//pub use crate::packets::data_packet::CommandPacket;

use crate::serialization::Serialization;
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
///! Command string (Sydney's commands)
pub const CMD_POFF               : u8 = 10;        
pub const CMD_PON                : u8 = 11;       
pub const CMD_PCYCLE             : u8 = 12;        
pub const CMD_RBSETUP            : u8 = 20;         
pub const CMD_SETTHRESHOLD       : u8 = 21;         
pub const CMD_SETMTCONFIG        : u8 = 22;        
pub const CMD_DATARUNSTOP        : u8 = 30;  
pub const CMD_DATARUNSTART       : u8 = 31;    
pub const CMD_STARTVALIDATIONRUN : u8 = 32;         
pub const CMD_GETFULLWAVEFORMS   : u8 = 41;      
pub const CMD_REQEUESTEVENT      : u8 = 42; 
pub const CMS_REQUESTMONI        : u8 = 43;
pub const CMD_VCALIB             : u8 = 51;       
pub const CMD_TCALIB             : u8 = 52;      
pub const CMD_CREATECALIBF       : u8 = 53;   

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
  DataRunStart , 
  DataRunEnd   ,
  VoltageCalibration,
  TimingCalibration,
  CreateCalibrationFile,
  RequestEvent(u32),
  RequestMoni ,
  Unknown
}

impl TofCommand {
  
  pub fn from_command_code(cc : u8, value : u32) -> TofCommand {
    match cc {
      CMD_POFF               => TofCommand::PowerOff             (value) ,        
      CMD_PON                => TofCommand::PowerOn              (value) ,       
      CMD_PCYCLE             => TofCommand::PowerCycle           (value) ,        
      CMD_RBSETUP            => TofCommand::RBSetup              (value) ,         
      CMD_SETTHRESHOLD       => TofCommand::SetThresholds        (value) ,         
      CMD_SETMTCONFIG        => TofCommand::SetMtConfig          (value) ,        
      CMD_DATARUNSTOP        => TofCommand::DataRunStart          ,  
      CMD_DATARUNSTART       => TofCommand::DataRunEnd            ,    
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
      TofCommand::DataRunStart             => Some(CMD_DATARUNSTOP       ),  
      TofCommand::DataRunEnd               => Some(CMD_DATARUNSTART      ),    
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
}


