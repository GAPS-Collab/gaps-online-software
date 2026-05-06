//! The following file is part of gaps-online-software and published 
//! under the GPLv3 license
//!
//! Calibration routines for the GAPS tracker system

use crate::prelude::*;

#[cfg_attr(feature="pybindings", pyclass)] 
pub struct TrackerOfflineCalibration {
  pub mask_map      : HashMap<u32,TrackerStripMask>, 
  pub tf_map        : HashMap<u32,TrackerStripTransferFunction>, 
  pub ped_map       : HashMap<u32,TrackerStripPedestal>,
  pub gain_map      : HashMap<u32,TrackerStripGain>,
  pub pulse_map     : HashMap<u32,TrackerStripPulse>,
  pub adc_sig_cut   : HashMap<u32,f32>,
  pub remove_cmn    : bool,
  pub ped_sigma_cut : f32,
  pub remove_pulsed : bool,
}

impl TrackerOfflineCalibration {

  pub fn new() -> Self {
    Self {
      mask_map      : HashMap::<u32, TrackerStripMask>::new(),
      tf_map        : HashMap::<u32, TrackerStripTransferFunction>::new(), 
      ped_map       : HashMap::<u32, TrackerStripPedestal>::new(),
      adc_sig_cut   : HashMap::<u32,f32>::new(),
      gain_map      : HashMap::<u32,TrackerStripGain>::new(),
      pulse_map     : HashMap::<u32,TrackerStripPulse>::new(),
      remove_cmn    : false,
      ped_sigma_cut : 0.0,
      remove_pulsed : false,
    }
  }

  /// Calculate the common noise level, according to the research of the GAPS tracker group
  ///
  /// General scheme is to find the pulsed channel on the strip, get the pulse value, 
  /// subtract the pedestal and subtract it from the hit adc (taking gain into account)
  pub fn get_common_noise(&self, hit : &TrackerHit, event_hits : &Vec<TrackerHit>) -> (f32, bool) {
    let mut cmn_level     = 0.0f32;
    // per default gains are 1 if they can not be looked up
    let strip_gain     : f32;
    let mut pulse_gain = 1.0f32; 
    let mut pulse_avg  = 0.0f32;
    let mut pulse_chn  = -1i32;
    let mut pulse_adc  = 0.0f32;
    let is_pulser      : bool;
    //let mut pulse_is_mean = false;
    if let Some(cmn_strip) = self.gain_map.get(&hit.get_stripid()) {
      strip_gain = cmn_strip.gain;
    } else {
      debug!("There is no entry for the strip gain for {}, default is 1.0", &hit.get_stripid());
      strip_gain = 1.0;
    }
    if let Some(pulse_chn_) = self.pulse_map.get(&hit.get_stripid()) { 
      pulse_chn  = pulse_chn_.pulse_chn;  
      pulse_avg  = pulse_chn_.pulse_avg;
      //pulse_is_mean = pulse_chn_.pulse_is_mean;
    }
    is_pulser = pulse_chn == hit.channel as i32;
    if pulse_chn < 0 || pulse_avg > 400.0 {
      return (0.0, is_pulser) 
    }
    let pulse_id = TrackerStrip::create_stripid(hit.layer, hit.row, hit.module, pulse_chn as u8);
    // now we need to find the hit in this event which is caused by the pulser 
    let mut p_hit  = TrackerHit::new();
    p_hit.layer    = hit.layer;
    p_hit.row      = hit.row;
    p_hit.module   = hit.module;
    p_hit.channel  = pulse_chn as u8;
    let mut found  = false;
    for h in event_hits {
      if h.get_stripid() == p_hit.get_stripid() {
        pulse_adc = h.adc as f32;
        found = true;
        break; // FIXME - there might be multiple hits
      }
    }
    if let Some(pulse_gain_) = self.gain_map.get(&pulse_id) {
      pulse_gain = pulse_gain_.gain; 
    } 
    if found {
      let adc_pulc_diff = pulse_adc - pulse_avg;
      if adc_pulc_diff < 250.0 && pulse_avg > 0.0 {
        cmn_level = adc_pulc_diff / pulse_gain;   
      }
    } 
    debug!("Calculated common level of {} gain {}", cmn_level, strip_gain);
    if cmn_level < 0.0 {
      //error!("CMN is negative! {}", cmn_level);
      //cmn_level = 0.0;
    }
    return (cmn_level * strip_gain, is_pulser);  
  }

  pub fn mask_hits(&self, event_hits : &mut Vec<TrackerHit>) {
    let mut active_strips = Vec::<u32>::new();
    for h in event_hits.iter() {
      if let Some(mask) = self.mask_map.get(&h.get_stripid()) {
        if mask.active {
          active_strips.push(h.get_stripid());
        }
      } else {
        warn!("No mask information for strip {}", h.get_stripid());
      }
    }
    event_hits.retain(|x| active_strips.contains(&x.get_stripid()));
  }

  pub fn calibrate_event(&self, event : &mut TelemetryEvent) -> Result<(),CalibrationError> {
    self.calibrate(&mut event.tracker_hits)?;
    Ok(())
  }

  /// The actual energy calibration of the individual tracker strips 
  ///
  /// This is a multistep process which includes 
  /// 1) Pedestal subtraction
  /// 2) Common noise reduction (if so desired) 
  /// 3) ADC -> Energy (Transfer functions) 
  /// 3) Deal with strips which had the pulser active 
  ///
  /// While this might not be the best way to do things, this 
  /// function matches the implementation in SimpleDet very 
  /// closely and produces the same results. 
  ///
  /// See also the compontents in the `database` related part 
  /// of this code.
  pub fn calibrate(&self, event_hits : &mut Vec<TrackerHit>) -> Result<(),CalibrationError> {
    let mut calibrated_hits = Vec::<TrackerHit>::with_capacity(event_hits.len());
    let mut c_hit : TrackerHit; //= TrackerHit::new();
    let mv_2_kev = 0.841f32;// mV to keV
    for hit in event_hits.iter() {
      let mut hit_ped = 0.0f32;
      let mut energy  = hit.adc as f32;
      let hit_tf  : Option<&TrackerStripTransferFunction>;
      if let Some(ped) = self.ped_map.get(&hit.get_stripid()) {
        hit_ped = ped.pedestal_mean;
        energy -= hit_ped;
        //if energy < self.ped_sigma_cut * ped.pedestal_sigma {
        //  // this basically means that the energy is within the given 
        //  // range of the pedestal. In this case, we set it basiccally to 0 
        //  //
        //  // Note - we don't raise the error, because this simply means this 
        //  // is a noisy strip!
        //  //return Ok(());i
        //  continue;
        //}
      } else {
        error!("No entry for {} in pedestal map", hit);
        //continue;
        //return Err(CalibrationError::NoPedestalAvailable); 
      }
      let mut hit_is_pulser = false;
      if self.remove_cmn {
        let cmn = self.get_common_noise(hit, event_hits); 
        energy -= cmn.0;
        hit_is_pulser = cmn.1;
      }
      // apply transfer functions 
      hit_tf = self.tf_map.get(&hit.get_stripid());
      if let Some(tf) = hit_tf {
        //println!("energy before : {}", energy);
        energy = tf.transfer_fn(energy);
        //println!("energy after : {}", energy);
      } else {
        error!("Trying to calibrate {}, but we don't have a transfer function for that!", hit.get_stripid());
        energy = 0.0;
        //return Err(CalibrationError::NoTransferFnAvailable);
      }
      // now we need to check if this is a pulsed channel 
      if hit_is_pulser {
        if let Some(pulse) = self.pulse_map.get(&hit.get_stripid()) {
          let mut p_avg = pulse.pulse_avg; 
          if p_avg > 0.0 {
            if p_avg > 400.0 && self.remove_pulsed {
              continue;
            }
            p_avg -= hit_ped;
            if let Some(tf) = hit_tf {
              p_avg = tf.transfer_fn(p_avg); 
              energy -= p_avg;
            }
          }
        } else {
          warn!("Hit is from the pulser, but we don't have any information about that strip in the cmn map!"); 
        }
      }
      c_hit = hit.clone();
      energy       *= mv_2_kev/1000.0; 
      c_hit.energy = energy;
      //println!("c_hit {}", c_hit);
      calibrated_hits.push(c_hit);
      //if energy < -0.13 {
      //  error!("Small energy!");
      //  error!("{}", c_hit);
      //}
    }
    event_hits.clear(); 
    *event_hits = calibrated_hits;
    Ok(())
  }
}

impl fmt::Display for TrackerOfflineCalibration {
  fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
    let mut repr = String::from("<TrackerOfflineCalibration:");
    repr += &(format!("\n  N masks  : {}", self.mask_map.len()));
    repr += &(format!("\n  N pedest : {}", self.ped_map.len()));
    repr += &(format!("\n  N tf_fns : {}", self.tf_map.len()));
    repr += &(format!("\n  N gains  : {}", self.gain_map.len()));
    repr += &(format!("\n  N pulses : {}", self.pulse_map.len()));
    repr += ">";
    write!(f, "{}", repr)
  }
}

#[cfg(feature="pybindings")] 
#[pymethods]
impl TrackerOfflineCalibration {
 
  #[pyo3(name="calibrate_hits")]
  fn calibrate_hits_py(&self, mut hits : Vec<TrackerHit>) -> Vec<TrackerHit> {
    let _ = self.calibrate(&mut hits);
    hits 
  }
   
  // all the getters/setters are here because I did not get
  // #[cfg_attr(feature="pybindings", pyo3(set,get)] 
  // to work. If we find out how to use that, get rid 
  // of all the getters/setters here

  #[getter]
  #[pyo3(name="remove_cmn")] 
  fn get_remove_cmn_py(&self) -> bool {
    self.remove_cmn
  }
  #[getter]
  #[pyo3(name="ped_sigma_cut")] 
  fn get_ped_sigma_cut_py(&self) -> f32 {
    self.ped_sigma_cut
  }

  #[getter]
  #[pyo3(name="remove_pulsed")] 
  fn get_remove_pulsed_py(&self) -> bool {
    self.remove_pulsed 
  }
  
  #[setter]
  #[pyo3(name="remove_cmn")] 
  fn set_remove_cmn_py(&mut self, value : bool) {
    self.remove_cmn = value;
  }

  #[setter]
  #[pyo3(name="ped_sigma_cut")] 
  fn set_ped_sigma_cut_py(&mut self, value : f32) {
    self.ped_sigma_cut = value;
  }

  #[setter]
  #[pyo3(name="remove_pulsed")] 
  fn set_remove_pulsed_py(&mut self, value : bool) {
    self.remove_pulsed = value; 
  }

  #[getter]
  #[pyo3(name="mask_map")]
  fn get_mask_map_py(&self) -> HashMap<u32,TrackerStripMask> {
    self.mask_map.clone()
  }
  #[setter]
  #[pyo3(name="mask_map")]
  fn set_mask_map_py(&mut self, value : HashMap<u32,TrackerStripMask>) {
    self.mask_map = value;
  }
  
  #[getter]
  #[pyo3(name="tf_map")]
  fn get_tf_map_py(&self) -> HashMap<u32,TrackerStripTransferFunction> {
    self.tf_map.clone()
  }
  #[setter]
  #[pyo3(name="tf_map")]
  fn set_tf_map_py(&mut self, value : HashMap<u32, TrackerStripTransferFunction>) {
    self.tf_map = value;
  }

  #[getter]
  #[pyo3(name="ped_map")]
  fn get_ped_map_py(&self) -> HashMap<u32,TrackerStripPedestal> {
    self.ped_map.clone()
  }
  #[setter] 
  #[pyo3(name="ped_map")]
  fn set_ped_map_py(&mut self, value : HashMap<u32,TrackerStripPedestal>) {
    self.ped_map = value;
  }

  #[getter]
  #[pyo3(name="pulse_map")]
  fn get_pulse_map_py(&self) -> HashMap<u32, TrackerStripPulse> {
    self.pulse_map.clone()
  }
  
  #[setter]
  #[pyo3(name="pulse_map")]
  fn set_pulse_map_py(&mut self, value : HashMap<u32,TrackerStripPulse>) {
    self.pulse_map = value;
  }
  
  #[getter]
  #[pyo3(name="gain_map")]
  fn get_gain_map_py(&self) -> HashMap<u32, TrackerStripGain> {
    self.gain_map.clone()
  }
  #[setter]
  #[pyo3(name="gain_map")]
  fn set_gain_map_py(&mut self, value : HashMap<u32,TrackerStripGain>) {
    self.gain_map = value;
  }

  #[getter]
  #[pyo3(name="adc_sig_cut")]
  fn get_adc_sig_cut_py(&self) -> HashMap<u32,f32> {
    self.adc_sig_cut.clone()
  }
  #[setter]
  #[pyo3(name="adc_sig_cut")]
  fn set_adc_sig_cut_py(&mut self, value : HashMap<u32,f32>) {
    self.adc_sig_cut = value;
  }
}

#[cfg(feature="pybindings")]
pythonize!(TrackerOfflineCalibration);

