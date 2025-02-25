//! Payloads for commands that configure an entity of 
//! the TOF system.
//!

use std::fmt;

#[cfg(feature = "pybindings")]
use pyo3::pyclass;

use crate::serialization::{
  Serialization,
  SerializationError,
  Packable,
  parse_bool, 
  parse_u8,
  parse_u16,
  parse_u32,
  parse_f32,
  parse_usize
};

use crate::packets::PacketType;
use crate::events::DataType;
use crate::commands::TofOperationMode;

use crate::events::TriggerType;

cfg_if::cfg_if! {
  if #[cfg(feature = "random")]  {
    use crate::FromRandom;
    use rand::Rng;
  }
}

/// Build Strategy
/// 
#[derive(Debug, Copy, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
#[cfg_attr(feature = "pybindings", pyclass(eq, eq_int))]
pub enum BuildStrategy {
  Unknown,
  Smart,
  /// adjust the number of boards based on nrbes/mtb
  Adaptive,
  /// Same as adaptive, but check if the rb events follow the 
  /// mapping
  AdaptiveThorough,
  /// like adaptive, but add usize to the expected number of boards
  AdaptiveGreedy,
  WaitForNBoards
}

impl fmt::Display for BuildStrategy {
  fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
    let r = serde_json::to_string(self).unwrap_or(
      String::from("N.A. - Invalid BuildStrategy (error)"));
    write!(f, "<BuildStrategy: {}>", r)
  }
}

impl BuildStrategy {
  pub fn to_u8(&self) -> u8 {
    match self {
      BuildStrategy::Unknown => {
        return 0;
      }
      BuildStrategy::Smart => {
        return 100;
      }
      BuildStrategy::Adaptive => {
        return 101;
      }
      BuildStrategy::AdaptiveThorough => {
        return 102;
      }
      BuildStrategy::AdaptiveGreedy => {
        return 1;
      }
      BuildStrategy::WaitForNBoards => {
        return 2;
      }
    }
  }
}

impl From<u8> for BuildStrategy {
  fn from(value: u8) -> Self {
    match value {
      0   => BuildStrategy::Unknown,
      100 => BuildStrategy::Smart,
      101 => BuildStrategy::Adaptive,
      102 => BuildStrategy::AdaptiveThorough,
      1   => BuildStrategy::AdaptiveGreedy,
      2   => BuildStrategy::WaitForNBoards,
      _   => BuildStrategy::Unknown
    }
  }
}

#[cfg(feature = "random")]
impl FromRandom for BuildStrategy {
  
  fn from_random() -> Self {
    let choices = [
      BuildStrategy::Unknown,
      BuildStrategy::Smart,
      BuildStrategy::Adaptive,
      BuildStrategy::AdaptiveThorough,
      BuildStrategy::AdaptiveGreedy,
      BuildStrategy::WaitForNBoards,
    ];
    let mut rng  = rand::thread_rng();
    let idx = rng.gen_range(0..choices.len());
    choices[idx]
  }
}

//////////////////////////////////////////////////

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

/////////////////////////////////////////////////////

#[derive(Copy, Clone, Debug, PartialEq)]
pub struct RBChannelMaskConfig {
  pub rb_id       : u8,
  pub channels      : [bool;9],
}

impl RBChannelMaskConfig {
  pub fn new() -> Self {
    Self {
      rb_id     : 0,
      channels    : [false;9],
    }
  }
}

impl Default for RBChannelMaskConfig {
  fn default() -> Self {
    Self::new()
  }
}

impl fmt::Display for RBChannelMaskConfig {
  fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
    let mut repr = String::from("<RBCHannelMaskConfig");
    repr += &(format!("\n  RB ID      : {}", self.rb_id));
    repr += &(format!("\n Problematic Channels >:( {:?}", self.channels));
    write!(f, "{}", repr)
  }
}

impl Packable for RBChannelMaskConfig {
  const PACKET_TYPE : PacketType = PacketType::RBChannelMaskConfig;
}

impl Serialization for RBChannelMaskConfig {

  const HEAD : u16 = 0xAAAA;
  const TAIL : u16 = 0x5555;
  const SIZE : usize = 14;

  fn from_bytestream(stream     : &Vec<u8>,
                     pos        : &mut usize)
    -> Result<Self, SerializationError>{
      Self::verify_fixed(stream, pos)?;
      let mut cfg = RBChannelMaskConfig::new();
      cfg.rb_id   = parse_u8(stream, pos);
      for k in 0..9 {
        cfg.channels[k] = parse_bool(stream, pos);
      }
      *pos += 2;
      Ok(cfg)
    }

  fn to_bytestream(&self) -> Vec<u8> {
    let mut bs = Vec::<u8>::with_capacity(Self::SIZE);
    bs.extend_from_slice(&Self::HEAD.to_le_bytes());
    bs.push(self.rb_id);
    for k in 0..9 {
      bs.push(self.channels[k] as u8);
    }
    bs.extend_from_slice(&Self::TAIL.to_le_bytes());
    bs
  }
} 

#[cfg(feature = "random")]
impl FromRandom for RBChannelMaskConfig {
  fn from_random() -> Self {
    let mut cfg   = RBChannelMaskConfig::new();
    let mut rng   = rand::thread_rng();
    cfg.rb_id     = rng.gen::<u8>();
    for k in 0..9 {
      cfg.channels[k] = rng.gen::<bool>();
    }
    cfg
  }
}

///////////////////////////////////////////////////////


/// Set ltb thresholds
#[derive(Copy, Clone, Debug, PartialEq)]
pub struct LTBThresholdConfig {
  pub rb_id       : u8,
  pub thresholds  : [f32;3]
}

impl LTBThresholdConfig {
  pub fn new() -> Self {
    Self {
      rb_id       : 0,
      thresholds  : [0.0;3]
    }
  }
}

impl Default for LTBThresholdConfig {
  fn default() -> Self {
    Self::new()
  }
}

impl fmt::Display for LTBThresholdConfig {
  fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
    let mut repr = String::from("<LTBThresholdConfig");
    repr += &(format!("\n  RB ID      : {}", self.rb_id));
    repr += "  -- thresholds per channel:";
    for k in 0..self.thresholds.len() {
      repr += &(format!("\n    Ch{} : {:.3}", k, self.thresholds[k]));
    }
    write!(f, "{}", repr)
  }
}

impl Packable for LTBThresholdConfig {
  const PACKET_TYPE : PacketType = PacketType::LTBThresholdConfig;
}

impl Serialization for LTBThresholdConfig {

  const HEAD : u16 = 0xAAAA;
  const TAIL : u16 = 0x5555;
  const SIZE : usize = 17;

  fn from_bytestream(stream     : &Vec<u8>,
                     pos        : &mut usize)
    -> Result<Self, SerializationError>{
      Self::verify_fixed(stream, pos)?;
      let mut cfg = LTBThresholdConfig::new();
      cfg.rb_id   = parse_u8(stream, pos);
      for k in 0..3 {
        cfg.thresholds[k] = parse_f32(stream, pos);
      }
      *pos += 2;
      Ok(cfg)
    }

  fn to_bytestream(&self) -> Vec<u8> {
    let mut bs = Vec::<u8>::with_capacity(Self::SIZE);
    bs.extend_from_slice(&Self::HEAD.to_le_bytes());
    bs.push(self.rb_id);
    for k in 0..3 {
      bs.extend_from_slice(&self.thresholds[k].to_le_bytes());
    }
    bs.extend_from_slice(&Self::TAIL.to_le_bytes());
    bs
  }
}

#[cfg(feature = "random")]
impl FromRandom for LTBThresholdConfig {
  fn from_random() -> Self {
    let mut cfg   = LTBThresholdConfig::new();
    let mut rng   = rand::thread_rng();
    cfg.rb_id     = rng.gen::<u8>();
    for k in 0..3 {
      cfg.thresholds[k] = rng.gen::<f32>();
    }
    cfg
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

  pub fn new() -> Self {
    Self {
      runid                   : 0,
      is_active               : false,
      nevents                 : 0,
      nseconds                : 0,
      tof_op_mode             : TofOperationMode::Default,
      trigger_poisson_rate    : 0,
      trigger_fixed_rate      : 0,
      data_type               : DataType::Unknown, 
      rb_buff_size            : 0,
    }
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

impl Packable for RunConfig {
  const PACKET_TYPE : PacketType = PacketType::RunConfig;
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



#[derive(Copy, Clone, Debug, PartialEq)]
pub struct TriggerConfig{
  /// When we create the LiftofConfig from 
  /// the TriggerConfig, this allows us to 
  /// deactivate fields, so we would can 
  /// only change a single field
  pub active_fields          : u32,
  /// Shall the gaps trigger use beta?
  pub gaps_trigger_use_beta  : Option<bool>, //1
  pub prescale               : Option<f32>, //4
  pub trigger_type           : Option<TriggerType>, //1 
  pub use_combo_trigger      : Option<bool>,
  pub combo_trigger_type     : Option<TriggerType>,
  pub combo_trigger_prescale : Option<f32>,
  pub trace_suppression      : Option<bool>,
  pub mtb_moni_interval      : Option<u16>,
  pub tiu_ignore_busy        : Option<bool>,
  pub hb_send_interval       : Option<u16>,
}

impl TriggerConfig {
  pub fn new() -> Self { 
    Self {
      active_fields           : 0,
      gaps_trigger_use_beta   : None,
      prescale                : None,
      trigger_type            : None,
      use_combo_trigger       : None,
      combo_trigger_type      : None,
      combo_trigger_prescale  : None,
      trace_suppression       : None,
      mtb_moni_interval       : None,
      tiu_ignore_busy         : None,
      hb_send_interval        : None,
    }
  }

  pub fn set_gaps_trigger_use_beta(&mut self, use_it : bool) {
    self.active_fields |= 1;
    self.gaps_trigger_use_beta = Some(use_it);
  }

  pub fn set_prescale(&mut self, prescale : f32) {
    self.active_fields |= 2;
    self.prescale = Some(prescale);
  }

  pub fn set_trigger_type(&mut self, ttype : TriggerType) {
    self.active_fields |= 4;
    self.trigger_type = Some(ttype);
  }

  pub fn set_use_combo_trigger(&mut self, combo : bool) {
    self.active_fields |= 8;
    self.use_combo_trigger = Some(combo);
  }

  pub fn set_combo_trigger_type(&mut self, ttype : TriggerType) {
    self.active_fields |= 16;
    self.combo_trigger_type = Some(ttype)
  }

  pub fn set_combo_trigger_prescale(&mut self, prescale : f32) {
    self.active_fields |= 32;
    self.combo_trigger_prescale = Some(prescale);
  }

  pub fn set_trace_suppression(&mut self, tsup : bool) {
    self.active_fields |= 64;
    self.trace_suppression = Some(tsup);
  }

  pub fn set_mtb_moni_interval(&mut self, interval : u16) {
    self.active_fields |= 128;
    self.mtb_moni_interval = Some(interval);
  }

  pub fn set_tiu_ignore_busy(&mut self, busy : bool) {
    self.active_fields |= 256;
    self.tiu_ignore_busy = Some(busy);
  }

  pub fn set_hb_send_interval(&mut self, interval : u16) {
    self.active_fields |= 512;
    self.hb_send_interval = Some(interval);
  }
}

impl Default for TriggerConfig {
  fn default() -> Self {
    Self::new()
  }
}

impl fmt::Display for TriggerConfig {
  fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
    let mut repr = String::from("<TriggerConfig: ");
    repr += &(format!("(active fields {:x})", self.active_fields));
    if self. gaps_trigger_use_beta.is_some() {
        repr += &(format!("\n  Beta is used by trigger      : {}", self.gaps_trigger_use_beta.unwrap())); 
    }
    if self. prescale.is_some() {
      repr += &(format!("\n  Prescale           : {:.3}", self.prescale.unwrap()));
    }
    if self.trigger_type.is_some() {
      repr += &(format!("\n  Trigger type       : {}",    self.trigger_type.unwrap()));
    }
    if self.use_combo_trigger.is_some() {
      if self.use_combo_trigger.unwrap() {
        repr += &(format!("\n  -- using combo trigger!"));
      } 
    }
    if self.combo_trigger_prescale.is_some() {
      repr += &(format!("\n  -- -- Combo Prescale     : {:.3}", self.combo_trigger_prescale.unwrap()));
    }
    if self.combo_trigger_type.is_some() { 
      repr += &(format!("\n  -- -- Combo Trigger type : {}",    self.combo_trigger_type.unwrap()));
    }
    if self. trace_suppression.is_some() {
      repr += &(format!("\n  trace_suppression       : {}", self.trace_suppression.unwrap()));
    }
    if self.mtb_moni_interval.is_some() {
      repr += &(format!("\n  mtb_moni_interval       : {}", self.mtb_moni_interval.unwrap()));
    }
    if self.tiu_ignore_busy.is_some() {
      repr += &(format!("\n  tiu_ignore_busy         : {}", self.tiu_ignore_busy.unwrap()));
    }
    if self.hb_send_interval.is_some() {
      repr += &(format!("\n  hb_send_interval        : {}", self.hb_send_interval.unwrap()));
    }
    repr += ">";
    write!(f, "{}", repr)
  }
}

impl Packable for TriggerConfig {
  const PACKET_TYPE : PacketType = PacketType::TriggerConfig;
}

impl Serialization for TriggerConfig {
  
  const HEAD : u16 = 0xAAAA;
  const TAIL : u16 = 0x5555;
  const SIZE : usize = 26; 
  
  fn from_bytestream(stream    : &Vec<u8>, 
                     pos       : &mut usize) 
    -> Result<Self, SerializationError>{
    Self::verify_fixed(stream, pos)?;  
    let mut cfg = TriggerConfig::new();
    cfg.active_fields          = parse_u32(stream, pos);
    cfg.gaps_trigger_use_beta  = Some(parse_bool(stream, pos));
    cfg.prescale               = Some(parse_f32 (stream, pos));
    cfg.trigger_type           = Some(TriggerType::from(parse_u8(stream, pos)));
    cfg.use_combo_trigger      = Some(parse_bool(stream, pos));
    cfg.combo_trigger_type     = Some(TriggerType::from(parse_u8(stream, pos)));
    cfg.combo_trigger_prescale = Some(parse_f32(stream, pos));
    cfg.trace_suppression      = Some(parse_bool(stream, pos));
    cfg.mtb_moni_interval      = Some(parse_u16(stream, pos));
    cfg.tiu_ignore_busy        = Some(parse_bool(stream, pos));
    cfg.hb_send_interval       = Some(parse_u16(stream, pos));
    // disable fields which where not explicitly marked as 
    // active
    if cfg.active_fields & 1 != 1 {
      cfg.gaps_trigger_use_beta = None;
    }
    if cfg.active_fields & 2 != 2 {
      cfg.prescale = None;
    }
    if cfg.active_fields & 4 != 4 {
      cfg.trigger_type = None;
    }
    if cfg.active_fields & 8 != 8 {
      cfg.use_combo_trigger = None;
    }
    if cfg.active_fields & 16 != 16 {
      cfg.combo_trigger_type = None;
    }
    if cfg.active_fields & 32 != 32 {
      cfg.combo_trigger_prescale = None;
    }
    if cfg.active_fields & 64 != 64 {
      cfg.trace_suppression = None;
    }
    if cfg.active_fields & 128 != 128 {
      cfg.mtb_moni_interval = None;
    }
    if cfg.active_fields & 256 != 256 {
      cfg.tiu_ignore_busy   = None;
    }
    if cfg.active_fields & 512 != 512 {
      cfg.hb_send_interval  = None;
    }
    *pos += 2;
    Ok(cfg)
  }

  fn to_bytestream(&self) -> Vec<u8> {
    let mut bs = Vec::<u8>::with_capacity(Self::SIZE);
    bs.extend_from_slice(&Self::HEAD        .to_le_bytes());
    bs.extend_from_slice(&self.active_fields.to_le_bytes());
    bs.push             (self.gaps_trigger_use_beta.unwrap_or(false) as u8);
    bs.extend_from_slice(&self.prescale.unwrap_or(0.0)     .to_le_bytes());
    bs.push             (self.trigger_type.unwrap_or(TriggerType::Unknown)  .to_u8());
    bs.push             (self.use_combo_trigger.unwrap_or(false) as u8);
    bs.push             (self.combo_trigger_type.unwrap_or(TriggerType::Unknown) as u8);
    bs.extend_from_slice(&self.combo_trigger_prescale.unwrap_or(0.0).to_le_bytes());
    bs.push             (self.trace_suppression.unwrap_or(false) as u8);
    bs.extend_from_slice(&self.mtb_moni_interval.unwrap_or(30).to_le_bytes());
    bs.push             (self.tiu_ignore_busy.unwrap_or(false) as u8);
    bs.extend_from_slice(&self.hb_send_interval.unwrap_or(30).to_le_bytes());
    bs.extend_from_slice(&Self::TAIL.to_le_bytes());
    bs
  }
}

#[cfg(feature = "random")]
impl FromRandom for TriggerConfig {
  fn from_random() -> Self {
    let mut cfg                 = TriggerConfig::new();
    let mut rng                 = rand::thread_rng();
    let active_fields           = rng.gen::<u32>();
    cfg.active_fields           = active_fields;
    if active_fields & 1 == 1 {
      cfg.gaps_trigger_use_beta   = Some(rng.gen::<bool>());
    } else {
      cfg.gaps_trigger_use_beta = None;
    }
    if active_fields & 2 == 2 {
      cfg.prescale                = Some(rng.gen::<f32>());
    } else {
      cfg.prescale = None;
    }
    if active_fields & 4 == 4 {
      cfg.trigger_type            = Some(TriggerType::from_random());
    } else {
      cfg.trigger_type = None;
    }
    if active_fields & 8 == 8 {
      cfg.use_combo_trigger       = Some(rng.gen::<bool>());
    } else {
      cfg.use_combo_trigger = None;
    }
    if active_fields & 16 == 16 {
      cfg.combo_trigger_type      = Some(TriggerType::from_random());
    } else {
      cfg.combo_trigger_type = None;
    }
    if active_fields & 32 == 32 {
      cfg.combo_trigger_prescale  = Some(rng.gen::<f32>());
    } else {
      cfg.combo_trigger_prescale = None;
    }
    if active_fields & 64 == 64 {
      cfg.trace_suppression       = Some(rng.gen::<bool>());
    } else {
      cfg.trace_suppression = None;
    }
    if active_fields & 128 == 128 {
      cfg.mtb_moni_interval       = Some(rng.gen::<u16>());
    } else {
      cfg.mtb_moni_interval = None;
    }
    if active_fields & 256 == 256 {
      cfg.tiu_ignore_busy         = Some(rng.gen::<bool>());
    } else {
      cfg.tiu_ignore_busy = None;
    }
    if active_fields & 512 == 512 {
      cfg.hb_send_interval        = Some(rng.gen::<u16>());
    } else {
      cfg.hb_send_interval = None;
    }
    cfg
  }
}

#[derive(Copy, Clone, Debug, PartialEq)]
pub struct TofRunConfig {
  pub active_fields            : u32,
  pub runtime                  : Option<u32>, 
}

impl TofRunConfig {
  pub fn new() -> Self {
    Self {
      active_fields : 0,
      runtime       : None
    }
  }

  pub fn set_runtime(&mut self, runtime : u32) {
    self.active_fields |= 1;
    self.runtime = Some(runtime);
  }
}

impl Default for TofRunConfig {
  fn default() -> Self {
    Self::new()
  }
}

impl fmt::Display for TofRunConfig {
  fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
    let mut repr = String::from("<TofRunConfig: ");
    repr += &(format!("(active fields {:x})", self.active_fields));
    if self.runtime.is_some() {
      repr += &(format!("\n  Run time        : {} [s]", self.runtime.unwrap())); 
    }
    repr += ">";
    write!(f, "{}", repr)
  }
}

impl Packable for TofRunConfig {
  const PACKET_TYPE : PacketType = PacketType::TofRunConfig;
}

impl Serialization for TofRunConfig {
  
  const HEAD : u16   = 0xAAAA;
  const TAIL : u16   = 0x5555;
  const SIZE : usize = 12; 
  
  fn from_bytestream(stream    : &Vec<u8>, 
                     pos       : &mut usize) 
    -> Result<Self, SerializationError>{
    Self::verify_fixed(stream, pos)?;  
    let mut cfg        = TofRunConfig::new();
    cfg.active_fields  = parse_u32(stream, pos);
    cfg.runtime        = Some(parse_u32 (stream, pos));
    // disable fields which where not explicitly marked as 
    // active
    if cfg.active_fields & 1 != 1 {
      cfg.runtime = None;
    }
    *pos += 2;
    Ok(cfg)
  }

  fn to_bytestream(&self) -> Vec<u8> {
    let mut bs = Vec::<u8>::with_capacity(Self::SIZE);
    bs.extend_from_slice(&Self::HEAD        .to_le_bytes());
    bs.extend_from_slice(&self.active_fields.to_le_bytes());
    bs.extend_from_slice(&self.runtime.unwrap_or(0).to_le_bytes());
    bs.extend_from_slice(&Self::TAIL.to_le_bytes());
    bs
  }
}

#[cfg(feature = "random")]
impl FromRandom for TofRunConfig {
  fn from_random() -> Self {
    let mut cfg                 = Self::new();
    let mut rng                 = rand::thread_rng();
    let active_fields           = rng.gen::<u32>();
    cfg.active_fields           = active_fields;
    if active_fields & 1 == 1 {
      cfg.runtime   = Some(rng.gen::<u32>());
    }
    cfg
  }
}

//////////////////////////////////////////

#[derive(Copy, Clone, Debug, PartialEq)]
pub struct TofRBConfig {
  pub active_fields                  : u32,
  pub rb_moni_interval               : Option<u32>, 
  pub pb_moni_every_x                : Option<u32>,
  pub pa_moni_every_x                : Option<u32>,
  pub ltb_moni_every_x               : Option<u32>,
  pub drs_deadtime_instead_fpga_temp : Option<bool>,
}

impl TofRBConfig {
  pub fn new() -> Self {
    Self {
     active_fields                 : 0,
     rb_moni_interval               : None, 
     pb_moni_every_x                : None,
     pa_moni_every_x                : None,
     ltb_moni_every_x               : None,
     drs_deadtime_instead_fpga_temp : None,
    }
  }

  pub fn set_rb_moni_interval(&mut self, interval : u32) {
    self.active_fields |= 1;
    self.rb_moni_interval = Some(interval);
  }
  
  pub fn set_pb_moni_every_x(&mut self, interval : u32) {
    self.active_fields |= 2;
    self.pb_moni_every_x = Some(interval);
  }
  
  pub fn set_pa_moni_every_x(&mut self, interval : u32) {
    self.active_fields |= 4;
    self.pa_moni_every_x = Some(interval);
  }
  
  pub fn set_ltb_moni_every_x(&mut self, interval : u32) {
    self.active_fields |= 8;
    self.ltb_moni_every_x = Some(interval);
  }
  
  pub fn set_drs_deadtime_instead_fpga_temp(&mut self, apply : bool) {
    self.active_fields |= 16;
    self.drs_deadtime_instead_fpga_temp = Some(apply);
  }
}

impl Default for TofRBConfig {
  fn default() -> Self {
    Self::new()
  }
}

impl fmt::Display for TofRBConfig {
  fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
    let mut repr = String::from("<TofRBConfig: ");
    repr += &(format!("(active fields {:x})", self.active_fields));
    if self.rb_moni_interval.is_some() {
      repr += &(format!("\n  RBMoni interval : {} [s]", self.rb_moni_interval.unwrap())); 
    }
    if self.pa_moni_every_x.is_some() {
      repr += &(format!("\n  PAMoni interval : {} [xRBMoni]", self.pa_moni_every_x.unwrap())); 
    }
    if self.pb_moni_every_x.is_some() {
      repr += &(format!("\n  PBMoni interval : {} [xRBMoni]", self.pb_moni_every_x.unwrap())); 
    }
    if self.ltb_moni_every_x.is_some() {
      repr += &(format!("\n  LTBMoni interval : {} [xRBMoni]", self.ltb_moni_every_x.unwrap())); 
    }
    if self.drs_deadtime_instead_fpga_temp.is_some() {
      if self.drs_deadtime_instead_fpga_temp.unwrap() {
        repr += &(format!("\n  -- using the fpga temp field to store drs deadtime values")); 
      } 
    }
    repr += ">";
    write!(f, "{}", repr)
  }
}

impl Packable for TofRBConfig {
  const PACKET_TYPE : PacketType = PacketType::TofRBConfig;
}

impl Serialization for TofRBConfig {
  
  const HEAD : u16   = 0xAAAA;
  const TAIL : u16   = 0x5555;
  const SIZE : usize = 25; 
  
  fn from_bytestream(stream    : &Vec<u8>, 
                     pos       : &mut usize) 
    -> Result<Self, SerializationError>{
    Self::verify_fixed(stream, pos)?;  
    let mut cfg          = Self::new();
    cfg.active_fields    = parse_u32(stream, pos);
    cfg.rb_moni_interval = Some(parse_u32 (stream, pos));
    cfg.pa_moni_every_x = Some(parse_u32 (stream, pos));
    cfg.pb_moni_every_x = Some(parse_u32 (stream, pos));
    cfg.ltb_moni_every_x = Some(parse_u32 (stream, pos));
    cfg.drs_deadtime_instead_fpga_temp = Some(parse_bool (stream, pos));
    // disable fields which where not explicitly marked as 
    // active
    if cfg.active_fields & 1 != 1 {
      cfg.rb_moni_interval = None;
    }
    if cfg.active_fields & 2 != 2 {
      cfg.pa_moni_every_x = None;
    }
    if cfg.active_fields & 4 != 4 {
      cfg.pb_moni_every_x = None;
    }
    if cfg.active_fields & 8 != 8 {
      cfg.ltb_moni_every_x = None;
    }
    if cfg.active_fields & 16 != 16 {
      cfg.drs_deadtime_instead_fpga_temp = None;
    }
    *pos += 2;
    Ok(cfg)
  }

  fn to_bytestream(&self) -> Vec<u8> {
    let mut bs = Vec::<u8>::with_capacity(Self::SIZE);
    bs.extend_from_slice(&Self::HEAD        .to_le_bytes());
    bs.extend_from_slice(&self.active_fields.to_le_bytes());
    bs.extend_from_slice(&self.rb_moni_interval.unwrap_or(0).to_le_bytes());
    bs.extend_from_slice(&self.pa_moni_every_x.unwrap_or(0).to_le_bytes());
    bs.extend_from_slice(&self.pb_moni_every_x.unwrap_or(0).to_le_bytes());
    bs.extend_from_slice(&self.ltb_moni_every_x.unwrap_or(0).to_le_bytes());
    bs.push             (self.drs_deadtime_instead_fpga_temp.unwrap_or(false) as u8);
    bs.extend_from_slice(&Self::TAIL.to_le_bytes());
    bs
  }
}

#[cfg(feature = "random")]
impl FromRandom for TofRBConfig {
  fn from_random() -> Self {
    let mut cfg          = Self::new();
    let mut rng          = rand::thread_rng();
    let active_fields    = rng.gen::<u32>();
    cfg.active_fields    = active_fields;
    if active_fields & 1 == 1 {
      cfg.rb_moni_interval   = Some(rng.gen::<u32>());
    }
    if active_fields & 2 == 2 {
      cfg.pa_moni_every_x   = Some(rng.gen::<u32>());
    }
    if active_fields & 4 == 4 {
      cfg.pb_moni_every_x   = Some(rng.gen::<u32>());
    }
    if active_fields & 8 == 8 {
      cfg.ltb_moni_every_x   = Some(rng.gen::<u32>());
    }
    if active_fields & 16 == 16 {
      cfg.drs_deadtime_instead_fpga_temp  = Some(rng.gen::<bool>());
    }
    cfg
  }
}

/////////////////////////////////////



#[derive(Copy, Clone, Debug, PartialEq)]
pub struct DataPublisherConfig {
  pub active_fields            : u32,
  pub mbytes_per_file          : Option<u16>,
  pub discard_event_fraction   : Option<f32>, 
  pub send_mtb_event_packets   : Option<bool>,
  pub send_rbwaveform_packets  : Option<bool>,
  pub send_rbwf_every_x_event  : Option<u32>,
  pub send_tof_summary_packets : Option<bool>,
  pub send_tof_event_packets   : Option<bool>,
  pub hb_send_interval         : Option<u16>,
}

impl DataPublisherConfig {
  pub fn new() -> Self {
    Self {
      active_fields            : 0,
      mbytes_per_file          : None, 
      discard_event_fraction   : None, 
      send_mtb_event_packets   : None, 
      send_rbwaveform_packets  : None, 
      send_rbwf_every_x_event  : None, 
      send_tof_summary_packets : None, 
      send_tof_event_packets   : None, 
      hb_send_interval         : None, 
    }
  }
      
  pub fn set_mbytes_per_file(&mut self, mbytes : u16) {
    self.active_fields |= 1;
    self.mbytes_per_file = Some(mbytes);
  }

  pub fn set_discard_event_fraction(&mut self, frac : f32) {
    self.active_fields |= 2;
    self.discard_event_fraction = Some(frac);
  }

  pub fn set_send_mtb_event_packets(&mut self, send : bool) {
    self.active_fields |= 4;
    self.send_mtb_event_packets = Some(send);
  }

  pub fn set_send_rbwaveform_packets(&mut self, send : bool) {
    self.active_fields |= 8;
    self.send_rbwaveform_packets = Some(send);
  }

  pub fn set_send_rbwf_every_x_event(&mut self, x : u32) {
    self.active_fields |= 16;
    self.send_rbwf_every_x_event = Some(x);
  }

  pub fn set_send_tof_summary_packets(&mut self, send : bool) {
    self.active_fields |= 32;
    self.send_tof_summary_packets = Some(send);
  }
  
  pub fn send_tof_event_packets(&mut self, send : bool) {
    self.active_fields |= 64;
    self.send_tof_event_packets = Some(send);
  }

  pub fn set_hb_send_interval(&mut self, interval : u16) {
    self.active_fields |= 128;
    self.hb_send_interval = Some(interval);
  }
}

impl Default for DataPublisherConfig {
  fn default() -> Self {
    Self::new()
  }
}

impl fmt::Display for DataPublisherConfig {
  fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
    let mut repr = String::from("<DataPublisherConfig: ");
    repr += &(format!("(active fields {:x})", self.active_fields));
    if self.mbytes_per_file.is_some() {
      repr += &(format!("\n  MBytes/FIle        : {}", self.mbytes_per_file.unwrap())); 
    }
    if self.discard_event_fraction.is_some() {
      repr += &(format!("\n  DIsc. event frac   : {}", self.discard_event_fraction.unwrap())); 
    }
    if self.send_mtb_event_packets.is_some() {
      repr += &(format!("\n  Send MTBPack       : {}", self.send_mtb_event_packets.unwrap())); 
    }
    if self.send_rbwaveform_packets.is_some() {
      repr += &(format!("\n  Send RBWfPack      : {}", self.send_rbwaveform_packets.unwrap())); 
    }
    if self.send_rbwf_every_x_event.is_some() {
      repr += &(format!("\n  RBWf every x event : {}", self.send_rbwf_every_x_event.unwrap())); 
    }
    if self.send_tof_summary_packets.is_some() {
      repr += &(format!("\n  Send TofSum        : {}", self.send_tof_summary_packets.unwrap())); 
    }
    if self.send_tof_event_packets.is_some() {
      repr += &(format!("\n  Send TOfEvent      : {}", self.send_tof_event_packets.unwrap())); 
    }
    if self.hb_send_interval.is_some() {
      repr += &(format!("\n  HeartBeat send int  : {}", self.hb_send_interval.unwrap())); 
    }
    repr += ">";
    write!(f, "{}", repr)
  }
}

impl Packable for DataPublisherConfig {
  const PACKET_TYPE : PacketType = PacketType::DataPublisherConfig;
}

impl Serialization for DataPublisherConfig {
  
  const HEAD : u16 = 0xAAAA;
  const TAIL : u16 = 0x5555;
  const SIZE : usize = 24; 
  
  fn from_bytestream(stream    : &Vec<u8>, 
                     pos       : &mut usize) 
    -> Result<Self, SerializationError>{
    Self::verify_fixed(stream, pos)?;  
    let mut cfg                = DataPublisherConfig::new();
    cfg.active_fields          = parse_u32(stream, pos);
    cfg.mbytes_per_file          = Some(parse_u16 (stream, pos));
    cfg.discard_event_fraction   = Some(parse_f32 (stream, pos));
    cfg.send_mtb_event_packets   = Some(parse_bool(stream, pos));
    cfg.send_rbwaveform_packets  = Some(parse_bool(stream, pos));
    cfg.send_rbwf_every_x_event  = Some(parse_u32 (stream, pos));
    cfg.send_tof_summary_packets = Some(parse_bool(stream, pos));
    cfg.send_tof_event_packets   = Some(parse_bool(stream, pos));
    cfg.hb_send_interval         = Some(parse_u16 (stream, pos));
    // disable fields which where not explicitly marked as 
    // active
    if cfg.active_fields & 1 != 1 {
      cfg.mbytes_per_file = None;
    }
    if cfg.active_fields & 2 != 2 {
      cfg.discard_event_fraction = None;
    }
    if cfg.active_fields & 4 != 4 {
      cfg.send_mtb_event_packets = None;
    }
    if cfg.active_fields & 8 != 8 {
      cfg.send_rbwaveform_packets = None;
    }
    if cfg.active_fields & 16 != 16 {
      cfg.send_rbwf_every_x_event = None;
    }
    if cfg.active_fields & 32 != 32 {
      cfg.send_tof_summary_packets = None;
    }
    if cfg.active_fields & 64 != 64 {
      cfg.send_tof_event_packets = None;
    }
    if cfg.active_fields & 128 != 128 {
      cfg.hb_send_interval = None;
    }
    *pos += 2;
    Ok(cfg)
  }

  fn to_bytestream(&self) -> Vec<u8> {
    let mut bs = Vec::<u8>::with_capacity(Self::SIZE);
    bs.extend_from_slice(&Self::HEAD        .to_le_bytes());
    bs.extend_from_slice(&self.active_fields.to_le_bytes());
    bs.extend_from_slice(&self.mbytes_per_file.unwrap_or(0).to_le_bytes());
    bs.extend_from_slice(&self.discard_event_fraction.unwrap_or(0.0).to_le_bytes());
    bs.push             (self .send_mtb_event_packets.unwrap_or(false)  as u8);
    bs.push             (self .send_rbwaveform_packets.unwrap_or(false) as u8);
    bs.extend_from_slice(&self.send_rbwf_every_x_event.unwrap_or(0).to_le_bytes());
    bs.push             (self.send_tof_summary_packets.unwrap_or(false) as u8);
    bs.push             (self .send_tof_event_packets.unwrap_or(false) as u8);
    bs.extend_from_slice(&self.hb_send_interval.unwrap_or(30).to_le_bytes());
    bs.extend_from_slice(&Self::TAIL.to_le_bytes());
    bs
  }
}

#[cfg(feature = "random")]
impl FromRandom for DataPublisherConfig {
  fn from_random() -> Self {
    let mut cfg                 = DataPublisherConfig::new();
    let mut rng                 = rand::thread_rng();
    let active_fields           = rng.gen::<u32>();
    cfg.active_fields           = active_fields;
    if active_fields & 1 == 1 {
      cfg.mbytes_per_file   = Some(rng.gen::<u16>());
    } else {
      cfg.mbytes_per_file = None;
    }
    if active_fields & 2 == 2 {
      cfg.discard_event_fraction = Some(rng.gen::<f32>());
    } else {
      cfg.discard_event_fraction = None;
    }
    if active_fields & 4 == 4 {
      cfg.send_mtb_event_packets = Some(rng.gen::<bool>());
    } else {
      cfg.send_mtb_event_packets = None;
    }
    if active_fields & 8 == 8 {
      cfg.send_rbwaveform_packets = Some(rng.gen::<bool>());
    } else {
      cfg.send_rbwaveform_packets = None;
    }
    if active_fields & 16 == 16 {
      cfg.send_rbwf_every_x_event = Some(rng.gen::<u32>());
    } else {
      cfg.send_rbwf_every_x_event = None;
    }
    if active_fields & 32 == 32 {
      cfg.send_tof_summary_packets  = Some(rng.gen::<bool>());
    } else {
      cfg.send_tof_summary_packets = None;
    }
    if active_fields & 64 == 64 {
      cfg.send_tof_event_packets       = Some(rng.gen::<bool>());
    } else {
      cfg.send_tof_event_packets = None;
    }
    if active_fields & 128 == 128 {
      cfg.hb_send_interval       = Some(rng.gen::<u16>());
    } else {
      cfg.hb_send_interval = None;
    }
    cfg
  }
}



///Analysis Engine Config
/// Settings to change the configuration of the analysis engine 
/// (pulse extraction)
#[derive(Copy, Clone, Debug, PartialEq)]
pub struct AnalysisEngineConfig{
  pub integration_start : f32, //4
  pub integration_window : f32, //4
  pub pedestal_thresh : f32, //4
  pub pedestal_begin_bin : usize, //8
  pub pedestal_win_bins : usize, //8
  pub use_zscore : bool, //1
  pub find_pks_t_start : f32, //4
  pub find_pks_t_window : f32, //4
  pub min_peak_size : usize, //8
  pub find_pks_thresh : f32, //4
  pub max_peaks : usize, //8
  pub cfd_fraction : f32 //4
}

impl AnalysisEngineConfig {
  pub fn new() -> Self {
    Self {
      integration_start         : 270.0,
      integration_window        : 70.0, 
      pedestal_thresh           : 10.0,
      pedestal_begin_bin        : 10,
      pedestal_win_bins         : 50,
      use_zscore                : false,
      find_pks_t_start          : 270.0,
      find_pks_t_window         : 70.0,
      min_peak_size             : 3,
      find_pks_thresh           : 10.0,
      max_peaks                 : 5, //max peak size?? ask
      cfd_fraction              : 0.2
    }
  }
}

impl Default for AnalysisEngineConfig {
  fn default() -> Self {
    Self::new()
  }
}

impl fmt::Display for AnalysisEngineConfig {
  fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
    let mut repr: String = String::from("<AnalysisEngineConfig");
    repr += &(format!("\n Integration start         : {:.1}", self.integration_start));
    repr += &(format!("\n Integration window        : {:.1}", self.integration_window));
    repr += &(format!("\n Pedestal threshold        : {:.1}", self.pedestal_thresh));
    repr += &(format!("\n Pedestal start bin        : {}", self.pedestal_begin_bin));
    repr += &(format!("\n Pedestal window num. bins : {}", self.pedestal_win_bins));
    repr += &(format!("\n Use zscore?               : {}", self.use_zscore));
    repr += &(format!("\n Peakfinder start time     : {:.1}", self.find_pks_t_start));
    repr += &(format!("\n Peakfinder window         : {:.1}", self.find_pks_t_window));
    repr += &(format!("\n Peakfinder threshold      : {:.1}", self.find_pks_thresh));
    repr += &(format!("\n Min. peak size            : {}", self.min_peak_size));
    repr += &(format!("\n Max num. peaks            : {}", self.max_peaks));
    repr += &(format!("\n CFD fraction              : {:.2}", self.cfd_fraction));
    write!(f, "{}", repr)
  }
}

impl Packable for AnalysisEngineConfig {
  const PACKET_TYPE : PacketType = PacketType::AnalysisEngineConfig;
}

impl Serialization for AnalysisEngineConfig {
  
  const HEAD : u16 = 0xAAAA; //2
  const TAIL : u16 = 0x5555; //2
  const SIZE : usize = 65; //61+2+2 = 65
  
  fn from_bytestream(stream    : &Vec<u8>, 
                     pos       : &mut usize) 
    -> Result<Self, SerializationError>{
    Self::verify_fixed(stream, pos)?;  
    let mut cfg: AnalysisEngineConfig = AnalysisEngineConfig::new();
      cfg.integration_start = parse_f32(stream, pos);
      cfg.integration_window = parse_f32(stream, pos);
      cfg.pedestal_thresh = parse_f32(stream, pos);
      cfg.pedestal_begin_bin = parse_usize(stream, pos);
      cfg.pedestal_win_bins = parse_usize(stream, pos);
      cfg.use_zscore = parse_bool(stream, pos);
      cfg.find_pks_t_start = parse_f32(stream, pos);
      cfg.find_pks_t_window = parse_f32(stream, pos);
      cfg.find_pks_thresh = parse_f32(stream, pos);
      cfg.min_peak_size = parse_usize(stream, pos);
      cfg.max_peaks = parse_usize(stream, pos);
      cfg.cfd_fraction = parse_f32(stream, pos);
    *pos += 2;
    Ok(cfg)
  }

  fn to_bytestream(&self) -> Vec<u8> {
    let mut bs = Vec::<u8>::with_capacity(Self::SIZE);
    bs.extend_from_slice(&Self::HEAD.to_le_bytes());
    bs.extend_from_slice(&self.integration_start.to_le_bytes());
    bs.extend_from_slice(&self.integration_window.to_le_bytes());
    bs.extend_from_slice(&self.pedestal_thresh.to_le_bytes());
    bs.extend_from_slice(&self.pedestal_begin_bin.to_le_bytes());
    bs.extend_from_slice(&self.pedestal_win_bins.to_le_bytes());
    bs.push(self.use_zscore as u8);
    bs.extend_from_slice(&self.find_pks_t_start.to_le_bytes());
    bs.extend_from_slice(&self.find_pks_t_window.to_le_bytes());
    bs.extend_from_slice(&self.find_pks_thresh.to_le_bytes());
    bs.extend_from_slice(&self.min_peak_size.to_le_bytes());
    bs.extend_from_slice(&self.max_peaks.to_le_bytes());
    bs.extend_from_slice(&self.cfd_fraction.to_le_bytes());
    bs.extend_from_slice(&Self::TAIL.to_le_bytes());
    bs
  }
}

#[cfg(feature = "random")]
impl FromRandom for AnalysisEngineConfig {
  fn from_random() -> Self {
    let mut cfg  = AnalysisEngineConfig::new();
    let mut rng      = rand::thread_rng();
    cfg.integration_start = rng.gen::<f32>();
    cfg.integration_window = rng.gen::<f32>();
    cfg.pedestal_thresh = rng.gen::<f32>();
    cfg.pedestal_begin_bin = rng.gen::<usize>();
    cfg.pedestal_win_bins = rng.gen::<usize>();
    cfg.use_zscore = rng.gen::<bool>();
    cfg.find_pks_t_start = rng.gen::<f32>();
    cfg.find_pks_t_window = rng.gen::<f32>();
    cfg.find_pks_thresh = rng.gen::<f32>();
    cfg.min_peak_size = rng.gen::<usize>();
    cfg.max_peaks = rng.gen::<usize>();
    cfg.cfd_fraction = rng.gen::<f32>();
    cfg
  }
}

#[cfg(feature = "random")]
#[test]
fn pack_analysisengineconfig() {
  for _ in 0..100 {
    let cfg  = AnalysisEngineConfig::from_random();
    let test : AnalysisEngineConfig = cfg.pack().unpack().unwrap();
    assert_eq!(cfg, test);
  }
}

/// TOF Event Builder Settings
/// Configuring the TOF event builder during flight
/// If a setting is set to None, it will keep the 
/// previous setting
#[derive(Copy, Clone, Debug, PartialEq)]
pub struct TOFEventBuilderConfig{
  pub active_fields    : u32, // supports up to 32 active components
  pub cachesize        : Option<u32>, 
  pub n_mte_per_loop   : Option<u32>, 
  pub n_rbe_per_loop   : Option<u32>, 
  pub te_timeout_sec   : Option<u32>, 
  pub sort_events      : Option<bool>,
  pub build_strategy   : Option<BuildStrategy>, 
  pub wait_nrb         : Option<u8>, 
  pub greediness       : Option<u8>, 
  pub hb_send_interval : Option<u16>,
  // NEW - mark events as not to be sent!
  pub only_save_interesting : Option<bool>,
  pub thr_n_hits_umb        : Option<u8>,
  pub thr_n_hits_cbe        : Option<u8>,
  pub thr_n_hits_cor        : Option<u8>,
  pub thr_tot_edep_umb      : Option<f32>,
  pub thr_tot_edep_cbe      : Option<f32>,
  pub thr_tot_edep_cor      : Option<f32>

}

impl TOFEventBuilderConfig {
  pub fn new() -> Self { 
    Self {
      active_fields         : 0,
      cachesize             : None,
      n_mte_per_loop        : None,
      n_rbe_per_loop        : None,
      te_timeout_sec        : None,
      sort_events           : None,
      build_strategy        : None,
      wait_nrb              : None, 
      greediness            : None,  
      hb_send_interval      : None,
      only_save_interesting : None,
      thr_n_hits_umb        : None,
      thr_n_hits_cbe        : None,
      thr_n_hits_cor        : None,
      thr_tot_edep_umb      : None,
      thr_tot_edep_cbe      : None,
      thr_tot_edep_cor      : None,
    }
  }
      
  pub fn set_cachesize(&mut self, csize : u32) {
    self.active_fields |= 1;
    self.cachesize = Some(csize);
  }
  
  pub fn set_n_mte_per_loop(&mut self, n : u32) {
    self.active_fields |= 2;
    self.n_mte_per_loop = Some(n);
  }

  pub fn set_n_rbe_per_loop(&mut self, n : u32) {
    self.active_fields |= 4;
    self.n_rbe_per_loop = Some(n);
  }

  pub fn set_te_timeout_sec(&mut self, te : u32) {
    self.active_fields |= 8;
    self.te_timeout_sec = Some(te);
  }

  pub fn set_sort_events(&mut self, sort : bool) {
    self.active_fields |= 16;
    self.sort_events = Some(sort);
  }

  pub fn set_build_strategy(&mut self, bs : BuildStrategy) {
    self.active_fields |= 32;
    self.build_strategy = Some(bs);
  }

  pub fn set_wait_nrb(&mut self, nrb : u8) {
    self.active_fields |= 64;
    self.wait_nrb = Some(nrb);
  }

  pub fn set_greediness(&mut self, greed : u8) {
    self.active_fields |= 128;
    self.greediness = Some(greed);
  }

  pub fn set_hb_send_interval(&mut self, interval : u16) {
    self.active_fields |= 256;
    self.hb_send_interval = Some(interval);
  }

  pub fn set_only_save_interesting(&mut self, do_it : bool) {
    self.active_fields |= 512;
    self.only_save_interesting = Some(do_it);
  }

  pub fn thr_n_hits_umb(&mut self, nhit : u8) {
    self.active_fields |= 1024;
    self.thr_n_hits_umb = Some(nhit);
  }
  
  pub fn thr_n_hits_cbe(&mut self, nhit : u8) {
    self.active_fields |= 2048;
    self.thr_n_hits_cbe = Some(nhit);
  }
  
  pub fn thr_n_hits_cor(&mut self, nhit : u8) {
    self.active_fields |= 2u32.pow(12);
    self.thr_n_hits_cor = Some(nhit);
  }
  
  pub fn thr_tot_edep_umb(&mut self, thr : f32) {
    self.active_fields |= 2u32.pow(13);
    self.thr_tot_edep_umb = Some(thr);
  }
  
  pub fn thr_tot_edep_cbe(&mut self, thr : f32) {
    self.active_fields |= 2u32.pow(14);
    self.thr_tot_edep_cbe = Some(thr);
  }
  
  pub fn thr_tot_edep_cor(&mut self, thr : f32) {
    self.active_fields |= 2u32.pow(15);
    self.thr_tot_edep_cor = Some(thr);
  }
}

impl Default for TOFEventBuilderConfig {
  fn default() -> Self {
    Self::new()
  }
}

impl fmt::Display for TOFEventBuilderConfig {
  fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
    let mut repr = String::from("<TOFEventBuilderConfig");
    repr += &(format!(" (active_fields {:x}", self.active_fields)); 
    if self.cachesize.is_some() {
      repr += &(format!("\n Cache size                              : {}", self.cachesize.unwrap())); 
    }
    if self.n_mte_per_loop.is_some() {
      repr += &(format!("\n Num. master trigger events per loop     : {}", self.n_mte_per_loop.unwrap()));
    }
    if self.n_rbe_per_loop.is_some() {
      repr += &(format!("\n Num. readout board events per loop      : {}", self.n_rbe_per_loop.unwrap()));
    }
    if self.te_timeout_sec.is_some() {
      repr += &(format!("\n TOF Event timeout window [sec]          : {:.3}", self.te_timeout_sec.unwrap()));
    }
    if self.sort_events.is_some() {
      repr += &(format!("\n Sort events by ID (high resource load!) : {}", self.sort_events.unwrap()));
    }
    if self.build_strategy.is_some() {
      repr += &(format!("\n Build strategy                          : {}", self.build_strategy.unwrap()));
      if self.build_strategy.unwrap() == BuildStrategy::AdaptiveGreedy {
        if self.greediness.is_some() {
          repr += &(format!("\n Additional RBs considered (greediness)  : {}", self.greediness.unwrap()));
        }
      } else if self.build_strategy.unwrap() == BuildStrategy::WaitForNBoards {
        if self.wait_nrb.is_some() {
          repr += &(format!("\n Waiting for {} boards", self.wait_nrb.unwrap()))
        }
      }
    }
    if self.hb_send_interval.is_some() {
      repr += &(format!("\n Heartbeat send interval : {}", self.hb_send_interval.unwrap()));
    }
    if self.only_save_interesting.is_some() {
      repr += &(format!("\n Saving only interesting events : {}", self.only_save_interesting.unwrap()));
    }
    if self.thr_n_hits_umb.is_some() {
      repr += &(format!("\n Interesting threshold for nhit umb : {}", self.thr_n_hits_umb.unwrap()));
    }
    if self.thr_n_hits_cbe.is_some() {
      repr += &(format!("\n Interesting threshold for nhit cbe : {}", self.thr_n_hits_cbe.unwrap()));
    }
    if self.thr_n_hits_cor.is_some() {
      repr += &(format!("\n Interesting threshold for nhit cor : {}", self.thr_n_hits_cor.unwrap()));
    }
    if self.thr_tot_edep_umb.is_some() {
      repr += &(format!("\n Interesting threshold for tot edep umb : {}", self.thr_tot_edep_umb.unwrap()));
    }
    if self.thr_tot_edep_cbe.is_some() {
      repr += &(format!("\n Interesting threshold for tot edep cbe : {}", self.thr_tot_edep_cbe.unwrap()));
    }
    if self.thr_tot_edep_cor.is_some() {
      repr += &(format!("\n Interesting threshold for tot edep cor : {}", self.thr_tot_edep_cor.unwrap()));
    }
    write!(f, "{}", repr)
  }
}

impl Packable for TOFEventBuilderConfig {
  const PACKET_TYPE : PacketType = PacketType::TOFEventBuilderConfig;
}

impl Serialization for TOFEventBuilderConfig {
  
  const HEAD : u16 = 0xAAAA;
  const TAIL : u16 = 0x5555;
  const SIZE : usize = 46; 
  
  fn from_bytestream(stream    : &Vec<u8>, 
                     pos       : &mut usize) 
    -> Result<Self, SerializationError> {
    Self::verify_fixed(stream, pos)?;  
    let mut cfg = TOFEventBuilderConfig::new();
    cfg.active_fields    = parse_u32(stream, pos);
    cfg.cachesize        = Some(parse_u32(stream, pos));
    cfg.n_mte_per_loop   = Some(parse_u32(stream, pos));
    cfg.n_rbe_per_loop   = Some(parse_u32(stream, pos));
    cfg.te_timeout_sec   = Some(parse_u32(stream, pos));
    cfg.sort_events      = Some(parse_bool(stream, pos));
    cfg.build_strategy   = Some(BuildStrategy::from(parse_u8(stream, pos)));
    cfg.wait_nrb         = Some(parse_u8(stream, pos));
    cfg.greediness       = Some(parse_u8(stream, pos));
    cfg.hb_send_interval = Some(parse_u16(stream, pos));
    // new stuff
    cfg.only_save_interesting = Some(parse_bool(stream, pos));
    cfg.thr_n_hits_umb = Some(parse_u8(stream, pos));
    cfg.thr_n_hits_cbe = Some(parse_u8(stream, pos));
    cfg.thr_n_hits_cor = Some(parse_u8(stream, pos));
    cfg.thr_tot_edep_umb = Some(parse_f32(stream, pos));
    cfg.thr_tot_edep_cbe = Some(parse_f32(stream, pos));
    cfg.thr_tot_edep_cor = Some(parse_f32(stream, pos));
    
    if cfg.active_fields & 1 != 1 {
      cfg.cachesize      = None;
    }
    if cfg.active_fields & 2 != 2 {
      cfg.n_mte_per_loop = None;
    }
    if cfg.active_fields & 4 != 4 {
      cfg.n_rbe_per_loop = None;
    }
    if cfg.active_fields & 8 != 8 {
      cfg.te_timeout_sec = None;
    }
    if cfg.active_fields & 16 != 16 {
      cfg.sort_events    = None;
    }
    if cfg.active_fields & 32 != 32 {
      cfg.build_strategy = None;
    }
    if cfg.active_fields & 64 != 64 {
      cfg.wait_nrb       = None;
    }
    if cfg.active_fields & 128 != 128 {
      cfg.greediness     = None;
    }
    if cfg.active_fields & 256 != 256 {
      cfg.hb_send_interval = None;
    }
    if cfg.active_fields & 512 != 512 {
      cfg.only_save_interesting = None;
    }
    if cfg.active_fields & 1024 != 1024 {
      cfg.thr_n_hits_umb = None;
    }
    if cfg.active_fields & 2048 != 2048 {
      cfg.thr_n_hits_cbe = None;
    }
    if cfg.active_fields & 2u32.pow(12) != 2u32.pow(12) {
      cfg.thr_n_hits_cor = None;
    }
    if cfg.active_fields & 2u32.pow(13) != 2u32.pow(13) {
      cfg.thr_tot_edep_umb = None;
    }
    if cfg.active_fields & 2u32.pow(14) != 2u32.pow(14) {
      cfg.thr_tot_edep_cbe = None;
    }
    if cfg.active_fields & 2u32.pow(15) != 2u32.pow(15) {
      cfg.thr_tot_edep_cor = None;
    }
    *pos += 2;
    Ok(cfg)
  }
 
  fn to_bytestream(&self) -> Vec<u8> {
    let mut bs = Vec::<u8>::with_capacity(Self::SIZE);
    bs.extend_from_slice(&Self::HEAD.to_le_bytes());
    bs.extend_from_slice(&self.active_fields.to_le_bytes());
    bs.extend_from_slice(&self.cachesize.unwrap_or(0).to_le_bytes());
    bs.extend_from_slice(&self.n_mte_per_loop.unwrap_or(0).to_le_bytes());
    bs.extend_from_slice(&self.n_rbe_per_loop.unwrap_or(0).to_le_bytes());
    bs.extend_from_slice(&self.te_timeout_sec.unwrap_or(0).to_le_bytes());
    bs.push(self.sort_events.unwrap_or(false) as u8);
    bs.push(self.build_strategy.unwrap_or(BuildStrategy::Unknown).to_u8());
    bs.push(self.wait_nrb.unwrap_or(0));
    bs.push(self.greediness.unwrap_or(0));
    bs.extend_from_slice(&self.hb_send_interval.unwrap_or(0).to_le_bytes());
    // new stuff
    bs.push(self.only_save_interesting.unwrap_or(false) as u8);
    bs.extend_from_slice(&self.thr_n_hits_umb.unwrap_or(0).to_le_bytes());
    bs.extend_from_slice(&self.thr_n_hits_cbe.unwrap_or(0).to_le_bytes());
    bs.extend_from_slice(&self.thr_n_hits_cor.unwrap_or(0).to_le_bytes());
    bs.extend_from_slice(&self.thr_tot_edep_umb.unwrap_or(0.0).to_le_bytes());
    bs.extend_from_slice(&self.thr_tot_edep_cbe.unwrap_or(0.0).to_le_bytes());
    bs.extend_from_slice(&self.thr_tot_edep_cor.unwrap_or(0.0).to_le_bytes());
    bs.extend_from_slice(&Self::TAIL.to_le_bytes());
    bs
  }
}

#[cfg(feature = "random")]
impl FromRandom for TOFEventBuilderConfig {
  fn from_random() -> Self {
    let mut cfg             = TOFEventBuilderConfig::new();
    let mut rng             = rand::thread_rng();
    cfg.active_fields       = rng.gen::<u32>();
    if cfg.active_fields & 1 == 1 {
      cfg.cachesize         = Some(rng.gen::<u32>());
    }
    if cfg.active_fields & 2 == 2 {
      cfg.n_mte_per_loop      = Some(rng.gen::<u32>());
    }
    if cfg.active_fields & 4 == 4 {
      cfg.n_rbe_per_loop      = Some(rng.gen::<u32>());
    }
    if cfg.active_fields & 8 == 8 {
      cfg.te_timeout_sec      = Some(rng.gen::<u32>());
    }
    if cfg.active_fields & 16 == 16 {
      cfg.sort_events         = Some(rng.gen::<bool>());
    }
    if cfg.active_fields & 32 == 32 {
      cfg.build_strategy      = Some(BuildStrategy::from_random());
    }
    if cfg.active_fields & 64 == 64 {
      cfg.wait_nrb = Some(rng.gen::<u8>());
    }
    if cfg.active_fields & 128 == 128 {
      cfg.greediness = Some(rng.gen::<u8>());
    }
    if cfg.active_fields & 256 == 256 {
      cfg.hb_send_interval = Some(rng.gen::<u16>());
    }
    if cfg.active_fields & 512 == 512 {
      cfg.only_save_interesting = Some(rng.gen::<bool>());
    }
    if cfg.active_fields & 1024 == 1024 {
      cfg.thr_n_hits_umb = Some(rng.gen::<u8>());
    }
    if cfg.active_fields & 2048 == 2048 {
      cfg.thr_n_hits_cbe = Some(rng.gen::<u8>());
    }
    if cfg.active_fields & 2u32.pow(12) == 2u32.pow(12) {
      cfg.thr_n_hits_cor = Some(rng.gen::<u8>());
    }
    if cfg.active_fields & 2u32.pow(13) == 2u32.pow(13) {
      cfg.thr_tot_edep_umb = Some(rng.gen::<f32>());
    }
    if cfg.active_fields & 2u32.pow(14) == 2u32.pow(14) {
      cfg.thr_tot_edep_cbe = Some(rng.gen::<f32>());
    }
    if cfg.active_fields & 2u32.pow(15) == 2u32.pow(15) {
      cfg.thr_tot_edep_cor = Some(rng.gen::<f32>());
    }
    cfg
  }
}


//////////////////////////////////////////
// TESTS

#[cfg(feature = "random")]
#[test]
fn pack_preampbiasconfig() {
  for _ in 0..100 {
    let cfg  = PreampBiasConfig::from_random();
    let test : PreampBiasConfig = cfg.pack().unpack().unwrap();
    assert_eq!(cfg, test);
  }
}

#[cfg(feature = "random")]
#[test]
fn pack_rb_channel_mask_config() {
  for _ in 0..100 {
    let cfg   = RBChannelMaskConfig::from_random();
    let test : RBChannelMaskConfig = cfg.pack().unpack().unwrap();
    assert_eq!(cfg, test);
  }
}


#[cfg(feature = "random")]
#[test]
fn pack_ltbthresholdconfig() {
  for _ in 0..100 {
    let cfg   = LTBThresholdConfig::from_random();
    let test : LTBThresholdConfig = cfg.pack().unpack().unwrap();
    assert_eq!(cfg, test);
  }
}

#[cfg(feature = "random")]
#[test]
fn pack_runconfig() {
  for _ in 0..100 {
    let cfg  = RunConfig::from_random();
    let test = cfg.pack().unpack().unwrap();
    //let test = RunConfig::from_bytestream(&cfg.to_bytestream(), &mut 0).unwrap();
    assert_eq!(cfg, test);

    let cfg_json = serde_json::to_string(&cfg).unwrap();
    let test_json 
      = serde_json::from_str::<RunConfig>(&cfg_json).unwrap();
    assert_eq!(cfg, test_json);
  }
}

#[cfg(feature = "random")]
#[test]
fn pack_triggerconfig() {
  for _ in 0..100 {
    let cfg  = TriggerConfig::from_random();
    let test : TriggerConfig = cfg.pack().unpack().unwrap();
    assert_eq!(cfg, test);
  }
}

#[cfg(feature = "random")]
#[test]
fn pack_tofeventbuilderconfig() {
  for _ in 0..100 {
    let cfg  = TOFEventBuilderConfig::from_random();
    let test : TOFEventBuilderConfig = cfg.pack().unpack().unwrap();
    assert_eq!(cfg, test);
  }
}

#[cfg(feature = "random")]
#[test]
fn pack_datapublisherconfig() {
  for _ in 0..100 {
    let cfg  = DataPublisherConfig::from_random();
    let test : DataPublisherConfig = cfg.pack().unpack().unwrap();
    assert_eq!(cfg, test);
  }
}

#[cfg(feature = "random")]
#[test]
fn pack_tofrunconfig() {
  for _ in 0..100 {
    let cfg  = TofRunConfig::from_random();
    let test : TofRunConfig = cfg.pack().unpack().unwrap();
    assert_eq!(cfg, test);
  }
}

#[cfg(feature = "random")]
#[test]
fn pack_tofrbconfig() {
  for _ in 0..100 {
    let cfg  = TofRBConfig::from_random();
    let test : TofRBConfig = cfg.pack().unpack().unwrap();
    assert_eq!(cfg, test);
  }
}

