// This file is part of gaps-online-software and published 
// under the GPLv3 license

use crate::prelude::*;

/// Keep track of run related statistics, errors
#[derive(Debug, Copy, Clone)]
#[cfg_attr(feature="pybindings",pyclass)]
pub struct RunStatistics {
  /// The number of events we have recorded
  pub n_events_rec      : usize,
  /// The number of packets going through 
  /// the event processing
  pub evproc_npack      : usize,
  /// The first event id we saw
  pub first_evid        : u32,
  /// The last event id we saw
  pub last_evid         : u32,
  /// The number of times we encountered 
  /// a deserialization issue
  pub n_err_deser       : usize,
  /// The number of times we encountered 
  /// an issue while sending over zmq
  pub n_err_zmq_send    : usize,
  /// The number of times we encountered
  /// an issue with a wrong channel identifier
  pub n_err_chid_wrong  : usize,
  /// How many times did we read out an incorrect
  /// tail?
  pub n_err_tail_wrong  : usize,
  /// The number of times we failed a crc32 check
  pub n_err_crc32_wrong : usize,
}

impl RunStatistics {
  
  pub fn new() -> Self {
    Self {
      n_events_rec      : 0,
      evproc_npack      : 0,
      first_evid        : 0,
      last_evid         : 0,
      n_err_deser       : 0,
      n_err_zmq_send    : 0,
      n_err_chid_wrong  : 0,
      n_err_tail_wrong  : 0,
      n_err_crc32_wrong : 0,
    }
  }

  pub fn get_n_anticipated(&self) -> i32 {
    self.last_evid as i32 - self.first_evid as i32
  }
}

impl fmt::Display for RunStatistics {
  fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
    let mut resp = String::from("<RunStatistics:\n");
    resp += &(format!("  first event id : {}\n", self.first_evid));
    resp += &(format!("  last  event id : {}\n", self.last_evid));
    resp += &(format!("  --> expected {} event (ids)\n", self.get_n_anticipated()));
    resp += &(format!("  event_processing #packets : {}\n", self.evproc_npack));
    if self.get_n_anticipated() != self.evproc_npack as i32 {
      resp += &(format!("  --> discrepancy of {} event (ids)\n", self.get_n_anticipated() - self.evproc_npack as i32))
    }
    resp += &(format!("  event_processing n tail err : {}\n", self.n_err_tail_wrong));
    resp += &(format!("  event_processing n chid err : {}\n", self.n_err_chid_wrong));
    write!(f, "{}", resp)
  }
}

#[cfg(feature="pybindings")]
pythonize!(RunStatistics);
