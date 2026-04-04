// The following file is part of gaps-online-software and published 
// under the GPLv3 license

use crate::prelude::*;

#[derive(Debug, Clone, PartialEq)]
#[cfg_attr(feature="pybindings", pyclass)] 
pub struct McEvent {
  pub run_id   : u32,
  pub event_id : u32,
  pub hits     : Vec<McHit>
}

impl McEvent {

  pub fn new() -> Self {
    Self {
      run_id   : 0,
      event_id : 0,
      hits     : Vec::<McHit>::new(),
    }
  }
}

impl Serialization for McEvent {
  fn from_bytestream(stream : &Vec<u8>,
                     pos    : &mut usize) 
    -> Result<Self, SerializationError> {
    let mut event = Self::new();
    let head      = parse_u16(stream, pos);
    if head != Self::HEAD {
      error!("Decoding of HEAD failed! Got {} instead!", head);
      return Err(SerializationError::HeadInvalid);
    }
    event.run_id   = parse_u32(stream, pos);
    event.event_id = parse_u32(stream, pos);
    let nhits      = parse_u16(stream, pos);
    for _ in 0..nhits {
      event.hits.push(McHit::from_bytestream(stream, pos)?);
    }
    let tail = parse_u16(stream, pos);
    if tail != Self::TAIL {
      //error!("Decoding of TAIL failed for version {}! Got {} instead!", version, tail);
      error!("Decoding of TAIL failed! Got {} instead!", tail);
      return Err(SerializationError::TailInvalid);
    }
    Ok(event) 
  }
}

impl fmt::Display for McEvent {
  fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
    let mut repr = format!("<McEvent ");
    repr += &(format!("\n run_id : {}",self.run_id));
    repr += &(format!("\n run_id : {}",self.event_id));
    repr += "\n === HITS ===";
    for h in &self.hits {
      repr += &(format!("\n {}", h));
    }
    repr += ">";
    write!(f, "{}", repr)
  }
}

impl Frameable for McEvent {
  const CRFRAMEOBJECT_TYPE : CRFrameObjectType = CRFrameObjectType::McTree;
}

#[cfg(feature="pybindings")]
pythonize!(McEvent);

