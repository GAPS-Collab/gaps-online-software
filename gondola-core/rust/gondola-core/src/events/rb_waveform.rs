// This file is part of gaps-online-software and published 
// under the GPLv3 license

use crate::prelude::*;


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

  /// Calculate the time in ns for which the waveform is 
  /// above a certain threshold for paddle end A
  pub fn time_over_threshold_a(&self, threshold : f32) -> f32 {
    let mut tot : f32 = 0.0;
    for k in 1..self.voltages_a.len() {
      if self.voltages_a[k] > threshold {
        tot += self.nanoseconds_a[k] - self.nanoseconds_a[k-1];
      }
    }
    return tot;
  }

  /// Calculate the time in ns for which the waveform is 
  /// above a certain threshold for paddle end B
  pub fn time_over_threshold_b(&self, threshold : f32) -> f32 {
    let mut tot : f32 = 0.0;
    for k in 1..self.voltages_b.len() {
      if self.voltages_b[k] > threshold {
        tot += self.nanoseconds_b[k] - self.nanoseconds_b[k-1];
      }
    }
    return tot;
  }
  
  pub fn charge_a_below_500(&self) -> f32 {

    let mut total_area = 0.0f32;
    for i in 0..(self.nanoseconds_a.len() - 1) {
        let h = self.nanoseconds_a[i + 1] - self.nanoseconds_a[i]; // The width of the trapezoid.
        let mut v1 = self.voltages_a[i];
        let mut v2 = self.voltages_a[i + 1];
        if v1 > 500.0 {
          v1 = 500.0;
        }
        if v2 > 500.0 { 
          v2 = 500.0; 
        }
        let area = h * (v1 + v2) / 2.0;
        total_area += area;
    } 
    total_area
  }
  
  pub fn charge_b_below_500(&self) -> f32 {

    let mut total_area = 0.0f32;
    for i in 0..(self.nanoseconds_b.len() - 1) {
        let h = self.nanoseconds_b[i + 1] - self.nanoseconds_b[i]; // The width of the trapezoid.
        let mut v1 = self.voltages_b[i];
        let mut v2 = self.voltages_b[i + 1];
        if v1 > 500.0 {
          v1 = 500.0;
        }
        if v2 > 500.0 { 
          v2 = 500.0; 
        }
        let area = h * (v1 + v2) / 2.0;
        total_area += area;
    } 
    total_area
  }

  //pub fn charge_b_trap_bottom_fwhm(&self) -> f32 {
  //  let mut total_area = 0.0f32;
  //  for i in 0..(self.nanoseconds_a.len() - 1) {
  //      let h = self.nanoseconds_a[i + 1] - self.nanoseconds_a[i]; // The width of the trapezoid.
  //      let area = h * (self.voltages_a[i] + self.voltages_a[i + 1]) / 2.0;
  //      total_area += area;
  //  } 
  //  total_area
  //}
  
  pub fn charge_a_trap(&self) -> f32 {
    let mut total_area = 0.0f32;
    for i in 0..(self.nanoseconds_a.len() - 1) {
        let h = self.nanoseconds_a[i + 1] - self.nanoseconds_a[i]; // The width of the trapezoid.
        let area = h * (self.voltages_a[i] + self.voltages_a[i + 1]) / 2.0;
        total_area += area;
    } 
    total_area
  }
  
  pub fn charge_b_trap(&self) -> f32 {
    let mut total_area = 0.0f32;
    for i in 0..(self.nanoseconds_b.len() - 1) {
        let h = self.nanoseconds_b[i + 1] - self.nanoseconds_b[i]; // The width of the trapezoid.
        let area = h * (self.voltages_b[i] + self.voltages_b[i + 1]) / 2.0;
        total_area += area;
    } 
    total_area
  }

  pub fn guess_max_peak_a(&self) -> f32 {
    let mut current_max = -1000.0f32;
    for k in 20..self.voltages_a.len() {
      if self.voltages_a[k] > current_max {
        current_max = self.voltages_a[k];
      }
    }
    current_max
  }
  
  pub fn guess_max_peak_b(&self) -> f32 {
    let mut current_max = -1000.0f32;
    for k in 20..self.voltages_b.len() {
      if self.voltages_b[k] > current_max {
        current_max = self.voltages_b[k];
      }
    }
    current_max
  }

  pub fn subtract_pedestals(&mut self) {
    let (ped_a, _ped_a_err) = calculate_pedestal(&self.voltages_a,
                                                 10.0,
                                                 700,
                                                 200);
    let (ped_b, _ped_b_err) = calculate_pedestal(&self.voltages_b,
                                                 10.0,
                                                 700,
                                                 200);
    for k in 0..self.voltages_a.len() {
      self.voltages_a[k] = self.voltages_a[k] - ped_a;
      self.voltages_b[k] = self.voltages_b[k] - ped_b;
    }
  }

  /// Apply a RB calibration to the waveform, filling the voltages and 
  /// nanoseconds fields
  pub fn calibrate(&mut self, cali : &RBCalibrations) -> Result<(), CalibrationError>  {
    if cali.rb_id != self.rb_id {
      error!("Calibration is for board {}, but wf is for {}", cali.rb_id, self.rb_id);
      return Err(CalibrationError::WrongBoardId);
    }
    let mut voltages = vec![0.0f32;1024];
    let mut nanosecs = vec![0.0f32;1024];
    cali.voltages(self.rb_channel_a as usize + 1,
                  self.stop_cell as usize,
                  &self.adc_a,
                  &mut voltages);
    self.voltages_a = voltages.clone();
    cali.nanoseconds(self.rb_channel_a as usize + 1,
                     self.stop_cell as usize,
                     &mut nanosecs);
    self.nanoseconds_a = nanosecs.clone();
    cali.voltages(self.rb_channel_b as usize + 1,
                  self.stop_cell as usize,
                  &self.adc_b,
                  &mut voltages);
    self.voltages_b = voltages;
    cali.nanoseconds(self.rb_channel_b as usize + 1,
                     self.stop_cell as usize,
                     &mut nanosecs);
    self.nanoseconds_b = nanosecs;
    Ok(())
  }

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
  
  /// Paddle ID of this wveform (1-160)
  #[getter]
  fn get_paddle_id(&self) -> u8 {
    self.paddle_id
  }

  #[getter]
  fn max_peak_a_guess(&self) -> f32 {
    self.guess_max_peak_a()
  }
  
  #[getter]
  fn max_peak_b_guess(&self) -> f32 {
    self.guess_max_peak_b()
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
 
  // FIXME - make this consistent 
  #[getter]
  fn get_adc_a<'_py>(&self, py: Python<'_py>) ->  Bound<'_py, PyArray1<u16>> {
    //let arr = PyArray1::<u16>::from_slice(py, self.adc_a.as_slice());
    self.adc_a.to_pyarray(py)
    //Ok(arr)
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
    let arr    = PyArray1::<f32>::from_slice(py, self.nanoseconds_a.as_slice());
    Ok(arr)
  }

  #[getter]
  fn get_voltages_b<'_py>(&self, py: Python<'_py>) ->  PyResult<Bound<'_py, PyArray1<f32>>> {
    let arr = PyArray1::<f32>::from_slice(py, self.voltages_b.as_slice());
    Ok(arr)
  }

  /// Time over threshold - waveform needs to be 
  /// calibrated. 
  /// Paddle end A
  ///
  /// # Arguments:
  ///   * threshold : value in mV
  fn get_tot_a(&self, threshold : f32) -> f32 {
    self.time_over_threshold_a(threshold)
  }
  
  /// Time over threshold - waveform needs to be 
  /// calibrated. 
  /// Paddle end B
  ///
  /// # Arguments:
  ///   * threshold : value in mV
  fn get_tot_b(&self, threshold : f32) -> f32 {
    self.time_over_threshold_b(threshold)
  }

  #[pyo3(name="subtract_pedestals")]
  fn subtract_pedestals_py(&mut self) {
    self.subtract_pedestals()
  }

  #[getter]
  fn get_charge_a_trap(&self) -> f32 {
    self.charge_a_trap()
  }
  
  #[getter]
  fn get_charge_a_below_500_trap(&self) -> f32 {
    self.charge_a_below_500() 
  } 
  
  #[getter]
  fn get_charge_b_below_500_trap(&self) -> f32 {
    self.charge_b_below_500() 
  } 


  #[getter]
  fn get_charge_b_trap(&self) -> f32 {
    self.charge_b_trap()
  }

  #[getter]
  fn get_times_b<'_py>(&self, py: Python<'_py>) ->  PyResult<Bound<'_py, PyArray1<f32>>> {
    let arr = PyArray1::<f32>::from_slice(py, self.nanoseconds_b.as_slice());
    Ok(arr)
  }
  
  /// Apply the readoutboard calibration to convert adc/bins
  /// to millivolts and nanoseconds
  #[pyo3(name="calibrate")]
  fn calibrate_py(&mut self, cali : &RBCalibrations) -> PyResult<()> {
    match self.calibrate(&cali) {
      Ok(_) => {
        return Ok(());
      }
      Err(err) => {
        return Err(PyValueError::new_err(err.to_string()));
      }
    }
  }
  
  #[pyo3(name="apply_spike_filter")]
  fn apply_spike_filter_py(&mut self) {
    self.apply_spike_filter();
  }
}

#[cfg(feature="pybindings")]
pythonize_packable!(RBWaveform);

//---------------------------------------------------

#[cfg(feature = "random")]
impl FromRandom for RBWaveform {
    
  fn from_random() -> Self {
    let mut wf      = Self::new();
    let mut rng     = rand::rng();
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

//---------------------------------------------------

#[test]
#[cfg(feature = "random")]
fn pack_rbwaveform() {
  for _ in 0..100 {
    let wf   = RBWaveform::from_random();
    let test : RBWaveform = wf.pack().unpack().unwrap();
    assert_eq!(wf, test);
  }
}


