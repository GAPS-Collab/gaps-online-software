//! The following file is part of gaps-online-software and published 
//! under the GPLv3 license
//!
//! Calibration routines for the GAPS tracker system

/// Provide an interface to convert the 
/// ADC values for the Si(Li) modules
/// to a voltage drop. (Reversal of the 
/// ASIC digitization). This voltage drop
/// can then be further used to calculate the 
/// energy deposition in the tracker strip.
/// 
/// The function fn(ADC) -> voltage is 
/// commonly called "TransferFunction"
/// 
/// The actual function is piecewise defined
/// for different ranges of the ADC value 
/// and can be represented by a polynomial fit
pub struct TrackerTransferFn {
  /// Each strip has its own transer function. 
  /// Identifier for the strip (hardware id)
  pub strip_id     : u32,
  /// 2nd order poly on ADC  [0,190)   
  pub pol_a_params : [f32;3],
  /// 3rd order poly on ADC  [190,500) 
  pub pol_b_params : [f32;4],
  /// 3rd order poly on ADC  [500,900) 
  pub pol_c_params : [f32;4],
  /// 3rd order poly on ADC  [9001600) 
  pub pol_d_params : [f32;4],
}

impl TrackerTransferFn {

  pub fn new() -> Self {
    Self {
      strip_id     : 0,
      pol_a_params : [0.0;3],
      pol_b_params : [0.0;4],
      pol_c_params : [0.0;4],
      pol_d_params : [0.0;4],
    }
  }
}

//use pyo3::prelude::*;
//use pyo3::exceptions::{
//  PyKeyError,
//  PyValueError,
//  PyIOError,
//};
//
//
///// Representation of all information to convert 
///// Tracker ADC to energy. This is commonly 
///// summarized under the umbrella term
///// "TrackerTransferFunction"
//#[pyclass]
//#[pyo3(name="TrkTransferFn")]
//pub struct PyTrkTransferFn {
//  pub ydata : Vec<f32>,
//  pub xdata : Vec<f32>,
//}
//
//#[pymethods]
//impl PyTrkTransferFn {
//
//  #[new]
//  pub fn new() -> Self {
//    Self {
//      ydata : Vec::<f32>::new(),
//      xdata : Vec::<f32>::new(),
//    }
//  }
//
//  #[getter]
//  fn get_xmin(&self) -> f32 {
//    0.0
//  }
//  
//  #[getter]
//  fn get_xmax(&self) -> f32 {
//    0.0
//  }
//  
//  #[getter]
//  fn get_ymin(&self) -> f32 {
//    0.0
//  }
//  
//  #[getter]
//  fn get_ymax(&self) -> f32 {
//    0.0
//  }
//
//  fn get_response(&self, x : f32) -> f32 {
//    0.0
//  }
//}
//
//
