//! Re-implementation of the tracker header which is attached 
//! to each telemetry packet.
//! Re-implemented from bfsw
// The following file is part of gaps-online-software and published 
// under the GPLv3 license

use crate::prelude::*;

/// A header 
#[derive(Clone)]
#[cfg_attr(feature="pybindings", pyclass)]
pub struct TrackerHeader {
  pub sync        : u16,
  pub crc         : u16,
  pub sys_id      : u8,
  pub packet_id   : u8,
  pub length      : u16,
  pub daq_count   : u16,
  pub sys_time    : u64,
  pub version     : u8,
} 

impl TrackerHeader {
  
  pub fn new() -> Self {
    Self {
      sync        : 0,
      crc         : 0,
      sys_id      : 0,
      packet_id   : 0,
      length      : 0,
      daq_count   : 0,
      sys_time    : 0,
      version     : 0,
    }
  }
} 

impl Serialization for TrackerHeader { 
  const SIZE : usize = 17;

  fn from_bytestream(stream: &Vec<u8>,
                     pos: &mut usize)
    -> Result<Self, SerializationError> {
    if stream.len() <= Self::SIZE {
      error!("Unable to decode TrackerHeader!"); 
      return Err(SerializationError::StreamTooShort);
    }
    let mut h     = TrackerHeader::new();
    h.sync        = parse_u16(stream, pos);
    h.crc         = parse_u16(stream, pos); 
    h.sys_id      = parse_u8 (stream, pos);
    h.packet_id   = parse_u8 (stream, pos);
    h.length      = parse_u16(stream, pos);
    h.daq_count   = parse_u16(stream, pos);
    let lower     = parse_u32(stream, pos);
    let upper     = parse_u16(stream, pos);
    h.sys_time    = make_systime(lower, upper);
    h.version     = parse_u8 (stream, pos);
    Ok(h)
  }
}

impl fmt::Display for TrackerHeader {
  fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
    let mut repr = String::from("<TrackerHeader");
    repr    += &(format!("\n  Sync     : {}", self.sync));
    repr    += &(format!("\n  Crc      : {}", self.crc));
    repr    += &(format!("\n  PacketID : {}", self.packet_id));
    repr    += &(format!("\n  Length   : {}", self.length));
    repr    += &(format!("\n  DAQ Cnt  : {}", self.daq_count));
    repr    += &(format!("\n  Sys Time : {}", self.sys_time));
    repr    += &(format!("\n  Version  : {}>", self.version));
    write!(f, "{}", repr)
  }
}

//------------------------------------------------------

#[cfg(feature="pybindings")]
#[pymethods]
impl TrackerHeader { 

  #[getter]
  fn get_sync(&self)        -> u16 {
    self.sync
  }
  
  #[getter]
  fn get_crc(&self)         -> u16 {
    self.crc
  }
  
  #[getter]
  fn get_sys_id(&self)      -> u8 {
    self.sys_id
  }
  
  #[getter]
  fn get_packet_id(&self)   -> u8 {
    self.packet_id
  }
  
  #[getter]
  fn get_length(&self)      -> u16 {
    self.length
  }
  
  #[getter]
  fn get_daq_count(&self)   -> u16 {
    self.daq_count
  }
  
  #[getter]
  fn get_sys_time(&self)    -> u64 {
    self.sys_time
  }
  
  #[getter]
  fn get_version(&self)     -> u8 {
    self.version
  }
}



#[cfg(feature="pybindings")]
pythonize!(TrackerHeader);

