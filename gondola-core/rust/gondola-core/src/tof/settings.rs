//! Aggregate settings for the TOF system
//!
//! Control the settings for the C&C server
//! as well as the liftof-clients on the RBs
//!
//! Different sections might represent different
//! threads/aspects of the code
//!
// This file is part of gaps-online-software and published 
// under the GPLv3 license

use crate::prelude::*;

/// Define which entitiy will configure another entity
#[derive(Debug, Clone, PartialEq, serde::Deserialize, serde::Serialize)]
pub enum ParameterSetStrategy {
  /// Configuration through the TOF cpu
  ControlServer,
  /// Configuration through the relevant board itself
  Board
}

/// Configure the trigger
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct MTBSettings {
  /// Select the trigger type for this run
  pub trigger_type           : TriggerType,
  /// Select the prescale factor for a run. The
  /// prescale factor is between 0 (no events)
  /// and 1.0 (all events). E.g. 0.1 means allow 
  /// only 10% of the events
  /// THIS DOES NOT APPLY TO THE GAPS OR POISSON 
  /// TRIGGER!
  pub trigger_prescale               : f32,
  /// Set to true if we want a combo trigger run
  pub use_combo_trigger              : bool,
  /// Set the global trigger type. This has to be less 
  /// strict than the trigger type   
  pub global_trigger_type            : TriggerType,
  /// Set the gloabl trigger prescale
  pub global_trigger_prescale        : f32,

  /// in case trigger_type = "Poisson", set rate here
  pub poisson_trigger_rate           : u32,
  /// in case trigger_type = "Gaps", set if we want to use 
  /// beta
  pub gaps_trigger_use_beta     : bool,
  /// In case we are running the fixed rate trigger, set the
  /// desired rate here
  /// not sure
  //pub gaps_trigger_inner_thresh : u32,
  ///// not sure
  //pub gaps_trigger_outer_thresh : u32, 
  ///// not sure
  //pub gaps_trigger_total_thresh : u32, 
  ///// not sure
  //pub gaps_trigger_hit_thresh   : u32,
  /// Enable trace suppression on the MTB. If enabled, 
  /// only those RB which hits will read out waveforms.
  /// In case it is disabled, ALL RBs will readout events
  /// ALL the time. For this, we need also the eventbuilder
  /// strategy "WaitForNBoards(40)"
  pub trace_suppression  : bool,
  /// The number of seconds we want to wait
  /// without hearing from the MTB before
  /// we attempt a reconnect
  pub mtb_timeout_sec    : u64,
  /// Time in seconds between housekkeping 
  /// packets
  pub mtb_moni_interval  : u64,
  pub rb_int_window      : u8,
  pub tiu_emulation_mode : bool,
  pub tiu_ignore_busy    : bool,
  pub tofbot_webhook     : String,
  pub hb_send_interval   : u64,
}

impl MTBSettings {
  pub fn new() -> Self {
    Self {
      trigger_type            : TriggerType::Unknown,
      trigger_prescale        : 0.0,
      poisson_trigger_rate    : 0,
      gaps_trigger_use_beta   : true,
      trace_suppression       : true,
      mtb_timeout_sec         : 60,
      mtb_moni_interval       : 30,
      rb_int_window           : 1,
      tiu_emulation_mode      : false,
      tiu_ignore_busy         : false,
      tofbot_webhook          : String::from(""),
      hb_send_interval        : 30,
      use_combo_trigger       : false,
      global_trigger_type     : TriggerType::Unknown,
      global_trigger_prescale : 1.0,
    }
  }

  /// Emit a config, so that infomraiton can be transported
  /// over the wire
  pub fn emit_triggerconfig(&self) -> TriggerConfig {
    let mut cfg = TriggerConfig::new();
    // all fields should be active, since the settings file 
    // contains all fields per definition. We can already 
    // be future proof and just set all of them
    cfg.active_fields          = u32::MAX;
    cfg.gaps_trigger_use_beta  = Some(self.gaps_trigger_use_beta);
    cfg.prescale               = Some(self.trigger_prescale);
    cfg.trigger_type           = Some(self.trigger_type);
    cfg.use_combo_trigger      = Some(self.use_combo_trigger);
    cfg.combo_trigger_type     = Some(self.global_trigger_type);
    cfg.combo_trigger_prescale = Some(self.global_trigger_prescale);
    cfg.trace_suppression      = Some(self.trace_suppression);
    cfg.mtb_moni_interval      = Some((self.mtb_moni_interval & 0xffff) as u16); 
    cfg.tiu_ignore_busy        = Some(self.tiu_ignore_busy); 
    cfg.hb_send_interval       = Some((self.hb_send_interval & 0xffff) as u16); 
    cfg
  }

  /// Change seetings accordingly to config 
  pub fn from_triggerconfig(&mut self, cfg : &TriggerConfig) {
    if cfg.gaps_trigger_use_beta.is_some() {
      self.gaps_trigger_use_beta   = cfg.gaps_trigger_use_beta.unwrap() ;
    }
    if cfg.prescale.is_some() {
      self.trigger_prescale        = cfg.prescale.unwrap()              ;
    }
    if cfg.trigger_type.is_some() {
      self.trigger_type            = cfg.trigger_type.unwrap()          ; 
    }
    if cfg.use_combo_trigger.is_some() {
      self.use_combo_trigger       = cfg.use_combo_trigger.unwrap()     ;
    }
    if cfg.combo_trigger_type.is_some() {
      self.global_trigger_type     = cfg.combo_trigger_type.unwrap()    ;
    }
    if cfg.combo_trigger_prescale.is_some() {
      self.global_trigger_prescale = cfg.combo_trigger_prescale.unwrap();
    }
    if cfg.trace_suppression.is_some() {
      self.trace_suppression       = cfg.trace_suppression.unwrap()     ;
    }
    if cfg.mtb_moni_interval.is_some() {
      self.mtb_moni_interval       = cfg.mtb_moni_interval.unwrap() as u64;
    }
    if cfg.tiu_ignore_busy.is_some() {
      self.tiu_ignore_busy         = cfg.tiu_ignore_busy.unwrap()       ;
    }
    if cfg.hb_send_interval.is_some() {
      self.hb_send_interval        = cfg.hb_send_interval.unwrap()  as u64;
    }
  }
}

impl fmt::Display for MTBSettings {
  fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
    let disp = toml::to_string(self).unwrap_or(
      String::from("-- DESERIALIZATION ERROR! --"));
    write!(f, "<MTBSettings :\n{}>", disp)
  }
}

impl Default for MTBSettings {
  fn default() -> Self {
    Self::new()
  }
}

#[derive(Debug, Clone, serde::Deserialize, serde::Serialize)]
pub struct PreampSettings {
  /// actually apply the below settings
  pub set_preamp_voltages    : bool,
  /// liftof-cc will send commands to set the 
  /// preamp bias voltages
  pub set_strategy           : ParameterSetStrategy,
  /// preamp biases (one set of 16 values per RAT
  pub rat_preamp_biases      : HashMap<String, [f32;16]>
}

impl PreampSettings {
  pub fn new() -> Self {
    //let default_biases = HashMap::<u8, [f32;16]>::new();
    let default_biases = HashMap::from([
      (String::from("RAT01"), [58.0;16]),
      (String::from("RAT02"), [58.0;16]),
      (String::from("RAT03"), [58.0;16]),
      (String::from("RAT04"), [58.0;16]),
      (String::from("RAT05"), [58.0;16]),
      (String::from("RAT06"), [58.0;16]),
      (String::from("RAT07"), [58.0;16]),
      (String::from("RAT08"), [58.0;16]),
      (String::from("RAT09"), [58.0;16]),
      (String::from("RAT10"), [58.0;16]),
      (String::from("RAT11"), [58.0;16]),
      (String::from("RAT12"), [58.0;16]),
      (String::from("RAT13"), [58.0;16]),
      (String::from("RAT14"), [58.0;16]),
      (String::from("RAT15"), [58.0;16]),
      (String::from("RAT16"), [58.0;16]),
      (String::from("RAT17"), [58.0;16]),
      (String::from("RAT18"), [58.0;16]),
      (String::from("RAT19"), [58.0;16]),
      (String::from("RAT20"), [58.0;16])]);

    Self {
      set_preamp_voltages    : false,
      set_strategy           : ParameterSetStrategy::ControlServer,
      rat_preamp_biases      : default_biases,
    }
  }

  #[cfg(feature="database")]
  pub fn emit_pb_settings_packets(&self, rats : &HashMap<u8,RAT>) -> Vec<TofPacket> {
    let mut packets = Vec::<TofPacket>::new();
    for k in rats.keys() {
      let rat          = &rats[&k];
      let rat_key      = format!("RAT{:2}", rat);
      let mut cmd      = TofCommand::new();
      cmd.command_code = TofCommandCode::SetPreampBias;
      let mut payload  = PreampBiasConfig::new();
      payload.rb_id    = rat.rb2_id as u8;
      if *k as usize >= self.rat_preamp_biases.len() {
        error!("RAT ID {k} larger than 20!");
        continue;
      }
      payload.biases = self.rat_preamp_biases[&rat_key];
      cmd.payload = payload.to_bytestream();
      let tp = cmd.pack();
      packets.push(tp);
    }
    packets
  }
}

impl fmt::Display for PreampSettings {
  fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
    let disp : String;
    match toml::to_string(self) {
      Err(err) => {
        error!("Deserialization error! {err}");
        disp = String::from("-- DESERIALIZATION ERROR! --");
      }
      Ok(_disp) => {
        disp = _disp;
      }
    }
    write!(f, "<PreampBiasSettings :\n{}>", disp)
  }
}

impl Default for PreampSettings {
  fn default() -> Self {
    Self::new()
  }
}

#[derive(Debug, Clone, serde::Deserialize, serde::Serialize)]
pub struct LTBThresholdSettings {
  /// actually apply the below settings
  pub set_ltb_thresholds    : bool,
  /// liftof-cc will send commands to set the 
  /// ltb thresholds
  pub set_strategy          : ParameterSetStrategy,
  /// ltb threshold voltages (one set of 3 values per RAT)
  pub rat_ltb_thresholds    : HashMap<String, [f32;3]>
}

impl LTBThresholdSettings {
  pub fn new() -> Self {
    let default_thresholds = HashMap::from([
      (String::from("RAT01"), [40.0,32.0,375.0]),
      (String::from("RAT02"), [40.0,32.0,375.0]),
      (String::from("RAT03"), [40.0,32.0,375.0]),
      (String::from("RAT04"), [40.0,32.0,375.0]),
      (String::from("RAT05"), [40.0,32.0,375.0]),
      (String::from("RAT06"), [40.0,32.0,375.0]),
      (String::from("RAT07"), [40.0,32.0,375.0]),
      (String::from("RAT08"), [40.0,32.0,375.0]),
      (String::from("RAT09"), [40.0,32.0,375.0]),
      (String::from("RAT10"), [40.0,32.0,375.0]),
      (String::from("RAT11"), [40.0,32.0,375.0]),
      (String::from("RAT12"), [40.0,32.0,375.0]),
      (String::from("RAT13"), [40.0,32.0,375.0]),
      (String::from("RAT14"), [40.0,32.0,375.0]),
      (String::from("RAT15"), [40.0,32.0,375.0]),
      (String::from("RAT16"), [40.0,32.0,375.0]),
      (String::from("RAT17"), [40.0,32.0,375.0]),
      (String::from("RAT18"), [40.0,32.0,375.0]),
      (String::from("RAT19"), [40.0,32.0,375.0]),
      (String::from("RAT20"), [40.0,32.0,375.0])]);

      Self {
        set_ltb_thresholds    : false,
        set_strategy          : ParameterSetStrategy::ControlServer,
        rat_ltb_thresholds    : default_thresholds,
      }
  }

  #[cfg(feature="database")]
  pub fn emit_ltb_settings_packets(&self, rats : &HashMap<u8,RAT>) -> Vec<TofPacket> {
    let mut packets = Vec::<TofPacket>::new();
    for k in rats.keys() {
      let rat          = &rats[&k];
      let rat_key      = format!("RAT{:2}", rat);
      let mut cmd      = TofCommand::new();
      cmd.command_code = TofCommandCode::SetLTBThresholds;
      let mut payload  = LTBThresholdConfig::new();
      payload.rb_id    = rat.rb1_id as u8;
      if *k as usize >= self.rat_ltb_thresholds.len() {
        error!("RAT ID {k} larger than 20!");
        continue;
      }
      payload.thresholds = self.rat_ltb_thresholds[&rat_key];
      cmd.payload = payload.to_bytestream();
      let tp = cmd.pack();
      packets.push(tp);
    }
    packets
  }
}

impl fmt::Display for LTBThresholdSettings {
  fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
    let disp : String;
    match toml::to_string(self) {
      Err(err) => {
        error!("Deserialization error! {err}");
        disp = String::from("-- DESERIALIZATION ERROR! --");
      }
      Ok(_disp) => {
        disp = _disp;
      }
    }
    write!(f, "<LTBThresholdSettings :\n{}>", disp)
  }
}

impl Default for LTBThresholdSettings {
  fn default() -> Self {
    Self::new()
  }
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct CommandDispatcherSettings {
  /// Log all commands into this file
  /// Set to "/dev/null" to turn off.
  /// The mode will be always "append", since we don't 
  /// expect a lot of logging
  pub cmd_log_path               : String,
  /// The address of the liftof-command & control server
  /// that is the ip address on the RBNetwork which the 
  /// liftof-cc instance runs on 
  /// This address will be used as "PUB" for the CommandDispather
  /// This address has to be within the RB network
  pub cc_server_address          : String,   
  /// The address ("tcp://xx.xx.xx.xx:xxxxx") the tof computer should subscribe to 
  /// to get commands from the flight computer. This address is within the 
  /// flight network
  pub fc_sub_address             : String,
  /// Interval of time that will elapse from a cmd check to the other
  pub cmd_listener_interval_sec  : u64,
  /// Safety mechanism - is this is on, the command listener will deny 
  /// every request. E.g. in staging mode to guarantee no tinkering
  pub deny_all_requests          : bool
}

impl CommandDispatcherSettings {
  pub fn new() -> Self {
    Self {
      cmd_log_path                   : String::from("/home/gaps/log"),
      cc_server_address              : String::from("tcp://10.0.1.10:42000"),   
      fc_sub_address                 : String::from("tcp://192.168.37.200:41662"),
      cmd_listener_interval_sec      : 1,
      deny_all_requests              : false
    }
  }
}

impl fmt::Display for CommandDispatcherSettings {
  fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
    let disp = toml::to_string(self).unwrap_or(
      String::from("-- DESERIALIZATION ERROR! --"));
    write!(f, "<CommandDispatcherSettings :\n{}>", disp)
  }
}

impl Default for CommandDispatcherSettings {
  fn default() -> Self {
    Self::new()
  }
}

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


/// Settings for the specific clients on the RBs (liftof-rb)
#[derive(Debug, Copy, Clone, serde::Serialize, serde::Deserialize)]
pub struct RBSettings {
  /// Don't send events if they have issues. Requires
  /// EventStatus::Perfect. This can not work in the
  /// OperationMode RBHighThroughput
  pub only_perfect_events           : bool,
  /// Calculate the crc32 sum for each channel and set
  /// the EventStatus flag accordingly.
  pub calc_crc32                    : bool,
  /// tof operation mode - either "StreamAny",
  /// "RequestReply" or "RBHighThroughput"
  pub tof_op_mode                   : TofOperationMode,
  /// if different from 0, activate RB self trigger
  /// in poisson mode
  pub trigger_poisson_rate          : u32,
  /// if different from 0, activate RB self trigger 
  /// with fixed rate setting
  pub trigger_fixed_rate            : u32,
  ///// if different from 0, activate RB self trigger
  ///// in poisson mode
  //pub trigger_poisson_rate    : u32,
  ///// if different from 0, activate RB self trigger 
  ///// with fixed rate setting
  //pub trigger_fixed_rate      : u32,
  /// Either "Physics" or a calibration related 
  /// data type, e.g. "VoltageCalibration".
  /// <div class="warning">This might get deprecated in a future version!</div>
  pub data_type                     : DataType,
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
  pub rb_buff_strategy               : RBBufferStrategy,
  /// The general moni interval. Whenever this time in seconds has
  /// passed, the RB will send a RBMoniData packet
  pub rb_moni_interval               : f32,
  /// Powerboard monitoring. Do it every multiple of rb_moni_interval
  pub pb_moni_every_x                : f32,
  /// Preamp monitoring. Do it every multiple of rb_moni_interval
  pub pa_moni_every_x                : f32,
  /// LTB monitoring. Do it every multiple of rb_moni_interval
  pub ltb_moni_every_x               : f32,
  /// Choose between drs deadtime or fpga 
  pub drs_deadtime_instead_fpga_temp : bool
}

impl RBSettings {
  pub fn new() -> Self {
    Self {
      only_perfect_events            : false,
      calc_crc32                     : false,
      tof_op_mode                    : TofOperationMode::Default,
      trigger_fixed_rate             : 0,
      trigger_poisson_rate           : 0,
      data_type                      : DataType::Physics,
      rb_buff_strategy               : RBBufferStrategy::AdaptToRate(5),
      rb_moni_interval               : 0.0,
      pb_moni_every_x                : 0.0,
      pa_moni_every_x                : 0.0,
      ltb_moni_every_x               : 0.0,
      drs_deadtime_instead_fpga_temp : false
    }
  }

  pub fn from_tofrbconfig(&mut self, cfg : &TofRBConfig) {
    if cfg.rb_moni_interval.is_some() {
      self.rb_moni_interval = cfg.rb_moni_interval.unwrap() as f32;              
    }
    if cfg.rb_moni_interval.is_some() {
      self.pb_moni_every_x  = cfg.pb_moni_every_x.unwrap() as f32;              
    }
    if cfg.rb_moni_interval.is_some() {
      self.pa_moni_every_x  = cfg.pa_moni_every_x.unwrap() as f32;              
    }
    if cfg.rb_moni_interval.is_some() {
      self.ltb_moni_every_x = cfg.ltb_moni_every_x.unwrap() as f32;              
    }
    if cfg.rb_moni_interval.is_some() {
      self.drs_deadtime_instead_fpga_temp = cfg.drs_deadtime_instead_fpga_temp.unwrap(); 
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
  pub cfd_fraction           : f32,
  /// Time over threshold threshold in mV
  pub tot_threshold          : Option<f32>
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
      cfd_fraction              : 0.2,
      tot_threshold             : Some(500.0),
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
  pub cachesize           : u32,
  pub n_mte_per_loop      : u32,
  pub n_rbe_per_loop      : u32,
  /// The timeout parameter for the TofEvent. If not
  /// complete after this time, send it onwards anyway
  pub te_timeout_sec      : u32,
  /// try to sort the events by id (uses more resources)
  pub sort_events         : bool,
  pub build_strategy      : BuildStrategy,
  pub greediness          : u8,
  pub wait_nrb            : u8,
  pub hb_send_interval    : u16,
  /// Allows to restrict saving the event to disk
  /// based on the interesting event parameters
  /// (These are minimum values)
  pub only_save_interesting : bool,
  pub thr_n_hits_umb        : u8,
  pub thr_n_hits_cbe        : u8,
  pub thr_n_hits_cor        : u8,
  pub thr_tot_edep_umb      : f32,
  pub thr_tot_edep_cbe      : f32,
  pub thr_tot_edep_cor      : f32,
}

impl TofEventBuilderSettings {
  pub fn new() -> TofEventBuilderSettings {
    TofEventBuilderSettings {
      cachesize             : 100000,
      n_mte_per_loop        : 1,
      n_rbe_per_loop        : 40,
      te_timeout_sec        : 30,
      sort_events           : false,
      build_strategy        : BuildStrategy::Adaptive,
      greediness            : 3,
      wait_nrb              : 40,
      hb_send_interval      : 30,
      only_save_interesting : false,
      thr_n_hits_umb        : 0,
      thr_n_hits_cbe        : 0,
      thr_n_hits_cor        : 0,
      thr_tot_edep_umb      : 0.0,
      thr_tot_edep_cbe      : 0.0,
      thr_tot_edep_cor      : 0.0,
    }
  }

  pub fn from_tofeventbuilderconfig(&mut self, cfg : &TOFEventBuilderConfig) {
    if cfg.cachesize.is_some() {
      self.cachesize = cfg.cachesize.unwrap();
    }
    if cfg.n_mte_per_loop.is_some() {
      self.n_mte_per_loop = cfg.n_mte_per_loop.unwrap();
    }
    if cfg.n_rbe_per_loop.is_some() {
      self.n_rbe_per_loop = cfg.n_rbe_per_loop.unwrap();
    }
    if cfg.te_timeout_sec.is_some() {
      self.te_timeout_sec = cfg.te_timeout_sec.unwrap();
    }
    if cfg.sort_events.is_some() {
      self.sort_events = cfg.sort_events.unwrap();
    }
    if cfg.build_strategy.is_some() {
      self.build_strategy = cfg.build_strategy.unwrap();
    }
    if cfg.greediness.is_some() {
      self.greediness = cfg.greediness.unwrap();
    }
    if cfg.wait_nrb.is_some() {
      self.wait_nrb = cfg.wait_nrb.unwrap();
    }
    if cfg.hb_send_interval.is_some() {
      self.hb_send_interval = cfg.hb_send_interval.unwrap();
    }
    if cfg.only_save_interesting.is_some() {
      self.only_save_interesting = cfg.only_save_interesting.unwrap();
    }
    if cfg.thr_n_hits_umb.is_some() { 
      self.thr_n_hits_umb = cfg.thr_n_hits_umb.unwrap();
    }
    if cfg.thr_n_hits_cbe.is_some() {      
      self.thr_n_hits_cbe = cfg.thr_n_hits_cbe.unwrap();
    }
    if cfg.thr_n_hits_cor.is_some()   {
      self.thr_n_hits_cor = cfg.thr_n_hits_cor.unwrap();
    }
    if cfg.thr_tot_edep_umb.is_some() {    
      self.thr_tot_edep_umb = cfg.thr_tot_edep_umb.unwrap();
    }
    if cfg.thr_tot_edep_cbe.is_some() {    
      self.thr_tot_edep_cbe = cfg.thr_tot_edep_cbe.unwrap();
    }
    if cfg.thr_tot_edep_cor.is_some() {    
      self.thr_tot_edep_cor = cfg.thr_tot_edep_cor.unwrap();
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
  pub data_dir                  : String,
  /// The data written on disk gets divided into 
  /// files of a fixed size. 
  pub mbytes_per_file           : usize,
  /// The address the flight computer should subscribe 
  /// to to get tof packets
  pub fc_pub_address            : String,
  /// Mark a certain fraction of events as to be discarded, 
  /// that is not to be stored on disk
  /// 1 = Throw away all events, 0 = throw away no events
  pub discard_event_fraction    : f32,
  ///// Don't save events which are non-interesting
  //pub discard_non_interesting   : bool,
  //pub filter_interesting_numb   : u8,
  //pub filter_interesting_ncbe   : u8,
  //pub filter_interesting_n
  /// Send also MastertriggerPackets (this should be 
  /// turned off in flight - only useful if 
  /// send_flight_packets is true, otherwise
  /// MTB events will get sent as part of TofEvents
  pub send_mtb_event_packets    : bool,
  /// switch off waveform sending (in case of we 
  /// are sending flight packets)
  pub send_rbwaveform_packets   : bool,
  /// send only a fraction of all RBWaveform packets
  /// 1 = all events, 1000 = every 1/1000 event
  pub send_rbwf_every_x_event   : u32,
  pub send_tof_summary_packets  : bool,
  pub send_tof_event_packets    : bool,
  /// Send the RBCalibration to ground
  pub send_cali_packets         : bool,
  pub hb_send_interval          : u16,
}

impl DataPublisherSettings {
  pub fn new() -> Self {
    Self {
      data_dir                  : String::from(""),
      mbytes_per_file           : 420,
      fc_pub_address            : String::from(""),
      discard_event_fraction    : 0.0,
      send_mtb_event_packets    : false,
      send_rbwaveform_packets   : false,
      send_rbwf_every_x_event   : 1,
      send_tof_summary_packets  : true,
      send_tof_event_packets    : false,
      send_cali_packets         : true,
      hb_send_interval          : 30,
    }
  }
  
  pub fn from_datapublisherconfig(&mut self, cfg : &DataPublisherConfig) {
    if cfg.mbytes_per_file.is_some() {
      self.mbytes_per_file = cfg.mbytes_per_file.unwrap() as usize;
    }
    if cfg.discard_event_fraction.is_some() {
      self.discard_event_fraction = cfg.discard_event_fraction.unwrap();
    }
    if cfg.send_mtb_event_packets.is_some() {
      self.send_mtb_event_packets = cfg.send_mtb_event_packets.unwrap();
    }
    if cfg.send_rbwaveform_packets.is_some() {
      self.send_rbwaveform_packets = cfg.send_rbwaveform_packets.unwrap();
    }
    if cfg.send_rbwf_every_x_event.is_some() {
      self.send_rbwf_every_x_event = cfg.send_rbwf_every_x_event.unwrap();
    }
    if cfg.send_tof_summary_packets.is_some() {
      self.send_tof_summary_packets = cfg.send_tof_summary_packets.unwrap();
    }
    if cfg.send_tof_event_packets.is_some() {
      self.send_tof_event_packets = cfg.send_tof_event_packets.unwrap();
    }
    if cfg.hb_send_interval.is_some() {
      self.hb_send_interval = cfg.hb_send_interval.unwrap();
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
#[cfg_attr(feature="pybindings", pyclass)]
pub struct LiftofSettings {
  /// read run .toml files from this directory and 
  /// automotically work through them 1by1
  pub staging_dir                : String,
  /// default location for RBCalibration files
  pub calibration_dir            : String,
  /// default location for the database
  pub db_path                    : String,
  /// Runtime in seconds
  pub runtime_sec                : u64,
  /// The UDP port to be used to get packets from the 
  /// MTB
  pub mtb_address                : String,
  /// The interval (in seconds) to retrive CPUMoniData from 
  /// the TOF CPU
  pub cpu_moni_interval_sec      : u64,
  /// In an intervall from 1-50, these RB simply do not exist
  /// or might have never existed. Always ingore these
  pub rb_ignorelist_always       : Vec<u8>,
  /// ignore these specific RB for this run
  pub rb_ignorelist_run          : Vec<u8>,
  /// Should TofHits be generated?
  pub run_analysis_engine        : bool,
  /// RB contols both, LTB and PB
  pub rb_controls_pb_and_ltb     : Option<Vec<u8>>,
  /// Run a full RB calibration before run 
  /// start?
  pub pre_run_calibration        : bool,
  /// Should the waveforms which go into te calibration 
  /// be saved in the package?
  pub save_cali_wf               : bool,
  /// Do a verification run before each run? The purpose 
  /// of the verification run is to generate a "DetectorStatus"
  /// packet. If a verification run is desired, change this 
  /// number to the number of seconds to do the verification 
  /// run
  pub verification_runtime_sec   : u32,
  /// Settings to control the MTB
  pub mtb_settings               : MTBSettings,
  /// Settings for the TOF event builder
  pub event_builder_settings     : TofEventBuilderSettings,
  /// Settings for the analysis engine
  pub analysis_engine_settings   : AnalysisEngineSettings,
  /// Configure data publshing and saving on local disc
  pub data_publisher_settings    : DataPublisherSettings,
  /// Configure cmmand reception and sending
  pub cmd_dispatcher_settings    : CommandDispatcherSettings,
  /// Settings for the individual RBs
  pub rb_settings                : RBSettings,
  /// Mask individual channels (e.g. dead preamps) 
  /// for the readout boards
  pub rb_channel_mask            : ChannelMaskSettings,
  /// Preamp configuration
  pub preamp_settings            : PreampSettings,
  /// LTB threshold configuration
  pub ltb_settings               : LTBThresholdSettings
}

impl LiftofSettings {
  pub fn new() -> Self {
    LiftofSettings {
      staging_dir               : String::from("/home/gaps/liftof-staging"),
      calibration_dir           : String::from(""),
      db_path                   : String::from("/home/gaps/config/gaps_flight.db"),
      runtime_sec               : 0,
      mtb_address               : String::from("10.0.1.10:50001"),
      cpu_moni_interval_sec     : 60,
      rb_ignorelist_always      : Vec::<u8>::new(),
      rb_ignorelist_run         : Vec::<u8>::new(),
      rb_controls_pb_and_ltb    : None,
      run_analysis_engine       : true,
      pre_run_calibration       : false,
      save_cali_wf              : false,
      verification_runtime_sec  : 0, // no verification run per default
      mtb_settings              : MTBSettings::new(),
      event_builder_settings    : TofEventBuilderSettings::new(),
      analysis_engine_settings  : AnalysisEngineSettings::new(),
      data_publisher_settings   : DataPublisherSettings::new(),
      cmd_dispatcher_settings   : CommandDispatcherSettings::new(),
      rb_settings               : RBSettings::new(),
      rb_channel_mask           : ChannelMaskSettings::new(),
      preamp_settings           : PreampSettings::new(),
      ltb_settings              : LTBThresholdSettings::new(),
    }
  }  

  /// Change the settings according to the ones in the 
  /// given config 
  pub fn from_tofrunconfig(&mut self, cfg : &TofRunConfig) {
    if cfg.runtime.is_some() {
      self.runtime_sec = cfg.runtime.unwrap() as u64;
    }
  }

  /// Change a value by giving the specific key as 
  /// a string, the value then will be parsed 
  /// accordingly
  #[deprecated(since="0.10.0", note="This is a dev deadend and will be nuked!")]
  pub fn set_by_key(&mut self, key : &str, value : String) {
    match key {
      "runtime_sec"               => {
        if let Ok(val) = value.parse::<u64>() {
          self.runtime_sec = val;
        } else {
          error!("Unable to parse {value}!");
        }
      }
      "cpu_moni_interval_sec"     => {
        if let Ok(val) = value.parse::<u64>() {
          self.cpu_moni_interval_sec = val;
        } else {
          error!("Unable to parse {value}!");
        }
      }
      "rb_ignorelist_run"         => {
      }
      "run_analysis_engine"       => {
        if let Ok(val) = value.parse::<bool>() {
          self.run_analysis_engine = val;
        } else {
          error!("Unable to parse {value}!");
        }
      }
      "pre_run_calibration"       => {
        if let Ok(val) = value.parse::<bool>() {
          self.pre_run_calibration = val;
        } else {
          error!("Unable to parse {value}!");
        }
      }
      "save_cali_wf"              => {
        if let Ok(val) = value.parse::<bool>() {
          self.save_cali_wf = val;
        } else {
          error!("Unable to parse {value}!");
        }
      }
      "verification_runtime_sec"  => {
        if let Ok(val) = value.parse::<u32>() {
          self.verification_runtime_sec = val;
        } else {
          error!("Unable to parse {value}!");
        }
      }
      _ => error!("Set by key for {} is not implemented!", key)
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

  pub fn from_toml(filename : &str) -> Result<LiftofSettings, SerializationError> {
    match File::open(filename) {
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
    let disp : String;
    match toml::to_string(self) {
      Err(err) => {
        println!("Deserialization error! {err}");
        disp = String::from("-- DESERIALIZATION ERROR! --");
      }
      Ok(_disp) => {
        disp = _disp;
      }
    }
    write!(f, "<LiftofSettings :\n{}>", disp)
  }
}

impl Default for LiftofSettings {
  fn default() -> Self {
    Self::new()
  }
}

#[cfg(feature="pybindings")]
#[pymethods]
impl LiftofSettings {

  /// Read settings from a .toml file
  ///
  /// # Arugments:
  ///
  /// * filename : A .toml file with settings fro the 
  ///              liftof flight suite
  #[staticmethod]
  fn from_file(filename : &str) -> PyResult<Self> {
    match LiftofSettings::from_toml(filename) {
      Ok(settings) => {
        return Ok(settings);
      }
      Err(err) => {
        return Err(PyValueError::new_err(err.to_string()));
      }
    }
  }
}

#[cfg(feature="pybindings")]
pythonize!(LiftofSettings);

//----------------------------------------------------

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
// #[derive(Debug, Clone, serde::Deserialize, serde::Serialize)]
// pub struct ChannelMaskSettings {
//   /// actually apply the below settings
//   pub set_channel_mask   : bool,
//   /// liftof-cc will send commands to set the 
//   /// preamp bias voltages
//   pub set_strategy           : ParameterSetStrategy,
//   /// channels to mask (one set of 18 values per RAT)
//   pub rat_channel_mask     : HashMap<String, [bool;18]>
// }

/// Ignore RB channnels
///
/// The values in these arrays correspond to 
/// (physical) channels 1-9
#[derive(Debug, Clone, serde::Deserialize, serde::Serialize)]
pub struct ChannelMaskSettings {
  /// actually apply the below settings
  pub set_channel_mask   : bool,
  /// The set strat defines who should acutally set
  /// the parameters. Will that be done by each board
  /// independently (ParameterSetStrategy::Board) or
  /// will a command be sent by liftof-cc 
  /// (ParameterSetStrategy::ControlServer)
  pub set_strategy           : ParameterSetStrategy,
  /// channels to mask (one set of 9 values per RB)
  /// "true" means the channel is enabled, "false", 
  /// disabled
  pub rb_channel_mask     : HashMap<String, [bool;9]>
}

impl ChannelMaskSettings {
  pub fn new() -> Self {
    let mut default_thresholds = HashMap::<String, [bool; 9]>::new();
    for k in 1..51 {
      let key = format!("RB{k:02}");
      default_thresholds.insert(key, [true;9]);
    }
//    let default_thresholds = HashMap::from([
//      (String::from("RAT01"), [false; 9]),
//      (String::from("RAT02"), [false; 9]),
//      (String::from("RAT03"), [false; 9]),
//      (String::from("RAT04"), [false; 9]),
//      (String::from("RAT05"), [false; 9]),
//      (String::from("RAT06"), [false; 9]),
//      (String::from("RAT07"), [false; 9]),
//      (String::from("RAT08"), [false; 9]),
//      (String::from("RAT09"), [false; 9]),
//      (String::from("RAT10"), [false; 9]),
//      (String::from("RAT11"), [false; 9]),
//      (String::from("RAT12"), [false; 9]),
//      (String::from("RAT13"), [false; 9]),
//      (String::from("RAT14"), [false; 9]),
//      (String::from("RAT15"), [false; 9]),
//      (String::from("RAT16"), [false; 9]),
//      (String::from("RAT17"), [false; 9]),
//      (String::from("RAT18"), [false; 9]),
//      (String::from("RAT19"), [false; 9]),
//      (String::from("RAT20"), [false; 9])]);

      Self {
        set_channel_mask    : false,
        set_strategy          : ParameterSetStrategy::ControlServer,
        rb_channel_mask    : default_thresholds,
      }
  }

  #[cfg(feature="database")]
  pub fn emit_ch_mask_packets(&self, rbs : &HashMap<u8,RAT>) -> Vec<TofPacket> {
    let mut packets = Vec::<TofPacket>::new();
    for k in rbs.keys() {
      let rb          = &rbs[&k];
      let rb_key      = format!("RB{:2}", rb);
      let mut cmd      = TofCommand::new();
      cmd.command_code = TofCommandCode::SetRBChannelMask;
      let mut payload  = RBChannelMaskConfig::new();
      payload.rb_id    = rb.rb2_id as u8;
      if *k as usize >= self.rb_channel_mask.len() {
        error!("RB ID {k} larger than 46!");
        continue;
      }
      payload.channels = self.rb_channel_mask[&rb_key];
      cmd.payload = payload.to_bytestream();
      let tp = cmd.pack();
      packets.push(tp);
    }
    packets
  }
}
impl fmt::Display for ChannelMaskSettings {
  fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
    let disp : String;
    match toml::to_string(self) {
      Err(err) => {
        error!("Deserialization error! {err}");
        disp = String::from("-- DESERIALIZATION ERROR! --");
      }
      Ok(_disp) => {
        disp = _disp;
      }
    }
    write!(f, "<RBChannelMaskConfig :\n{}>", disp)
  }
}

impl Default for ChannelMaskSettings {
  fn default() -> Self {
    Self::new()
  }
}

#[cfg(feature="random")]
#[test]
fn mtb_config() {

  for _ in 0..100 {
    let cfg  = TriggerConfig::from_random();
    let mut settings = MTBSettings::new();
    settings.from_triggerconfig(&cfg);
    let test = settings.emit_triggerconfig();
    if cfg.gaps_trigger_use_beta.is_some() {
      assert_eq!(cfg.gaps_trigger_use_beta, test.gaps_trigger_use_beta);
    }
    if cfg.prescale.is_some() {
      assert_eq!(cfg.prescale, test.prescale);
    }
    if cfg.trigger_type.is_some() {
      assert_eq!(cfg.trigger_type, test.trigger_type);
    }
    if cfg.use_combo_trigger.is_some() {
      assert_eq!(cfg.use_combo_trigger, test.use_combo_trigger);
    }
    if cfg.combo_trigger_type.is_some() {
      assert_eq!(cfg.combo_trigger_type, test.combo_trigger_type);
    }
    if cfg.combo_trigger_prescale.is_some() {
      assert_eq!(cfg.combo_trigger_prescale, test.combo_trigger_prescale);
    }
    if cfg.trace_suppression.is_some() {
      assert_eq!(cfg.trace_suppression, test.trace_suppression);
    }
    if cfg.mtb_moni_interval.is_some() {
      assert_eq!(cfg.mtb_moni_interval, test.mtb_moni_interval);
    }
    if cfg.tiu_ignore_busy.is_some() {
      assert_eq!(cfg.tiu_ignore_busy, test.tiu_ignore_busy);
    }
    if cfg.hb_send_interval.is_some() {
      assert_eq!(cfg.hb_send_interval, test.hb_send_interval);
    }
  }
}

#[test]
fn write_config_file() {
  let settings = LiftofSettings::new();
  //println!("{}", settings);
  settings.to_toml(String::from("liftof-config-test.toml"));
}

#[test]
fn read_config_file() {
  let _settings = LiftofSettings::from_toml("liftof-config-test.toml");
}


