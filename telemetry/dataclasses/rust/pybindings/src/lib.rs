use pyo3::prelude::*;
//use pyo3::exceptions::PyValueError;

extern crate pyo3_log;
//use numpy::PyArray1;

use telemetry_dataclasses::packets as tel_api;
use telemetry_dataclasses::io as tel_io_api;

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
    tof.set_tes(self.event.tof.clone());
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

#[pyclass]
#[pyo3(name="TofHit")]
struct PyTofHit {
  th : tof_api::TofHit,
}


impl PyTofHit {
  fn set_hit(&mut self, th : tof_api::TofHit) {
    self.th = th;
  }
}

#[pymethods]
impl PyTofHit {
  #[new]
  fn new() -> Self {
    Self {
      th : tof_api::TofHit::new(),
    }
  }

  fn get_time_a(&self) -> f32 {
    self.th.get_time_a()
  }
  
  fn get_time_b(&self) -> f32 {
    self.th.get_time_b()
  }
  
  fn get_peak_a(&self) -> f32 {
    self.th.get_peak_a()
  }
  
  fn get_peak_b(&self) -> f32 {
    self.th.get_peak_b()
  }
  
  fn get_charge_a(&self) -> f32 {
    self.th.get_charge_a()
  }
  
  fn get_charge_b(&self) -> f32 {
    self.th.get_charge_b()
  }

  fn get_edep(&self) -> f32 {
    self.th.get_edep()
  }

  fn get_pos_across(&self) -> f32 {
    self.th.get_pos_across()
  }

  fn get_t0(&self) -> f32 {
    self.th.get_t0()
  }

  #[getter]
  fn paddle_id(&self) -> u8 {
    self.th.paddle_id
  }
  
  fn __repr__(&self) -> PyResult<String> {
    Ok(format!("<PyO3Wrapper: {}>", self.th))
  }
}

#[pyclass]
#[pyo3(name="TofEventSummary")]
struct PyTofEventSummary {
  tes : tof_api::TofEventSummary,
}

impl PyTofEventSummary {
  pub fn set_tes(&mut self, tes : tof_api::TofEventSummary) {
    self.tes = tes;
  }
}
   // pub status: u8,
   // pub quality: u8,
   // pub trigger_setting: u8,
   // pub n_trigger_paddles: u8,
   // pub event_id: u32,
   // pub timestamp32: u32,
   // pub timestamp16: u16,
   // pub primary_beta: u16,
   // pub primary_charge: u16,
   // pub hits: Vec<TofHit>,

#[pymethods]
impl PyTofEventSummary {
  #[new]
  fn new() -> Self {
    Self {
      tes : tof_api::TofEventSummary::new()
    }
  }

  #[getter]
  fn event_id(&self) -> u32 {
    self.tes.event_id
  }

  #[getter]
  fn n_trigger_paddles(&self) -> u8 {
    self.tes.n_trigger_paddles
  }

  fn get_timestamp48(&self) -> u64 {
    self.tes.get_timestamp48()
  }

  fn get_hits(&self) -> Vec<PyTofHit> {
    let mut hits = Vec::<PyTofHit>::new();
    for h in &self.tes.hits {
      let mut py_hit = PyTofHit::new();
      py_hit.set_hit(*h);
      hits.push(py_hit);
    }
    hits
  }

  fn __repr__(&self) -> PyResult<String> {
    Ok(format!("<PyO3Wrapper: {}>", self.tes))
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

