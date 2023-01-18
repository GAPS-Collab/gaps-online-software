//! Wrap commands to be send over the wire
//
//

use crate::serialization::Serialization;
use crate::errors::SerializationError;
use crate::commands::TofCommand;


#[deprecated(since = "0.2.0", note = "There is no need for a packet since TofCommand has Serialization trait ")]
#[derive(Debug, PartialEq)]
pub struct CommandPacket {
  pub command : TofCommand,
  pub value   : u32
}

impl CommandPacket {
  const HEAD : u16 = 0xAAAA;
  const TAIL : u16 = 0x5555;

  pub fn new() -> CommandPacket {
    CommandPacket {
      command : TofCommand::Unknown(0),
      value   : 0
    }
  }

  pub fn to_bytestream(&self) -> Vec<u8> {
    let mut bytestream = Vec::<u8>::with_capacity(9);
    bytestream.extend_from_slice(&CommandPacket::HEAD.to_le_bytes());
    let cc = TofCommand::to_command_code(&self.command);
    match cc {
      None => bytestream.push(0),
      Some(code) => bytestream.push(code)
    }
    bytestream.extend_from_slice(&self.value.to_le_bytes());
    bytestream.extend_from_slice(&CommandPacket::TAIL.to_le_bytes());
    bytestream
  }
}

impl Serialization for CommandPacket {

  fn from_bytestream(stream : &Vec<u8>, 
                     start_pos : usize) 
    -> Result<CommandPacket, SerializationError>{
  
    let mut pos      = start_pos; 
    let mut two_bytes : [u8;2];
    let four_bytes    : [u8;4];
    two_bytes = [stream[pos],
                 stream[pos+1]];
    pos += 2;
    if CommandPacket::HEAD != u16::from_le_bytes(two_bytes) {
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
    let command = TofCommand::from_command_code(cc, value);
    if CommandPacket::TAIL != u16::from_le_bytes(two_bytes) {
      warn!("Packet does not end with TAIL signature");
      return Err(SerializationError::TailInvalid {});
    }
    Ok(CommandPacket{
        command, 
        value
    })   
  }
}


