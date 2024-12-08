//! Pybindings for the RUST dataclasses
//!
pub mod analysis;
pub mod io;
pub mod dataclasses;
#[cfg(feature="telemetry")]
pub mod telemetry;
#[cfg(feature="liftof")]
pub mod liftof;
#[cfg(feature="liftof")]
pub mod liftof_dataclasses;
#[cfg(feature="liftof")]
pub mod master_trigger;
#[cfg(feature="caraspace-serial")]
pub mod caraspace;
#[cfg(feature="telemetry")]
use telemetry_dataclasses::packets as tel_api;

use tof_dataclasses::events::EventStatus;

cfg_if::cfg_if! {
  if #[cfg(feature = "telemetry")] {
    use crate::telemetry::{
      PyTelemetryPacket,
      PyTelemetryPacketReader,
      PyMergedEvent,
      PyTrackerHit,
      PyTrackerHitV2,
      PyTrackerEvent,
      PyTrackerPacket,
      PyTrackerTempLeakPacket,
      PyGPSPacket,
      PyTrackerDAQTempPacket,
      PyTrackerDAQHSKPacket,
      PyTrackerEventIDEchoPacket,
    };
  }
}

cfg_if::cfg_if! {
  if #[cfg(feature = "caraspace-serial")] {
    use crate::caraspace::{
      py_parse_u8,
      py_parse_u16,
      py_parse_u32,
      py_parse_u64,
      PyCRFrame,
      PyCRReader,
      PyCRWriter,
    };
  }
}

cfg_if::cfg_if! {
  if #[cfg(feature = "liftof")] {
    use crate::liftof::{
      py_waveform_analysis,
      wrap_prescale_to_u32,
      wrap_calc_edep_simple,
      wrap_fit_sine_sydney,
      test_db,
      PyLiftofSettings,
      PyIPBus,
      PyMasterTrigger
    };
  }
}

use pyo3::prelude::*;
use pyo3::wrap_pymodule;
use pyo3::exceptions::{
    PyIOError,
};

use crate::analysis::*;
use crate::dataclasses::*;
use crate::io::*;

// these are already wrapped in a pyclass (enum)
use tof_dataclasses::packets::PacketType;
use tof_dataclasses::commands::TofCommandCode;
use tof_dataclasses::events::master_trigger::LTBThreshold;
// additionally, let's add this functionality
use tof_dataclasses::database::{
  get_dsi_j_ch_pid_map,
  DsiJChPidMapping,
  RbChPidMapping,
  get_rb_ch_pid_map,
  get_rb_ch_pid_a_map,
  get_rb_ch_pid_b_map,
  Paddle,
  connect_to_db
};

/// Create a map from the database which allows to map
/// DSI,J,LTB channel for a connected LTB to the respective
/// Paddle ID
///
/// This will query the database and then create a map
/// structure, which can then be used for further queries
///
/// # Arguments:
///     db_path : Path to the gaps_flight.db (or similar 
///               db with paddle information)
#[pyfunction]
#[pyo3(name="create_mtb_connection_to_pid_map")]
fn py_create_mtb_connection_to_pid_map(db_path : String) -> PyResult<DsiJChPidMapping> {
  match connect_to_db(db_path) {
    Err(err) => {
      return Err(PyIOError::new_err(err.to_string()));
    }
    Ok(mut conn) => {
      match Paddle::all(&mut conn) {
        None => {
          return Err(PyIOError::new_err("Unable to retrieve paddle information from DB!"));
        }
        Some(paddles) => {
          let mapping = get_dsi_j_ch_pid_map(&paddles);
          Ok(mapping)
        }
      }
    }
  }
}

/// Map RB ID/Ch to Paddle ID. A side exclusively.
///
/// This will query the database and then create a map
/// structure, which can then be used for further queries
///
/// # Arguments:
///     db_path : Path to the gaps_flight.db (or similar 
///               db with paddle information)
#[pyfunction]
#[pyo3(name="create_rb_ch_to_pid_map")]
fn py_create_rb_ch_to_pid_map(db_path : String) -> PyResult<RbChPidMapping> {
  match connect_to_db(db_path) {
    Err(err) => {
      return Err(PyIOError::new_err(err.to_string()));
    }
    Ok(mut conn) => {
      match Paddle::all(&mut conn) {
        None => {
          return Err(PyIOError::new_err("Unable to retrieve paddle information from DB!"));
        }
        Some(paddles) => {
          let mapping = get_rb_ch_pid_map(&paddles);
          Ok(mapping)
        }
      }
    }
  }
}

/// Map RB ID/Ch to Paddle ID. A side exclusively.
///
/// This will query the database and then create a map
/// structure, which can then be used for further queries
///
/// # Arguments:
///     db_path : Path to the gaps_flight.db (or similar 
///               db with paddle information)
#[pyfunction]
#[pyo3(name="create_rb_ch_to_pid_a_map")]
fn py_create_rb_ch_to_pid_a_map(db_path : String) -> PyResult<RbChPidMapping> {
  match connect_to_db(db_path) {
    Err(err) => {
      return Err(PyIOError::new_err(err.to_string()));
    }
    Ok(mut conn) => {
      match Paddle::all(&mut conn) {
        None => {
          return Err(PyIOError::new_err("Unable to retrieve paddle information from DB!"));
        }
        Some(paddles) => {
          let mapping = get_rb_ch_pid_a_map(&paddles);
          Ok(mapping)
        }
      }
    }
  }
}

/// Map RB ID/Ch to Paddle ID. A side exclusively.
///
/// This will query the database and then create a map
/// structure, which can then be used for further queries
///
/// # Arguments:
///     db_path : Path to the gaps_flight.db (or similar 
///               db with paddle information)
#[pyfunction]
#[pyo3(name="create_rb_ch_to_pid_b_map")]
fn py_create_rb_ch_to_pid_b_map(db_path : String) -> PyResult<RbChPidMapping> {
  match connect_to_db(db_path) {
    Err(err) => {
      return Err(PyIOError::new_err(err.to_string()));
    }
    Ok(mut conn) => {
      match Paddle::all(&mut conn) {
        None => {
          return Err(PyIOError::new_err("Unable to retrieve paddle information from DB!"));
        }
        Some(paddles) => {
          let mapping = get_rb_ch_pid_b_map(&paddles);
          Ok(mapping)
        }
      }
    }
  }
}


#[pymodule]
#[pyo3(name = "analysis")]
fn tof_analysis<'_py>(m: &Bound<'_py, PyModule>) -> PyResult<()> {
  m.add_function(wrap_pyfunction!(py_get_periods, m)?)?;
  m.add_function(wrap_pyfunction!(py_time2bin,m)?)?;
  m.add_function(wrap_pyfunction!(py_find_peaks,m)?)?;
  m.add_function(wrap_pyfunction!(py_find_peaks_zscore,m)?)?;
  m.add_function(wrap_pyfunction!(py_integrate,m)?)?;
  m.add_function(wrap_pyfunction!(py_interpolate_time,m)?)?;
  m.add_function(wrap_pyfunction!(py_cfd_simple,m)?)?;
  m.add_function(wrap_pyfunction!(py_find_zero_crossings,m)?)?;
  Ok(())
}

#[pymodule]
#[pyo3(name = "moni")]
fn tof_moni<'_py>(m: &Bound<'_py, PyModule>) -> PyResult<()> {
  m.add_class::<PyPAMoniSeries>()?;
  m.add_class::<PyPBMoniSeries>()?;
  m.add_class::<PyRBMoniSeries>()?;
  m.add_class::<PyMtbMoniSeries>()?;
  m.add_class::<PyCPUMoniSeries>()?;
  m.add_class::<PyLTBMoniSeries>()?;
  m.add_class::<PyRBMoniData>()?;
  m.add_class::<PyPAMoniData>()?;
  m.add_class::<PyPBMoniData>()?;
  m.add_class::<PyLTBMoniData>()?;
  m.add_class::<PyMtbMoniData>()?;
  m.add_class::<PyTofDetectorStatus>()?;
  Ok(())
}


/// I/O features to read TofPackets from disk
#[pymodule]
#[pyo3(name = "io")]
fn tof_io<'_py>(m: &Bound<'_py, PyModule>) -> PyResult<()> {
  m.add_function(wrap_pyfunction!(py_create_rb_ch_to_pid_map, m)?)?;
  m.add_function(wrap_pyfunction!(py_create_rb_ch_to_pid_a_map, m)?)?;
  m.add_function(wrap_pyfunction!(py_create_rb_ch_to_pid_b_map, m)?)?;
  m.add_function(wrap_pyfunction!(py_create_mtb_connection_to_pid_map, m)?)?;
  m.add_function(wrap_pyfunction!(py_summarize_toffile, m)?)?;
  m.add_class::<PyTofPacket>()?;
  m.add_class::<PyTofPacketReader>()?;
  m.add_class::<PacketType>()?;
  Ok(())
}

/// Event structures for the TOF part of the GAPS experiment
#[pymodule]
#[pyo3(name = "events")]
fn tof_events<'_py>(m: &Bound<'_py, PyModule>) -> PyResult<()> {
  m.add_class::<PyMasterTriggerEvent>()?;
  m.add_class::<PyRBEvent>()?;
  m.add_class::<PyRBEventHeader>()?;
  m.add_class::<PyTofEvent>()?;
  m.add_class::<PyTofHit>()?;
  m.add_class::<PyRBWaveform>()?;
  m.add_class::<PyRBCalibration>()?;
  m.add_class::<PyTofEventSummary>()?;
  m.add_class::<LTBThreshold>()?;
  m.add_class::<EventStatus>()?;
  Ok(())
}

cfg_if::cfg_if! {
  if #[cfg(feature = "telemetry")] {
    #[pymodule]
    #[pyo3(name = "telemetry")]
    fn py_telemetry<'_py> (m: &Bound<'_py, PyModule>) -> PyResult<()> {
      //m.add_function(wrap_pyfunction!(get_gapsevents,m)?)?;
      m.add_class::<tel_api::TelemetryPacketType>()?;
      m.add_class::<PyTelemetryPacket>()?;
      m.add_class::<PyTelemetryPacketReader>()?;
      m.add_class::<PyMergedEvent>()?;
      //m.add_class::<PyGapsEvent>()?;
      //m.add_class::<PyTofHit>()?;
      //m.add_class::<PyTofEventSummary>()?;
      m.add_class::<PyTrackerHit>()?;
      m.add_class::<PyTrackerHitV2>()?;
      m.add_class::<PyTrackerEvent>()?;
      m.add_class::<PyTrackerPacket>()?;
      m.add_class::<PyTrackerTempLeakPacket>()?;
      m.add_class::<PyGPSPacket>()?;
      m.add_class::<PyTrackerDAQTempPacket>()?;
      m.add_class::<PyTrackerDAQHSKPacket>()?;
      m.add_class::<PyTrackerEventIDEchoPacket>()?;
      Ok(())
    }
  }
}

cfg_if::cfg_if! {
  if #[cfg(feature = "liftof")] {
   /// Python API to rust version of tof-dataclasses.
   ///
   /// Currently, this contains only the analysis 
   /// functions
    #[pymodule]
    #[pyo3(name = "liftof")]
    fn py_liftof<'_py> (m: &Bound<'_py, PyModule>) -> PyResult<()> {
      m.add_function(wrap_pyfunction!(py_waveform_analysis,m)?)?;
      m.add_function(wrap_pyfunction!(wrap_calc_edep_simple,m)?)?;
      m.add_function(wrap_pyfunction!(test_db,m)?)?;
      m.add_function(wrap_pyfunction!(wrap_prescale_to_u32,m)?)?;
      m.add_function(wrap_pyfunction!(wrap_fit_sine_sydney,m)?)?;
      m.add_class::<PyLiftofSettings>()?;
      m.add_class::<PyIPBus>()?;
      m.add_class::<PyMasterTrigger>()?;
      Ok(())
    }
  }
}

cfg_if::cfg_if! {
  if #[cfg(feature = "caraspace-serial")] {
    #[pymodule]
    #[pyo3(name = "caraspace")]
    fn py_caraspace<'_py> (m: &Bound<'_py, PyModule>) -> PyResult<()> {
      m.add_function(wrap_pyfunction!(py_parse_u8, m)?)?;
      m.add_function(wrap_pyfunction!(py_parse_u16, m)?)?;
      m.add_function(wrap_pyfunction!(py_parse_u32, m)?)?;
      m.add_function(wrap_pyfunction!(py_parse_u64, m)?)?;
      //m.add_class::<tel_api::TelemetryPacketType>()?;
      m.add_class::<PyCRFrame>()?;
      m.add_class::<PyCRReader>()?;
      m.add_class::<PyCRWriter>()?;
      Ok(())
    }
  }
}

/// Commands for the whole TOF system
#[pymodule]
#[pyo3(name = "commands")]
fn tof_commands<'_py>(m: &Bound<'_py, PyModule>) -> PyResult<()> {
  m.add_class::<TofCommandCode>()?;
  m.add_class::<PyTofCommand>()?;
  m.add_class::<PyTriggerConfig>()?;
  m.add_class::<PyAnalysisEngineConfig>()?;
  m.add_class::<PyTOFEventBuilderConfig>()?;
  m.add_class::<PyHeartBeatDataSink>()?;
  m.add_class::<PyMTBHeartbeat>()?;
  m.add_class::<PyEVTBLDRHeartbeat>()?;
  Ok(())
}

/// Python API to rust version of tof-dataclasses.
///
/// Currently, this contains only the analysis 
/// functions
#[pymodule]
#[pyo3(name = "go_pybindings")]
fn go_pybindings<'_py>(m : &Bound<'_py, PyModule>) -> PyResult<()> { //: Python<'_>, m: &PyModule) -> PyResult<()> {
  pyo3_log::init();
  //m.add_function(wrap_pyfunction!(py_get_periods,m)?)?;
  m.add_wrapped(wrap_pymodule!(tof_analysis))?;
  m.add_wrapped(wrap_pymodule!(tof_moni))?;
  m.add_wrapped(wrap_pymodule!(tof_io))?;
  m.add_wrapped(wrap_pymodule!(tof_events))?;
  m.add_wrapped(wrap_pymodule!(tof_commands))?;
  #[cfg(feature="telemetry")]
  m.add_wrapped(wrap_pymodule!(py_telemetry))?;
  #[cfg(feature="liftof")]
  m.add_wrapped(wrap_pymodule!(py_liftof))?;
  #[cfg(feature="caraspace-serial")]
  m.add_wrapped(wrap_pymodule!(py_caraspace))?;
  Ok(())
}
