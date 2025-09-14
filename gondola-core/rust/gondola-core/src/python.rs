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

//use numpy::{
//  PyArray1,
//  PyArrayMethods
//};
//use pyo3::Bound;

/// Adds the __repr__  and __str__ functions to 
/// a pybindings wrapped class
#[macro_export]
macro_rules! pythonize_display {
  ($pyclass:ty) => {
    #[pymethods]
    impl $pyclass {
      fn __repr__(&self) -> PyResult<String> {
          Ok(format!("<{}: {}>", stringify!($pyclass), self.to_string()))
      }
      fn __str__(&self) -> PyResult<String> {
          Ok(format!("<{}: {}>", stringify!($pyclass), self.to_string()))
      }
    }
  };
}

/// Adds the __repr__  and __str__ functions to 
/// a pybindings wrapped class
#[macro_export]
macro_rules! pythonize {
  ($pyclass:ty) => {
    pythonize_display!($pyclass);

    #[pymethods]
    impl $pyclass {

      #[new]
      fn new_py() -> Self {
        Self::new()
      }

    }
  };
}

#[macro_export]
macro_rules! pythonize_telemetry {
  ($pyclass:ty) => {
    
    pythonize!($pyclass);

    #[pymethods]
    impl $pyclass {
      /// Unpack Self from a TelemetryPacket.
      #[staticmethod]
      fn from_telemetrypacket(packet : TelemetryPacket) -> PyResult<Self> {
        match Self::from_bytestream(&packet.payload, &mut 0) {
          Ok(mut tl) => {
            tl.telemetry_header = packet.header.clone();
            return Ok(tl);
          }
          Err(err) => {
            return Err(PyValueError::new_err(err.to_string()));
          }  
        }
      } 
    }
  }
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
/// have, but don't set a constructor
#[macro_export]
macro_rules! pythonize_packable_no_new {
  ($pyclass:ty) => {
    //use pyo3::prelude::*;

    #[pymethods]
    impl $pyclass {

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
      
      /// Pack self into a TofPacket to be written on disk
      /// or sent over network
      #[pyo3(name="pack")]
      fn pack_py(&self) -> TofPacket {
        self.pack()
      }

      #[cfg(feature="random")]
      #[staticmethod]
      #[pyo3(name="from_random")]
      fn from_random_py() -> Self {
        Self::from_random()
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
    }

    pythonize_packable_no_new!($pyclass);
  };
}



//fn convert_pyarray1<'_py>(arr : Bound<'_py, PyArray1<f32>>) -> Vec<f32> {
//  let mut vec = Vec::<f32>::new();
//  unsafe {
//    vec.extend_from_slice(arr.as_slice().unwrap());
//  }
//  return vec;
//}

//--------------------------------------------------

