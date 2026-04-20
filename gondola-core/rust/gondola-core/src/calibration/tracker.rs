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
  //pub cmn_map       : HashMap<u32,TrackerStripCmnNoise>,
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

  pub fn get_common_noise(&self, hit : &TrackerHit, event_hits : &Vec<TrackerHit>) -> (f32, bool) {
    let mut cmn_level     = 0.0f32;
    let mut strip_gain    = 0.0f32;
    let mut hit_is_pulser = false;
    if let Some(cmn_strip) = self.gain_map.get(&hit.get_stripid()) {
      strip_gain = cmn_strip.gain;
    } 
    if let Some(cmn_strip) = self.pulse_map.get(&hit.get_stripid()) {
      // get the pulsed channel and the adc  for it 
      let pulse_ch = cmn_strip.pulse_chn; 
      let mut pulse_adc  : f32;// = 0.0;
      //let mut pulse_ped  : u16;// = 0;
      let mut pulse_gain : f32 = 1.0;
      let pulse_strip_id : u32 = TrackerStrip::create_stripid(hit.layer as u8, hit.row as u8,
          hit.module as u8, pulse_ch as u8); 
      hit_is_pulser = hit.get_stripid() == pulse_strip_id;
      if let Some(cmn_pulsed_strip) = self.gain_map.get(&pulse_strip_id) {
        pulse_gain = cmn_pulsed_strip.gain;
      }        
      for h in event_hits {
        if h.get_stripid() == pulse_strip_id {
          // we have now found the hit of the pulsed channel
          pulse_adc = h.adc as f32; 
          if let Some(pulse_ped) = self.ped_map.get(&h.get_stripid()) {
            pulse_adc -= pulse_ped.pedestal_mean;
            let adc_pulc_diff = pulse_adc as f32 - cmn_strip.pulse_avg;
            if adc_pulc_diff < 250.0 && cmn_strip.pulse_avg > 0.0 {
              // get the gain for the pulsed channel by selecting the cmn_noise 
              // not for the actual strip but for the pulsed strip instead 
              cmn_level = adc_pulc_diff / pulse_gain;
            } else {
              warn!("Can not get pedestal for pulsed channel!");
            }
          }
        }
      }
    }
    return (cmn_level*strip_gain, hit_is_pulser);
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

  pub fn calibrate(&self, event_hits : &mut Vec<TrackerHit>) -> Result<(),CalibrationError> {
    let mut calibrated_hits = Vec::<TrackerHit>::with_capacity(event_hits.len());
    let mut c_hit : TrackerHit; //= TrackerHit::new();
    for hit in event_hits.iter() {
      let hit_ped : f32; //= 0.0f32;
      let mut energy  = hit.adc as f32;
      let hit_tf  : Option<&TrackerStripTransferFunction>;
      if let Some(ped) = self.ped_map.get(&hit.get_stripid()) {
        hit_ped = ped.pedestal_mean;
        energy -= hit_ped;
        if energy < self.ped_sigma_cut * ped.pedestal_sigma {
          // this basically means that the energy is within the given 
          // range of the pedestal. In this case, we set it basiccally to 0 
          //
          // Note - we don't raise the error, because this simply means this 
          // is a noisy strip!
          //return Ok(());i
          continue;
        }
      } else {
        return Err(CalibrationError::NoStripMaskAvailable); 
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
        energy = tf.transfer_fn(energy);
      } else {
        return Err(CalibrationError::NoTransferFnAvailable);
      }
      // now we need to check if this is a pulsed channel 
      if hit_is_pulser {
        if let Some(pulse) = self.pulse_map.get(&hit.get_stripid()) {
          let mut p_avg = pulse.pulse_avg; 
          if p_avg > 0.0 && self.remove_pulsed {
            continue
          }
          p_avg -= hit_ped;
          if let Some(tf) = hit_tf {
            p_avg = tf.transfer_fn(p_avg); 
            energy -= p_avg;
          }
        } else {
          warn!("Hit is from the pulser, but we don't have any information about that strip in the cmn map!"); 
        }
      }
      c_hit = hit.clone();
      c_hit.energy = energy;
      calibrated_hits.push(c_hit);
    }
    event_hits.clear(); 
    *event_hits = calibrated_hits;
    Ok(())
    //  signal = adc - GetPED(layer, row, module, channel);
    //  //FIX maybe this cut should be moved after CMN sub
    //  if (signal < sig_cut_ * GetSIG(layer, row, module, channel))
    //    continue;
    //   }
    ////---------------------------------------------------------
    //// LEVEL 1 -- substract CMN
    ////---------------------------------------------------------
    //if (CMN_option_ ==true) {
    //  signal = signal - GetCMN(raw, layer, row, module, channel);
    //}
    ////---------------------------------------------------------
    //// LEVEL 2 -- convert to energy
    ////---------------------------------------------------------
    //if (level_ > 1) {
    //  // if (!tf_is_set_)
    //  //   SetTransferFunctions("");
    //  signal = TrackerEnergyResponseFunction(signal, layer, row, module, channel);
    //  //-------------------------------------------------------------
    //  // PULSED CHANNEL
    //  //-------------------------------------------------------------
    //  int pulch = GetPulChs(layer, row, module, channel);
    //  if(channel==pulch){ //this is a pulsed channel
    //    // if(mask_pulses_ )continue; // exclude pulsed channel       
    //    //
    //    // energy conversion
    //    //
    //    double adc_pulch_ave = GetPulave(layer, row, module, pulch);
    //    if(adc_pulch_ave>0.){
    //      if(mask_pulses_ && adc_pulch_ave>400)continue; // exclude pulsed channel
    //      double signal_pulch = adc_pulch_ave - GetPED(layer, row, module, pulch);
    //      signal_pulch = TrackerEnergyResponseFunction(signal_pulch, layer, row, module, channel);
    //      signal = signal - signal_pulch;
    //    }
    //  }
    //  if(signal < mev_cut_)continue; // energy cut
    //}
  }

  //pub fn get_energy(&self, strip_id : u32, adc : u16) -> f32 {
  //  let energy = 0.0f32;
  //  energy
  //}
}

#[cfg(feature="pybindings")] 
#[pymethods]
impl TrackerOfflineCalibration {
 
  #[new]
  fn new_py() -> Self {
    Self::new()
  }

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

