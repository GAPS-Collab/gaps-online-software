//! Tracker event as stored seperately in telemetry packet type 80
//!
//! This has been used in tracker stand-alone mode and for calibrations.
//! The event as described here is NOT part of the merged event
// This file is part of gaps-online-software and published 
// under the GPLv3 license


use crate::prelude::*;

/// Tracker stand-alone complete event information from up-to multiple
/// DAQ boxes as used in TelemetryPacketType::Tracker (80) 
#[derive(Debug, Clone, PartialEq)]
#[cfg_attr(feature="pybindings", pyclass)]
pub struct TrackerDAQEventPacket {
  pub daq_header : TrackerHeader,
  pub events     : Vec<TrackerDAQEvent>,
  pub run_id     : u16,
  pub run_id_old : u8,
  // not serialized 
  /// internal counter for number of tracker hits in 
  /// this event 
  pub n_hits     : u16,
}

impl TrackerDAQEventPacket {

  pub fn new() -> Self {
    Self {
      daq_header : TrackerHeader::new(),
      events     : Vec::<TrackerDAQEvent>::new(),
      run_id     : 0,
      run_id_old : 0,
      n_hits     : 0,
    }
  }

  /// Get all hits from all daq boxes participating 
  /// in this event 
  pub fn get_hits(&self) -> Vec<TrackerHit> {
    let mut hits = Vec::<TrackerHit>::new();
    for ev in &self.events {
      hits.extend_from_slice(&ev.hits);
    }
    hits
  }

  /// Get all event ids in the packet
  /// (these might be different) 
  pub fn get_event_ids(&self) -> Vec<u32> {
    let mut evids = Vec::<u32>::new();
    for ev in &self.events {
      evids.push(ev.event_id);
    }
    evids 
  }

  /// Get a copy specific TrackerDAQEvent for a given event id
  pub fn get_event_for_evid(&self, evid : u32) -> Option<TrackerDAQEvent> {
    for ev in &self.events {
      if ev.event_id == evid {
        return Some(ev.clone())
      }
    }
    None 
  }

  /// Return all tracker hits which belong to a certain event id
  pub fn get_hits_for_evid(&self, evid : u32) -> Vec<TrackerHit> {
    let mut hits = Vec::<TrackerHit>::new();
    for ev in &self.events {
      if ev.event_id == evid {
        hits.extend_from_slice(&ev.hits);
      }
    }
    hits
  }

  /// Populate a tracker event from a TelemetryPacket.
  ///
  /// Telemetry packet type should be 80 (Tracker)
  pub fn from_bytestream(stream : &Vec<u8>,
                         pos    : &mut usize)
    -> Result<Self, SerializationError> {
    let mut ev    = Self::new();
    ev.daq_header = TrackerHeader::from_bytestream(stream, pos)?; 
    if ev.daq_header.version >= 5 {
      ev.run_id     = parse_u16(stream, pos);
    } else {
      ev.run_id_old = parse_u8(stream, pos);
    }
    let event_header_size = 12usize;  
    loop {
      if ev.events.len() > 170 {
        error!("There seem to be more than 170 events (!) in the tracker. This is nonsense!");
        return Err(SerializationError::TooManyTrackerEvents); 
      }
      if ev.daq_header.version >= 4 {
        if *pos + 1 == stream.len() {
          if stream[*pos] == 0xff {
            return Ok(ev);
          }
        }
        if *pos == stream.len() {
          return Ok(ev);
        }
      } 
      if *pos + event_header_size > stream.len() { 
        error!("Unable to read more TrackerEvents! Stream is too short!");
        return Err(SerializationError::StreamTooShort);
      }
      
      let mut daq_event      = TrackerDAQEvent::new();
      daq_event.layer        = ev.daq_header.sys_id;
      let n_hits             = parse_u8(stream, pos);
      daq_event.flags1       = parse_u8(stream, pos);
      daq_event.event_id     = parse_u32(stream, pos);
      daq_event.event_time32 = parse_u32(stream, pos); 
      daq_event.event_time16 = parse_u16(stream, pos);
      if n_hits > 192 {
        error!("We see more than 192 hits in the event! This seems to be an issue.");
        return Err(SerializationError::TooManyTrackerHits);
      } 
      if (*pos + (3*(n_hits as usize))) > stream.len() {
        error!("Unable to read all {} tracker hits! Stream is too short!", n_hits);
        return Err(SerializationError::StreamTooShort);
      }
      for _ in 0..n_hits {
        let h0 = parse_u8(stream, pos);
        let h1 = parse_u8(stream, pos);
        let h2 = parse_u8(stream, pos);
        let asic_event_code = h2 >> 6;
        let channel = h0 & 0b11111;
        let module = h0 >> 5;
        let row = h1 & 0b111;
        let adc : u16 = ((h2 & 0b00111111) << 5) as u16 | (h1 >> 3) as u16;

        let mut hit = TrackerHit::new();
        hit.channel = channel as u16;
        hit.module  = module  as u16;
        hit.row     = row     as u16;
        hit.adc     = adc     as u16;
        hit.asic_event_code   = asic_event_code;
        daq_event.hits.push(hit);
        ev.n_hits += 1; 
      }
      ev.events.push(daq_event);
    }
  }
}

impl fmt::Display for TrackerDAQEventPacket {
  fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
    let mut repr = String::from("<TrackerDAQEventPacket:");
    repr += &(format!("\n  TrackerHeader       : {}", self.daq_header));
    repr += &(format!("\n  Run ID/Run ID (old) : {} {}", self.run_id, self.run_id_old));
    repr += &(format!("\n  - N DAQ ev., N Hits : {} {}", self.events.len(), self.n_hits));
    for daq in &self.events {
      repr += &(format!("\n  {}", daq));
    }
    repr += "\n";
    write!(f, "{}", repr)
  }
}

#[cfg(feature="pybindings")]
#[pymethods]
impl TrackerDAQEventPacket {
 
  #[staticmethod]
  fn from_telemetrypacket(packet : TelemetryPacket) -> PyResult<Self> {
    match Self::from_bytestream(&packet.payload, &mut 0) {
      Ok(event) => {
        return Ok(event);
      }
      Err(err) => {
        return Err(PyValueError::new_err(err.to_string()));
      }  
    }
  }

  #[getter]
  fn get_nhits(&self) -> u16 {
    self.n_hits
  }

  #[getter]
  #[pyo3(name="event_ids")]
  fn get_event_ids_py(&self) -> Vec<u32> {
    self.get_event_ids()
  }
  
  /// Get a copy specific TrackerDAQEvent for a given event id
  #[pyo3(name="get_event_for_evid")]
  fn get_event_for_evid_py(&self, evid : u32) -> Option<TrackerDAQEvent> {
    self.get_event_for_evid(evid)
  }

  #[getter] 
  fn get_header(&self) -> TrackerHeader {
    self.daq_header 
  }

  #[getter] 
  fn get_events(&self) -> Vec<TrackerDAQEvent> {
    self.events.clone()
  }
  
  /// Return all tracker hits which belong to a certain event id
  #[pyo3(name="get_hits_for_evid")]
  fn get_hits_for_evid_py(&self, evid : u32) -> Vec<TrackerHit> {
    self.get_hits_for_evid(evid).clone()
  }

  #[getter]
  fn get_run_id(&self) -> u16 {
    self.run_id 
  }

  #[getter]
  fn get_run_id_old(&self) -> u8 {
    self.run_id_old 
  }
}

#[cfg(feature="pybindings")]
pythonize!(TrackerDAQEventPacket);

