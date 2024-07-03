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

//#[pyfunction]
//#[pyo3(name="test_db")]
//pub fn test_db() {
//  let mut conn = connect_to_db(String::from("/srv/gaps/gaps-online-software/gaps-db/gaps_db/gaps_flight.db")).unwrap();
//  let rats = RAT::all(&mut conn).unwrap();
//  for r in rats {
//    println!("{}", r);
//  }
//  let dsis = DSICard::all(&mut conn).unwrap();
//  for dsi in dsis {
//    println!("{}", dsi);
//  }
//  let paddles = Paddle::all(&mut conn).unwrap();
//  for pdl in paddles {
//    println!("{}", pdl);
//  }
//  let mtbch = MTBChannel::all(&mut conn).unwrap();
//  for chnl in mtbch {
//    println!("{}", chnl);
//  }
//  let ltbs = LocalTriggerBoard::all(&mut conn).unwrap();
//  for ltb in ltbs {
//    println!("{}", ltb);
//  }
//  let rbs = ReadoutBoard::all(&mut conn).unwrap();
//  for rb in rbs {
//    println!("{}", rb);
//  }
//  let panels = Panel::all(&mut conn).unwrap();
//  for pnl in panels {
//    println!("{}", pnl);
//  }
//}


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
  Ok(())
}


/// I/O features to read TofPackets from disk
///
/// # Example 
/// ```
/// import gaps_online.rust_api as api
/// # read 100 TofEvents from the file
/// reader = api.io.TofPacketReader("/path/to/your/file", filter=api.io.PacketType.TofEvent, nevents=100)
/// for pack in reader:
///    ev = api.events.TofEvent()
///    ev.from_tofpacket(ev)
///    for h in ev.hits:
///        print (h.t0)
///        print (h)
///
///
/// ```
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
  m.add_class::<PyRBWaveform>()?;
  m.add_class::<PyRBCalibration>()?;
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
