//! The TelemetryEvent or former "MergedEvent" is that what gets 
//! sent over telemetry during flight
// The following file is part of gaps-online-software and published 
// under the GPLv3 license

use crate::prelude::*;

#[cfg_attr(feature="pybindings", pyclass)]
pub struct TelemetryEvent {
  pub header              : TelemetryPacketHeader,
  pub creation_time       : u64,
  pub event_id            : u32,
  pub tracker_hits        : Vec<TrackerHit>,
  pub tracker_oscillators : Vec<u64>,
  pub tof_event           : TofEvent,
  pub raw_data            : Vec<u8>,
  pub flags0              : u8,
  pub flags1              : u8,
  pub version             : u8
}

impl TelemetryEvent {

  pub fn new() -> Self {
    let mut tracker_oscillators = Vec::<u64>::new();
    for _ in 0..10 {
      tracker_oscillators.push(0);
    }
    Self {
      header              : TelemetryPacketHeader::new(),
      creation_time       : 0,
      event_id            : 0,
      tracker_hits        : Vec::<TrackerHit>::new(),
      tracker_oscillators : tracker_oscillators,
      tof_event           : TofEvent::new(),
      raw_data            : Vec::<u8>::new(),
      flags0              : 0,
      flags1              : 1,
      version             : 0, 
    }
  }
}

impl Serialization for TelemetryEvent {
  
  fn from_bytestream(stream : &Vec<u8>,
                     pos    : &mut usize)
    -> Result<Self, SerializationError> {
    let mut me       = Self::new();
    let version      = parse_u8(stream, pos);
    me.version       = version;
    //println!("_version {}", _version);
    me.flags0         = parse_u8(stream, pos);
    // skip a bunch of Alex newly implemented things
    // FIXME
    if version == 0 {
      me.flags1      = parse_u8(stream, pos);
    } else {
      *pos += 8;
    }

    me.event_id       = parse_u32(stream, pos);
    //println!("EVENT ID {}", me.event_id);
    let _tof_delim    = parse_u8(stream, pos);
    //println!("TOF delim : {}", _tof_delim);
    if stream.len() <= *pos + 2 {
      error!("Not able to parse merged event!");
      return Err(SerializationError::StreamTooShort);
    }
   let num_tof_bytes = parse_u16(stream, pos) as usize;
    //println!("Num TOF bytes : {}", num_tof_bytes);
    if stream.len() < *pos+num_tof_bytes {
      error!("Not enough bytes for TOF packet! Expected {}, seen {}", *pos+num_tof_bytes as usize, stream.len());
      return Err(SerializationError::StreamTooShort); 
    }
    let pos_before = *pos;
    let tof_pack   = TofPacket::from_bytestream(stream, pos)?;
    let ts         = tof_pack.unpack::<TofEvent>()?;
    // sanity check - is tofpacket as long as num_tof_bytes lets us believe?
    if pos_before + num_tof_bytes != *pos {
      println!("Tofpacket {}", tof_pack);
      error!("Byte misalignment. Expected {num_tof_bytes}, got {pos} - {pos_before}"); 
      return Err(SerializationError::WrongByteSize);
    }
    me.tof_event = ts;
    let trk_delim    = parse_u8(stream, pos);

    //println!("TRK delim {}", trk_delim);
    if trk_delim != 0xbb {
      return Err(SerializationError::HeadInvalid);
    }
    if version == 1 {
      let num_trk_hits = parse_u16(stream, pos);
      if (*pos + (num_trk_hits as usize)*4 ) > stream.len() {
        return Err(SerializationError::StreamTooShort);
      }
      for _ in 0..num_trk_hits { 
        let mut hit  = TrackerHit::new();
        let strip_id = parse_u16(stream, pos);
        let adc      = parse_u16(stream, pos);
        hit.channel  = strip_id & 0b11111;
        hit.module   = (strip_id >> 5) & 0b111;
        hit.row      = (strip_id >> 8) & 0b111;
        hit.layer    = (strip_id >> 11) & 0b1111;
        hit.adc      = adc;
        me.tracker_hits.push(hit);
      }
      // oscillators
      let oscillators_delimiter = parse_u8(stream, pos);
      if oscillators_delimiter != 0xcc {
        return Err(SerializationError::HeadInvalid);
      }
      let osc_flags = parse_u8(stream, pos);
      let mut oscillator_idx = Vec::<u8>::new();
      for j in 0..8 {
        if (osc_flags >> j & 0b1) > 0 {
          oscillator_idx.push(j)
        }
      }
      if (*pos + oscillator_idx.len()*6) > stream.len() {
        return Err(SerializationError::StreamTooShort);
      }
      for idx in oscillator_idx.iter() {
        let lower = parse_u32(stream, pos);
        let upper = parse_u16(stream, pos);
        let osc : u64 = (upper as u64) << 32 | (lower as u64);
        me.tracker_oscillators[*idx as usize] = osc;
      }
    } else if version == 0 {
      error!("Unsupported {version}!");
      return Err(SerializationError::UnsupportedVersion);
    } else {
      error!("Unsuported version {version}!");
      return Err(SerializationError::UnsupportedVersion);
    } 
    Ok(me)
  }
}

impl fmt::Display for TelemetryEvent {
  fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
    let mut repr     = String::from("<TelemetryEvent:");
    let tof_str  = format!("\n  {}", self.tof_event);
    let mut good_hits = 0;
    if self.version == 0 {
      repr += "\n VERSION 0 NOT SUPPORTED!!";
    } else if self.version == 1 {
      for _ in &self.tracker_hits {
        good_hits += 1;
      }
    }
    repr += &(format!("  {}", self.header));
    repr += "\n  ** ** ** MERGED  ** ** **";
    repr += &(format!("\n  version         {}", self.version));
    repr += &(format!("\n  event ID        {}", self.event_id));  
    if self.version == 0 {
      repr += "\n VERSION 0 NOT SUPPORTED!!"; 
    }
    repr += "\n  ** ** ** TRACKER ** ** **";
    if self.version == 0 {
      repr += "\n VERSION 0 NOT SUPPORTED!!"; 
    } else if self.version == 1 {
      repr += &(format!("\n  Trk oscillators {:?}", self.tracker_oscillators)); 
    }
    repr += &(format!("\n  N Good Trk Hits {}", good_hits));
    repr += &tof_str;
    write!(f,"{}", repr)
  }
}

//----------------------------------------

#[cfg(feature="pybindings")]
#[pymethods]
impl TelemetryEvent {

  #[getter]
  #[pyo3(name="version")]
  fn version_py(&self) -> u8 {
    self.version
  }

  #[getter]
  fn tracker(&self) -> PyResult<Vec<TrackerHit>> {
    Ok(self.tracker_hits.clone())
  }

  #[getter]
  fn get_event_id(&self) -> u32 {
    self.event_id
  }

  // FIXME - do this with bound
  #[getter]
  fn get_tof(&self) -> PyResult<TofEvent> {
    Ok(self.tof_event.clone())
  }

  #[getter]
  fn tracker_pointcloud(&self) -> Vec<(f32, f32, f32, f32, f32)> {
    let mut pts = Vec::<(f32,f32,f32,f32,f32)>::new();
    for h in &self.tracker_hits {
      // uses adc
      // FIXME - factor 10!
      let pt = (10.0*h.x, 10.0*h.y, 10.0*h.z, f32::NAN, h.adc as f32);
      pts.push(pt);
    }
    pts
  }

  /// Populate a merged event from a TelemetryPacket.
  ///
  /// Telemetry packet type should be 90 (MergedEvent)
  #[staticmethod]
  fn from_telemetrypacket(packet : TelemetryPacket) -> PyResult<Self> {
    match Self::from_bytestream(&packet.payload, &mut 0) {
      Ok(mut event) => {
        event.header = packet.header.clone();
        return Ok(event);
      }
      Err(err) => {
        return Err(PyValueError::new_err(err.to_string()));
      }  
    }
  }
}

pythonize!(TelemetryEvent);

