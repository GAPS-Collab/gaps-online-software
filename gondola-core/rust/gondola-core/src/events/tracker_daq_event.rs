// This file is part of gaps-online-software and published 
// under the GPLv3 license

use crate::prelude::*;

/// Each individual DAQ box of the tracker records its hit individually 
/// and then the event needs to be assemlbed. 
///
/// During flight, this will be part of the "TrackerPacket" type 80
#[derive(Debug, Clone, PartialEq)]
#[cfg_attr(feature="pybindings", pyclass)]
pub struct TrackerDAQEvent {
  pub layer        : u8,
  pub flags1       : u8,
  pub event_id     : u32, 
  pub event_time16 : u16,
  pub event_time32 : u32,
  pub hits         : Vec<TrackerHit>
}

impl TrackerDAQEvent {
  pub fn new() -> Self {
    Self {
      layer        : u8::MAX,
      flags1       : u8::MAX,
      event_id     : u32::MAX, 
      event_time16 : u16::MAX,
      event_time32 : u32::MAX,
      hits         : Vec::<TrackerHit>::new()
    }
  }
  
  pub fn get_event_time(&self) -> u64 {
    //0x273000000000000 | (((self.event_time16 as u64) << 32) | self.event_time32 as u64)
    ((self.event_time16 as u64) << 32) | self.event_time32 as u64
  }
}

impl fmt::Display for TrackerDAQEvent {
  fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
    let mut repr = String::from("<TrackerDAQEvent:");
    repr += &(format!("\n  Layer      : {}" ,self.layer));
    repr += &(format!("\n  Flags1     : {}" ,self.flags1));
    repr += &(format!("\n  Event ID   : {}" ,self.event_id)); 
    repr += &(format!("\n  Event Time : {}" ,self.get_event_time())); 
    if self.hits.len() > 0 {
      repr += "\n -- hits:"; 
      for h in &self.hits {
        repr += &(format!("\n {}", h));
      }
    } else {
      repr += "\n -- no hits!"; 
    }
    write!(f, "{}", repr)
  }
}


impl Serialization for TrackerDAQEvent { 
  fn from_bytestream(stream: &Vec<u8>,
                     pos: &mut usize)
    -> Result<Self, SerializationError> {
    let mut ev  = Self::new();
    ev.event_time32 = parse_u32(stream, pos);
    ev.event_time16 = parse_u16(stream, pos);
    ev.layer        = parse_u8(stream, pos);
    let n_hits      = parse_u8(stream, pos);
    for _ in 0..n_hits {
      // in this version of the TrackerHit, it has 
      // 6 bytes
      if stream.len() < *pos + 6 {
        error!("Expected to get 6 more bytes for the hit, but the input stream is too short!");
        return Err(SerializationError::StreamTooShort);
      }
      let mut h = TrackerHit::new();
      h.row             = parse_u8(stream, pos) as u16;
      h.module          = parse_u8(stream, pos) as u16;
      h.channel         = parse_u8(stream, pos) as u16;
      h.adc             = parse_u16(stream, pos);
      h.asic_event_code = parse_u8(stream, pos);
      ev.hits.push(h);
    }
    Ok(ev)
  }
}

#[cfg(feature="pybindings")]
#[pymethods]
impl TrackerDAQEvent {

  #[getter]
  fn get_layer(&self) -> u8 {
    self.layer 
  }

  #[getter] 
  fn get_flags1(&self) -> u8 {
    self.flags1 
  }

  #[getter]
  fn get_event_id(&self) -> u32 {
    self.event_id
  }

  #[getter] 
  fn get_hits(&self) -> Vec<TrackerHit> {
    self.hits.clone()
  }
 
  #[getter]
  #[pyo3(name="event_time")]
  fn get_event_time_py(&self) -> u64 {
    self.get_event_time()
  }
}

#[cfg(feature="pybindings")]
pythonize!(TrackerDAQEvent);

