//! The following file is part of gaps-online-software and published 
//! under the GPLv3 license

use std::fmt;

use crate::io::serialization::Serialization;
use crate::io::parsers::*;
use crate::errors::SerializationError;
use crate::packets::TelemetryPacketHeader;
use crate::packets::TelemetryPacketType;

#[cfg(feature="pybindings")]
use crate::impl_pythonize_display;

#[cfg(feature="pybindings")]
use pyo3::prelude::*;

#[cfg(feature="pybindings")]
use pyo3::types::PyBytes;
//use pyo3::types::PyMemoryView;


/// A wrapper for packets from the telemetry stream
///
/// This is very compact and mostly used as an 
/// intermediary
#[derive(Debug, Clone, PartialEq)]
#[cfg_attr(feature = "pybindings", pyclass, pyo3(name="TelemetryPacket"))]
pub struct TelemetryPacket {
  pub header  : TelemetryPacketHeader,
  pub payload : Vec<u8>
}

#[cfg(feature="pybindings")]
#[pymethods]
impl TelemetryPacket {
  #[new]
  fn new_py() -> Self {
    Self::new()
  }

  /// Get a zero copy view of the payload 
  /// Might be mostly useful for debugging purposes
  #[getter]
  fn payload<'py>(&self, py: Python<'py>) -> PyResult<Bound<'py, PyBytes>> {
    Ok(PyBytes::new(py, &self.payload))
  }

  #[getter]
  fn header(&self) -> TelemetryPacketHeader {
    // clone is fine here, since the packet header 
    // is pretty small
    self.header.clone()
  }

  #[getter]
  fn packet_type(&self) -> TelemetryPacketType {
    TelemetryPacketType::from(self.header.ptype)
  }
}

impl TelemetryPacket {

  pub fn new() -> Self {
    Self {
      header  : TelemetryPacketHeader::new(),
      payload : Vec::<u8>::new()
    }
  }

} 

impl Serialization for TelemetryPacket {

  /// No "classical" head byte marker
  const HEAD : u16 = 0;
  /// No "classical" tail byte marker
  const TAIL : u16 = 0;
  /// variable size
  const SIZE : usize = 0;

  fn from_bytestream(stream : &Vec<u8>, pos : &mut usize) -> Result<Self, SerializationError> {
    let mut tpacket: TelemetryPacket = TelemetryPacket::new();
    let header: TelemetryPacketHeader  = TelemetryPacketHeader::from_bytestream(stream, pos)?;
    tpacket.header = header;
    tpacket.payload = stream[*pos..*pos + header.length as usize - TelemetryPacketHeader::SIZE].to_vec();
    Ok(tpacket)
  }

  fn to_bytestream(&self) -> Vec<u8> {
    let mut stream: Vec<u8> = Vec::<u8>::new();
    let mut s_head = self.header.to_bytestream();
    stream.append(&mut s_head);
    stream.extend_from_slice(self.payload.as_slice());
    stream
  }
}

impl Default for TelemetryPacket { 
  fn default() -> Self {
    Self::new()
  }
}

impl fmt::Display for TelemetryPacket {
  fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
    let mut repr: String = String::from("<TelemetryPacket:");
    repr += &(format!("\n  Header      : {}",self.header));
    repr += &(format!("\n  Payload len : {}>",self.payload.len()));
    write!(f, "{}", repr)
  }
}

#[cfg(feature="pybindings")]
impl_pythonize_display!(TelemetryPacket, |s: &TelemetryPacket| s.to_string());


