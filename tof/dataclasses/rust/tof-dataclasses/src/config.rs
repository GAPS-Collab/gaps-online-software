//! Payloads for commands that configure an entity of 
//! the TOF system.
//!

use std::fmt;

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
    extern crate rand;
    use rand::Rng;
  }
}

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

#[cfg(feature = "random")]
#[test]
fn pack_ltbthresholdconfig() {
  for _ in 0..100 {
    let cfg   = LTBThresholdConfig::from_random();
    let test : LTBThresholdConfig = cfg.pack().unpack().unwrap();
    assert_eq!(cfg, test);
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
#[derive(Copy, Clone, Debug, PartialEq)]
pub struct TriggerConfig{
  pub gaps_trigger_use_beta : bool, //1
  pub tiu_emulation_mode : bool, //1
  pub prescale : f32, //4
  pub trigger_type : TriggerType //1 
}

impl TriggerConfig {
  pub fn new() -> Self { 
    Self {
      gaps_trigger_use_beta   : false,
      tiu_emulation_mode  : false,
      prescale : 0.0,
      trigger_type : TriggerType::Unknown
    }
  }
}

impl Default for TriggerConfig {
  fn default() -> Self {
    Self::new()
  }
}

impl fmt::Display for TriggerConfig {
  fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
    let mut repr = String::from("<TriggerConfig");
    repr += &(format!("\n  Beta is used by trigger      : {}", self.gaps_trigger_use_beta)); 
    repr += &(format!("\n  TIU Emulation Mode : {}", self.tiu_emulation_mode));
    repr += &(format!("\n  Prescale : {:.3}", self.prescale));
    repr += &(format!("\n  Trigger type : {}", self.trigger_type));
    write!(f, "{}", repr)
  }
}




impl Packable for TriggerConfig {
  const PACKET_TYPE : PacketType = PacketType::TriggerConfig;
}

impl Serialization for TriggerConfig {
  
  const HEAD : u16 = 0xAAAA;
  const TAIL : u16 = 0x5555;
  const SIZE : usize = 11; 
  
  fn from_bytestream(stream    : &Vec<u8>, 
                     pos       : &mut usize) 
    -> Result<Self, SerializationError>{
    Self::verify_fixed(stream, pos)?;  
    let mut cfg = TriggerConfig::new();
      cfg.gaps_trigger_use_beta = parse_bool(stream, pos);
      cfg.tiu_emulation_mode = parse_bool(stream, pos);
      cfg.prescale = parse_f32 (stream, pos);
      cfg.trigger_type = TriggerType::from(parse_u8(stream, pos));
    
    *pos += 2;
    Ok(cfg)
  }
  fn to_bytestream(&self) -> Vec<u8> {
    let mut bs = Vec::<u8>::with_capacity(Self::SIZE);
    bs.extend_from_slice(&Self::HEAD.to_le_bytes());
    bs.push(self.gaps_trigger_use_beta as u8);
    bs.push(self.tiu_emulation_mode as u8);
    bs.extend_from_slice(&self.prescale.to_le_bytes());
    bs.push(self.trigger_type.to_u8());
    bs.extend_from_slice(&Self::TAIL.to_le_bytes());
    bs
  }
}

#[cfg(feature = "random")]
impl FromRandom for TriggerConfig {
  fn from_random() -> Self {
    let mut cfg  = TriggerConfig::new();
    let mut rng      = rand::thread_rng();
    cfg.gaps_trigger_use_beta  = rng.gen::<bool>();
    cfg.tiu_emulation_mode = rng.gen::<bool>();
    cfg.prescale = rng.gen::<f32>();
    cfg.trigger_type = TriggerType::from_random();
    cfg
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