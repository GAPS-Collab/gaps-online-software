use std::collections::HashMap;

use pyo3::prelude::*;
use pyo3::exceptions::PyValueError;
use pyo3::types::PyFunction;

use std::fmt;

extern crate pyo3_log;
//extern crate rpy-tof-dataclasses;

use telemetry_dataclasses::packets as tel_api;
use telemetry_dataclasses::io as tel_io_api;
//extern crate rpy_tof_dataclasses;
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

  /// Alex' special time convention
  #[getter]
  fn gcutime(&self) -> f64 {
    self.header.get_gcutime()
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
  fn version(&self) -> u8 {
    self.event.version
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



  /// Check if TOF/trackler data can be unpacked an no errors are thrown
  #[getter]
  fn broken(&self) -> bool {
    // since the tracker part is already deserialized, the check
    // is only relevant for the tof part
    match TofPacket::from_bytestream(&self.event.tof_data, &mut 0) {
      Err(err) => {
        //error!("Unable to parse TofPacket! {err}");
        return true;
      }
      Ok(pack) => {
        match pack.unpack::<tof_api::TofEventSummary>() {
          Err(err) => {
            return true;
            //error!("Unable to parse TofEventSummary! {err}");
          }
          Ok(ts)    => {
            return false;
          }
        }
      }
    }
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

  /// Populate a merged event from a TelemetryPacket.
  ///
  /// Telemetry packet type should be 90 (MergedEvent)
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

  #[getter]
  fn packet_type(&self) -> tel_api::TelemetryPacketType {
    let ptype = tel_api::TelemetryPacketType::from(self.packet.header.ptype);
    ptype
  }

  fn __repr__(&self) -> PyResult<String> {
    Ok(format!("<PyO3Wrapper: {}>", self.packet))
  }
}


/// Read the GAPS binary data stream, dubbed as "telemetry"
///
/// These are binary files, typically with a name like RAW240716_094940.bin,
/// where the numbers are the UTC timestamp when the file has been written.
///
/// These telemetry files contains data seperated by delimiters, called "packets".
/// The TelemetryPacketReader will recognized the delimiters, and emit these 
/// individual packets.
///  
/// When creating a new instalnce of TelemetryPacketReader, the intance will emit packets
/// until the whole file is consumed. To re-use the same instance, call
/// TelemetryPacketReader::rewind
///
/// # Arguments
///
/// * filename - the name of the binary file to be read
/// * filter   - emit only TelemetryPackets of a certain type. If set to 
///              TelemetryPacketType::Unknown, all packets will be emitted
/// * start    - [NOT IMPLEMENTED]
/// * stop     - [NOT IMPLEMENTED]
#[pyclass]
#[pyo3(name="TelemetryPacketReader")]
struct PyTelemetryPacketReader {
  reader   : tel_io_api::TelemetryPacketReader,
}

#[pymethods]
impl PyTelemetryPacketReader {
  #[new]
  #[pyo3(signature = (filename, filter=tel_api::TelemetryPacketType::Unknown,start=0, nevents=0))]
  fn new(filename : String, filter : tel_api::TelemetryPacketType, start : usize, nevents : usize) -> Self {
    let mut reader_init = Self {
      reader     : tel_io_api::TelemetryPacketReader::new(filename)
    };
    reader_init.reader.filter = filter;
    reader_init
  }

  /// Any filter will be selecting packets of only this type
  ///
  /// If all packets should be allowed, set the packet type to Unknown
  #[getter]
  fn get_filter(&self) -> PyResult<tel_api::TelemetryPacketType> {
    Ok(self.reader.filter)
  }

  #[setter]
  fn set_filter(&mut self, ptype : tel_api::TelemetryPacketType) -> PyResult<()> {
    self.reader.filter = ptype;
    Ok(())
  }

  /// Return an inventory of packets in this file, where TelemetryPacketType is
  /// represented by its associtated integer
  ///
  /// # Arguments
  /// * verbose    : print the associated TelemetryPacketTypes (names)
  #[pyo3(signature = (verbose=false))]
  fn get_packet_index(&mut self, verbose : bool) -> PyResult<HashMap<u8, usize>> {
    let idx = self.reader.get_packet_index()?;
    if verbose {
      println!("<TelemetryPacketReader::index");
      for k in idx.keys() {
        let ptype = tel_api::TelemetryPacketType::from(*k);
        println!("--> {} ({}) : {}",k, ptype, idx.get(&k).unwrap());
      }
      println!(">");
    }
    self.reader.rewind();
    Ok(idx)
  }

  /// "Rewind" the file, meaning set the cursor to the beginning again.
  ///
  /// All packets can be emitted again
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


/// Representation of a TrackerHit 
///
/// This is the same representation as in the GSE DB
#[pyclass]
#[pyo3(name="TrackerHit")]
#[derive(Debug, Clone)]
struct PyTrackerHit {
  //th : tel_api::TrackerHit,
  pub row             : u8,
  pub module          : u8,
  pub channel         : u8,
  pub adc             : u16,
  pub asic_event_code : u8,
}

impl PyTrackerHit {
  pub fn set_hit(&mut self, th : tel_api::TrackerHit) {
    self.row             = th.row;
    self.module          = th.module;
    self.channel         = th.channel;
    self.adc             = th.adc;
    self.asic_event_code = th.asic_event_code;
  }
}

#[pymethods]
impl PyTrackerHit {

  #[new]
  fn new() -> Self {
    Self {
      row             : 0,
      module          : 0,
      channel         : 0,
      adc             : 0,
      asic_event_code : 0,
    }
  }

  #[getter]
  fn row(&self) -> u8 {
    self.row
  }

  #[getter]
  fn module(&self) -> u8 {
    self.module
  }

  #[getter]
  fn channel(&self) -> u8 {
    self.channel
  }

  #[getter]
  fn adc(&self) -> u16 {
    self.adc
  }

  #[getter]
  fn asic_event_code(&self) -> u8 {
    self.asic_event_code
  }
  
  fn __repr__(&self) -> PyResult<String> {
    Ok(format!("{}", self))
  }
}

impl fmt::Display for PyTrackerHit {
  fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
    let mut repr = String::from("<PyTrackerHit:");
    repr += &(format!("\n  Row           : {}" ,self.row));
    repr += &(format!("\n  Module        : {}" ,self.module));
    repr += &(format!("\n  Channel       : {}" ,self.channel));
    repr += &(format!("\n  ADC           : {}" ,self.adc));
    repr += &(format!("\n  ASIC Ev. Code : {}>",self.asic_event_code));
    write!(f, "{}", repr)
  }
}

//// Implement the AsRef<PyAny> trait
//impl AsRef<PyAny> for PyTrackerHit {
//  fn as_ref(&self) -> &PyAny {
//    Python::with_gil(|py| {
//      let py_object = PyCell::new(py, self).unwrap();
//      py_object.as_ref(py)
//    })
//  }
//}

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

  /// Loop over the filtered hits, returning only those satisfying a condition
  ///
  /// # Arguments:
  /// 
  /// * filter : filter function - take input hit and decide if it should be 
  ///            returned.
  ///            E.g, this can be something like 
  ///            .filter_hits(lambda h : h.asic_event_code == 0 or h.asic_event_code ==2)
  pub fn filter_hits(&self, filter : &PyFunction) -> PyResult<Vec<PyTrackerHit>> {
    let mut filtered_hits = Vec::<PyTrackerHit>::new();
    for h in self.hits() {
      //let hit_ref = h.as_ref(py);
      let result : bool = filter.call1((h.clone(),))?.extract()?;
      if result {
        filtered_hits.push(h);
      }
    }
    Ok(filtered_hits)
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

#[pyclass]
#[pyo3(name="GPSPacket")]
struct PyGPSPacket {
  gps : tel_api::GPSPacket,
}

impl PyGPSPacket {
  pub fn set_gps(&mut self, gps : tel_api::GPSPacket) {
    self.gps = gps;
  }
}

#[pymethods]
impl PyGPSPacket {

  #[new]
  fn new() -> Self {
    Self {
      gps : tel_api::GPSPacket::new(),
    }
  }

  #[getter]
  fn utctime(&self) -> u32 {
    self.gps.utc_time
  }
  
  #[getter]
  fn gps_info(&self) -> u8 {
    self.gps.gps_info
  }

  /// Populate a GPSPacket from a TelemetryPacket.
  ///
  /// Telemetry packet type should be 82 (MergedEvent)
  fn from_telemetrypacket(&mut self, packet : PyTelemetryPacket) -> PyResult<()> {
    match tel_api::GPSPacket::from_bytestream(&packet.packet.payload, &mut 0) {
      Ok(gps) => {
        self.set_gps(gps);
        self.gps.telemetry_header = packet.packet.header.clone();
      }
      Err(err) => {
        return Err(PyValueError::new_err(err.to_string()));
      }  
    }
    Ok(())
  }

  fn __repr__(&self) -> PyResult<String> {
    Ok(format!("<PyO3Wrapper: {}>", self.gps))
  }
}

#[pyclass]
#[pyo3(name="TrackerTempLeakPacket")]
struct PyTrackerTempLeakPacket {
  tl : tel_api::TrackerTempLeakPacket,
}

impl PyTrackerTempLeakPacket {
  pub fn set_tl(&mut self, tl : tel_api::TrackerTempLeakPacket) {
    self.tl = tl;
  }
}

#[pymethods]
impl PyTrackerTempLeakPacket {

  #[new]
  fn new() -> Self {
    Self {
      tl : tel_api::TrackerTempLeakPacket::new(),
    }
  }

  #[getter]
  fn row_offset(&self) -> u8 {
    self.tl.row_offset
  }
  
  #[getter]
  fn temp_leak(&self) -> [[u32;6];6] {
    self.tl.templeak
  }
  
  #[getter]
  fn seu(&self) -> [[u32;6];6] {
    self.tl.seu
  }

  /// Populate a TrackerTempLeakPacket from a TelemetryPacket.
  ///
  /// Telemetry packet type should be 82 (MergedEvent)
  fn from_telemetrypacket(&mut self, packet : PyTelemetryPacket) -> PyResult<()> {
    match tel_api::TrackerTempLeakPacket::from_bytestream(&packet.packet.payload, &mut 0) {
      Ok(tl) => {
        self.set_tl(tl);
        self.tl.telemetry_header = packet.packet.header.clone();
      }
      Err(err) => {
        return Err(PyValueError::new_err(err.to_string()));
      }  
    }
    Ok(())
  }

  fn __repr__(&self) -> PyResult<String> {
    Ok(format!("<PyO3Wrapper: {}>", self.tl))
  }
}

#[pyclass]
#[pyo3(name="TrackerDAQTempPacket")]
struct PyTrackerDAQTempPacket {
  tp : tel_api::TrackerDAQTempPacket,
}

impl PyTrackerDAQTempPacket {
  pub fn set_tp(&mut self, tp : tel_api::TrackerDAQTempPacket) {
    self.tp = tp;
  }
}

#[pymethods]
impl PyTrackerDAQTempPacket {

  #[new]
  fn new() -> Self {
    Self {
      tp : tel_api::TrackerDAQTempPacket::new(),
    }
  }

  #[getter]
  fn rom_id(&self) -> [u64;256] {
    self.tp.rom_id
  }
  
  #[getter]
  fn temp(&self) -> [u16;256] {
    self.tp.temp
  }
  
  /// Populate a TrackerTempLeakPacket from a TelemetryPacket.
  ///
  /// Telemetry packet type should be 82 (MergedEvent)
  fn from_telemetrypacket(&mut self, packet : PyTelemetryPacket) -> PyResult<()> {
    let ptype = tel_api::TelemetryPacketType::from(packet.packet.header.ptype);
    if ptype != tel_api::TelemetryPacketType::AnyTrackerHK {
      return Err(PyValueError::new_err(format!("This is packet has type {}, but it should have {}", ptype, tel_api::TelemetryPacketType::AnyTrackerHK)));
    }
    if packet.packet.payload.len() <= 18 {
      return Err(PyValueError::new_err("StreamTooShort"));
    }
    match tel_api::TrackerDAQTempPacket::from_bytestream(&packet.packet.payload, &mut 0) {
      Ok(tp) => {
        self.set_tp(tp);
        self.tp.telemetry_header = packet.packet.header.clone();
      }
      Err(err) => {
        return Err(PyValueError::new_err(err.to_string()));
      }  
    }
    Ok(())
  }

  fn __repr__(&self) -> PyResult<String> {
    Ok(format!("<PyO3Wrapper: {}>", self.tp))
  }
}

#[pyclass]
#[pyo3(name="TrackerDAQHSKPacket")]
struct PyTrackerDAQHSKPacket {
  tp : tel_api::TrackerDAQHSKPacket,
}

impl PyTrackerDAQHSKPacket {
  pub fn set_tp(&mut self, tp : tel_api::TrackerDAQHSKPacket) {
    self.tp = tp;
  }
}

#[pymethods]
impl PyTrackerDAQHSKPacket {

  #[new]
  fn new() -> Self {
    Self {
      tp : tel_api::TrackerDAQHSKPacket::new(),
    }
  }

  #[getter]
  fn temp(&self) -> [u16;12] {
    self.tp.temp
  }
  
  /// Populate a TrackerTempLeakPacket from a TelemetryPacket.
  ///
  /// Telemetry packet type should be 82 (MergedEvent)
  fn from_telemetrypacket(&mut self, packet : PyTelemetryPacket) -> PyResult<()> {
    let ptype = tel_api::TelemetryPacketType::from(packet.packet.header.ptype);
    if ptype != tel_api::TelemetryPacketType::AnyTrackerHK {
      return Err(PyValueError::new_err(format!("This is packet has type {}, but it should have {}", ptype, tel_api::TelemetryPacketType::AnyTrackerHK)));
    }
    if packet.packet.payload.len() <= 18 {
      return Err(PyValueError::new_err("StreamTooShort"));
    }
    match tel_api::TrackerDAQHSKPacket::from_bytestream(&packet.packet.payload, &mut 0) {
      Ok(tp) => {
        self.set_tp(tp);
        self.tp.telemetry_header = packet.packet.header.clone();
      }
      Err(err) => {
        return Err(PyValueError::new_err(err.to_string()));
      }  
    }
    Ok(())
  }

  fn __repr__(&self) -> PyResult<String> {
    Ok(format!("<PyO3Wrapper: {}>", self.tp))
  }
}

#[pyclass]
#[pyo3(name="TrackerEventIDEchoPacket")]
struct PyTrackerEventIDEchoPacket {
  tp : tel_api::TrackerEventIDEchoPacket,
}

impl PyTrackerEventIDEchoPacket {
  pub fn set_tp(&mut self, tp : tel_api::TrackerEventIDEchoPacket) {
    self.tp = tp;
  }
}

#[pymethods]
impl PyTrackerEventIDEchoPacket {

  #[new]
  fn new() -> Self {
    Self {
      tp : tel_api::TrackerEventIDEchoPacket::new(),
    }
  }

  #[getter]
  fn temp(&self) -> [u16;12] {
    self.tp.temp
  }
  
  /// Populate a TrackerEventIDEchoPacket from a TelemetryPacket.
  ///
  /// Telemetry packet type should be 82 (MergedEvent)
  fn from_telemetrypacket(&mut self, packet : PyTelemetryPacket) -> PyResult<()> {
    let ptype = tel_api::TelemetryPacketType::from(packet.packet.header.ptype);
    if ptype != tel_api::TelemetryPacketType::AnyTrackerHK {
      return Err(PyValueError::new_err(format!("This is packet has type {}, but it should have {}", ptype, tel_api::TelemetryPacketType::AnyTrackerHK)));
    }
    if packet.packet.payload.len() <= 18 {
      return Err(PyValueError::new_err("StreamTooShort"));
    }
    match tel_api::TrackerEventIDEchoPacket::from_bytestream(&packet.packet.payload, &mut 0) {
      Ok(tp) => {
        self.set_tp(tp);
        self.tp.telemetry_header = packet.packet.header.clone();
      }
      Err(err) => {
        return Err(PyValueError::new_err(err.to_string()));
      }  
    }
    Ok(())
  }

  fn __repr__(&self) -> PyResult<String> {
    Ok(format!("<PyO3Wrapper: {}>", self.tp))
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
    m.add_class::<tel_api::TelemetryPacketType>()?;
    m.add_class::<PyTelemetryPacket>()?;
    m.add_class::<PyTelemetryPacketReader>()?;
    m.add_class::<PyMergedEvent>()?;
    m.add_class::<PyGapsEvent>()?;
    m.add_class::<PyTofHit>()?;
    m.add_class::<PyTofEventSummary>()?;
    m.add_class::<PyTrackerHit>()?;
    m.add_class::<PyTrackerEvent>()?;
    m.add_class::<PyTrackerPacket>()?;
    m.add_class::<PyTrackerTempLeakPacket>()?;
    m.add_class::<PyGPSPacket>()?;
    m.add_class::<PyTrackerDAQTempPacket>()?;
    m.add_class::<PyTrackerDAQHSKPacket>()?;
    m.add_class::<PyTrackerEventIDEchoPacket>()?;
    Ok(())
}

