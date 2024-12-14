//! The wrapped commands from the 
//! command factory
//!
//!
//!

use std::collections::HashMap;

use pyo3::prelude::*;
use pyo3::exceptions::PyValueError;

use tof_dataclasses::commands::{
  get_rbratmap_hardcoded,
  get_ratrbmap_hardcoded,
  get_ratpdumap_hardcoded,
  shutdown_rb,
  shutdown_rat,
  shutdown_ratpair
};

use crate::PyTofCommand;

#[pyfunction]
#[pyo3(name="get_rbratmap_hardcoded")]
pub fn py_get_rbratmap_hardcoded() -> HashMap::<u8,u8> {
  get_rbratmap_hardcoded()
}

#[pyfunction]
#[pyo3(name="get_ratrbmap_hardcoded")]
pub fn py_get_ratrbmap_hardcoded() -> HashMap::<u8,(u8,u8)> {
  get_ratrbmap_hardcoded()
}

#[pyfunction]
#[pyo3(name="get_ratpdumap_hardcoded")]
pub fn py_get_ratpdumap_hardcoded() -> HashMap::<u8,HashMap::<u8,(u8,u8)>> {
  get_ratpdumap_hardcoded()
}

#[pyfunction]
#[pyo3(name="shutdown_rb")]
pub fn py_shutdown_rb(rb : u8) -> PyResult<PyTofCommand> {
  let cmd = shutdown_rb(rb).unwrap();
  Ok(PyTofCommand { 
    command : cmd
  })
}

#[pyfunction]
#[pyo3(name="shutdown_rat")]
pub fn py_shutdown_rat(rat : u8) -> PyResult<PyTofCommand> {
  match shutdown_rat(rat) {
    None => {
      return Err(PyValueError::new_err(format!("There might not be a RAT{}!", rat)));
    }
    Some(cmd) => {
      let pycmd = PyTofCommand { 
       command : cmd
      };
      return Ok(pycmd);
    }
  }
}

#[pyfunction]
#[pyo3(name="shutdown_rat")]
pub fn py_shutdown_ratpair(pdu : u8, pduchannel : u8) -> PyResult<PyTofCommand> {
  match shutdown_ratpair(pdu, pduchannel) {
    None => {
      return Err(PyValueError::new_err(format!("There might be an issue with the pdu mapping. Can nto find RATs at PDU {} channel {}!", pdu, pduchannel)));
    }
    Some(cmd) => {
      let pycmd = PyTofCommand { 
       command : cmd
      };
      return Ok(pycmd);
    }
  }
}

