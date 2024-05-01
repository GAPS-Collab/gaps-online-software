use pyo3::prelude::*;
use pyo3::exceptions::PyValueError;

extern crate pyo3_log;
use numpy::PyArray1;

use tof_dataclasses as api;

#[pyclass]
pub struct PyRBCalibration {
  calibration : api::calibrations::RBCalibrations,
}

impl PyRBCalibration {
  pub fn set_calibration(&mut self, calibration : api::calibrations::RBCalibrations) {
    self.calibration = calibration;
  }
}

#[pymethods]
impl PyRBCalibration {
  #[new]
  pub fn new(rb_id : u8) -> Self {
    Self {
      calibration : api::calibrations::RBCalibrations::new(rb_id),
    }
  }
  
  pub fn voltages(&self, py : Python,  ev : &PyRBEvent,  channel : usize) -> Py<PyArray1<f32>> {
    let mut cali_wf = Vec::<f32>::new();
    let rbev = ev.get_event();
    let adc = rbev.get_channel_by_id(channel).unwrap();
    for k in adc {
      cali_wf.push(*k as f32);
    }
    self.calibration.voltages(channel, rbev.header.stop_cell as usize, &adc, &mut cali_wf);
    let arr = PyArray1::<f32>::from_vec( py,  cali_wf.clone() );
    arr.to_owned() 
  }
}

#[pyclass]
pub struct PyRBEvent {
  event : api::events::RBEvent,
}

impl PyRBEvent {
  pub fn set_event(&mut self, event : api::events::RBEvent) {
    self.event = event;
  }

  pub fn get_event(&self) -> api::events::RBEvent {
    self.event.clone()
  }

}

#[pymethods]
impl PyRBEvent {
  #[new]
  pub fn new() -> Self {
    Self {
      event : api::events::RBEvent::new(),
    }
  }
  
  pub fn get_waveform(&self, py : Python,  channel : usize) -> Py<PyArray1<u16>> {
    let wf  = self.event.get_channel_by_id(channel).unwrap();
    let arr = PyArray1::<u16>::from_vec( py,  wf.clone() );
    arr.to_owned() 
  }
}

#[pyclass]
pub struct PyMasterTriggerEvent {
  event : api::events::MasterTriggerEvent,
}

impl PyMasterTriggerEvent {
  pub fn set_event(&mut self,event : api::events::MasterTriggerEvent) {
    self.event = event;
  }
}

#[pymethods]
impl PyMasterTriggerEvent {

  #[new]
  pub fn new() -> Self {
    Self {
      event : api::events::MasterTriggerEvent::new(),
    }
  }

  /// Get the RB link IDs according to the mask
  pub fn get_rb_link_ids(&self) -> Vec<u8> {
    self.event.get_rb_link_ids()
  }

  ///// Get the combination of triggered DSI/J/CH on 
  ///// the MTB which formed the trigger. This does 
  ///// not include further hits which fall into the 
  ///// integration window. For those, se rb_link_mask
  /////
  ///// The returned values follow the TOF convention
  ///// to start with 1, so that we can use them to 
  ///// look up LTB ids in the db.
  /////
  ///// # Returns
  /////
  /////   Vec<(hit)> where hit is (DSI, J, CH) 
  //pub fn get_trigger_hits(&self) -> Vec<(u8, u8, u8)> {
  //  self.event.get_trigger_hits()
  //}

  /// combine the tiu gps 16 and 32bit timestamps 
  /// into a 48bit timestamp
  pub fn get_timestamp_gps48(&self) -> u64 {
    self.event.get_timestamp_gps48()
  }

  /// Get absolute timestamp as sent by the GPS
  pub fn get_timestamp_abs48(&self) -> u64 {
    self.event.get_timestamp_abs48()
  }
  
  fn __repr__(&self) -> PyResult<String> {
    Ok(format!("<PyO3Wrapper: {}>", self.event))
  }

  ///// Get the trigger sources from trigger source byte
  ///// FIXME! (Does not return anything)
  //pub fn get_trigger_sources(&self) -> Vec<TriggerType> {
  //
  //}
}

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

