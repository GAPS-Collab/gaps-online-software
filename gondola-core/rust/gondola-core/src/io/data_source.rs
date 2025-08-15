//! Generic approach to read all kinds of data within the GAPS wider
//! ecosystem. 
//!
//! We have:
//! * binary data (typically with the ending .bin) - telemetered data
//! * tof data which is written to the TOF CPU disks
//! * Caraspace data - this is merged TOF CPU and binary data. This is 
//!   currently used in L0 data.
//! * data in a customized, special root format as it is created by 
//!   SimpleDet
// This file is part of gaps-online-software and published 
// under the GPLv3 license

use crate::prelude::*;

/// A generic data source which can digest all 
/// kinds of GAPS input data
/// 
/// The Datasource can combine all necessary 
/// meta information, such as information about
/// paddles as well as calibration data for 
/// tracker and TOF.
pub struct DataSource<T> 
  where T:  Default + Serialization { 
  pub kind            : DataSourceKind,
  #[cfg(feature="database")]
  pub paddles         : HashMap<u8,TofPaddle>,
  pub rb_calibrations : HashMap<u8,RBCalibrations>,
  //pub strips  : HashMap<u8,TrackerStrip>,
  pub reader  : dyn DataReader<T>,
}

impl<T> DataSource<T>  
  where T: Default + Serialization {
  //pub fn new(source : &str, pattern : Option<&str>) -> Self {
  //  // at this point, source can be anything. Either a filename, 
  //  // directory or a stream address.
  //  let data_kind     : DataSourceKind;
  //  let regex_pattern : Regex;
  //  match list_path_contents_sorted(source, Some(regex_pattern)) {
  //    Err(err) => (),
  //    Ok(foo) => ()
  //  }
  //}
}





