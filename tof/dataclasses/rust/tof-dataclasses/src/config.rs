//! Payloads for commands that configure an entity of 
//! the TOF system.
//!

use std::fmt;

use crate::serialization::{
  Serialization,
  SerializationError,
  Packable,
  parse_u8,
  parse_f32
};

use crate::packets::PacketType;

#[cfg(feature="random")]
use crate::FromRandom;
#[cfg(feature="random")]
use rand::Rng;

/// Set preamp voltages
#[derive(Copy, Clone, Debug, PartialEq)]
pub struct PreampBiasConfig {
  pub rb_id   : u8,
  pub biases  : [f32;16]
}

impl PreampBiasConfig {
  pub fn new() -> Self { 
    Self {
      rb_id   : 0,
      biases  : [0.0;16]
    }
  }
}

impl Default for PreampBiasConfig {
  fn default() -> Self {
    Self::new()
  }
}

impl fmt::Display for PreampBiasConfig {
  fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
    //let cc = RBCommand::command_code_to_string(self.command_code);
    let mut repr = String::from("<PreampBiasConfig");
    repr += &(format!("\n  RB ID      : {}", self.rb_id)); 
    repr += "  -- biases per channel:";
    for k in 0..self.biases.len() {
      repr += &(format!("\n    Ch{} : {:.2}", k+1, self.biases[k]));
    }
    write!(f, "{}", repr)
  }
}

impl Packable for PreampBiasConfig {
  const PACKET_TYPE : PacketType = PacketType::PreampBiasConfig;
}

impl Serialization for PreampBiasConfig {
  
  const HEAD : u16 = 0xAAAA;
  const TAIL : u16 = 0x5555;
  const SIZE : usize = 69; // nice! 
  
  fn from_bytestream(stream    : &Vec<u8>, 
                     pos       : &mut usize) 
    -> Result<Self, SerializationError>{
    Self::verify_fixed(stream, pos)?;  
    let mut cfg = PreampBiasConfig::new();
    cfg.rb_id   = parse_u8(stream, pos);
    for k in 0..16 {
      cfg.biases[k] = parse_f32(stream, pos);
    }
    *pos += 2;
    Ok(cfg)
  }
  
  fn to_bytestream(&self) -> Vec<u8> {
    let mut bs = Vec::<u8>::with_capacity(Self::SIZE);
    bs.extend_from_slice(&Self::HEAD.to_le_bytes());
    bs.push(self.rb_id);
    for k in 0..16 {
      bs.extend_from_slice(&self.biases[k].to_le_bytes());
    }
    bs.extend_from_slice(&Self::TAIL.to_le_bytes());
    bs
  }
}

#[cfg(feature = "random")]
impl FromRandom for PreampBiasConfig {
  fn from_random() -> Self {
    let mut cfg  = PreampBiasConfig::new();
    let mut rng      = rand::thread_rng();
    cfg.rb_id  = rng.gen::<u8>();
    for k in 0..16 {
      cfg.biases[k] = rng.gen::<f32>();
    }
    cfg
  }
}

#[cfg(feature = "random")]
#[test]
fn pack_preampbiasconfig() {
  for _ in 0..100 {
    let cfg  = PreampBiasConfig::from_random();
    let test : PreampBiasConfig = cfg.pack().unpack().unwrap();
    assert_eq!(cfg, test);
  }
}


