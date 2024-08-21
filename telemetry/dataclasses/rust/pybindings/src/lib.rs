use std::collections::HashMap;

use pyo3::prelude::*;
use pyo3::exceptions::PyValueError;

extern crate pyo3_log;
//extern crate rpy-tof-dataclasses;

use telemetry_dataclasses::packets as tel_api;
use telemetry_dataclasses::io as tel_io_api;
extern crate rpy_tof_dataclasses;
use rpy_tof_dataclasses::dataclasses::{
  PyTofHit,
  PyTofEventSummary,
};

// FIXME - this needs to go to liftof-python
// or maybe we want to revive the dataclasses pybindings
// in tof-dataclasses?
use tof_dataclasses::events as tof_api;
use tof_dataclasses::packets::TofPacket;
use tof_dataclasses::serialization::Serialization;

#[pyclass]
#[pyo3(name="TelemetryHeader")]
struct PyTelemetryHeader {
  header : tel_api::TelemetryHeader,
}

impl PyTelemetryHeader {
  pub fn set_header(&mut self, header : tel_api::TelemetryHeader) {
    self.header = header
  }
}

#[pymethods]
impl PyTelemetryHeader {
  #[new]
  fn new() -> Self {
    Self {
      header : tel_api::TelemetryHeader::new(),
    }
  }

  /// Get the current packet count
  /// 
  /// (16bit number) so rollovers are 
  /// expected
  #[getter]
  fn counter(&self) -> u16 { 
    self.header.counter
  } 

  #[getter]
  fn packet_type(&self) -> u8 {
    self.header.ptype
  }

  /// GCU time of packet creation
  #[getter]
  fn timestamnp(&self) -> u32 {
    self.header.timestamp
  }

  /// The length of the following payload
  #[getter]
  fn length(&self) -> u16 {
    self.header.length
  }

  fn __repr__(&self) -> PyResult<String> {
    Ok(format!("<PyO3Wrapper: {}>", self.header))
  }
}

#[pyclass]
#[pyo3(name="MergedEvent")]
struct PyMergedEvent {
  event : tel_api::MergedEvent,
}

impl PyMergedEvent {
  pub fn set_event(&mut self, event : tel_api::MergedEvent) {
    self.event = event;
  }
}

#[pymethods]
impl PyMergedEvent {
  #[new]
  fn new() -> Self {
    Self {
      event : tel_api::MergedEvent::new(),
    }
  }

  #[getter]
  fn tracker(&self) -> PyResult<Vec<PyTrackerEvent>> {
    let mut events = Vec::<PyTrackerEvent>::new();
    for k in &self.event.tracker_events {
      let mut pytrk = PyTrackerEvent::new();
      pytrk.set_event(k.clone());
      events.push(pytrk);
    }
    Ok(events)
  }

  #[getter]
  fn tof(&self) -> PyResult<PyTofEventSummary> {
    match TofPacket::from_bytestream(&self.event.tof_data, &mut 0) {
      Err(err) => {
        //error!("Unable to parse TofPacket! {err}");
        return Err(PyValueError::new_err(err.to_string()));
      }
      Ok(pack) => {
        match pack.unpack::<tof_api::TofEventSummary>() {
          Err(err) => {
            return Err(PyValueError::new_err(err.to_string()));
            //error!("Unable to parse TofEventSummary! {err}");
          }
          Ok(ts)    => {
            let mut pyts = PyTofEventSummary::new();
            pyts.set_event(ts);
            return Ok(pyts);
          }
        }
      }
    }
  }

  fn from_telemetrypacket(&mut self, packet : PyTelemetryPacket) -> PyResult<()> {
    match tel_api::MergedEvent::from_bytestream(&packet.packet.payload, &mut 0) {
      Ok(event) => {
        self.set_event(event);
        self.event.header = packet.packet.header.clone();
      }
      Err(err) => {
        return Err(PyValueError::new_err(err.to_string()));
      }  
    }
    Ok(())
  }
  
  fn __repr__(&self) -> PyResult<String> {
    Ok(format!("<PyO3Wrapper: {}>", self.event))
  }
}

#[pyclass]
#[pyo3(name="TrackerHeader")]
struct PyTrackerHeader {
  header : tel_api::TrackerHeader,
}

impl PyTrackerHeader {
  pub fn set_header(&mut self, header : tel_api::TrackerHeader) {
    self.header = header;
  }
}

#[pymethods]
impl PyTrackerHeader {

  #[new]
  fn new() -> Self {
    Self {
      header : tel_api::TrackerHeader::new(),
    }
  }

  #[getter]
  fn sync     (&self) -> u16 {
    self.header.sync
  }

  #[getter]
  fn crc      (&self) -> u16 {
    self.header.crc
  }

  #[getter]
  fn sys_id   (&self) -> u8 { 
    self.header.sys_id
  }

  #[getter]
  fn packet_id(&self) -> u8  {
    self.header.packet_id
  }

  #[getter]
  fn length   (&self) -> u16 {
    self.header.length
  }

  #[getter]
  fn daq_count(&self) -> u16 {
    self.header.daq_count
  }

  #[getter]
  fn sys_time (&self) -> u64 {
    self.header.sys_time
  }

  #[getter]
  fn version  (&self) -> u8  {
    self.header.version
  }

  fn __repr__(&self) -> PyResult<String> {
    Ok(format!("<PyO3Wrapper: {}>", self.header))
  }

}

#[pyclass]
#[pyo3(name="TrackerPacket")]
struct PyTrackerPacket {
  packet : tel_api::TrackerPacket,
}

impl PyTrackerPacket {
  pub fn set_packet(&mut self, packet : tel_api::TrackerPacket) {
    self.packet = packet;
  }
}

#[pymethods]
impl PyTrackerPacket {
  #[new]
  fn new() -> Self {
    Self {
      packet : tel_api::TrackerPacket::new(),
    }
  }

  #[getter]
  fn header(&self) -> PyTrackerHeader {
    let mut pth = PyTrackerHeader::new();
    pth.set_header(self.packet.tracker_header.clone());
    pth
  }

  #[getter]
  fn events(&self) -> Vec<PyTrackerEvent> {
    let mut events = Vec::<PyTrackerEvent>::new();
    for k in &self.packet.events {
      let mut pyev = PyTrackerEvent::new();
      pyev.set_event(k.clone());
      events.push(pyev);
    }
    events
  }

  fn from_telemetrypacket(&mut self, packet : PyTelemetryPacket) -> PyResult<()> {
    match tel_api::TrackerPacket::from_bytestream(&packet.packet.payload, &mut 0) {
      Ok(tpacket) => {
        self.set_packet(tpacket);
        self.packet.telemetry_header = packet.packet.header.clone();
      }
      Err(err) => {
        return Err(PyValueError::new_err(err.to_string()));
      }  
    }
    Ok(())
  }
  
  fn __repr__(&self) -> PyResult<String> {
    Ok(format!("<PyO3Wrapper: {}>", self.packet))
  }
}

#[derive(Clone)]
#[pyclass]
#[pyo3(name="TelemetryPacket")]
struct PyTelemetryPacket {
  packet : tel_api::TelemetryPacket,
}

impl PyTelemetryPacket {
  pub fn set_packet(&mut self, packet : tel_api::TelemetryPacket) {
    self.packet = packet;
  }
}

#[pymethods]
impl PyTelemetryPacket {
  #[new]
  fn new() -> Self {
    Self {
      packet : tel_api::TelemetryPacket::new(),
    }
  }
  
  #[getter]
  fn header(&self) -> PyTelemetryHeader {
    let mut header = PyTelemetryHeader::new();
    header.set_header(self.packet.header.clone());
    header
  }
  
  #[getter]
  fn payload(&self) -> Vec<u8> {
    // FIXME
    self.packet.payload.clone()
  }

  fn __repr__(&self) -> PyResult<String> {
    Ok(format!("<PyO3Wrapper: {}>", self.packet))
  }
}

#[pyclass]
#[pyo3(name="TelemetryPacketReader")]
struct PyTelemetryPacketReader {
  reader   : tel_io_api::TelemetryPacketReader,
}

#[pymethods]
impl PyTelemetryPacketReader {
  #[new]
  fn new(filename : String) -> Self {
    Self {
      reader     : tel_io_api::TelemetryPacketReader::new(filename)
    }
  }

  #[getter]
  fn packet_index(&mut self) -> PyResult<HashMap<u8, usize>> {
    let index = self.reader.get_packet_index()?;
    self.reader.rewind();
    Ok(index)
  }
 
  fn rewind(&mut self) -> PyResult<()> {
    Ok(self.reader.rewind()?)
  }

  fn __next__(mut slf: PyRefMut<'_, Self>) -> Option<PyTelemetryPacket> {
    match slf.reader.next() { 
      Some(tp) => {
        let mut pytp = PyTelemetryPacket::new();
        pytp.set_packet(tp);
        return Some(pytp)
      }
      None => {
        return None;
      }
    }
    //
  }

  fn __iter__(slf: PyRef<'_, Self>) -> PyRef<'_, Self> {
    slf 
  }

  fn __repr__(&self) -> PyResult<String> {
    Ok(format!("<PyO3Wrapper: {}>", self.reader))
  }
}

#[pyfunction]
fn get_gapsevents(fname : String) -> Vec<PyGapsEvent> {
  let mut pyevents = Vec::<PyGapsEvent>::new();
  let events = tel_io_api::get_gaps_events(fname);
  for ev in events {
    let mut pyev = PyGapsEvent::new();
    pyev.set_tof(ev.tof.clone());
    pyev.set_tracker(ev.tracker.clone());
    pyevents.push(pyev);
  }
  pyevents
}


#[pyclass]
#[pyo3(name="GapsTelemetryEvent")]
struct PyGapsEvent {
  event   : tel_api::GapsEvent,
}

impl PyGapsEvent {
  pub fn set_tof(&mut self, tes : tof_api::TofEventSummary) {
    self.event.tof = tes;
  }
  
  pub fn set_tracker(&mut self, trk : Vec<tel_api::TrackerEvent>) {
    self.event.tracker = trk;
  }
}

#[pymethods]
impl PyGapsEvent {
  #[new]
  fn new() -> Self {
    Self {
      event     : tel_api::GapsEvent::new(),
    }
  }

  #[getter]
  fn tof(&self) -> PyTofEventSummary {
    let mut tof =  PyTofEventSummary::new();
    tof.set_event(self.event.tof.clone());
    tof
  }

  #[getter]
  fn tracker(&self) -> Vec<PyTrackerEvent> {
    let mut trk_ev = Vec::<PyTrackerEvent>::new();
    for ev in &self.event.tracker {
      let mut py_ev = PyTrackerEvent::new();
      py_ev.set_event(ev.clone());
      trk_ev.push(py_ev)
    }
    trk_ev
  }
  
  fn __repr__(&self) -> PyResult<String> {
    Ok(format!("<PyO3Wrapper: {}>", self.event))
  }
}

#[pyclass]
#[pyo3(name="TrackerHit")]
struct PyTrackerHit {
  th : tel_api::TrackerHit,
}

impl PyTrackerHit {
  pub fn set_hit(&mut self, th : tel_api::TrackerHit) {
    self.th = th;
  }
}

#[pymethods]
impl PyTrackerHit {

  #[new]
  fn new() -> Self {
    Self {
      th : tel_api::TrackerHit::new(),
    }
  }

  #[getter]
  fn row(&self) -> u8 {
    self.th.row
  }

  #[getter]
  fn module(&self) -> u8 {
    self.th.module
  }

  #[getter]
  fn channel(&self) -> u8 {
    self.th.channel
  }

  #[getter]
  fn adc(&self) -> u16 {
    self.th.adc
  }

  #[getter]
  fn asic_event_code(&self) -> u8 {
    self.th.asic_event_code
  }
  
  fn __repr__(&self) -> PyResult<String> {
    Ok(format!("<PyO3Wrapper: {}>", self.th))
  }
}

#[pyclass]
#[pyo3(name="Trackerevent")]
struct PyTrackerEvent {
  te : tel_api::TrackerEvent
}

impl PyTrackerEvent {
  fn set_event(&mut self, te : tel_api::TrackerEvent) {
    self.te = te;
  }
}

#[pymethods]
impl PyTrackerEvent {
  #[new]
  fn new() -> Self {
    Self {
      te : tel_api::TrackerEvent::new(),
    }
  }

  #[getter]
  fn layer(&self) -> u8 {
    self.te.layer
  }
  
  #[getter]
  fn flags1(&self) -> u8 {
    self.te.flags1
  }
  
  #[getter]
  fn event_id(&self) -> u32 {
    self.te.event_id
  }
  
  #[getter]
  fn event_time(&self) -> u64 {
    self.te.event_time
  }

  //fn from_trackerpacket(&self, TrackerPacket) -> PyResult<()> {
  //  match tel_api::TrackerEvent::from_bytestream(&packet.packet.payload, &mut 0) {
  //    Ok(event) => {
  //      self.set_event(event);
  //      self.event.header = packet.packet.header.clone();
  //    }
  //    Err(err) => {
  //      return Err(PyValueError::new_err(err.to_string()));
  //    }  
  //  }
  //  Ok(()) 
  //}

  #[getter]
  fn hits(&self) -> Vec<PyTrackerHit> {
    let mut hits = Vec::<PyTrackerHit>::new();
    for h in &self.te.hits {
      let mut py_hit = PyTrackerHit::new();
      py_hit.set_hit(*h);
      hits.push(py_hit);
    }
    hits
  }

  fn __repr__(&self) -> PyResult<String> {
    Ok(format!("<PyO3Wrapper: {}>", self.te))
  }
}


/// Python API to rust version of tof-dataclasses.
///
/// Currently, this contains only the analysis 
/// functions
#[pymodule]
#[pyo3(name = "rust_telemetry")]
fn rust_dataclasses(_py: Python<'_>, m: &PyModule) -> PyResult<()> {
    pyo3_log::init();
    m.add_function(wrap_pyfunction!(get_gapsevents,m)?)?;
    m.add_class::<PyTelemetryPacket>()?;
    m.add_class::<PyTelemetryPacketReader>()?;
    m.add_class::<PyMergedEvent>()?;
    m.add_class::<PyGapsEvent>()?;
    m.add_class::<PyTofHit>()?;
    m.add_class::<PyTofEventSummary>()?;
    m.add_class::<PyTrackerHit>()?;
    m.add_class::<PyTrackerEvent>()?;
    m.add_class::<PyTrackerPacket>()?;
    Ok(())
}

