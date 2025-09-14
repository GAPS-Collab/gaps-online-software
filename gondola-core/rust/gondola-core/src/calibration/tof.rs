//! Calibration routines for the GAPS TOF system
//!
// This file is part of gaps-online-software and published 
// under the GPLv3 license

use crate::prelude::*;

/// Roll over the entries from the end of a vector 
/// to the beginning by a given offset.
/// 
/// This is similar to 
/// https://numpy.org/doc/2.2/reference/generated/numpy.roll.html
///
/// # Arguments:
///   * `vec`   : The vector to be rolled over. It will 
///               be changed in place 
///   * `offset`: The signed number to shift elements by (can be to 
///               the left or right)
pub fn roll<T: Clone>(vec: &mut Vec<T>, offset: isize) {
  let len = vec.len() as isize;
  if len <= 1 {
      return;
  }
  let offset = offset % len;
  if offset == 0 {
      return;
  }
  let split_point = if offset > 0 {
      len - offset
  } else {
      -offset
  } as usize;

  let mut temp = Vec::with_capacity(len as usize);
  temp.extend_from_slice(&vec[split_point..]);
  temp.extend_from_slice(&vec[..split_point]);

  vec.clear();
  vec.extend_from_slice(&temp);
}

//-----------------------------------------------

/// Simplified version of spike cleaning 
///
/// Taken over from Jamie's python code
pub fn clean_spikes(trace : &mut Vec<f32>, vcaldone : bool) {
  //# TODO: make robust (symmetric, doubles, fixed/estimated spike height)
  let mut thresh : f32 = 360.0;
  if vcaldone {
    thresh = 16.0;
  }

  let mut spf_allch = vec![0usize;1023];
  let mut spf_sum   = vec![0f32;1024];
  let tracelen      = trace.len();
  let spikefilter0 = &trace[0..tracelen-3];
  let spikefilter1 = &trace[1..tracelen-2];
  let spikefilter2 = &trace[2..tracelen-1];
  let spikefilter3 = &trace[3..tracelen];
  let spf_len      = spikefilter0.len();
  for k in 0..spf_len {
    spf_sum[k] += spikefilter1[k] - spikefilter0[k] + spikefilter2[k] - spikefilter3[k];
  }
  for k in 0..spf_len {
    if spf_sum[k] > thresh {
      spf_allch[k] += 1;
    }
  }
  let mut spikes = Vec::<usize>::new();
  for k in 0..spf_allch.len() {
    if spf_allch[k] >= 2 {
      spikes.push(k);
    }
  }
  for spike in spikes.iter() {
    let d_v : f32 = (trace[spike+3] - trace[*spike])/3.0;
    trace[spike+1] = trace[*spike] + d_v;
    trace[spike+2] = trace[*spike] + 2.0*d_v;
  }
}

//-----------------------------------------------

//#[cfg(feature="pybindings")]
//#[pyfunction]
//#[pyo3(name="clean_spikes")]
//pub fn clean_spikes_pyx<'_py>(value : Bound<'_py,PyArray1<f32>>, vcal_done : bool) {
//  unsafe {
//    match clean_spikes(value.as_slice().unwrap(), vcal_done) {
//      Err(err) => {
//        return Err(PyValueError::new_err(err.to_string()));
//      }
//      Ok(max_val) => {
//        return Ok(max_val);
//      }
//    }
//  }
//}


//-----------------------------------------------

#[derive(Debug, Copy, Clone)]
#[repr(u8)]
pub enum Edge {
  Unknown = 0u8,
  Rising  = 10u8,
  Falling = 20u8,
  Average = 30u8,
  None    = 40u8
}

impl fmt::Display for Edge {
  fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
    let r : &str;
    match self {
      Edge::Unknown => {r = "Unknown"} 
      Edge::Rising  => {r = "Rising"},
      Edge::Falling => {r = "Falling"},
      Edge::Average => {r = "Average"},
      Edge::None    => {r = "None"}
    }
    write!(f, "<Edge: {}>", r)
  }
}

impl From<u8> for Edge {
  fn from(value: u8) -> Self {
    match value {
      0u8  => Edge::Unknown,
      10u8 => Edge::Rising,
      20u8 => Edge::Falling,
      30u8 => Edge::Average,
      40u8 => Edge::None,
      _    => Edge::Unknown
    }
  }
}

#[cfg(feature = "random")]
impl FromRandom for Edge {
  
  fn from_random() -> Self {
    let choices = [
      Edge::Rising,
      Edge::Falling,
      Edge::Average,
      Edge::None,
    ];
    let mut rng  = rand::rng();
    let idx = rng.random_range(0..choices.len());
    choices[idx]
  }
}


/// smaller packet to bring through the gcu
#[derive(Debug, Clone, PartialEq)]
#[cfg_attr(feature="pybindings", pyclass)]
pub struct RBCalibrationFlightT {
  pub rb_id      : u8,
  pub timestamp  : u32, // time the calibration was taken
  pub tbins      : [[f16;NWORDS];NCHN], // cell width (ns)
}

//-----------------------------------------------

/// Get the periods of a sine wave
///
/// # Arguments:
///   * dts : some version of tcal data? FIXME
pub fn get_periods(trace   : &Vec<f32>,
                   dts     : &Vec<f32>,
                   nperiod : f32,
                   nskip   : f32,
                   edge    : &Edge) -> (Vec<usize>, Vec<f32>) {
  let mut trace_c = trace.clone();
  let mut periods = Vec::<f32>::new();
  if trace_c.len() == 0 {
    let foo = Vec::<usize>::new();
    return (foo, periods);
  }
  let firstbin : usize = 20;
  let lastbin = firstbin + (nperiod * (900.0/nperiod).floor()).floor() as usize;
  //info!("firstbin {} lastbin {}", firstbin, lastbin);
  let mut vec_for_median = Vec::<f32>::new();
  for bin in firstbin..lastbin {
    if trace[bin].is_nan() {
      continue;
    }
    vec_for_median.push(trace[bin]);
  }
  let median_val = median(&vec_for_median);
  debug!("median val {median_val}");
  for k in 0..trace_c.len() {
    trace_c[k] -= median_val;
  }
  let mut zcs = find_zero_crossings(&trace_c);
  //trace!("Found {} zero crossings!", zcs.len());
  let mut zcs_nskip = Vec::<usize>::with_capacity(zcs.len());
  for k in 0..zcs.len() {
    if zcs[k] > nskip as usize {
      zcs_nskip.push(zcs[k]);
    }
  }
  zcs = zcs_nskip;
  let mut zcs_filter = Vec::<usize>::with_capacity(zcs.len());
  for k in 0..zcs.len() {
    match edge {
      Edge::Rising => {
        if trace_c[zcs[k]] < 0.0 {
          zcs_filter.push(zcs[k]);
        }
      }
      Edge::Falling => {
        // What about the equal case?
        if trace_c[zcs[k]] > 0.0 {
          zcs_filter.push(zcs[k]);
        }
      },
      Edge::None => {
        warn!("Unsure what to do for Edge::None");
      },
      Edge::Average => {
        warn!("Unsure what to do for Edge::Average");
      },
      Edge::Unknown => {
        warn!("Unsure what to do for Edge::Unknown");
      }
    }
  }
  debug!("Found {} zero crossings!", zcs_filter.len()); 
  zcs = zcs_filter;
  if zcs.len() < 3 {
    return (zcs, periods);
  }
 
  for k in 0..zcs.len() -1 {
    let zcs_a  = &zcs[k];
    let zcs_b  = &zcs[k+1];
    // FIXME - there is an issue with the last
    // zero crossings
    if zcs_a + 2 > trace_c.len() || zcs_b + 2 > trace_c.len() {
      continue;
    }
    let tr_a   = &trace_c[*zcs_a..*zcs_a+2];
    let tr_b   = &trace_c[*zcs_b..*zcs_b+2];
    let mut period : f32 = 0.0;
    for n in zcs_a+1..*zcs_b {
      period += dts[n];
    }
    period += dts[*zcs_a]*f32::abs(tr_a[1]/(tr_a[1] - tr_a[0])); // first semi bin
    period += dts[*zcs_b]*f32::abs(tr_b[0]/(tr_b[1] - tr_b[0])); // first semi bin
    if period.is_nan() {
      error!("NAN in period found!");
      continue;
    }
    
    if f32::abs(*zcs_b as f32 - *zcs_a as f32 - nperiod) > 5.0 {
      let mut zcs_tmp = Vec::<usize>::new();
      zcs_tmp.extend_from_slice(&zcs[0..k+1]);
      zcs = zcs_tmp;
      break;
    }
    trace!("zcs_a, zcs_b, period, nperiod {} {} {} {}", zcs_a, zcs_b, period, nperiod);
    periods.push(period);
  }
  debug!("Calculated {} zero-crossings and {} periods!", zcs.len(), periods.len());
  (zcs, periods)
}

//-----------------------------------------------

/// Designed to match np.where(np.diff(np.signbit(trace)))\[0\] 
/// FIXME -> This needs to be moved to analysis!
pub fn find_zero_crossings(trace: &Vec<f32>) -> Vec<usize> {
  let mut zero_crossings = Vec::new();
  for i in 1..trace.len() {
    // acccount for the fact that sometimes the second/first point can be 0
    if (trace[i - 1] > 0.0 && trace[i] < 0.0) || (trace[i - 1] < 0.0 && trace[i] > 0.0) {
      zero_crossings.push(i - 1);
    }
    if i < trace.len() - 1 {
      if trace[i - 1] > 0.0 && trace[i] == 0.0 && trace[i+1] < 0.0 {
        zero_crossings.push(1);
      }
      if trace[i - 1] < 0.0 && trace[i] == 0.0 && trace[i+1] > 0.0 {
        zero_crossings.push(i);
      }
    }
  }
  zero_crossings
}

//-----------------------------------------------

impl RBCalibrationFlightT {
  pub fn new() -> Self {
    Self {
      rb_id     : 0,
      timestamp : 0,
      tbins     : [[f16::from_f32(0.0);NWORDS];NCHN],
    }
  }
}

impl Serialization for RBCalibrationFlightT {
  const SIZE            : usize = NCHN*NWORDS*2 + 4 + 1; 
  const HEAD            : u16   = 0xAAAA; // 43690 
  const TAIL            : u16   = 0x5555; // 21845 
  
  fn from_bytestream(bytestream : &Vec<u8>, 
                     pos        : &mut usize)
    -> Result<Self, SerializationError> { 
    let mut rb_cal = Self::new();
    if parse_u16(bytestream, pos) != Self::HEAD {
      return Err(SerializationError::HeadInvalid {});
    }
    rb_cal.rb_id                = parse_u8(bytestream, pos);
    rb_cal.timestamp            = parse_u32(bytestream, pos);
    for ch in 0..NCHN {
      for k in 0..NWORDS {
        let value         = parse_f16(bytestream, pos);
        rb_cal.tbins[ch][k]      = value;
      }
    }
    *pos += 2;
    Ok(rb_cal)
  }

  fn to_bytestream(&self) -> Vec<u8> {
    
    let mut bs = Vec::<u8>::with_capacity(Self::SIZE);
    bs.extend_from_slice(&Self::HEAD.to_le_bytes());
    bs.extend_from_slice(&self.rb_id.to_le_bytes());
    bs.extend_from_slice(&self.timestamp.to_le_bytes());
    for ch in 0..NCHN {
      for k in 0..NWORDS {
        bs.extend_from_slice(&self.tbins[ch][k]     .to_le_bytes());
      }
    }
    bs.extend_from_slice(&Self::TAIL.to_le_bytes());
    bs
  }
}

impl TofPackable for RBCalibrationFlightT {
  const TOF_PACKET_TYPE : TofPacketType = TofPacketType::RBCalibrationFlightT;
}

impl fmt::Display for RBCalibrationFlightT {
  fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
    let mut timestamp_str = String::from("?");
    match Utc.timestamp_opt(self.timestamp.into(), 0) {
      LocalResult::Single(datetime_utc) => {
        timestamp_str = datetime_utc.format("%Y/%m/%d %H:%M:%S").to_string();
      },
      LocalResult::Ambiguous(_, _) => {
        error!("The given timestamp is ambiguous.");
      },
      LocalResult::None => {
        error!("The given timestamp is not valid.");
      },
    }

    //let datetime_utc: DateTime<Utc> = Utc.timestamp_opt(self.timestamp as i64, 0).earliest().unwrap_or(DateTime::<Utc>::from_timestamp_millis(0).unwrap());
    write!(f, 
  "<RBCalibrationFlightT [{} UTC]:
      RB             : {}
      T Bins    [ch0]: .. {:?} {:?} ..>",
      timestamp_str,
      self.rb_id,
      self.tbins[0][98],
      self.tbins[0][99]
    )
  } 
}

#[cfg(feature = "random")]
impl FromRandom for RBCalibrationFlightT {
    
  fn from_random() -> Self {
    let mut cali   = Self::new();
    let mut rng    = rand::rng();
    cali.rb_id     = rng.random::<u8>();
    cali.timestamp = rng.random::<u32>();
    for ch in 0..NCHN {
      for n in 0..NWORDS { 
        cali.tbins     [ch][n] = f16::from_f32(rng.random::<f32>());
      }
    }
    cali
  }
}

//-----------------------------------------------

#[cfg(feature="pybindings")]
#[pymethods]
impl RBCalibrationFlightT {
  #[getter]
  fn get_rb_id(&self) -> u8 {
    self.rb_id
  }
  
  #[getter]
  fn get_timestamp(&self) -> u32 {
    self.timestamp
  }

  fn get_tbins<'_py>(&self, py: Python<'_py>, channel : u8 ) -> PyResult<Bound<'_py, PyArray1<f32>>> {
    if channel < 1 || channel > 9 {
      return Err(PyValueError::new_err("Channel must be > 0 and < 9"));
    }
    let mut py_tbins = [0.0f32;NWORDS];
    for k in 0..NWORDS {
      py_tbins[k] = self.tbins[channel as usize][k].to_f32();
    };
    let py_array = py_tbins.to_pyarray(py);
    Ok(py_array)
  }
}

#[cfg(feature="pybindings")]
pythonize_packable!(RBCalibrationFlightT);

//-----------------------------------------------

#[derive(Debug, Clone, PartialEq)]
#[cfg_attr(feature="pybindings", pyclass)]
pub struct RBCalibrationFlightV {
  pub rb_id      : u8,
  pub d_v        : f32, // input voltage difference between 
                        // vcal_data and noi data
  pub timestamp  : u32, // time the calibration was taken
  pub v_offsets : [[f16;NWORDS];NCHN], // voltage offset
  pub v_dips    : [[f16;NWORDS];NCHN], // voltage "dip" (time-dependent correction)
  pub v_inc     : [[f16;NWORDS];NCHN], // voltage increment (mV/ADC unit)
}

impl RBCalibrationFlightV {
  pub fn new() -> Self {
    Self {
      rb_id     : 0,
      d_v       : 0.9,
      timestamp : 0,
      v_offsets : [[f16::from_f32(0.0);NWORDS];NCHN],
      v_dips    : [[f16::from_f32(0.0);NWORDS];NCHN],
      v_inc     : [[f16::from_f32(0.0);NWORDS];NCHN],
    }
  }
}

impl Serialization for RBCalibrationFlightV {
  const SIZE            : usize = NCHN*NWORDS*2*3 + 4 + 4 + 1; 
  const HEAD            : u16   = 0xAAAA; // 43690 
  const TAIL            : u16   = 0x5555; // 21845 

  fn from_bytestream(bytestream : &Vec<u8>, 
                     pos        : &mut usize)
    -> Result<Self, SerializationError> { 
    let mut rb_cal = Self::new();
    if parse_u16(bytestream, pos) != Self::HEAD {
      return Err(SerializationError::HeadInvalid {});
    }
    rb_cal.rb_id                = parse_u8(bytestream, pos);
    rb_cal.d_v                  = parse_f32(bytestream, pos);
    rb_cal.timestamp            = parse_u32(bytestream, pos);
    for ch in 0..NCHN {
      for k in 0..NWORDS {
        let mut value = parse_f16(bytestream, pos);
        rb_cal.v_offsets[ch][k] = value;
        value         = parse_f16(bytestream, pos);
        rb_cal.v_dips[ch][k]    = value;
        value         = parse_f16(bytestream, pos);
        rb_cal.v_inc[ch][k]     = value;
      }
    }
    *pos += 2;
    Ok(rb_cal)
  }
  
  fn to_bytestream(&self) -> Vec<u8> {
    let mut bs = Vec::<u8>::with_capacity(Self::SIZE);
    bs.extend_from_slice(&Self::HEAD.to_le_bytes());
    bs.extend_from_slice(&self.rb_id.to_le_bytes());
    bs.extend_from_slice(&self.d_v.to_le_bytes());
    bs.extend_from_slice(&self.timestamp.to_le_bytes());
    for ch in 0..NCHN {
      for k in 0..NWORDS {
        bs.extend_from_slice(&self.v_offsets[ch][k].to_le_bytes());
        bs.extend_from_slice(&self.v_dips[ch][k]   .to_le_bytes());
        bs.extend_from_slice(&self.v_inc[ch][k]    .to_le_bytes());
      }
    }
    bs.extend_from_slice(&Self::TAIL.to_le_bytes());
    bs
  }
}

impl TofPackable for RBCalibrationFlightV {
  const TOF_PACKET_TYPE : TofPacketType = TofPacketType::RBCalibrationFlightV;
}

impl fmt::Display for RBCalibrationFlightV {
  fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
    let mut timestamp_str = String::from("?");
    match Utc.timestamp_opt(self.timestamp.into(), 0) {
      LocalResult::Single(datetime_utc) => {
        timestamp_str = datetime_utc.format("%Y/%m/%d %H:%M:%S").to_string();
      },
      LocalResult::Ambiguous(_, _) => {
        error!("The given timestamp is ambiguous.");
      },
      LocalResult::None => {
        error!("The given timestamp is not valid.");
      },
    }
    write!(f, 
  "<RBCalibrationFlightV [{} UTC]:
      RB             : {}
      V Offsets [ch0]: .. {:?} {:?} ..
      V Incrmts [ch0]: .. {:?} {:?} ..
      V Dips    [ch0]: .. {:?} {:?} ..>",
      timestamp_str,
      self.rb_id,
      self.v_offsets[0][98],
      self.v_offsets[0][99],
      self.v_inc[0][98],
      self.v_inc[0][99],
      self.v_dips[0][98],
      self.v_dips[0][99]
    )
  } 
}

#[cfg(feature = "random")]
impl FromRandom for RBCalibrationFlightV {
    
  fn from_random() -> Self {
    let mut cali     = Self::new();
    let mut rng      = rand::rng();
    cali.rb_id       = rng.random::<u8>();
    cali.d_v         = rng.random::<f32>();
    cali.timestamp   = rng.random::<u32>();
    for ch in 0..NCHN {
      for n in 0..NWORDS { 
        cali.v_offsets  [ch][n] = f16::from_f32(rng.random::<f32>());
        cali.v_dips     [ch][n] = f16::from_f32(rng.random::<f32>());
        cali.v_inc      [ch][n] = f16::from_f32(rng.random::<f32>());
      }
    }
    cali
  }
}

//-----------------------------------------------

#[cfg(feature="pybindings")]
#[pymethods]
impl RBCalibrationFlightV {
  #[getter]
  fn get_rb_id(&self) -> u8 {
    self.rb_id
  }
 
  #[getter]
  fn get_d_v(&self) -> f32 {
    self.d_v
  }

  #[getter]
  fn get_timestamp(&self) -> u32 {
    self.timestamp
  }

  fn get_v_offsets<'_py>(&self, py: Python<'_py>, channel : u8 ) -> PyResult<Bound<'_py, PyArray1<f32>>> {
    if channel < 1 || channel > 9 {
      return Err(PyValueError::new_err("Channel must be > 0 and < 9"));
    }
    let mut py_voffsets = [0.0f32;NWORDS];
    for k in 0..NWORDS {
      py_voffsets[k] = self.v_offsets[channel as usize][k].to_f32();
    };
    let py_array = py_voffsets.to_pyarray(py);
    Ok(py_array)
  }

  fn get_v_dips<'_py>(&self, py: Python<'_py>, channel : u8 ) -> PyResult<Bound<'_py, PyArray1<f32>>> {
    if channel < 1 || channel > 9 {
      return Err(PyValueError::new_err("Channel must be > 0 and < 9"));
    }
    let mut py_vdips = [0.0f32;NWORDS];
    for k in 0..NWORDS {
      py_vdips[k] = self.v_dips[channel as usize][k].to_f32();
    };
    let py_array = py_vdips.to_pyarray(py);
    Ok(py_array)
  }

  fn get_v_inc<'_py>(&self, py: Python<'_py>, channel : u8 ) -> PyResult<Bound<'_py, PyArray1<f32>>> {
    if channel < 1 || channel > 9 {
      return Err(PyValueError::new_err("Channel must be > 0 and < 9"));
    }
    let mut py_vinc = [0.0f32;NWORDS];
    for k in 0..NWORDS {
      py_vinc[k] = self.v_inc[channel as usize][k].to_f32();
    };
    let py_array = py_vinc.to_pyarray(py);
    Ok(py_array)
  }
}

#[cfg(feature="pybindings")]
pythonize_packable!(RBCalibrationFlightV);

//-----------------------------------------------

// new way of property testing using quicktest!
#[test]
fn prop_roll_then_unroll_gives_original() {
  fn prop(mut vec: Vec<u8>, offset: i8) -> bool {
    let original = vec.clone();
    let offset = offset as isize;

    roll(&mut vec, offset);
    roll(&mut vec, -offset);

    vec == original
  }
  quickcheck::QuickCheck::new().tests(100).quickcheck(prop as fn(Vec<u8>, i8) -> bool);
}

//-----------------------------------------------

#[derive(Debug, Clone, PartialEq)]
#[cfg_attr(feature="pybindings", pyclass)]
pub struct RBCalibrations {
  pub rb_id      : u8,
  pub d_v        : f32, // input voltage difference between 
                        // vcal_data and noi data
  pub timestamp  : u32, // time the calibration was taken
  pub serialize_event_data : bool,
  // calibration constants
  pub v_offsets : [[f32;NWORDS];NCHN], // voltage offset
  pub v_dips    : [[f32;NWORDS];NCHN], // voltage "dip" (time-dependent correction)
  pub v_inc     : [[f32;NWORDS];NCHN], // voltage increment (mV/ADC unit)
  pub tbin      : [[f32;NWORDS];NCHN], // cell width (ns)

  // calibration data
  pub vcal_data : Vec::<RBEvent>,
  pub tcal_data : Vec::<RBEvent>,
  pub noi_data  : Vec::<RBEvent>,
}

impl RBCalibrations {
  // skip the first n cells for the 
  // voltage calibration. Historically,
  // this had been 2.
  pub const NSKIP       : usize = 2;
  pub const SINMAX      : usize = 60; // ~1000 ADC units
  pub const DVCUT       : f32   = 15.0; // ns away that should be considered
  pub const NOMINALFREQ : f32   = 2.0; // nominal sampling frequency,
                                       // GHz
  pub const CALFREQ     : f32   = 0.025; // calibration sine wave frequency (25MHz)
 
  /// Re-assemble a RBCalibration from chopped up parts
  pub fn assemble_from_flightcal(fcal_t : RBCalibrationFlightT,
                                 fcal_v : RBCalibrationFlightV)
    -> Result<Self, CalibrationError> {
    if (fcal_t.timestamp != fcal_v.timestamp) || (fcal_t.rb_id != fcal_v.rb_id) {
      error!("These calibrations do not match! {} , {}", fcal_t, fcal_v);
      return Err(CalibrationError::IncompatibleFlightCalibrations);
    }
    let mut cal              = Self::new(fcal_t.rb_id);
    cal.rb_id                = fcal_t.rb_id;
    cal.timestamp            = fcal_t.timestamp;
    cal.d_v                  = fcal_v.d_v;
    cal.serialize_event_data = false;
    for ch in 0..NCHN {
      for k in 0..NWORDS {
        cal.tbin     [ch][k] = f16::to_f32(fcal_t.tbins[ch][k]);
        cal.v_offsets[ch][k] = f16::to_f32(fcal_v.v_offsets[ch][k]); 
        cal.v_dips   [ch][k] = f16::to_f32(fcal_v.v_dips[ch][k]);
        cal.v_inc    [ch][k] = f16::to_f32(fcal_v.v_inc[ch][k]); 
      }
    }
    Ok(cal)
  }

  /// Return the timing part of the calibration in a 
  /// package digestable by the flight computer.
  ///
  /// Additonal compression by using f16
  pub fn emit_flighttcal(&self) -> RBCalibrationFlightT {
    let mut cal = RBCalibrationFlightT::new();
    cal.rb_id = self.rb_id;
    cal.timestamp = self.timestamp;
    for ch in 0..NCHN {
      for n in 0..NWORDS { 
        cal.tbins  [ch][n] = f16::from_f32(self.tbin[ch][n]);
      }
    }
    cal
  }
  
  /// Return the voltage part of the calibration in a 
  /// package digestable by the flight computer.
  ///
  /// Additional compression by using f16
  pub fn emit_flightvcal(&self) -> RBCalibrationFlightV {
    let mut cal = RBCalibrationFlightV::new();
    cal.rb_id = self.rb_id;
    cal.timestamp = self.timestamp;
    cal.d_v   = self.d_v;
    for ch in 0..NCHN {
      for n in 0..NWORDS { 
        cal.v_offsets  [ch][n] = f16::from_f32(self.v_offsets[ch][n]);
        cal.v_dips     [ch][n] = f16::from_f32(self.v_dips[ch][n]);
        cal.v_inc      [ch][n] = f16::from_f32(self.v_inc[ch][n]);
      }
    }
    cal
  }

  /// Remove events with invalid traces or event fragment bits set
  pub fn clean_input_data(&mut self) {
    self.vcal_data.retain(|x|  !x.header.drs_lost_trigger()
        && !x.header.is_event_fragment()
        && x.trace_check()); 
    self.tcal_data.retain(|x|  !x.header.drs_lost_trigger()
        && !x.header.is_event_fragment()
        && x.trace_check()); 
    self.noi_data.retain(|x|   !x.header.drs_lost_trigger() 
        && !x.header.is_event_fragment()
        && x.trace_check()); 
    self.noi_data.sort_by(|a, b| a.header.event_id.cmp(&b.header.event_id));
    self.vcal_data.sort_by(|a, b| a.header.event_id.cmp(&b.header.event_id));
    self.tcal_data.sort_by(|a, b| a.header.event_id.cmp(&b.header.event_id));
  }

  // apply the vcal to a dataset of the calibration
  // (e.g. timing calibration)
  fn apply_vcal(&self, 
                data      : &Vec<RBEvent>)
      -> (Vec<Vec<Vec<f32>>>,Vec<isize>) {
    let nevents          = data.len();
    let mut traces       = Vec::<Vec::<Vec::<f32>>>::new();
    let mut trace        = Vec::<f32>::with_capacity(NWORDS);
    let mut stop_cells   = Vec::<isize>::new();
    let mut empty_events = Vec::<Vec::<f32>>::new();
    for _ in 0..nevents {
        empty_events.push(trace.clone());
    }
    for ch in 0..NCHN {
      traces.push(empty_events.clone());
      for (k,ev) in data.iter().enumerate() {
        trace.clear();
        stop_cells.push(ev.header.stop_cell as isize);
        for k in 0..NWORDS {
          trace.push(ev.adc[ch][k] as f32);
        }
        self.voltages(ch + 1, ev.header.stop_cell as usize,
                      &ev.adc[ch], &mut trace);
        traces[ch][k] = trace.clone();
      }
    }
    (traces, stop_cells)
  }

  // channel is from 0-8
  pub fn apply_vcal_constants(&self,
                              adc       : &Vec<f32>,
                              channel   : usize,  
                              stop_cell : usize) -> Vec<f32> {
    let mut waveform = Vec::<f32>::with_capacity(adc.len());
    let mut value : f32;
    for k in 0..adc.len() {
      value  = adc[k] as f32;
      value -= self.v_offsets[channel][(k + (stop_cell)) %NWORDS];
      value -= self.v_dips   [channel][k];
      value *= self.v_inc    [channel][(k + (stop_cell)) %NWORDS];
      waveform.push(value);
    } 
    waveform
  }

  /// Calculate the offset and dips calibration constants 
  /// for input data. 
  ///
  /// # Return:
  ///
  /// offsets, dips
  fn voltage_offset_and_dips(input_vcal_data : &Vec<RBEvent>) 
  -> Result<(Vec<Vec<f32>>, Vec<Vec<f32>>), CalibrationError> {
    if input_vcal_data.len() == 0 {
      return Err(CalibrationError::EmptyInputData);
    }
    let mut all_v_offsets = Vec::<Vec::<f32>>::new();
    let mut all_v_dips    = Vec::<Vec::<f32>>::new();
    let nchn = input_vcal_data[0].header.get_nchan();

    debug!("Found {nchn} channels!");
    for _ in 0..nchn {
      let empty_vec_off : Vec<f32> = vec![0.0;NWORDS];
      all_v_offsets.push(empty_vec_off);  
      let empty_vec_dip : Vec<f32> = vec![0.0;NWORDS];
      all_v_dips.push(empty_vec_dip);  
    }

    // we temporarily get the adc traces
    // traces are [channel][event][adc_cell]
    let mut traces        = unpack_traces(&input_vcal_data);
    let mut rolled_traces = traces.clone();
    for ch in 0..nchn {
      for n in 0..input_vcal_data.len() {
        for k in 0..Self::NSKIP {
          traces[ch][n][k] = f32::NAN;
          rolled_traces[ch][n][k] = f32::NAN;
        }
        // the traces are filled and the first 2 bins
        // marked with nan, now need to get "rolled over",
        // so that they start with the stop cell
        roll(&mut rolled_traces[ch][n],
             input_vcal_data[n].header.stop_cell as isize); 
      }// first loop over events done
      all_v_offsets[ch] = calculate_column_stat(&rolled_traces[ch], mean);
      //let v_offsets = calculate_column_medians(&rolled_traces[ch]);
      debug!("We calculated {} voltage offset values for ch {}", all_v_offsets[ch].len(), ch);
      // fill these in the prepared array structure
      //for k in 0..v_offsets.len() {
      //  all_v_offsets[ch][k] = v_offsets[k];
      //}
      for (n, ev) in input_vcal_data.iter().enumerate() {
        // now we roll the v_offsets back
        let mut v_offsets_rolled = all_v_offsets[ch].clone();
        roll(&mut v_offsets_rolled, -1*ev.header.stop_cell as isize);
        for k in 0..traces[ch][n].len() {
          traces[ch][n][k] -= v_offsets_rolled[k];
        }
      }
      let v_dips = calculate_column_stat(&traces[ch], median);
      for k in 0..v_dips.len() {
        if k < Self::NSKIP {
          all_v_dips[ch][k] = 0.0;
        } else {
          all_v_dips[ch][k] = v_dips[k];
        }
      }
    }
    Ok((all_v_offsets, all_v_dips))
  }


  /// Voltage calibration has to be applied
  /// 
  /// # Returns
  ///
  ///   vec\[ch\[9\], tbin\[1024\]\]
  ///
  pub fn timing_calibration(&self,
                            edge       : &Edge,
                            apply_vcal : bool) 
  -> Result<Vec<Vec<f32>>, CalibrationError> {
    if self.tcal_data.len() == 0 {
      error!("Input data for timing calibration is empty!");
      return Err(CalibrationError::EmptyInputData);
    }
    // tcal values are [channel][adc_cell] 
    let mut all_tcal = Vec::<Vec::<f32>>::new();
    for _ in 0..NCHN {
      all_tcal.push(Vec::<f32>::new());
    }
    // traces are [channel][event][adc_cell]
    let mut traces       : Vec<Vec<Vec<f32>>>;
    let mut stop_cells   = Vec::<isize>::new();
    if apply_vcal {
      let result = self.apply_vcal(&self.tcal_data);
      traces     = result.0;
      stop_cells = result.1;
    } else {
      warn!("Not applying voltage calibration to tcal data. This most likely makes no sense!");
      traces  = unpack_traces(&self.tcal_data);
      for ev in self.tcal_data.iter() {
        stop_cells.push(ev.header.event_id as isize);
      }
    }
    let do_spike_cleaning = true;
    if do_spike_cleaning {
      for k_ch in 0..traces.len() {
        for k_ev in 0..traces[k_ch].len() {
          clean_spikes(&mut traces[k_ch][k_ev], true);
        }
      }
    }
    let nwords = traces[0][0].len();
    for ch in 0..NCHN {
      for ev in 0..traces[ch].len() {
        for k in 0..nwords {
          if k < Self::NSKIP {
            traces[ch][ev][k]  = f32::NAN;
          }
          if f32::abs(traces[ch][ev][k]) > Self::SINMAX as f32 { 
            traces[ch][ev][k]  = f32::NAN;
          }// the traces are filled and the first 2 bins
        }
      }  
    }

    let mut rolled_traces = traces.clone();
    let mut drolled_traces = traces.clone();
    for ch in 0..NCHN {
      for ev in 0..traces[ch].len() {
        roll(&mut rolled_traces[ch][ev],
             stop_cells[ev]); 
      }
    }
    for ch in 0..NCHN {
      for ev in 0..traces[ch].len() {
        for n in 0..traces[ch][ev].len() {
          let mut dval : f32;
          if n == traces[ch][ev].len() - 1 {
            dval = rolled_traces[ch][ev][0] - rolled_traces[ch][ev][traces[ch][ev].len() -1];
          } else {
            dval = rolled_traces[ch][ev][n + 1] - rolled_traces[ch][ev][n];      
          }
          match edge {
            Edge::Rising | Edge::Average => {
              if dval < 0.0 {
                dval = f32::NAN;
              }
            },
            Edge::Falling => {
              if dval > 0.0 {
                dval = f32::NAN;
              }
            },
            _ => {
              // FIXME - better error handling
              error!("Only average, rising or falling edge supported!");
            }
          } // end match
          dval = f32::abs(dval); 
          if f32::abs(dval - 15.0) > Self::DVCUT {
            dval = f32::NAN;
          }
          drolled_traces[ch][ev][n] = dval;
        } // end loop over adc bins
      } // end loop over events
      let col_means = calculate_column_stat(&drolled_traces[ch], mean);
      let nfreq_vec : Vec<f32> = vec![1.0/Self::NOMINALFREQ;NWORDS];
      all_tcal[ch]  = nfreq_vec;
      let ch_mean   = mean(&col_means);
      for n in 0..all_tcal[ch].len() {
        all_tcal[ch][n] *= col_means[n]/ch_mean;  
      }
    } // end loop over channels
    Ok(all_tcal)
  }

  /// Call to the calibration routine, using
  /// the set input data
  pub fn calibrate(&mut self) -> Result<(), CalibrationError>{
    if self.vcal_data.len() == 0
    || self.tcal_data.len() == 0 
    || self.noi_data.len() == 0 {
      return Err(CalibrationError::EmptyInputData);
    }
    info!("Starting calculating voltage calibration constants!");
    let (v_offsets_high, _v_dips_high) 
        = Self::voltage_offset_and_dips(&self.vcal_data)?;
    let (v_offsets_low, v_dips_low) 
        = Self::voltage_offset_and_dips(&self.noi_data)?;
    // which of the v_offsets do we actually use?
    for ch in 0..NCHN {
      for k in 0..NWORDS {
        self.v_offsets[ch][k] = v_offsets_low[ch][k];
        self.v_dips[ch][k]    = v_dips_low[ch][k];
        self.v_inc[ch][k]     = self.d_v/(v_offsets_high[ch][k] - v_offsets_low[ch][k]);
      }
    }
    // at this point, the voltage calibration is complete
    info!("Filnished calculating voltage calibration constants!");
    info!("Starting calculating timing calibration constants!");
    warn!("Calibration only supported for Edge::Average!");
    // this just suppresses a warning
    // We have to think if edge will be
    // a parameter or a constant.
    let mut edge    = Edge::None;
    if matches!(edge, Edge::None) {
      edge = Edge::Average;
    }

    let mut tcal_av = self.timing_calibration( &edge, true)?;
    if matches!(edge, Edge::Average) {
      edge = Edge::Falling;
      let tcal_falling = self.timing_calibration(&edge, true)?;
      for ch in 0..NCHN {
        for k in 0..tcal_av.len() {
          tcal_av[ch][k] += tcal_falling[ch][k];
          tcal_av[ch][k] /= 2.0;
        }
      } 
      // for further calibration procedure
      edge = Edge::Rising;
    } else {
      error!("This is not implemented for any other case yet!");
      todo!();
    }
    
    // another set of constants
    //nevents,nchan,tracelen = gbf.traces.shape
    let mut damping   : f32 = 0.1;
    let corr_limit    : f32 = 0.05;
    //let n_iter_period : f32 = 1000; //#500 or nevents #

    let nperiod = Self::NOMINALFREQ/Self::CALFREQ; 
    let global = true;
    if global {
      
      //let mut tcal_traces   = Vec::<Vec::<Vec::<f32>>>::new();
      //let mut stop_cells    = Vec::<isize>::new();
      
      let result  = self.apply_vcal(&self.tcal_data);
      let tcal_traces = result.0;
      let stop_cells  = result.1;

      //for n in 0..1000 {
      for ch in 0..NCHN {
        for ev in 0..tcal_traces[ch].len() {
          let tracelen = tcal_traces[ch][ev].len();
          let stop_cell = stop_cells[ev];
          let mut tcal_av_cp = tcal_av.clone();
          roll(&mut tcal_av_cp[ch], -1* (stop_cell as isize));
          
          let (zcs, periods) = get_periods(&tcal_traces[ch][ev],
                                           &tcal_av_cp[ch],
                                           nperiod,
                                           Self::NSKIP as f32,
                                           &edge);
          debug!("Will iterate over {} periods!", periods.len());
          for (n_p,period) in periods.iter().enumerate() {
            if *period == 0.0 {
              warn!("period is 0 {:?}", periods);
            }
            if period.is_nan() {
              warn!("period is nan! {:?}", periods);
            }
            let zcs_a = zcs[n_p]     + stop_cell as usize;
            let zcs_b = zcs[n_p + 1] + stop_cell as usize;
            let mut corr = (1.0/Self::CALFREQ)/period;
            if matches!(edge, Edge::None) {
              corr *= 0.5;
            }
            if f32::abs(corr - 1.0) > corr_limit {
              continue;
            }
            corr = (corr-1.0)*damping + 1.0;
            let zcs_a_tl = zcs_a%tracelen;
            let zcs_b_tl = zcs_b%tracelen;
            if zcs_a < tracelen && zcs_b > tracelen {
              for m in zcs_a..tcal_av[ch].len() {
                tcal_av[ch][m] *= corr;
              }
              for m in 0..zcs_b_tl {
                tcal_av[ch][m] *= corr;
              }
            } else {
              for m in zcs_a_tl..zcs_b_tl {
                tcal_av[ch][m] *= corr;
              }
            }
          }
          //n_correct[ch] += 1.0;
        } // end over nchannel
        damping *= 0.99;
      } // end loop over n_iter_period
   
    } // end global
    for ch in 0..NCHN {
      for k in 0..NWORDS {
        self.tbin[ch][k] = tcal_av[ch][k];
      }
    }
    Ok(())
  }

  /// Apply the spike cleaning to all channels
  pub fn spike_cleaning(voltages  : &mut Vec<Vec<f32>>,
                        stop_cell : u16) 
    -> Result<(), WaveformError> {

    let mut spikes      : [i32;10] = [0;10];
    let mut filter      : f32;
    let mut dfilter     : f32;
    let mut n_neighbor  : usize;
    let mut n_rsp       = 0usize;
    let mut rsp : [i32;10]    = [-1;10];
    // to me, this seems that should be u32
    // the 10 is for a maximum of 10 spikes (Jeff)
    let mut sp   : [[usize;10];NCHN] = [[0;10];NCHN];
    let mut n_sp : [usize;10]        = [0;10];

    for j in 0..NWORDS as usize {
      for i in 0..NCHN as usize {
        filter = -voltages[i][j] + voltages[i][(j + 1) % NWORDS] + voltages[i][(j + 2) % NWORDS] - voltages[i][(j + 3) % NWORDS];
        dfilter = filter + 2.0 * voltages[i][(j + 3) % NWORDS] + voltages[i][(j + 4) % NWORDS] - voltages[i][(j + 5) % NWORDS];
        if filter > 20.0  && filter < 100.0 {
          if n_sp[i] < 10 {   // record maximum of 10 spikes
            sp[i][n_sp[i] as usize] = (j + 1) % NWORDS ;
            n_sp[i] += 1;
          } else {
            return Err(WaveformError::TooSpiky);
          }            // too many spikes -> something wrong
        }// end of if
        else if dfilter > 40.0 && dfilter < 100.0 && filter > 10.0 {
          if n_sp[i] < 9 {  // record maximum of 10 spikes
            sp[i][n_sp[i] as usize] = (j + 1) % NWORDS ;
            sp[i][(n_sp[i] + 1) as usize] = (j + 3) % NWORDS ;
            n_sp[i] += 2;
          } else {
            return Err(WaveformError::TooSpiky);
          } // too many spikes -> something wrong
        } // end of else if

      }// end loop over NCHN
    } // end loop over NWORDS

    // go through all spikes and look for neighbors */
    for i in 0..NCHN {
      for j in 0..n_sp[i] as usize {
        //n_symmetric = 0;
        n_neighbor = 0;
        for k in 0..NCHN {
          for l in 0..n_sp[k] as usize {
          //check if this spike has a symmetric partner in any channel
            if (sp[i][j] as i32 + sp[k][l] as i32 - 2 * stop_cell as i32) as i32 % NWORDS as i32 == 1022 {
              //n_symmetric += 1;
              break;
            }
          }
        } // end loop over k
        // check if this spike has same spike is in any other channels */
        //for (k = 0; k < nChn; k++) {
        for k in 0..NCHN {
          if i != k {
            for l in 0..n_sp[k] {
              if sp[i][j] == sp[k][l] {
              n_neighbor += 1;
              break;
              }
            } // end loop over l   
          } // end if
        } // end loop over k

        if n_neighbor >= 2 {
          for k in 0..n_rsp {
            if rsp[k] == sp[i][j] as i32 {break;} // ignore repeats
            if n_rsp < 10 && k == n_rsp {
              rsp[n_rsp] = sp[i][j] as i32;
              n_rsp += 1;
            }
          }  
        }

      } // end loop over j
    } // end loop over i

    // recognize spikes if at least one channel has it */
    //for (k = 0; k < n_rsp; k++)
    let magic_value : f32 = 14.8;
    let mut x       : f32;
    let mut y       : f32;

    let mut skip_next : bool = false;
    for k in 0..n_rsp {
      if skip_next {
        skip_next = false;
        continue;
      }
      spikes[k] = rsp[k];
      //for (i = 0; i < nChn; i++)
      for i in 0..NCHN {
        if k < n_rsp && i32::abs(rsp[k] as i32 - rsp[k + 1] as i32 % NWORDS as i32) == 2
        {
          // remove double spike 
          let j = if rsp[k] > rsp[k + 1] {rsp[k + 1] as usize}  else {rsp[k] as usize};
          x = voltages[i][(j - 1) % NWORDS];
          y = voltages[i][(j + 4) % NWORDS];
          if f32::abs(x - y) < 15.0 {
            voltages[i][j % NWORDS] = x + 1.0 * (y - x) / 5.0;
            voltages[i][(j + 1) % NWORDS] = x + 2.0 * (y - x) / 5.0;
            voltages[i][(j + 2) % NWORDS] = x + 3.0 * (y - x) / 5.0;
            voltages[i][(j + 3) % NWORDS] = x + 4.0 * (y - x) / 5.0;
          } else {
            voltages[i][j % NWORDS] -= magic_value;
            voltages[i][(j + 1) % NWORDS] -= magic_value;
            voltages[i][(j + 2) % NWORDS] -= magic_value;
            voltages[i][(j + 3) % NWORDS] -= magic_value;
          }
        } else {
          // remove single spike 
          x = voltages[i][((rsp[k] - 1) % NWORDS as i32) as usize];
          y = voltages[i][(rsp[k] + 2) as usize % NWORDS];
          if f32::abs(x - y) < 15.0 {
            voltages[i][rsp[k] as usize] = x + 1.0 * (y - x) / 3.0;
            voltages[i][(rsp[k] + 1) as usize % NWORDS] = x + 2.0 * (y - x) / 3.0;
          } else {
            voltages[i][rsp[k] as usize] -= magic_value;
            voltages[i][(rsp[k] + 1) as usize % NWORDS] -= magic_value;
          }
        } // end loop over nchn
      } // end loop over n_rsp
      if k < n_rsp && i32::abs(rsp[k] - rsp[k + 1] % NWORDS as i32) == 2
        {skip_next = true;} // skip second half of double spike
    } // end loop over k
  Ok(())
  }

  /// Apply the voltage calibration to a single channel 
  /// FIXME - mixing of naming conventions for the channels
  ///
  /// FIXME - make it return Result<(), CalibrationError>
  ///
  /// # Arguments
  ///
  /// * channel   : Channel id 1-9
  /// * stop_cell : This channels stop cell 
  /// * adc       : Uncalibrated channel data
  /// * waveform  : Pre-allocated array to hold 
  ///               calibrated waveform data.
  pub fn voltages(&self,
                  channel   : usize,
                  stop_cell : usize,
                  adc       : &Vec<u16>,
                  waveform  : &mut Vec<f32>) {
                  //waveform  : &mut [f32;NWORDS]) {
    if channel > 9 || channel == 0 {
      error!("There is no channel larger than 9 and no channel 0! Channel {channel} was requested. Can not perform voltage calibration!");
      return;
    }
    if adc.len() != waveform.len() {
      error!("Ch{} has {} adc values, however we are expecting {}!", channel,  adc.len(), waveform.len());
      return;
    }

    let mut value : f32; 
    for k in 0..NWORDS {
      value  = adc[k] as f32;
      value -= self.v_offsets[channel -1][(k + (stop_cell)) %NWORDS];
      value -= self.v_dips   [channel -1][k];
      value *= self.v_inc    [channel -1][(k + (stop_cell)) %NWORDS];
      waveform[k] = value;
    }
  }
  
  /// Apply the timing calibration to a single channel 
  /// 
  /// This will allocate the array for the waveform 
  /// time bins (unit is ns)
  ///
  /// # Arguments
  ///
  /// * channel   : Channel id 1-9
  /// * stop_cell : This channels stop cell 
  pub fn nanoseconds(&self,
                     channel   : usize,
                     stop_cell : usize,
                     times     : &mut Vec<f32>) {
    if channel > 9 || channel == 0 {
      error!("There is no channel larger than 9 and no channel 0! Channel {channel} was requested. Can not perform timing calibration!");
    }
    for k in 1..NWORDS { 
      times[k] = times[k-1] + self.tbin[channel -1][(k-1+(stop_cell))%NWORDS];
    }
  }

  pub fn new(rb_id : u8) -> Self {
    let timestamp = Utc::now().timestamp() as u32;
    Self {
      rb_id     : rb_id,
      d_v       : 182.0, // FIXME - this needs to be a constant
      timestamp : timestamp,
      serialize_event_data : false, // per default, don't serialize the data 
      v_offsets : [[0.0;NWORDS];NCHN], 
      v_dips    : [[0.0;NWORDS];NCHN], 
      v_inc     : [[0.0;NWORDS];NCHN], 
      tbin      : [[0.0;NWORDS];NCHN],
      vcal_data : Vec::<RBEvent>::new(),
      tcal_data : Vec::<RBEvent>::new(),
      noi_data  : Vec::<RBEvent>::new()
    }
  }

  /// Discard the data to reduce the memory footprint
  pub fn discard_data(&mut self) {
    self.vcal_data = Vec::<RBEvent>::new();
    self.tcal_data = Vec::<RBEvent>::new();
    self.noi_data  = Vec::<RBEvent>::new();
  }

  /// Gets the calibration from a file which 
  /// has the RBCalibration stored in a 
  /// TofPacket
  ///
  /// E.g. if it was written with TofPacketWriter
  pub fn from_file(filename : String, discard_data : bool) -> Result<Self, SerializationError> {
    let mut reader = TofPacketReader::new(&filename);
    loop {
      match reader.next() {
        None => {
          error!("Can't load calibration!");
          break;
        },
        Some(pack) => {
          if pack.packet_type == TofPacketType::RBCalibration { 
            let mut cali = RBCalibrations::from_bytestream(&pack.payload, &mut 0)?;
            if discard_data {
              cali.discard_data();
            }
            return Ok(cali);
          } else {
            continue;
          }
        }
      }
    }
    Err(SerializationError::StreamTooShort)
  }


  /// Infer the readoutboard id from the filename
  ///
  /// Assuming a certain naming scheme for the filename "rbXX_cal.txt"
  /// we extract the readoutboard id
  pub fn get_id_from_filename(&mut self, filename : &Path) -> u8 {
    let rb_id : u8;
    match filename.file_name() {
      None   => {
        error!("Path {} seems non-sensical!", filename.display());
        self.rb_id = 0;
        return 0;
      }
      Some(name) => {
        // TODO This might panic! Is it ok?
        let fname = name.to_os_string().into_string().unwrap();
        let id    = &fname[2..4];
        // TODO This might panic! Is it ok?
        rb_id     = id.parse::<u8>().unwrap();
        debug!("Extracted RB ID {} from filename {}", rb_id, fname);
      }
    }
  self.rb_id = rb_id;
  rb_id
  }
  
  /// Self check if the timing constants are sane 
  pub fn passes_timing_checks(&self) -> bool {
    for ch in 0..8 {
      let mut mean = 0.0;
      for k in 0..NWORDS {
        mean += self.tbin[ch][k];
      }
      mean /= NWORDS as f32;
      if mean < 0.48824 || mean > 0.48834 {
        error!("Timing calibration for ch {} / RB {} failed. Got mean of {}", ch + 1, self.rb_id, mean);
        return false;
      }
      let tbin_ch = self.tbin[ch].to_vec();
      let var = standard_deviation(&tbin_ch).unwrap_or(0.0);
      if var < 0.08 || var > 0.15 {
        error!("Timing calibration for ch {}/ RB {} failed. Got st dev of {}", ch + 1, self.rb_id, var);
        return false;
      }
    }
    debug!("Passed timing calibration sanity checks for RB {}!", self.rb_id);
    true
  }

  /// Self check if the voltage constants are sane
  pub fn passes_voltage_checks(&self) -> bool {
    for ch in 0..8 {
      let mut mean = 0.0;
      for k in 0..NWORDS {
        mean += self.v_offsets[ch][k];
      }
      mean /= NWORDS as f32;
      if mean < 4200.0 || mean > 5200.0 {
        error!("Voltage offset calibration for ch {} / RB {} failed. Got mean of {}", ch + 1, self.rb_id, mean);
        return false;
      }
      let v_off = self.tbin[ch].to_vec();
      let var = standard_deviation(&v_off).unwrap_or(0.0);
      if var > 150.0 {
        error!("Voltage offset calibration for ch {} / RB {} failed. Got st dev of {}", ch + 1, self.rb_id, var);
        return false;
      }
      mean = 0.0;
      for k in 0..NWORDS {
        mean += self.v_dips[ch][k];
      }
      mean /= NWORDS as f32;
      if mean < -0.5 || mean > 0.5 {
        error!("Voltage droop calibration for ch {} / RB {} failed. Got mean of {}", ch + 1, self.rb_id, mean);
        return false;
      }
      mean = 0.0;
      for k in 0..NWORDS {
        mean += self.v_inc[ch][k];
      }
      mean /= NWORDS as f32;
      if mean < 0.06 || mean > 0.07 {
        error!("Voltage gain calibration for ch {} / RB {} failed. Got mean of {}", ch + 1, self.rb_id, mean);
        return false;
      }
      let v_inc = self.v_inc[ch].to_vec();
      let var = standard_deviation(&v_inc).unwrap_or(0.0);
      if var > 0.00025 {
        error!("Voltage gain calibration for ch {} / RB {} failed. Got st dev of {}", ch + 1, self.rb_id, var);
        //return false;
      }
    }
    debug!("Passed voltage calibration sanity checks for RB {}!", self.rb_id);
    true 
  }


  /// Check voltage and timing constants for sanity 
  ///
  /// for each RB channel
  /// take distribution of 1024 calibration constants and confirm
  /// 
  /// Tcal:
  ///     average value between 0.48824 and 0.48834
  ///     standard deviation between 0.08 and 0.15.
  ///     Most common problem: avg of tcal distribution is < 0.48824
  /// 
  /// Vcal1 (offsets):
  ///     average value between 4200 and 5200
  ///     standard deviation less than 150
  /// 
  /// Vcal2 (droop):
  ///     average value between -0.5 and 0.5
  ///     print out a warning if the standard deviation for any channel is > 5.0
  ///     Note that ch1 on any particular board has a more dramatic droop so if ch1 droop is the only channel on the board with this warning, it can be ignored
  /// 
  /// Vcal3 (gain):
  ///     average value between 0.06 and 0.07
  ///     standard deviation less than 0.00025
  pub fn check(&self) -> bool {
    self.passes_timing_checks() && self.passes_voltage_checks() 
  }
}

impl TofPackable for RBCalibrations {
  const TOF_PACKET_TYPE : TofPacketType = TofPacketType::RBCalibration;
}

impl Serialization for RBCalibrations {
  const SIZE            : usize = NCHN*NWORDS*4*8 + 4 + 1 + 1; 
  const HEAD            : u16   = 0xAAAA; // 43690 
  const TAIL            : u16   = 0x5555; // 21845 
  
  fn from_bytestream(bytestream : &Vec<u8>, 
                     pos        : &mut usize)
    -> Result<Self, SerializationError> { 
    let mut rb_cal = Self::new(0);
    if parse_u16(bytestream, pos) != Self::HEAD {
      return Err(SerializationError::HeadInvalid {});
    }
    rb_cal.rb_id                = parse_u8(bytestream, pos);
    rb_cal.d_v                  = parse_f32(bytestream, pos);
    rb_cal.timestamp            = parse_u32(bytestream, pos);
    rb_cal.serialize_event_data = parse_bool(bytestream, pos);
    for ch in 0..NCHN {
      for k in 0..NWORDS {
        let mut value = parse_f32(bytestream, pos);
        rb_cal.v_offsets[ch][k] = value;
        value         = parse_f32(bytestream, pos);
        rb_cal.v_dips[ch][k]    = value;
        value         = parse_f32(bytestream, pos);
        rb_cal.v_inc[ch][k]     = value;
        value         = parse_f32(bytestream, pos);
        rb_cal.tbin[ch][k]      = value;
      }
    }
    if rb_cal.serialize_event_data {
      let broken_event = RBEvent::new();
      let n_noi  = parse_u16(bytestream, pos);
      debug!("Found {n_noi} no input data events!");
      for _ in 0..n_noi {
        match RBEvent::from_bytestream(bytestream, pos) {
          Ok(ev) => {
            rb_cal.noi_data.push(ev);            
          }
          Err(err) => {
            error!("Unable to read RBEvent! {err}");
          }
        }
        // FIXME - broken event won't advance the pos marker
      }
      let n_vcal = parse_u16(bytestream, pos); 
      debug!("Found {n_vcal} VCal data events!");
      for _ in 0..n_vcal {
        match RBEvent::from_bytestream(bytestream, pos) {
          Err(err) => {
            error!("Found broken event {err}");
          },
          Ok(good_ev) => {
            rb_cal.vcal_data.push(good_ev);
          }
        }
      }
      let n_tcal = parse_u16(bytestream, pos); 
      debug!("Found {n_tcal} TCal data events!");
      for _ in 0..n_tcal {
        rb_cal.tcal_data.push(RBEvent::from_bytestream(bytestream, pos).unwrap_or(broken_event.clone()));
      }
    } else {
      // we can skip the next 6 bytes, 
      // which just contain 0
      *pos += 6;
    }
    if parse_u16(bytestream, pos) != Self::TAIL {
      return Err(SerializationError::TailInvalid {});
    }
    Ok(rb_cal)
  }

  fn to_bytestream(&self) -> Vec<u8> {
    let mut bs = Vec::<u8>::with_capacity(Self::SIZE);
    bs.extend_from_slice(&Self::HEAD.to_le_bytes());
    bs.extend_from_slice(&self.rb_id.to_le_bytes());
    bs.extend_from_slice(&self.d_v.to_le_bytes());
    bs.extend_from_slice(&self.timestamp.to_le_bytes());
    let serialize_event_data = self.serialize_event_data as u8;
    bs.push(serialize_event_data);
    for ch in 0..NCHN {
      for k in 0..NWORDS {
        bs.extend_from_slice(&self.v_offsets[ch][k].to_le_bytes());
        bs.extend_from_slice(&self.v_dips[ch][k]   .to_le_bytes());
        bs.extend_from_slice(&self.v_inc[ch][k]    .to_le_bytes());
        bs.extend_from_slice(&self.tbin[ch][k]     .to_le_bytes());
      }
    }
    if self.serialize_event_data {
      info!("Serializing calibration event data!");
      let n_noi  = self.noi_data.len()  as u16;
      let n_vcal = self.vcal_data.len() as u16;
      let n_tcal = self.tcal_data.len() as u16;
      bs.extend_from_slice(&n_noi.to_le_bytes());
      for ev in &self.noi_data {
        bs.extend_from_slice(&ev.to_bytestream());
      }
      bs.extend_from_slice(&n_vcal.to_le_bytes());
      for ev in &self.vcal_data {
        bs.extend_from_slice(&ev.to_bytestream());
      }
      bs.extend_from_slice(&n_tcal.to_le_bytes());
      for ev in &self.tcal_data {
        bs.extend_from_slice(&ev.to_bytestream());
      }
    } else { // if we don't serialize event data, write 0 
             // for the empty data
      // (3 16bit 0s) for noi, vcal, tcal
      for _ in 0..6 {
        bs.push(0);
      }
      //bs.push(0); // noi data
      //bs.push(0); // vcal data
      //bs.push(0); // tcal data
    }
    bs.extend_from_slice(&Self::TAIL.to_le_bytes());
    bs
  }
}

impl Default for RBCalibrations {
  fn default() -> Self {
    Self::new(0)
  }
}

impl fmt::Display for RBCalibrations {
  fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
    let mut timestamp_str = String::from("?");
    match Utc.timestamp_opt(self.timestamp.into(), 0) {
      LocalResult::Single(datetime_utc) => {
        timestamp_str = datetime_utc.format("%Y/%m/%d %H:%M:%S").to_string();
      },
      LocalResult::Ambiguous(_, _) => {
        println!("The given timestamp is ambiguous.");
      },
      LocalResult::None => {
        println!("The given timestamp is not valid.");
      },
    }

    //let datetime_utc: DateTime<Utc> = Utc.timestamp_opt(self.timestamp as i64, 0).earliest().unwrap_or(DateTime::<Utc>::from_timestamp_millis(0).unwrap());
    if !self.check() {
      return write!(f, "<RBCalibrations [{} UTC] for board {}:
   -- fails self checks!>", timestamp_str, self.rb_id);
    }  
    write!(f, 
  "<RBCalibrations [{} UTC]:
      ** all self checks passed! ** 
      RB             : {}
      VCalData       : {} (events)
      TCalData       : {} (events)
      NoInputData    : {} (events)
      V Offsets [ch0]: .. {:?} {:?} ..
      V Incrmts [ch0]: .. {:?} {:?} ..
      V Dips    [ch0]: .. {:?} {:?} ..
      T Bins    [ch0]: .. {:?} {:?} ..>",
      timestamp_str,
      self.rb_id,
      self.vcal_data.len(),
      self.tcal_data.len(),
      self.noi_data.len(),
      self.v_offsets[0][98],
      self.v_offsets[0][99],
      self.v_inc[0][98],
      self.v_inc[0][99],
      self.v_dips[0][98],
      self.v_dips[0][99],
      self.tbin[0][98],
      self.tbin[0][99])
  } 
}

#[cfg(feature = "pybindings")] 
#[pymethods]
impl RBCalibrations {
  #[getter]
  fn rb_id(&self) -> u8 {
    self.rb_id
  }

  #[getter]
  fn d_v(&self) -> f32 {
    self.d_v
  }

  #[getter]
  fn vcal_data(&self) -> Vec<RBEvent> {
    self.vcal_data.clone()
  }
  
  #[getter]
  fn tcal_data(&self) -> Vec<RBEvent> {
    self.tcal_data.clone()
  }
  
  #[getter]
  fn noi_data(&self) -> Vec<RBEvent> {
    self.noi_data.clone()
  }
 
  #[getter]
  fn v_offsets<'_py>(&self, py: Python<'_py>) -> PyResult<Bound<'_py, PyArray2<f32>>> {  
    let mut data = Vec::<Vec<f32>>::with_capacity(9);
    for ch in 0..9 {
      data.push(self.v_offsets[ch].to_vec());
    }
    let pyarray = PyArray2::from_vec2(py, &data).unwrap();
    Ok(pyarray)
  }
  
  #[getter]
  fn v_dips<'_py>(&self, py: Python<'_py>) -> PyResult<Bound<'_py, PyArray2<f32>>> {  
    let mut data = Vec::<Vec<f32>>::with_capacity(9);
    for ch in 0..9 {
      data.push(self.v_dips[ch].to_vec());
    }
    let pyarray = PyArray2::from_vec2(py, &data).unwrap();
    Ok(pyarray)
  }
  
  #[getter]
  fn v_inc<'_py>(&self, py: Python<'_py>) -> PyResult<Bound<'_py, PyArray2<f32>>> {  
    let mut data = Vec::<Vec<f32>>::with_capacity(9);
    for ch in 0..9 {
      data.push(self.v_inc[ch].to_vec());
    }
    let pyarray = PyArray2::from_vec2(py, &data).unwrap();
    Ok(pyarray)
  }
  
  #[getter]
  fn tbin<'_py>(&self, py: Python<'_py>) -> PyResult<Bound<'_py, PyArray2<f32>>> {  
    let mut data = Vec::<Vec<f32>>::with_capacity(9);
    for ch in 0..9 {
      data.push(self.tbin[ch].to_vec());
    }
    let pyarray = PyArray2::from_vec2(py, &data).unwrap();
    Ok(pyarray)
  }
  
  /// Apply the voltage calibration to a single channel 
  /// FIXME - mixing of naming conventions for the channels
  ///
  /// FIXME - make it return Result<(), CalibrationError>
  ///
  /// # Arguments
  ///
  /// * channel   : Channel id 1-9
  /// * stop_cell : This channels stop cell 
  /// * adc       : Uncalibrated channel data
  /// * waveform  : Pre-allocated array to hold 
  ///               calibrated waveform data.
  #[pyo3(name="voltages")]
  pub fn voltages_py<'_py>(&self,
                           py        : Python<'_py>,
                           channel   : usize,
                           stop_cell : usize,
                           adc       : Bound<'_py, PyArray1<u16>>)
      -> PyResult<Bound<'_py, PyArray1<f32>>>{
                  //waveform  : &mut [f32;NWORDS]) {
    let mut voltages = vec![0.0f32; 1024];
    let adc_data = adc.to_vec().unwrap();
    self.voltages(channel, stop_cell, &adc_data, &mut voltages); 
    let pyarray = PyArray1::from_vec(py, voltages);
    Ok(pyarray)
  }
  
  /// Apply the timing calibration to a single channel 
  /// 
  /// This will allocate the array for the waveform 
  /// time bins (unit is ns)
  ///
  /// # Arguments
  ///
  /// * channel   : Channel id 1-9
  /// * stop_cell : This channels stop cell 
  #[pyo3(name="nanoseconds")]
  pub fn nanoseconds_py<'_py>(&self,
                              py        : Python<'_py>,                           
                              channel   : usize,
                              stop_cell : usize) 
      -> PyResult<Bound<'_py, PyArray1<f32>>> {
    let mut times = vec![0.0f32; 1024];
    self.nanoseconds(channel, stop_cell, &mut times); 
    let pyarray = PyArray1::from_vec(py, times);
    Ok(pyarray)
  }

  /// Load the calibration from a file with a 
  /// TofPacket of type RBCalibration in it
  ///
  /// # Arguments:
  ///
  /// * filename     : File with a TofPacket of type RBCalibration in it
  /// * discard_data : Throw away event data after loading
  #[pyo3(name = "from_file", signature = (filename, discard_data = true))]
  #[staticmethod]
  fn from_file_py(filename : String, discard_data : bool) -> PyResult<Self> {
    let cali = RBCalibrations::from_file(filename, discard_data);
    match cali {
      Ok(c) => {
        return Ok(c);
      },
      Err(err) => {
        return Err(PyValueError::new_err(err.to_string()));
      }
    }
  }

  #[pyo3(name="passes_voltage_checks")]
  fn passes_voltage_checks_py(&self) -> bool {
    self.passes_voltage_checks() 
  }

  #[pyo3(name="passes_timing_checks")]
  fn passes_timing_checks_py(&self) -> bool {
    self.passes_timing_checks()
  }

  #[pyo3(name="check")]
  fn check_py(&self) -> bool {
    self.check()
  }
}

pythonize_packable_no_new!(RBCalibrations);

#[cfg(feature = "random")]
impl FromRandom for RBCalibrations {
    
  fn from_random() -> Self {
    let mut cali   = Self::new(0);
    let mut rng    = rand::rng();
    cali.rb_id     = rng.random::<u8>();
    cali.d_v       = rng.random::<f32>(); 
    cali.serialize_event_data = rng.random::<bool>();
    for ch in 0..NCHN {
      for n in 0..NWORDS { 
        cali.v_offsets[ch][n] = rng.random::<f32>();
        cali.v_dips   [ch][n] = rng.random::<f32>(); 
        cali.v_inc    [ch][n] = rng.random::<f32>(); 
        cali.tbin     [ch][n] = rng.random::<f32>();
      }
    }
    if cali.serialize_event_data {
      for _ in 0..1000 {
        let mut ev = RBEvent::from_random();
        cali.vcal_data.push(ev);
        ev = RBEvent::from_random();
        cali.noi_data.push(ev);
        ev = RBEvent::from_random();
        cali.tcal_data.push(ev);
      }
    }
    cali
  }
}

//-----------------------------------------------

#[cfg(feature = "random")]
#[test]
fn serialization_rbcalibration_noeventpayload() {
  let mut calis = Vec::<RBCalibrations>::new();
  for _ in 0..100 {
    let cali = RBCalibrations::from_random();
    if cali.serialize_event_data {
      continue;
    }
    calis.push(cali);
    break;
  }
  let test = RBCalibrations::from_bytestream(&calis[0].to_bytestream(), &mut 0).unwrap();
  assert_eq!(calis[0], test);
}

#[cfg(feature = "random")]
#[test]
fn serialization_rbcalibration_witheventpayload() {
  let mut calis = Vec::<RBCalibrations>::new();
  for _ in 0..100 {
    let cali = RBCalibrations::from_random();
    if !cali.serialize_event_data {
      continue;
    }
    calis.push(cali);
    break;
  }
  let test = RBCalibrations::from_bytestream(&calis[0].to_bytestream(), &mut 0).unwrap();
  assert_eq!(calis[0], test);
}

#[cfg(feature = "random")]
#[test]
fn pack_rbcalibrationfilghtt() {
  for _ in 0..100 {
    let cfg  = RBCalibrationFlightT::from_random();
    let test : RBCalibrationFlightT = cfg.pack().unpack().unwrap();
    assert_eq!(cfg, test);
  }
}

#[cfg(feature = "random")]
#[test]
fn pack_rbcalibfilghtv() {
  for _ in 0..100 {
    let cfg  = RBCalibrationFlightV::from_random();
    let test : RBCalibrationFlightV = cfg.pack().unpack().unwrap();
    assert_eq!(cfg, test);
  }
}

#[cfg(feature = "random")]
#[test]
fn assemble_flightcal() {
  for _ in 0..10 {
    let cal  = RBCalibrations::from_random();
    let fct  = cal.emit_flighttcal();
    let fcv  = cal.emit_flightvcal();
    let test = RBCalibrations::assemble_from_flightcal(fct, fcv).unwrap();
    assert_eq!(cal.rb_id, test.rb_id);
    assert_eq!(cal.d_v  , test.d_v);
    assert_eq!(cal.timestamp, test.timestamp);
    assert_eq!(test.serialize_event_data, false);
    for ch in 0..NCHN {
      for k in 0..NWORDS {
        assert_eq!(f16::from_f32(cal.tbin[ch][k]),     f16::from_f32(test.tbin[ch][k])); 
        assert_eq!(f16::from_f32(cal.v_offsets[ch][k]),f16::from_f32(test.v_offsets[ch][k])); 
        assert_eq!(f16::from_f32(cal.v_dips[ch][k]),   f16::from_f32(test.v_dips[ch][k])); 
        assert_eq!(f16::from_f32(cal.v_inc[ch][k]),    f16::from_f32(test.v_inc[ch][k]));
      }
    }
  }
}


