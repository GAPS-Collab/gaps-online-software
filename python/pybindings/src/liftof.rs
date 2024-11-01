use pyo3::prelude::*;

//use std::collections::HashMap;
use pyo3::exceptions::PyValueError;

pub use crate::dataclasses::{
  PyMasterTriggerEvent,
  PyRBEvent,
  PyTofEvent
};

pub use crate::liftof_dataclasses::PyIPBus;

pub use crate::master_trigger::{
  PyMasterTrigger,
  wrap_prescale_to_u32,
};

use tof_dataclasses::analysis::{
    calc_edep_simple
};

use tof_dataclasses::database::{
    RAT,
    DSICard,
    Paddle,
    MTBChannel,
    LocalTriggerBoard,
    ReadoutBoard,
    Panel,
    connect_to_db,
};

use liftof_lib::{
  waveform_analysis,
  fit_sine_sydney,
};

use liftof_lib::settings::{
  //AnalysisEngineSettings,
  LiftofSettings
};

#[pyfunction]
#[pyo3(name="test_db")]
pub fn test_db() {
  let mut conn = connect_to_db(String::from("/srv/gaps/gaps-online-software/gaps-db/gaps_db/gaps_flight.db")).unwrap();
  let rats = RAT::all(&mut conn).unwrap();
  for r in rats {
    println!("{}", r);
  }
  let dsis = DSICard::all(&mut conn).unwrap();
  for dsi in dsis {
    println!("{}", dsi);
  }
  let paddles = Paddle::all(&mut conn).unwrap();
  for pdl in paddles {
    println!("{}", pdl);
  }
  let mtbch = MTBChannel::all(&mut conn).unwrap();
  for chnl in mtbch {
    println!("{}", chnl);
  }
  let ltbs = LocalTriggerBoard::all(&mut conn).unwrap();
  for ltb in ltbs {
    println!("{}", ltb);
  }
  let rbs = ReadoutBoard::all(&mut conn).unwrap();
  for rb in rbs {
    println!("{}", rb);
  }
  let panels = Panel::all(&mut conn).unwrap();
  for pnl in panels {
    println!("{}", pnl);
  }
}

/// A wrapper for Liftof settings. Can be useful to pass 
/// settings to functions
#[pyclass]
#[pyo3(name="LiftofSettings")]
pub struct PyLiftofSettings {
  pub settings : LiftofSettings
}

impl PyLiftofSettings {
  pub fn set_settings(&mut self, settings : &LiftofSettings) {
    self.settings = settings.clone()
  }
}

#[pymethods]
impl PyLiftofSettings {
  #[new]
  fn new() -> Self {
    let settings = LiftofSettings::new();
    Self { 
      settings : settings
    }
  }

  /// Read settings from a .toml file
  ///
  /// # Arugments:
  ///
  /// * filename : A .toml file with settings fro the 
  ///              liftof flight suite
  #[staticmethod]
  fn from_file(filename : String) -> PyResult<Self> {
    let mut pysettings = PyLiftofSettings::new();
    match LiftofSettings::from_toml(filename) {
      Ok(settings) => {
        pysettings.settings = settings;
      }
      Err(err) => {
        return Err(PyValueError::new_err(err.to_string()));
      }
    }
    Ok(pysettings)
  }
 
  fn __repr__(&self) -> PyResult<String> {
    Ok(format!("<PyO3Wrapper: {}>", self.settings))
  } 

}

#[pyfunction]
#[pyo3(name="calc_edep_simple")]
pub fn wrap_calc_edep_simple(peak_voltage : f32) -> f32 {
  calc_edep_simple(peak_voltage)
}

#[pyfunction]
#[pyo3(name="fit_sine_sydney")]
pub fn wrap_fit_sine_sydney(volts: Vec<f32>, times: Vec<f32>) -> (f32,f32,f32) {
  fit_sine_sydney(&volts, &times)
}

#[pyfunction]
#[pyo3(name="waveform_analysis")]
pub fn py_waveform_analysis(event : &PyTofEvent,
                            settings : &PyLiftofSettings) -> PyResult<PyTofEvent> {
//match waveform_analysis(
//  &mut rbev,
//  &rb_map[&rb_id],
//  settings.clone()
//) {
  let ana_settings = settings.settings.analysis_engine_settings;
  let pth          = settings.settings.db_path.clone();
  let mut conn     = connect_to_db(pth).expect("Check the DB path in the liftof settings!");
  let rbs          = ReadoutBoard::all(&mut conn).expect("Check DB");
  //let mut rb       = ReadoutBoard::new();
  let mut ev_new   = event.clone();
  let mut new_rb_evs = event.event.rb_events.clone();
  for rb_ev in new_rb_evs.iter_mut() {
    for rb_ in &rbs {
      if rb_.rb_id == rb_ev.header.rb_id {
        match waveform_analysis(rb_ev, rb_, ana_settings) {
          Err(err) => {
            println!("Unable to perform waveform_analysis! {err}");
          }
          Ok(_) => ()
        }
      }
    }
    //if rb_ev.rb_id = 
  }
  ev_new.event.rb_events = new_rb_evs;
  //match waveform_analysis(
  Ok(ev_new)
}



