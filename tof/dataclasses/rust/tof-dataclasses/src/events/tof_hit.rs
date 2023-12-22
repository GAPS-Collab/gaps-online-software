use crate::errors::SerializationError;
use crate::serialization::{
    parse_u8,
    parse_u16,
    parse_u32,
    Serialization
};
use std::fmt;

#[cfg(feature="random")]
extern crate rand;
#[cfg(feature="random")]
use rand::Rng;


/// Comprehensive paddle information
///
/// Results of the (online) waveform analysis
///
/// A and B are the different ends of the paddle
///
#[derive(Debug,Copy,Clone,PartialEq)]
pub struct TofHit {
  
  //unsigned short head = 0xF0F0;
  pub paddle_id    : u8,
  pub time_a       : u16,
  pub time_b       : u16,
  pub peak_a       : u16,
  pub peak_b       : u16,
  pub charge_a     : u16,
  pub charge_b     : u16,
  pub charge_min_i : u16,
  pub pos_across   : u16,
  pub t_average    : u16,
  pub ctr_etx      : u8,

  // this might be not needed, 
  // unsure
  pub timestamp32   : u32,
  pub timestamp16   : u16,

  // fields which won't get 
  // serialized
  pub valid        : bool,
}

impl Default for TofHit {
  fn default() -> Self {
    Self::new()
  }
}

impl fmt::Display for TofHit {
  fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
    write!(f, "<TofHit:
  Peak:
    LE Time A/B   {} {}   
    Height  A/B   {} {}
    Charge  A/B   {} {}
  charge_min_i    {}   
  pos_across      {}   
  t_average       {}   
  ctr_etx         {}   
  timestamp32     {}  
  timestamp16     {}
  |-> timestamp48 {}
  VALID           {}>", 
            self.time_a,
            self.time_b,
            self.peak_a,
            self.peak_b,
            self.charge_a,
            self.charge_b,
            self.charge_min_i,
            self.pos_across,
            self.t_average,
            self.ctr_etx,
            self.timestamp32,
            self.timestamp16,
            self.get_timestamp48(),
            self.valid,
            )
  }
}

impl Serialization for TofHit {
  
  const HEAD          : u16  = 61680; //0xF0F0)
  const TAIL          : u16  = 3855;
  const SIZE : usize = 30; // size in bytes with HEAD and TAIL

  /// Serialize the packet
  ///
  /// Not all fields will get serialized, 
  /// only the relevant data for the 
  /// flight computer
  ///
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
    bytestream.extend_from_slice(&self.pos_across  .to_le_bytes()); 
    bytestream.extend_from_slice(&self.t_average   .to_le_bytes()); 
    bytestream.push(self.ctr_etx); 
    bytestream.extend_from_slice(&self.timestamp32 .to_le_bytes());
    bytestream.extend_from_slice(&self.timestamp16 .to_le_bytes());
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
    pp.valid     = true;
    pp.paddle_id     = parse_u8(stream, pos);
    pp.time_a        = parse_u16(stream, pos);
    pp.time_b        = parse_u16(stream, pos);
    pp.peak_a        = parse_u16(stream, pos);
    pp.peak_b        = parse_u16(stream, pos);
    pp.charge_a      = parse_u16(stream, pos);
    pp.charge_b      = parse_u16(stream, pos);
    pp.charge_min_i  = parse_u16(stream, pos);
    pp.pos_across    = parse_u16(stream, pos);
    pp.t_average     = parse_u16(stream, pos);
    pp.ctr_etx       = parse_u8(stream, pos);
    pp.timestamp32   = parse_u32(stream, pos);
    pp.timestamp16   = parse_u16(stream, pos);
    *pos += 2; // always have to do this when using verify fixed
    Ok(pp)
  }
}

impl TofHit {

  // update Feb 2023 - add 4 byte timestamp
  pub const VERSION       : &'static str = "1.2";

  pub fn new() -> Self {
    Self{
         paddle_id    : 0,
         time_a       : 0,
         time_b       : 0,
         peak_a       : 0,
         peak_b       : 0,
         charge_a     : 0,
         charge_b     : 0,
         charge_min_i : 0,
         pos_across   : 0,
         t_average    : 0,
         ctr_etx      : 0,
         timestamp32  : 0,
         timestamp16  : 0,
         // non-serialize fields
         valid        : true,
    }
  }

  pub fn get_timestamp48(&self) -> u64 {
    ((self.timestamp16 as u64) << 32) | self.timestamp32 as u64
  }

  pub fn set_peak_a(&mut self, peak : f32 ) {
    let prec : f64 = 0.004;
    self.peak_a = (peak as f64/prec) as u16;
  }

  pub fn set_peak_b(&mut self, peak : f32 ) {
    let prec : f64 = 0.004;
    self.peak_b = (peak as f64/prec) as u16;
  }
  
  pub fn set_peak(&mut self, peak : f32, side : usize ) {
    assert!(side == 0 || side == 1);
    if side == 0 {self.set_peak_a(peak);}
    if side == 1 {self.set_peak_b(peak);}
  }

  pub fn set_time_a(&mut self, time : f32 ) {
    //println!("time {time}");
    let prec : f64 = 0.004;
    self.time_a = (time as f64/prec) as u16;
    //println!("time_a {}", self.time_a);
  }

  pub fn set_time_b(&mut self, time : f32 ) {
    let prec : f64 = 0.004;
    self.time_b = (time as f64/prec) as u16;
  }
  
  pub fn set_time(&mut self, time : f32, side : usize ) {
    assert!(side == 0 || side == 1);
    if side == 0 {self.set_time_a(time);}
    if side == 1 {self.set_time_b(time);}
  }

  pub fn set_charge_a(&mut self, charge : f32 ) {
    let prec : f64 = 0.004;
    self.charge_a = (charge as f64/prec) as u16;
  }

  pub fn set_charge_b(&mut self, charge : f32 ) {
    let prec : f64 = 0.004;
    self.charge_b = (charge as f64/prec) as u16;
  }
  
  pub fn set_charge(&mut self, charge : f32, side : usize ) {
    assert!(side == 0 || side == 1);
    if side == 0 {self.set_charge_a(charge);}
    if side == 1 {self.set_charge_b(charge);}
  }


  #[cfg(feature="random")]
  pub fn from_random() -> TofHit {
    let mut pp  = TofHit::new();
    let mut rng = rand::thread_rng();

    pp.paddle_id    = rng.gen::<u8> ();
    pp.time_a       = rng.gen::<u16>();
    pp.time_b       = rng.gen::<u16>();
    pp.peak_a       = rng.gen::<u16>();
    pp.peak_b       = rng.gen::<u16>();
    pp.charge_a     = rng.gen::<u16>();
    pp.charge_b     = rng.gen::<u16>();
    pp.charge_min_i = rng.gen::<u16>();
    pp.pos_across   = rng.gen::<u16>();
    pp.t_average    = rng.gen::<u16>();
    pp.ctr_etx      = rng.gen::<u8>();
    pp.timestamp32  = rng.gen::<u32>();
    pp.timestamp16  = rng.gen::<u16>();
    pp
  }
}

#[cfg(feature = "random")]
#[test]
fn serialization_tofhit() {
    let mut pos = 0;
    let data = TofHit::from_random();
    let test = TofHit::from_bytestream(&data.to_bytestream(),&mut pos).unwrap();
    assert_eq!(pos, TofHit::SIZE);
    assert_eq!(data, test);
}
