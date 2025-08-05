//! The following file is part of gaps-online-software and published 
//! under the GPLv3 license

use std::fmt;
#[cfg(feature="random")]
use crate::random::FromRandom;
#[cfg(feature="random")]
use rand::Rng;

use crate::io::serialization::Serialization;
use crate::packets::{
  TofPackable,
  TofPacketType
};
use crate::calibration::tof::clean_spikes;
use crate::errors::{
  SerializationError,
  CalibrationError
};
use crate::constants::NWORDS;
use crate::io::parsers::{
  parse_u8,
  parse_u16,
  parse_u32,
  u8_to_u16,
};


#[cfg(feature="pybindings")]
use pyo3::prelude::*;

#[cfg(feature="pybindings")]
use pyo3::exceptions::PyIOError;

#[cfg(feature="pybindings")]
use crate::packets::TofPacket;

#[cfg(feature="pybindings")]
use numpy::PyArray1;

#[cfg(feature="pybindings")]
use crate::impl_pythonize_display;

/// Waveform container for Tof waveforms
/// This holds the waveforms for both 
/// paddle ends. Fields are available to 
/// hold calibrated waveforms, however,
/// only adc will be saved to disk.
#[derive(Debug, Clone, PartialEq)]
#[cfg_attr(feature="pybindings", pyclass)]
pub struct RBWaveform {
  pub event_id      : u32,
  pub rb_id         : u8,
  /// FIXME - this is form 0-8, but should it be from 1-9?
  pub rb_channel_a  : u8,
  pub rb_channel_b  : u8,
  /// DRS4 stop cell
  pub stop_cell     : u16,
  pub adc_a         : Vec<u16>,
  pub adc_b         : Vec<u16>,
  pub paddle_id     : u8,
  pub voltages_a    : Vec<f32>,
  pub nanoseconds_a : Vec<f32>,
  pub voltages_b    : Vec<f32>,
  pub nanoseconds_b : Vec<f32>
}

impl RBWaveform {
  
  pub fn new() -> Self {
    Self {
      event_id       : 0,
      rb_id          : 0,
      rb_channel_a   : 0,
      rb_channel_b   : 0,
      stop_cell      : 0,
      paddle_id      : 0,
      adc_a          : Vec::<u16>::new(),
      voltages_a     : Vec::<f32>::new(),
      nanoseconds_a  : Vec::<f32>::new(),
      adc_b          : Vec::<u16>::new(),
      voltages_b     : Vec::<f32>::new(),
      nanoseconds_b  : Vec::<f32>::new()
    }
  }

  //pub fn calibrate(&mut self, cali : &RBCalibrations) -> Result<(), CalibrationError>  {
  //  if cali.rb_id != self.rb_id {
  //    error!("Calibration is for board {}, but wf is for {}", cali.rb_id, self.rb_id);
  //    return Err(CalibrationError::WrongBoardId);
  //  }
  //  let mut voltages = vec![0.0f32;1024];
  //  let mut nanosecs = vec![0.0f32;1024];
  //  cali.voltages(self.rb_channel_a as usize + 1,
  //                self.stop_cell as usize,
  //                &self.adc_a,
  //                &mut voltages);
  //  self.voltages_a = voltages.clone();
  //  cali.nanoseconds(self.rb_channel_a as usize + 1,
  //                   self.stop_cell as usize,
  //                   &mut nanosecs);
  //  self.nanoseconds_a = nanosecs.clone();
  //  cali.voltages(self.rb_channel_b as usize + 1,
  //                self.stop_cell as usize,
  //                &self.adc_b,
  //                &mut voltages);
  //  self.voltages_b = voltages;
  //  cali.nanoseconds(self.rb_channel_b as usize + 1,
  //                   self.stop_cell as usize,
  //                   &mut nanosecs);
  //  self.nanoseconds_b = nanosecs;
  //  Ok(())
  //}

  /// Apply Jamie's simple spike filter to the calibrated voltages
  pub fn apply_spike_filter(&mut self) {
    clean_spikes(&mut self.voltages_a, true);
    clean_spikes(&mut self.voltages_b, true);
  }
}

impl TofPackable for RBWaveform {
  const TOF_PACKET_TYPE : TofPacketType = TofPacketType::RBWaveform;
}

impl Serialization for RBWaveform {
  const HEAD               : u16    = 43690; //0xAAAA
  const TAIL               : u16    = 21845; //0x5555
  
  fn from_bytestream(stream : &Vec<u8>, pos : &mut usize)
    -> Result<Self, SerializationError> {
    let mut wf           = RBWaveform::new();
    if parse_u16(stream, pos) != Self::HEAD {
      error!("The given position {} does not point to a valid header signature of {}", pos, Self::HEAD);
      return Err(SerializationError::HeadInvalid {});
    }
    wf.event_id          = parse_u32(stream, pos);
    wf.rb_id             = parse_u8 (stream, pos);
    wf.rb_channel_a      = parse_u8 (stream, pos);
    wf.rb_channel_b      = parse_u8 (stream, pos);
    wf.stop_cell         = parse_u16(stream, pos);
    wf.paddle_id         = parse_u8 (stream, pos);
    if stream.len() < *pos+2*NWORDS {
      return Err(SerializationError::StreamTooShort);
    }
    let data_a           = &stream[*pos..*pos+2*NWORDS];
    wf.adc_a             = u8_to_u16(data_a);
    *pos += 2*NWORDS;
    let data_b           = &stream[*pos..*pos+2*NWORDS];
    wf.adc_b             = u8_to_u16(data_b);
    *pos += 2*NWORDS;
    if parse_u16(stream, pos) != Self::TAIL {
      error!("The given position {} does not point to a tail signature of {}", pos, Self::TAIL);
      return Err(SerializationError::TailInvalid);
    }
    Ok(wf)
  }

  fn to_bytestream(&self) -> Vec<u8> {
    let mut stream = Vec::<u8>::new();
    stream.extend_from_slice(&Self::HEAD.to_le_bytes());
    stream.extend_from_slice(&self.event_id.to_le_bytes());
    stream.extend_from_slice(&self.rb_id.to_le_bytes());
    stream.extend_from_slice(&self.rb_channel_a.to_le_bytes());
    stream.extend_from_slice(&self.rb_channel_b.to_le_bytes());
    stream.extend_from_slice(&self.stop_cell.to_le_bytes());
    stream.push(self.paddle_id);
    if self.adc_a.len() != 0 {
      for k in 0..NWORDS {
        stream.extend_from_slice(&self.adc_a[k].to_le_bytes());  
      }
    }
    if self.adc_b.len() != 0 {
      for k in 0..NWORDS {
        stream.extend_from_slice(&self.adc_b[k].to_le_bytes());  
      }
    }
    stream.extend_from_slice(&Self::TAIL.to_le_bytes());
    stream
  }
}

impl fmt::Display for RBWaveform {
  fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
    let mut repr = String::from("<RBWaveform:");
    repr += &(format!("\n  Event ID  : {}", self.event_id));
    repr += &(format!("\n  RB        : {}", self.rb_id));
    repr += &(format!("\n  ChannelA  : {}", self.rb_channel_a));
    repr += &(format!("\n  ChannelB  : {}", self.rb_channel_b));
    repr += &(format!("\n  Paddle ID : {}", self.paddle_id));
    repr += &(format!("\n  Stop cell : {}", self.stop_cell));
    if self.adc_a.len() >= 273 {
      repr += &(format!("\n  adc [A] [{}]      : .. {} {} {} ..",self.adc_a.len(), self.adc_a[270], self.adc_a[271], self.adc_a[272]));
    } else {
      repr += &(String::from("\n  adc [A] [EMPTY]"));
    }
    if self.adc_b.len() >= 273 {
      repr += &(format!("\n  adc [B] [{}]      : .. {} {} {} ..",self.adc_b.len(), self.adc_b[270], self.adc_b[271], self.adc_b[272]));
    } else {
      repr += &(String::from("\n  adc [B] [EMPTY]"));
    }
    write!(f, "{}", repr)
  }
}

//---------------------------------------------------

#[cfg(feature="pybindings")]
#[pymethods]
impl RBWaveform {
  
  #[new]
  fn new_py() -> Self {
    Self::new()
  }

  /// Paddle ID of this wveform (1-160)
  #[getter]
  fn get_paddle_id(&self) -> u8 {
    self.paddle_id
  }

  #[getter]
  fn get_rb_id(&self) -> u8 {
    self.rb_id
  }
  
  #[getter]
  fn get_event_id(&self) -> u32 {
    self.event_id
  }
  
  #[getter]
  fn get_rb_channel_a(&self) -> u8 {
    self.rb_channel_a
  }
  
  #[getter]
  fn get_rb_channel_b(&self) -> u8 {
    self.rb_channel_b
  }
  
  #[getter]
  fn get_stop_cell(&self) -> u16 {
    self.stop_cell
  }
  
  #[getter]
  fn get_adc_a<'_py>(&self, py: Python<'_py>) ->  PyResult<Bound<'_py, PyArray1<u16>>> {
    let arr = PyArray1::<u16>::from_slice(py, self.adc_a.as_slice());
    Ok(arr)
  }
  
  #[getter]
  fn get_adc_b<'_py>(&self, py: Python<'_py>) ->  PyResult<Bound<'_py, PyArray1<u16>>> {
    let arr = PyArray1::<u16>::from_slice(py, self.adc_b.as_slice());
    Ok(arr)
  }
  
  #[getter]
  fn get_voltages_a<'_py>(&self, py: Python<'_py>) ->  PyResult<Bound<'_py, PyArray1<f32>>> {
    let arr = PyArray1::<f32>::from_slice(py, self.voltages_a.as_slice());
    Ok(arr)
  }

  #[getter]
  fn get_times_a<'_py>(&self, py: Python<'_py>) ->  PyResult<Bound<'_py, PyArray1<f32>>> {
    let times  = self.nanoseconds_a.clone();
    let arr    = PyArray1::<f32>::from_vec(py, times);
    Ok(arr)
  }

  #[getter]
  fn get_voltages_b<'_py>(&self, py: Python<'_py>) ->  PyResult<Bound<'_py, PyArray1<f32>>> {
    let arr = PyArray1::<f32>::from_slice(py, self.voltages_b.as_slice());
    Ok(arr)
  }

  #[getter]
  fn get_times_b<'_py>(&self, py: Python<'_py>) ->  PyResult<Bound<'_py, PyArray1<f32>>> {
    let arr = PyArray1::<f32>::from_slice(py, self.nanoseconds_b.as_slice());
    Ok(arr)
  }
  
  #[pyo3(name="apply_spike_filter")]
  fn apply_spike_filter_py(&mut self) {
    self.apply_spike_filter();
  }
  
  #[staticmethod]
  #[pyo3(name="from_tofpacket")]
  fn from_tofpacket(packet : &TofPacket) -> PyResult<Self> {
    match packet.unpack::<Self>() {
      Ok(wf) => {
        return Ok(wf);
      }
      Err(err) => {
        let err_msg = format!("Unable to unpack TofPacket of type {}! Is this really a RBWaveform? {err}", packet.packet_type);
        return Err(PyIOError::new_err(err_msg));
      }
    }
  }

  #[cfg(feature="random")]
  #[staticmethod]
  #[pyo3(name="from_random")]
  fn from_random_py() -> Self {
    Self::from_random()
  }
}

#[cfg(feature="pybindings")]
impl_pythonize_display!(RBWaveform, |s: &RBWaveform | s.to_string());

//---------------------------------------------------

#[cfg(feature = "random")]
impl FromRandom for RBWaveform {
    
  fn from_random() -> Self {
    let mut wf      = Self::new();
    let mut rng     = rand::thread_rng();
    wf.event_id     = rng.random::<u32>();
    wf.rb_id        = rng.random::<u8>();
    wf.rb_channel_a = rng.random::<u8>();
    wf.rb_channel_b = rng.random::<u8>();
    wf.stop_cell    = rng.random::<u16>();
    wf.paddle_id    = rng.random::<u8>();
    let random_numbers_a: Vec<u16> = (0..NWORDS).map(|_| rng.random()).collect();
    wf.adc_a        = random_numbers_a;
    let random_numbers_b: Vec<u16> = (0..NWORDS).map(|_| rng.random()).collect();
    wf.adc_b        = random_numbers_b;
    wf
  }
}

#[test]
#[cfg(feature = "random")]
fn pack_rbwaveform() {
  for _ in 0..100 {
    let wf   = RBWaveform::from_random();
    let test : RBWaveform = wf.pack().unpack().unwrap();
    assert_eq!(wf, test);
  }
}


