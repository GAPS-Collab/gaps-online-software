// This file is part of gaps-online-software and published 
// under the GPLv3 license

use crate::prelude::*;

#[cfg_attr(feature="pybindings", pyclass)]
pub struct GPSPacket {
  pub telemetry_header : TelemetryPacketHeader,
  pub tracker_header   : TrackerHeader,
  pub utc_time         : u32,
  pub gps_info         : u8
}

impl GPSPacket {
  pub fn new() -> Self {
    Self {
      telemetry_header : TelemetryPacketHeader::new(),
      tracker_header   : TrackerHeader::new(),
      utc_time         : 0,
      gps_info         : 0,
    }
  }
  
  pub fn from_bytestream(stream: &Vec<u8>,
                         pos: &mut usize)
    -> Result<Self, SerializationError> {
    let mut gps_p       = GPSPacket::new();
    gps_p.tracker_header   = TrackerHeader::from_bytestream(stream, pos)?;
    if stream.len() == *pos as usize {
      error!("Packet contains only header!");
      return Ok(gps_p);
    }
    gps_p.utc_time = parse_u32(stream, pos);
    gps_p.gps_info = parse_u8(stream, pos);
    Ok(gps_p)
  }
}

impl fmt::Display for GPSPacket {
  fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
    let mut repr = String::from("<GPSPacket");
    repr    += &(format!("\n {}", self.telemetry_header));
    repr    += &(format!("\n {}", self.tracker_header));
    repr    += "\n*** GPS TIME ***";
    repr    += &(format!("\n UTC TIME (32bit) {}", self.utc_time));
    repr    += &(format!("\n GSP INFO (8bit)  {}", self.gps_info));
    repr    += ">";
    write!(f, "{}", repr)
  }
}

//----------------------------------------------------------------

#[cfg(feature="pybindings")]
#[pymethods]
impl GPSPacket {
  #[getter]
  fn get_telemetry_header(&self) -> TelemetryPacketHeader {
    self.telemetry_header.clone()
  }
  #[getter]
  fn get_tracker_header  (&self) -> TrackerHeader {
    self.tracker_header.clone()
  }
  #[getter]
  fn get_utc_time        (&self) -> u32 {
    self.utc_time 
  }
  #[getter]
  fn get_gps_info        (&self) -> u8 {
    self.gps_info
  }
}

#[cfg(feature="pybindings")]
pythonize_telemetry!(GPSPacket);

