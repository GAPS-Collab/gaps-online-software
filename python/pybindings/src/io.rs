use std::collections::HashMap;
use std::env;
//use log::error;
use tof_dataclasses::io as io_api;

use pyo3::prelude::*;
use pyo3::exceptions::PyIOError;
//use pyo3::exceptions::PyValueError;

use tof_dataclasses::packets::PacketType;
use tof_dataclasses::database::{
  Paddle,
  connect_to_db
};

use crate::dataclasses::{
  PyTofPacket,
  PyTofEventSummary
};

/// Remove the waveforms from a .tof.gaps 
/// file and replace TofEvents with
/// TofEventSummary
#[pyfunction]
#[pyo3(name="summarize_toffile")]
pub fn py_summarize_toffile(fname : String) {
  io_api::summarize_toffile(fname);
}

///// New style, agnostic reader for events from 
///// any source
//#[pyclass]
//#[pyo3(name="Adapter")]
//pub struct PyAdapter {
//  pub paddles  : HashMap<u8, Paddle>,
//  pub tpreader : Option<PyTofPacketReader>
//}
//
//#[pymethods]
//impl PyAdapter {
//
//  /// Instanciate a new adapter. An adapter can connect to any type of (online) data
//  /// file - .tof.gaps, .tofsum.gaps, telemetry (.bin), a network port or a directory with 
//  /// these files
//  #[new]
//  #[pyo3(signature = (source, filter=PacketType::Unknown,start=0, nevents=0))]
//  fn new<'py>(source : Bound<'py, PyAny>, filter : PacketType, start : usize, nevents : usize) -> PyResult<Self> {
//    let reader  = PyTofPacketReader::new(source, filter, start, nevents).expect("Can not init reader!");
//    let mut paddles = HashMap::<u8, Paddle>::new();
//    let db_path = env::var("DATABASE_URL").unwrap_or_else(|_| "".to_string());
//    match connect_to_db(db_path) {
//      Err(err) => {
//        println!("Database can not be found! Did you load the setup-env.sh shell?");
//        return Err(PyIOError::new_err(err.to_string()));
//      }
//      Ok(mut conn) => {
//        match Paddle::all(&mut conn) {
//          None => {
//            return Err(PyIOError::new_err("Unable to retrieve paddle information from DB!"));
//          }
//          Some(pdls) => {
//            for p in pdls {
//              paddles.insert(p.paddle_id as u8, p.clone());
//            }
//          }
//        }
//      }
//    }
//
//    Ok(PyAdapter {
//      paddles  : paddles,
//      tpreader : Some(reader)
//    })
//  }  
//  
//  #[getter]
//  fn packet_index(&mut self) -> PyResult<HashMap<PacketType, usize>> {
//    if self.tpreader.is_some() {
//      let idx = self.tpreader.get_packet_index()?;
//      self.tpreader.rewind()?;
//      Ok(idx)
//    }
//    Err(PyIOError::new_err("No reader is set! Can't return packet index!"));
//  }
//
//  fn rewind(&mut self) {
//    let _ = self.tpreader.rewind();
//  }
//
//  fn __repr__(&self) -> PyResult<String> {
//    if self.tpreader.is_some() {
//      Ok(format!("<PyO3Wrapper: {}>", self.tpreader.unwrap())) 
//    } else {
//      Err(PyIOError::new_err("No reader is set! Can't return packet index!"));
//    }
//  }
//  
//  fn __iter__(slf: PyRef<'_, Self>) -> PyRef<'_, Self> {
//    slf
//  }
//  
//  pub fn __next__(mut slf: PyRefMut<'_, Self>) -> Option<PyTofPacket> {
//    if slf.tpreader.is_some() {
//      //match slf.tpreader.unwrap().reader.next() { 
//      match <Option<PyTofPacketReader> as Clone>::clone(&slf.tpreader).unwrap().reader.next() {
//        Some(tp) => {
//          let mut pytp = PyTofPacket::new();
//          pytp.set_tp(tp);
//          return Some(pytp)
//        }
//        None => {
//          return None;
//        }
//      }
//    }
//    None
//  }
//}

#[pyclass]
#[pyo3(name="TofPacketReader")]
pub struct PyTofPacketReader {
  pub reader          : io_api::TofPacketReader,
  pub paddles         : HashMap<u8,Paddle>,
  //pub with_paddleinfo : bool
}

#[pymethods]
impl PyTofPacketReader {
  
  /// Create a new instance of a TofPacketReader. 
  #[new]
  #[pyo3(signature = (filename, filter=PacketType::Unknown,start=0, nevents=0))]
  pub fn new<'py>(filename : Bound<'py, PyAny>, filter : PacketType, start : usize, nevents : usize) -> PyResult<Self> {
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
    let mut paddles = HashMap::<u8, Paddle>::new();
    let db_path = env::var("DATABASE_URL").unwrap_or_else(|_| "".to_string());
    match connect_to_db(db_path) {
      Err(err) => {
        println!("Database can not be found! Did you load the setup-env.sh shell?");
        return Err(PyIOError::new_err(err.to_string()));
      }
      Ok(mut conn) => {
        match Paddle::all(&mut conn) {
          None => {
            return Err(PyIOError::new_err("Unable to retrieve paddle information from DB!"));
          }
          Some(pdls) => {
            for p in pdls {
              paddles.insert(p.paddle_id as u8, p.clone());
            }
          }
        }
      }
    }

    let mut pyreader = Self {
      reader          : io_api::TofPacketReader::new(input_str),
      paddles         : paddles,
      //with_paddleinfo : false
    };
    pyreader.reader.filter     = filter;
    pyreader.reader.skip_ahead = start;
    pyreader.reader.stop_after = nevents;
    Ok(pyreader)
  }
 
  fn add_paddleinfo(&self, event : &mut PyTofEventSummary) {
    event.event.set_paddles(&self.paddles);
  }

  #[getter]
  fn first(&mut self) -> Option<PyTofPacket> {
    let mut ptp = PyTofPacket::new();
    let tp = self.reader.first_packet()?;
    ptp.packet = tp;
    return Some(ptp);
  }

  #[getter]
  fn last(&mut self) -> Option<PyTofPacket> {
    let mut ptp = PyTofPacket::new();
    let tp = self.reader.last_packet()?;
    ptp.packet = tp;
    return Some(ptp);
  }
  //#[getter]
  //fn get_with_paddleinfo(&self) -> PyResult<u32> {
  //  Ok(self.with_paddleinfo)
  //}
  //
  //#[setter]
  //fn set_with_paddleinfo(&mut self, pinfo : bool) -> PyResult<()> {
  //  self.with_paddleinfo = pinfo;
  //  Ok(())
  //}

  #[getter]
  fn packet_index(&mut self) -> PyResult<HashMap<PacketType, usize>> {
    let idx = self.reader.get_packet_index()?;
    self.reader.rewind()?;
    Ok(idx)
  }

  fn rewind(&mut self) {
    let _ = self.reader.rewind();
  }

  pub fn __repr__(&self) -> PyResult<String> {
    Ok(format!("<PyO3Wrapper: {}>", self
            .reader)) 
  }
  
  pub fn __iter__(slf: PyRef<'_, Self>) -> PyRef<'_, Self> {
    slf
  }
  
  pub fn __next__(mut slf: PyRefMut<'_, Self>) -> Option<PyTofPacket> {
    match slf.reader.next() { 
      Some(tp) => {
        let mut pytp = PyTofPacket::new();
        if tp.packet_type == PacketType::TofEventSummary {
          //if tp.with_paddleinfo {
          //
          //}
        }
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
