//! Pybindings for the caraspace library

use std::collections::HashMap;

use pyo3::prelude::*;
use pyo3::types::PyBytes;

use caraspace::prelude::*;

use tof_dataclasses::packets::TofPacket;
use telemetry_dataclasses::packets::TelemetryPacket;
use crate::dataclasses::{
  PyTofPacket,
};

use crate::telemetry::{
  PyTelemetryPacket,
};

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
pub struct PyCRFrame {
  frame : CRFrame
}

#[pymethods]
impl PyCRFrame {
  #[new]
  fn new() -> Self {
    Self {
      frame : CRFrame::new(),
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

  #[getter]
  fn index(&self) -> HashMap<String, (u64, CRFrameObjectType)> {
    self.frame.index.clone()
  }
  
  fn __repr__(&self) -> PyResult<String> {
    Ok(format!("<PyO3Wrapper: {}>", self.frame)) 
  }
}

/// Read a file written by CRWriter containing 
/// frames in a subsequent fashion
#[pyclass]
#[pyo3(name="CRReader")]
pub struct PyCRReader {
  reader : CRReader
}

#[pymethods]
impl PyCRReader {
  #[new]
  fn new(filename : String) -> Self {
    Self {
      reader : CRReader::new(filename),
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
        return Some(pyframe)
      }   
      None => {
        return None;
      }   
    }   
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
  fn new(filename : String, run_id : u32) -> Self {
    Self {
      writer : CRWriter::new(filename, run_id),
    }
  }
  
  fn add_frame(&mut self, frame : PyCRFrame) {
    self.writer.add_frame(&frame.frame);  
  }
}


