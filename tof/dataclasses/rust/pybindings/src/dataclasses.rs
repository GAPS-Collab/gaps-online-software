use numpy::{
    PyArray,
    PyArray1,
    PyArray2, 
    //pyarray_bound,
    //PyArrayMethods,
    //ndarray::Array,
};

extern crate pyo3_polars;
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
use tof_dataclasses::calibrations::RBCalibrations;
use tof_dataclasses::config::{AnalysisEngineConfig, RunConfig};


use pyo3::prelude::*;
use pyo3::exceptions::{
    PyKeyError,
    PyValueError,
    PyIOError,
};

use tof_dataclasses::config::TriggerConfig;
use tof_dataclasses::events::TriggerType;
use tof_dataclasses::events::master_trigger::LTBThreshold;
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
#[pyo3(name="TriggerConfig")]
pub struct PyTriggerConfig {
  pub config : TriggerConfig
}

impl PyTriggerConfig {
  pub fn set_config(&mut self, cfg : TriggerConfig) {
    self.config = cfg;
  }
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
  //prescale
  #[getter] 
  fn get_prescale(&self) -> PyResult<f32> {
    Ok(self.config.prescale)
  }
  
  #[setter]
  fn set_prescale(&mut self, prescale: f32) -> PyResult<()> {
    self.config.prescale = prescale;
    Ok(())
  }
  //trigger use beta?
  #[getter] 
  fn get_gaps_trigger_use_beta(&self) -> PyResult<bool> {
    Ok(self.config.gaps_trigger_use_beta)
  }
  #[setter]
  fn set_gaps_trigger_use_beta(&mut self, gaps_trigger_use_beta: bool) -> PyResult<()> {
    self.config.gaps_trigger_use_beta = gaps_trigger_use_beta;
    Ok(())
  }

  //tiu emulation mode?
  #[getter] 
  fn get_tiu_emulation_mode(&self) -> PyResult<bool> {
    Ok(self.config.tiu_emulation_mode)
  }

  #[setter]
  fn set_tiu_emulation_mode(&mut self, tiu_emulation_mode: bool) -> PyResult<()> {
    self.config.tiu_emulation_mode = tiu_emulation_mode;
    Ok(())
  }
//trigger type
  #[getter] 
  fn get_trigger_type(&self) -> PyResult<TriggerType> {
    Ok(self.config.trigger_type)
  }

  #[setter]
  fn set_trigger_type(&mut self, trigger_type: TriggerType) -> PyResult<()> {
    self.config.trigger_type = trigger_type;
    Ok(())
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
}

#[pyclass]
#[pyo3(name="TofCommand")]
pub struct PyTofCommand {
  pub command : TofCommandV2
}

impl PyTofCommand {
  pub fn set_command(&mut self, cmd : TofCommandV2) {
    self.command = cmd;
  }
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

  fn set_command_code(&mut self, command_code : TofCommandCode) {
    self.command.command_code = command_code;
  }

  fn to_bytestream(&self) -> Vec<u8> {
    self.command.to_bytestream()
  }

  fn pack(&self) -> PyTofPacket {
    let packet   = self.command.pack();
    let mut pytp = PyTofPacket::new();
    pytp.set_tp(packet);
    pytp
  }


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
    RBMoniData::keys()
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

  /// Get the RB link IDs according to the mask
  pub fn get_rb_link_ids(&self) -> Vec<u8> {
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
  pub fn get_trigger_hits(&self) -> PyResult<Vec<(u8, u8, (u8, u8), LTBThreshold)>> {
    Ok(self.event.get_trigger_hits())
  }

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
  //pub fn get_trigger_sources(&self) -> Vec<x> {
  //
  //}
}

#[pyclass]
#[pyo3(name="RBEventHeader")]
pub struct PyRBEventHeader {
  header : RBEventHeader,
}

impl PyRBEventHeader {
  pub fn set_header(&mut self, header : RBEventHeader) {
    self.header = header;
  }
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
  
  #[getter]
  fn status_byte(&self) -> u8 {
    self.header.status_byte
  }
  
  #[getter]
  fn channel_mask(&self) -> u16 {
    self.header.channel_mask
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

}

#[pyclass]
#[pyo3(name="TofEvent")]
pub struct PyTofEvent {
  event : TofEvent,
}

impl PyTofEvent {
  pub fn set_event(&mut self, event : TofEvent) {
    self.event = event;
  }
}


#[pymethods]
impl PyTofEvent {
  #[new]
  fn new() -> Self {
    Self {
      event : TofEvent::new(),
    }
  }

  #[getter]
  fn mastertriggerevent(&self) ->  PyMasterTriggerEvent {
    let mut mte = PyMasterTriggerEvent::new();
    mte.set_event(self.event.mt_event.clone());
    mte
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
        pyhit.set_hit(*h);
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
      pywf.set_wf(wf.clone());
      wfs.push(pywf);
    }
    wfs
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
    py_header.set_header(self.event.header.clone());
    py_header
  }
  
  #[getter]
  fn waveforms(&self) -> Vec<PyRBWaveform> {
    let mut wfs = Vec::<PyRBWaveform>::new();
    for wf in &self.event.get_rbwaveforms() {
      let mut pywf = PyRBWaveform::new();
      pywf.set_wf(wf.clone());
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

impl PyTofHit {
  pub fn set_hit(&mut self, hit : TofHit) {
    self.hit = hit;
  }
}

#[pymethods]
impl PyTofHit {
  #[new]
  fn new() -> Self {
    Self {
      hit : TofHit::new(),
    }
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
  wf : RBWaveform,
}

impl PyRBWaveform {
  pub fn set_wf(&mut self, wf : RBWaveform) {
    self.wf = wf;
  }
}

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
  fn rb_channel(&self) -> u8 {
    self.wf.rb_channel
  }
  
  #[getter]
  fn stop_cell(&self) -> u16 {
    self.wf.stop_cell
  }
  
  #[getter]
  fn adc<'_py>(&self, py: Python<'_py>) ->  PyResult<Bound<'_py, PyArray1<u16>>> {
    let wf  = self.wf.adc.clone();
    let arr = PyArray1::<u16>::from_vec_bound(py, wf);
    Ok(arr)
  }
  
  #[getter]
  fn voltages<'_py>(&self, py: Python<'_py>) ->  PyResult<Bound<'_py, PyArray1<f32>>> {
    let wf  = self.wf.voltages.clone();
    let arr = PyArray1::<f32>::from_vec_bound(py, wf);
    Ok(arr)
  }

  #[getter]
  fn times<'_py>(&self, py: Python<'_py>) ->  PyResult<Bound<'_py, PyArray1<f32>>> {
    let times  = self.wf.nanoseconds.clone();
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



