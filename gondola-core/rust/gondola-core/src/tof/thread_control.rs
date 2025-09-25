//! Thread control structures as used in the TOF flight software 
//! liftof. 
// This file is part of gaps-online-software and published 
// under the GPLv3 license

use crate::prelude::*;

/// Send runtime information 
/// to threads via shared memory
/// (Arc(Mutex)
#[derive(Default, Debug)]
pub struct ThreadControl {
  /// Stop ALL threads
  pub stop_flag                  : bool,
  /// Received INT signal        
  pub sigint_recvd               : bool,
  /// signal to end all rb thread activity
  pub end_all_rb_threads         : bool,
  /// Trigger calibration thread
  pub calibration_active         : bool,
  /// Keep track on how many calibration 
  /// packets we have received
  pub finished_calibrations      : HashMap<u8,bool>,
  /// Hold the actual calibration data
  pub calibrations               : HashMap<u8, RBCalibrations>,
  /// Hold off the master trigger thread, until everything else
  /// is ready
  pub holdoff_mtb_thread         : bool,
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
  /// Manage CTRL+C (or CMD+C/Mac) input
  pub thread_signal_hdlr_active  : bool,
  /// The current run id
  pub run_id                     : u32,
  /// The number of boards available
  pub n_rbs                      : u32,
  #[cfg(feature = "database")]
  /// The active readoutboards in the Tof
  pub rb_list                    : Vec<ReadoutBoard>,
  /// Verification run currently active
  pub verification_active        : bool,
  /// TOF Detector status - which channels are active?
  pub detector_status            : TofDetectorStatus,
  /// Decide if data is actually written to disk
  pub write_data_to_disk         : bool,
  /// indicator that a new 
  /// run has started
  /// (data sinks need to know)
  pub new_run_start_flag         : bool,
  /// initiate a MTB DAQ reset (if the queue is behind)
  pub reset_mtb_daq              : bool,
  pub liftof_settings            : LiftofSettings,
}

impl ThreadControl {
  pub fn new() -> Self {
    Self {
      stop_flag                  : false,
      calibration_active         : false,
      finished_calibrations      : HashMap::<u8,bool>::new(),
      calibrations               : HashMap::<u8, RBCalibrations>::new(),
      sigint_recvd               : false,
      end_all_rb_threads         : false,
      holdoff_mtb_thread         : false,
      thread_cmd_dispatch_active : false,
      thread_data_sink_active    : false,
      thread_runner_active       : false,
      thread_event_bldr_active   : false,
      thread_master_trg_active   : false,
      thread_monitoring_active   : false,
      thread_rbcomm_active       : HashMap::<u8,bool>::new(),
      // in principle this should always be active
      thread_signal_hdlr_active  : true,
      run_id                     : 0,
      n_rbs                      : 0,
      #[cfg(feature = "database")]
      rb_list                    : Vec::<ReadoutBoard>::new(),
      verification_active        : false,
      detector_status            : TofDetectorStatus::new(),
      write_data_to_disk         : false,
      new_run_start_flag         : false,
      reset_mtb_daq              : false,
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
    for k in self.finished_calibrations.keys() {
      repr        += &(format!("\n  -- finished  {}  : {}", k, self.finished_calibrations.get(k).unwrap()));       
    }
    repr        += &(format!("\n    -- verification run: {}", self.verification_active));
    repr        += "\n    -- program status:";
    repr        += &(format!("\n  stop flag        : {}", self.stop_flag));
    repr        += "\n    -- reported thread activity:";
    repr        += &(format!("\n  holdoff mtb thr. : {}", self.holdoff_mtb_thread));
    repr        += &(format!("\n  cmd dispatcher   : {}", self.thread_cmd_dispatch_active));
    repr        += &(format!("\n  runner           : {}", self.thread_runner_active));
    repr        += &(format!("\n  data sink        : {}", self.thread_data_sink_active));
    repr        += &(format!("\n  monitoring       : {}", self.thread_monitoring_active));
    repr        += &(format!("\n  evt builder      : {}", self.thread_event_bldr_active));
    repr        += &(format!("\n  master_trigger   : {}", self.thread_master_trg_active));
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

