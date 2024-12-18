//! The wrapped commands from the 
//! command factory
//!
//!
//!

use std::collections::HashMap;

use pyo3::prelude::*;
use pyo3::exceptions::PyValueError;

use tof_dataclasses::commands::factory::*;

use crate::{
  PyTofCommand,
  PyTriggerConfig,
  PyTOFEventBuilderConfig,
};

/// A hardwired map of RB -> RAT
#[pyfunction]
#[pyo3(name="get_rbratmap_hardcoded")]
pub fn py_get_rbratmap_hardcoded() -> HashMap::<u8,u8> {
  get_rbratmap_hardcoded()
}

/// A hardwired map of RAT -> (RB1, RB2)
#[pyfunction]
#[pyo3(name="get_ratrbmap_hardcoded")]
pub fn py_get_ratrbmap_hardcoded() -> HashMap::<u8,(u8,u8)> {
  get_ratrbmap_hardcoded()
}

/// A hardwired map of PDU #id PDUCHANNEL #id to (RAT,RAT)
///
/// Can be used to synchronize powering down proces for 
/// RATs
#[pyfunction]
#[pyo3(name="get_ratpdumap_hardcoded")]
pub fn py_get_ratpdumap_hardcoded() -> HashMap::<u8,HashMap::<u8,(u8,u8)>> {
  get_ratpdumap_hardcoded()
}

/// Send the 'sudo shutdown now' command to a single RB
///
/// # Arguements:
///   * rb :  The RB id of the RB to be shutdown 
///           (NOT RAT)
#[pyfunction]
#[pyo3(name="shutdown_rb")]
pub fn py_shutdown_rb(rb : u8) -> PyResult<PyTofCommand> {
  let cmd = shutdown_rb(rb).unwrap();
  Ok(PyTofCommand { 
    command : cmd
  })
}

/// Send the 'sudo shutdown now' command to all RBs in a RAT
///
/// # Arguments:
///   * rat : The RAT id for the rat the RBs to be 
///           shutdown live in 
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

/// Send the 'sudo shutdown now' command to all RBs 
/// in the 2 RATs connected to a certain PDU channel
/// 
/// This will prepare the shutdown command for the RBs in the 
/// RATs which are connected to a specific pdu channel
///
/// # Arguments:
///   * pdu        : PDU ID (0-3)
///   * pduchannel : PDU Channel (0-7)
#[pyfunction]
#[pyo3(name="shutdown_ratpair")]
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

/// Send the 'sudo shutdown now command to
/// the TOF main computer ("TOFCPU")
#[pyfunction]
#[pyo3(name="shutdown_ratpair")]
pub fn py_shutdown_tofcpu() -> PyResult<PyTofCommand> {
  match shutdown_tofcpu() {
    None => {
      return Err(PyValueError::new_err(format!("You encounterd a dragon \u{1f409}! We don't know what's going on either.")));
    }
    Some(cmd) => {
      let pycmd = PyTofCommand { 
       command : cmd
      };
      return Ok(pycmd);
    }
  }
}


/// Restart the liftof-rb clients on the given boards
///
/// # Arguments
///   * rbs: restart the client on the given rb ids, 
///          if empty, restart on all of them
#[pyfunction]
#[pyo3(name="restart_liftofrb")]
pub fn py_restart_liftofrb(rbs : Vec<u8>) -> PyResult<PyTofCommand> {
  match restart_liftofrb(&rbs) {
    None => {
      return Err(PyValueError::new_err(format!("You encounterd a dragon \u{1f409}! We don't know what's going on either.")));
    }
    Some(cmd) => {
      let pycmd = PyTofCommand { 
       command : cmd
      };
      return Ok(pycmd);
    }
  }
}

/// Trigger the start of a new data run with 
/// the next active config
#[pyfunction]
#[pyo3(name="start_run")]
pub fn py_start_run() -> PyResult<PyTofCommand> {
  match start_run() {
    None => {
      return Err(PyValueError::new_err(format!("You encounterd a dragon \u{1f409}! We don't know what's going on either.")));
    }
    Some(cmd) => {
      let pycmd = PyTofCommand { 
       command : cmd
      };
      return Ok(pycmd);
    }
  }
}

/// Stop the current active run and idle
#[pyfunction]
#[pyo3(name="stop_run")]
pub fn py_stop_run() -> PyResult<PyTofCommand> {
  match stop_run() {
    None => {
      return Err(PyValueError::new_err(format!("You encounterd a dragon \u{1f409}! We don't know what's going on either.")));
    }
    Some(cmd) => {
      let pycmd = PyTofCommand { 
       command : cmd
      };
      return Ok(pycmd);
    }
  }
}

/// Run a calibration of all RBs
///
/// # Arguments:
///   * pre_run_calibration : Run the RBCalibration routine before 
///                           every run start
///   * send_packetes       : Send the RBCalibration packets
///   * save_events         : Save the events to the RBCalibration
///                           packets
#[pyfunction]
#[pyo3(name="rb_calibration")]
pub fn py_rb_calibration(pre_run_calibration : bool, send_packets : bool, save_events : bool) -> PyResult<PyTofCommand> {
  match rb_calibration(pre_run_calibration,send_packets, save_events) {
    None => {
      return Err(PyValueError::new_err(format!("You encounterd a dragon \u{1f409}! We don't know what's going on either.")));
    }
    Some(cmd) => {
      let pycmd = PyTofCommand { 
       command : cmd
      };
      return Ok(pycmd);
    }
  }
}


/// Change the MTBSettings in the config file with relevant trigger settings
#[pyfunction]
#[pyo3(name="change_triggerconfig")]
pub fn py_change_triggerconfig(cfg : &PyTriggerConfig) -> PyResult<PyTofCommand> {
  match change_triggerconfig(&cfg.config) {
    None => {
      return Err(PyValueError::new_err(format!("You encounterd a dragon \u{1f409}! We don't know what's going on either.")));
    }
    Some(cmd) => {
      let pycmd = PyTofCommand { 
       command : cmd
      };
      return Ok(pycmd);
    }
  }
}


/// Change the TOFEventBuilderSettings in the config
#[pyfunction]
#[pyo3(name="change_tofeventbuilderconfig")]
pub fn py_change_tofeventbuilderconfig(cfg : &PyTOFEventBuilderConfig) -> PyResult<PyTofCommand> {
  match change_tofeventbuilderconfig(&cfg.config) {
    None => {
      return Err(PyValueError::new_err(format!("You encounterd a dragon \u{1f409}! We don't know what's going on either.")));
    }
    Some(cmd) => {
      let pycmd = PyTofCommand { 
       command : cmd
      };
      return Ok(pycmd);
    }
  }
}

