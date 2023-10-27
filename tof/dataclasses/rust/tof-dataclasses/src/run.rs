use std::fmt;
use crate::serialization::{parse_u8,
                           parse_u16,
                           parse_u32,
                           parse_bool, 
                           Serialization,
                           SerializationError};
use crate::events::DataType;

#[cfg(feature = "random")] 
use crate::FromRandom;
#[cfg(feature = "random")]
extern crate rand;
#[cfg(feature = "random")]
use rand::Rng;



extern crate serde;
extern crate serde_json;

/// A collection of parameters for tof runs
///
/// * active_channel_mask : 8bit mask (1bit/channel)
///                         for active data channels 
///                         channel in ascending order with 
///                         increasing bit significance.
///
#[derive(Debug, Copy, Clone, PartialEq, serde::Deserialize, serde::Serialize)]
pub struct RunConfig {
  pub nevents                 : u32,
  pub is_active               : bool,
  pub nseconds                : u32,
  pub stream_any              : bool,
  pub trigger_poisson_rate    : u32,
  pub trigger_fixed_rate      : u32,
  pub latch_to_mtb            : bool,
  pub data_type               : DataType,
  pub rb_buff_size            : u16
}

impl RunConfig {

  pub const VERSION            : &'static str = "1.4";

  pub fn new() -> Self {
    Self {
      nevents                 : 0,
      is_active               : false,
      nseconds                : 0,
      stream_any              : false,
      trigger_poisson_rate    : 0,
      trigger_fixed_rate      : 0,
      latch_to_mtb            : false,
      data_type               : DataType::Unknown, 
      rb_buff_size            : 0,
    }
  }

  pub fn set_forever(&mut self) {
    self.nevents = 0;
  }

  pub fn runs_forever(&self) -> bool {
    self.nevents == 0 
  }

  pub fn from_json_serde(config: &str) -> serde_json::Result<Self> {
    let rc = serde_json::from_str(config)?;
    Ok(rc)
  }

  pub fn to_json_serde(&self) -> serde_json::Result<String> {
    let s = serde_json::to_string(self)?;
    Ok(s)
  }
}

impl Serialization for RunConfig {
  const HEAD               : u16   = 43690; //0xAAAA
  const TAIL               : u16   = 21845; //0x5555
  const SIZE               : usize = 26; // bytes including HEADER + FOOTER
  
  fn from_bytestream(bytestream : &Vec<u8>,
                     pos        : &mut usize)
    -> Result<Self, SerializationError> {
    let mut pars = Self::new();
    Self::verify_fixed(bytestream, pos)?;
    pars.nevents    = parse_u32 (bytestream, pos);
    pars.is_active  = parse_bool(bytestream, pos);
    pars.nseconds   = parse_u32 (bytestream, pos);
    pars.stream_any = parse_bool(bytestream, pos);
    pars.trigger_poisson_rate    = parse_u32(bytestream, pos);
    pars.trigger_fixed_rate      = parse_u32(bytestream, pos);
    pars.latch_to_mtb            = parse_bool(bytestream, pos);
    pars.data_type    = DataType::from_u8(&parse_u8(bytestream, pos));
    pars.rb_buff_size = parse_u16(bytestream, pos);
    *pos += 2; // for the tail 
    //_ = parse_u16(bytestream, pos);
    Ok(pars)
  }
  
  fn to_bytestream(&self) -> Vec<u8> {
    let mut stream = Vec::<u8>::with_capacity(Self::SIZE);
    stream.extend_from_slice(&Self::HEAD.to_le_bytes());
    stream.extend_from_slice(&self.  nevents.to_le_bytes());    
    stream.extend_from_slice(&u8::from(self.  is_active).to_le_bytes());
    stream.extend_from_slice(&self.  nseconds.to_le_bytes());
    stream.extend_from_slice(&u8::from(self.  stream_any).to_le_bytes());
    stream.extend_from_slice(&self.trigger_poisson_rate.to_le_bytes());
    stream.extend_from_slice(&self.trigger_fixed_rate.to_le_bytes());
    stream.extend_from_slice(&u8::from(self.latch_to_mtb).to_le_bytes());
    stream.extend_from_slice(&self.data_type.to_u8().to_le_bytes());
    stream.extend_from_slice(&self.rb_buff_size.to_le_bytes());
    stream.extend_from_slice(&Self::TAIL.to_le_bytes());
    stream
  }
}

impl Default for RunConfig {
  fn default() -> Self {
    Self::new()
  }
}

impl fmt::Display for RunConfig {
  fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
    if !self.is_active {
      return write!(f, "<RunConfig -- is_active : false>");
    } else {
      write!(f, 
"<RunConfig -- is_active : true
    nevents     : {}
    nseconds    : {}
    stream any  : {}
    data type   : {}
    tr_poi_rate : {}
    tr_fix_rate : {}
    mtb_latch   : {}
      |--> data channels (ch 9 separate)
    buff size : {} [ev]>",
      self.nevents,
      self.nseconds,
      self.stream_any,
      self.data_type.string_repr(),
      self.trigger_poisson_rate,
      self.trigger_fixed_rate,
      self.latch_to_mtb,
      self.rb_buff_size)
    }
  }
}

#[cfg(feature = "random")]
impl FromRandom for RunConfig {
    
  fn from_random() -> Self {
    let mut cfg = Self::new();
    let mut rng  = rand::thread_rng();
    cfg.nevents                 = rng.gen::<u32>();
    cfg.is_active               = rng.gen::<bool>();
    cfg.nseconds                = rng.gen::<u32>();
    cfg.stream_any              = rng.gen::<bool>();
    cfg.trigger_poisson_rate    = rng.gen::<u32>();
    cfg.trigger_fixed_rate      = rng.gen::<u32>();
    cfg.latch_to_mtb            = rng.gen::<bool>();
    cfg.data_type               = DataType::from_u8(&rng.gen::<u8>());
    cfg.rb_buff_size            = rng.gen::<u16>();
    cfg
  }
}

#[cfg(all(test,feature = "random"))]
fn serialization_runconfig() {
  let cfg  = RunConfig::from_random();
  let test = RunConfig::from_bytestream(&cfg.to_bytestream(), &mut 0).unwrap();
  assert_eq!(cfg, test);

  let cfg_json = RunConfig::to_json_serde(&cfg).unwrap();
  let test_json = RunConfig::from_json_serde(&cfg_json).unwrap();
  assert_eq!(cfg, test_json);
}

