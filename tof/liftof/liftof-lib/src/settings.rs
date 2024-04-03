//! Aggregate settings for the TOF system
//!
//! Control the settings for the C&C server
//! as well as the liftof-clients on the RBs
//!
//! Different sections might represent different
//! threads/aspects of the code
//!

use std::fs::File;
use std::io::{
    Write,
    Read,
};
use std::fmt;

extern crate toml;
//use tof_dataclasses::events::master_trigger::TriggerType;
use tof_dataclasses::events::DataType;
use tof_dataclasses::commands::TofOperationMode;
use tof_dataclasses::run::RunConfig;
//#[cfg(feature = "random")]
//use tof_dataclasses::FromRandom;
use crate::master_trigger::MTBSettings;

use tof_dataclasses::serialization::{
    parse_u8,
    parse_u16,
    parse_u32,
    //parse_bool, 
    Serialization,
    SerializationError
};

/// Readout strategy for RB (onboard) (RAM) memory buffers
#[derive(Clone, Copy, Debug, PartialEq, serde::Serialize, serde::Deserialize)]
pub enum RBBufferStrategy {
  /// Readout and switch the buffers every
  /// x events
  NEvents(u16),
  /// Adapt to the RB rate and readout the buffers
  /// so that we get switch them every X seconds.
  /// (Argument is in seconds
  AdaptToRate(u16),
}

impl fmt::Display for RBBufferStrategy {
  fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
    let r = serde_json::to_string(self).unwrap_or(
      String::from("N.A. - Invalid RBBufferStrategy (error)"));
    write!(f, "<RBBufferStrategy: {}>", r)
  }
}

#[derive(Debug, Copy, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
pub enum BuildStrategy {
  Unknown,
  Smart,
  /// adjust the number of boards based on nrbes/mtb
  Adaptive,
  /// Same as adaptive, but check if the rb events follow the 
  /// mapping
  AdaptiveThorough,
  /// like adaptive, but add usize to the expected number of boards
  AdaptiveGreedy(usize),
  WaitForNBoards(usize)
}

/// Settings for the specific clients on the RBs (liftof-rb)
#[derive(Debug, Copy, Clone, serde::Serialize, serde::Deserialize)]
pub struct RBSettings {
  /// Don't send events if they have issues. Requires
  /// EventStatus::Perfect. This can not work in the
  /// OperationMode RBHighThroughput
  pub only_perfect_events : bool,
  /// Calculate the crc32 sum for each channel and set
  /// the EventStatus flag accordingly.
  pub calc_crc32          : bool,
  /// tof operation mode - either "StreamAny",
  /// "RequestReply" or "RBHighThroughput"
  pub tof_op_mode         : TofOperationMode,
  /// if different from 0, activate RB self trigger
  /// in poisson mode
  pub trigger_poisson_rate    : u32,
  /// if different from 0, activate RB self trigger 
  /// with fixed rate setting
  pub trigger_fixed_rate      : u32,
  ///// if different from 0, activate RB self trigger
  ///// in poisson mode
  //pub trigger_poisson_rate    : u32,
  ///// if different from 0, activate RB self trigger 
  ///// with fixed rate setting
  //pub trigger_fixed_rate      : u32,
  /// Either "Physics" or a calibration related 
  /// data type, e.g. "VoltageCalibration".
  /// <div class="warning">This might get deprecated in a future version!</div>
  pub data_type               : DataType,
  /// This allows for different strategies on how to readout 
  /// the RB buffers. The following refers to the NEvent strategy.
  /// The value when the readout of the RB buffers is triggered.
  /// This number is in size of full events, which correspond to 
  /// 18530 bytes. Maximum buffer size is a bit more than 3000 
  /// events. Smaller buffer allows for a more snappy reaction, 
  /// but might require more CPU resources (on the board)
  /// For RBBufferStrategy::AdaptToRate(k), readout (and switch) the buffers every
  /// k seconds. The size of the buffer will be determined
  /// automatically depending on the rate.
  pub rb_buff_strategy        : RBBufferStrategy,
  /// The general moni interval. Whenever this time in seconds has
  /// passed, the RB will send a RBMoniData packet
  pub rb_moni_interval        : f32,
  /// Powerboard monitoring. Do it every multiple of rb_moni_interval
  pub pb_moni_every_x         : f32,
  /// Preamp monitoring. Do it every multiple of rb_moni_interval
  pub pa_moni_every_x         : f32,
  /// LTB monitoring. Do it every multiple of rb_moni_interval
  pub ltb_moni_every_x        : f32,
}

impl RBSettings {
  pub fn new() -> Self {
    Self {
      only_perfect_events  : false,
      calc_crc32           : false,
      tof_op_mode          : TofOperationMode::Default,
      trigger_fixed_rate   : 0,
      trigger_poisson_rate : 0,
      data_type            : DataType::Physics,
      rb_buff_strategy     : RBBufferStrategy::AdaptToRate(5),
      rb_moni_interval     : 0.0,
      pb_moni_every_x      : 0.0,
      pa_moni_every_x      : 0.0,
      ltb_moni_every_x     : 0.0,
    }
  }

  pub fn get_runconfig(&self) -> RunConfig {
    // missing here - run id, nevents, nseconds,
    //
    let mut rcfg              = RunConfig::new();
    rcfg.is_active            = true;
    rcfg.tof_op_mode          = self.tof_op_mode.clone();
    rcfg.trigger_fixed_rate   = self.trigger_fixed_rate;
    rcfg.trigger_poisson_rate = self.trigger_poisson_rate;
    rcfg.data_type            = self.data_type.clone();
    let buffer_trip : u16;
    match self.rb_buff_strategy {
      RBBufferStrategy::NEvents(buff_size) => {
        buffer_trip = buff_size;
      },
      RBBufferStrategy::AdaptToRate(_) => {
        // For now, let's just set the initial value to
        // 50 FIXME
        buffer_trip = 50;
        //match rate = get_trigger_rate() {
        //  Err(err) {
        //    error!("Unable to obtain trigger rate!");
        //    buffer_trip = 50;
        //  },
        //  Ok(rate) => {
        //    buffer_trip = rate*n_secs as u16;
        //  }
        //}
      }
    }
    rcfg.rb_buff_size         = buffer_trip as u16;
    rcfg
  }
}

impl fmt::Display for RBSettings {
  fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
    let disp = toml::to_string(self).unwrap_or(
      String::from("-- DESERIALIZATION ERROR! --"));
    write!(f, "<RBSettings :\n{}>", disp)
  }
}

impl Default for RBSettings {
  fn default() -> Self {
    Self::new()
  }
}


/// Settings to change the configuration of the analysis engine 
/// (pulse extraction)
#[derive(Debug, Copy, Clone, serde::Serialize, serde::Deserialize)]
pub struct AnalysisEngineSettings {
  /// pulse integration start
  pub integration_start      : f32,
  /// pulse integration window
  pub integration_window     : f32, 
  /// Pedestal threshold
  pub pedestal_thresh        : f32,
  /// Pedestal begin bin
  pub pedestal_begin_bin     : usize,
  /// Pedestal width (bins)
  pub pedestal_win_bins      : usize,
  /// Use a zscore algorithm to find the peaks instead
  /// of Jeff's algorithm
  pub use_zscore             : bool,
  /// Peakfinding start time
  pub find_pks_t_start       : f32,
  /// Peakfinding window
  pub find_pks_t_window      : f32,
  /// Minimum peaksize (bins)
  pub min_peak_size          : usize,
  /// Threshold for peak recognition
  pub find_pks_thresh        : f32,
  /// Max allowed peaks
  pub max_peaks              : usize,
  /// Timing CFG fraction
  pub cfd_fraction           : f32
}

impl AnalysisEngineSettings {
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
      max_peaks                 : 5,
      cfd_fraction              : 0.2
    }
  }
}

impl fmt::Display for AnalysisEngineSettings {
  fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
    let disp = toml::to_string(self).unwrap_or(
      String::from("-- DESERIALIZATION ERROR! --"));
    write!(f, "<AnalysisEngineSettings :\n{}>", disp)
  }
}

impl Default for AnalysisEngineSettings {
  fn default() -> Self {
    Self::new()
  }
}

/// Settings to change the configuration of the TOF Eventbuilder
#[derive(Debug, Copy, Clone, serde::Serialize, serde::Deserialize)]
pub struct TofEventBuilderSettings {
  pub cachesize           : usize,
  pub n_mte_per_loop      : usize,
  pub n_rbe_per_loop      : usize,
  /// The timeout parameter for the TofEvent. If not
  /// complete after this time, send it onwards anyway
  pub te_timeout_sec      : u32,
  pub build_strategy      : BuildStrategy,
}

impl TofEventBuilderSettings {
  pub fn new() -> TofEventBuilderSettings {
    TofEventBuilderSettings {
      cachesize           : 100000,
      n_mte_per_loop      : 1,
      n_rbe_per_loop      : 40,
      te_timeout_sec      : 30,
      build_strategy      : BuildStrategy::WaitForNBoards(40),
    }
  }
}

impl fmt::Display for TofEventBuilderSettings {
  fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
    let disp = toml::to_string(self).unwrap_or(
      String::from("-- DESERIALIZATION ERROR! --"));
    write!(f, "<TofEventBuilderSettings :\n{}>", disp)
  }
}

impl Default for TofEventBuilderSettings {
  fn default() -> Self {
    Self::new()
  }
}

/// Configure data storage and packet publishing
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct DataPublisherSettings {
  /// location to store data on TOF computer
  pub data_dir               : String,
  /// TOFPackets per file. This defines the "length" of 
  /// a subrun. 
  pub packs_per_file         : usize,
  /// The address the flight computer should subscribe 
  /// to to get tof packets
  pub fc_pub_address         : String,
  /// Send TofSummary + RBWaveforms instead of 
  /// TofEvents
  pub send_flight_packets    : bool,
  /// Send also MastertriggerPackets (this should be 
  /// turned off in flight - only useful if 
  /// send_flight_packets is true, otherwise
  /// MTB events will get sent as part of TofEvents
  pub send_mtb_event_packets : bool,
  /// switch off waveform sending (in case of we 
  /// are sending flight packets)
  pub send_rbwaveforms       : bool,
}

impl DataPublisherSettings {
  pub fn new() -> Self {
    Self {
      data_dir                  : String::from(""),
      packs_per_file            : 1000,
      fc_pub_address            : String::from(""),
      send_flight_packets       : false,
      send_mtb_event_packets    : false,
      send_rbwaveforms          : false,
    }
  }
}

impl fmt::Display for DataPublisherSettings {
  fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
    let disp = toml::to_string(self).unwrap_or(
      String::from("-- DESERIALIZATION ERROR! --"));
    write!(f, "<DataPublisherSettings :\n{}>", disp)
  }
}

impl Default for DataPublisherSettings {
  fn default() -> Self {
    Self::new()
  }
}


#[derive(Debug, Clone, serde::Deserialize, serde::Serialize)]
pub struct LiftofSettings {
  /// default location for RBCalibration files
  pub calibration_dir            : String,
  /// default location for the database
  pub db_path                    : String,
  /// Runtime in seconds
  pub runtime_sec                : u64,
  /// The address of the liftof-command & control server
  /// that is the ip address on the RBNetwork which the 
  /// liftof-cc instance runs on 
  pub cc_server_address          : String,   
  /// The address ("tcp://xx.xx.xx.xx:xxxxx") the tof computer should subscribe to 
  /// to get commands from the flight computer
  pub fc_sub_address             : String,
  /// Interval of time that will elapse from a cmd check to the other
  pub cmd_listener_interval_sec  : u64,
  /// The UDP port to be used to get packets from the 
  /// MTB
  pub mtb_address                : String,
  /// The interval (in seconds) to retrive CPUMoniData from 
  /// the TOF CPU
  pub cpu_moni_interval_sec      : u64,
  /// ignore these RB. These RB ids do not exist in the configuration.
  /// Every RB Id > 50 will be ignored by default
  pub rb_ignorelist              : Vec<u8>,
  /// Should TofHits be generated?
  pub run_analysis_engine        : bool,
  /// Settings to control the MTB
  pub mtb_settings               : MTBSettings,
  /// Settings for the TOF event builder
  pub event_builder_settings     : TofEventBuilderSettings,
  /// Settings for the analysis engine
  pub analysis_engine_settings   : AnalysisEngineSettings,
  /// Configure data publshing and saving on local disc
  pub data_publisher_settings    : DataPublisherSettings,
  /// Settings for the individual RBs
  pub rb_settings                : RBSettings,
}

impl LiftofSettings {
  pub fn new() -> Self {
    LiftofSettings {
      calibration_dir           : String::from(""),
      db_path                   : String::from("/home/gaps/config/gaps_flight.db"),
      runtime_sec               : 0,
      cc_server_address         : String::from("tcp://10.0.1.10:42000"),   
      fc_sub_address            : String::from(""),
      cmd_listener_interval_sec : 1,
      mtb_address               : String::from("10.0.1.10:50001"),
      cpu_moni_interval_sec     : 60,
      rb_ignorelist             : Vec::<u8>::new(),
      run_analysis_engine       : true,
      mtb_settings              : MTBSettings::new(),
      event_builder_settings    : TofEventBuilderSettings::new(),
      analysis_engine_settings  : AnalysisEngineSettings::new(),
      data_publisher_settings   : DataPublisherSettings::new(),
      rb_settings               : RBSettings::new(),
    }
  }

  /// Write the settings to a toml file
  pub fn to_toml(&self, mut filename : String) {
    if !filename.ends_with(".toml") {
      filename += ".toml";
    }
    info!("Will write to file {}!", filename);
    match File::create(&filename) {
      Err(err) => {
        error!("Unable to open file {}! {}", filename, err);
      }
      Ok(mut file) => {
        match toml::to_string_pretty(&self) {
          Err(err) => {
            error!("Unable to serialize toml! {err}");
          }
          Ok(toml_string) => {
            match file.write_all(toml_string.as_bytes()) {
              Err(err) => error!("Unable to write to file {}! {}", filename, err),
              Ok(_)    => debug!("Wrote settings to {}!", filename)
            }
          }
        }
      }
    }
  }

  /// Write the settings to a json file
  pub fn to_json(&self, mut filename : String) {
    if !filename.ends_with(".json") {
      filename += ".json";
    }
    info!("Will write to file {}!", filename);
    match File::create(&filename) {
      Err(err) => {
        error!("Unable to open file {}! {}", filename, err);
      }
      Ok(file) => {
        match serde_json::to_writer_pretty(file, &self) {
          Err(err) => {
            error!("Unable to serialize json! {err}");
          }
          Ok(_) => debug!("Wrote settings to {}!", filename)
        }
      }
    }
  }

  pub fn from_toml(filename : String) -> Result<LiftofSettings, SerializationError> {
    match File::open(&filename) {
      Err(err) => {
        error!("Unable to open {}! {}", filename, err);
        return Err(SerializationError::TomlDecodingError);
      }
      Ok(mut file) => {
        let mut toml_string = String::from("");
        match file.read_to_string(&mut toml_string) {
          Err(err) => {
            error!("Unable to read {}! {}", filename, err);
            return Err(SerializationError::TomlDecodingError);
          }
          Ok(_) => {
            match toml::from_str(&toml_string) {
              Err(err) => {
                error!("Can't interpret toml! {}", err);
                return Err(SerializationError::TomlDecodingError);
              }
              Ok(settings) => {
                return Ok(settings);
              }
            }
          }
        }
      }
    }
  }
}

impl fmt::Display for LiftofSettings {
  fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
    let disp = toml::to_string(self).unwrap_or(
      String::from("-- DESERIALIZATION ERROR! --"));
    write!(f, "<LiftofSettings :\n{}>", disp)
  }
}

impl Default for LiftofSettings {
  fn default() -> Self {
    Self::new()
  }
}

/// Readoutboard configuration for a specific run
#[derive(Debug, Copy, Clone, PartialEq, serde::Deserialize, serde::Serialize)]
pub struct LiftofRBConfig {
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

impl LiftofRBConfig {

  pub const VERSION            : &'static str = "1.5";

  pub fn new() -> Self {
    Self {
      nseconds                : 0,
      tof_op_mode             : TofOperationMode::Default,
      trigger_poisson_rate    : 0,
      trigger_fixed_rate      : 0,
      data_type               : DataType::Unknown, 
      rb_buff_size            : 0,
    }
  }
}

impl Serialization for LiftofRBConfig {
  const HEAD               : u16   = 43690; //0xAAAA
  const TAIL               : u16   = 21845; //0x5555
  const SIZE               : usize = 24; // bytes including HEADER + FOOTER
  
  fn from_bytestream(bytestream : &Vec<u8>,
                     pos        : &mut usize)
    -> Result<Self, SerializationError> {
    let mut pars = Self::new();
    Self::verify_fixed(bytestream, pos)?;
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

impl Default for LiftofRBConfig {
  fn default() -> Self {
    Self::new()
  }
}

impl fmt::Display for LiftofRBConfig {
  fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
    write!(f, 
"<LiftofRBConfig -- is_active : true
    nseconds     : {}
    TOF op. mode : {}
    data type    : {}
    tr_poi_rate  : {}
    tr_fix_rate  : {}
    buff size    : {} [events]>",
      self.nseconds,
      self.tof_op_mode,
      self.data_type,
      self.trigger_poisson_rate,
      self.trigger_fixed_rate,
      self.rb_buff_size)
  }
}

//#[cfg(feature = "random")]
//impl FromRandom for LiftofRBConfig {
//    
//  fn from_random() -> Self {
//    let mut cfg = Self::new();
//    let mut rng  = rand::thread_rng();
//    cfg.runid                   = rng.gen::<u32>();
//    cfg.is_active               = rng.gen::<bool>();
//    cfg.nevents                 = rng.gen::<u32>();
//    cfg.nseconds                = rng.gen::<u32>();
//    cfg.tof_op_mode             = TofOperationMode::from_random();
//    cfg.trigger_poisson_rate    = rng.gen::<u32>();
//    cfg.trigger_fixed_rate      = rng.gen::<u32>();
//    cfg.data_type               = DataType::from_random();
//    cfg.rb_buff_size            = rng.gen::<u16>();
//    cfg
//  }
//}
//
//#[cfg(feature = "random")]
//#[test]
//fn serialization_runconfig() {
//  for k in 0..100 {
//    let cfg  = LiftofRBConfig::from_random();
//    let test = LiftofRBConfig::from_bytestream(&cfg.to_bytestream(), &mut 0).unwrap();
//    assert_eq!(cfg, test);
//
//    let cfg_json = serde_json::to_string(&cfg).unwrap();
//    let test_json 
//      = serde_json::from_str::<LiftofRBConfig>(&cfg_json).unwrap();
//    assert_eq!(cfg, test_json);
//  }
//}

