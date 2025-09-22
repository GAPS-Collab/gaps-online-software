//! Dataclasses provides structures to facilitate the work with data drom 
//! the GAPS experiment. Most noticeably, there are
//!
//! * events       - TOF/Tracker data, TOF events on disk, MergedEvents send over telemetry
//!
//! * packets      - containers to serialize/deserialize the described structures so that 
//!                  these can be stored on disk or send over the network
//!
//! * calibration  - TOF/Tracker related calibration routines and containers to hold results
//! 
//! * io           - read/write packets to/from disk or receive them over the network
//! 
//! * random       - random numbers for software tests
//! 
//! * tof          - Very specific TOF related code which does not fall under a different 
//!                  category
//! 
//! * tracker      - Very specific TRK related code which does not fall under any other 
//!                  category
//!
//! # features:
//!
//! * random - allow random number generated data classes for 
//!            testing
//!
//! * database - access a data base for advanced paddle
//!              mapping, readoutboard and ltb information etc.
//!              This will introduce a dependency on sqlite and 
//!              diesel
//!
//!
// This file is part of gaps-online-software and published 
// under the GPLv3 license

#[macro_use] extern crate log; 

pub mod prelude;
#[cfg(feature="random")]
pub mod random;
pub mod constants;
pub mod events;
pub mod packets;
pub mod version;
pub mod io;
pub mod calibration;
pub mod errors;
pub mod tof;
pub mod tracker;
pub mod monitoring;
pub mod stats;
#[cfg(feature="pybindings")]
pub mod python;
#[cfg(feature="database")]
pub mod database;

// python convention
pub const VERSION: &str = env!("CARGO_PKG_VERSION"); 

/// A simple helper macro adding an as_str function 
/// as well as the Display method to any enum.
///
/// Avoids writing boilerplate
#[macro_export]
macro_rules! expand_and_test_enum {
  ($name:ident, $test_name:ident) => {
    impl fmt::Display for $name {
      fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "<{}: {}>",stringify!($name), self.as_ref())
      }
    }

    impl From<u8> for $name {
      fn from(value: u8) -> Self {
        match Self::from_repr(value)  {
          None => {
            return Self::Unknown;
          }
          Some(variant) => {
            return variant;
          }
        }
      }
    }

    #[cfg(feature="random")]
    impl FromRandom for $name {
      fn from_random() -> Self {
        let mut choices = Vec::<Self>::new();
        for k in Self::iter() {
          choices.push(k);
        }
        let mut rng  = rand::rng();
        let idx = rng.random_range(0..choices.len());
        choices[idx]
      }
    }

    #[test]
    fn $test_name() {
      for _ in 0..100 {
        let data = $name::from_random();
        assert_eq!($name::from(data as u8), data);
      }
    }
  };
}

//---------------------------------------
// helpers to init the logging system
//

use colored::{
    Colorize,
    ColoredString
};
use chrono::Utc;
use log::Level;
use std::io::Write;

/// Make sure that the loglevel is in color, even though not using pretty_env logger
pub fn color_log(level : &Level) -> ColoredString {
  match level {
    Level::Error    => String::from(" ERROR!").red(),
    Level::Warn     => String::from(" WARN  ").yellow(),
    Level::Info     => String::from(" Info  ").green(),
    Level::Debug    => String::from(" debug ").blue(),
    Level::Trace    => String::from(" trace ").cyan(),
  }
}

/// Set up the environmental (env) logger
/// with our format
///
/// Ensure that the lines and module paths
/// are printed in the logging output
pub fn init_env_logger() {
  env_logger::builder()
    .format(|buf, record| {
    writeln!( buf, "[{ts} - {level}][{module_path}:{line}] {args}",
      ts    = Utc::now().format("%Y/%m/%d-%H:%M:%SUTC"), 
      level = color_log(&record.level()),
      module_path = record.module_path().unwrap_or("<unknown>"),
      line  = record.line().unwrap_or(0),
      args  = record.args()
      )
    }).init();
}

//-------------------------------------------------
// Build the python library

#[cfg(feature="pybindings")]
pub use pyo3::prelude::*; 
#[cfg(feature="pybindings")]
pub use pyo3::wrap_pymodule; 
#[cfg(feature="pybindings")]
pub use pyo3::wrap_pyfunction; 

#[cfg(feature="pybindings")]
#[pymodule]
#[pyo3(name = "tof")]
fn tof_py<'_py>(m: &Bound<'_py, PyModule>) -> PyResult<()> {
  use crate::tof::*;
  m.add_class::<RBPaddleID>()?;
  m.add_class::<TofDetectorStatus>()?;
  m.add_class::<TofCommandCode>()?;
  m.add_class::<TofCommand>()?;
  m.add_class::<TofOperationMode>()?;
  m.add_class::<BuildStrategy>()?;
  m.add_class::<PreampBiasConfig>()?;
  m.add_class::<RBChannelMaskConfig>()?;
  m.add_class::<TriggerConfig>()?;
  m.add_class::<TofRunConfig>()?;
  m.add_class::<TofCuts>()?;
  m.add_class::<PyMasterTrigger>()?;
  m.add_function(wrap_pyfunction!(to_board_id_string, m)?)?;
  Ok(())
}

#[cfg(feature="pybindings")]
#[pymodule]
#[pyo3(name = "tracker")]
fn tracker_py<'_py>(m: &Bound<'_py, PyModule>) -> PyResult<()> {
  use crate::tracker::*;
  //m.add_function(wrap_pyfunction!(mt_event_get_timestamp_abs48,m)?)?;
  m.add_function(wrap_pyfunction!(strip_lines, m)?)?;
  Ok(())
}

#[cfg(feature="pybindings")]
#[pymodule]
#[pyo3(name = "calibration")]
fn calibration_py<'_py>(m: &Bound<'_py, PyModule>) -> PyResult<()> {
  use crate::calibration::tof::*;
  m.add_class::<RBCalibrations>()?;
  Ok(())
}

#[cfg(feature="pybindings")]
#[pymodule]
#[pyo3(name = "events")]
fn events_py<'_py>(m: &Bound<'_py, PyModule>) -> PyResult<()> {
  use crate::events::*;
  m.add_class::<TofHit>()?;
  m.add_class::<TrackerHit>()?;
  m.add_class::<RBEventHeader>()?;
  m.add_class::<RBEvent>()?;
  m.add_class::<RBWaveform>()?;
  m.add_class::<EventStatus>()?;
  m.add_class::<DataType>()?;
  m.add_class::<TofEvent>()?;
  m.add_class::<TelemetryEvent>()?;
  m.add_function(wrap_pyfunction!(strip_id, m)?)?;
  m.add_class::<EventQuality>()?;
  m.add_class::<TriggerType>()?;
  m.add_class::<LTBThreshold>()?;
  m.add_function(wrap_pyfunction!(mt_event_get_timestamp_abs48,m)?)?;
  Ok(())
}

#[cfg(feature="pybindings")]
#[pymodule]
#[pyo3(name = "packets")]
fn packets_py<'_py>(m: &Bound<'_py, PyModule>) -> PyResult<()> {
  use crate::packets::*;
  m.add_class::<TofPacketType>()?;
  m.add_class::<TofPacket>()?;
  m.add_class::<TelemetryPacketType>()?;
  m.add_class::<TelemetryPacket>()?;
  m.add_class::<TelemetryPacketHeader>()?;
  m.add_class::<TrackerHeader>()?;
  m.add_function(wrap_pyfunction!(make_systime,m)?)?;
  Ok(())
}

#[cfg(feature="pybindings")]
#[pymodule]
#[pyo3(name = "io")]
fn io_py<'_py>(m: &Bound<'_py, PyModule>) -> PyResult<()> {
  use crate::io::*;
  use crate::io::caraspace::*;
  #[cfg(feature="root")]
  use crate::io::root_reader::read_example;
  use crate::io::caraspace::frame::get_all_telemetry_event_names;
  #[cfg(feature="root")]
  m.add_function(wrap_pyfunction!(read_example, m)?)?;
  m.add_function(wrap_pyfunction!(get_all_telemetry_event_names, m)?)?;
  m.add_function(wrap_pyfunction!(get_runfilename, m)?)?;
  m.add_function(wrap_pyfunction!(get_califilename, m)?)?;
  m.add_function(wrap_pyfunction!(list_path_contents_sorted_py, m)?)?;
  m.add_function(wrap_pyfunction!(get_utc_timestamp, m)?)?;
  m.add_function(wrap_pyfunction!(get_utc_date, m)?)?;
  m.add_function(wrap_pyfunction!(get_rundata_from_file, m)?)?;
  m.add_function(wrap_pyfunction!(get_datetime, m)?)?;
  m.add_function(wrap_pyfunction!(get_unix_timestamp, m)?)?;
  m.add_function(wrap_pyfunction!(get_unix_timestamp_from_telemetry, m)?)?;
  m.add_class::<CRFrameObject>()?;
  m.add_class::<CRFrame>()?;
  m.add_class::<DataSourceKind>()?;
  m.add_class::<CRReader>()?;
  m.add_class::<CRWriter>()?;
  m.add_class::<TofPacketReader>()?;
  m.add_class::<TelemetryPacketReader>()?;
  //m.add_class::<PyDataSource>()?;
  Ok(())
}

#[cfg(feature="pybindings")]
#[pymodule]
#[pyo3(name = "monitoring")]
fn monitoring_py<'_py>(m: &Bound<'_py, PyModule>) -> PyResult<()> {
  use crate::monitoring::*;
  m.add_class::<EventBuilderHB>()?;
  m.add_class::<EventBuilderHBSeries>()?;
  m.add_class::<DataSinkHB>()?;
  m.add_class::<DataSinkHBSeries>()?;
  m.add_class::<MasterTriggerHB>()?;
  m.add_class::<MasterTriggerHBSeries>()?;
  m.add_class::<PAMoniData>()?;
  m.add_class::<PAMoniDataSeries>()?;
  m.add_class::<PBMoniData>()?;
  m.add_class::<PBMoniDataSeries>()?;
  m.add_class::<MtbMoniData>()?;
  m.add_class::<MtbMoniDataSeries>()?;
  m.add_class::<LTBMoniData>()?;
  m.add_class::<LTBMoniDataSeries>()?;
  m.add_class::<RBMoniData>()?;
  m.add_class::<RBMoniDataSeries>()?;
  m.add_class::<CPUMoniData>()?;
  m.add_class::<CPUMoniDataSeries>()?;
  m.add_class::<RunStatistics>()?;
  Ok(())
}

#[cfg(feature="pybindings")]
#[pymodule]
#[pyo3(name = "stats")]
fn stats_py<'_py>(m: &Bound<'_py, PyModule>) -> PyResult<()> {
  //use crate::io::*;
  use crate::stats::py_gamma_pdf;
  m.add_function(wrap_pyfunction!(py_gamma_pdf, m)?)?;
  Ok(())
}

#[cfg(feature="pybindings")]
#[pymodule]
#[pyo3(name = "algo")]
fn algo_py<'_py>(m: &Bound<'_py, PyModule>) -> PyResult<()> {
  //use crate::io::*;
  use crate::tof::algorithms::*;
  m.add_function(wrap_pyfunction!(get_max_value_idx_py, m)?)?;
  m.add_function(wrap_pyfunction!(interpolate_time_py, m)?)?;
  m.add_function(wrap_pyfunction!(fit_sine_simple_py, m)?)?;
  Ok(())
}

#[cfg(feature="database")]
#[cfg(feature="pybindings")]
#[pymodule]
#[pyo3(name = "db")]
fn db_py<'_py>(m: &Bound<'_py, PyModule>) -> PyResult<()> {
  use crate::database::*;
  m.add_class::<TofPaddle>()?;
  m.add_class::<ReadoutBoard>()?;
  m.add_class::<TrackerStrip>()?;
  m.add_class::<TrackerStripMask>()?;
  m.add_class::<TrackerStripPedestal>()?;
  m.add_class::<TrackerStripTransferFunction>()?;
  m.add_class::<TrackerStripCmnNoise>()?;
  m.add_function(wrap_pyfunction!(get_all_rbids_in_db, m)?)?;
  m.add_function(wrap_pyfunction!(get_hid_vid_map, m)?)?;
  m.add_function(wrap_pyfunction!(get_vid_hid_map, m)?)?;
  m.add_function(wrap_pyfunction!(get_dsi_j_ch_pid_map_py, m)?)?;
  Ok(())
}

#[cfg(feature="pybindings")]
#[pyfunction]
fn get_version() -> &'static str {
  return VERSION;
}

/// Python API to rust version of tof-dataclasses.
///
/// Currently, this contains only the analysis 
/// functions
#[cfg(feature="pybindings")]
#[pymodule]
#[pyo3(name = "gondola_core")]
fn gondola_core_py<'_py>(m : &Bound<'_py, PyModule>) -> PyResult<()> { //: Python<'_>, m: &PyModule) -> PyResult<()> {
  pyo3_log::init();
  m.add_function(wrap_pyfunction!(get_version, m)?)?;
  m.add_wrapped(wrap_pymodule!(events_py))?;
  m.add_wrapped(wrap_pymodule!(monitoring_py))?;
  m.add_wrapped(wrap_pymodule!(packets_py))?;
  m.add_wrapped(wrap_pymodule!(tof_py))?;
  m.add_wrapped(wrap_pymodule!(tracker_py))?;
  m.add_wrapped(wrap_pymodule!(io_py))?;
  m.add_wrapped(wrap_pymodule!(db_py))?;
  m.add_wrapped(wrap_pymodule!(stats_py))?;
  m.add_wrapped(wrap_pymodule!(algo_py))?;
  m.add_wrapped(wrap_pymodule!(calibration_py))?;
  Ok(())
}
