//! Event strucutures for data reconrded by the tof
//!
//!  These are used internally, and will get
//!  serialized and send over the wire to the
//!  flight computer. 
//!
//!  Find the corresponding C++ dataclasses
//!  in this project

use crate::errors::SerializationError;
use crate::serialization::{parse_u8,
                           parse_u16,
                           parse_u32,
                           Serialization};
use std::time::Instant;
use std::fmt;

#[cfg(feature="random")]
extern crate rand;
#[cfg(feature="random")]
use rand::Rng;

const PADDLE_TIMEOUT : u64 = 30;

/// Representation of analyzed data from a paddle
///
/// Holds the results of waveform analysis for a 
/// paddle, that is the reustl for 2 individual 
/// waveforms from each end of the paddle.
///
/// Thus it is having methods like `get_time_a`
/// where a and be refere to different 
/// paddle ends.
///
///
#[derive(Debug,Copy,Clone, PartialEq)]
pub struct PaddlePacket  {
  
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
  pub timestamp_32    : u32,
  pub timestamp_16    : u16,

  // fields which won't get 
  // serialized
  pub event_id     : u32,
  pub valid        : bool,

  pub creation_time      : Instant,
}

impl fmt::Display for PaddlePacket {
  fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
    write!(f, "<PaddlePacket:
            \t VALID         {},   
            \t time_a        {},   
            \t time_b        {},   
            \t peak_a        {},   
            \t peak_b        {},   
            \t charge_a      {},   
            \t charge_b      {},   
            \t charge_min_i  {},   
            \t pos_across    {},   
            \t t_average     {},   
            \t ctr_etx       {},   
            \t timestamp_32  {},  
            \t timestamp_16  {}>", 
            self.valid,
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
            self.timestamp_32,
            self.timestamp_16)
  }
}

impl Serialization for PaddlePacket {
  
  const HEAD : u16 = 0xAAAA;
  const TAIL : u16 = 0x5555;
  const SIZE : usize = 28; // size in bytes with HEAD and TAIL

  ///! Serialize the packet
  ///
  ///  Not all fields witll get serialized, 
  ///  only the relevant data for the 
  ///  flight computer
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
    bytestream.extend_from_slice(&self.timestamp_32   .to_le_bytes());
    bytestream.extend_from_slice(&self.timestamp_16   .to_le_bytes());
    bytestream.extend_from_slice(&Self::TAIL        .to_le_bytes()); 
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
    pp.paddle_id = parse_u8(stream, pos);
    pp.time_a    = parse_u16(stream, pos);
    pp.time_b    = parse_u16(stream, pos);
    pp.peak_a    = parse_u16(stream, pos);
    pp.peak_b    = parse_u16(stream, pos);
    pp.charge_a  = parse_u16(stream, pos);
    pp.charge_b  = parse_u16(stream, pos);
    pp.charge_min_i  = parse_u16(stream, pos);
    pp.pos_across    = parse_u16(stream, pos);
    pp.t_average     = parse_u16(stream, pos);
    pp.ctr_etx       = parse_u8(stream, pos);
    pp.timestamp_32  = parse_u32(stream, pos);
    pp.timestamp_16  = parse_u16(stream, pos);
    Ok(pp)
  }
}

impl PaddlePacket {

  //pub const PACKETSIZE    : usize = 24;
  // update Feb 2023 - add 4 byte timestamp
  pub const PACKETSIZE    : usize = 28;
  pub const VERSION       : &'static str = "1.2";
  pub const HEAD          : u16  = 61680; //0xF0F0)
  pub const TAIL          : u16  = 3855;

  pub fn new() -> PaddlePacket {
    let creation_time = Instant::now(); 
    PaddlePacket{
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
                  timestamp_32 : 0,
                  timestamp_16 : 0,
                  // non-serialize fields
                  event_id     : 0,
                  valid        : true,
                  creation_time : creation_time
                }

  }

  pub fn invalidate(&mut self) {
    self.valid = false;
  }
  
  pub fn has_timed_out(&self) -> bool {
    return self.age() > PADDLE_TIMEOUT;
  }
 
  pub fn is_valid(&self, use_timeout : bool) -> bool {
    if use_timeout {
      return self.valid && !self.has_timed_out();
    } else {
      return self.valid;
    }
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

  pub fn age(&self) -> u64 {
    self.creation_time.elapsed().as_secs()
  }

  pub fn reset(&mut self) {
    self.paddle_id    =  0;
    self.time_a       =  0;
    self.time_b       =  0;
    self.peak_a       =  0;
    self.peak_b       =  0;
    self.charge_a     =  0;
    self.charge_b     =  0;
    self.charge_min_i =  0;
    self.pos_across   =  0;
    self.t_average    =  0;
    self.ctr_etx      =  0;
    self.timestamp_32 =  0;
    self.timestamp_16 =  0;
    self.event_id     =  0;
    self.valid        =  true;
  }


  #[cfg(feature="random")]
  pub fn from_random() -> PaddlePacket {
    let mut pp  = PaddlePacket::new();
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
    pp.timestamp_32    = rng.gen::<u32>();
    pp.timestamp_16    = rng.gen::<u16>();
    pp
  }
}

#[test]
fn serialization_paddle_packet() {
    let data = PaddlePacket::from_random();
    let test = PaddlePacket::from_bytestream(&data.to_bytestream(), &mut 0).unwrap();
    assert_eq!(data, test);
}
