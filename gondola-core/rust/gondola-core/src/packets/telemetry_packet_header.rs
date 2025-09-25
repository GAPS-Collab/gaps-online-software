//! Re-implementation of the telemetry header which is attached 
//! to each telemetry packet.
//! Re-implemented from bfsw
// The following file is part of gaps-online-software and published 
// under the GPLv3 license

use crate::prelude::*;

/// This gets attached to each telemetry packet 
/// "in front" of it, conveying meta information
#[derive(Debug, Copy, Clone, PartialEq)]
#[cfg_attr(feature = "pybindings", pyclass, pyo3(name="TelemetryPacketHeader"))]
pub struct TelemetryPacketHeader {
  /// A constant identifying the start of a packet
  pub sync      : u16,
  /// A unique identifier describing the following 
  /// packet's content
  pub packet_type     : TelemetryPacketType,
  /// The timestap from the gcu (flight computer) (NOT GPS!)
  /// when this packet header has been created
  pub timestamp : u32,
  /// Counting packets send out by the gcu (flight computer), 
  /// rolling over every u16::MAX
  pub counter   : u16,
  /// The length in bytes of the following packet
  pub length    : u16,
  /// A checksum to verify the integrity of the 
  /// following packet
  /// FIXME - currently the algorithm used is 
  ///         unknown
  pub checksum  : u16
}

impl TelemetryPacketHeader {

  pub fn new() -> Self {
    Self {
      sync      : 0,
      packet_type     : TelemetryPacketType::Unknown,
      timestamp : 0,
      counter   : 0,
      length    : 0,
      checksum  : 0,
    }
  }

  /// A re-implementation of make_packet_stub
  pub fn forge(packet_type : TelemetryPacketType) -> Self {
    let mut header = Self::new();
    header.sync    = 0x90EB;
    header.packet_type   = packet_type;
    header
  }

  /// Blatent copy of bfsw's timestamp_to_double
  pub fn get_gcutime(&self) -> f64 {
    (self.timestamp as f64) * 0.064 + 1631030675.0
  }
}

// methods which should be available through python, but 
// just need to be wrapped
#[cfg(feature="pybindings")]
#[pymethods]
impl TelemetryPacketHeader {
  
  #[getter]
  fn gcutime(&self) -> f64 {
    self.get_gcutime()
  }

  #[getter]
  fn get_packet_type(&self) -> TelemetryPacketType {
    self.packet_type
  }

  #[getter]
  fn get_timestamp(&self) -> u32 {
    self.timestamp 
  }

  #[getter]
  fn get_counter(&self) -> u16 {
    self.counter 
  }

  #[getter]
  fn get_length(&self) -> u16 {
    self.length
  } 

  #[getter] 
  fn get_checksum(&self) -> u16 {
    self.checksum 
  }
}

// Trait implementations

impl Serialization for TelemetryPacketHeader {
  
  const HEAD : u16 = 0x90eb;
  const TAIL : u16 = 0x0000; // there is no tail for telemetry packets
  const SIZE : usize = 13; 

  fn from_bytestream(stream : &Vec<u8>,
                     pos    : &mut usize)
    -> Result<Self, SerializationError> {
    if stream.len() < *pos + Self::SIZE {
      return Err(SerializationError::StreamTooShort);
    }
    if parse_u16(stream, pos) != 0x90eb {
      error!("The given position {} does not point to a valid header signature of {}", pos, 0x90eb);
      return Err(SerializationError::HeadInvalid {});
    }
    let mut thead   = TelemetryPacketHeader::new();
    thead.sync      = 0x90eb;
    thead.packet_type     = TelemetryPacketType::from(parse_u8 (stream, pos));
    thead.timestamp = parse_u32(stream, pos);
    thead.counter   = parse_u16(stream, pos);
    thead.length    = parse_u16(stream, pos);
    thead.checksum  = parse_u16(stream, pos);
    Ok(thead)
  }
  
  fn to_bytestream(&self) -> Vec<u8> {
    let mut stream = Vec::<u8>::new();
    //let head : u16 = 0x90eb;
    // "SYNC" is the header signature
    stream.extend_from_slice(&self.sync.to_le_bytes());
    stream.extend_from_slice(&(self.packet_type as u8).to_le_bytes());
    stream.extend_from_slice(&self.timestamp.to_le_bytes());
    stream.extend_from_slice(&self.counter.to_le_bytes());
    stream.extend_from_slice(&self.length.to_le_bytes());
    stream.extend_from_slice(&self.checksum.to_le_bytes());
    stream
  }
}

impl fmt::Display for TelemetryPacketHeader {
  fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
    let mut repr = String::from("<TelemetryPacketHeader:");
    repr += &(format!("\n  Header      : {}",self.sync));
    repr += &(format!("\n  Packet Type : {}",self.packet_type));
    repr += &(format!("\n  Timestamp   : {}",self.timestamp));
    repr += &(format!("\n  Counter     : {}",self.counter));
    repr += &(format!("\n  Length      : {}",self.length));
    repr += &(format!("\n  Checksum    : {}>",self.checksum));
    write!(f, "{}", repr)
  }
}

#[cfg(feature="pybindings")]
pythonize!(TelemetryPacketHeader);
