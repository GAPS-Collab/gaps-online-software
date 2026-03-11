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
  pub header     : TelemetryPacketHeader,
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
      header     : TelemetryPacketHeader::new(),
      daq_header : TrackerHeader::new(),
      events     : Vec::<TrackerDAQEvent>::new(),
      run_id     : 0,
      run_id_old : 0,
      n_hits     : 0,
    }
  }
 
  /// Create a telemetrypacket 
  pub fn pack(&self) -> TelemetryPacket {
    let mut tp = TelemetryPacket::new();
    tp.header  = self.header.clone();
    tp.payload = self.to_bytestream();
    tp
  }

  /// Remove a single TrackerDAQEvent from the 
  /// associated list of events 
  ///
  /// # Arguments:
  ///   * evid : Event ID of the event in the 
  ///            list of associated events to 
  ///            be removed
  pub fn remove_event(&mut self,evid : u32) {
    self.events.retain(|x| x.event_id != evid);
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
}

impl Serialization for TrackerDAQEventPacket { 

  fn from_bytestream(stream : &Vec<u8>,
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
        println!("{}", ev); 
        println!("pos {} , stream {}",pos, stream.len());
        println!("Unable to read more TrackerEvents! Stream is too short!");
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
        println!("Unable to read all {} tracker hits! Stream is too short!", n_hits);
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
        let adc : u16 = ((h2 as u16 & 0b00111111) << 5) | (h1 >> 3) as u16;
        let mut hit = TrackerHit::new();
        hit.channel = channel;
        hit.module  = module ;
        hit.row     = row    ;
        hit.adc     = adc    ;
        hit.asic_event_code   = asic_event_code;
        daq_event.hits.push(hit);
        ev.n_hits += 1; 
      }
      ev.events.push(daq_event);
    }
  }
  
  fn to_bytestream(&self) -> Vec<u8> {
    let mut stream = self.daq_header.to_bytestream();
    stream.extend_from_slice(&self.run_id.to_le_bytes());
    for ev in &self.events {
      stream.push(ev.hits.len() as u8);
      stream.push(ev.flags1);
      stream.extend_from_slice(&ev.event_id.to_le_bytes());
      stream.extend_from_slice(&ev.event_time32.to_le_bytes());
      stream.extend_from_slice(&ev.event_time16.to_le_bytes()); 
      for h in &ev.hits {
        let h0 = ((h.module << 5) | (h.channel & 0b11111)) as u8;
        let h1_adc = h.adc & 0b11111;
        let h2_adc = h.adc >> 5;
        let h1 = ((h1_adc << 3) as u8) | (h.row as u8 & 0b111);
        let h2 = (h.asic_event_code << 6) as u16 | h2_adc;
        stream.push(h0);
        stream.push(h1);
        stream.push(h2 as u8);
      }
    }
    stream
  }
  
}

impl TelemetryPackable for TrackerDAQEventPacket {
  const TEL_PACKET_TYPE : TelemetryPacketType = TelemetryPacketType::Tracker;
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

#[cfg(feature="random")]
impl FromRandom for TrackerDAQEventPacket {

  fn from_random() -> Self {
    let mut packet    = Self::new();
    let mut rng       = rand::rng();
    packet.header     = TelemetryPacketHeader::from_random();
    packet.daq_header = TrackerHeader::from_random();
    packet.events     = Vec::<TrackerDAQEvent>::new();
    packet.run_id     = rng.random::<u16>();
    //packet.run_id_old = rng.random::<u8>();
    let n_events : u8 = rng.random_range(0..6);
    for _ in 0..n_events {
      let mut ev = TrackerDAQEvent::from_random();
      ev.layer   = packet.daq_header.sys_id;
      for h in &mut ev.hits {
        //h.adc = h.adc & 0b11111;
        h.oscillator = 0;
      }
      packet.events.push(ev);
    }
    packet
  }
}

#[test]
#[cfg(feature="random")]
fn serialize_deserialize_trackerdaqeventpacket() {
  for _ in 0..100 {
    let packet = TrackerDAQEventPacket::from_random();  
    let stream = packet.to_bytestream();
    let test   = TrackerDAQEventPacket::from_bytestream(&stream, &mut 0).unwrap();
    assert_eq!(packet.run_id    , test.run_id); 
    assert_eq!(packet.run_id_old, test.run_id_old); 
    assert_eq!(packet.events.len(),   test.events.len());
    assert_eq!(packet.daq_header, test.daq_header);
    println!("Have {} events!", packet.events.len());
    for k in 0..packet.events.len() {
      assert_eq!(packet.events[k],test.events[k]);
    }
    println!("-- Success! --");
  }
} 

#[cfg(feature="pybindings")]
#[pymethods]
impl TrackerDAQEventPacket {
 
  #[staticmethod]
  fn from_telemetrypacket(packet : TelemetryPacket) -> PyResult<Self> {
    match Self::from_bytestream(&packet.payload, &mut 0) {
      Ok(mut event) => {
        event.header  = packet.header.clone();
        return Ok(event);
      }
      Err(err) => {
        return Err(PyValueError::new_err(err.to_string()));
      }  
    }
  }

  /// Create a telemetry packet to be send or 
  /// written to disk
  #[pyo3(name="pack")]
  fn pack_py(&self) -> TelemetryPacket {
    self.pack()
  }

  /// Remove a single TrackerDAQEvent from the 
  /// associated list of events 
  ///
  /// # Arguments:
  ///   * evid : Event ID of the event in the 
  ///            list of associated events to 
  ///            be removed
  #[pyo3(name="remove_event")]
  fn remove_event_py(&mut self, evid : u32) {
    self.remove_event(evid);
  }

  #[getter] 
  fn get_gcutime(&self) -> f64 {
    self.header.get_gcutime() 
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

