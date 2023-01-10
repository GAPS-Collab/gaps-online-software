///! Commmands which can be issued
///  to the various components of 
///  the tof system.
use crate::packets::data_packet::CommandPacket;
use crate::packets::generic_packet::GenericPacket;

///! Command strings
const CMD_COLDRESTART   : &'static str = "CMD::COLDRESTART";
const CMD_WARMRESTART   : &'static str = "CMD::WARMRESTART";
const CMD_DATARUNSTART  : &'static str = "CMD::DATARUNSTART";
const CMD_DATARUNSTOP   : &'static str = "CMD::DATARUNSTOP";
const CMD_REQUESTEVENT  : &'static str = "CMD::REQUESTEVENT";
const CMD_REQUESTMONI   : &'static str = "CMD::REQUESTMONI";
const CMD_CALIBRATE     : &'static str = "CMD::CALIBRATE";



pub enum TofCommand {
  ColdRestart  ,
  WarmRestart  ,
  Calibrate    ,
  DataRunStart , 
  DataRunEnd   ,
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
      CMD_COLDRESTART  => Some(TofCommand::ColdRestart ) ,
      CMD_WARMRESTART  => Some(TofCommand::WarmRestart ) ,
      CMD_CALIBRATE    => Some(TofCommand::Calibrate   ) ,
      CMD_DATARUNSTART => Some(TofCommand::DataRunStart) , 
      CMD_DATARUNSTOP  => Some(TofCommand::DataRunEnd  ) ,
      CMD_REQUESTEVENT => Some(TofCommand::RequestEvent(packet.data)) ,
      CMD_REQUESTMONI  => Some(TofCommand::RequestMoni ) ,
      _                => Some(TofCommand::Unknown     ) 
    }
  }

  ///! Get the command from a value packet
  ///
  ///  In case the value packet contains something
  ///  else, return None
  pub fn from_generic_packet(packet : &GenericPacket) 
    -> Option<TofCommand> {
    let data = CommandPacket::from_vp(packet)?;
    let command = TofCommand::from_command_packet(&data);
    command
  }

}


