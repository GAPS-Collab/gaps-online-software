//! The following file is part of gaps-online-software and published 
//! under the GPLv3 license
//!
//! Calibration routines for the GAPS tracker system

use crate::prelude::*;

#[cfg_attr(feature="pybindings", pyclass)] 
pub struct TrackerOfflineCalibration {
  pub tf_map     : HashMap<u32,TrackerStripTransferFunction>, 
  pub ped_map    : HashMap<u32,f32>,
  pub pulser_map : HashMap<u32,bool>
}

impl TrackerOfflineCalibration {

  pub fn new() -> Self {
    Self {
      tf_map     : HashMap::<u32,TrackerStripTransferFunction>::new(), 
      ped_map    : HashMap::<u32,f32>::new(),
      pulser_map : HashMap::<u32,bool>::new()
    }
  }
}


