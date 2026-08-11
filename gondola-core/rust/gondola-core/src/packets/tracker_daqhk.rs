// This file is part of gaps-online-software and published 
// under the GPLv3 license

use crate::prelude::*;

//---------------------------------------------------

#[cfg_attr(feature="pybindings", pyclass)]
pub struct TrackerDAQHSKPacket {
  pub telemetry_header : TelemetryPacketHeader,
  pub tracker_header   : TrackerHeader,
  pub temp             : [u16;12],
}

impl fmt::Display for TrackerDAQHSKPacket {
  fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
    let mut repr = String::from("<TrackerDAQHSKPacket");
    repr    += &(format!("\n {}", self.telemetry_header));
    repr    += &(format!("\n {}", self.tracker_header));
    repr    += "\n*** TEMP ***";
    repr    += &(format!("\n {:?}>", self.temp));
    write!(f, "{}", repr)
  }
}

impl TrackerDAQHSKPacket {
  pub fn new() -> Self {
    Self {
      telemetry_header : TelemetryPacketHeader::new(),
      tracker_header   : TrackerHeader::new(),
      temp             : [0;12]
    }
  }
}

impl Serialization for TrackerDAQHSKPacket {
  fn from_bytestream(stream: &Vec<u8>,
                     pos: &mut usize)
    -> Result<Self, SerializationError> {
    let mut tp          = TrackerDAQHSKPacket::new();
    tp.tracker_header   = TrackerHeader::from_bytestream(stream, pos)?;
    if tp.tracker_header.packet_id != 0xff {
      error!("This is not a TrackerDAQHSKPacket, but has packet_id {} instead!", tp.tracker_header.packet_id);
      return Err(SerializationError::IncorrectPacketType);
    }
    if stream.len() == *pos as usize {
      error!("Packet contains only header!");
      return Ok(tp);
    }
    //if stream.len() - *pos < (36*3 + 1) {
    //  return Err(SerializationError::StreamTooShort);
    //}
    // this is hack, since the TreckerHeader in this packet does not have a 
    // version (-> Alex) 
    *pos += 193; // skip a bunch of other stuff right now (Alex)
    for k in 0..12usize {
      tp.temp[k]   = parse_u16(stream, pos);
    }
    Ok(tp)
  }
}

//---------------------------------------------------

#[cfg(feature="pybindings")]
#[pymethods]
impl TrackerDAQHSKPacket {
  
  #[getter]
  fn get_temp(&self) -> [u16;12] {
    self.temp
  }
}

#[cfg(feature="pybindings")]
pythonize_telemetry!(TrackerDAQHSKPacket);

