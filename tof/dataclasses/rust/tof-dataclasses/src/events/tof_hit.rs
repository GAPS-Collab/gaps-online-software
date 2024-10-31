use std::fmt;

use half::f16;

use crate::errors::SerializationError;
use crate::serialization::{
    parse_u8,
    parse_u16,
    parse_f16,
    Serialization
};
use crate::ProtocolVersion;

use crate::constants::{
  C_LIGHT_PADDLE,
  C_LIGHT_CABLE
};

#[cfg(feature="random")]
use rand::Rng;

#[cfg(feature="database")]
use crate::database::Paddle;

///// We will save the values for the peak heigth, time and charge
///// as u16. The calculations yield f32 though. We need to convert
///// them using MIN/MAX and a range
//const MAX_PEAK_HEIGHT      : f32 = 150.0; //mV
////const MIN_PEAK_HEIGHT      : f32 = 0.0;
//const U16TOF32_PEAK_HEIGHT : f32 = MAX_PEAK_HEIGHT/(u16::MAX as f32);
//const F32TOU16_PEAK_HEIGHT : u16 = ((u16::MAX as f32)/MAX_PEAK_HEIGHT) as u16;
//const MAX_PEAK_CHARGE      : f32 = 100.0; 
////const MIN_PEAK_CHARGE      : f32 = 0.0;
//const U16TOF32_PEAK_CHARGE : f32 = MAX_PEAK_CHARGE/(u16::MAX as f32);
//const F32TOU16_PEAK_CHARGE : u16 = ((u16::MAX as f32)/MAX_PEAK_CHARGE) as u16;
//const MAX_PEAK_TIME        : f32 = 500.0;
////const MIN_PEAK_TIME        : f32 = 0.0;
//const U16TOF32_PEAK_TIME   : f32 = MAX_PEAK_TIME/(u16::MAX as f32);
//const F32TOU16_PEAK_TIME   : u16 = ((u16::MAX as f32)/MAX_PEAK_TIME) as u16;
//const U16TOF32_T0          : f32 = MAX_PEAK_TIME/(u16::MAX as f32);
//const F32TOU16_T0          : u16 = ((u16::MAX as f32)/MAX_PEAK_TIME) as u16;
//const U16TOF32_POS_ACROSS  : f32 = 1800.0/(u16::MAX as f32);
//const F32TOU16_POS_ACROSS  : u16 = ((u16::MAX as f32)/1800.0) as u16;
//const U16TOF32_EDEP        : f32 = 180.0/(u16::MAX as f32);
//const F32TOU16_EDEP        : u16 = ((u16::MAX as f32)/100.0) as u16;

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
  p_end_id : {}
  time     : {}
  charge   : {}
  height   : {}>",
            self.paddle_end_id,
            self.time,
            self.charge,
            self.height)
  }
}

/// Comprehensive paddle information
///
/// Results of the (online) waveform analysis
///
/// A and B are the different ends of the paddle
///
#[derive(Debug,Copy,Clone,PartialEq)]
pub struct TofHit {
  
  /// The ID of the paddle in TOF notation
  /// (1-160)
  pub paddle_id      : u8,
  pub time_a         : f16,
  pub time_b         : f16,
  pub peak_a         : f16,
  pub peak_b         : f16,
  pub charge_a       : f16,
  pub charge_b       : f16,
  
  /// The paddle length will not get serialized
  /// and has to be set after the hit has been 
  /// created
  pub paddle_len     : f32,
  /// The Harting cable length to the RB will not get
  /// serialized and has to be set after the hit has been 
  /// created
  pub cable_len      : f32,

  // deprecated values (prior to V1 version)
  pub timestamp32    : u32,
  pub timestamp16    : u16,
  pub ctr_etx        : u8,
  pub charge_min_i   : u16,
  /// Reconstructed particle interaction position
  /// across the paddle
  pub pos_across     : u16,
  /// Reconstructed particle interaction time
  pub t0             : u16,
  
  // new values
  pub reserved       : u8,
  // only 2 bytes of version
  // are used
  pub version        : ProtocolVersion,
  // for now, but we want to use half instead
  pub baseline_a     : f16,
  pub baseline_a_rms : f16,
  pub baseline_b     : f16,
  pub baseline_b_rms : f16,
  // phase of the sine fit
  pub phase          : f16,
  // fields which won't get 
  // serialized
  pub valid          : bool,
  // for debugging purposes
  pub ftime_a        : f32,
  pub ftime_b        : f32,
  pub fpeak_a        : f32,
  pub fpeak_b        : f32,
}

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
    Harting cable length {:.2}
  ** reconstructed interaction
    energy_dep    {:.2}   
    pos_across    {:.2}   
    t0            {:.2}  
  ** V1 variables
    phase (ch9)   {:.4}
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
            self.cable_len,
            self.get_edep(),
            self.get_pos(),
            self.get_t0(),
            self.phase,
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
    bytestream.extend_from_slice(&self.charge_min_i.to_le_bytes()); 
    //bytestream.extend_from_slice(&self.pos_across  .to_le_bytes()); 
    //bytestream.extend_from_slice(&self.t0          .to_le_bytes()); 
    bytestream.extend_from_slice(&self.baseline_a   .to_le_bytes());
    bytestream.extend_from_slice(&self.baseline_a_rms.to_le_bytes());
    // instead of ctr_etx and reserved, we now have phase in V1
    bytestream.extend_from_slice(&self.phase       .to_le_bytes());
    //bytestream.push(self.ctr_etx); 
    //bytestream.extend_from_slice(&self.timestamp32 .to_le_bytes());
    //bytestream.extend_from_slice(&self.timestamp16 .to_le_bytes());
    //bytestream.push(self.reserved);
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
    pp.charge_min_i   = parse_u16(stream, pos);
    pp.baseline_a     = parse_f16(stream, pos);
    pp.baseline_a_rms = parse_f16(stream, pos);
    //pp.time_a        = parse_u16(stream, pos);
    //pp.time_b        = parse_u16(stream, pos);
    //pp.peak_a        = parse_u16(stream, pos);
    //pp.peak_b        = parse_u16(stream, pos);
    //pp.charge_a      = parse_u16(stream, pos);
    //pp.charge_b      = parse_u16(stream, pos);
    //pp.charge_min_i  = parse_u16(stream, pos);
    //pp.pos_across    = parse_u16(stream, pos);
    //pp.t0            = parse_u16(stream, pos);
    let mut phase_vec = Vec::<u8>::new();
    phase_vec.push(parse_u8(stream, pos));
    phase_vec.push(parse_u8(stream, pos));
    pp.phase    = parse_f16(&phase_vec, &mut 0);
    //pp.ctr_etx       = parse_u8(stream, pos);
    //pp.reserved      = parse_u8(stream, pos);
    let version      = ProtocolVersion::from(parse_u8(stream, pos));
    pp.version       = version;
    match pp.version {
      ProtocolVersion::V1 => {
        // in this version we do have phase instead of
        // ctr_etx and reserved
        //let mut phase_vec = Vec::<u8>::new();
        //phase_vec.push(pp.ctr_etx);
        //phase_vec.push(pp.reserved);
        //pp.phase    = parse_f16(&phase_vec, &mut 0);
      }
      _ => ()
    }
    pp.baseline_b      = parse_f16(stream, pos);
    pp.baseline_b_rms  = parse_f16(stream, pos);

    //pp.timestamp32   = parse_u32(stream, pos);
    //pp.timestamp16   = parse_u16(stream, pos);
    *pos += 2; // always have to do this when using verify fixed
    Ok(pp)
  }
}

impl TofHit {

  pub fn new() -> Self {
    Self{
      paddle_id      : 0,
      time_a         : f16::from_f32(0.0),
      time_b         : f16::from_f32(0.0),
      peak_a         : f16::from_f32(0.0),
      peak_b         : f16::from_f32(0.0),
      charge_a       : f16::from_f32(0.0),
      charge_b       : f16::from_f32(0.0),
      paddle_len     : 0.0,
      cable_len      : 0.0,
      
      charge_min_i   : 0,
      // deprecated  
      pos_across     : 0,
      t0             : 0,
      ctr_etx        : 0,
      timestamp32    : 0,
      timestamp16    : 0,
      valid          : true,
      // v1 variables
      version        : ProtocolVersion::V1,
      reserved       : 0,
      baseline_a     : f16::from_f32(0.0),
      baseline_a_rms : f16::from_f32(0.0),
      baseline_b     : f16::from_f32(0.0),
      baseline_b_rms : f16::from_f32(0.0),
      phase          : f16::from_f32(0.0),
      // non-serialize fields
      ftime_a        : 0.0,
      ftime_b        : 0.0,
      fpeak_a        : 0.0,
      fpeak_b        : 0.0,
    }
  }
  
  #[cfg(feature="database")]
  pub fn set_paddle(&mut self, paddle : &Paddle) {
    self.cable_len  = paddle.cable_len;
    self.paddle_len = paddle.length;
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


  // rework the whole getter/setter cluster, since 
  // we switched to f16 instead of our custom 
  // conversion
  
  /// Calculate the position across the paddle from
  /// the two times at the paddle ends
  ///
  /// **This will be measured from the A side**
  pub fn get_pos(&self) -> f32 {
    //(self.time_a.to_f32() - self.get_t0())*C_LIGHT_PADDLE*10.0 // 10 for cm->mm 
    // FIX - we are actually resetting the particle interaction time to 0 for this
    //(self.time_a.to_f32() - self.get_t0())*C_LIGHT_PADDLE*10.0 // 10 for cm->mm
    if self.time_a == self.time_b {
      return 0.5*self.paddle_len;
    }
    if self.time_a < self.time_b {
      // it is closer to A side
      return 0.5*self.paddle_len - (self.time_b.to_f32() - self.time_a.to_f32())*0.5*C_LIGHT_PADDLE*10.0;
      //return (self.time_b.to_f32() - self.time_a.to_f32())*C_LIGHT_PADDLE*10.0; 
    }
    else {
      return self.paddle_len*0.5 + (self.time_a.to_f32() - self.time_b.to_f32())*0.5*C_LIGHT_PADDLE*10.0;
    }
  }

  /// Calculate the interaction time based on the peak timings measured 
  /// at the paddle ends A and B
  ///
  /// That this works, the length of the paddle has to 
  /// be set before (in mm).
  /// This assumes that the cable on both sides of the paddle are 
  /// the same length
  pub fn get_t0(&self) -> f32 {
    0.5*(self.time_a.to_f32() + self.time_b.to_f32() - (self.paddle_len/(10.0*C_LIGHT_PADDLE)) - ((self.cable_len*2.0)/(10.0*C_LIGHT_CABLE)))
  }

  pub fn get_edep(&self) -> f32 {
    0.0
  }

  pub fn get_time_a(&self) -> f32 {
    self.time_a.to_f32()
  }

  pub fn set_time_a(&mut self, t : f32) {
    self.time_a = f16::from_f32(t);
  }

  pub fn get_time_b(&self) -> f32 {
    self.time_b.to_f32()
  }

  pub fn set_time_b(&mut self, t : f32) {
    self.time_b = f16::from_f32(t)
  }

  pub fn get_peak_a(&self) -> f32 {
    self.peak_a.to_f32()
  }

  pub fn set_peak_a(&mut self, p : f32) {
    self.peak_a = f16::from_f32(p)
  }

  pub fn get_peak_b(&self) -> f32 {
    self.peak_b.to_f32()
  }

  pub fn set_peak_b(&mut self, p : f32) {
    self.peak_b = f16::from_f32(p)
  }

  pub fn get_charge_a(&self) -> f32 {
    self.charge_a.to_f32()
  }

  pub fn set_charge_a(&mut self, c : f32) {
    self.charge_a = f16::from_f32(c)
  }

  pub fn get_charge_b(&self) -> f32 {
    self.charge_b.to_f32()
  }

  pub fn set_charge_b(&mut self, c : f32) {
    self.charge_b = f16::from_f32(c)
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

  ////pub fn get_timestamp48(&self) -> u64 {
  ////  ((self.timestamp16 as u64) << 32) | self.timestamp32 as u64
  ////}
  //
  //pub fn set_edep(&mut self, edep : f32) {
  //  if edep >= 100.0 {
  //    self.charge_min_i = u16::MAX;
  //  } else {
  //    self.charge_min_i = F32TOU16_EDEP*(edep.floor() as u16);
  //  }
  //}

  //pub fn get_edep(&self) -> f32 {
  //  self.charge_min_i as f32 * U16TOF32_EDEP
  //}
  //
  //pub fn set_pos_across(&mut self, pa : f32) {
  //  if pa >= 1800.0 {
  //    self.pos_across = u16::MAX;
  //  } else {
  //    self.pos_across = F32TOU16_POS_ACROSS*(pa.floor() as u16);
  //  }
  //}

  //pub fn get_pos_across(&self) -> f32 {
  //  self.pos_across as f32 * U16TOF32_POS_ACROSS
  //}

  //pub fn set_t0(&mut self, t0 : f32) {
  //  if t0 >= MAX_PEAK_TIME {
  //    self.t0 = u16::MAX;
  //  } else {
  //    self.t0 = F32TOU16_T0*(t0.floor() as u16);
  //  }
  //}

  //pub fn get_t0(&self) -> f32 {
  //  self.t0 as f32 * U16TOF32_T0
  //}

  //pub fn set_peak_a(&mut self, peak : f32 ) {
  //  if peak >= MAX_PEAK_HEIGHT {
  //    self.peak_a = u16::MAX;
  //  } else {
  //    self.peak_a = F32TOU16_PEAK_HEIGHT*(peak.floor() as u16); 
  //  }
  //}
  //
  //pub fn get_peak_a(&self) -> f32 {
  //  self.peak_a as f32 * U16TOF32_PEAK_HEIGHT
  //}

  //pub fn set_peak_b(&mut self, peak : f32 ) {
  //  if peak >= MAX_PEAK_HEIGHT {
  //    self.peak_b = u16::MAX;
  //  } else {
  //    self.peak_b = F32TOU16_PEAK_HEIGHT*(peak.floor() as u16); 
  //  }
  //}
  //
  //pub fn get_peak_b(&self) -> f32 {
  //  self.peak_b as f32 * U16TOF32_PEAK_HEIGHT
  //}
  //
  //pub fn set_peak(&mut self, peak : f32, side : usize ) {
  //  assert!(side == 0 || side == 1);
  //  if side == 0 {self.set_peak_a(peak);}
  //  if side == 1 {self.set_peak_b(peak);}
  //}

  //pub fn set_time_a(&mut self, time : f32 ) {
  //  if time >= MAX_PEAK_TIME {
  //    self.time_a = u16::MAX;
  //  } else {
  //    self.time_a = F32TOU16_PEAK_TIME*(time.floor() as u16); 
  //  }
  //}

  //pub fn get_time_a(&self) -> f32 {
  //  self.time_a as f32 * U16TOF32_PEAK_TIME 
  //}

  //pub fn set_time_b(&mut self, time : f32 ) {
  //  if time >= MAX_PEAK_TIME {
  //    self.time_b = u16::MAX;
  //  } else {
  //    self.time_b = F32TOU16_PEAK_TIME*(time.floor() as u16); 
  //  }
  //}
  //
  //pub fn get_time_b(&self) -> f32 {
  //  self.time_b as f32 * U16TOF32_PEAK_TIME 
  //}
  //
  //pub fn set_time(&mut self, time : f32, side : usize ) {
  //  assert!(side == 0 || side == 1);
  //  if side == 0 {self.set_time_a(time);}
  //  if side == 1 {self.set_time_b(time);}
  //}

  //pub fn set_charge_a(&mut self, charge : f32 ) {
  //  if charge >= MAX_PEAK_CHARGE {
  //    self.charge_a = u16::MAX;
  //  } else {
  //    self.charge_a = F32TOU16_PEAK_CHARGE*(charge.floor() as u16); 
  //  }
  //}
  //
  //pub fn get_charge_a(&self) -> f32 {
  //  self.charge_a as f32 * U16TOF32_PEAK_CHARGE
  //}

  //pub fn set_charge_b(&mut self, charge : f32 ) {
  //  if charge >= MAX_PEAK_CHARGE {
  //    self.charge_b = u16::MAX;
  //  } else {
  //    self.charge_b = F32TOU16_PEAK_CHARGE*(charge.floor() as u16); 
  //  }
  //}
  //
  //pub fn get_charge_b(&self) -> f32 {
  //  self.charge_b as f32 * U16TOF32_PEAK_CHARGE
  //}
  //
  //pub fn set_charge(&mut self, charge : f32, side : usize ) {
  //  assert!(side == 0 || side == 1);
  //  if side == 0 {self.set_charge_a(charge);}
  //  if side == 1 {self.set_charge_b(charge);}
  //}


  #[cfg(feature="random")]
  pub fn from_random() -> TofHit {
    let mut pp  = TofHit::new();
    let mut rng = rand::thread_rng();

    pp.paddle_id      = rng.gen::<u8> ();
    pp.time_a         = f16::from_f32(rng.gen::<f32>());
    pp.time_b         = f16::from_f32(rng.gen::<f32>());
    pp.peak_a         = f16::from_f32(rng.gen::<f32>());
    pp.peak_b         = f16::from_f32(rng.gen::<f32>());
    pp.charge_a       = f16::from_f32(rng.gen::<f32>());
    pp.charge_b       = f16::from_f32(rng.gen::<f32>());
    //pp.charge_min_i = rng.gen::<>();
    //pp.pos_across   = rng.gen::<>();
    //pp.t0           = rng.gen::<>();
    //pp.ctr_etx      = rng.gen::<u8>();
    pp.version        = ProtocolVersion::from(rng.gen::<u8>());
    pp.baseline_a     = f16::from_f32(rng.gen::<f32>());
    pp.baseline_a_rms = f16::from_f32(rng.gen::<f32>());
    pp.baseline_b     = f16::from_f32(rng.gen::<f32>());
    pp.baseline_b_rms = f16::from_f32(rng.gen::<f32>());
    pp.phase          = f16::from_f32(rng.gen::<f32>());
    pp
  }
}

#[cfg(feature = "random")]
#[test]
fn serialization_tofhit() {
  for _ in 0..100 {
    let mut pos = 0;
    let data = TofHit::from_random();
    let test = TofHit::from_bytestream(&data.to_bytestream(),&mut pos).unwrap();
    assert_eq!(pos, TofHit::SIZE);
    assert_eq!(data, test);
  }
}
