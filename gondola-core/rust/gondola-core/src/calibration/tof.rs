//! The following file is part of gaps-online-software and published 
//! under the GPLv3 license
//!
//! Calibration routines for the GAPS TOF system
//!

//#[cfg(feature="random")]
//use rand;
#[cfg(feature="pybindings")]
use pyo3::prelude::*;
#[cfg(feature="pybindings")]
use numpy::PyArray1;
//#[cfg(feature="pybindings")]
//use pyo3::exceptions::PyIOError;
//
//#[cfg(feature="pybindings")]
//use crate::packets::TofPacket;
//
//#[cfg(feature="pybindings")]
//use crate::impl_pythonize_display;

/// Roll over the entries from the end of a vector 
/// to the beginning by a given offset.
/// 
/// This is similar to 
/// https://numpy.org/doc/2.2/reference/generated/numpy.roll.html
///
/// # Arguments:
///   * `vec`   : The vector to be rolled over. It will 
///               be changed in place 
///   * `offset`: The signed number to shift elements by (can be to 
///               the left or right)
pub fn roll<T: Clone>(vec: &mut Vec<T>, offset: isize) {
  let len = vec.len() as isize;
  if len <= 1 {
      return;
  }
  let offset = offset % len;
  if offset == 0 {
      return;
  }
  let split_point = if offset > 0 {
      len - offset
  } else {
      -offset
  } as usize;

  let mut temp = Vec::with_capacity(len as usize);
  temp.extend_from_slice(&vec[split_point..]);
  temp.extend_from_slice(&vec[..split_point]);

  vec.clear();
  vec.extend_from_slice(&temp);
}

//-----------------------------------------------

/// Simplified version of spike cleaning 
///
/// Taken over from Jamie's python code
pub fn clean_spikes(trace : &mut Vec<f32>, vcaldone : bool) {
  //# TODO: make robust (symmetric, doubles, fixed/estimated spike height)
  let mut thresh : f32 = 360.0;
  if vcaldone {
    thresh = 16.0;
  }

  let mut spf_allch = vec![0usize;1023];
  let mut spf_sum   = vec![0f32;1024];
  let tracelen      = trace.len();
  let spikefilter0 = &trace[0..tracelen-3];
  let spikefilter1 = &trace[1..tracelen-2];
  let spikefilter2 = &trace[2..tracelen-1];
  let spikefilter3 = &trace[3..tracelen];
  let spf_len      = spikefilter0.len();
  for k in 0..spf_len {
    spf_sum[k] += spikefilter1[k] - spikefilter0[k] + spikefilter2[k] - spikefilter3[k];
  }
  for k in 0..spf_len {
    if spf_sum[k] > thresh {
      spf_allch[k] += 1;
    }
  }
  let mut spikes = Vec::<usize>::new();
  for k in 0..spf_allch.len() {
    if spf_allch[k] >= 2 {
      spikes.push(k);
    }
  }
  for spike in spikes.iter() {
    let d_v : f32 = (trace[spike+3] - trace[*spike])/3.0;
    trace[spike+1] = trace[*spike] + d_v;
    trace[spike+2] = trace[*spike] + 2.0*d_v;
  }
}

//-----------------------------------------------

//#[cfg(feature="pybindings")]
//#[pyfunction]
//#[pyo3(name="clean_spikes")]
//pub fn clean_spikes_pyx<'_py>(value : Bound<'_py,PyArray1<f32>>, vcal_done : bool) {
//  unsafe {
//    match clean_spikes(value.as_slice().unwrap(), vcal_done) {
//      Err(err) => {
//        return Err(PyValueError::new_err(err.to_string()));
//      }
//      Ok(max_val) => {
//        return Ok(max_val);
//      }
//    }
//  }
//}




//-----------------------------------------------

// new way of property testing using quicktest!
#[test]
fn prop_roll_then_unroll_gives_original() {
  fn prop(mut vec: Vec<u8>, offset: i8) -> bool {
    let original = vec.clone();
    let offset = offset as isize;

    roll(&mut vec, offset);
    roll(&mut vec, -offset);

    vec == original
  }
  quickcheck::QuickCheck::new().tests(100).quickcheck(prop as fn(Vec<u8>, i8) -> bool);
}


