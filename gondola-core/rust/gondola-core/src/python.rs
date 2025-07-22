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

fn convert_pyarray1<'_py>(arr : Bound<'_py, PyArray1<f32>>) -> Vec<f32> {
  let mut vec = Vec::<f32>::new();
  unsafe {
    vec.extend_from_slice(arr.as_slice().unwrap());
  }
  return vec;
}

//--------------------------------------------------

