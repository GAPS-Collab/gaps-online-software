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
struct TofAnalysis {
  skip_mangled  : bool,
  skip_timeout  : bool,
  beta_analysis : bool,
  nbins         : u64,
  cuts          : TofCuts,
  use_offsets   : bool,
  pid_inner     : Option<u8>,
  pid_outer     : Option<u8>,
  active        : bool,
  nhit          : u64, 
  no_hitmiss    : u64, 
  one_hitmiss   : u64, 
  two_hitmiss   : u64, 
  extra_hits    : u64, 
  occupancy     : HashMap<u8,u64>,
  occupancy_t   : HashMap<u8,u64>
}

impl TofAnalysis {
}
