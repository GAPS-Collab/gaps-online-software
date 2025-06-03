//! The following file is part of gaps-online-software and published 
//! under the GPLv3 license
//!
//! This file contains the source for pybindings with pyO3 for the 
//! caraspace i/o system

use std::collections::HashMap;

use pyo3::prelude::*;
use pyo3::types::{
  PyBytes,
};

//use log::error;
use caraspace::prelude::*;
use tof_dataclasses::database::{
  Paddle,
  TrackerStrip,
  get_tofpaddles,
  get_trackerstrips
};
use tof_dataclasses::packets::TofPacket;
//use tof_dataclasses::events::TofEventSummary;
use tof_dataclasses::events::TofEvent;
use telemetry_dataclasses::packets::{
  TelemetryPacket,
  MergedEvent
};

use crate::dataclasses::{
  PyTofPacket,
  PyTofEvent,
};

use crate::telemetry::{
  PyTelemetryPacket,
  PyMergedEvent,
};

use pyo3::exceptions::PyValueError;

/// Parse an u8 from python bytes. 
///
/// # Arguments:
///
/// * stream (bytes)  : parse the number from this stream
/// * start_pos (int) : begin parsing at this position 
#[pyfunction]
#[pyo3(name="parse_u8")]
pub fn py_parse_u8<'_py>(stream: Bound<'_py, PyBytes>, start_pos : usize) -> (u8, usize) {
  let bs : Vec<u8> = stream.extract().expect("Don't understand input!");
  let mut pos = start_pos;
  let value = parse_u8(&bs, &mut pos);
  (value, pos)
}

/// Parse an u16 from python bytes. 
///
/// # Arguments:
///
/// * stream (bytes)  : parse the number from this stream
/// * start_pos (int) : begin parsing at this position 
#[pyfunction]
#[pyo3(name="parse_u16")]
pub fn py_parse_u16<'_py>(stream: Bound<'_py, PyBytes>, start_pos : usize) -> (u16, usize) {
  let bs : Vec<u8> = stream.extract().expect("Don't understand input!");
  let mut pos = start_pos;
  let value = parse_u16(&bs, &mut pos);
  (value, pos)
}

/// Parse an u32 from python bytes. 
///
/// # Arguments:
///
/// * stream (bytes)  : parse the number from this stream
/// * start_pos (int) : begin parsing at this position 
#[pyfunction]
#[pyo3(name="parse_u32")]
pub fn py_parse_u32<'_py>(stream: Bound<'_py, PyBytes>, start_pos : usize) -> (u32, usize) {
  let bs : Vec<u8> = stream.extract().expect("Don't understand input!");
  let mut pos = start_pos;
  let value = parse_u32(&bs, &mut pos);
  (value, pos)
}

/// Parse an u64 from python bytes. 
///
/// # Arguments:
///
/// * stream (bytes)  : parse the number from this stream
/// * start_pos (int) : begin parsing at this position 
#[pyfunction]
#[pyo3(name="parse_u64")]
pub fn py_parse_u64<'_py>(stream: Bound<'_py, PyBytes>, start_pos : usize) -> (u64, usize) {
  let bs : Vec<u8> = stream.extract().expect("Don't understand input!");
  let mut pos = start_pos;
  let value = parse_u64(&bs, &mut pos);
  (value, pos)
}

/// The building blocks of the caraspace serialization 
/// library
///
/// A CRFrame is capable of storing multiple packets of 
/// any type.
#[pyclass]
#[pyo3(name="CRFrameObject")]
#[derive(Clone, Debug)]
pub struct PyCRFrameObject {
  frame_object : CRFrameObject
}

#[pymethods]
impl PyCRFrameObject {
  #[new]
  fn new() -> Self {
    Self {
      frame_object : CRFrameObject::new(),
    }
  }
    
  fn __repr__(&self) -> PyResult<String> {
    Ok(format!("<PyO3Wrapper: {}>", self.frame_object)) 
  }
}

/// The building blocks of the caraspace serialization 
/// library
///
/// A CRFrame is capable of storing multiple packets of 
/// any type.
#[pyclass]
#[pyo3(name="CRFrame")]
#[derive(Clone, Debug)]
pub struct PyCRFrame{
  frame   : CRFrame,
  paddles : HashMap<u8, Paddle>, 
  strips  : HashMap<u32, TrackerStrip>
}

#[pymethods]
impl PyCRFrame {
  #[new]
  fn new() -> Self {
    Self {
      frame   : CRFrame::new(),
      paddles : HashMap::<u8, Paddle>::new(),
      strips  : HashMap::<u32, TrackerStrip>::new(),
    }
  }
 
  fn put_telemetrypacket(&mut self, packet : PyTelemetryPacket, name : String) {
    let packet = packet.packet;
    self.frame.put(packet, name)
      //let packet = packet.p;
  }

  fn put_tofpacket(&mut self, packet : PyTofPacket, name : String) {
    let packet = packet.packet;
    self.frame.put(packet, name);
  }

  fn get_telemetrypacket(&mut self, name : String) -> PyResult<PyTelemetryPacket> {
    let mut py_packet = PyTelemetryPacket::new();
    let packet    = self.frame.get::<TelemetryPacket>(name).unwrap();
    py_packet.packet = packet;
    Ok(py_packet)
  }
  
  fn get_mergedevent(&mut self, name : String) -> PyResult<PyMergedEvent> {
    let mut py_event    = PyMergedEvent::new();
    let packet        = self.frame.get::<TelemetryPacket>(name).map_err(|_| pyo3::exceptions::PyValueError::new_err("Merged Event not found"))?;
    match MergedEvent::from_bytestream(&packet.payload, &mut 0) {
      Ok(mut event) => {
        event.tof_event.set_paddles(&self.paddles);
        event.tof_event.normalize_hit_times();
        py_event.event        = event;
        py_event.event.header = packet.header.clone();
      }
      Err(err) => {
        return Err(PyValueError::new_err(err.to_string()));
      }
    }
    for h in &mut py_event.event.tracker_hitsv2 {
      h.set_coordinates(&self.strips);
    }
    Ok(py_event)
  }
  
  fn get_tofevent(&mut self, name : String) -> PyResult<PyTofEvent> {
    let mut py_event  = PyTofEvent::new();
    // FIXME
    let packet    = self.frame.get::<TofPacket>(name).unwrap();
    let mut event = packet.unpack::<TofEvent>().unwrap();
    event.set_paddles(&self.paddles);
    py_event.event  = event;
    Ok(py_event)
  }

  fn get_tofpacket(&mut self, name : String) -> PyResult<PyTofPacket> {
    let mut py_packet = PyTofPacket::new();
    // FIXME
    let packet    = self.frame.get::<TofPacket>(name).unwrap();
    py_packet.packet = packet;
    Ok(py_packet)
  }
  //fn put(&mut self, stream :  Vec<u8>, name : String) {
  //  let mut bs = stream.clone();
  //  self.frame.put_stream(&mut bs, name);
  //}

  /// Check if the frame contains an object with the given name
  ///
  /// # Arguments:
  ///   * name : The name of the object as it appears in the index
  fn has(&self, name : &str) -> bool {
    self.frame.has(name)
  }

  #[getter]
  fn index(&self) -> HashMap<String, (u64, CRFrameObjectType)> {
    self.frame.index.clone()
  }
  
  fn __repr__(&self) -> PyResult<String> {
    Ok(format!("<PyO3Wrapper: {}>", self.frame)) 
  }
}

/// Read caraspace files. Caraspace files are an aggregate filetype
/// which allows to hold information from multiple sources in an 
/// efficient binary format. 
/// For the use within the GAPS experiment, L0 files are caraspace files
/// and contain the TOF waveforms as sourced by the files written by the 
/// TOFComputer to disk as well as the files emitted by the flight 
/// computer (telemetry).
///
/// To create a new CRReader, simply call
/// CRReader(filename_or_directory : path/string) where the argument can be either a name
/// of an existing file or a directory with caraspace files.
#[pyclass]
#[pyo3(name="CRReader")]
pub struct PyCRReader {
  reader  : CRReader,
  paddles : HashMap<u8,Paddle>,
  strips  : HashMap<u32, TrackerStrip>
}

#[pymethods]
impl PyCRReader {
  #[new]
  fn new(filename_or_directory : &Bound<'_,PyAny>) -> PyResult<Self> {
    let mut string_value = String::from("foo");
    if let Ok(s) = filename_or_directory.extract::<String>() {
       string_value = s;
    } //else if let Ok(p) = filename_or_directory.extract::<&Path>() {
    if let Ok(fspath_method) = filename_or_directory.getattr("__fspath__") {
      if let Ok(fspath_result) = fspath_method.call0() {
        if let Ok(py_string) = fspath_result.extract::<String>() {
          string_value = py_string;
        }
      }
    }

    //   string_value = p.display().to_string();
    //} else {
    //   return Err(pyo3::exceptions::PyTypeError::new_err(
    //     "Expected a string or a path-like object",
    //   ));
    //}
    let mut paddles = HashMap::<u8, Paddle>::new();
    let mut strips  = HashMap::<u32, TrackerStrip>::new();
    match get_tofpaddles() {
      Ok(pdls) => {
        paddles = pdls;
      }
      Err(_err) => {
        // FIXME!
        //error!("Unable to get paddles from database. Maybe the 'DATABASE_URL' path is not set. Are you sure you loaded the setup-env.sh shell?");
      }
    }
    match get_trackerstrips() {
      Ok(_strips) => {
        strips = _strips;
      }
      Err(_err) => {
        // FIXME!
        //error!("Unable to get paddles from database. Maybe the 'DATABASE_URL' path is not set. Are you sure you loaded the setup-env.sh shell?");
      }
    }
    Ok(Self {
      reader  : CRReader::new(string_value)?,
      paddles : paddles,
      strips  : strips,
    })
  }

  /// This is the filename we are currently 
  /// extracting frames from 
  #[getter]
  fn get_current_filename(&self) -> Option<String> {
    self.reader.get_current_filename()
  }

  /// Start the reader from the beginning
  /// This is equivalent to a re-initialization
  /// of that reader.
  fn rewind(&mut self) -> PyResult<()> {
    match self.reader.rewind() {
      Err(err) => {
        return Err(PyValueError::new_err(err.to_string()));
      }
      Ok(_) => Ok(())
    }
  }

  fn __iter__(slf: PyRef<'_, Self>) -> PyRef<'_, Self> {
    slf 
  }
  
  fn __next__(mut slf: PyRefMut<'_, Self>) -> Option<PyCRFrame> {
    match slf.reader.next() { 
      Some(frame) => {
        let mut pyframe = PyCRFrame::new();
        pyframe.frame = frame;
        //FIXME - these are huge! This needs to be solved
        //by some other method
        pyframe.paddles = slf.paddles.clone();
        pyframe.strips  = slf.strips.clone();
        return Some(pyframe)
      }   
      None => {
        return None;
      }   
    }   
  }

  /// Get the number of frames this reader can walkthrough.
  /// Since this is going through all files, it might take
  /// a long time
  fn count_frames(&mut self) -> usize {
    self.reader.get_n_frames()
  }

  #[getter]
  fn get_first_frame(&mut self) -> Option<PyCRFrame> {
    match self.reader.first_frame() {
      Some(frame) => {
        let mut pyframe = PyCRFrame::new();
        pyframe.frame = frame;
        return Some(pyframe);
      }
      None => {
        return None;
      }
    }
  }
  
  #[getter]
  fn get_last_frame(&mut self) -> Option<PyCRFrame> {
    match self.reader.last_frame() {
      Some(frame) => {
        let mut pyframe = PyCRFrame::new();
        pyframe.frame = frame;
        return Some(pyframe);
      }
      None => {
        return None;
      }
    }
  }

  fn __repr__(&self) -> PyResult<String> {
    Ok(format!("<PyO3Wrapper: {}>", self.reader)) 
  }
}

#[pyclass]
#[pyo3(name="CRWriter")]
pub struct PyCRWriter {
  writer : CRWriter
}

#[pymethods]
impl PyCRWriter {
  #[new]
  #[pyo3(signature = (filename, run_id, timestamp = None))]
  fn new(filename : String, run_id : u32, timestamp : Option<String>) -> Self {
    Self {
      writer : CRWriter::new(filename, run_id, timestamp ),
    }
  }
  
  fn set_file_timestamp(&mut self, timestamp : String) {
    self.writer.file_timestamp = Some(timestamp);
  }
  
  fn add_frame(&mut self, frame : PyCRFrame) {
    self.writer.add_frame(&frame.frame);  
  }
}


