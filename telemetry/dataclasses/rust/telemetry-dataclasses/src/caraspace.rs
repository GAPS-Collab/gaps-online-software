//! Register TelemetryPacket with the caraspace system

use caraspace::prelude::{
  CRFrameObjectType,
  CRSerializeable,
  CRSerializationError,
  Frameable,
};

use crate::packets::TelemetryPacket;

impl Frameable for TelemetryPacket {
  const CRFRAMEOBJECT_TYPE : CRFrameObjectType = CRFrameObjectType::TelemetryPacket;
}

impl CRSerializeable for TelemetryPacket {

  fn deserialize(stream : &Vec<u8>, pos : &mut usize)
  -> Result<Self, CRSerializationError> {
    match Self::from_bytestream(stream, pos) {
      Err(err) => {
        return Err(CRSerializationError::UnknownError);
      }
      Ok(obj) => {
        return Ok(obj)
      }
    }
  }
  
  fn serialize(&self) 
    -> Vec<u8> {
    self.to_bytestream()
  }
}

