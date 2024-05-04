//! Pybindings for the RUST dataclasses
//!
pub mod analysis;
pub mod io;
pub mod dataclasses;

use pyo3::prelude::*;

use crate::analysis::*;
use crate::dataclasses::*;
use crate::io::*;


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

/// Python API to rust version of tof-dataclasses.
///
/// Currently, this contains only the analysis 
/// functions
#[pymodule]
#[pyo3(name = "rpy_tof_dataclasses")]
fn rpy_tof_dataclasses<'_py>(m : &Bound<'_py, PyModule>) -> PyResult<()> { //: Python<'_>, m: &PyModule) -> PyResult<()> {
  pyo3_log::init();
  m.add_function(wrap_pyfunction!(py_get_periods,m)?)?;
  m.add_function(wrap_pyfunction!(py_time2bin,m)?)?;
  m.add_function(wrap_pyfunction!(py_find_peaks,m)?)?;
  m.add_function(wrap_pyfunction!(py_find_peaks_zscore,m)?)?;
  m.add_function(wrap_pyfunction!(py_integrate,m)?)?;
  m.add_function(wrap_pyfunction!(py_interpolate_time,m)?)?;
  m.add_function(wrap_pyfunction!(py_cfd_simple,m)?)?;
  m.add_function(wrap_pyfunction!(py_find_zero_crossings,m)?)?;
  //m.add_function(wrap_pyfunction!(test_waveform_analysis,m)?)?;
  //m.add_function(wrap_pyfunction!(wrap_calc_edep_simple,m)?)?;
  //m.add_function(wrap_pyfunction!(test_db,m)?)?;
  m.add_class::<PyTofPacket>()?;
  m.add_class::<PyTofPacketReader>()?;
  m.add_class::<PyPAMoniSeries>()?;
  m.add_class::<PyPBMoniSeries>()?;
  m.add_class::<PyRBMoniSeries>()?;
  m.add_class::<PyMtbMoniSeries>()?;
  m.add_class::<PyCPUMoniSeries>()?;
  m.add_class::<PyLTBMoniSeries>()?;
  //m.add_class::<PyPAMoniSeries>()?;
  //m.add_class::<PyIPBus>()?;
  //m.add_class::<PyMasterTrigger>()?;
  //m.add_class::<PyMasterTriggerEvent>()?;
  //m.add_class::<PyRBCalibration>()?;
  Ok(())
}
