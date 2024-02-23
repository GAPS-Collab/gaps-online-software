/// Generalized settings for liftof-cc
///
/// Configure it from a .json config 
/// file
///

use std::fs::File;
use std::io::{
    Write,
    Read,
};
use std::fmt;

extern crate toml;
use tof_dataclasses::errors::SerializationError;
use tof_dataclasses::events::master_trigger::TriggerType;

use liftof_lib::master_trigger::MTBSettings;

#[derive(Debug, Copy, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
pub enum BuildStrategy {
  Unknown,
  Smart,
  /// adjust the number of boards based on nrbes/mtb
  Adaptive,
  /// like adaptive, but add usize to the expected number of boards
  AdaptiveGreedy(usize),
  WaitForNBoards(usize)
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
    AnalysisEngineSettings {
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
  pub cachesize         : usize,
  pub n_mte_per_loop    : usize,
  pub n_rbe_per_loop    : usize,
  /// The timeout parameter for the TofEvent. If not
  /// complete after this time, send it onwards anyway
  pub te_timeout_sec    : u32,
  pub build_strategy    : BuildStrategy,
}

impl TofEventBuilderSettings {
  pub fn new() -> TofEventBuilderSettings {
    TofEventBuilderSettings {
      cachesize         : 100000,
      n_mte_per_loop    : 1,
      n_rbe_per_loop    : 40,
      te_timeout_sec    : 30,
      build_strategy    : BuildStrategy::WaitForNBoards(40),
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


#[derive(Debug, Clone, serde::Deserialize, serde::Serialize)]
pub struct LiftofCCSettings {
  /// location to store data on TOF computer
  pub data_dir                   : String,
  /// default location for RBCalibration files
  pub calibration_dir            : String,
  /// default location for the database
  pub db_path                    : String,
  /// Runtime in seconds
  pub runtime_sec                : u64,
  /// TOFPackets per file. This defines the "length" of 
  /// a subrun. 
  pub packs_per_file             : usize,
  /// The address the flight computer should subscribe 
  /// to to get tof packets
  pub fc_pub_address             : String,
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
}

impl LiftofCCSettings {
  pub fn new() -> Self {
    LiftofCCSettings {
      data_dir                  : String::from(""),
      calibration_dir           : String::from(""),
      db_path                   : String::from("/home/gaps/config/gaps_flight.db"),
      runtime_sec               : 0,
      packs_per_file            : 0,
      fc_pub_address            : String::from(""),
      fc_sub_address            : String::from(""),
      cmd_listener_interval_sec : 60,
      mtb_address               : String::from("10.0.1.10:50001"),
      cpu_moni_interval_sec     : 60,
      rb_ignorelist             : Vec::<u8>::new(),
      run_analysis_engine       : true,
      mtb_settings              : MTBSettings::new(),
      event_builder_settings    : TofEventBuilderSettings::new(),
      analysis_engine_settings  : AnalysisEngineSettings::new(),
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

  pub fn from_toml(filename : String) -> Result<LiftofCCSettings, SerializationError> {
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

impl fmt::Display for LiftofCCSettings {
  fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
    let disp = toml::to_string(self).unwrap_or(
      String::from("-- DESERIALIZATION ERROR! --"));
    write!(f, "<LiftofCCSettings :\n{}>", disp)
  }
}

impl Default for LiftofCCSettings {
  fn default() -> Self {
    LiftofCCSettings::new()
  }
}

