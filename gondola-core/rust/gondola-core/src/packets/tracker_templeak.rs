// This file is part of gaps-online-software and published 
// under the GPLv3 license

use crate::prelude::*;

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

