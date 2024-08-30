use pyo3::prelude::*;
use pyo3::exceptions::PyValueError;

extern crate pyo3_log;

use tof_dataclasses as api;

#[pyclass]
#[pyo3(name = "IPBus")]
pub struct PyIPBus {
  ipbus : api::ipbus::IPBus,
}

#[pymethods]
impl PyIPBus {
  #[new]
  fn new(target_address : String) -> Self {
    let ipbus = api::ipbus::IPBus::new(target_address).expect("Unable to connect to {target_address}");
    Self {
      ipbus : ipbus,
    }
  }

  /// Make a IPBus status query
  pub fn get_status(&mut self) -> PyResult<()> {
    match self.ipbus.get_status() {
      Ok(_) => {
        return Ok(());
      },
      Err(err)   => {
        return Err(PyValueError::new_err(err.to_string()));
      }
    }
  }
 
  pub fn get_buffer(&self) -> [u8;api::ipbus::MT_MAX_PACKSIZE] {
    return self.ipbus.buffer.clone();
  }

  pub fn set_packet_id(&mut self, pid : u16) {
    self.ipbus.pid = pid;
  }
 
  pub fn get_packet_id(&self) -> u16 {
    self.ipbus.pid
  }

  pub fn get_expected_packet_id(&self) -> u16 {
    self.ipbus.expected_pid
  }

  /// Set the packet id to that what is expected from the targetr
  pub fn realign_packet_id(&mut self) -> PyResult<()> {
    match self.ipbus.realign_packet_id() {
      Ok(_) => {
        return Ok(());
      },
      Err(err)   => {
        return Err(PyValueError::new_err(err.to_string()));
      }
    }
  }
  
  /// Get the next packet id, which is expected by the target
  pub fn get_target_next_expected_packet_id(&mut self) 
    -> PyResult<u16> {
    match self.ipbus.get_target_next_expected_packet_id() {
      Ok(result) => {
        return Ok(result);
      },
      Err(err)   => {
        return Err(PyValueError::new_err(err.to_string()));
      }
    }
  }

  pub fn read_multiple(&mut self,
                       addr           : u32,
                       nwords         : usize,
                       increment_addr : bool) 
    -> PyResult<Vec<u32>> {
  
    match self.ipbus.read_multiple(addr,
                                   nwords,
                                   increment_addr) {
      Ok(result) => {
        return Ok(result);
      },
      Err(err)   => {
        return Err(PyValueError::new_err(err.to_string()));
      }
    }
  }

  pub fn write(&mut self,
               addr   : u32,
               data   : u32) 
    -> PyResult<()> {
    
    match self.ipbus.write(addr, data) {
      Ok(_) => Ok(()),
      Err(err)   => {
        return Err(PyValueError::new_err(err.to_string()));
      }
    }
  }
 

  pub fn read(&mut self, addr   : u32) 
    -> PyResult<u32> {
    match self.ipbus.read(addr) {
      Ok(result) => {
        return Ok(result);
      },
      Err(err)   => {
        return Err(PyValueError::new_err(err.to_string()));
      }
    }
  }
}

