//! The following file is part of gaps-online-software and published 
//! under the GPLv3 license
//!
//! This file provides identifier for different kinds of data which 
//! can be used with the gaps-online-software i/o system. 
//!
//! Furtheron, this file sets up the io module. 

pub mod ipbus;
pub mod parsers;
pub mod serialization;
pub mod caraspace;
pub mod root_reader;
pub use root_reader::read_example;

//pub mod streamers;
//pub use streamers::RBMemoryStreamer;

#[cfg(feature = "random")]
use crate::random::FromRandom;

#[cfg(feature = "random")]
use rand::Rng;

use std::fmt;

#[cfg(feature = "pybindings")]
use pyo3::{
  pyclass,
  pymethods
};

use std::path::Path;
use std::fs;
use std::io::{                                                                                    
  self,                                                                                           
  ErrorKind                            
};                                     
use regex::Regex;    

//----------------------------------------------------------

/// Get all filenames in the current path sorted by timestamp if available
/// If the given path is a file and not a directory, return only that 
/// file instead
///
/// # Arguments:
///
///    * input   : name of the target directory
///    * pattern : the regex pattern to look for. That the sorting works,
///                the pattern needs to return a date for the first
///                captured argument and a time for the second captured argument
pub fn list_path_contents_sorted(input: &str, pattern: Option<Regex>) -> Result<Vec<String>, io::Error> {
  let path = Path::new(input);
  match fs::metadata(path) {
    Ok(metadata) => {
      if metadata.is_file() {
        let fname = String::from(input);
        return Ok(vec![fname]);
      } 
      if metadata.is_dir() {
        let re : Regex;
        match pattern {
          None => {
            // use a default pattern which matches mmost cases  
            re = Regex::new(r"Run\d+_\d+\.(\d{6})_(\d{6})UTC(\.tof)?\.gaps$").unwrap();
          }
          Some(_re) => {
            re = _re;
          }
        }
        let mut entries: Vec<(u32, u32, String)> = fs::read_dir(path)?
          .filter_map(Result::ok) // Ignore unreadable entries
          .filter_map(|entry| {
            let filename = format!("{}/{}", path.display(), entry.file_name().into_string().ok()?);
            re.captures(&filename.clone()).map(|caps| {
              let date = caps.get(1)?.as_str().parse::<u32>().ok()?;
              let time = caps.get(2)?.as_str().parse::<u32>().ok()?;
              Some((date, time, filename))
            })?
          })
          .collect();

        // Sort by (date, time)
        entries.sort_by(|a, b| (a.0, a.1).cmp(&(b.0, b.1)));
        // Return only filenames
        return Ok(entries.into_iter().map(|(_, _, name)| name).collect());
      } 
      Err(io::Error::new(ErrorKind::Other, "Path exists but is neither a file nor a directory"))
    }
    Err(e) => Err(e),
  }
}

//----------------------------------------------------------

/// Identifier for different data sources
#[derive(Debug, Copy, Clone, PartialEq)]
#[cfg_attr(feature = "pybindings", pyclass(eq, eq_int))]
#[repr(u8)]
pub enum DataSourceKind {
  Unknown            = 0,
  /// The "classic" written to the TOF-CPU on disk in flight
  /// season 24/25 style
  TofFiles           = 10,
  /// As TofFiles, but sent over the network
  TofStream          = 11,
  /// The files as written on disk when received by a GSE 
  /// system
  TelemetryFiles     = 20,
  /// Flight telemetry stream as sent out directly by the 
  /// instrument
  TelemetryStream    = 21,
  /// Caraspace is a comprehensive, highly efficient data 
  /// format which is used to combine Telemetry + TofStream 
  /// data. Data written in this format as stored on disk.
  CaraspaceFiles     = 30,
  /// The same as above, however, represented as a network
  /// stream
  CaraspaceStream    = 31,
  /// Philip's SimpleDet ROOT files
  ROOTFiles          = 40,
}

impl DataSourceKind {
  fn to_string(&self) -> String {
    match self {
      DataSourceKind::Unknown         => String::from("Unknown"),
      DataSourceKind::TofFiles        => String::from("TofFiles"),
      DataSourceKind::TofStream       => String::from("TofStream"),
      DataSourceKind::TelemetryFiles  => String::from("TelemetryFiles"), 
      DataSourceKind::TelemetryStream => String::from("TelemetryStream"),
      DataSourceKind::CaraspaceFiles  => String::from("CaraspaceFiles"),
      DataSourceKind::CaraspaceStream => String::from("CaraspaceStream"),
      DataSourceKind::ROOTFiles       => String::from("ROOTFIles"),
    }
  }
}

impl From<u8> for DataSourceKind {
  fn from(value: u8) -> Self {
    match value {
      0     => DataSourceKind::Unknown,
      10    => DataSourceKind::TofFiles,
      11    => DataSourceKind::TofStream,
      20    => DataSourceKind::TelemetryFiles,
      21    => DataSourceKind::TelemetryStream,
      30    => DataSourceKind::CaraspaceFiles,
      31    => DataSourceKind::CaraspaceStream,
      40    => DataSourceKind::ROOTFiles,
      // in case of any other number, we know nuthin
      _     => DataSourceKind::Unknown, 
    }
  }
}

impl fmt::Display for DataSourceKind {
  fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
    let r = self.to_string();
    write!(f, "<DataSourceKind: {}>", r)
  }
}

#[cfg(feature = "random")]
impl FromRandom for DataSourceKind {  
  fn from_random() -> Self {
    let choices = [
      DataSourceKind::Unknown           ,
      DataSourceKind::TofFiles,
      DataSourceKind::TofStream,
      DataSourceKind::TelemetryFiles,
      DataSourceKind::TelemetryStream,
      DataSourceKind::CaraspaceFiles,
      DataSourceKind::CaraspaceStream,
      DataSourceKind::ROOTFiles
    ];
    let mut rng  = rand::rng();
    let idx      = rng.random_range(0..choices.len());
    choices[idx]
  }
}

//--------------------------------------------------------------

// in case we have pybindings for this type, 
// expand it so that it can be used as keys
// in dictionaries
#[cfg(feature = "pybindings")]
#[pymethods]
impl DataSourceKind {

  #[getter]
  fn __hash__(&self) -> usize {
    (*self as u8) as usize
  } 
}

//--------------------------------------------------------------

#[test]
#[cfg(feature = "random")]
fn datasourcekind_from_to_u8() {
  let mut type_codes_u8 = Vec::<u8>::new();
  let mut type_codes= Vec::<DataSourceKind>::new(); 
  for _ in 0..100 {
    let ds = DataSourceKind::from_random();
    type_codes_u8.push(ds as u8);
    type_codes.push(ds);
  }
  for idx in 0..type_codes_u8.len() {
    assert_eq!(DataSourceKind::from(type_codes_u8[idx]),type_codes[idx]);  
  }
}


