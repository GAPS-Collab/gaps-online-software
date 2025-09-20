// The following file is part of gaps-online-software and published 
// under the GPLv3 license

use crate::prelude::*;
use std::f32::consts::PI;

/// Waveform peak
///
/// Helper to form TofHits
#[derive(Debug,Copy,Clone,PartialEq)]
pub struct Peak {
  pub paddle_end_id : u16,
  pub time          : f32,
  pub charge        : f32,
  pub height        : f32
}

impl Peak {
  pub fn new() -> Self {
    Self {
      // but why??
      paddle_end_id : 40,
      time          : 0.0,
      charge        : 0.0,
      height        : 0.0,
    }
  }
}

impl Default for Peak {
  fn default() -> Self {
    Self::new()
  }
}

impl fmt::Display for Peak {
  fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
    write!(f, "<Peak:
  p_end_id : {:.2} 
  time // charge // height    : {:.2} // {:.2} // {:.2}>", 
            self.paddle_end_id,
            self.time,
            self.charge,
            self.height)
  }
}

//----------------------------------------------------------------

/// An extracted hit from a TofPaddle, as extracted by the 
/// online software and provided algorithm
/// (in v0.11 algorithm is provided by J.Zweerink)
///
/// A TofHit holds the information for an extracted single 
/// hit on a peak, which is defined by a peak in at least one 
/// of the two waveforms. 
/// The TofHit holds extracted information for both of the 
/// waveforms, only if both are available a position reconstruction
/// on the paddle can be attempted.
///
/// A and B are the different ends of the paddle

#[derive(Debug,Copy,Clone,PartialEq)]
#[cfg_attr(feature = "pybindings", pyclass, pyo3(name="TofHit2"))]
pub struct TofHit {
  
  // We currently have 3 bytes to spare

  /// The ID of the paddle in TOF notation
  /// (1-160)
  pub paddle_id      : u8,
  pub time_a         : f16,
  pub time_b         : f16,
  pub peak_a         : f16,
  pub peak_b         : f16,
  pub charge_a       : f16,
  pub charge_b       : f16,
  // only 2 bytes of version
  // are used thus we have a reserved field
  pub reserved       : u8,
  pub version        : ProtocolVersion,
  // for now, but we want to use half instead
  pub baseline_a     : f16,
  pub baseline_a_rms : f16,
  pub baseline_b     : f16,
  pub baseline_b_rms : f16,
  // phase of the sine fit
  pub phase          : f16,
  
  //-------------------------------
  // NON-SERIALIZED FIELDS
  //-------------------------------

  /// Length of the paddle the hit is on, will get 
  /// populated from db
  pub paddle_len     : f32,
  /// (In)famous constant timing offset per paddle
  pub timing_offset  : f32,
  pub coax_cable_time: f32,
  pub hart_cable_time: f32,
  /// normalized t0, where we have the phase difference
  /// limited to -pi/2 -> pi/2
  pub event_t0       : f32,
  pub x              : f32,
  pub y              : f32,
  pub z              : f32,

  // fields which won't get 
  // serialized
  pub valid          : bool,
}

// methods without pybindings
impl TofHit {
  pub fn new() -> Self {
    Self{
      paddle_id       : 0,
      time_a          : f16::from_f32(0.0),
      time_b          : f16::from_f32(0.0),
      peak_a          : f16::from_f32(0.0),
      peak_b          : f16::from_f32(0.0),
      charge_a        : f16::from_f32(0.0),
      charge_b        : f16::from_f32(0.0),
      paddle_len      : f32::NAN,
      timing_offset   : 0.0,
      coax_cable_time : f32::NAN,
      hart_cable_time : f32::NAN,
      event_t0        : f32::NAN,
      x               : f32::NAN,
      y               : f32::NAN,
      z               : f32::NAN,
      
      valid           : true,
      // v1 variables 
      version         : ProtocolVersion::V1,
      reserved        : 0,
      baseline_a      : f16::from_f32(0.0),
      baseline_a_rms  : f16::from_f32(0.0),
      baseline_b      : f16::from_f32(0.0),
      baseline_b_rms  : f16::from_f32(0.0),
      phase           : f16::from_f32(0.0),
    }
  }
  
  /// Adds an extracted peak to this TofHit. A peak will be 
  /// for only a single waveform only, so we have to take 
  /// care of the A/B sorting by means of PaddleEndId
  pub fn add_peak(&mut self, peak : &Peak)  {
    if self.paddle_id != TofHit::get_pid(peak.paddle_end_id) {
      //error!("Can't add peak to 
    }
    if peak.paddle_end_id < 1000 {
      error!("Invalide paddle end id {}", peak.paddle_end_id);
    }
    if peak.paddle_end_id > 2000 {
      self.set_time_b  (peak.time);
      self.set_peak_b  (peak.height);
      self.set_charge_b(peak.charge);
    } else if peak.paddle_end_id < 2000 {
      self.set_time_a  (peak.time);
      self.set_peak_a  (peak.height);
      self.set_charge_a(peak.charge);
    }
  }
  
  // None of the setters will have pybindings
  pub fn set_time_b(&mut self, t : f32) {
    self.time_b = f16::from_f32(t)
  }
  
  pub fn set_time_a(&mut self, t : f32) {
    self.time_a = f16::from_f32(t);
  }

  pub fn set_peak_a(&mut self, p : f32) {
    self.peak_a = f16::from_f32(p)
  }

  pub fn set_peak_b(&mut self, p : f32) {
    self.peak_b = f16::from_f32(p)
  }

  pub fn set_charge_a(&mut self, c : f32) {
    self.charge_a = f16::from_f32(c)
  }

  pub fn set_charge_b(&mut self, c : f32) {
    self.charge_b = f16::from_f32(c)
  }
}

// wrapper methods which duplicate the rust code,
// but need addional configuration with pyo3, e.g.
// the #getter attribute
#[cfg(feature="pybindings")]
#[pymethods]
impl TofHit {
  
  /// The paddle id (1-160) of the hit paddle
  #[getter]
  #[pyo3(name="paddle_id")]
  fn paddle_id_py(&self) -> u8 {
    self.paddle_id
  }

  /// The length of the paddle, only available after 
  /// the paddle information has been added through
  /// "set_paddle"
  #[getter]
  #[pyo3(name="paddle_len")]
  fn get_paddle_len_py(&self) -> f32 {
    self.paddle_len
  }
  
  /// Set the length and cable length for the paddle
  /// FIXME - take gaps_online.db.Paddle as argument
  #[pyo3(name="set_paddle")]
  fn set_paddle_py(&mut self, plen : f32, coax_cbl_time : f32, hart_cbl_time : f32 ) {
    self.paddle_len      = plen;
    self.coax_cable_time = coax_cbl_time;
    self.hart_cable_time = hart_cbl_time;
  }
  
  /// The time in ns the signal spends in the coax 
  /// cables from the SiPMs to the RAT
  #[getter]
  #[pyo3(name="coax_cbl_time")]
  fn get_coax_cbl_time(&self) -> f32 {
    self.coax_cable_time
  }

  /// The time in ns the signal spends in the Harting
  /// cables from the RATs to the MTB
  #[getter]
  #[pyo3(name="hart_cbl_time")]
  fn get_hart_cbl_time(&self) -> f32 {
    self.hart_cable_time
  }
  
  /// Calculate the position across the paddle from
  /// the two times at the paddle ends
  ///
  /// **This will be measured from the A side**
  ///
  /// Just to be extra clear, this assumes the two 
  /// sets of cables for each paddle end have the
  /// same length
  #[getter]
  #[pyo3(name="pos")] 
  fn get_pos_py(&self) -> f32 {
    self.get_pos() 
  } 
  
  #[getter]
  #[pyo3(name="x")]
  fn x_py(&self) -> f32 {
    self.x
  }
  
  #[getter]
  #[pyo3(name="y")]
  fn y_py(&self) -> f32 {
    self.y
  }

  #[getter]
  #[pyo3(name="z")]
  fn z_py(&self) -> f32 {
    self.z
  }
  
  #[getter]
  #[pyo3(name="version")]
  fn version_py(&self) -> ProtocolVersion {
    self.version
  }

  #[getter]
  #[pyo3(name="phase")]
  fn phase_py(&self) -> f32 {
    self.phase.to_f32()
  }
 
  #[getter]
  #[pyo3(name="cable_delay")]
  /// Get the cable correction time
  fn get_cable_delay_py(&self) -> f32 {
    self.get_cable_delay()
  }

  /// Get the delay relative to other readoutboards based 
  /// on the channel9 sine wave
  #[getter]
  #[pyo3(name="phase_delay")]
  fn get_phase_delay_py(&self) -> f32 { 
    self.get_phase_delay()
  }
  
  /// That this works, the length of the paddle has to 
  /// be set before (in mm).
  /// This assumes that the cable on both sides of the paddle are 
  /// the same length
  #[getter]
  #[pyo3(name="t0")]
  fn get_t0_py(&self) -> f32 {
    self.get_t0()
  }
  
  /// Event t0 is the calculated interaction time based on 
  /// the RELATIVE phase shifts consdering ALL hits in this
  /// event. This might be of importance to catch rollovers
  /// in the phase of channel9. 
  /// In total, we are restricting ourselves to a time of 
  /// 50ns per events and adjust the phase in such a way that 
  /// everything fits into this interval. This will 
  /// significantly import the beta reconstruction for particles
  /// which hit the TOF within this timing window.
  ///
  /// If a timing offset is set, this will be added
  #[getter]
  #[pyo3(name="event_t0")]
  fn get_event_t0_py(&self) -> f32 {
    self.get_t0()
  }

  /// Calculate the interaction time based on the peak timings measured 
  /// at the paddle ends A and B
  ///
  /// This does not correct for any cable length
  /// or ch9 phase shift
  #[getter]
  #[pyo3(name="t0_uncorrected")]
  fn get_t0_uncorrected_py(&self) -> f32 {
    self.get_t0_uncorrected()
  }

  /// Philip's energy deposition based on peak height
  #[getter]
  #[pyo3(name="edep")]
  fn get_edep_py(&self) -> f32 {
    self.get_edep()
  }
  
  /// Elena's energy deposition based on peak height
  #[getter]
  #[pyo3(name="edep")]
  fn get_edep_att_py(&self) -> f32 {
    self.get_edep_att()
  }

  /// Arrival time of the photons at side A
  #[getter]
  #[pyo3(name="time_a")]
  fn get_time_a_py(&self) -> f32 {
    self.get_time_a()
  }

  /// Arrival time of the photons at side B
  #[getter]
  #[pyo3(name="time_b")]
  fn get_time_b_py(&self) -> f32 {
    self.get_time_b()
  }
  
  #[getter]
  #[pyo3(name="peak_a")]
  fn get_peak_a_py(&self) -> f32 {
    self.get_peak_a()
  }
  
  #[getter]
  #[pyo3(name="peak_b")]
  fn get_peak_b_py(&self) -> f32 {
    self.get_peak_b()
  }
  
  #[getter]
  #[pyo3(name="charge_a")]
  fn get_charge_a_py(&self) -> f32 {
    self.get_charge_a()
  }
  
  #[getter]
  #[pyo3(name="charge_b")]
  fn get_charge_b_py(&self) -> f32 {
    self.get_charge_b()
  }
  
  #[getter]
  #[pyo3(name="baseline_a")]
  fn get_bl_a_py(&self) -> f32 {
    self.get_bl_a()
  }
  
  #[getter]
  #[pyo3(name="baseline_b")]
  fn get_bl_b_py(&self) -> f32 {
    self.get_bl_b()
  }
  
  #[getter]
  #[pyo3(name="baseline_a_rms")]
  fn get_bl_a_rms_py(&self) -> f32 {
    self.get_bl_a_rms()
  }
  
  #[getter]
  #[pyo3(name="baseline_b_rms")]
  fn get_bl_b_rms_py(&self) -> f32 {
    self.get_bl_b_rms()
  }
}

// methods which are available in rust, but can 
// have an implementation in python.
// Sets the pymethods attribute conditionally
#[cfg_attr(feature="pybindings", pymethods)]
impl TofHit {
  /// Calculate the distance to another hit. For this 
  /// to work, the hit coordinates have had to be 
  /// determined, so this will only return a 
  /// propper result after the paddle information 
  /// is added
  pub fn distance(&self, other : &TofHit) -> f32 {
    ((self.x - other.x).powi(2) + (self.y - other.y).powi(2) + (self.z - other.z).powi(2)).sqrt()
  } 
  
  /// If the two reconstructed pulse times are not related to each other by the paddle length,
  /// meaning that they can't be caused by the same event, we dub this hit as "not following
  /// causality"
  pub fn obeys_causality(&self) -> bool {
    (self.paddle_len/(10.0*C_LIGHT_PADDLE)) - f32::abs(self.time_a.to_f32() - self.time_b.to_f32()) > 0.0
    && self.get_t0_uncorrected() > 0.0
  }
}

#[cfg(feature="pybindings")]
pythonize!(TofHit);

// methods which have wrapped pybindings
impl TofHit {
  
  
  
  /// Calculate the position across the paddle from
  /// the two times at the paddle ends
  ///
  /// **This will be measured from the A side**
  ///
  /// Just to be extra clear, this assumes the two 
  /// sets of cables for each paddle end have the
  /// same length
  pub fn get_pos(&self) -> f32 {
    let t0 = self.get_t0_uncorrected();
    let clean_t_a = self.time_a.to_f32() - t0;
    return clean_t_a*C_LIGHT_PADDLE*10.0; 
  }
  
  /// Get the cable correction time
  pub fn get_cable_delay(&self) -> f32 {
    self.hart_cable_time - self.coax_cable_time 
  }

  /// Get the delay relative to other readoutboards based 
  /// on the channel9 sine wave
  pub fn get_phase_delay(&self) -> f32 { 
    let freq : f32 = 20.0e6;
    let phase = self.phase.to_f32();
    // fit allows for negative phase shift.
    // that means to distinguish 2 points, we
    // only have HALF of the sine wave
    // FIXME - implement warning?
    //while phase < -PI {
    //  phase += 2.0*PI;
    //}
    //while phase > PI {
    //  phase -= 2.0*PI;
    //}
    (phase/(2.0*PI*freq))*1.0e9f32 
  }
  
  /// That this works, the length of the paddle has to 
  /// be set before (in mm).
  /// This assumes that the cable on both sides of the paddle are 
  /// the same length
  pub fn get_t0(&self) -> f32 {
    //self.get_t0_uncorrected() + self.get_phase_delay() + self.get_cable_delay()
    self.event_t0 + self.timing_offset
  }

  /// Calculate the interaction time based on the peak timings measured 
  /// at the paddle ends A and B
  ///
  /// This does not correct for any cable length
  /// or ch9 phase shift
  pub fn get_t0_uncorrected(&self) -> f32 {
    0.5*(self.time_a.to_f32() + self.time_b.to_f32() - (self.paddle_len/(10.0*C_LIGHT_PADDLE)))
  }

  /// Philip's energy deposition based on peak height
  pub fn get_edep(&self) -> f32 {
    (1.29/34.3)*(self.peak_a.to_f32() + self.peak_b.to_f32()) / 2.0
  }
  
  /// Elena's energy deposition including attenuation
  pub fn get_edep_att(&self) -> f32 {
    let x0    = self.get_pos();
    let att_a = ((3.9-0.00126*( x0+self.paddle_len/2.))+22.15).exp() / ((3.9)+22.15).exp();
    let att_b = ((3.9-0.00126*(-x0+self.paddle_len/2.))+22.15).exp() / ((3.9)+22.15).exp();
    let edep  = 0.0159 * (self.get_peak_a()/att_a + self.get_peak_b()/att_b) / 2.; // vertical muon peak @ 0.97 MeV
    return edep; 
  }

  /// Arrival time of the photons at side A
  pub fn get_time_a(&self) -> f32 {
    self.time_a.to_f32()
  }

  /// Arrival time of the photons at side B
  pub fn get_time_b(&self) -> f32 {
    self.time_b.to_f32()
  }
  
  pub fn get_peak_a(&self) -> f32 {
    self.peak_a.to_f32()
  }
  
  pub fn get_peak_b(&self) -> f32 {
    self.peak_b.to_f32()
  }
  
  pub fn get_charge_a(&self) -> f32 {
    self.charge_a.to_f32()
  }
  
  pub fn get_charge_b(&self) -> f32 {
    self.charge_b.to_f32()
  }
  
  pub fn get_bl_a(&self) -> f32 {
    self.baseline_a.to_f32()
  }
  
  pub fn get_bl_b(&self) -> f32 {
    self.baseline_b.to_f32()
  }
  
  pub fn get_bl_a_rms(&self) -> f32 {
    self.baseline_a_rms.to_f32()
  }
  
  pub fn get_bl_b_rms(&self) -> f32 {
    self.baseline_b_rms.to_f32()
  }


  /// Get the (official) paddle id
  ///
  /// Convert the paddle end id following 
  /// the convention
  ///
  /// A-side : paddle id + 1000
  /// B-side : paddle id + 2000
  ///
  /// FIXME - maybe return Result?
  //#[deprecated(since="0.10", note="We are not using a paddle end id anymore")]
  pub fn get_pid(paddle_end_id : u16) -> u8 {
    if paddle_end_id < 1000 {
      return 0;
    }
    if paddle_end_id > 2000 {
      return (paddle_end_id - 2000) as u8;
    }
    if paddle_end_id < 2000 {
      return (paddle_end_id - 1000) as u8;
    }
    return 0;
  }



  pub fn get_phase_rollovers(&self) -> i16 {
    let mut phase = self.phase.to_f32();
    let mut ro = 0i16;
    while phase < PI/2.0 {
      phase += PI/2.0;
      ro += 1;
    }
    while phase > PI/2.0 {
      phase -= PI/2.0;
      ro -= 1;
    }
    ro
  }
  

}

#[cfg(feature="database")]
impl TofHit {
  pub fn set_paddle(&mut self, paddle : &TofPaddle) {
    self.coax_cable_time = paddle.coax_cable_time;
    self.hart_cable_time = paddle.harting_cable_time;
    self.paddle_len = paddle.length * 10.0; // stupid units!
    let pr          = paddle.principal();
    //println!("Principal {:?}", pr);
    let rel_pos     = self.get_pos();
    let pos         = (paddle.global_pos_x_l0_A*10.0 + pr.0*rel_pos,
                       paddle.global_pos_y_l0_A*10.0 + pr.1*rel_pos,
                       paddle.global_pos_z_l0_A*10.0 + pr.2*rel_pos);
    self.x          = pos.0;
    self.y          = pos.1;
    self.z          = pos.2;
  }
}


// Implementation of traits 
//-------------------------

impl Default for TofHit {
  fn default() -> Self {
    Self::new()
  }
}

impl fmt::Display for TofHit {
  fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
    let mut paddle_info = String::from("");
    if self.paddle_len == 0.0 {
      paddle_info = String::from("NOT SET!");
    }
    write!(f, "<TofHit (version : {}):
  Paddle ID       {}
  Peak:
    LE Time A/B   {:.2} {:.2}   
    Height  A/B   {:.2} {:.2}
    Charge  A/B   {:.2} {:.2}
  ** paddle {} ** 
    Length        {:.2}
    Timing offset {:.2} (ns)
    Coax cbl time {:.2}
    Hart cbl time {:.2}
  ** reconstructed interaction
    energy_dep    {:.2}   
    edep_att      {:.2}
    pos_across    {:.2}   
    t0            {:.2}  
    x, y, z       {:.2} {:.2} {:.2}
  ** V1 variables
    phase (ch9)   {:.4}
      n phs ro    {}
    baseline A/B  {:.2} {:.2}
    bl. RMS  A/B  {:.2} {:.2}>",
            self.version,
            self.paddle_id,
            self.get_time_a(),
            self.get_time_b(),
            self.get_peak_a(),
            self.get_peak_b(),
            self.get_charge_a(),
            self.get_charge_b(),
            paddle_info,
            self.paddle_len,
            self.timing_offset,
            self.coax_cable_time,
            self.hart_cable_time,
            self.get_edep(),
            self.get_edep_att(),
            self.get_pos(),
            self.get_t0(),
            self.x,
            self.y,
            self.z,
            self.phase,
            self.get_phase_rollovers(),
            self.baseline_a,
            self.baseline_b,
            self.baseline_a_rms,
            self.baseline_b_rms,
            )
  }
}

impl Serialization for TofHit {
  
  const HEAD          : u16   = 61680; //0xF0F0)
  const TAIL          : u16   = 3855;
  const SIZE          : usize = 30; // size in bytes with HEAD and TAIL

  /// Serialize the packet
  ///
  /// Not all fields will get serialized, 
  /// only the relevant data for the 
  /// flight computer
  //
  /// **A note about protocol versions **
  /// When we serialize (to_bytestream) we will
  /// always write the latest version.
  /// Deserialization can also read previous versions
  fn to_bytestream(&self) -> Vec<u8> {

    let mut bytestream = Vec::<u8>::with_capacity(Self::SIZE);
    bytestream.extend_from_slice(&Self::HEAD.to_le_bytes());
    bytestream.push(self.paddle_id); 
    bytestream.extend_from_slice(&self.time_a      .to_le_bytes()); 
    bytestream.extend_from_slice(&self.time_b      .to_le_bytes()); 
    bytestream.extend_from_slice(&self.peak_a      .to_le_bytes()); 
    bytestream.extend_from_slice(&self.peak_b      .to_le_bytes()); 
    bytestream.extend_from_slice(&self.charge_a    .to_le_bytes()); 
    bytestream.extend_from_slice(&self.charge_b    .to_le_bytes()); 
    // charge_min_i has been removed, so just insert a 0 here 
    bytestream.extend_from_slice(&0u16.to_le_bytes()); 
    bytestream.extend_from_slice(&self.baseline_a   .to_le_bytes());
    bytestream.extend_from_slice(&self.baseline_a_rms.to_le_bytes());
    bytestream.extend_from_slice(&self.phase       .to_le_bytes());
    bytestream.push(self.version.to_u8());
    bytestream.extend_from_slice(&self.baseline_b.to_le_bytes());
    bytestream.extend_from_slice(&self.baseline_b_rms.to_le_bytes());
    bytestream.extend_from_slice(&Self::TAIL       .to_le_bytes()); 
    bytestream
  }


  /// Deserialization
  ///
  ///
  /// # Arguments:
  ///
  /// * bytestream : 
  fn from_bytestream(stream : &Vec<u8>, pos : &mut usize) 
    -> Result<Self, SerializationError> {
    let mut pp  = Self::new();
    Self::verify_fixed(stream, pos)?;
    // since we passed the above test, the packet
    // is valid
    pp.valid          = true;
    pp.paddle_id      = parse_u8(stream, pos);
    pp.time_a         = parse_f16(stream, pos);
    pp.time_b         = parse_f16(stream, pos);
    pp.peak_a         = parse_f16(stream, pos);
    pp.peak_b         = parse_f16(stream, pos);
    pp.charge_a       = parse_f16(stream, pos);
    pp.charge_b       = parse_f16(stream, pos);
    // we have a currently empty field
    *pos += 2;
    //pp.charge_min_i   = parse_u16(stream, pos);
    pp.baseline_a     = parse_f16(stream, pos);
    pp.baseline_a_rms = parse_f16(stream, pos);
    let mut phase_vec = Vec::<u8>::new();
    phase_vec.push(parse_u8(stream, pos));
    phase_vec.push(parse_u8(stream, pos));
    pp.phase    = parse_f16(&phase_vec, &mut 0);
    let version      = ProtocolVersion::from(parse_u8(stream, pos));
    pp.version       = version;
    match pp.version {
      ProtocolVersion::V1 => {
      }
      _ => ()
    }
    pp.baseline_b      = parse_f16(stream, pos);
    pp.baseline_b_rms  = parse_f16(stream, pos);
    *pos += 2; // always have to do this when using verify fixed
    Ok(pp)
  }
}

#[cfg(feature="random")]
impl FromRandom for TofHit {
  fn from_random() -> TofHit {
    let mut pp  = TofHit::new();
    let mut rng = rand::rng();

    pp.paddle_id      = rng.random::<u8> ();
    pp.time_a         = f16::from_f32(rng.random::<f32>());
    pp.time_b         = f16::from_f32(rng.random::<f32>());
    pp.peak_a         = f16::from_f32(rng.random::<f32>());
    pp.peak_b         = f16::from_f32(rng.random::<f32>());
    pp.charge_a       = f16::from_f32(rng.random::<f32>());
    pp.charge_b       = f16::from_f32(rng.random::<f32>());
    pp.version        = ProtocolVersion::from(rng.random::<u8>());
    pp.baseline_a     = f16::from_f32(rng.random::<f32>());
    pp.baseline_a_rms = f16::from_f32(rng.random::<f32>());
    pp.baseline_b     = f16::from_f32(rng.random::<f32>());
    pp.baseline_b_rms = f16::from_f32(rng.random::<f32>());
    pp.phase          = f16::from_f32(rng.random::<f32>());
    
    pp.paddle_len       = 0.0; 
    pp.coax_cable_time  = 0.0; 
    pp.hart_cable_time  = 0.0; 
    pp.x                = 0.0; 
    pp.y                = 0.0; 
    pp.z                = 0.0; 
    pp.event_t0         = 0.0; 
    pp
  }
}

//---------------------------------------------------------------

//#[cfg(feature="pybindings")]
//#[pyclass]
//#[pyo3(name="TofHit")]
//pub struct PyTofHit {
//  hit : TofHit,
//}
//
//#[cfg(feature="pybindings")]
//#[pymethods]
//impl PyTofHit {
//  //#[new]
//  //fn new() -> Self {
//  //  Self {
//  //    hit : TofHit::new(),
//  //  }
//  //}
//
//  pub fn set_timing_offset(&mut self, offset : f32) {
//    self.hit.timing_offset = offset;
//  }
//
//  /// Calculate the distance to another hit. For this 
//  /// to work, the hit coordinates have had to be 
//  /// determined, so this will only return a 
//  /// propper result after the paddle information 
//  /// is added
//  pub fn distance(&self, other : &PyTofHit) -> f32 {
//    //((self.x - other.x).powi(2) + (self.y - other.y).powi(2) + (self.z - other.z).powi(2)).sqrt()
//    self.hit.distance(&other.hit)
//  }
// 
//  /// Set the length and cable length for the paddle
//  /// FIXME - take gaps_online.db.Paddle as argument
//  fn set_paddle(&mut self, plen : f32, coax_cbl_time : f32, hart_cbl_time : f32 ) {
//    self.hit.paddle_len = plen;
//    self.hit.coax_cable_time = coax_cbl_time;
//    self.hit.hart_cable_time = hart_cbl_time;
//  }
//
//  #[getter]
//  fn x(&self) -> f32 {
//    self.hit.x
//  }
//  
//  #[getter]
//  fn y(&self) -> f32 {
//    self.hit.y
//  }
//
//  #[getter]
//  fn z(&self) -> f32 {
//    self.hit.z
//  }
//
//  #[getter]
//  fn get_t0_uncorrected(&self) -> f32 {
//    self.hit.get_t0_uncorrected()
//  }
//  
//  /// Event t0 is the calculated interaction time based on 
//  /// the RELATIVE phase shifts consdering ALL hits in this
//  /// event. This might be of importance to catch rollovers
//  /// in the phase of channel9. 
//  /// In total, we are restricting ourselves to a time of 
//  /// 50ns per events and adjust the phase in such a way that 
//  /// everything fits into this interval. This will 
//  /// significantly import the beta reconstruction for particles
//  /// which hit the TOF within this timing window.
//  ///
//  /// If a timing offset is set, this will be added
//  #[getter]
//  fn get_event_t0(&self) -> f32 {
//    self.hit.get_t0()
//  }
//
//  #[getter]
//  fn get_obeys_causality(&self) -> bool {
//    self.hit.obeys_causality()
//  }
//
//  #[getter]
//  fn get_coax_cbl_time(&self) -> f32 {
//    self.hit.coax_cable_time
//  }
//
//  #[getter]
//  fn get_hart_cbl_time(&self) -> f32 {
//    self.hit.hart_cable_time
//  }
//
//  /// Reconstructed particle interaction time,
//  /// calculated from the waveforms of the two
//  /// different paddle ends. If the paddle has 
//  /// been set, this takes phase and cable 
//  /// length into account
//  #[getter]
//  fn t0(&self) -> f32 {
//    self.hit.get_t0()
//  }
//
//  #[getter]
//  fn get_phase_delay(&self) -> f32 {
//    self.hit.get_phase_delay()
//  }
//
//  #[getter]
//  fn get_cable_delay(&self) -> f32 {
//    self.hit.get_cable_delay()
//  }
//
//  #[getter]
//  fn version(&self) -> ProtocolVersion {
//    self.hit.version
//  }
//
//  #[getter]
//  fn phase(&self) -> f32 {
//    self.hit.phase.to_f32()
//  }
//
//  #[getter]
//  fn baseline_a(&self) -> f32 {
//    self.hit.baseline_a.to_f32()
//  }
//
//  #[getter]
//  fn baseline_a_rms(&self) -> f32 {
//    self.hit.baseline_a_rms.to_f32()
//  }
//  
//  #[getter]
//  fn baseline_b(&self) -> f32 {
//    self.hit.baseline_b.to_f32()
//  }
//
//  #[getter]
//  fn baseline_b_rms(&self) -> f32 {
//    self.hit.baseline_b_rms.to_f32()
//  }
//
//  #[getter]
//  fn peak_a(&self) -> f32 {
//    self.hit.get_peak_a()
//  }
//  
//  #[getter]
//  fn peak_b(&self) -> f32 {
//    self.hit.get_peak_b()
//  }
//  
//  #[getter]
//  fn charge_a(&self) -> f32 {
//    self.hit.get_charge_a()
//  }
//  
//  #[getter]
//  fn charge_b(&self) -> f32 {
//    self.hit.get_charge_b()
//  }
//
//  #[getter]
//  fn time_a(&self) -> f32 {
//    self.hit.get_time_a()
//  }
//  
//  #[getter]
//  fn time_b(&self) -> f32 {
//    self.hit.get_time_b()
//  }
//
//  /// Reconstructed particle interaction position
//  /// along the long axis of the paddle.
//  /// For the other dimensions, there is no information
//  /// about the position.
//  /// Reconstructed with the waveforms of both paddle ends.
//  #[getter]
//  fn pos(&self) -> f32 {
//    self.hit.get_pos()
//  }
// 
//  /// The paddle id (1-160) of the hit paddle
//  #[getter]
//  fn paddle_id(&self) -> u8 {
//    self.hit.paddle_id
//  }
//
//  #[getter]
//  fn edep(&self) -> f32 {
//    self.hit.get_edep()
//  }
//
//  #[getter]
//  fn get_paddle_len(&self) -> f32 {
//    self.hit.paddle_len
//  }
//
//}
//
//#[cfg(feature="pybindings")]
//impl_pythonize_display!(PyTofHit, |s: &PyTofHit| s.hit.to_string());


#[cfg(feature = "random")]
#[test]
fn serialization_tofhit() {
  for _ in 0..100 {
    let mut pos = 0;
    let data = TofHit::from_random();
    let mut test = TofHit::from_bytestream(&data.to_bytestream(),&mut pos).unwrap();
    // Manually zero these fields, since comparison with nan will fail and 
    // from_random did not touch these
    test.paddle_len       = 0.0; 
    test.coax_cable_time  = 0.0; 
    test.hart_cable_time  = 0.0; 
    test.x                = 0.0; 
    test.y                = 0.0; 
    test.z                = 0.0; 
    test.event_t0         = 0.0;
    assert_eq!(pos, TofHit::SIZE);
    assert_eq!(data, test);
  }
}
