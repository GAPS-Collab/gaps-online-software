use tof_dataclasses::io as io_api;

use pyo3::prelude::*;
//use pyo3::exceptions::PyValueError;

use crate::dataclasses::PyTofPacket;

#[pyclass]
#[pyo3(name="TofPacketReader")]
pub struct PyTofPacketReader {
  reader : io_api::TofPacketReader,
}

#[pymethods]
impl PyTofPacketReader {
  
  #[new]
  fn new(filename : String) -> Self {
    Self {
      reader : io_api::TofPacketReader::new(filename),
    }
  }

  //fn set_filter(&self) {
  //}

  fn __repr__(&self) -> PyResult<String> {
    Ok(format!("<PyO3Wrapper: {}>", self.reader)) 
  }
  
  fn __iter__(slf: PyRef<'_, Self>) -> PyRef<'_, Self> {
    slf
  }
  
  fn __next__(mut slf: PyRefMut<'_, Self>) -> Option<PyTofPacket> {
    match slf.reader.next() { 
      Some(tp) => {
        let mut pytp = PyTofPacket::new();
        pytp.set_tp(tp);
        return Some(pytp)
      }
      None => {
        return None;
      }
    }
    //  slf.reader.next()
  }
}
