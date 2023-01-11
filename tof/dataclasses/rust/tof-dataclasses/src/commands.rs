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





pub use crate::packets::data_packet::CommandPacket;

use crate::serialization::Serialization;
use crate::packets::{TofPacket,
                     PacketType};


///! Command string (Sydney's commands
pub const CMD_PON                : &'static str = "CMD::PON";       
pub const CMD_POFF               : &'static str = "CMD::POFF";        
pub const CMD_PCYCLE             : &'static str = "CMD::PCYCLE";        
pub const CMD_RBSETUP            : &'static str = "CMD::RBSETUP";         
pub const CMD_SETTHRESHOLD       : &'static str = "CMD::SETTHR";         
pub const CMD_SETMTCONFIG        : &'static str = "CMD::SETMTCONF";        
pub const CMD_STARTVALIDATIONRUN : &'static str = "CMD::STARTVRUN";         
pub const CMD_GETFULLWAVEFORMS   : &'static str = "CMD::GETWF";      
pub const CMD_DATARUNSTART       : &'static str = "CMD::DATARUNSTART";    
pub const CMD_REQEUESTEVENT      : &'static str = "CMD::REQUESTEVENT";      
pub const CMD_DATARUNSTOP        : &'static str = "CMD::DATARUNSTOP";  
pub const CMD_VCALIB             : &'static str = "CMD::VCALIB";       
pub const CMD_TCALIB             : &'static str = "CMD::TCALIB";      
pub const CMD_CREATECALIBF       : &'static str = "CMD::CREATECFILE";   


pub enum TofCommand {
  PowerOn(u32),
  PowerOff(u32),
  PowerCycle(u32),
  RBSetup(u32), 
  SetThresholds(u32),
  SetMtConfig(u32),
  StartValidationRun(u32),
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
  pub fn from_command_packet(packet : &CommandPacket)
    -> Option<TofCommand> {
    if !packet.label.starts_with("CMD::") {
      return None;
    }
    match packet.label.as_str() {
      CMD_PON                => Some(TofCommand::PowerOn           (packet.data)), 
      CMD_POFF               => Some(TofCommand::PowerOff          (packet.data)), 
      CMD_PCYCLE             => Some(TofCommand::PowerCycle        (packet.data)), 
      CMD_RBSETUP            => Some(TofCommand::RBSetup           (packet.data)),  
      CMD_SETTHRESHOLD       => Some(TofCommand::SetThresholds     (packet.data)), 
      CMD_SETMTCONFIG        => Some(TofCommand::SetMtConfig       (packet.data)),   
      CMD_STARTVALIDATIONRUN => Some(TofCommand::StartValidationRun(packet.data)),    
      CMD_GETFULLWAVEFORMS   => Some(TofCommand::RequestWaveforms  (packet.data)), 
      CMD_DATARUNSTART       => Some(TofCommand::DataRunStart),  
      CMD_REQEUESTEVENT      => Some(TofCommand::RequestEvent      (packet.data)),    
      CMD_DATARUNSTOP        => Some(TofCommand::DataRunEnd), 
      CMD_VCALIB             => Some(TofCommand::VoltageCalibration), 
      CMD_TCALIB             => Some(TofCommand::TimingCalibration), 
      CMD_CREATECALIBF       => Some(TofCommand::CreateCalibrationFile), 
      _                => Some(TofCommand::Unknown     ) 
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
        TofCommand::from_command_packet(&cmd)

      }
    } // end match
  }

  //////! Get the command from a value packet
  //////
  //////  In case the value packet contains something
  //////  else, return None
  /////pub fn from_generic_packet(packet : &GenericPacket) 
  /////  -> Option<TofCommand> {
  /////  let data = CommandPacket::from_vp(packet)?;
  /////  let command = TofCommand::from_command_packet(&data);
  /////  command
  /////}

}


