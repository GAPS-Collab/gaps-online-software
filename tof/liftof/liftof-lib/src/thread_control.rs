//! Thread control structures
//! FIXME - this should go to liftof-lib

use std::collections::HashMap;
use std::fmt;

use crate::settings::LiftofSettings;

/// Send runtime information 
/// to threads via shared memory
/// (Arc(Mutex)
#[derive(Default, Debug)]
pub struct ThreadControl {
  /// Stop ALL threads
  pub stop_flag                  : bool,
  /// Trigger calibration thread
  pub calibration_active         : bool,
  /// Keep track on how many calibration 
  /// packets we have received
  pub finished_calibrations      : HashMap<u8,bool>,
  /// alive indicator for cmd dispatch thread
  pub thread_cmd_dispatch_active : bool,
  /// alive indicator for data sink thread
  pub thread_data_sink_active    : bool,
  /// alive indicator for runner thread
  pub thread_runner_active       : bool,
  /// alive indicator for event builder thread
  pub thread_event_bldr_active   : bool,
  /// alive indicator for master trigger thread
  pub thread_master_trg_active   : bool,
  /// alive indicator for monitoring thread
  pub thread_monitoring_active   : bool,
  /// Running readoutboard communicator threads - the key is associated rb id
  pub thread_rbcomm_active       : HashMap<u8, bool>,
  /// The current run id
  pub run_id                     : u32,
  /// The number of boards available
  pub n_rbs                      : u32,
  /// Write data to disk
  pub write_data_to_disk         : bool,
  pub liftof_settings            : LiftofSettings,
}

impl ThreadControl {
  pub fn new() -> Self {
    Self {
      stop_flag                  : false,
      calibration_active         : false,
      finished_calibrations      : HashMap::<u8,bool>::new(),
      thread_cmd_dispatch_active : false,
      thread_data_sink_active    : false,
      thread_runner_active       : false,
      thread_event_bldr_active   : false,
      thread_master_trg_active   : false,
      thread_monitoring_active   : false,
      thread_rbcomm_active       : HashMap::<u8,bool>::new(),
      run_id                     : 0,
      n_rbs                      : 0,
      write_data_to_disk         : false,
      liftof_settings            : LiftofSettings::new(),
    }
  }
}

impl fmt::Display for ThreadControl {
  fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
    let mut repr = String::from("<ThreadControl:");
    repr        += &(format!("\n  Run ID         : {}", self.run_id));
    repr        += &(format!("\n  N RBs          : {}", self.n_rbs));
    repr        += &(format!("\n  wr to disk     : {}", self.write_data_to_disk));
    repr        += "\n    -- reported RB calibration activity:";
    repr        += &(format!("\n  RB cali active : {}", self.calibration_active));
    repr        += &(format!("\n  -- finished    : \n{:?}", self.finished_calibrations));       
    repr        += "\n    -- program status:";
    repr        += &(format!("\n  stop flag : {}", self.stop_flag));
    repr        += "\n    -- reported thread activity:";
    repr        += &(format!("\n  cmd dispatcher : {}", self.thread_cmd_dispatch_active));
    repr        += &(format!("\n  runner         : {}", self.thread_runner_active));
    repr        += &(format!("\n  data sink      : {}", self.thread_data_sink_active));
    repr        += &(format!("\n  monitoring     : {}", self.thread_monitoring_active));
    if self.thread_rbcomm_active.len() > 0 {
      repr        += "\n -- active RB threads";
      for k in self.thread_rbcomm_active.keys() {
        repr      += &(format!("\n -- -- {} : {}", k, self.thread_rbcomm_active.get(k).unwrap()));
      }
    }
    repr        += &(format!("\n  master trig    : {}>", self.thread_master_trg_active));
    write!(f, "{}", repr)
  }
}

