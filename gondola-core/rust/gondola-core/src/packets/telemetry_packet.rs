//! Wrapper for all telemetry data - original implementation in 
//! bfsw
// The following file is part of gaps-online-software and published 
// under the GPLv3 license

use crate::prelude::*;

/// A wrapper for packets from the telemetry stream
///
/// This is very compact and mostly used as an 
/// intermediary
#[derive(Debug, Clone, PartialEq)]
#[cfg_attr(feature = "pybindings", pyclass, pyo3(name="TelemetryPacket"))]
pub struct TelemetryPacket {
  pub header       : TelemetryPacketHeader,
  pub payload      : Vec<u8>,
  pub tof_paddles  : Arc<HashMap<u8,  TofPaddle>>, 
  pub trk_strips   : Arc<HashMap<u32, TrackerStrip>>,
}

#[cfg(feature="pybindings")]
#[pymethods]
impl TelemetryPacket {

  #[staticmethod]
  #[pyo3(name="get_gcutime_unpacked")]
  fn get_gcutime_unpacked_py(stream : Vec<u8>) -> PyResult<f64> {
    Ok(Self::get_gcutime_unpacked(&stream)?)
  }
  
  /// For a packet of any merged event type, retrieve the run id from 
  /// the TOF part 
  #[pyo3(name="get_runid")]
  fn get_runid_py(&self) -> PyResult<u16> {
    let runid_opt = self.get_runid();
    match runid_opt {
      Some(runid_res) => {
        match runid_res { 
          Ok(runid) => {
            return Ok(runid);
          }
          Err(err) => {
            return Err(PyValueError::new_err(err.to_string()));
          }
        }
      }
      None => {
        return Err(PyValueError::new_err("This packet does not seem to contain a (useful) runid!")); 
      }
    }
  }

  /// For a packet of type 80 (tracker standalooe) retrieve the GPS time from 
  /// the tracker header 
  #[pyo3(name="get_gpstime_tracker")]
  fn get_gpstime_tracker_py(&self) -> PyResult<u64> {
    let gpstime_opt = self.get_gpstime_tracker();
    match gpstime_opt {
      Some(gpstime_res) => {
        match gpstime_res { 
          Ok(gpstime) => {
            return Ok(gpstime);
          }
          Err(err) => {
            return Err(PyValueError::new_err(err.to_string()));
          }
        }
      }
      None => {
        return Err(PyValueError::new_err("This packet does not seem to contain a GPS time!")); 
      }
    }
  }
  
  /// In case this is any type of event packet which has a tof event, we can 
  /// also get the GPS time (as long as it is assigned) 
  #[pyo3(name="get_gpstime_tof")]
  fn get_gpstime_tof_py(&self) -> PyResult<u64> {
    let gpstime_opt = self.get_gpstime_tof();
    match gpstime_opt {
      Some(gpstime_res) => {
        match gpstime_res { 
          Ok(gpstime) => {
            return Ok(gpstime);
          }
          Err(err) => {
            return Err(PyValueError::new_err(err.to_string()));
          }
        }
      }
      None => {
        return Err(PyValueError::new_err("This packet does not seem to contain a GPS time!")); 
      }
    }
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
    TelemetryPacketType::from(self.header.packet_type)
  }

  /// Check if this is either any of the different merged event 
  /// types 
  #[getter]
  #[pyo3(name="is_event_packet")]
  fn is_event_packet_py(&self) -> bool {
    self.is_event_packet()
  }

  /// Check if this packet is a complete run configuration 
  /// send by the TOF system
  #[getter] 
  #[pyo3(name="is_tof_toml_packet")]
  fn is_tof_toml_packet_py(&self) -> bool {
    if self.header.packet_type == TelemetryPacketType::AnyTofHK {
      // check the TOF packet type 
      // (self.payload contains a TOF packet) 
      let tof_packet_type = TofPacketType::from(parse_u8(&self.payload, &mut 2));
      return tof_packet_type == TofPacketType::LiftofSettings;
    }
    false
  }


  #[pyo3(name="to_bytestream")]
  fn to_bytestream_py(&self) -> Vec<u8> {
    self.to_bytestream()
  }

  #[staticmethod]
  #[pyo3(name="from_bytestream")]
  fn from_bytestream_py(stream : Vec<u8>, pos : usize) -> Result<Self, SerializationError> {
    let mut pos_ = pos;
    Self::from_bytestream(&stream, &mut pos_)
  }
}

impl TelemetryPacket {

  pub fn new() -> Self {
    Self {
      header      : TelemetryPacketHeader::new(),
      payload     : Vec::<u8>::new(),
      tof_paddles : Arc::new(HashMap::<u8, TofPaddle>::new()),
      trk_strips  : Arc::new(HashMap::<u32,TrackerStrip>::new()),
    }
  }
 
  pub fn is_event_packet(&self) -> bool {
    if self.header.packet_type == TelemetryPacketType::NoTofDataEvent
      || self.header.packet_type == TelemetryPacketType::NoGapsTriggerEvent 
      || self.header.packet_type == TelemetryPacketType::InterestingEvent 
      || self.header.packet_type == TelemetryPacketType::BoringEvent {
      true 
    } else {
      false
    }
  }

  /// Unpack the TelemetryPacket and return its content
  pub fn unpack<T>(&self) -> Result<T, SerializationError>
    where T: TelemetryPackable + Serialization {
    if !T::TEL_PACKET_TYPES_EVENT.contains(&self.header.packet_type) &&
      T::TEL_PACKET_TYPE != self.header.packet_type {
      error!("This bytestream is not for a {} packet!", self.header.packet_type);
      return Err(SerializationError::IncorrectPacketType);
    }
    let unpacked : T = T::from_bytestream(&self.payload, &mut 0)?;
    Ok(unpacked)
  }

  /// For a packet of any merged event type, retrieve the run id from 
  /// the TOF part 
  pub fn get_runid(&self) -> Option<Result<u16, SerializationError>> {
    if !self.is_event_packet() {
      return None;
    }
    if self.header.packet_type == TelemetryPacketType::NoTofDataEvent {
      return None;
    }
    // the run id is right after the timestamp (6 byte) 
    // and is 2 bytes long 
    if self.payload.len() < 37 {
      return Some(Err(SerializationError::StreamTooShort));
    }
    let mut pos   = 41usize;
    let runid = parse_u16(&self.payload, &mut pos);
    return Some(Ok(runid));
  }

  /// For a packet of type 80 (tracker standalooe) retrieve the GPS time from 
  /// the tracker header 
  pub fn get_gpstime_tracker(&self) -> Option<Result<u64, SerializationError>> {
    if self.header.packet_type != TelemetryPacketType::Tracker {
      return None;
    }
    // the tracker header with the gps time is the first 
    let mut pos = 10usize; // skip the first bytes in the TrackerHeader
    if self.payload.len() < 16 { 
      return Some(Err(SerializationError::StreamTooShort));
    }
    let lower = parse_u32(&self.payload, &mut pos);
    let upper = parse_u16(&self.payload, &mut pos);
    let ts    = make_systime(lower, upper);
    return Some(Ok(ts));
  }

  /// In case this is any type of event packet which has a tof event, we can 
  /// also get the GPS time (as long as it is assigned) 
  pub fn get_gpstime_tof(&self) -> Option<Result<u64, SerializationError>> {
    if !self.is_event_packet() {
      return None;
    }
    if self.header.packet_type == TelemetryPacketType::NoTofDataEvent {
      return None;
    }
    // then in the TelemetryEvent 
    // version (1byte)
    // flags0  (1byte)
    // -- only for version 1 supported 
    // + 8byte
    // event id (4byte)
    // tof dl   (1byte)
    // tof nby  (2byte)
    // -> 17 byte
    // ------ TofPacket 2 + 1 + 4 overhead (7byte) 
    // -> 24 byte 
    // TofEvent 2 + 1 + 2 + 1 + 4 + 1 
    // -> 35 byte 
    if self.payload.len() < 41 {
      return Some(Err(SerializationError::StreamTooShort));
    }
    //let mut pos = 42usize;
    let mut pos = 35usize;
    let ts32 = parse_u32(&self.payload, &mut pos);
    let ts16 = parse_u16(&self.payload, &mut pos);
    let ts   = 0x273000000000000 | (((ts16 as u64) << 32) | ts32 as u64);
    return Some(Ok(ts));
  }

  /// Get the gcutime from a packet without unpacking the full thing
  pub fn get_gcutime_unpacked(stream : &Vec<u8>) -> Result<f64, SerializationError> {
    // it starts with the serialized header and the packet byte, 
    // and then the timestamp is 32 bit. So we need to jump 3 bytes 
    // and then read 4 
    if stream.len() < 7 {
      error!("Can get gcutime from a bytestream shorter than 7 bytes!");
      return Err(SerializationError::StreamTooShort);
    }
    let mut pos = 3usize;
    let ts = parse_u32(stream, &mut pos);
    Ok(TelemetryPacketHeader::convert_telemetry_header_ts(ts))
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
    // to note here: The payload does not contain the payload for the header, it was just parsed
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

impl Frameable for TelemetryPacket {
  const CRFRAMEOBJECT_TYPE : CRFrameObjectType = CRFrameObjectType::TelemetryPacket;
}


#[cfg(feature="pybindings")]
pythonize!(TelemetryPacket);


