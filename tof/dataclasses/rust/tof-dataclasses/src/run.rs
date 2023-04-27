use std::fmt;
use crate::serialization::{parse_u16,
                           parse_u32,
                           parse_bool, 
                           Serialization,
                           SerializationError};


/// Parameters for tof runs
///
///
#[derive(Debug, Copy, Clone)]
pub struct RunConfig {
  pub nevents    : u32,
  pub is_active  : bool,
  pub nseconds   : u32,
  pub stream_any : bool,
  pub forced_trigger_poisson  : u32,
  pub forced_trigger_periodic : u32,
  pub vcal       : bool,
  pub tcal       : bool,
  pub noi        : bool
}

impl RunConfig {

  pub const SIZE               : usize = 14; // bytes
  pub const VERSION            : &'static str = "1.0";
  pub const HEAD               : u16  = 43690; //0xAAAA
  pub const TAIL               : u16  = 21845; //0x5555

  pub fn new() -> RunConfig {
    RunConfig {
      nevents    : 0,
      is_active  : false,
      nseconds   : 0,
      stream_any : false,
      forced_trigger_poisson  : 0,
      forced_trigger_periodic : 0,
      vcal       : false,
      tcal       : false,
      noi        : false
    }
  }

  pub fn to_bytestream(&self) -> Vec<u8> {
    let mut stream = Vec::<u8>::with_capacity(RunConfig::SIZE);
    stream.extend_from_slice(&RunConfig::HEAD.to_le_bytes());
    stream.extend_from_slice(&self.  nevents.to_le_bytes());    
    stream.extend_from_slice(&u8::from(self.  is_active).to_le_bytes());
    stream.extend_from_slice(&self.  nseconds.to_le_bytes());
    stream.extend_from_slice(&u8::from(self.  stream_any).to_le_bytes());
    stream.extend_from_slice(&self.forced_trigger_poisson.to_le_bytes());
    stream.extend_from_slice(&self.forced_trigger_periodic.to_le_bytes());
    stream.extend_from_slice(&u8::from(self.  vcal).to_le_bytes());
    stream.extend_from_slice(&u8::from(self.  tcal).to_le_bytes());
    stream.extend_from_slice(&u8::from(self.  noi).to_le_bytes());
    stream.extend_from_slice(&RunConfig::TAIL.to_le_bytes());
    stream
  }
}

impl Serialization for RunConfig {
  
  fn from_bytestream(bytestream : &Vec<u8>,
                     pos        : &mut usize)
    -> Result<Self, SerializationError> {
    let mut pars = RunConfig::new();
    if parse_u16(bytestream, pos) != RunConfig::HEAD {
      return Err(SerializationError::HeadInvalid {});
    }
    pars.nevents    = parse_u32 (bytestream, pos);
    pars.is_active  = parse_bool(bytestream, pos);
    pars.nseconds   = parse_u32 (bytestream, pos);
    pars.stream_any = parse_bool(bytestream, pos);
    pars.forced_trigger_poisson  = parse_u32(bytestream, pos);
    pars.forced_trigger_periodic = parse_u32(bytestream, pos);
    pars.vcal       = parse_bool(bytestream, pos);
    pars.tcal       = parse_bool(bytestream, pos); 
    pars.noi        = parse_bool(bytestream, pos); 
    Ok(pars)
  }
}

impl Default for RunConfig {
  fn default() -> RunConfig {
    RunConfig::new()
  }
}

impl fmt::Display for RunConfig {
  fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
    write!(f, "<RunConfig : active {}>", self.is_active)
  }
}

