//! Statistics tools 
//!
// This file is part of gaps-online-software and published 
// under the GPLv3 license

use statrs::distribution::{Gamma, Continuous};
use num_traits::{
  Float,
  NumAssign
};

#[cfg(feature="pybindings")]
use pyo3::prelude::*;

#[cfg(feature="pybindings")]
use numpy::{
  PyArray1,
  PyArrayMethods,
};

/// Calculates the standard deviation of a vector.
///
/// This function returns an Option<f32> because the standard deviation is
/// undefined for an empty vector or a vector with a single element.
pub fn standard_deviation(data: &Vec<f32>) -> Option<f32> {
  // The standard deviation is not defined for vectors with less than 2 elements.
  if data.len() < 2 {
    return None;
  }

  // First, calculate the mean of the data.
  let mean = mean(data);

  // Then, calculate the sum of the squared differences from the mean.
  // This is the variance.
  let variance_sum: f32 = data.iter()
  .map(|x| (x - mean).powi(2))
  .sum();

  // The sample standard deviation is the square root of the variance
  // divided by (n-1), where n is the number of data points.
  let variance = variance_sum / ((data.len() - 1) as f32);

  // Take the square root to get the standard deviation.
  Some(variance.sqrt())
}




/// Calculate the provided statistic over the columns of the given matrix 
/// representation, respecting nans 
///
/// <div class="info">
/// **Note:** We don't want an external dependency for this allone,
/// but might switch to ndarray in the future.
/// </div>
///
/// # Arguments:
///
///   * data: input data in form of a 2d matrix
///   * func: statistics to apply, e.g. mean or median
pub fn calculate_column_stat<T, F>(data: &Vec<Vec<T>>, func: F) -> Vec<T> 
  where T: Float + NumAssign + Copy,
        F: Fn(&[T]) -> T {  
  let num_columns = data[0].len();
  let num_rows    = data.len();
  // Initialize a Vec to store the column-wise medians
  let mut column_stats: Vec<T> = vec![T::zero(); num_columns];
  debug!("Calculating stat for {} columns!", num_columns);
  debug!("Calculating stat for {} rows!", num_rows);
  // Calculate the median for each column across all sub-vectors, ignoring NaN values
  for col in 0..num_columns  {
    let mut col_vals: Vec<T> = vec![T::zero(); num_rows];
    //let mut col_vals = Vec::<f32>::new();
    for k in 0..num_rows {
      col_vals[k] = data[k][col];
    }
    col_vals.retain(|x| !x.is_nan());
    if col_vals.len() == 0 {
      column_stats[col] = T::nan();
    } else {
      //column_medians[col] = statistical::median(col_vals.as_slice());//.unwrap_or(f32::NAN);
      column_stats[col] = func(col_vals.as_slice());
    }
  }
  column_stats
}


/// Simply calculate the mean of a vector of numbers
///
/// <div class="info">
/// **Note:** We don't want an external dependency for thisallone,
/// but might switch to ndarray in the future.
/// </div>
pub fn mean<T>(input: &[T]) -> T 
  where T: Float + NumAssign + Copy {  
  if input.len() == 0 {
    error!("Vector is empty, can not calculate mean!");
    return T::nan();
  }
  let mut n_entries = T::zero();
  let mut sum       = T::zero();
  for k in input.iter() {
    if k.is_nan() {
      continue;
      
    }
    sum += *k;
    n_entries += T::one();
  }
  sum / n_entries
}

/// A simple gamma function e.g. to generate fake waveforms
pub fn gamma_pdf(xs : &[f32], shape : f64, scale : f64) -> Vec<f32> {
  let mut ys = Vec::<f32>::with_capacity(xs.len());
  
  let gamma = Gamma::new(shape, scale).unwrap();
  for x in xs {
    ys.push(gamma.pdf(*x as f64) as f32);
  }
  return ys;
}

//---------------------------------------------------

#[cfg(feature="pybindings")]
#[pyfunction]
#[pyo3(name="gamma_pdf")]
pub fn py_gamma_pdf<'_py>(xs    : Bound<'_py,PyArray1<f32>>,
                          shape : f64,
                          scale : f64) -> Vec<f32> {
  let ys : Vec::<f32>;
  unsafe {
    ys = gamma_pdf(xs.as_slice().unwrap(), shape, scale);
  }
  return ys;
}

#[cfg(feature="pybindings")]
#[pyfunction]
#[pyo3(name="mean")]
pub fn py_mean<'_py>(xs    : Bound<'_py,PyArray1<f32>>) -> f32 { 
  let mean_val : f32;
  unsafe {
    mean_val = mean(xs.as_slice().unwrap());
  }
  mean_val
}

