//! The TelemetryEvent or former "MergedEvent" is that what gets 
//! sent over telemetry during flight
// The following file is part of gaps-online-software and published 
// under the GPLv3 license

use crate::prelude::*;

#[cfg_attr(feature="pybindings", pyclass)]
pub struct TelemetryEvent {
  pub header              : TelemetryPacketHeader,
  pub creation_time       : u64,
  pub event_id            : u32,
  pub tracker_hits        : Vec<TrackerHit>,
  pub tracker_oscillators : Vec<u64>,
  pub tof_event           : TofEvent,
  pub raw_data            : Vec<u8>,
  pub flags0              : u8,
  pub flags1              : u8,
  pub version             : u8
}

impl TelemetryEvent {

  pub fn new() -> Self {
    let mut tracker_oscillators = Vec::<u64>::new();
    for _ in 0..10 {
      tracker_oscillators.push(0);
    }
    Self {
      header              : TelemetryPacketHeader::new(),
      creation_time       : 0,
      event_id            : 0,
      tracker_hits        : Vec::<TrackerHit>::new(),
      tracker_oscillators : tracker_oscillators,
      tof_event           : TofEvent::new(),
      raw_data            : Vec::<u8>::new(),
      flags0              : 0,
      flags1              : 1,
      version             : 0, 
    }
  }

  /// Restore position information from database
  #[cfg(feature="database")]
  pub fn dehydrate(&mut self, tof_paddles : &HashMap<u8,TofPaddle>, trk_strips : &HashMap<u32, TrackerStrip>) {
    self.tof_event.set_paddles(tof_paddles);
    for h in &mut self.tracker_hits {  
      h.set_coordinates(trk_strips);
    }
  }
 
  #[cfg(feature="database")]
  pub fn mask_strips(&mut self, masks : &HashMap<u32, TrackerStripMask>) {
    let mut clean_hits = Vec::<TrackerHit>::with_capacity(self.tracker_hits.len());
    for h in &self.tracker_hits {
      if !masks.contains_key(&h.get_stripid()) {
        warn!("We don't have a mask information for strip id {}", h.get_stripid());
        continue;
      }
      if masks[&h.get_stripid()].active {
        clean_hits.push(h.clone());
      }
    }
    self.tracker_hits = clean_hits;
  }


  /// Applies a cut based on adc values
  #[cfg(feature="database")]
  pub fn apply_signal_cut(&mut self, cut : f32, pedestals : &HashMap<u32, TrackerStripPedestal>) {
    let mut clean_hits = Vec::<TrackerHit>::with_capacity(self.tracker_hits.len());
    for h in &self.tracker_hits {
      if !pedestals.contains_key(&h.get_stripid()) {
        warn!("We don't have pedestal information for strip id {}", h.get_stripid());
        continue;
      }
      let ped = &pedestals[&h.get_stripid()];
      if (h.adc as f32) - ped.pedestal_mean > (cut * ped.pedestal_sigma){
        clean_hits.push(h.clone());
      }
    }
    self.tracker_hits = clean_hits;
  }
 
  /// Calculate the absolute common noise (adc)
  #[cfg(feature="database")]
  pub fn cmn_noise(hit       : &TrackerHit, 
                   pedestals : &HashMap<u32, TrackerStripPedestal>,
                   cmn_noise : &HashMap<u32, TrackerStripCmnNoise>) -> Option<f32> {
    let stripid = hit.get_stripid();
    if !cmn_noise.contains_key(&stripid) {
      warn!("We don't have pedestal information for strip id {}", stripid);
      return None;
    }
    if !pedestals.contains_key(&stripid) {
      warn!("We don't have pedestal information for strip id {}", stripid);
      return None;
    }
    let mut cmn_level = 0.0f32;
    let adc_ped_sub   = hit.adc as f32 - pedestals[&stripid].pedestal_mean; 
    if adc_ped_sub < 400.0 {
      cmn_level = cmn_noise[&stripid].common_level(hit.adc as f32);
    }
    return Some(adc_ped_sub - cmn_noise[&stripid].gain * cmn_level);
  }

  /// Calculate the energy deposition
  #[cfg(feature="database")]
  pub fn get_trk_energy(adc : f32, tf : &TrackerStripTransferFunction) -> f32 {
    let mut energy = 0.0f32;
    let mut adc_m  = adc; 
    if adc_m <= 0.0 {
      return energy;
    }
    if adc_m > 1600.0 {
      adc_m = 1600.0;
    }
    #[allow(non_snake_case)]
    let mV2keV  = 0.841f32;
    // the max and min range are basically defined by the 
    // polynominal [0-1600]
    let voltage = tf.transfer_fn(adc_m);
    println!("voltage {}", voltage);
    energy  = voltage*mV2keV;
    energy /= 1000.0;
    println!("adc : {} , energy {}", adc_m, energy);
    return energy;
  }

  #[cfg(feature="database")]
  pub fn calibrate_tracker(&mut self, 
                           remove_cmn_noise : bool,
                           pedestals        : &HashMap<u32, TrackerStripPedestal>,
                           transfer_fn      : &HashMap<u32, TrackerStripTransferFunction>,
                           cmn_noise        : &HashMap<u32, TrackerStripCmnNoise>) {
    //println!("Will remove CMN noise {}", remove_cmn_noise);
    for h in &mut self.tracker_hits {
      let stripid = h.get_stripid();
      //println!("Running TRK calibration for strip {}", stripid);
      if !pedestals.contains_key(&stripid) {
        warn!("Pedestal map does not contain strip {}. Will not calculate energy!", stripid);
        continue;
      }
      if !transfer_fn.contains_key(&stripid) {
        warn!("Transfer fn for strip {} not available. Will not calculate energy!", stripid);
        continue;
      }
      let adc_no_ped = h.adc as f32 - pedestals[&stripid].pedestal_mean;
      //println!("ADC NO PED {}", adc_no_ped);
      if remove_cmn_noise {
        match Self::cmn_noise(&h, pedestals, cmn_noise) { 
          None => {
            h.energy = Self::get_trk_energy(adc_no_ped, &transfer_fn[&stripid]); 
          }
          Some(cmn) => {
            h.energy = Self::get_trk_energy(cmn, &transfer_fn[&stripid]);
          }
        }    
      } else {
        h.energy = Self::get_trk_energy(adc_no_ped, &transfer_fn[&stripid]);
      }
    }
  }
}


impl TelemetryPackable for TelemetryEvent {
  // trivial for TelemetryEvent, since the default 
  // already contains the packet types
}

impl Serialization for TelemetryEvent {
  
  fn from_bytestream(stream : &Vec<u8>,
                     pos    : &mut usize)
    -> Result<Self, SerializationError> {
    let mut me       = Self::new();
    let version      = parse_u8(stream, pos);
    me.version       = version;
    //println!("_version {}", _version);
    me.flags0         = parse_u8(stream, pos);
    // skip a bunch of Alex newly implemented things
    // FIXME
    if version == 0 {
      me.flags1      = parse_u8(stream, pos);
    } else {
      *pos += 8;
    }

    me.event_id       = parse_u32(stream, pos);
    //println!("EVENT ID {}", me.event_id);
    let _tof_delim    = parse_u8(stream, pos);
    //println!("TOF delim : {}", _tof_delim);
    if stream.len() <= *pos + 2 {
      error!("Not able to parse merged event!");
      return Err(SerializationError::StreamTooShort);
    }
   let num_tof_bytes = parse_u16(stream, pos) as usize;
    //println!("Num TOF bytes : {}", num_tof_bytes);
    if stream.len() < *pos+num_tof_bytes {
      error!("Not enough bytes for TOF packet! Expected {}, seen {}", *pos+num_tof_bytes as usize, stream.len());
      return Err(SerializationError::StreamTooShort); 
    }
    let pos_before = *pos;
    if num_tof_bytes != 0 {
      let tof_pack   = TofPacket::from_bytestream(stream, pos)?;
      let ts         = tof_pack.unpack::<TofEvent>()?;
    // sanity check - is tofpacket as long as num_tof_bytes lets us believe?
      me.tof_event = ts;
    }
    if pos_before + num_tof_bytes != *pos {
      error!("Byte misalignment. Expected {num_tof_bytes}, got {pos} - {pos_before}"); 
      return Err(SerializationError::WrongByteSize);
    }
    let trk_delim    = parse_u8(stream, pos);

    //println!("TRK delim {}", trk_delim);
    if trk_delim != 0xbb {
      return Err(SerializationError::HeadInvalid);
    }
    if version == 1 {
      let num_trk_hits = parse_u16(stream, pos);
      if (*pos + (num_trk_hits as usize)*4 ) > stream.len() {
        return Err(SerializationError::StreamTooShort);
      }
      for _ in 0..num_trk_hits { 
        let mut hit  = TrackerHit::new();
        let strip_id = parse_u16(stream, pos);
        let adc      = parse_u16(stream, pos);
        hit.channel  = strip_id & 0b11111;
        hit.module   = (strip_id >> 5) & 0b111;
        hit.row      = (strip_id >> 8) & 0b111;
        hit.layer    = (strip_id >> 11) & 0b1111;
        hit.adc      = adc;
        me.tracker_hits.push(hit);
      }
      // oscillators
      let oscillators_delimiter = parse_u8(stream, pos);
      if oscillators_delimiter != 0xcc {
        return Err(SerializationError::HeadInvalid);
      }
      let osc_flags = parse_u8(stream, pos);
      let mut oscillator_idx = Vec::<u8>::new();
      for j in 0..8 {
        if (osc_flags >> j & 0b1) > 0 {
          oscillator_idx.push(j)
        }
      }
      if (*pos + oscillator_idx.len()*6) > stream.len() {
        return Err(SerializationError::StreamTooShort);
      }
      for idx in oscillator_idx.iter() {
        let lower = parse_u32(stream, pos);
        let upper = parse_u16(stream, pos);
        let osc : u64 = (upper as u64) << 32 | (lower as u64);
        me.tracker_oscillators[*idx as usize] = osc;
      }
    } else if version == 0 {
      error!("Unsupported {version}!");
      return Err(SerializationError::UnsupportedVersion);
    } else {
      error!("Unsuported version {version}!");
      return Err(SerializationError::UnsupportedVersion);
    } 
    Ok(me)
  }
}

impl fmt::Display for TelemetryEvent {
  fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
    let mut repr     = String::from("<TelemetryEvent:");
    let tof_str  = format!("\n  {}", self.tof_event);
    let mut good_hits = 0;
    if self.version == 0 {
      repr += "\n VERSION 0 NOT SUPPORTED!!";
    } else if self.version == 1 {
      for _ in &self.tracker_hits {
        good_hits += 1;
      }
    }
    repr += &(format!("  {}", self.header));
    repr += "\n  ** ** ** MERGED  ** ** **";
    repr += &(format!("\n  version         {}", self.version));
    repr += &(format!("\n  event ID        {}", self.event_id));  
    if self.version == 0 {
      repr += "\n VERSION 0 NOT SUPPORTED!!"; 
    }
    repr += "\n  ** ** ** TRACKER ** ** **";
    if self.version == 0 {
      repr += "\n VERSION 0 NOT SUPPORTED!!"; 
    } else if self.version == 1 {
      repr += &(format!("\n  Trk oscillators {:?}", self.tracker_oscillators)); 
    }
    repr += &(format!("\n  N Good Trk Hits {}", good_hits));
    repr += &tof_str;
    write!(f,"{}", repr)
  }
}

//----------------------------------------

#[cfg(feature="pybindings")]
#[pymethods]
impl TelemetryEvent {

  #[getter]
  #[pyo3(name="version")]
  fn version_py(&self) -> u8 {
    self.version
  }
  
  #[staticmethod]
  #[pyo3(name = "get_trk_energy")]
  fn get_trk_energy_py(adc : f32, tf : &TrackerStripTransferFunction) -> f32 {
    Self::get_trk_energy(adc, tf)
  }

  #[getter]
  fn get_header(&self) -> TelemetryPacketHeader {
    self.header
  }

  #[getter]
  fn tracker(&self) -> PyResult<Vec<TrackerHit>> {
    Ok(self.tracker_hits.clone())
  }

  #[getter]
  fn get_event_id(&self) -> u32 {
    self.event_id
  }

  // FIXME - do this with bound
  #[getter]
  fn get_tof(&self) -> PyResult<TofEvent> {
    Ok(self.tof_event.clone())
  }

  #[getter]
  fn tracker_pointcloud(&self) -> Vec<(f32, f32, f32, f32, f32)> {
    let mut pts = Vec::<(f32,f32,f32,f32,f32)>::new();
    for h in &self.tracker_hits {
      // uses adc
      // FIXME - factor 10!
      let pt = (10.0*h.x, 10.0*h.y, 10.0*h.z, f32::NAN, h.adc as f32);
      pts.push(pt);
    }
    pts
  }

  /// Populate a merged event from a TelemetryPacket.
  ///
  /// Telemetry packet type should be 90 (MergedEvent)
  #[staticmethod]
  fn from_telemetrypacket(packet : TelemetryPacket) -> PyResult<Self> {
    match Self::from_bytestream(&packet.payload, &mut 0) {
      Ok(mut event) => {
        event.header = packet.header.clone();
        return Ok(event);
      }
      Err(err) => {
        return Err(PyValueError::new_err(err.to_string()));
      }  
    }
  }

//  #[cfg(feature="database")]
//  #[pyo3(name="mask_strips")]
//  pub fn mask_strips_py(&mut self, masks : &HashMap<u32, TrackerStripMask>) {
//  }
//
//  #[cfg(feature="database")]
//  #[pyo3(name="remove_cmn_noise")]
//  pub fn remove_cmn_noise_py(&mut self, cmn_noise : &HashMap<u32, TrackerStripCmnNoise>) {
//  }
//
//  #[cfg(feature="database")]
//  #[pyo3(name="calibrate_tracker")]
//  pub fn calibrate_tracker_py(&mut self, 
//                              pedestals   : &HashMap<u32, TrackerStripPedestal>,
//                              transfer_fn : &HashMap<u32, TrackerStripTransferFunction>) {
//  }


}

#[cfg(feature="pybindings")]
pythonize!(TelemetryEvent);

