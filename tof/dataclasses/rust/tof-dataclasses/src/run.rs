//! Provides run configuration and 
//! settings for RBs and Tof CPU
//!
//!
//!
//!

use std::fmt;
use crate::serialization::{
    parse_u8,
    parse_u16,
    parse_u32,
    parse_bool, 
    Serialization,
    SerializationError
};

use crate::events::DataType;
use crate::commands::TofOperationMode;

cfg_if::cfg_if! {
  if #[cfg(feature = "random")]  {
    use crate::FromRandom;
    extern crate rand;
    use rand::Rng;
  }
}

/// Readoutboard configuration for a specific run
#[derive(Debug, Copy, Clone, PartialEq, serde::Deserialize, serde::Serialize)]
pub struct RunConfig {
  /// an unique identifier for this run
  pub runid                   : u32,
  /// start/stop run
  /// <div class="warning">This might get deprecated in a future version!</div>
  pub is_active               : bool,
  /// limit run to number of events
  pub nevents                 : u32,
  /// limit run time to number of seconds
  pub nseconds                : u32,
  /// tof operation mode - either "StreamAny",
  /// "RequestReply" or "RBHighThroughput"
  pub tof_op_mode             : TofOperationMode,
  /// if different from 0, activate RB self trigger
  /// in poisson mode
  pub trigger_poisson_rate    : u32,
  /// if different from 0, activate RB self trigger 
  /// with fixed rate setting
  pub trigger_fixed_rate      : u32,
  /// Either "Physics" or a calibration related 
  /// data type, e.g. "VoltageCalibration".
  /// <div class="warning">This might get deprecated in a future version!</div>
  pub data_type               : DataType,
  /// The value when the readout of the RB buffers is triggered.
  /// This number is in size of full events, which correspond to 
  /// 18530 bytes. Maximum buffer size is a bit more than 3000 
  /// events. Smaller buffer allows for a more snappy reaction, 
  /// but might require more CPU resources (on the board)
  pub rb_buff_size            : u16
}

impl RunConfig {

  pub const VERSION            : &'static str = "1.5";

  pub fn new() -> Self {
    Self {
      runid                   : 0,
      is_active               : false,
      nevents                 : 0,
      nseconds                : 0,
      tof_op_mode             : TofOperationMode::StreamAny,
      trigger_poisson_rate    : 0,
      trigger_fixed_rate      : 0,
      data_type               : DataType::Unknown, 
      rb_buff_size            : 0,
    }
  }

  #[deprecated(since="0.8.3", note="Public argument does not need setter. Semantics need to be clear though.")]
  pub fn set_forever(&mut self) {
    self.nevents = 0;
  }

  #[deprecated(since="0.8.3", note="Public argument does not need setter. Semantics need to be clear though.")]
  pub fn runs_forever(&self) -> bool {
    self.nevents == 0 
  }
}

impl Serialization for RunConfig {
  const HEAD               : u16   = 43690; //0xAAAA
  const TAIL               : u16   = 21845; //0x5555
  const SIZE               : usize = 29; // bytes including HEADER + FOOTER
  
  fn from_bytestream(bytestream : &Vec<u8>,
                     pos        : &mut usize)
    -> Result<Self, SerializationError> {
    let mut pars = Self::new();
    Self::verify_fixed(bytestream, pos)?;
    pars.runid                   = parse_u32 (bytestream, pos);
    pars.is_active               = parse_bool(bytestream, pos);
    pars.nevents                 = parse_u32 (bytestream, pos);
    pars.nseconds                = parse_u32 (bytestream, pos);
    pars.tof_op_mode           
      = TofOperationMode::try_from(
          parse_u8(bytestream, pos))
      .unwrap_or_else(|_| TofOperationMode::Unknown);
    pars.trigger_poisson_rate    = parse_u32 (bytestream, pos);
    pars.trigger_fixed_rate      = parse_u32 (bytestream, pos);
    pars.data_type    
      = DataType::try_from(parse_u8(bytestream, pos))
      .unwrap_or_else(|_| DataType::Unknown);
    pars.rb_buff_size = parse_u16(bytestream, pos);
    *pos += 2; // for the tail 
    //_ = parse_u16(bytestream, pos);
    Ok(pars)
  }
  
  fn to_bytestream(&self) -> Vec<u8> {
    let mut stream = Vec::<u8>::with_capacity(Self::SIZE);
    stream.extend_from_slice(&Self::HEAD.to_le_bytes());
    stream.extend_from_slice(&self.runid.to_le_bytes());
    stream.extend_from_slice(&u8::from(self.  is_active).to_le_bytes());
    stream.extend_from_slice(&self.nevents.to_le_bytes());    
    stream.extend_from_slice(&self.  nseconds.to_le_bytes());
    stream.extend_from_slice(&(self.tof_op_mode as u8).to_le_bytes());
    stream.extend_from_slice(&self.trigger_poisson_rate.to_le_bytes());
    stream.extend_from_slice(&self.trigger_fixed_rate.to_le_bytes());
    stream.extend_from_slice(&(self.data_type as u8).to_le_bytes());
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
    nevents      : {}
    nseconds     : {}
    TOF op. mode : {}
    data type    : {}
    tr_poi_rate  : {}
    tr_fix_rate  : {}
    buff size    : {} [events]>",
      self.nevents,
      self.nseconds,
      self.tof_op_mode,
      self.data_type,
      self.trigger_poisson_rate,
      self.trigger_fixed_rate,
      self.rb_buff_size)
    }
  }
}

#[cfg(feature = "random")]
impl FromRandom for RunConfig {
    
  fn from_random() -> Self {
    let mut cfg = Self::new();
    let mut rng  = rand::thread_rng();
    cfg.runid                   = rng.gen::<u32>();
    cfg.is_active               = rng.gen::<bool>();
    cfg.nevents                 = rng.gen::<u32>();
    cfg.nseconds                = rng.gen::<u32>();
    cfg.tof_op_mode             = TofOperationMode::from_random();
    cfg.trigger_poisson_rate    = rng.gen::<u32>();
    cfg.trigger_fixed_rate      = rng.gen::<u32>();
    cfg.data_type               = DataType::from_random();
    cfg.rb_buff_size            = rng.gen::<u16>();
    cfg
  }
}

#[cfg(feature = "random")]
#[test]
fn serialization_runconfig() {
  for k in 0..100 {
    let cfg  = RunConfig::from_random();
    let test = RunConfig::from_bytestream(&cfg.to_bytestream(), &mut 0).unwrap();
    assert_eq!(cfg, test);

    let cfg_json = serde_json::to_string(&cfg).unwrap();
    let test_json 
      = serde_json::from_str::<RunConfig>(&cfg_json).unwrap();
    assert_eq!(cfg, test_json);
  }
}

