// This file is part of gaps-online-software and published 
// under the GPLv3 license

use crate::prelude::*;

//---------------------------------------------------

#[cfg_attr(feature="pybindings", pyclass)]
pub struct TrackerDAQTempPacket {
  pub telemetry_header : TelemetryPacketHeader,
  pub tracker_header   : TrackerHeader,
  pub rom_id           : [u64;256],
  pub temp             : [u16;256]
}

impl fmt::Display for TrackerDAQTempPacket {
  fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
    let mut repr = String::from("<TrackerDAQTempPacket");
    repr    += &(format!("\n {}", self.telemetry_header));
    repr    += &(format!("\n {}", self.tracker_header));
    repr    += "\n*** ROM ID ***";
    repr  += &(format!("\n {:?}", self.rom_id));
    repr    += "\n*** TEMP ***";
    repr  += &(format!("\n {:?}>", self.temp));
    write!(f, "{}", repr)
  }
}

impl TrackerDAQTempPacket {
  pub fn new() -> Self {
    Self {
      telemetry_header : TelemetryPacketHeader::new(),
      tracker_header   : TrackerHeader::new(),
      rom_id           : [0;256],
      temp             : [0;256]
    }
  }
}

impl Serialization for TrackerDAQTempPacket {
  fn from_bytestream(stream: &Vec<u8>,
                     pos: &mut usize)
    -> Result<Self, SerializationError> {
    let mut tp          = TrackerDAQTempPacket::new();
    tp.tracker_header   = TrackerHeader::from_bytestream(stream, pos)?;
    if tp.tracker_header.packet_id != 0x09 {
      error!("This is not a TrackerDAQTempPacket, but has packet_id {} instead!", tp.tracker_header.packet_id);
      return Err(SerializationError::IncorrectPacketType);
    }
    debug!("tracker header {}", tp.tracker_header);
    if stream.len() == *pos as usize {
      error!("Packet contains only header!");
      return Ok(tp);
    }
    //if stream.len() - *pos < (36*3 + 1) {
    //  return Err(SerializationError::StreamTooShort);
    //}
    // this is hack, since the TreckerHeader in this packet does not have a 
    // version (-> Alex) 
    *pos -= 1;
    let dummy64 = 0u64;
    let dummy16 = 0u16;
    error!("{}", tp.tracker_header);
    error!("Expected of the packet {}", (tp.tracker_header.length as usize)/2);
    for k in 0..256usize {
      if k < (tp.tracker_header.length as usize)/2 {
        tp.rom_id[k] = parse_u64(stream, pos);
        tp.temp[k]   = parse_u16(stream, pos);
      } else {
        tp.rom_id[k] = dummy64;
        tp.temp[k]   = dummy16;
      }
    }
    Ok(tp)
  }
}

//---------------------------------------------------

#[cfg(feature="pybindings")]
#[pymethods]
impl TrackerDAQTempPacket {
  
  #[getter]
  fn get_rom_id(&self) -> [u64;256] {
    self.rom_id
  }
  
  #[getter]
  fn get_temp(&self) -> [u16;256] {
    self.temp
  }

}

#[cfg(feature="pybindings")]
pythonize_telemetry!(TrackerDAQTempPacket);

