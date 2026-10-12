// This file is part of gaps-online-software and published 
// under the GPLv3 license
//! TOF pulse template.
//!
//! A `TofPulseTemplate` contains the mean and RMS waveform for a
//! particular TOF paddle, paddle side, position interval, and charge
//! interval.
//!
//! Position is measured from paddle side A in mm.
//!
//! The template contains 108 regularly spaced waveform samples. The
//! corresponding time axis is described by `time_start` and
//! `time_step`.
//!
//! The type implements Gondola's `Serialization` and `TofPackable`
//! traits and can therefore be stored in a `TofPacket`.

use std::fmt;

use crate::prelude::*;

// ============================================================================
// Constants
// ============================================================================

/// Number of samples in a TOF pulse template.
pub const N_PULSE_TEMPLATE_BINS: usize = 108;

/// Side A identifier.
pub const PULSE_TEMPLATE_SIDE_A: u8 = 0;

/// Side B identifier.
pub const PULSE_TEMPLATE_SIDE_B: u8 = 1;

/// Average waveform template for a TOF paddle.
///
/// Each template corresponds to:
///
/// * one paddle,
/// * one paddle side,
/// * one position interval,
/// * one charge interval.
///
/// Position is measured from paddle side A in mm.
///
/// `side` uses the convention:
///
/// * `0` = side A
/// * `1` = side B
///
/// The waveform time axis is regular and is given by
///
/// ```text
/// time[i] = time_start + i * time_step
/// ```
#[derive(Debug, Clone, PartialEq)]
#[cfg_attr(feature = "pybindings", pyclass)]
pub struct TofPulseTemplate {
  /// Paddle ID.
  pub paddle_id: u8,
  /// Paddle side:
  ///
  /// * 0 = A
  /// * 1 = B
  pub side: u8,
  /// Number of waveforms which went into this template 
  pub n_waveforms : u32,
  /// Lower edge of the position interval, measured from side A [mm].
  pub position_low: f32,
  /// Upper edge of the position interval, measured from side A [mm].
  pub position_high: f32,
  /// Lower edge of the charge interval [pC].
  pub charge_low: f32,
  /// Upper edge of the charge interval [pC].
  pub charge_high: f32,
  /// Time of the first waveform sample [ns].
  pub time_start: f32,
  /// Time spacing between waveform samples [ns].
  pub time_step: f32,
  /// Mean waveform [mV].
  pub mean: [f32; N_PULSE_TEMPLATE_BINS],
  /// RMS waveform [mV].
  pub rms: [f32; N_PULSE_TEMPLATE_BINS],
}


impl TofPulseTemplate {
  /// Create an empty TOF pulse template.
  pub fn new() -> Self {
    Self {
      paddle_id    : 0,
      side         : PULSE_TEMPLATE_SIDE_A,
      n_waveforms  : 0,
      position_low : 0.0,
      position_high: 0.0,
      charge_low   : 0.0,
      charge_high  : 0.0,
      time_start   : 0.0,
      time_step    : 1.0,
      mean         : [0.0; N_PULSE_TEMPLATE_BINS],
      rms          : [0.0; N_PULSE_TEMPLATE_BINS],
    }
  }

  /// Return the time coordinate for a waveform sample.
  pub fn time_at(&self, index: usize) -> Option<f32> {
      if index >= N_PULSE_TEMPLATE_BINS {
        return None;
      }
      Some(
        self.time_start  + index as f32 * self.time_step
      )
  }

  /// Return the complete waveform time axis.
  pub fn time_axis(&self) -> Vec<f32> {
    (0..N_PULSE_TEMPLATE_BINS)
      .map(|index| {
        self.time_start + index as f32 * self.time_step
      })
      .collect()
  }

  /// Return `true` if this template describes paddle side A.
  pub fn is_side_a(&self) -> bool {
    self.side == PULSE_TEMPLATE_SIDE_A
  }

  /// Return `true` if this template describes paddle side B.
  pub fn is_side_b(&self) -> bool {
    self.side == PULSE_TEMPLATE_SIDE_B
  }
}


impl Default for TofPulseTemplate {
  fn default() -> Self {
    Self::new()
  }
}


// ============================================================================
// Display
// ============================================================================

impl fmt::Display for TofPulseTemplate {
  fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
    let side = match self.side {
      PULSE_TEMPLATE_SIDE_A => "A",
      PULSE_TEMPLATE_SIDE_B => "B",
      _ => "?",
    };
    let time_end = self.time_start
      + (N_PULSE_TEMPLATE_BINS - 1) as f32
          * self.time_step;
    let mean_min = self
      .mean
      .iter()
      .copied()
      .filter(|value| value.is_finite())
      .fold(f32::INFINITY, f32::min);
    let mean_max = self
      .mean
      .iter()
      .copied()
      .filter(|value| value.is_finite())
      .fold(f32::NEG_INFINITY, f32::max);
    let rms_min = self
      .rms
      .iter()
      .copied()
      .filter(|value| value.is_finite())
      .fold(f32::INFINITY, f32::min);
    let rms_max = self
      .rms
      .iter()
      .copied()
      .filter(|value| value.is_finite())
      .fold(f32::NEG_INFINITY, f32::max);
    write!(
      f,
      concat!(
        "<TofPulseTemplate {{ ",
        "paddle: {}, ",
        "side: {}, ",
        "n wf: {}, ",
        "position: [{:.1}, {:.1}) mm, ",
        "charge: [{:.1}, {:.1}) pC, ",
        "time: [{:.1}, {:.1}] ns (dt={:.2} ns), ",
        "mean: [{:.2}, {:.2}] mV, ",
        "rms: [{:.2}, {:.2}] mV ",
        "}}>"
      ),
      self.paddle_id,
      side,
      self.n_waveforms,
      self.position_low,
      self.position_high,
      self.charge_low,
      self.charge_high,
      self.time_start,
      time_end,
      self.time_step,
      mean_min,
      mean_max,
      rms_min,
      rms_max,
    )
  }
}


// ============================================================================
// Gondola serialization
// ============================================================================

impl Serialization for TofPulseTemplate {
  const HEAD: u16 = 0xAAAA;
  const TAIL: u16 = 0x5555;

  const SIZE: usize =
    2                                      // HEAD
    + 1                                    // paddle_id
    + 1                                    // side
    + 4                                    // n waveforms
    + 6 * std::mem::size_of::<f32>()       // scalar f32 metadata
    + N_PULSE_TEMPLATE_BINS
        * std::mem::size_of::<f32>()       // mean
    + N_PULSE_TEMPLATE_BINS
        * std::mem::size_of::<f32>()       // rms
    + 2;                                   // TAIL

  fn to_bytestream(&self) -> Vec<u8> {
    let mut stream = Vec::<u8>::with_capacity(Self::SIZE);
    // HEAD
    stream.extend_from_slice(&Self::HEAD.to_le_bytes());
    // Template identification
    stream.push(self.paddle_id);
    stream.push(self.side);
    stream.extend_from_slice(&self.n_waveforms.to_le_bytes());
    // Position interval
    stream.extend_from_slice(&self.position_low.to_le_bytes());
    stream.extend_from_slice(&self.position_high.to_le_bytes());
    // Charge interval
    stream.extend_from_slice(&self.charge_low.to_le_bytes());
    stream.extend_from_slice(&self.charge_high.to_le_bytes());
    // Time axis
    stream.extend_from_slice(&self.time_start.to_le_bytes());
    stream.extend_from_slice(&self.time_step.to_le_bytes());
    // Mean waveform
    for value in &self.mean {
      stream.extend_from_slice(&value.to_le_bytes() );
    }
    // RMS waveform
    for value in &self.rms {
      stream.extend_from_slice(&value.to_le_bytes());
    }
    stream.extend_from_slice(&Self::TAIL.to_le_bytes());
    stream
  }

  fn from_bytestream(stream: &Vec<u8>,pos: &mut usize,) 
    -> Result<Self, SerializationError> {

    // verify_fixed() checks HEAD and advances pos past it.
    Self::verify_fixed(stream, pos)?;
    let mut temp       = Self::new();
    temp.paddle_id     = parse_u8(stream, pos);
    temp.side          = parse_u8(stream, pos);
    temp.n_waveforms   = parse_u32(stream, pos);
    temp.position_low  = parse_f32(stream, pos);
    temp.position_high = parse_f32(stream, pos);
    temp.charge_low    = parse_f32(stream, pos);
    temp.charge_high   = parse_f32(stream, pos);
    temp.time_start    = parse_f32(stream, pos);
    temp.time_step     = parse_f32(stream, pos);
    let mut mean       = [0.0_f32; N_PULSE_TEMPLATE_BINS];
    let mut rms        = [0.0_f32; N_PULSE_TEMPLATE_BINS];
    for value in &mut mean {
      *value           = parse_f32(stream, pos);
    }
    for value in &mut rms {
      *value           = parse_f32(stream, pos);
    }
    let tail =        parse_u16(stream, pos);
    if tail != Self::TAIL {
      error!("Got wrong tail bytes! Found {} instead of {}", tail, Self::TAIL);
      return Err(SerializationError::TailInvalid);
    }
    temp.mean = mean;
    temp.rms  = rms;
    Ok(temp)
  }
}


impl TofPackable for TofPulseTemplate {
  const TOF_PACKET_TYPE: TofPacketType = TofPacketType::TofPulseTemplate;
}

#[cfg(feature="random")]
impl FromRandom for TofPulseTemplate {

  fn from_random() -> Self {
    let mut template            = Self::new();
    let mut rng                 = rand::rng();
    template.paddle_id          = rng.random::<u8>();
    template.side               = rng.random::<u8>();
    template.n_waveforms        = rng.random::<u32>();
    template.position_low       = rng.random::<f32>();
    template.position_high      = rng.random::<f32>();
    template.charge_low         = rng.random::<f32>();
    template.charge_high        = rng.random::<f32>();
    template.time_start         = rng.random::<f32>();
    template.time_step          = rng.random::<f32>();
    for k in 0..N_PULSE_TEMPLATE_BINS {
      template.mean[k] = rng.random::<f32>();
      template.rms [k] = rng.random::<f32>();
    }
    template
  }
}

#[cfg(feature = "pybindings")]
#[pymethods]
impl TofPulseTemplate {
  //#[new]
  //fn new_py() -> Self {
  //    Self::new()
  //}

  #[getter]
  fn paddle_id(&self) -> u8 {
    self.paddle_id
  }

  #[setter]
  fn set_paddle_id(&mut self, value: u8) {
    self.paddle_id = value;
  }

  #[getter]
  #[pyo3(name="n_waveforms")]
  fn get_n_waveforms_py(&self) -> u32 {
    self.n_waveforms
  }

  #[setter]
  #[pyo3(name="n_waveforms")]
  fn set_n_waveforms_py(&mut self, value: u32) {
    self.n_waveforms = value;
  }

  // ------------------------------------------------------------------------
  // Side
  // ------------------------------------------------------------------------

  #[getter]
  fn side(&self) -> u8 {
      self.side
  }

  #[setter]
  fn set_side(&mut self, value: u8) -> PyResult<()> {
      if value != PULSE_TEMPLATE_SIDE_A
          && value != PULSE_TEMPLATE_SIDE_B
      {
          return Err(
              pyo3::exceptions::PyValueError::new_err(
                  "side must be 0 (A) or 1 (B)"
              )
          );
      }
      self.side = value;
      Ok(())
  }

  #[getter]
  fn position_low(&self) -> f32 {
    self.position_low
  }

  #[setter]
  fn set_position_low(&mut self, value: f32) {
    self.position_low = value;
  }

  #[getter]
  fn position_high(&self) -> f32 {
    self.position_high
  }

  #[setter]
  fn set_position_high(&mut self, value: f32) {
    self.position_high = value;
  }

  #[getter]
  fn charge_low(&self) -> f32 {
    self.charge_low
  }

  #[setter]
  fn set_charge_low(&mut self, value: f32) {
    self.charge_low = value;
  }

  #[getter]
  fn charge_high(&self) -> f32 {
    self.charge_high
  }

  #[setter]
  fn set_charge_high(&mut self, value: f32) {
    self.charge_high = value;
  }

  #[getter]
  fn time_start(&self) -> f32 {
    self.time_start
  }

  #[setter]
  fn set_time_start(&mut self, value: f32) {
      self.time_start = value;
  }

  #[getter]
  fn time_step(&self) -> f32 {
      self.time_step
  }

  #[setter]
  fn set_time_step(&mut self, value: f32) {
      self.time_step = value;
  }

  /// Return the complete time axis.
  #[pyo3(name = "time_axis")]
  fn time_axis_py(&self) -> Vec<f32> {
      self.time_axis()
  }


  // ------------------------------------------------------------------------
  // Mean waveform
  // ------------------------------------------------------------------------

  #[getter]
  fn mean(&self) -> Vec<f32> {
      self.mean.to_vec()
  }

  #[setter]
  fn set_mean(
      &mut self,
      values: Vec<f32>,
  ) -> PyResult<()> {

      if values.len()
          != N_PULSE_TEMPLATE_BINS
      {
          return Err(
              pyo3::exceptions::PyValueError::new_err(
                  format!(
                      "mean must contain exactly {} values",
                      N_PULSE_TEMPLATE_BINS
                  )
              )
          );
      }

      self.mean.copy_from_slice(&values);
      Ok(())
  }


  // ------------------------------------------------------------------------
  // RMS waveform
  // ------------------------------------------------------------------------

  #[getter]
  fn rms(&self) -> Vec<f32> {
    self.rms.to_vec()
  }

  #[setter]
  fn set_rms(
    &mut self,
    values: Vec<f32>,
  ) -> PyResult<()> {

    if values.len()
        != N_PULSE_TEMPLATE_BINS
    {
        return Err(
            pyo3::exceptions::PyValueError::new_err(
                format!(
                    "rms must contain exactly {} values",
                    N_PULSE_TEMPLATE_BINS
                )
            )
        );
    }

    self.rms.copy_from_slice(
        &values
    );

    Ok(())
  }

  #[pyo3(name = "is_side_a")]
  fn is_side_a_py(&self) -> bool {
    self.is_side_a()
  }

  #[pyo3(name = "is_side_b")]
  fn is_side_b_py(&self) -> bool {
    self.is_side_b()
  }

  //fn __repr__(&self) -> String {
  //    format!("{}", self)
  //}

  //fn __str__(&self) -> String {
  //    format!("{}", self)
  //}
}

#[cfg(feature = "pybindings")]
pythonize_packable!(TofPulseTemplate);


// ============================================================================
// Tests
// ============================================================================

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn packable_tofpulsetemplate() {
    for _ in 0..500 {
      let mut data = TofPulseTemplate::from_random();
      let mut test : TofPulseTemplate = data.pack().unpack().unwrap();
      assert_eq!(data, test);
    }
    let template = TofPulseTemplate::from_random();
    let stream = template.to_bytestream();
    assert_eq!(stream.len(), TofPulseTemplate::SIZE);

    let mut pos = 0;
    let restored = TofPulseTemplate::from_bytestream(&stream, &mut pos).unwrap();
    assert_eq!(template, restored);
    assert_eq!(pos, TofPulseTemplate::SIZE);
  }


  #[test]
  fn time_axis_is_correct() {
    let mut template =
        TofPulseTemplate::new();

    template.time_start = -7.5;
    template.time_step = 1.0;

    assert_eq!(
        template.time_at(0),
        Some(-7.5)
    );

    assert_eq!(
        template.time_at(1),
        Some(-6.5)
    );

    assert_eq!(
        template.time_at(107),
        Some(99.5)
    );

    assert_eq!(
        template.time_at(108),
        None
    );
  }


  #[test]
  fn display_works() {
    let mut template = TofPulseTemplate::new();
    template.paddle_id = 42;
    template.side = PULSE_TEMPLATE_SIDE_A;
    template.position_low = 250.0;
    template.position_high = 350.0;
    template.charge_low = 72.0;
    template.charge_high = 92.0;
    template.time_start = -7.5;
    template.time_step = 1.0;
    let output = format!("{}", template);
    
    assert!(output.contains("paddle: 42"));
    assert!(output.contains("side: A"));
  }
}
