use pyo3::prelude::*;
use pyo3::exceptions::PyValueError;

use numpy::PyArray1;
use numpy::PyArrayMethods;

use tof_dataclasses::analysis::{
    find_peaks,
    find_peaks_zscore,
    interpolate_time,
    cfd_simple,
    integrate,
    time2bin,
    calc_edep_simple
};

use tof_dataclasses::calibrations::{
    find_zero_crossings,
    get_periods,
    Edge,
};

///helper
fn convert_pyarray1<'_py>(arr : Bound<'_py, PyArray1<f32>>) -> Vec<f32> {
  let mut vec = Vec::<f32>::new();
  unsafe {
    vec.extend_from_slice(arr.as_slice().unwrap());
  }
  return vec;
}

#[pyfunction]
#[pyo3(name="get_periods")]
pub fn py_get_periods<'_py>(trace   : Bound<'_py, PyArray1<f32>>,
                            dts     : Bound<'_py, PyArray1<f32>>,
                            nperiod : f32,
                            nskip   : f32)
    -> PyResult<(Vec<usize>, Vec<f32>)> {
  // we fix the edge here
  let edge = Edge::Rising;
  let wr_trace : Vec<f32>;
  let wr_dts   : Vec<f32>;
  wr_trace = convert_pyarray1(trace);
  wr_dts   = convert_pyarray1(dts);
  let result   = get_periods(&wr_trace, &wr_dts, nperiod, nskip, &edge);
  Ok(result)
}



#[pyfunction]
#[pyo3(name="calc_edep_simple")]
pub fn py_calc_edep_simple(peak_voltage : f32) -> f32 {
  calc_edep_simple(peak_voltage)
}

#[pyfunction]
#[pyo3(name="find_zero_crossings")]
/// Get a vector with the indizes where 
/// the input array crosses zero
pub fn py_find_zero_crossings<'_py>(trace : Bound<'_py,PyArray1<f32>>) 
  -> PyResult<Vec<usize>> {
  let tr  = convert_pyarray1(trace);
  let zcs = find_zero_crossings(&tr);
  Ok(zcs)
}

#[pyfunction]
#[pyo3(name="cfd_simple")]
/// Find the peak onset time based on a cfd
/// "Constant fraction discrimination" algorithm
///
/// # Arguments
///
/// * start_peak : bin
/// * end_peak   : bin
/// * cfd_frac   : 0.2 is the typical default
pub fn py_cfd_simple<'_py>(voltages    : Bound<'_py,PyArray1<f32>>,
                           nanoseconds : Bound<'_py,PyArray1<f32>>,
                           cfd_frac    : f32,
                           start_peak  : usize,
                           end_peak    : usize) -> PyResult<f32> {
  let voltages_vec    = convert_pyarray1(voltages);
  let nanoseconds_vec = convert_pyarray1(nanoseconds);
  match cfd_simple(&voltages_vec   ,
                   &nanoseconds_vec,
                   cfd_frac       ,
                   start_peak  ,
                   end_peak) {
    Ok(result) => Ok(result),
    Err(err)   => {
     return Err(PyValueError::new_err(err.to_string()));
    } 
  }
}

#[pyfunction]
#[pyo3(name="interpolate_time")]
pub fn py_interpolate_time<'_py>(voltages    : Bound<'_py,PyArray1<f32>>,
                                 nanoseconds : Bound<'_py,PyArray1<f32>>,
                                 threshold   : f32,
                                 idx         : usize,
                                 size        : usize) -> PyResult<f32> {
  let mut voltages_vec    = Vec::<f32>::new();
  let mut nanoseconds_vec = Vec::<f32>::new(); 
  unsafe {
    voltages_vec.extend_from_slice(voltages.as_slice().unwrap());
    nanoseconds_vec.extend_from_slice(nanoseconds.as_slice().unwrap());
  }
  match interpolate_time (&voltages_vec   ,
                          &nanoseconds_vec, 
                          threshold      ,
                          idx            ,
                          size) {
   Ok(result) => Ok(result),
   Err(err)   => {
    return Err(PyValueError::new_err(err.to_string()));
   } 
  }
}

#[pyfunction]
#[pyo3(name="time2bin")]
pub fn py_time2bin<'_py>(nanoseconds : Bound<'_py,PyArray1<f32>>,
                         t_ns        : f32) -> PyResult<usize> {
 let mut nanoseconds_vec = Vec::<f32>::new(); 
 unsafe {
   nanoseconds_vec.extend_from_slice(nanoseconds.as_slice().unwrap());
 }
 match time2bin (&nanoseconds_vec,
                 t_ns){
   Ok(result) => Ok(result),
   Err(err)   => {
    return Err(PyValueError::new_err(err.to_string()));
   } 
 }
}

#[pyfunction]
#[pyo3(name="integrate")]
pub fn py_integrate<'_py>(voltages    : Bound<'_py,PyArray1<f32>>,
                          nanoseconds : Bound<'_py,PyArray1<f32>>,
                          lower_bin   : usize,
                          upper_bin   : usize,
                          impedance   : f32) -> PyResult<f32>  {
 let mut voltages_vec    = Vec::<f32>::new();
 let mut nanoseconds_vec = Vec::<f32>::new(); 
 unsafe {
   voltages_vec.extend_from_slice(voltages.as_slice().unwrap());
   nanoseconds_vec.extend_from_slice(nanoseconds.as_slice().unwrap());
 }
 match integrate(&voltages_vec, &nanoseconds_vec, lower_bin, upper_bin, impedance) {
   Ok(result) => Ok(result),
   Err(err)   => {
    return Err(PyValueError::new_err(err.to_string()));
   }
 }
}

#[pyfunction]
#[pyo3(name = "find_peaks")]
/// The GAPS peak finding algorithm, based on 
/// legacy code written by the UCLA TOF team.
///
/// This needs to be applied AFTER the peakfinding
/// and takes a specific peak as input argument
///
/// # Arguments
/// 
/// * voltages     (np.ndarray) | These both together
/// * nanosecondes (np.ndarray) | are a calibrated waveform
/// * start_time   (float)      - begin peak search at this time
/// * window_size  (float)      - limit peak search to start_time 
///                               + start_time + window_size (in ns)
/// * min_peak_width (usize)    - If a peak has a lower width, it 
///                               will get discarded (in bins)
/// * threshold      (f32)      - Ingore peaks which fall below this
///                               voltage (in mV)
/// * max_peaks      (usize)    - Stop peak search after max_peaks are
///                              found
pub fn py_find_peaks<'_py>(voltages       : Bound<'_py, PyArray1<f32>>,
                           nanoseconds    : Bound<'_py, PyArray1<f32>>,
                           start_time     : f32,
                           window_size    : f32,
                           min_peak_width : usize,
                           threshold      : f32,
                           max_peaks      : usize) -> PyResult<Vec<(usize,usize)>> {
 let mut voltages_vec    = Vec::<f32>::new();
 let mut nanoseconds_vec = Vec::<f32>::new(); 
 unsafe {
   voltages_vec.extend_from_slice(voltages.as_slice().unwrap());
   nanoseconds_vec.extend_from_slice(nanoseconds.as_slice().unwrap());
 }

 match find_peaks(&voltages_vec  , 
                  &nanoseconds_vec   , 
                  start_time    , 
                  window_size   , 
                  min_peak_width, 
                  threshold     , 
                  max_peaks     ) {
   Ok(result) => Ok(result),
   Err(err)   => {
    return Err(PyValueError::new_err(err.to_string()));
   }
 }
}

#[pyfunction]
#[pyo3(name = "find_peaks_zscore")]
pub fn py_find_peaks_zscore<'_py>(voltages       : Bound<'_py,PyArray1<f32>>,
                                  nanoseconds    : Bound<'_py,PyArray1<f32>>,
                                  start_time     : f32,
                                  window_size    : f32,
                                  lag            : usize,
                                  threshold      : f64,
                                  influence      : f64) -> PyResult<Vec<(usize,usize)>> {
 let mut voltages_vec    = Vec::<f32>::new();
 let mut nanoseconds_vec = Vec::<f32>::new(); 
 unsafe {
   voltages_vec.extend_from_slice(voltages.as_slice().unwrap());
   nanoseconds_vec.extend_from_slice(nanoseconds.as_slice().unwrap());
 }

 match find_peaks_zscore(&nanoseconds_vec, 
                         &voltages_vec   ,   
                         start_time      , 
                         window_size     , 
                         lag             , 
                         threshold       , 
                         influence) {
   Ok(result) => Ok(result),
   Err(err)   => {
     return Err(PyValueError::new_err(err.to_string()));
   }
 }
}

