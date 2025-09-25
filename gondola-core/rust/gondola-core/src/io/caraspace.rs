//! The following file is part of gaps-online-software and published 
//! under the GPLv3 license

mod reader; 
pub use reader::CRReader;
pub mod writer;
pub use writer::CRWriter;

pub mod frame;
pub use frame::{
  CRFrameObjectType,
  CRFrameObject,
  CRFrame,
};

use crate::prelude::Serialization;

/// Allows to pack a certain structure within 
/// a CRFrameObject
pub trait Frameable {
  const CRFRAMEOBJECT_TYPE : CRFrameObjectType;

  /// Wrap myself in a CRFrameObject
  fn pack(&self) -> CRFrameObject 
    where Self: Serialization {
    let mut cr     = CRFrameObject::new();
    cr.payload     = self.to_bytestream();
    cr.ftype       = Self::CRFRAMEOBJECT_TYPE;
    //cr.size        = cr.payload.len();
    cr
  }
}

