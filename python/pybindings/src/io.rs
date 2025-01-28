use std::collections::HashMap;

use tof_dataclasses::io as io_api;

use pyo3::prelude::*;
//use pyo3::exceptions::PyValueError;

use tof_dataclasses::packets::PacketType;

use crate::dataclasses::PyTofPacket;



/// Remove the waveforms from a .tof.gaps 
/// file and replace TofEvents with
/// TofEventSummary
#[pyfunction]
#[pyo3(name="summarize_toffile")]
pub fn py_summarize_toffile(fname : String) {
  io_api::summarize_toffile(fname);
}

#[pyclass]
#[pyo3(name="TofPacketReader")]
pub struct PyTofPacketReader {
  reader : io_api::TofPacketReader,
}

#[pymethods]
impl PyTofPacketReader {
  
  /// Create a new instance of a TofPacketReader. 
  #[new]
  #[pyo3(signature = (filename, filter=PacketType::Unknown,start=0, nevents=0))]
  fn new<'py>(filename : Bound<'py, PyAny>, filter : PacketType, start : usize, nevents : usize) -> PyResult<Self> {
    let input_str : String;
    match filename.extract::<String>() {
      Ok(_fname) => {
        input_str = _fname;
      }
      Err(_) => {
        match filename.extract::<std::path::PathBuf>() {
          Ok(_fname) => {
            input_str = _fname.to_str().expect("Unable to convert input to string!").to_owned();
          }
          Err(_) => {
          return Err(pyo3::exceptions::PyTypeError::new_err(
              "Expected str or pathlib.Path",));
          }
        }
      }
    }

    let mut pyreader = Self {
      reader : io_api::TofPacketReader::new(input_str),
    };
    pyreader.reader.filter     = filter;
    pyreader.reader.skip_ahead = start;
    pyreader.reader.stop_after = nevents;
    Ok(pyreader)
  }

  #[getter]
  fn packet_index(&mut self) -> PyResult<HashMap<PacketType, usize>> {
    let idx = self.reader.get_packet_index()?;
    self.reader.rewind()?;
    Ok(idx)
  }

  fn rewind(&mut self) {
    let _ = self.reader.rewind();
  }

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
