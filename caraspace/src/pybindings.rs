use std::collections::HashMap;

use pyo3::prelude::*;
use pyo3::types::PyBytes;

use crate::prelude::*;


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

#[cfg(feature="pybindings")]
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

#[cfg(feature="pybindings")]
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

#[cfg(feature="pybindings")]
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
#[pyo3(name="CRFrame")]
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

