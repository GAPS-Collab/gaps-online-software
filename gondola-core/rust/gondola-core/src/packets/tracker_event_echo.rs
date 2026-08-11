// This file is part of gaps-online-software and published 
// under the GPLv3 license

use crate::prelude::*;



#[cfg_attr(feature="pybindings", pyclass)]
pub struct TrackerEventIDEchoPacket {
  pub telemetry_header : TelemetryPacketHeader,
  pub tracker_header   : TrackerHeader,
  pub temp             : [u16;12],
  pub event_id         : u32,
  pub event_id_errors  : u16,
}

impl TrackerEventIDEchoPacket {
  pub fn new() -> Self {
    Self {
      telemetry_header : TelemetryPacketHeader::new(),
      tracker_header   : TrackerHeader::new(),
      temp             : [0;12],
      event_id         : 0,
      event_id_errors  : 0,
    }
  }
}

impl fmt::Display for TrackerEventIDEchoPacket {
  fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
    let mut repr = String::from("<TrackerEventIDEchoPacket");
    repr    += &(format!("\n {}", self.telemetry_header));
    repr    += &(format!("\n {}", self.tracker_header));
    repr    += "\n*** TEMP ***";
    repr    += &(format!("\n {:?}>", self.temp));
    write!(f, "{}", repr)
  }
}

impl Serialization for TrackerEventIDEchoPacket {

  fn from_bytestream(stream: &Vec<u8>,
                         pos: &mut usize)
    -> Result<Self, SerializationError> {
    let mut tp          = TrackerEventIDEchoPacket::new();
    tp.tracker_header   = TrackerHeader::from_bytestream(stream, pos)?;
    if tp.tracker_header.packet_id != 0x03 {
      error!("This is not a TrackerEventIDEchoPacket, but has packet_id {} instead!", tp.tracker_header.packet_id);
      return Err(SerializationError::IncorrectPacketType);
    }
    if stream.len() == *pos as usize {
      error!("Packet contains only header!");
      return Ok(tp);
    }
    //if stream.len() - *pos < (36*3 + 1) {
    //  return Err(SerializationError::StreamTooShort);
    //}
    tp.event_id        = parse_u32(stream, pos);
    tp.event_id_errors = parse_u16(stream, pos);
    Ok(tp)
  }
}

#[cfg(feature="pybindings")]
#[pymethods]
impl TrackerEventIDEchoPacket {


  #[getter]
  fn get_temp(&self) -> [u16;12] {
    self.temp
  }
}

#[cfg(feature="pybindings")]
pythonize_telemetry!(TrackerEventIDEchoPacket);

