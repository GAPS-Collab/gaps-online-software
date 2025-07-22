//! The following file is part of gaps-online-software and published 
//! under the GPLv3 license
//! 
//! Statistics tools
//!
//!

use statrs::distribution::{Gamma, Continuous};

#[cfg(feature="pybindings")]
use pyo3::prelude::*;

#[cfg(feature="pybindings")]
use numpy::{
  PyArray1,
  PyArrayMethods,
};

pub fn gamma_pdf(xs : &[f32], shape : f64, scale : f64) -> Vec<f32> {
  let mut ys = Vec::<f32>::with_capacity(xs.len());
  
  let gamma = Gamma::new(shape, scale).unwrap();
  for x in xs {
    ys.push(gamma.pdf(*x as f64) as f32);
  }
  return ys;
}

//---------------------------------------------------

#[cfg(feature="pybindings")]
#[pyfunction]
#[pyo3(name="gamma_pdf")]
pub fn py_gamma_pdf<'_py>(xs    : Bound<'_py,PyArray1<f32>>,
                          shape : f64,
                          scale : f64) -> Vec<f32> {
  let ys : Vec::<f32>;
  unsafe {
    ys = gamma_pdf(xs.as_slice().unwrap(), shape, scale);
  }
  return ys;
}

