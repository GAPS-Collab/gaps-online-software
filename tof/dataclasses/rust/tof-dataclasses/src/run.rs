use std::error::Error;
use std::fmt;
use crate::serialization::{parse_u8,
                           parse_u16,
                           parse_u32,
                           parse_bool, 
                           Serialization,
                           SerializationError};

use crate::errors::DecodingError;

extern crate json;
use json::JsonValue;

/// A collection of parameters for tof runs
///
/// * active_channel_mask : 8bit mask (1bit/channel)
///                         for active data channels 
///                         channel in ascending order with 
///                         increasing bit significance.
///
#[derive(Debug, Copy, Clone)]
pub struct RunConfig {
  pub nevents                 : u32,
  pub is_active               : bool,
  pub nseconds                : u32,
  pub stream_any              : bool,
  pub forced_trigger_poisson  : u32,
  pub forced_trigger_periodic : u32,
  pub vcal                    : bool,
  pub tcal                    : bool,
  pub noi                     : bool,
  pub active_channel_mask     : u8,
  pub data_format             : u8,
  pub rb_buff_size            : u16
}

impl RunConfig {

  pub const VERSION            : &'static str = "1.1";

  pub fn new() -> RunConfig {
    RunConfig {
      nevents                 : 0,
      is_active               : false,
      nseconds                : 0,
      stream_any              : false,
      forced_trigger_poisson  : 0,
      forced_trigger_periodic : 0,
      vcal                    : false,
      tcal                    : false,
      noi                     : false,
      active_channel_mask     : u8::MAX,
      data_format             : 0,
      rb_buff_size            : 0,
    }
  }

  pub fn set_forever(&mut self) {
    self.nevents = 0;
  }

  pub fn runs_forever(&self) -> bool {
    self.nevents == 0 
  }

  /// Mark a channel as active
  ///
  /// # Arguments
  ///
  /// ch : 1-9 
  pub fn activate_channel(&mut self, ch : u8) -> Result<(), DecodingError> {
    if ch < 1 || ch > 9 {
      error!("Channel id {ch} is invalid!");
      return Err(DecodingError::ChannelOutOfBounds);
    }
    self.active_channel_mask = self.active_channel_mask | u8::pow(ch -1 ,2);
    Ok(())
  }
  
  pub fn deactivate_channel(&mut self, ch : u8) -> Result<(), DecodingError> {
    if ch < 1 || ch > 9 {
      error!("Channel id {ch} is invalid!");
      return Err(DecodingError::ChannelOutOfBounds);
    }
    self.active_channel_mask = self.active_channel_mask & !u8::pow(ch -1,2);
    Ok(())
  }

  pub fn is_active_channel(&self, ch : u8) -> Result<bool, DecodingError> {
    if ch < 1 || ch > 9 {
      error!("Channel id {ch} is invalid!");
      return Err(DecodingError::ChannelOutOfBounds);
    }
    Ok(self.active_channel_mask & u8::pow(ch - 1,2) > 0) 
  }
}

impl Serialization for RunConfig {
  const HEAD               : u16  = 43690; //0xAAAA
  const TAIL               : u16  = 21845; //0x5555
  const SIZE               : usize = 18; // bytes
  
  fn from_bytestream(bytestream : &Vec<u8>,
                     pos        : &mut usize)
    -> Result<Self, SerializationError> {
    let mut pars = Self::new();
    Self::verify_fixed(bytestream, pos)?;
    pars.nevents    = parse_u32 (bytestream, pos);
    pars.is_active  = parse_bool(bytestream, pos);
    pars.nseconds   = parse_u32 (bytestream, pos);
    pars.stream_any = parse_bool(bytestream, pos);
    pars.forced_trigger_poisson  = parse_u32(bytestream, pos);
    pars.forced_trigger_periodic = parse_u32(bytestream, pos);
    pars.vcal       = parse_bool(bytestream, pos);
    pars.tcal       = parse_bool(bytestream, pos); 
    pars.noi        = parse_bool(bytestream, pos); 
    pars.active_channel_mask = parse_u8(bytestream, pos);
    pars.data_format = parse_u8(bytestream, pos);
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
    stream.extend_from_slice(&self.forced_trigger_poisson.to_le_bytes());
    stream.extend_from_slice(&self.forced_trigger_periodic.to_le_bytes());
    stream.extend_from_slice(&u8::from(self.  vcal).to_le_bytes());
    stream.extend_from_slice(&u8::from(self.  tcal).to_le_bytes());
    stream.extend_from_slice(&u8::from(self.  noi).to_le_bytes());
    stream.push(self.active_channel_mask);
    stream.extend_from_slice(&self.data_format.to_le_bytes());
    stream.extend_from_slice(&self.rb_buff_size.to_le_bytes());
    stream.extend_from_slice(&Self::TAIL.to_le_bytes());
    stream
  }

  fn from_json(config : &JsonValue)
    -> Result<RunConfig, Box<dyn Error>> {
    let mut rc = RunConfig::new();
    rc.nevents                 = config["nevents"]                .as_u32 ().ok_or(SerializationError::JsonDecodingError)?; 
    rc.is_active               = config["is_active"]              .as_bool().ok_or(SerializationError::JsonDecodingError)?;
    rc.nseconds                = config["nseconds"]               .as_u32 ().ok_or(SerializationError::JsonDecodingError)?; 
    rc.stream_any              = config["stream_any"]             .as_bool().ok_or(SerializationError::JsonDecodingError)?;
    rc.forced_trigger_poisson  = config["forced_trigger_poisson"] .as_u32 ().ok_or(SerializationError::JsonDecodingError)?; 
    rc.forced_trigger_periodic = config["forced_trigger_periodic"].as_u32 ().ok_or(SerializationError::JsonDecodingError)?; 
    rc.vcal                    = config["vcal"]                   .as_bool().ok_or(SerializationError::JsonDecodingError)?;
    rc.tcal                    = config["tcal"]                   .as_bool().ok_or(SerializationError::JsonDecodingError)?;
    rc.noi                     = config["noi"]                    .as_bool().ok_or(SerializationError::JsonDecodingError)?;
    rc.active_channel_mask     = config["active_channel_mask"]    .as_u8  ().ok_or(SerializationError::JsonDecodingError)?; 
    rc.data_format             = config["data_format"]            .as_u8  ().ok_or(SerializationError::JsonDecodingError)?;
    rc.rb_buff_size            = config["rb_buff_size"]           .as_u16 ().ok_or(SerializationError::JsonDecodingError)?;
    Ok(rc)
  }
}

impl Default for RunConfig {
  fn default() -> Self {
    Self::new()
  }
}

impl fmt::Display for RunConfig {
  fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
    write!(f, "<RunConfig : active {}>", self.is_active)
  }
}

