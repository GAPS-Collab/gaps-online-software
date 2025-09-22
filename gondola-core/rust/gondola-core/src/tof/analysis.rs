//! A high level analysis which interplays with 
//! the tof cuts and can histogram TOF relevant 
//! quantities 
// This file is part of gaps-online-software and published 
// under the GPLv3 license

use crate::prelude::*;
use crate::tof::cuts::TofCuts;

/// A container to hold a cut selection and allows 
/// to walk over files and fills a number of histograms 
///
/// FIXME - typically these monolithic structures are 
///         not a good idea
///
pub struct TofAnalysis {
  pub skip_mangled  : bool,
  pub skip_timeout  : bool,
  pub beta_analysis : bool,
  pub nbins         : u64,
  pub cuts          : TofCuts,
  pub use_offsets   : bool,
  pub pid_inner     : Option<u8>,
  pub pid_outer     : Option<u8>,
  pub active        : bool,
  pub nhit          : u64, 
  pub no_hitmiss    : u64, 
  pub one_hitmiss   : u64, 
  pub two_hitmiss   : u64, 
  pub extra_hits    : u64, 
  pub occupancy     : HashMap<u8,u64>,
  pub occupancy_t   : HashMap<u8,u64>
}

impl TofAnalysis {
}
