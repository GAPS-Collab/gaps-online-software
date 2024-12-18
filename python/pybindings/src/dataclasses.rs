use numpy::{
    PyArray,
    PyArray1,
    PyArray2, 
    //pyarray_bound,
    //PyArrayMethods,
    //ndarray::Array,
};

use pyo3_polars::{
    PyDataFrame,
    //PySeries
};

use pyo3::Python;

use tof_dataclasses::ProtocolVersion;
use tof_dataclasses::io::TofPacketReader;
use tof_dataclasses::packets::{
  TofPacket,
  PacketType
};
use tof_dataclasses::database::DsiJChPidMapping;

use tof_dataclasses::heartbeats::HeartBeatDataSink;
use tof_dataclasses::heartbeats::MTBHeartbeat;
use tof_dataclasses::heartbeats::EVTBLDRHeartbeat;

use tof_dataclasses::monitoring::{
  MoniData,
  MoniSeries,
  PAMoniData,
  PBMoniData,
  RBMoniData,
  MtbMoniData, 
  CPUMoniData,
  LTBMoniData,
};

use tof_dataclasses::status::TofDetectorStatus;

use tof_dataclasses::series::{
  PAMoniDataSeries,
  PBMoniDataSeries,
  RBMoniDataSeries,
  MtbMoniDataSeries,
  CPUMoniDataSeries,
  LTBMoniDataSeries,
};

use tof_dataclasses::events::{
  TofEvent,
  TofEventHeader,
  TofEventSummary,
  EventStatus,
  TofHit,
  MasterTriggerEvent,
  RBEvent,
  RBEventHeader,
  RBWaveform
};

use tof_dataclasses::serialization::{
  Serialization,
  Packable
};

use tof_dataclasses::commands::{
  TofCommandV2,
  TofCommandCode
};

use tof_dataclasses::calibrations::{
  RBCalibrations,
  //clean_spikes,
  //spike_cleaning
};

use tof_dataclasses::commands::config::{
  AnalysisEngineConfig,
  RunConfig,
  TriggerConfig,
  TOFEventBuilderConfig,
  DataPublisherConfig,
  BuildStrategy
};

use pyo3::prelude::*;
use pyo3::exceptions::{
  PyKeyError,
  PyValueError,
  PyIOError,
};

use tof_dataclasses::events::TriggerType;
use tof_dataclasses::events::master_trigger::LTBThreshold;
use tof_dataclasses::events::rb_event::RBPaddleID;

//trait<T> Wrapper {
//  where T : Packable
//
//  /// Return the name of the underlying struct
//  fn wrapped_name() -> &str;
//
//  /// Unpack from a wrapped TofPacket
//  fn from_tofpacket(&mut self, packet : &PyTofPacket) -> PyResult<()> {
//    let tp = packet.get_tp();
//    match tp.unpack::<T>() {
//      Ok(moni) => {
//        self.moni = moni;
//        return Ok(());
//      }
//      Err(err) => {
//        let err_msg = format!("Unable to unpack TofPacket! {err}");
//        return Err(PyIOError::new_err(err_msg));
//      }
//    }
//  }
//
//  fn __repr__(&self) -> PyResult<String> {
//    Ok(format!("<PyO3Wrapper: {}>", self.moni)) 
//  }
//

#[pyclass]
#[pyo3(name="RBPaddleID")]
pub struct PyRBPaddleID {
  pub pid : RBPaddleID
}

#[pymethods]
impl PyRBPaddleID {

  #[new]
  fn new() -> Self { 
    Self {
      pid : RBPaddleID::new()    
    }
  }
 
  fn to_u64(&self) -> u64 {
    self.pid.to_u64()
  }
  
  fn get_order_flipped(&self, channel : u8) -> bool {
    self.pid.get_order_flipped(channel)
  }
  
  fn get_order_str(&self, channel : u8) -> String {
    self.pid.get_order_str(channel)
  }
  
  fn is_a(&self, channel : u8) -> bool {
    self.pid.is_a(channel)
  }
  
  fn from_u64(&self, val : u64) -> Self {
    let pid = RBPaddleID::from_u64(val);
    Self {
      pid
    }
  }

  //fn from_rb(
  fn get_paddle_id(&self, channel : u8) -> (u8, bool) {  
    self.pid.get_paddle_id(channel)
  }
  
  fn __repr__(&self) -> PyResult<String> {
    Ok(format!("<PyO3Wrapper: {}>", self.pid)) 
  }
}

#[pyclass]
#[pyo3(name="TofDetectorStatus")]
pub struct PyTofDetectorStatus {
  pub status : TofDetectorStatus
}

#[pymethods]
impl PyTofDetectorStatus {
  
  #[new]
  fn new() -> Self {
    let status =  TofDetectorStatus::new();
    Self {
      status
    }
  }
  //fn to_bytestream(&self) -> Vec<u8> {
  //  self.config.to_bytestream()
  //}
  fn from_tofpacket(&mut self, packet : &PyTofPacket) -> PyResult<()> {
    let tp = packet.get_tp();
    match tp.unpack::<TofDetectorStatus>() {
      Ok(status) => {
        self.status = status;
        return Ok(());
      }
      Err(err) => {
        let err_msg = format!("Unable to unpack TofPacket! {err}");
        return Err(PyIOError::new_err(err_msg));
      }
    }
  }

  #[getter]
  fn channels000_031(&self) -> u32 {
    self.status.channels000_031
  }

  #[getter]
  fn channels032_063(&self) -> u32 { 
    self.status.channels032_063
  }

  #[getter]
  fn channels064_095(&self) -> u32 { 
    self.status.channels064_095
  }

  #[getter]
  fn channels096_127(&self) -> u32 { 
    self.status.channels096_127
  }  

  #[getter]
  fn channels128_159(&self) -> u32 { 
    self.status.channels128_159
  }

  #[getter]
  fn channels160_191(&self) -> u32 { 
    self.status.channels160_191
  }

  #[getter]
  fn channels192_223(&self) -> u32 { 
    self.status.channels192_223
  }

  #[getter]
  fn channels224_255(&self) -> u32 { 
    self.status.channels224_255
  }
  
  #[getter]
  fn channels256_297(&self) -> u32 { 
    self.status.channels256_297
  }

  #[getter]
  fn channels298_319(&self) -> u32 { 
    self.status.channels298_319
  }

  fn __repr__(&self) -> PyResult<String> {
    Ok(format!("<PyO3Wrapper: {}>", self.status)) 
  }
}


#[pyclass]
#[pyo3(name="RunConfig")]
pub struct PyRunConfig {
  pub config : RunConfig
}

impl PyRunConfig {
  pub fn set_config(&mut self, config : RunConfig) {
    self.config = config
  }
}

#[pymethods]
impl PyRunConfig {
  #[new]
  fn new() -> Self {
    let config =  RunConfig::new();
    Self {
      config
    }
  }

  #[getter]
  fn get_runid(&self) -> PyResult<u32> {
    Ok(self.config.runid)
  }
  
  #[setter]
  fn set_runid(&mut self, runid: u32) -> PyResult<()> {
    self.config.runid = runid;
    Ok(())
  }
  
  fn to_bytestream(&self) -> Vec<u8> {
    self.config.to_bytestream()
  }
  
  fn __repr__(&self) -> PyResult<String> {
    Ok(format!("<PyO3Wrapper: {}>", self.config)) 
  }
}

#[pyclass]
#[pyo3(name="DataPublisherConfig")]
pub struct PyDataPublisherConfig {
  pub config : DataPublisherConfig
}

#[pymethods]
impl PyDataPublisherConfig {
  #[new]
  fn new() -> Self {
    let config =  DataPublisherConfig::new();
    Self {
      config
    }
  }

  #[getter]
  pub fn get_mbytes_per_file(&self) -> Option<u16> {
    self.config.mbytes_per_file
  }

  #[setter]
  pub fn set_mbytes_per_file(&mut self, mbytes : u16) {
    self.config.mbytes_per_file = Some(mbytes);
  }
  
  #[getter]
  pub fn get_discard_event_fraction(&self) -> Option<f32> {
    self.config.discard_event_fraction
  }

  #[setter]
  pub fn set_discard_event_fraction(&mut self, frac : f32) {
    self.config.discard_event_fraction = Some(frac);
  }
  
  #[getter]
  pub fn get_send_mtb_event_packets(&self) -> Option<bool> {
    self.config.send_mtb_event_packets
  }

  #[setter]
  pub fn set_send_mtb_event_packets(&mut self, send : bool ) {
    self.config.send_mtb_event_packets = Some(send);
  }
  
  #[getter]
  pub fn get_send_rbwaveform_packets(&self) -> Option<bool> {
    self.config.send_rbwaveform_packets
  }

  #[setter]
  pub fn set_send_rbwaveform_packets(&mut self, send : bool ) {
    self.config.send_rbwaveform_packets = Some(send);
  }
  
  #[getter]
  pub fn get_send_rbwf_every_x_event(&self) -> Option<u32> {
    self.config.send_rbwf_every_x_event
  }

  #[setter]
  pub fn set_send_rbwf_every_x_event(&mut self, nevent : u32) {
    self.config.send_rbwf_every_x_event = Some(nevent);
  }
  
  #[getter]
  pub fn get_send_tof_summary_packets(&self) -> Option<bool> {
    self.config.send_tof_summary_packets
  }

  #[setter]
  pub fn set_send_tof_summary_packets(&mut self, send : bool ) {
    self.config.send_tof_summary_packets = Some(send);
  }
  
  #[getter]
  pub fn get_send_tof_event_packets(&self) -> Option<bool> {
    self.config.send_tof_event_packets
  }

  #[setter]
  pub fn set_send_tof_event_packets(&mut self, send : bool ) {
    self.config.send_tof_event_packets = Some(send);
  }
  
  #[getter]
  pub fn get_hb_send_interval(&self) -> Option<u16> {
    self.config.hb_send_interval
  }

  #[setter]
  pub fn set_hb_send_interfal(&mut self, interv : u16) {
    self.config.hb_send_interval = Some(interv)
  }
  
  fn to_bytestream(&self) -> Vec<u8> {
    self.config.to_bytestream()
  }
  
  fn __getitem__(&self, py: Python, key: &str) -> PyResult<Option<PyObject>> {  
    match key {
      "mbytes_per_file"          => Ok(Some(self.config.mbytes_per_file.into_py(py))),
      "discard_event_fraction"   => Ok(Some(self.config.discard_event_fraction.into_py(py))),
      "send_mtb_event_packets"   => Ok(Some(self.config.send_mtb_event_packets.into_py(py))),
      "send_rbwaveform_packets"  => Ok(Some(self.config.send_rbwaveform_packets.into_py(py))),
      "send_rbwf_every_x_event"  => Ok(Some(self.config.send_rbwf_every_x_event.into_py(py))),
      "send_tof_summary_packets" => Ok(Some(self.config.send_tof_summary_packets.into_py(py))),
      "send_tof_event_packets"   => Ok(Some(self.config.send_tof_event_packets.into_py(py))),
      "hb_send_interval"         => Ok(Some(self.config.hb_send_interval.into_py(py))),
      _     => Err(PyKeyError::new_err(format!("Key '{}' not found", key)))
    }
  }

  fn __setitem__(&mut self, key: &str, value: &Bound<'_, PyAny>) -> PyResult<()> {
    match key {
      "mbytes_per_file" => {
          self.config.active_fields |= 1;
          self.config.mbytes_per_file = Some(value.extract::<u16>()?);
          Ok(())
      }
      "discard_event_fraction" => {
          self.config.active_fields |= 2;
          self.config.discard_event_fraction = Some(value.extract::<f32>()?);
          Ok(())
      }
      "send_mtb_event_packets" => {
          self.config.active_fields |= 4;
          self.config.send_mtb_event_packets = Some(value.extract::<bool>()?);
          Ok(())
      }
      "send_rbwaveform_packets" => {
          self.config.active_fields |= 8;
          self.config.send_rbwaveform_packets = Some(value.extract::<bool>()?);
          Ok(())
      }
      "send_rbwf_every_x_event" => {
          self.config.active_fields |= 16;
          self.config.send_rbwf_every_x_event = Some(value.extract::<u32>()?);
          Ok(())
      }
      "send_tof_summary_packets" => {
          self.config.active_fields |= 32;
          self.config.send_tof_summary_packets = Some(value.extract::<bool>()?);
          Ok(())
      }
      "send_tof_event_packets" => {
          self.config.active_fields |= 64;
          self.config.send_tof_event_packets = Some(value.extract::<bool>()?);
          Ok(())
      }
      "hb_send_interval" => {
          self.config.active_fields |= 128;
          self.config.hb_send_interval = Some(value.extract::<u16>()?);
          Ok(())
      }
      _ => Err(PyKeyError::new_err(format!("Key '{}' not found", key))),
    }
  }
  
  fn __repr__(&self) -> PyResult<String> {
    Ok(format!("<PyO3Wrapper: {}>", self.config)) 
  }
}

#[pyclass]
#[pyo3(name="TriggerConfig")]
pub struct PyTriggerConfig {
  pub config : TriggerConfig
}

#[pymethods]
impl PyTriggerConfig {
  #[new]
  fn new() -> Self {
    let cfg =  TriggerConfig::new();
    Self {
      config : cfg
    }
  }

  #[getter] 
  fn get_prescale(&self) -> Option<f32> {
    self.config.prescale
  }
  
  #[setter]
  fn set_prescale(&mut self, prescale: f32) -> PyResult<()> {
    self.config.set_prescale (prescale);
    Ok(())
  }

  #[getter] 
  fn get_gaps_trigger_use_beta(&self) -> Option<bool> {
    self.config.gaps_trigger_use_beta
  }
  
  #[setter]
  fn set_gaps_trigger_use_beta(&mut self, gaps_trigger_use_beta: bool) -> PyResult<()> {
    self.config.set_gaps_trigger_use_beta(gaps_trigger_use_beta);
    Ok(())
  }

  #[getter] 
  fn get_trigger_type(&self) -> Option<TriggerType> {
    self.config.trigger_type
  }

  #[setter]
  fn set_trigger_type(&mut self, trigger_type: TriggerType) -> PyResult<()> {
    self.config.set_trigger_type(trigger_type);
    Ok(())
  }
  
  #[getter]
  fn get_use_combo_trigger(&self) -> Option<bool> {
    self.config.use_combo_trigger 
  }
  #[setter]
  fn set_use_combo_trigger(&mut self, combo : bool) {
    self.config.set_use_combo_trigger(combo);
  }
  #[getter]
  fn get_combo_trigger_type(&self) -> Option<TriggerType> {
    self.config.combo_trigger_type
  }
  #[setter]
  fn set_combo_trigger_type(&mut self, combo_trigger_type : TriggerType) {
    self.config.set_combo_trigger_type(combo_trigger_type);
  }
  #[getter]
  fn get_combo_trigger_prescale(&self) -> Option<f32> {
    self.config.combo_trigger_prescale
  }
  #[setter]
  fn set_combo_trigger_prescale(&mut self, prescale : f32) {
    self.config.set_combo_trigger_prescale(prescale);
  }
  #[getter]
  fn get_trace_suppression(&self) -> Option<bool> {
    self.config.trace_suppression
  }
  #[setter]
  fn set_trace_suppression(&mut self, tsup : bool) {
    self.config.set_trace_suppression(tsup);
  }
  #[getter]
  fn get_mtb_moni_interval(&mut self) -> Option<u16> {
    self.config.mtb_moni_interval
  }
  #[setter]
  fn set_mtb_moni_interval(&mut self, moni_int : u16) {
    self.config.set_mtb_moni_interval(moni_int);
  }
  #[getter]
  fn get_tiu_ignore_busy(&self) -> Option<bool> {
    self.config.tiu_ignore_busy
  }

  #[setter]
  fn set_tiu_ignore_busy(&mut self, ignore_busy : bool) {
    self.config.set_tiu_ignore_busy(ignore_busy);
  }
  #[getter]
  fn get_hb_send_interval(&self) -> Option<u16> {
    self.config.hb_send_interval
  }

  fn to_bytestream(&self) -> Vec<u8> {
    self.config.to_bytestream()
  }

  #[setter]
  fn set_hb_send_interval(&mut self, hb_int :  Option<u16>) {
    self.config.hb_send_interval = hb_int;
  }

  fn __getitem__(&self, py: Python, key: &str) -> PyResult<Option<PyObject>> {  
    match key {
      "gaps_trigger_use_beta"  => Ok(Some(self.config.gaps_trigger_use_beta.into_py(py))),
      "prescale"               => Ok(Some(self.config.prescale.into_py(py))),
      "trigger_type"           => Ok(Some(self.config.trigger_type.into_py(py))),
      "use_combo_trigger"      => Ok(Some(self.config.use_combo_trigger.into_py(py))),
      "combo_trigger_type"     => Ok(Some(self.config.combo_trigger_type.into_py(py))),
      "combo_trigger_prescale" => Ok(Some(self.config.combo_trigger_prescale.into_py(py))),
      "trace_suppression"      => Ok(Some(self.config.trace_suppression.into_py(py))),
      "mtb_moni_interval"      => Ok(Some(self.config.mtb_moni_interval.into_py(py))),
      "tiu_ignore_busy"        => Ok(Some(self.config.tiu_ignore_busy.into_py(py))),
      "hb_send_interval"       => Ok(Some(self.config.hb_send_interval.into_py(py))),
      _     => Err(PyKeyError::new_err(format!("Key '{}' not found", key)))
    }
  }

  fn __setitem__(&mut self, key: &str, value: &Bound<'_, PyAny>) -> PyResult<()> {
    match key {
      "gaps_trigger_use_beta" => {
          self.config.active_fields |= 1;
          self.config.gaps_trigger_use_beta = Some(value.extract::<bool>()?);
          Ok(())
      }
      "prescale" => {
          self.config.active_fields |= 2;
          self.config.prescale = Some(value.extract::<f32>()?);
          Ok(())
      }
      "trigger_type" => {
          self.config.active_fields |= 4;
          self.config.trigger_type = Some(value.extract::<TriggerType>()?);
          Ok(())
      }
      "use_combo_trigger" => {
          self.config.active_fields |= 8;
          self.config.use_combo_trigger = Some(value.extract::<bool>()?);
          Ok(())
      }
      "combo_trigger_type" => {
          self.config.active_fields |= 16;
          self.config.combo_trigger_type = Some(value.extract::<TriggerType>()?);
          Ok(())
      }
      "combo_trigger_prescale" => {
          self.config.active_fields |= 32;
          self.config.combo_trigger_prescale = Some(value.extract::<f32>()?);
          Ok(())
      }
      "trace_suppression" => {
          self.config.active_fields |= 64;
          self.config.trace_suppression = Some(value.extract::<bool>()?);
          Ok(())
      }
      "mtb_moni_interval" => {
          self.config.active_fields |= 128;
          self.config.mtb_moni_interval = Some(value.extract::<u16>()?);
          Ok(())
      }
      "tiu_ignore_busy" => {
          self.config.active_fields |= 256;
          self.config.tiu_ignore_busy = Some(value.extract::<bool>()?);
          Ok(())
      }
      "hb_send_interval" => {
          self.config.active_fields |= 512;
          self.config.hb_send_interval = Some(value.extract::<u16>()?);
          Ok(())
      }
      _ => Err(PyKeyError::new_err(format!("Key '{}' not found", key))),
    }
  }

  fn __repr__(&self) -> PyResult<String> {
    Ok(format!("<PyO3Wrapper: {}>", self.config)) 
  }

}


#[pyclass]
#[pyo3(name="TOFEventBuilderConfig")]

pub struct PyTOFEventBuilderConfig{
  pub config : TOFEventBuilderConfig
}

//impl PyTOFEventBuilderConfig {
//  pub fn set_config(&mut self, cfg : TOFEventBuilderConfig) {
//    self.config = cfg;
//  }
//}

#[pymethods]
impl PyTOFEventBuilderConfig{
  #[new]
  fn new() -> Self {
    let cfg: TOFEventBuilderConfig = TOFEventBuilderConfig::new();
    Self {
      config : cfg
    }
  }
  
  #[getter]
  fn get_greediness(&self) -> Option<u8> {
    self.config.greediness
  }

  #[setter]
  fn set_greediness(&mut self, greediness: u8) -> PyResult<()> {
    self.config.set_greediness(greediness);
    Ok(())
  }
  
  // wait for num. RB
  #[getter]
  fn get_wait_nrb(&self) -> Option<u8> {
    self.config.wait_nrb
  }
  #[setter]
  fn set_wait_nrb(&mut self, wait_nrb: u8) -> PyResult<()> {
    self.config.set_wait_nrb(wait_nrb);
    Ok(())
  }
  // Cache size
  #[getter]
  fn get_cachesize(&self) -> Option<u32> {
    self.config.cachesize
  }
  #[setter]
  fn set_cachesize(&mut self, cachesize: u32) -> PyResult<()> {
    self.config.set_cachesize(cachesize);
    Ok(())
  }
  // Num. MTB events per loop
  #[getter]
  fn get_n_mte_per_loop(&self) -> Option<u32> {
    self.config.n_mte_per_loop
  }
  #[setter]
  fn set_n_mte_per_loop(&mut self, n_mte_per_loop: u32) -> PyResult<()> {
    self.config.set_n_mte_per_loop(n_mte_per_loop);
    Ok(())
  }
  // Num. RB events per loop
  #[getter]
  fn get_n_rbe_per_loop(&self) -> Option<u32> {
    self.config.n_rbe_per_loop
  }
  #[setter]
  fn set_n_rbe_per_loop(&mut self, n_rbe_per_loop: u32) -> PyResult<()> {
    self.config.set_n_rbe_per_loop(n_rbe_per_loop);
    Ok(())
  }  
  // TOF Event timescale window
  #[getter]
  fn get_te_timeout_sec(&self) -> Option<u32> {
    self.config.te_timeout_sec
  }
  #[setter]
  fn set_te_timeout_sec(&mut self, te_timeout_sec: u32) -> PyResult<()> {
    self.config.set_te_timeout_sec(te_timeout_sec);
    Ok(())
  }
  // Sort events
  #[getter]
  fn get_sort_events(&self) -> Option<bool> {
    self.config.sort_events
  }
  #[setter]
  fn set_sort_events(&mut self, sort_events: bool) -> PyResult<()> {
    self.config.set_sort_events(sort_events);
    Ok(())
  }
  // build strategy
  #[getter] 
  fn get_build_strategy(&self) -> Option<BuildStrategy> {
    self.config.build_strategy
  }

  #[setter]
  fn set_build_strategy(&mut self, build_strategy: BuildStrategy) -> PyResult<()> {
    self.config.set_build_strategy(build_strategy);
    Ok(())
  }
  
  #[getter] 
  fn get_hb_send_interval(&self) -> Option<u16> {
    self.config.hb_send_interval
  }

  #[setter]
  fn set_hb_send_interval(&mut self, hb_send_interval: u16) -> PyResult<()> {
    self.config.set_hb_send_interval(hb_send_interval);
    Ok(())
  }

  fn to_bytestream(&self) -> Vec<u8> {
    self.config.to_bytestream()
  }
 
  fn __getitem__(&self, py: Python, key: &str) -> PyResult<Option<PyObject>> {  
    match key {
      "cachesize"        => Ok(Some(self.config.cachesize.into_py(py))),
      "n_mte_per_loop"   => Ok(Some(self.config.n_mte_per_loop.into_py(py))),
      "n_rbe_per_loop"   => Ok(Some(self.config.n_rbe_per_loop.into_py(py))),
      "te_timeout_sec"   => Ok(Some(self.config.te_timeout_sec.into_py(py))),
      "sort_events"      => Ok(Some(self.config.sort_events.into_py(py))),
      "build_strategy"   => Ok(Some(self.config.build_strategy.into_py(py))),
      "wait_nrb"         => Ok(Some(self.config.wait_nrb.into_py(py))),
      "greediness"       => Ok(Some(self.config.greediness.into_py(py))),
      "hb_send_interval" => Ok(Some(self.config.hb_send_interval.into_py(py))),
      _     => Err(PyKeyError::new_err(format!("Key '{}' not found", key)))
    }
  }

  fn __setitem__(&mut self, key: &str, value: &Bound<'_, PyAny>) -> PyResult<()> {
    match key {
      "cachesize" => {
          self.config.active_fields |= 1;
          self.config.cachesize = Some(value.extract::<u32>()?);
          Ok(())
      }
      "n_mte_per_loop" => {
          self.config.active_fields |= 2;
          self.config.n_mte_per_loop = Some(value.extract::<u32>()?);
          Ok(())
      }
      "n_rbe_per_loop" => {
          self.config.active_fields |= 4;
          self.config.n_rbe_per_loop = Some(value.extract::<u32>()?);
          Ok(())
      }
      "te_timeout_sec" => {
          self.config.active_fields |= 8;
          self.config.te_timeout_sec = Some(value.extract::<u32>()?);
          Ok(())
      }
      "sort_events" => {
          self.config.active_fields |= 16;
          self.config.sort_events = Some(value.extract::<bool>()?);
          Ok(())
      }
      "build_strategy" => {
          self.config.active_fields |= 32;
          self.config.build_strategy = Some(value.extract::<BuildStrategy>()?);
          Ok(())
      }
      "wait_nrb" => {
          self.config.active_fields |= 64;
          self.config.wait_nrb = Some(value.extract::<u8>()?);
          Ok(())
      }
      "greediness" => {
          self.config.active_fields |= 128;
          self.config.greediness = Some(value.extract::<u8>()?);
          Ok(())
      }
      "hb_send_interval" => {
          self.config.active_fields |= 256;
          self.config.hb_send_interval = Some(value.extract::<u16>()?);
          Ok(())
      }
      _ => Err(PyKeyError::new_err(format!("Key '{}' not found", key))),
    }
  }

  fn __repr__(&self) -> PyResult<String> {
    Ok(format!("<PyO3Wrapper: {}>", self.config)) 
  }

}

#[pyclass]
#[pyo3(name="AnalysisEngineConfig")]
pub struct PyAnalysisEngineConfig{
  pub config : AnalysisEngineConfig
}

impl PyAnalysisEngineConfig {
  pub fn set_config(&mut self, cfg : AnalysisEngineConfig) {
    self.config = cfg;
  }
}

#[pymethods]
impl PyAnalysisEngineConfig {
  #[new]
  fn new() -> Self {
    let cfg: AnalysisEngineConfig = AnalysisEngineConfig::new();
    Self {
      config : cfg
    }
  }
  // beginning with f32s
  // integration start
  #[getter]
  fn get_integration_start(&self) -> PyResult<f32> {
    Ok(self.config.integration_start)
  }

  #[setter]
  fn set_integration_start(&mut self, integration_start: f32) -> PyResult<()> {
    self.config.integration_start = integration_start;
    Ok(())
  }
  // integration window
  #[getter]
  fn get_integration_window(&self) -> PyResult<f32> {
    Ok(self.config.integration_window)
  }

  #[setter]
  fn set_integration_window(&mut self, integration_window: f32) -> PyResult<()> {
    self.config.integration_window = integration_window;
    Ok(())
  } 
  // pedestal threshold
  #[getter]
  fn get_pedestal_thresh(&self) -> PyResult<f32> {
    Ok(self.config.pedestal_thresh)
  }

  #[setter]
  fn set_pedestal_thresh(&mut self, pedestal_thresh: f32) -> PyResult<()> {
    self.config.pedestal_thresh = pedestal_thresh;
    Ok(())
  }
  //Peakfinder time start
  #[getter]
  fn get_find_pks_t_start(&self) -> PyResult<f32> {
    Ok(self.config.find_pks_t_start)
  }

  #[setter]
  fn set_find_pks_t_start(&mut self, find_pks_t_start: f32) -> PyResult<()> {
    self.config.find_pks_t_start = find_pks_t_start;
    Ok(())
  }
  //Peakfinder time window
  #[getter]
  fn get_find_pks_t_window(&self) -> PyResult<f32> {
    Ok(self.config.find_pks_t_window)
  }

  #[setter]
  fn set_find_pks_t_window(&mut self, find_pks_t_window: f32) -> PyResult<()> {
    self.config.find_pks_t_window = find_pks_t_window;
    Ok(())
  }
  //Peakfinder threshold
  #[getter]
  fn get_find_pks_thresh(&self) -> PyResult<f32> {
    Ok(self.config.find_pks_thresh)
  }

  #[setter]
  fn set_find_pks_thresh(&mut self, find_pks_thresh: f32) -> PyResult<()> {
    self.config.find_pks_thresh = find_pks_thresh;
    Ok(())
  }
  // CFD fraction
  #[getter]
  fn get_cfd_fraction(&self) -> PyResult<f32> {
    Ok(self.config.cfd_fraction)
  }

  #[setter]
  fn set_cfd_fraction(&mut self, cfd_fraction: f32) -> PyResult<()> {
    self.config.cfd_fraction = cfd_fraction;
    Ok(())
  }
  //moving on to the bool
  // use zscore?
  #[getter] 
  fn get_use_zscore(&self) -> PyResult<bool> {
    Ok(self.config.use_zscore)
  }

  #[setter]
  fn set_use_zscore(&mut self, use_zscore: bool) -> PyResult<()> {
    self.config.use_zscore = use_zscore;
    Ok(())
  }
  //finally, usize
  // pedestal start bin
  #[getter] 
  fn get_pedestal_begin_bin(&self) -> PyResult<usize> {
    Ok(self.config.pedestal_begin_bin)
  }

  #[setter]
  fn set_pedestal_begin_bin(&mut self, pedestal_begin_bin: usize) -> PyResult<()> {
    self.config.pedestal_begin_bin = pedestal_begin_bin;
    Ok(())
  }
  // pedestal bin window
  #[getter] 
  fn get_pedestal_win_bins(&self) -> PyResult<usize> {
    Ok(self.config.pedestal_win_bins)
  }

  #[setter]
  fn set_pedestal_win_bins(&mut self, pedestal_win_bins: usize) -> PyResult<()> {
    self.config.pedestal_win_bins = pedestal_win_bins;
    Ok(())
  }
  // min peak size
  #[getter] 
  fn get_min_oeak_size(&self) -> PyResult<usize> {
    Ok(self.config.min_peak_size)
  }

  #[setter]
  fn set_min_peak_size(&mut self, min_peak_size: usize) -> PyResult<()> {
    self.config.min_peak_size = min_peak_size;
    Ok(())
  }
  // max peaks
  #[getter] 
  fn get_max_peaks(&self) -> PyResult<usize> {
    Ok(self.config.max_peaks)
  }

  #[setter]
  fn set_max_peaks(&mut self, max_peaks: usize) -> PyResult<()> {
    self.config.max_peaks = max_peaks;
    Ok(())
  }
  
  fn __repr__(&self) -> PyResult<String> {
    Ok(format!("<PyO3Wrapper: {}>", self.config)) 
  }
}

#[pyclass]
#[pyo3(name="TofCommand")]
pub struct PyTofCommand {
  pub command : TofCommandV2
}

#[pymethods]
impl PyTofCommand {
  #[new]
  fn new() -> Self {
    let cmd =  TofCommandV2::new();
    Self {
      command : cmd
    }
  }

  #[getter]
  fn get_command_code(&mut self) -> TofCommandCode {
    self.command.command_code
  }
  
  #[setter]
  fn set_command_code(&mut self, command_code : TofCommandCode) {
    self.command.command_code = command_code;
  }

  /// Pack myself nicely in a TofPacket and 
  /// serialize myself
  ///
  /// Can be used to interface with BFSW/GSE
  /// systems
  fn wrap_n_pack(&self) -> Vec<u8> {
    self.pack().to_bytestream()
  }

  /// An explicit getter for the 
  /// command code, to interface 
  /// with BFSW/GSE systems
  fn get_cc_u8(&self) -> u8 {
    self.command.command_code as u8
  }

  fn to_bytestream(&self) -> Vec<u8> {
    self.command.to_bytestream()
  }
  
  fn from_tofpacket(&mut self, packet : &PyTofPacket) -> PyResult<()> {
    let tp = packet.get_tp();
    match tp.unpack::<TofCommandV2>() {
      Ok(cmd) => {
        self.command = cmd;
        return Ok(());
      }
      Err(err) => {
        let err_msg = format!("Unable to unpack TofPacket! {err}");
        return Err(PyIOError::new_err(err_msg));
      }
    }
  }

  fn pack(&self) -> PyTofPacket {
    let packet   = self.command.pack();
    let mut pytp = PyTofPacket::new();
    pytp.set_tp(packet);
    pytp
  }

  //fn change_next_runconfig(&mut self, key_values : Vec<String>) {
  //  self.command = TofCommandV2::forge_changerunconfig(key_values);
  //}

  fn __repr__(&self) -> PyResult<String> {
    Ok(format!("<PyO3Wrapper: {}>", self.command)) 
  }
}

#[pyclass]
#[pyo3(name="RBCalibration")]
pub struct PyRBCalibration {
  pub cali : RBCalibrations,
}

impl PyRBCalibration {
  pub fn set_cali(&mut self, cali : RBCalibrations) {
    self.cali = cali;
  }
}

#[pymethods]
impl PyRBCalibration {
  #[new]
  fn new() -> Self {
    let cali = RBCalibrations::new(0);
    Self {
      cali,
    }
  }


  #[getter]
  fn rb_id(&self) -> u8 {
    self.cali.rb_id
  }

  #[getter]
  fn d_v(&self) -> f32 {
    self.cali.d_v
  }
 

  #[getter]
  fn vcal_data(&self) -> Vec<PyRBEvent> {
    let mut events = Vec::<PyRBEvent>::with_capacity(1000);
    for ev in &self.cali.vcal_data {
      let mut pyev = PyRBEvent::new();
      pyev.set_event(ev.clone());
      events.push(pyev);
    }
    events
  }
  
  #[getter]
  fn tcal_data(&self) -> Vec<PyRBEvent> {
    let mut events = Vec::<PyRBEvent>::with_capacity(1000);
    for ev in &self.cali.tcal_data {
      let mut pyev = PyRBEvent::new();
      pyev.set_event(ev.clone());
      events.push(pyev);
    }
    events
  }
  
  #[getter]
  fn noi_data(&self) -> Vec<PyRBEvent> {
    let mut events = Vec::<PyRBEvent>::with_capacity(1000);
    for ev in &self.cali.noi_data {
      let mut pyev = PyRBEvent::new();
      pyev.set_event(ev.clone());
      events.push(pyev);
    }
    events
  }
 
  #[getter]
  fn v_offsets<'_py>(&self, py: Python<'_py>) -> PyResult<Bound<'_py, PyArray2<f32>>> {  
    let mut data = Vec::<Vec<f32>>::with_capacity(9);
    for ch in 0..9 {
      data.push(self.cali.v_offsets[ch].to_vec());
    }
    let pyarray = PyArray::from_vec2_bound(py, &data).unwrap();
    Ok(pyarray)
  }
  
  #[getter]
  fn v_dips<'_py>(&self, py: Python<'_py>) -> PyResult<Bound<'_py, PyArray2<f32>>> {  
    let mut data = Vec::<Vec<f32>>::with_capacity(9);
    for ch in 0..9 {
      data.push(self.cali.v_dips[ch].to_vec());
    }
    let pyarray = PyArray::from_vec2_bound(py, &data).unwrap();
    Ok(pyarray)
  }
  
  #[getter]
  fn v_inc<'_py>(&self, py: Python<'_py>) -> PyResult<Bound<'_py, PyArray2<f32>>> {  
    let mut data = Vec::<Vec<f32>>::with_capacity(9);
    for ch in 0..9 {
      data.push(self.cali.v_inc[ch].to_vec());
    }
    let pyarray = PyArray::from_vec2_bound(py, &data).unwrap();
    Ok(pyarray)
  }
  
  #[getter]
  fn tbin<'_py>(&self, py: Python<'_py>) -> PyResult<Bound<'_py, PyArray2<f32>>> {  
    let mut data = Vec::<Vec<f32>>::with_capacity(9);
    for ch in 0..9 {
      data.push(self.cali.tbin[ch].to_vec());
    }
    let pyarray = PyArray::from_vec2_bound(py, &data).unwrap();
    Ok(pyarray)
  }

  /// Load the calibration from a file with a 
  /// TofPacket of type RBCalibration in it
  ///
  /// # Arguments:
  ///
  /// * filename     : File with a TofPacket of type RBCalibration in it
  /// * discard_data : Throw away event data after loading
  #[pyo3(signature = (filename, discard_data = true))]
  fn from_file(&mut self, filename : String, discard_data : bool) -> PyResult<()> {
    let cali = RBCalibrations::from_file(filename, discard_data);
    match cali {
      Ok(c) => {
        self.cali = c;
      },
      Err(err) => {
        return Err(PyValueError::new_err(err.to_string()));
      }
    }
    Ok(())
  }
}
  
//#[getter]
//fn spike_cleaning_all_channel<'_py>(&self, py: Python<'_py>) -> PyResult<Bound<'_py, PyArray2<f32>>> {  
//  let mut data = Vec::<Vec<f32>>::with_capacity(9);
//  for ch in 0..9 {
//    data.push(self.cali.tbin[ch].to_vec());
//  }
//  let pyarray = PyArray::from_vec2_bound(py, &data).unwrap();
//  Ok(pyarray)
//}
#[pyclass]
#[pyo3(name="PAMoniData")]
pub struct PyPAMoniData {
  moni : PAMoniData
}

#[pymethods]
impl PyPAMoniData {
  #[new]
  fn new() -> Self {
    Self {
      moni : PAMoniData::new()
    }
  }
  
  fn from_tofpacket(&mut self, packet : &PyTofPacket) -> PyResult<()> {
    let tp = packet.get_tp();
    match tp.unpack::<PAMoniData>() {
      Ok(moni) => {
        self.moni = moni;
        return Ok(());
      }
      Err(err) => {
        let err_msg = format!("Unable to unpack TofPacket! {err}");
        return Err(PyIOError::new_err(err_msg));
      }
    }
  }
  
  #[getter]
  fn board_id(&self) ->  u8 {
    self.moni.board_id
  }

  /// The temperature for the 16 preamp channels 
  #[getter]
  fn temps(&self) -> [f32;16] {
    self.moni.temps
  }

  /// Pramp bias voltages (mV) for the 16 channels
  #[getter]
  fn biases(&self) -> [f32;16] {
    self.moni.biases
  }
  
  fn __repr__(&self) -> PyResult<String> {
    Ok(format!("<PyO3Wrapper: {}>", self.moni)) 
  }

  fn keys(&self) -> Vec<&'static str> {
    PAMoniData::keys()
  }

  /// Access the (data) members by name
  fn get(&self, varname : &str) -> PyResult<f32> {
    match self.moni.get(varname) {
      None => {
        let err_msg = format!("LTBMoniData does not have a key with name {}! See RBmoniData.keys() for a list of available keys!", varname);
        return Err(PyKeyError::new_err(err_msg));
      }
      Some(val) => {
        return Ok(val)
      }
    }
  }
}

#[pyclass]
#[pyo3(name="PAMoniSeries")]
pub struct PyPAMoniSeries {
  pamoniseries : PAMoniDataSeries,
}

#[pymethods]
impl PyPAMoniSeries {
  #[new]
  fn new() -> Self {
    let pamoniseries = PAMoniDataSeries::new();
    Self {
      pamoniseries,
    }
  }
  
  fn from_file(&mut self, filename : String) -> PyResult<PyDataFrame> {
    let mut reader = TofPacketReader::new(filename);
    reader.filter = PacketType::PAMoniData;
    for tp in reader {
      if let Ok(moni) =  tp.unpack::<PAMoniData>() {
        self.pamoniseries.add(moni);
      }
    }
    match self.pamoniseries.get_dataframe() {
      Ok(df) => {
        let pydf = PyDataFrame(df);
        return Ok(pydf);
      },
      Err(err) => {
        return Err(PyValueError::new_err(err.to_string()));
      }
    }
  }
}

#[pyclass]
#[pyo3(name="PBMoniData")]
pub struct PyPBMoniData {
  moni : PBMoniData,
}

#[pymethods]
impl PyPBMoniData {
  #[new]
  fn new() -> Self {
    Self {
      moni : PBMoniData::new()
    }
  }
  
  fn from_tofpacket(&mut self, packet : &PyTofPacket) -> PyResult<()> {
    let tp = packet.get_tp();
    match tp.unpack::<PBMoniData>() {
      Ok(moni) => {
        self.moni = moni;
        return Ok(());
      }
      Err(err) => {
        let err_msg = format!("Unable to unpack TofPacket! {err}");
        return Err(PyIOError::new_err(err_msg));
      }
    }
  }
  
  #[getter]
  fn board_id(&self) -> u8 {
    self.moni.board_id
  }

  #[getter]
  fn p3v6_preamp_vcp(&self) -> [f32; 3] {
    self.moni.p3v6_preamp_vcp
  }
  
  #[getter]
  fn n1v6_preamp_vcp(&self) -> [f32; 3] {
    self.moni.n1v6_preamp_vcp
  }
  
  #[getter]
  fn p3v4f_ltb_vcp(&self) -> [f32; 3] {
    self.moni.p3v4f_ltb_vcp
  }
  
  #[getter]
  fn p3v4d_ltb_vcp(&self) -> [f32; 3] {
    self.moni.p3v4d_ltb_vcp
  }
  
  #[getter]
  fn p3v6_ltb_vcp(&self) -> [f32; 3] {
    self.moni.p3v6_ltb_vcp
  }
  
  #[getter]
  fn n1v6_ltb_vcp(&self) -> [f32; 3] {
    self.moni.n1v6_ltb_vcp
  }
  
  #[getter]
  fn pds_temp(&self) -> f32 {  
    self.moni.pds_temp
  }
  #[getter]
  fn pas_temp(&self) -> f32 {
    self.moni.pas_temp
  }
  #[getter]
  fn nas_temp(&self) -> f32 {
    self.moni.nas_temp
  }

  #[getter]
  fn shv_temp(&self) -> f32 {
    self.moni.shv_temp
  }

  fn __repr__(&self) -> PyResult<String> {
    Ok(format!("<PyO3Wrapper: {}>", self.moni)) 
  }

  fn keys(&self) -> Vec<&'static str> {
    PBMoniData::keys()
  }

  /// Access the (data) members by name
  fn get(&self, varname : &str) -> PyResult<f32> {
    match self.moni.get(varname) {
      None => {
        let err_msg = format!("LTBMoniData does not have a key with name {}! See RBmoniData.keys() for a list of available keys!", varname);
        return Err(PyKeyError::new_err(err_msg));
      }
      Some(val) => {
        return Ok(val)
      }
    }
  }
}

#[pyclass]
#[pyo3(name="PBMoniSeries")]
pub struct PyPBMoniSeries {
  pbmoniseries : PBMoniDataSeries,
}

#[pymethods]
impl PyPBMoniSeries {
  #[new]
  fn new() -> Self {
    let pbmoniseries = PBMoniDataSeries::new();
    Self {
      pbmoniseries,
    }
  }
  
  fn from_file(&mut self, filename : String) -> PyResult<PyDataFrame> {
    let mut reader = TofPacketReader::new(filename);
    reader.filter = PacketType::PBMoniData;
    for tp in reader {
      //if tp.packet_type == PacketType::PBMoniData {
      if let Ok(moni) =  tp.unpack::<PBMoniData>() {
        self.pbmoniseries.add(moni);
      }
      //}
    }
    match self.pbmoniseries.get_dataframe() {
      Ok(df) => {
        let pydf = PyDataFrame(df);
        return Ok(pydf);
      },
      Err(err) => {
        return Err(PyValueError::new_err(err.to_string()));
      }
    }
  }
}

#[pyclass]
#[pyo3(name="LTBMoniData")]
pub struct PyLTBMoniData {
  moni : LTBMoniData,
}

#[pymethods]
impl PyLTBMoniData {
  #[new]
  fn new() -> Self {
    let moni = LTBMoniData::new();
    Self {
      moni,
    }
  }
 
  fn from_tofpacket(&mut self, packet : &PyTofPacket) -> PyResult<()> {
    let tp = packet.get_tp();
    match tp.unpack::<LTBMoniData>() {
      Ok(moni) => {
        self.moni = moni;
        return Ok(());
      }
      Err(err) => {
        let err_msg = format!("Unable to unpack TofPacket! {err}");
        return Err(PyIOError::new_err(err_msg));
      }
    }
  }

  fn __repr__(&self) -> PyResult<String> {
    Ok(format!("<PyO3Wrapper: {}>", self.moni)) 
  }

  fn keys(&self) -> Vec<&'static str> {
    LTBMoniData::keys()
  }

  /// Access the (data) members by name
  fn get(&self, varname : &str) -> PyResult<f32> {
    match self.moni.get(varname) {
      None => {
        let err_msg = format!("LTBMoniData does not have a key with name {}! See RBmoniData.keys() for a list of available keys!", varname);
        return Err(PyKeyError::new_err(err_msg));
      }
      Some(val) => {
        return Ok(val)
      }
    }
  }

  #[getter]
  fn board_id      (&self)  -> u8  {
    self.moni.board_id
  }

  #[getter]
  fn trenz_temp    (&self)  -> f32  {
    self.moni.trenz_temp
  }

  #[getter]
  fn ltb_temp      (&self)  -> f32  {
    self.moni.ltb_temp
  }
  #[getter]
  fn thresh0       (&self)  -> f32  {
    self.moni.thresh[0]
  }
  #[getter]
  fn thresh1       (&self)  -> f32  {
    self.moni.thresh[1]
  }
  #[getter]
  fn thresh2       (&self)  -> f32  {
    self.moni.thresh[2]
  }
}

#[pyclass]
#[pyo3(name="RBMoniData")]
pub struct PyRBMoniData {
  moni : RBMoniData,
}

#[pymethods]
impl PyRBMoniData {
  #[new]
  fn new() -> Self {
    let moni = RBMoniData::new();
    Self {
      moni,
    }
  }
 
  fn from_tofpacket(&mut self, packet : &PyTofPacket) -> PyResult<()> {
    let tp = packet.get_tp();
    match tp.unpack::<RBMoniData>() {
      Ok(moni) => {
        self.moni = moni;
        return Ok(());
      }
      Err(err) => {
        let err_msg = format!("Unable to unpack TofPacket! {err}");
        return Err(PyIOError::new_err(err_msg));
      }
    }
  }

  fn __repr__(&self) -> PyResult<String> {
    Ok(format!("<PyO3Wrapper: {}>", self.moni)) 
  }

  fn keys(&self) -> Vec<&'static str> {
    RBMoniData::keys()
  }

  /// Access the (data) members by name
  fn get(&self, varname : &str) -> PyResult<f32> {
    match self.moni.get(varname) {
      None => {
        let err_msg = format!("RBMoniData does not have a key with name {}! See RBmoniData.keys() for a list of available keys!", varname);
        return Err(PyKeyError::new_err(err_msg));
      }
      Some(val) => {
        return Ok(val)
      }
    }
  }
  
  #[getter]
  fn board_id         (&self)  -> u8  {
    self.moni.board_id
  }
  #[getter]
  fn rate             (&self)  -> u16 {
    self.moni.rate
  }
  #[getter]
  fn tmp_drs          (&self)  -> f32 {
    self.moni.tmp_drs
  }
  #[getter]
  fn tmp_clk          (&self)  -> f32 {
    self.moni.tmp_clk
  }
  #[getter]
  fn tmp_adc          (&self)  -> f32 {
    self.moni.tmp_adc
  }
  #[getter]
  fn tmp_zynq         (&self)  -> f32 {
    self.moni.tmp_zynq
  }
  #[getter]
  fn tmp_lis3mdltr    (&self)  -> f32 {
    self.moni.tmp_lis3mdltr
  }
  
  #[getter]
  fn tmp_bm280        (&self)  -> f32 {
    self.moni.tmp_bm280
  }
  #[getter]
  fn pressure         (&self)  -> f32 {
    self.moni.pressure
  }
  #[getter]
  fn humidity         (&self)  -> f32 {
    self.moni.humidity
  }
  #[getter]
  fn mag_x            (&self)  -> f32 {
    self.moni.mag_x
  }
  #[getter]
  fn mag_y            (&self)  -> f32 {
    self.moni.mag_y
  }
  #[getter]
  fn mag_z            (&self)  -> f32 {
    self.moni.mag_z
  }
  #[getter]
  fn drs_dvdd_voltage (&self)  -> f32 { 
    self.moni.drs_dvdd_voltage
  }
  #[getter]
  fn drs_dvdd_current (&self)  -> f32 {
    self.moni.drs_dvdd_current
  }
  #[getter]
  fn drs_dvdd_power   (&self)  -> f32 {
    self.moni.drs_dvdd_power
  }
  #[getter]
  fn p3v3_voltage     (&self)  -> f32 {
    self.moni.p3v3_voltage
  }
  #[getter]
  fn p3v3_current     (&self)  -> f32 {
    self.moni.p3v3_current
  }
  #[getter]
  fn p3v3_power       (&self)  -> f32 {
    self.moni.p3v3_current
  }
  #[getter]
  fn zynq_voltage     (&self)  -> f32 {
    self.moni.zynq_voltage
  }
  #[getter]
  fn zynq_current     (&self)  -> f32 {
    self.moni.zynq_current
  }
  #[getter]
  fn zynq_power       (&self)  -> f32 {
    self.moni.zynq_power
  }
  #[getter]
  fn p3v5_voltage     (&self)  -> f32 { 
    self.moni.p3v5_voltage
  }
  #[getter]
  fn p3v5_current     (&self)  -> f32 {
    self.moni.p3v5_current
  }
  #[getter]
  fn p3v5_power       (&self)  -> f32 {
    self.moni.p3v5_power
  }
  #[getter]
  fn adc_dvdd_voltage (&self)  -> f32 {
    self.moni.adc_dvdd_voltage
  }
  #[getter]
  fn adc_dvdd_current (&self)  -> f32 {
    self.moni.adc_dvdd_current
  }
  #[getter]
  fn adc_dvdd_power   (&self)  -> f32 {
    self.moni.adc_dvdd_power
  }
  #[getter]
  fn adc_avdd_voltage (&self)  -> f32 {
    self.moni.adc_avdd_voltage
  }
  #[getter]
  fn adc_avdd_current (&self)  -> f32 {
    self.moni.adc_avdd_current
  }
  #[getter]
  fn adc_avdd_power   (&self)  -> f32 {
    self.moni.adc_avdd_power
  }
  #[getter]
  fn drs_avdd_voltage (&self)  -> f32 { 
    self.moni.drs_avdd_voltage
  }
  #[getter]
  fn drs_avdd_current (&self)  -> f32 {
    self.moni.drs_avdd_current
  }
  #[getter]
  fn drs_avdd_power   (&self)  -> f32 {
    self.moni.drs_avdd_power
  }
  #[getter]
  fn n1v5_voltage     (&self)  -> f32 {
    self.moni.n1v5_voltage
  }
  #[getter]
  fn n1v5_current     (&self)  -> f32 {
    self.moni.n1v5_current
  }
  #[getter]
  fn n1v5_power       (&self)  -> f32 {
    self.moni.n1v5_power
  }
}

#[pyclass]
#[pyo3(name="RBMoniSeries")]
pub struct PyRBMoniSeries {
  rbmoniseries : RBMoniDataSeries,
}

#[pymethods]
impl PyRBMoniSeries {
  #[new]
  fn new() -> Self {
    let rbmoniseries = RBMoniDataSeries::new();
    Self {
      rbmoniseries,
    }
  }
  
  fn from_file(&mut self, filename : String) -> PyResult<PyDataFrame> {
    let mut reader = TofPacketReader::new(filename);
    reader.filter = PacketType::RBMoniData;
    for tp in reader {
      if let Ok(moni) =  tp.unpack::<RBMoniData>() {
        self.rbmoniseries.add(moni);
      }
    }
    match self.rbmoniseries.get_dataframe() {
      Ok(df) => {
        let pydf = PyDataFrame(df);
        return Ok(pydf);
      },
      Err(err) => {
        return Err(PyValueError::new_err(err.to_string()));
      }
    }
  }
}

#[pyclass]
#[pyo3(name="HeartbeatDataSink")]

pub struct PyHeartBeatDataSink{
  pub config : HeartBeatDataSink
}

impl PyHeartBeatDataSink {
  pub fn set_config(&mut self, cfg : HeartBeatDataSink) {
    self.config = cfg;
  }
}
#[pymethods]
impl PyHeartBeatDataSink{
  #[new]
  fn new() -> Self {
    let cfg: HeartBeatDataSink = HeartBeatDataSink::new();
    Self {
      config : cfg
    }
  }
  //mission elapsed time
  #[getter]
  fn get_met(&self) -> PyResult<u64> {
    Ok(self.config.met)
  }
  // num. packets sent
  #[getter]
  fn get_n_packets_sent(&self) -> PyResult<u64> {
    Ok(self.config.n_packets_sent)
  }
  // num. packets incoming
  #[getter]
  fn get_n_packets_incoming(&self) -> PyResult<u64> {
    Ok(self.config.n_packets_incoming)
  }
  // num. bytes written
  #[getter]
  fn get_n_bytes_written(&self) -> PyResult<u64> {
    Ok(self.config.n_bytes_written)
  }
  // num. missing their event id
  #[getter]
  fn get_n_evid_chunksize(&self) -> PyResult<u64> {
    Ok(self.config.n_evid_chunksize)
  }
  // num. missing event id
  #[getter]
  fn get_evid_missing(&self) -> PyResult<u64> {
    Ok(self.config.evid_missing)
  }
  // probe size for missing evid check
  #[getter]
  fn get_evid_check_len(&self) -> PyResult<u64> {
    Ok(self.config.evid_check_len)
  }
  // num. packets written to disk
  #[getter]
  fn get_n_pack_write_disk(&self) -> PyResult<u64> {
    Ok(self.config.n_pack_write_disk)
  }
  
  fn from_tofpacket(&mut self, packet : &PyTofPacket) -> PyResult<()> {
    let tp = packet.get_tp();
    match tp.unpack::<HeartBeatDataSink>() {
      Ok(hb) => {
        self.config = hb;
        return Ok(());
      }
      Err(err) => {
        let err_msg = format!("Unable to unpack TofPacket! {err}");
        return Err(PyIOError::new_err(err_msg));
      }
    }
  }
  fn __repr__(&self) -> PyResult<String> {
    Ok(format!("<PyO3Wrapper: {}>", self.config)) 
  }

}

#[pyclass]
#[pyo3(name="MTBHeartbeat")]
pub struct PyMTBHeartbeat{
  pub config : MTBHeartbeat
}

impl PyMTBHeartbeat {
  pub fn set_config(&mut self, cfg : MTBHeartbeat) {
    self.config = cfg;
  }
}
#[pymethods]
impl PyMTBHeartbeat {
  #[new]
  fn new () -> Self {
    let cfg: MTBHeartbeat = MTBHeartbeat::new();
    Self {
      config : cfg
    }
  }
  #[getter]
  fn get_total_elapsed(&self) -> PyResult<u64> {
    Ok(self.config.total_elapsed)
  }
  #[getter]
  fn get_n_events(&self) -> PyResult<u64> {
    Ok(self.config.n_events)
  }
  #[getter]
  fn get_evq_num_events_last(&self) -> PyResult<u64> {
    Ok(self.config.evq_num_events_last)
  }
  #[getter]
  fn get_evq_num_events_avg(&self) -> PyResult<u64> {
    Ok(self.config.evq_num_events_avg)
  }
  #[getter]
  fn get_n_ev_unsent(&self) -> PyResult<u64> {
    Ok(self.config.n_ev_unsent)
  }
  #[getter]
  fn get_n_ev_missed(&self) -> PyResult<u64> {
    Ok(self.config.n_ev_missed)
  }
  #[getter]
  fn get_trate(&self) -> PyResult<u64> {
    Ok(self.config.trate)
  }
  #[getter]
  fn get_lost_trate(&self) -> PyResult<u64> {
    Ok(self.config.lost_trate)
  }
  
  fn from_tofpacket(&mut self, packet : &PyTofPacket) -> PyResult<()> {
    let tp = packet.get_tp();
    match tp.unpack::<MTBHeartbeat>() {
      Ok(hb) => {
        self.config = hb;
        return Ok(());
      }
      Err(err) => {
        let err_msg = format!("Unable to unpack TofPacket! {err}");
        return Err(PyIOError::new_err(err_msg));
      }
    }
  }
  fn __repr__(&self) -> PyResult<String> {
    Ok(format!("<PyO3Wrapper: {}>", self.config)) 
  }
}
#[pyclass]
#[pyo3(name="EVTBLDRHeartbeat")]
pub struct PyEVTBLDRHeartbeat {
  pub config : EVTBLDRHeartbeat 
}

impl PyEVTBLDRHeartbeat {
  pub fn set_config(&mut self, cfg : EVTBLDRHeartbeat) {
    self.config = cfg;
  }
}
#[pymethods]
impl PyEVTBLDRHeartbeat {
  #[new]
  fn new () -> Self {
    let hb: EVTBLDRHeartbeat = EVTBLDRHeartbeat::new();
    Self {
      config : hb
    }
  }
  #[getter]
  fn get_met_seconds(&self) -> PyResult<usize> {
    Ok(self.config.met_seconds)
  }
  #[getter]
  fn get_n_mte_received_tot(&self) -> PyResult<usize> {
    Ok(self.config.n_mte_received_tot)
  }
  #[getter]
  fn get_n_rbe_received_tot(&self) -> PyResult<usize> {
    Ok(self.config.n_rbe_received_tot )
  }
  #[getter]
  fn get_n_rbe_per_te(&self) -> PyResult<usize> {
    Ok(self.config.n_rbe_per_te)
  }
  #[getter]
  fn get_n_rbe_discarded_tot(&self) -> PyResult<usize> {
    Ok(self.config.n_rbe_discarded_tot)
  }
  #[getter]
  fn get_n_mte_skipped(&self) -> PyResult<usize> {
    Ok(self.config.n_mte_skipped)
  }
  #[getter]
  fn get_n_timed_out(&self) -> PyResult<usize> {
    Ok(self.config.n_timed_out)
  }
  #[getter]
  fn get_n_sent(&self) -> PyResult<usize> {
    Ok(self.config.n_sent)
  }
  #[getter]
  fn get_delta_mte_rbe(&self) -> PyResult<usize> {
    Ok(self.config.delta_mte_rbe)
  }
  #[getter]
  fn get_event_cache_size(&self) -> PyResult<usize> {
    Ok(self.config.event_cache_size)
  }
  #[getter]
  fn get_rbe_wo_mte(&self) -> PyResult<usize> {
    Ok(self.config.rbe_wo_mte)
  }
  #[getter]
  fn get_drs_bsy_lost_hg_hits(&self) -> PyResult<usize> {
    Ok(self.config.drs_bsy_lost_hg_hits)
  }
  #[getter]
  fn get_mte_receiver_cbc_len(&self) -> PyResult<usize> {
    Ok(self.config.mte_receiver_cbc_len)
  }
  #[getter]
  fn get_rbe_receiver_cbc_len(&self) -> PyResult<usize> {
    Ok(self.config.rbe_receiver_cbc_len)
  }
  #[getter]
  fn get_tp_sender_cbc_len(&self) -> PyResult<usize> {
    Ok(self.config.tp_sender_cbc_len)
  }
  #[getter]
  fn get_data_mangled_ev(&self) -> PyResult<usize> {
    Ok(self.config.data_mangled_ev)
  }
  
  fn from_tofpacket(&mut self, packet : &PyTofPacket) -> PyResult<()> {
    let tp = packet.get_tp();
    match tp.unpack::<EVTBLDRHeartbeat>() {
      Ok(hb) => {
        self.config = hb;
        return Ok(());
      }
      Err(err) => {
        let err_msg = format!("Unable to unpack TofPacket! {err}");
        return Err(PyIOError::new_err(err_msg));
      }
    }
  }
  fn __repr__(&self) -> PyResult<String> {
    Ok(format!("<PyO3Wrapper: {}>", self.config)) 
  }
}

#[pyclass]
#[pyo3(name="MtbMoniData")]
pub struct PyMtbMoniData {
  moni : MtbMoniData,
}

impl PyMtbMoniData {
  pub fn set_moni(&mut self, moni : MtbMoniData) {
    self.moni = moni;
  }
}

#[pymethods]
impl PyMtbMoniData {
  
  #[new]
  fn new() -> Self {
    let moni = MtbMoniData::new();
    Self {
      moni,
    }
  }

  fn from_tofpacket(&mut self, packet : &PyTofPacket) -> PyResult<()> {
    let tp = packet.get_tp();
    match tp.unpack::<MtbMoniData>() {
      Ok(moni) => {
        self.moni = moni;
        return Ok(());
      }
      Err(err) => {
        let err_msg = format!("Unable to unpack TofPacket! {err}");
        return Err(PyIOError::new_err(err_msg));
      }
    }
  }

  #[getter]
  pub fn get_rate(&self) -> u16 {
    self.moni.rate
  }
  
  #[getter]
  pub fn get_lost_rate(&self) -> u16 {
    self.moni.lost_rate
  }

  #[getter]
  /// Length of the received BUSY signal from 
  /// the TIU in nanoseconds
  pub fn get_tiu_busy_len(&self) -> u32 {
    self.moni.tiu_busy_len * 10
  }

  #[getter]
  pub fn get_daq_queue_len(&self) -> u16 {
    self.moni.daq_queue_len
  }

  #[getter]
  pub fn get_tiu_emulation_mode(&self) -> bool {
    self.moni.get_tiu_emulation_mode()
  }
  
  #[getter]
  pub fn get_tiu_use_aux_link(&self) -> bool {
    self.moni.get_tiu_use_aux_link()
  }

  #[getter]
  pub fn get_tiu_bad(&self) -> bool { 
    self.moni.get_tiu_bad()
  }

  #[getter]
  pub fn get_tiu_busy_stuck(&self) -> bool {
    self.moni.get_tiu_busy_stuck()
  }

  #[getter]
  pub fn get_tiu_ignore_busy(&self) -> bool {
    self.moni.get_tiu_ignore_busy()
  }


  #[getter]
  pub fn get_fpga_temp(&self) -> f32 {
    self.moni.get_fpga_temp()
  }


  fn __repr__(&self) -> PyResult<String> {
    Ok(format!("<PyO3Wrapper: {}>", self.moni)) 
  }
}


#[pyclass]
#[pyo3(name="MtbMoniSeries")]
pub struct PyMtbMoniSeries {
  mtbmoniseries : MtbMoniDataSeries,
}

#[pymethods]
impl PyMtbMoniSeries {
  #[new]
  fn new() -> Self {
    let mtbmoniseries = MtbMoniDataSeries::new();
    Self {
      mtbmoniseries,
    }
  }

  /// Add an additional file to the series
  fn add_file(&mut self, filename : String) {
    let mut reader = TofPacketReader::new(filename);
    reader.filter = PacketType::MonitorMtb;
    for tp in reader {
      if let Ok(moni) =  tp.unpack::<MtbMoniData>() {
        self.mtbmoniseries.add(moni);
      }
    }
  }

  fn get_dataframe(&mut self) -> PyResult<PyDataFrame> {
    match self.mtbmoniseries.get_dataframe() {
      Ok(df) => {
        let pydf = PyDataFrame(df);
        return Ok(pydf);
      },
      Err(err) => {
        return Err(PyValueError::new_err(err.to_string()));
      }
    }
  }

  fn from_file(&mut self, filename : String) -> PyResult<PyDataFrame> {
    let mut reader = TofPacketReader::new(filename);
    reader.filter = PacketType::MonitorMtb;
    for tp in reader {
      if let Ok(moni) =  tp.unpack::<MtbMoniData>() {
        self.mtbmoniseries.add(moni);
      }
    }
    match self.mtbmoniseries.get_dataframe() {
      Ok(df) => {
        let pydf = PyDataFrame(df);
        return Ok(pydf);
      },
      Err(err) => {
        return Err(PyValueError::new_err(err.to_string()));
      }
    }
  }
}

#[pyclass]
#[pyo3(name="CPUMoniSeries")]
pub struct PyCPUMoniSeries {
  cpumoniseries : CPUMoniDataSeries,
}

#[pymethods]
impl PyCPUMoniSeries {
  #[new]
  fn new() -> Self {
    let cpumoniseries = CPUMoniDataSeries::new();
    Self {
      cpumoniseries,
    }
  }
  
  fn from_file(&mut self, filename : String) -> PyResult<PyDataFrame> {
    let mut reader = TofPacketReader::new(filename);
    reader.filter = PacketType::CPUMoniData;
    for tp in reader {
      if let Ok(moni) =  tp.unpack::<CPUMoniData>() {
        self.cpumoniseries.add(moni);
      }
    }
    match self.cpumoniseries.get_dataframe() {
      Ok(df) => {
        let pydf = PyDataFrame(df);
        return Ok(pydf);
      },
      Err(err) => {
        return Err(PyValueError::new_err(err.to_string()));
      }
    }
  }
}

#[pyclass]
#[pyo3(name="LTBMoniSeries")]
pub struct PyLTBMoniSeries {
  ltbmoniseries : LTBMoniDataSeries,
}

#[pymethods]
impl PyLTBMoniSeries {
  #[new]
  fn new() -> Self {
    let ltbmoniseries = LTBMoniDataSeries::new();
    Self {
      ltbmoniseries,
    }
  }
  
  fn from_file(&mut self, filename : String) -> PyResult<PyDataFrame> {
    let mut reader = TofPacketReader::new(filename);
    reader.filter = PacketType::LTBMoniData;
    for tp in reader {
      if let Ok(moni) =  tp.unpack::<LTBMoniData>() {
        self.ltbmoniseries.add(moni);
      }
    }
    match self.ltbmoniseries.get_dataframe() {
      Ok(df) => {
        let pydf = PyDataFrame(df);
        return Ok(pydf);
      },
      Err(err) => {
        return Err(PyValueError::new_err(err.to_string()));
      }
    }
  }
}


#[pyclass]
#[pyo3(name="TofPacket")]
#[derive(Clone)]
pub struct PyTofPacket {
  pub packet : TofPacket,
}

impl PyTofPacket {
  pub fn set_tp(&mut self, tp : TofPacket) {
    self.packet = tp;
  }

  pub fn get_tp(&self) -> &TofPacket {
    &self.packet
  }
}

#[pymethods]
impl PyTofPacket {
  #[new]
  pub fn new() -> Self {
    Self {
      packet : TofPacket::new(),
    }
  }
 
  #[getter]
  fn packet_type(&self) -> PacketType {
    self.packet.packet_type
  }

  fn to_bytestream(&self) -> Vec<u8> {
    self.packet.to_bytestream()
  }

  fn from_bytestream(&mut self, stream : Vec<u8>, mut pos : usize) -> PyResult<()>{
    match TofPacket::from_bytestream(&stream, &mut pos) {
      Ok(tp) => {
        self.packet = tp;
        return Ok(());
      }
      Err(err) => {
        let err_msg = format!("Unable to TofPacket from bytestream! {err}");
        return Err(PyIOError::new_err(err_msg));
      }
    }
  }

  fn __repr__(&self) -> PyResult<String> {
    Ok(format!("<PyO3Wrapper: {}>", self.packet)) 
  }
}

#[pyclass]
#[pyo3(name="MasterTriggerEvent")]
pub struct PyMasterTriggerEvent {
  event : MasterTriggerEvent,
}

impl PyMasterTriggerEvent {
  pub fn set_event(&mut self,event : MasterTriggerEvent) {
    self.event = event;
  }
}

#[pymethods]
impl PyMasterTriggerEvent {

  #[new]
  pub fn new() -> Self {
    Self {
      event : MasterTriggerEvent::new(),
    }
  }

  #[getter]
  fn event_id(&self) -> u32 {
    self.event.event_id
  }

  #[getter]
  fn status(&self) -> EventStatus {
    self.event.event_status
  }

  /// Get the RB link IDs according to the mask
  #[getter]
  pub fn rb_link_ids(&self) -> Vec<u8> {
    self.event.get_rb_link_ids()
  }

  /// Get the combination of triggered DSI/J/CH on 
  /// the MTB which formed the trigger. This does 
  /// not include further hits which fall into the 
  /// integration window. For those, se rb_link_mask
  ///
  /// The returned values follow the TOF convention
  /// to start with 1, so that we can use them to 
  /// look up LTB ids in the db.
  ///
  /// # Returns
  ///
  ///   Vec<(hit)> where hit is (DSI, J, (CH, CH), LTBThreshold) 
  #[getter]
  pub fn trigger_hits(&self) -> PyResult<Vec<(u8, u8, (u8, u8), LTBThreshold)>> {
    Ok(self.event.get_trigger_hits())
  }

  #[getter]
  pub fn timestamp_mtb(&self) -> u32 {
    self.event.timestamp
  }


  #[getter]
  pub fn timestamp_gps32(&self) -> u32 {
    self.event.tiu_gps32
  }
  
  #[getter]
  pub fn timestamp_gps16(&self) -> u16 {
    self.event.tiu_gps16
  }

  #[getter]
  pub fn timestamp_tiu(&self) -> u32 { 
    self.event.tiu_timestamp
  }

  /// Get absolute timestamp as sent by the GPS
  #[getter]
  pub fn timestamp_abs48(&self) -> u64 {
    self.event.get_timestamp_abs48()
  }
  
  /// Get the trigger sources from trigger source byte
  /// This returns a list with the fired triggers for 
  /// this event
  #[getter]
  pub fn trigger_sources(&self) -> Vec<TriggerType> {
    self.event.get_trigger_sources() 
  }
  
  fn __repr__(&self) -> PyResult<String> {
    Ok(format!("<PyO3Wrapper: {}>", self.event))
  }

}

#[pyclass]
#[pyo3(name="RBEventHeader")]
pub struct PyRBEventHeader {
  header : RBEventHeader,
}

impl PyRBEventHeader {

  //pub rb_id                : u8   ,    
  //pub event_id             : u32  , 
  //pub status_byte          : u8   ,
  //// FIXME - channel mask still has space
  //// for the status_bytes, since it only
  //// uses 9bits
  //pub channel_mask         : u16  , 
  //pub stop_cell            : u16  , 
  //// we change this by keeping the byte
  //// order the same to accomodate the sine 
  //// values
  //pub ch9_amp              : u16, 
  //pub ch9_freq             : u16, 
  //pub ch9_phase            : u32, 
  ////pub crc32              : u32  , 
  ////pub dtap0              : u16  , 
  ////pub drs4_temp          : u16  , 
  //pub fpga_temp            : u16  , 
  //pub timestamp32          : u32  ,
} 

#[pymethods]
impl PyRBEventHeader {
  #[new]
  pub fn new() -> Self {
    Self {
      header : RBEventHeader::new(),
    }
  }

  #[getter]
  fn rb_id(&self) -> u8 {
    self.header.rb_id
  }
  
  #[getter]
  fn event_id(&self) -> u32 {
    self.header.event_id
  }
  
  //#[getter]
  //fn status_byte(&self) -> u8 {
  //  self.header.status_byte
  //}
  
  #[getter]
  fn channel_mask(&self) -> u16 {
    self.header.get_channel_mask()
  }
  
  #[getter]
  fn stop_cell(&self) -> u16 {
    self.header.stop_cell
  }
  
  #[getter]
  fn fpga_temp(&self) -> f32 {
    self.header.get_fpga_temp()
  }
  
  #[getter]
  fn drs_deadtime(&self) -> u16 {
    self.header.drs_deadtime 
  }

  #[getter]
  fn timestamp32(&self) -> u32 {
    self.header.timestamp32
  }
  
  #[getter]
  fn timestamp16(&self) -> u16 {
    self.header.timestamp16
  }

  //  pub ch9_amp: u16,
  //  pub ch9_freq: u16,
  //  pub ch9_phase: u32,

  fn get_channels(&self) -> Vec<u8> {
    self.header.get_channels()
  }

  #[getter]
  pub fn is_event_fragment(&self) -> bool {
    self.header.is_event_fragment()
  }

  #[getter]
  pub fn drs_lost_trigger(&self) -> bool {
    self.header.drs_lost_trigger()
  }

  #[getter]
  fn lost_lock(&self) -> bool {
    self.header.lost_lock()
  }

  #[getter]
  fn lost_lock_last_sec(&self) -> bool {
    self.header.lost_lock_last_sec()
  }

  #[getter]
  fn is_locked(&self) -> bool {
    self.header.is_locked()
  }

  #[getter]
  fn is_locked_last_sec(&self) -> bool {
    self.header.is_locked_last_sec()
  }


  fn __repr__(&self) -> PyResult<String> {
    Ok(format!("<PyO3Wrapper: {}>", self.header)) 
  }

}

#[pyclass]
#[pyo3(name="TofEventSummary")]
pub struct PyTofEventSummary {
  event : TofEventSummary,
}

impl PyTofEventSummary {
  pub fn set_event(&mut self, event : TofEventSummary) {
    self.event = event;
  }
}

#[pymethods]
impl PyTofEventSummary {
  #[new]
  pub fn new() -> Self {
    Self {
      event : TofEventSummary::new(),
    }
  }

  #[getter]
  fn event_id(&self) -> u32 {
    self.event.event_id
  }
  
  #[getter]
  fn event_status(&self) -> EventStatus {
    self.event.status
  }
  
  /// Compare the hg hits of the event with the triggered paddles and 
  /// return the paddles which have at least a missing HG hit
  fn get_missing_paddles_hg(&self, mapping : DsiJChPidMapping) -> Vec<u8> {
    self.event.get_missing_paddles_hg(&mapping)
  }

  /// Get all the paddle ids which have been triggered
  fn get_triggered_paddles(&self, mapping : DsiJChPidMapping) -> Vec<u8> {
    self.event.get_triggered_paddles(mapping)
  }

  /// The hits we were not able to read out because the DRS4 chip
  /// on the RBs was busy
  #[getter]
  fn lost_hits(&self) -> u16 {
    self.event.drs_dead_lost_hits
  }

  /// RB Link IDS (not RB ids) which fall into the 
  /// trigger window
  #[getter]
  fn rb_link_ids(&self) -> Vec<u8> {
    self.event.get_rb_link_ids()
  }

  /// Hits which formed a trigger
  #[getter]
  pub fn trigger_hits(&self) -> PyResult<Vec<(u8, u8, (u8, u8), LTBThreshold)>> {
    Ok(self.event.get_trigger_hits())
  }
  
  /// The active triggers in this event. This can be more than one, 
  /// if multiple trigger conditions are satisfied.
  #[getter]
  pub fn trigger_sources(&self) -> Vec<TriggerType> {
    self.event.get_trigger_sources()
  } 

  #[getter]
  pub fn hits(&self) -> Vec<PyTofHit> {
    let mut hits = Vec::<PyTofHit>::new();
    for h in &self.event.hits {
      let mut pyhit = PyTofHit::new();
      pyhit.hit = h.clone();
      hits.push(pyhit);
    }
    hits
  }

  //#[getter]
  //fn beta(&self) -> f32 {
  //  self.event.get_beta()
  //}

  #[getter]
  fn timestamp16(&self) -> u16 {
    self.event.timestamp16
  }
  
  #[getter]
  fn timestamp32(&self) -> u32 {
    self.event.timestamp32
  }
  
  #[getter]
  fn timestamp48(&self) -> u64 {
    self.event.get_timestamp48()
  }
  
  #[getter]
  fn status(&self) -> EventStatus {
    self.event.status
  }

  /// Unpack a tofpacket
  fn from_tofpacket(&mut self, packet : &PyTofPacket) -> PyResult<()> {
    let tp = packet.get_tp();
    match tp.unpack::<TofEventSummary>() {
      Ok(event) => {
        self.event = event;
        return Ok(());
      }
      Err(err) => {
        let err_msg = format!("Unable to unpack TofPacket! {err}");
        return Err(PyIOError::new_err(err_msg));
      }
    }
  }

  fn __repr__(&self) -> PyResult<String> {
    Ok(format!("<PyO3Wrapper: {}>", self.event)) 
  }
}

#[pyclass]
#[pyo3(name="TofEventHeader")]
#[derive(Debug, Clone)]
pub struct PyTofEventHeader {
  pub header : TofEventHeader
}

impl PyTofEventHeader {
  fn __repr__(&self) -> PyResult<String> {
    Ok(format!("<PyO3Wrapper: {}>", self.header)) 
  }
}

#[pyclass]
#[pyo3(name="TofEvent")]
#[derive(Debug, Clone)]
pub struct PyTofEvent {
  pub event : TofEvent,
}

impl PyTofEvent {
  pub fn set_event(&mut self, event : TofEvent) {
    self.event = event;
  }
}


#[pymethods]
impl PyTofEvent {
  #[new]
  pub fn new() -> Self {
    Self {
      event : TofEvent::new(),
    }
  }

  fn get_missing_paddles_hg(&self, mapping : DsiJChPidMapping) -> Vec<u8> {
    self.event.get_missing_paddles_hg(&mapping)
  }

  #[getter]
  fn event_id(&self) -> u32 {
    self.event.header.event_id
  }

  //#[getter]
  //fn header(&self) -> Py

  #[getter]
  fn mastertriggerevent(&self) ->  PyMasterTriggerEvent {
    let mut mte = PyMasterTriggerEvent::new();
    mte.set_event(self.event.mt_event.clone());
    mte
  }
  
  /// Get the combination of triggered DSI/J/CH on 
  /// the MTB which formed the trigger. This does 
  /// not include further hits which fall into the 
  /// integration window. For those, se rb_link_mask
  ///
  /// The returned values follow the TOF convention
  /// to start with 1, so that we can use them to 
  /// look up LTB ids in the db.
  ///
  /// # Returns
  ///
  ///   Vec<(hit)> where hit is (DSI, J, (CH, CH), LTBThreshold) 
  #[getter]
  pub fn trigger_hits(&self) -> PyResult<Vec<(u8, u8, (u8, u8), LTBThreshold)>> {
    Ok(self.event.mt_event.get_trigger_hits())
  }
  
  /// RB Link IDS (not RB ids) which fall into the 
  /// trigger window
  #[getter]
  fn rb_link_ids(&self) -> Vec<u8> {
    self.event.mt_event.get_rb_link_ids()
  }

  #[getter]
  fn rbevents(&self) -> Vec<PyRBEvent> {
    // use a bit more than typically exepcted number of rbevents
    let mut rbevents = Vec::<PyRBEvent>::with_capacity(5);
    for k in &self.event.rb_events {
      let mut pyrbevent = PyRBEvent::new();
      pyrbevent.set_event(k.clone());
      rbevents.push(pyrbevent);
    }
    rbevents
  }
  
  #[getter]
  fn hits(&self) -> Vec<PyTofHit> {
    let mut hits = Vec::<PyTofHit>::new();
    for ev in &self.event.rb_events {
      for h in &ev.hits {
        let mut pyhit = PyTofHit::new();
        pyhit.hit = *h;
        hits.push(pyhit);
      }
    }
    hits
  }

  #[getter]
  fn waveforms(&self) -> Vec<PyRBWaveform> {
    let mut wfs = Vec::<PyRBWaveform>::new();
    for wf in &self.event.get_rbwaveforms() {
      let mut pywf = PyRBWaveform::new();
      pywf.wf = wf.clone();
      wfs.push(pywf);
    }
    wfs
  }

  fn get_summary(&self) -> PyTofEventSummary {
    let ts = self.event.get_summary();
    let mut pyts = PyTofEventSummary::new();
    pyts.set_event(ts);
    return pyts;
  }

  fn pack(&self) -> PyTofPacket {
    let packet   = self.event.pack();
    let mut pytp = PyTofPacket::new();
    pytp.set_tp(packet);
    pytp
  }

  fn from_tofpacket(&mut self, packet : &PyTofPacket) -> PyResult<()> {
    let tp = packet.get_tp();
    if tp.packet_type != PacketType::TofEvent {
      let err_msg = format!("This packet is of type {} but needs to be of type 'TofEvent'!", tp.packet_type);
      return Err(PyValueError::new_err(err_msg));
    }
    match tp.unpack::<TofEvent>() {
      Ok(ev) => {
        self.event = ev;
        return Ok(());
      }
      Err(err) => {
        let err_msg = format!("Unable to unpack TofPacket! {err}");
        return Err(PyIOError::new_err(err_msg));
      }
    }
  }

  #[pyo3(signature = (filename, start=0, nevents=0))]
  fn from_file(&self, filename : String, start : usize, nevents : usize) -> Vec<PyTofEvent> {
    let mut reader    = TofPacketReader::new(filename);
    reader.filter     = PacketType::TofEvent;
    reader.skip_ahead = start;
    reader.stop_after = nevents;
    let mut capacity  = 1000;
    if nevents > 0 {
      capacity = nevents;
    }
    let mut events = Vec::<PyTofEvent>::with_capacity(capacity);
    while let Some(tp) = reader.get_next_packet() {
    //for tp in reader.get_next_packet() {
      if let Ok(ev) = tp.unpack::<TofEvent>() {
        let mut pyev = PyTofEvent::new();
        pyev.set_event(ev);
        events.push(pyev);
      } else {
        continue;
      }
    }
    events
  }
  
  fn __repr__(&self) -> PyResult<String> {
    Ok(format!("<PyO3Wrapper: {}>", self.event)) 
  }
}


#[pyclass]
#[pyo3(name="RBEvent")]
pub struct PyRBEvent {
  event : RBEvent,
}

impl PyRBEvent {
  pub fn set_event(&mut self, event : RBEvent) {
    self.event = event;
  }
}

#[pymethods]
impl PyRBEvent {
  #[new]
  pub fn new() -> Self {
    Self {
      event : RBEvent::new(),
    }
  }
 
  #[getter]
  fn status(&self) -> EventStatus {
    self.event.status
  }

  fn get_waveform<'_py>(&self, py: Python<'_py>, channel : usize) -> PyResult<Bound<'_py, PyArray1<u16>>> {  
    let wf  = self.event.get_channel_by_id(channel).unwrap().clone();
    let arr = PyArray1::<u16>::from_vec_bound(py, wf);
    Ok(arr)
  }
  
  //#[getter]
  //fn hits(&self) -> Vec<PyTofHit> {
  //  let mut hits = Vec::<PyTofHit>::new();
  //  for h in &self.event.hits {
  //    let mut pyhit = PyTofHit::new();
  //    pyhit.set_hit(*h);
  //    hits.push(pyhit);
  //  }
  //  hits
  //}
  
  fn from_tofpacket(&mut self, packet : &PyTofPacket) -> PyResult<()> {
    let tp = packet.get_tp();
    match tp.unpack::<RBEvent>() {
      Ok(event) => {
        self.event = event;
        return Ok(());
      }
      Err(err) => {
        let err_msg = format!("Unable to unpack TofPacket! {err}");
        return Err(PyIOError::new_err(err_msg));
      }
    }
  }
  
  #[getter]
  fn header(&self) -> PyRBEventHeader {
    let mut py_header = PyRBEventHeader::new();
    //let mut header = self.event.header;
    py_header.header = self.event.header.clone();
    py_header
  }
  
  #[getter]
  fn waveforms(&self) -> Vec<PyRBWaveform> {
    let mut wfs = Vec::<PyRBWaveform>::new();
    for wf in &self.event.get_rbwaveforms() {
      let mut pywf = PyRBWaveform::new();
      pywf.wf = wf.clone();
      wfs.push(pywf);
    }
    wfs
  }
  

  fn __repr__(&self) -> PyResult<String> {
    Ok(format!("<PyO3Wrapper: {}>", self.event)) 
  }
}

#[pyclass]
#[pyo3(name="TofHit")]
pub struct PyTofHit {
  hit : TofHit,
}

#[pymethods]
impl PyTofHit {
  #[new]
  fn new() -> Self {
    Self {
      hit : TofHit::new(),
    }
  }
 
  /// Set the length and cable length for the paddle
  /// FIXME - take gaps_online.db.Paddle as argument
  fn set_paddle(&mut self, plen : f32, clen : f32) {
    self.hit.paddle_len = plen;
    self.hit.cable_len  = clen;
  }

  /// Reconstructed particle interaction time,
  /// calculated from the waveforms of the two
  /// different paddle ends
  #[getter]
  fn t0(&self) -> f32 {
    self.hit.get_t0()
  }

  #[getter]
  fn version(&self) -> ProtocolVersion {
    self.hit.version
  }

  #[getter]
  fn phase(&self) -> f32 {
    self.hit.phase.to_f32()
  }

  #[getter]
  fn baseline_a(&self) -> f32 {
    self.hit.baseline_a.to_f32()
  }

  #[getter]
  fn baseline_a_rms(&self) -> f32 {
    self.hit.baseline_a_rms.to_f32()
  }
  
  #[getter]
  fn baseline_b(&self) -> f32 {
    self.hit.baseline_b.to_f32()
  }

  #[getter]
  fn baseline_b_rms(&self) -> f32 {
    self.hit.baseline_b_rms.to_f32()
  }

  #[getter]
  fn peak_a(&self) -> f32 {
    self.hit.get_peak_a()
  }
  
  #[getter]
  fn peak_b(&self) -> f32 {
    self.hit.get_peak_b()
  }
  
  #[getter]
  fn charge_a(&self) -> f32 {
    self.hit.get_charge_a()
  }
  
  #[getter]
  fn charge_b(&self) -> f32 {
    self.hit.get_charge_b()
  }

  #[getter]
  fn time_a(&self) -> f32 {
    self.hit.get_time_a()
  }
  
  #[getter]
  fn time_b(&self) -> f32 {
    self.hit.get_time_b()
  }

  /// Reconstructed particle interaction position
  /// along the long axis of the paddle.
  /// For the other dimensions, there is no information
  /// about the position.
  /// Reconstructed with the waveforms of both paddle ends.
  #[getter]
  fn pos(&self) -> f32 {
    self.hit.get_pos()
  }
 
  /// The paddle id (1-160) of the hit paddle
  #[getter]
  fn paddle_id(&self) -> u8 {
    self.hit.paddle_id
  }

  fn __repr__(&self) -> PyResult<String> {
    Ok(format!("<PyO3Wrapper: {}>", self.hit)) 
  }
}

#[pyclass]
#[pyo3(name="RBWaveform")]
pub struct PyRBWaveform {
  pub wf : RBWaveform,
}

//impl PyRBWaveform {
//  pub fn set_wf(&mut self, wf : RBWaveform) {
//    self.wf = wf;
//  }
//}

#[pymethods]
impl PyRBWaveform {
  #[new]
  fn new() -> Self {
    Self {
      wf : RBWaveform::new(),
    }
  }
 
  /// Apply the readoutboard calibration to convert adc/bins
  /// to millivolts and nanoseconds
  fn calibrate(&mut self, cali : &PyRBCalibration) -> PyResult<()> {
    match self.wf.calibrate(&cali.cali) {
      Ok(_) => {
        return Ok(());
      }
      Err(err) => {
        return Err(PyValueError::new_err(err.to_string()));
      }
    }
  }

  /// Paddle ID of this wveform (1-160)
  #[getter]
  fn paddle_id(&self) -> u8 {
    self.wf.paddle_id
  }

  #[getter]
  fn rb_id(&self) -> u8 {
    self.wf.rb_id
  }
  
  #[getter]
  fn event_id(&self) -> u32 {
    self.wf.event_id
  }
  
  #[getter]
  fn rb_channel_a(&self) -> u8 {
    self.wf.rb_channel_a
  }
  
  #[getter]
  fn rb_channel_b(&self) -> u8 {
    self.wf.rb_channel_b
  }
  
  #[getter]
  fn stop_cell(&self) -> u16 {
    self.wf.stop_cell
  }
  
  #[getter]
  fn adc_a<'_py>(&self, py: Python<'_py>) ->  PyResult<Bound<'_py, PyArray1<u16>>> {
    let wf  = self.wf.adc_a.clone();
    let arr = PyArray1::<u16>::from_vec_bound(py, wf);
    Ok(arr)
  }
  
  #[getter]
  fn adc_b<'_py>(&self, py: Python<'_py>) ->  PyResult<Bound<'_py, PyArray1<u16>>> {
    let wf  = self.wf.adc_b.clone();
    let arr = PyArray1::<u16>::from_vec_bound(py, wf);
    Ok(arr)
  }
  
  #[getter]
  fn voltages_a<'_py>(&self, py: Python<'_py>) ->  PyResult<Bound<'_py, PyArray1<f32>>> {
    let wf  = self.wf.voltages_a.clone();
    let arr = PyArray1::<f32>::from_vec_bound(py, wf);
    Ok(arr)
  }

  #[getter]
  fn times_a<'_py>(&self, py: Python<'_py>) ->  PyResult<Bound<'_py, PyArray1<f32>>> {
    let times  = self.wf.nanoseconds_a.clone();
    let arr    = PyArray1::<f32>::from_vec_bound(py, times);
    Ok(arr)
  }

  #[getter]
  fn voltages_b<'_py>(&self, py: Python<'_py>) ->  PyResult<Bound<'_py, PyArray1<f32>>> {
    let wf  = self.wf.voltages_b.clone();
    let arr = PyArray1::<f32>::from_vec_bound(py, wf);
    Ok(arr)
  }

  #[getter]
  fn times_b<'_py>(&self, py: Python<'_py>) ->  PyResult<Bound<'_py, PyArray1<f32>>> {
    let times  = self.wf.nanoseconds_b.clone();
    let arr    = PyArray1::<f32>::from_vec_bound(py, times);
    Ok(arr)
  }

  fn apply_spike_filter(&mut self) {
    self.wf.apply_spike_filter();
  }
  
  fn __repr__(&self) -> PyResult<String> {
    Ok(format!("<PyO3Wrapper: {}>", self.wf)) 
  }
} 



