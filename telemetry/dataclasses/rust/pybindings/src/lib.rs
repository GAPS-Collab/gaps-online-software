use pyo3::prelude::*;
//use pyo3::exceptions::PyValueError;

extern crate pyo3_log;
//extern crate rpy-tof-dataclasses;

use telemetry_dataclasses::packets as tel_api;
use telemetry_dataclasses::io as tel_io_api;
extern crate rpy_tof_dataclasses;
use rpy_tof_dataclasses::dataclasses::{
    PyTofHit,
    PyTofEventSummary,
};

// FIXME - this needs to go to liftof-python
// or maybe we want to revive the dataclasses pybindings
// in tof-dataclasses?
use tof_dataclasses::events as tof_api;

#[pyfunction]
fn get_gapsevents(fname : String) -> Vec<PyGapsEvent> {
  let mut pyevents = Vec::<PyGapsEvent>::new();
  let events = tel_io_api::get_gaps_events(fname);
  for ev in events {
    let mut pyev = PyGapsEvent::new();
    pyev.set_tof(ev.tof.clone());
    pyev.set_tracker(ev.tracker.clone());
    pyevents.push(pyev);
  }
  pyevents
}


#[pyclass]
#[pyo3(name="GapsTelemetryEvent")]
struct PyGapsEvent {
  event   : tel_api::GapsEvent,
}

impl PyGapsEvent {
  pub fn set_tof(&mut self, tes : tof_api::TofEventSummary) {
    self.event.tof = tes;
  }
  
  pub fn set_tracker(&mut self, trk : Vec<tel_api::TrackerEvent>) {
    self.event.tracker = trk;
  }
}

#[pymethods]
impl PyGapsEvent {
  #[new]
  fn new() -> Self {
    Self {
      event     : tel_api::GapsEvent::new(),
    }
  }

  #[getter]
  fn tof(&self) -> PyTofEventSummary {
    let mut tof =  PyTofEventSummary::new();
    tof.set_event(self.event.tof.clone());
    tof
  }

  #[getter]
  fn tracker(&self) -> Vec<PyTrackerEvent> {
    let mut trk_ev = Vec::<PyTrackerEvent>::new();
    for ev in &self.event.tracker {
      let mut py_ev = PyTrackerEvent::new();
      py_ev.set_event(ev.clone());
      trk_ev.push(py_ev)
    }
    trk_ev
  }
  
  fn __repr__(&self) -> PyResult<String> {
    Ok(format!("<PyO3Wrapper: {}>", self.event))
  }
}

#[pyclass]
#[pyo3(name="TrackerHit")]
struct PyTrackerHit {
  th : tel_api::TrackerHit,
}

impl PyTrackerHit {
  pub fn set_hit(&mut self, th : tel_api::TrackerHit) {
    self.th = th;
  }
}

#[pymethods]
impl PyTrackerHit {

  #[new]
  fn new() -> Self {
    Self {
      th : tel_api::TrackerHit::new(),
    }
  }

  #[getter]
  fn row(&self) -> u8 {
    self.th.row
  }

  #[getter]
  fn module(&self) -> u8 {
    self.th.module
  }

  #[getter]
  fn channel(&self) -> u8 {
    self.th.channel
  }

  #[getter]
  fn adc(&self) -> u16 {
    self.th.adc
  }

  #[getter]
  fn asic_event_code(&self) -> u8 {
    self.th.asic_event_code
  }
  
  fn __repr__(&self) -> PyResult<String> {
    Ok(format!("<PyO3Wrapper: {}>", self.th))
  }
}

#[pyclass]
#[pyo3(name="Trackerevent")]
struct PyTrackerEvent {
  te : tel_api::TrackerEvent
}

impl PyTrackerEvent {
  fn set_event(&mut self, te : tel_api::TrackerEvent) {
    self.te = te;
  }
}

#[pymethods]
impl PyTrackerEvent {
  #[new]
  fn new() -> Self {
    Self {
      te : tel_api::TrackerEvent::new(),
    }
  }

  #[getter]
  fn layer(&self) -> u8 {
    self.te.layer
  }
  
  #[getter]
  fn flags1(&self) -> u8 {
    self.te.flags1
  }
  
  #[getter]
  fn event_id(&self) -> u32 {
    self.te.event_id
  }
  
  #[getter]
  fn event_time(&self) -> u64 {
    self.te.event_time
  }

  fn get_hits(&self) -> Vec<PyTrackerHit> {
    let mut hits = Vec::<PyTrackerHit>::new();
    for h in &self.te.hits {
      let mut py_hit = PyTrackerHit::new();
      py_hit.set_hit(*h);
      hits.push(py_hit);
    }
    hits
  }

  fn __repr__(&self) -> PyResult<String> {
    Ok(format!("<PyO3Wrapper: {}>", self.te))
  }
}


/// Python API to rust version of tof-dataclasses.
///
/// Currently, this contains only the analysis 
/// functions
#[pymodule]
#[pyo3(name = "rust_telemetry")]
fn rust_dataclasses(_py: Python<'_>, m: &PyModule) -> PyResult<()> {
    pyo3_log::init();
    m.add_function(wrap_pyfunction!(get_gapsevents,m)?)?;
    m.add_class::<PyGapsEvent>()?;
    m.add_class::<PyTofHit>()?;
    m.add_class::<PyTofEventSummary>()?;
    m.add_class::<PyTrackerHit>()?;
    m.add_class::<PyTrackerEvent>()?;
    Ok(())
}

