//! The following file is part of gaps-online-software and published 
//! under the GPLv3 license

//use std::fmt::Display;
//use pyo3::PyResult;

//pub trait Pythonize {
// 
//  fn get_wrapped(&self) -> &dyn Display;
//
//  fn __repr__(&self) -> PyResult<String> {
//    Ok(format!("<PyO3Wrapper: {}>", self.get_wrapped())) 
//  }
//}

use numpy::{
  PyArray1,
  PyArrayMethods
};
use pyo3::Bound;

/// Adds the __repr__  and __str__ functions to 
/// a pybindings wrapped class
#[macro_export]
macro_rules! pythonize {
  ($pyclass:ty) => {
    //use pyo3::prelude::*;

    #[pymethods]
    impl $pyclass {

      #[new]
      fn new_py() -> Self {
        Self::new()
      }

      fn __repr__(&self) -> PyResult<String> {
          Ok(format!("<{}: {}>", stringify!($pyclass), self.to_string()))
      }
      fn __str__(&self) -> PyResult<String> {
          Ok(format!("<{}: {}>", stringify!($pyclass), self.to_string()))
      }
    }
  };
}

#[macro_export]
macro_rules! pythonize_monidata {
  ($pyclass:ty) => {

    #[pymethods]
    impl $pyclass {
    
      #[getter]
      #[pyo3(name="board_id")]
      fn board_id_py(&self) -> u8 {
        self.get_board_id()
      }


      #[staticmethod]
      #[pyo3(name="keys")]
      fn keys_py() -> Vec<&'static str> {
        Self::keys()
      }

      /// Access the (data) members by name
      #[pyo3(name="get")]
      fn get_py(&self, varname : &str) -> PyResult<f32> {
        match self.get(varname) {
          None => {
            let err_msg = format!("{} does not have a key with name {}! See {}.keys() for a list of available keys!",stringify!($pyclass), stringify!($pyclass), varname);
            return Err(PyKeyError::new_err(err_msg));
          }
          Some(val) => {
            return Ok(val)
          }
        }
      }
    }
  }
}

/// Adds common features any class exposed 
/// obeying the TofPackable trait should 
/// have
#[macro_export]
macro_rules! pythonize_packable {
  ($pyclass:ty) => {
    //use pyo3::prelude::*;

    #[pymethods]
    impl $pyclass {

      #[new]
      fn new_py() -> Self {
        Self::new()
      }
      
      #[staticmethod]
      #[pyo3(name="from_tofpacket")]
      fn from_tofpacket_py(packet : &TofPacket) -> PyResult<Self> {
        match packet.unpack::<Self>() {
          Ok(status) => {
            return Ok(status);
          }
          Err(err) => {
            let err_msg = format!("Unable to unpack TofPacket! {err}");
            return Err(PyIOError::new_err(err_msg));
          }
        }
      }

      fn __repr__(&self) -> PyResult<String> {
        Ok(format!("<PyO3Wrapper: {}>", self.to_string()))
      }
      fn __str__(&self) -> PyResult<String> {
        Ok(format!("<PyO3Wrapper: {}>", self.to_string()))
      }
    }
  };
}

/// Adds the __repr__  and __str__ functions to 
/// a pybindings wrapped class
#[macro_export]
macro_rules! impl_pythonize_display {
  ($pyclass:ty, $getter:expr) => {
    //use pyo3::prelude::*;

    #[pymethods]
    impl $pyclass {
      fn __repr__(&self) -> PyResult<String> {
          Ok(format!("<{}: {}>", stringify!($pyclass), $getter(self)))
      }
      fn __str__(&self) -> PyResult<String> {
          Ok(format!("<{}: {}>", stringify!($pyclass), $getter(self)))
      }
    }
  };
}

//--------------------------------------------------

//fn convert_pyarray1<'_py>(arr : Bound<'_py, PyArray1<f32>>) -> Vec<f32> {
//  let mut vec = Vec::<f32>::new();
//  unsafe {
//    vec.extend_from_slice(arr.as_slice().unwrap());
//  }
//  return vec;
//}

//--------------------------------------------------

