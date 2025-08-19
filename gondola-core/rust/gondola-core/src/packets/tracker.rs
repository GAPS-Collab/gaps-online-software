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

//---------------------------------------------------

#[cfg_attr(feature="pybindings",pyclass)]
pub struct TrackerTempLeakPacket {
  pub telemetry_header : TelemetryPacketHeader,
  pub tracker_header   : TrackerHeader,
  pub row_offset       : u8,
  pub templeak         : [[u32;6];6],
  pub seu              : [[u32;6];6]
}

impl fmt::Display for TrackerTempLeakPacket {
  fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
    let mut repr = String::from("<TrackerTempLeakPacket");
    repr    += &(format!("\n {}", self.telemetry_header));
    repr    += &(format!("\n {}", self.tracker_header));
    repr    += &(format!("\n ROW OFFSET {}", self.row_offset));
    repr    += "\n*** TEMPLEAK ***";
    for k in 0..6 {
      repr  += &(format!("\n {:?}", self.templeak[k]));
    }
    repr    += "\n*** SEU ***";
    for k in 0..6 {
      repr  += &(format!("\n {:?}", self.seu[k]));
    }
    repr    += ">";
    write!(f, "{}", repr)
  }
}

impl TrackerTempLeakPacket {
  pub fn new() -> Self {
    Self {
      telemetry_header : TelemetryPacketHeader::new(),
      tracker_header   : TrackerHeader::new(),
      row_offset       : 0,
      templeak         : [[0;6];6],
      seu              : [[0;6];6]
    }
  }
}

impl Serialization for TrackerTempLeakPacket { 
  fn from_bytestream(stream: &Vec<u8>,
                     pos: &mut usize)
    -> Result<Self, SerializationError> {
    let mut tp          = TrackerTempLeakPacket::new();
    tp.tracker_header   = TrackerHeader::from_bytestream(stream, pos)?;
    if stream.len() == *pos as usize {
      error!("Packet contains only header!");
      return Ok(tp);
    }
    if stream.len() - *pos < (36*3 + 1) {
      return Err(SerializationError::StreamTooShort);
    }
    let row_info = parse_u8(stream, pos);
    tp.row_offset = row_info & 0x7;
    for row in 0..6 {
      for module in 0..6 {
        let b0 = parse_u8(stream, pos) as u32;
        let b1 = parse_u8(stream, pos) as u32;
        let b2 = parse_u8(stream, pos) as u32;
        let seu_ : u32 = b2 >> 1;
        let mut templeak_ : u32 = (b2 << 10) | (b1 << 2)  | (b0 >> 6);
        templeak_ &= 0x7ff;
        tp.templeak[row][module] = templeak_;
        tp.seu[row][module] = seu_;
      }
    }
    Ok(tp)
  }
}

#[cfg(feature="pybindings")]
#[pymethods]
impl TrackerTempLeakPacket {

  #[getter]
  fn get_row_offset(&self) -> u8 {
    self.row_offset
  }
  
  #[getter]
  fn temp_leak(&self) -> [[u32;6];6] {
    self.templeak
  }
  
  #[getter]
  fn get_seu(&self) -> [[u32;6];6] {
    self.seu
  }
}

#[cfg(feature="pybindings")]
pythonize_telemetry!(TrackerTempLeakPacket);

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

