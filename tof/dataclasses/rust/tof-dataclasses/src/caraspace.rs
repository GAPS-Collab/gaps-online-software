//! Register TofPacket with the caraspace system

use caraspace::prelude::{
  CRFrameObjectType,
  CRSerializeable,
  CRSerializationError,
  Frameable,
};

use crate::packets::TofPacket;
use crate::serialization::Serialization;

impl Frameable for TofPacket {
  const CRFRAMEOBJECT_TYPE : CRFrameObjectType = CRFrameObjectType::TofPacket;
}

impl CRSerializeable for TofPacket {

  fn deserialize(stream : &Vec<u8>, pos : &mut usize)
  -> Result<Self, CRSerializationError> {
    match Self::from_bytestream(stream, pos) {
      Err(_err) => {
        // FIXME - that should get better. Maybe we want
        // to unify CRSerializationError and SerializationError?
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

