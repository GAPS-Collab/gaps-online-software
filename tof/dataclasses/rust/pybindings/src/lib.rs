//! Pybindings for the RUST dataclasses
//!
pub mod analysis;
pub mod io;
pub mod dataclasses;

use pyo3::prelude::*;
use pyo3::wrap_pymodule;

use crate::analysis::*;
use crate::dataclasses::*;
use crate::io::*;

// these are already wrapped in a pyclass (enum)
use tof_dataclasses::packets::PacketType;
use tof_dataclasses::commands::TofCommandCode;
use tof_dataclasses::events::master_trigger::LTBThreshold;

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
  m.add_class::<PyMtbMoniData>()?;
  Ok(())
}


/// I/O features to read TofPackets from disk
#[pymodule]
#[pyo3(name = "io")]
fn tof_io<'_py>(m: &Bound<'_py, PyModule>) -> PyResult<()> {
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
  Ok(())
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
#[pyo3(name = "rpy_tof_dataclasses")]
fn rpy_tof_dataclasses<'_py>(m : &Bound<'_py, PyModule>) -> PyResult<()> { //: Python<'_>, m: &PyModule) -> PyResult<()> {
  pyo3_log::init();
  //m.add_function(wrap_pyfunction!(py_get_periods,m)?)?;
  m.add_wrapped(wrap_pymodule!(tof_analysis))?;
  m.add_wrapped(wrap_pymodule!(tof_moni))?;
  m.add_wrapped(wrap_pymodule!(tof_io))?;
  m.add_wrapped(wrap_pymodule!(tof_events))?;
  m.add_wrapped(wrap_pymodule!(tof_commands))?;
  //m.add_function(wrap_pyfunction!(test_waveform_analysis,m)?)?;
  //m.add_function(wrap_pyfunction!(wrap_calc_edep_simple,m)?)?;
  //m.add_function(wrap_pyfunction!(test_db,m)?)?;
  //m.add_class::<PyPAMoniSeries>()?;
  //m.add_class::<PyIPBus>()?;
  //m.add_class::<PyMasterTrigger>()?;
  //m.add_class::<PyMasterTriggerEvent>()?;
  //m.add_class::<PyRBCalibration>()?;
  Ok(())
}
