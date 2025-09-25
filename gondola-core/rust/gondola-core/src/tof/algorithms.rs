//! Algorithms used to exract information from 
//! the TOF waveforms.
// This file is part of gaps-online-software and published 
// under the GPLv3 license

use crate::prelude::*;

/// Return the index of the maximum value in an 
/// array of floats. 
///
/// Protip: f32 does not obey Ord, because of NaN, 
///         so this is done "by hand"
///
/// # Arguments:
///
///
pub fn get_max_value_idx<T : std::cmp::PartialOrd + std::fmt::Display + Copy>(values    : &[T],
         start_idx : usize,
         n_idx     : usize) -> Result<usize, WaveformError> {
  if start_idx >= values.len() {
    error!("Invalid value for start index {}", start_idx);
    return Err(WaveformError::OutOfRangeLowerBound);
  }
  if start_idx + n_idx >= values.len() {
    error!("Start index {} + n steps of {} is larger tan array size!", start_idx, n_idx); 
    return Err(WaveformError::OutOfRangeUpperBound);
  }
  let mut maxval   = values[start_idx];
  let mut maxbin = start_idx;
  for n in start_idx..start_idx + n_idx {
    if values[n] > maxval {
      maxval  = values[n];
      maxbin  = n;
    }
  } // end for
  trace!("Got index {} for a max value of {}", maxbin, maxval);
  Ok(maxbin)
} // end fn

//---------------------------------------------------

#[cfg(feature="pybindings")]
#[pyfunction]
#[pyo3(name="get_max_value_idx")]
pub fn get_max_value_idx_py<'_py>(value : Bound<'_py,PyArray1<f32>>,
                                  start_idx :usize,
                                  n_idx : usize) -> PyResult<usize> {
  unsafe {
    match get_max_value_idx::<f32>(value.as_slice().unwrap(), start_idx, n_idx) {
      Err(err) => {
        return Err(PyValueError::new_err(err.to_string()));
      }
      Ok(max_val) => {
        return Ok(max_val);
      }
    }
  }
}

/// Linear interpolation of the time within a single bin of a TOF waveform
///
/// # Arguments:
///   * voltages    : Waveform in mV 
///   * nanoseconds : Calibrated time for the waveform bins in ns 
///   * threshold   : Threshold in mV which is supposed to be crossed 
///                   within the bin 
///   * idx         : Together with size define a range for the search for 
///                   the bin which should have the implementation applied 
///                   to [voltages[idx], voltages[idx + size]]
///   * size        : Together with idx define a range for the search for 
///                   the bin which should have the implementation applied 
///                   to [voltages[idx], voltages[idx + size]]
pub fn interpolate_time<T : AsRef<[f32]>> (volts         : &T,
                                           times         : &T, 
                                           mut threshold : f32,
                                           mut idx       : usize,
                                           size          : usize) -> Result<f32, WaveformError> {
  let voltages    = volts.as_ref();
  let nanoseconds = times.as_ref();
  if idx + 1 > nanoseconds.len() {
    return Err(WaveformError::OutOfRangeUpperBound);
  }
  threshold     = threshold.abs();
  let mut lval  = (voltages[idx]).abs();
  let mut hval : f32 = 0.0; 
  if size == 1 {
    hval = (voltages[idx+1]).abs();
  } else {
    for n in idx+1..idx+size {
      hval = voltages[n].abs();
      if (hval>=threshold) && (threshold<=lval) { // Threshold crossing?
        idx = n-1; // Reset idx to point before crossing
        break;
      }
      lval = hval;
    }
  }
  if ((lval > threshold) && (size != 1)) || lval == hval {
    return Ok(nanoseconds[idx]);
  } else {
    return Ok(nanoseconds[idx] 
          + (threshold-lval)/(hval-lval) * (nanoseconds[idx+1]
          - nanoseconds[idx]));
  }
}


#[pyfunction]
#[pyo3(name="interpolate_time")]
/// Linear interpolation of the time within a single bin of a TOF waveform
///
/// # Arguments:
///   * voltages    : Waveform in mV 
///   * nanoseconds : Calibrated time for the waveform bins in ns 
///   * threshold   : Threshold in mV which is supposed to be crossed 
///                   within the bin 
///   * idx         : Together with size define a range for the search for 
///                   the bin which should have the implementation applied 
///                   to [voltages[idx], voltages[idx + size]]
///   * size        : Together with idx define a range for the search for 
///                   the bin which should have the implementation applied 
///                   to [voltages[idx], voltages[idx + size]]
pub fn interpolate_time_py(voltages    : PyReadonlyArray1<f32>,
                           nanoseconds : PyReadonlyArray1<f32>,
                           threshold   : f32,
                           idx         : usize,
                           size        : usize) -> PyResult<f32> {
  let i   = idx;
  match interpolate_time(&voltages.readonly().as_slice().unwrap(),
                         &nanoseconds.readonly().as_slice().unwrap(),
                         threshold, i, size) {
    Ok(time) => {
      return Ok(time);
    }
    Err(err) => {
      return Err(PyValueError::new_err(err.to_string()));
    }
  }
}


/// Integrate a waveform
///
/// That this works right, prior to the 
/// integration we should subtract the 
/// baseline.
///
/// # Arguments:
///
/// * impedance : typically this is 
pub fn integrate(voltages     : &Vec<f32>,
                 nanoseconds  : &Vec<f32>,
                 lo_bin       : usize,
                 upper_bin    : usize,
                 impedance    : f32) -> Result<f32, WaveformError>  {
  if upper_bin > voltages.len() {
    return Err(WaveformError::OutOfRangeUpperBound);
  }
  if lo_bin < 1 {
    return Err(WaveformError::OutOfRangeLowerBound);
  }
  let mut sum = 0f32;
  for n in lo_bin..upper_bin {
    sum += voltages[n] * (nanoseconds[n] - nanoseconds[n-1]) ;
  }
  sum /= impedance;
  Ok(sum)
}

/// Given a time in ns, find the bin most closely corresponding to that time
/// # Arguments
/// 
pub fn time2bin(nanoseconds : &Vec<f32>,
                t_ns        : f32) -> Result<usize, WaveformError> {
  for n in 0..nanoseconds.len() {
    if nanoseconds[n] > t_ns {
      return Ok(n-1);
    }
  }
  debug!("Did not find a bin corresponding to the given time {}!", t_ns);
  return Err(WaveformError::TimesTooSmall);
}

/// The pedestal is the baseline of the waveform
///
/// # Arguments
///
/// * voltages      : calibrated waveform
/// * threshold     : consider everything below threshold
///                   the pedestal (typical 10mV)
/// * ped_begin_bin : beginning of the window for pedestal
///                   calculation (bin)
/// * ped_range_bin : length of the window for pedestal
///                   calculation (in bins)
///
/// # Return
/// pedestal value with error (quadratic error)
pub fn calculate_pedestal(voltages      : &Vec<f32>,
                          threshold     : f32,
                          ped_begin_bin : usize,
                          ped_range_bin : usize) -> (f32,f32) {
  let mut sum  = 0f32;
  let mut sum2 = 0f32;
  for k in ped_begin_bin..ped_begin_bin + ped_range_bin {
    if f32::abs(voltages[k]) < threshold {
      sum  += voltages[k];
      sum2 += voltages[k]*voltages[k];
    }
  }
  let average = sum/(ped_range_bin as f32);
  let sigma   = f32::sqrt(sum2/(ped_range_bin as f32 - (average*average)));
  (average, sigma)
}

/// Find the onset time of a peak with a 
/// constant fraction discrimination method.
///
/// The peaks have to be sane
/// FIXME: Maybe introduce a separate check?
pub fn cfd_simple(voltages    : &Vec<f32>,
                  nanoseconds : &Vec<f32>,
                  cfd_frac    : f32,
                  start_peak  : usize,
                  end_peak    : usize) -> Result<f32, WaveformError> {

  let idx = get_max_value_idx(&voltages, start_peak, end_peak-start_peak)?;
  let mut sum : f32 = 0.0;
  for n in idx-1..idx+1{
    sum += voltages[n];
  }
  let tmp_thresh : f32 = f32::abs(cfd_frac * (sum / 3.0));
  trace!("Calculated tmp threshold of {}", tmp_thresh);
  // Now scan through the waveform around the peak to find the bin
  // crossing the calculated threshold. Bin idx is the peak so it is
  // definitely above threshold. So let's walk backwards through the
  // trace until we find a bin value less than the threshold.
  let mut lo_bin : usize = voltages.len();
  let mut n = idx;
  if idx < start_peak {
    error!("The index {} is smaller than the beginning of the peak {}!", idx, start_peak);
    return Err(WaveformError::OutOfRangeLowerBound);
  }
  if start_peak >= 10 {
    while n > start_peak - 10 {
      if f32::abs(voltages[n]) < tmp_thresh {
        lo_bin = n;
        break;
      }
      n -= 1;
    }  
  } else {
    debug!("We require that the peak is at least 10 bins away from the start!");
    return Err(WaveformError::OutOfRangeLowerBound);
  }

  trace!("Lo bin {} , start peak {}", lo_bin, start_peak);
  let cfd_time : f32;
  if lo_bin < nanoseconds.len() -1 {
    cfd_time = interpolate_time(voltages, nanoseconds, tmp_thresh, lo_bin, 1)?;  
  } else {
    cfd_time = nanoseconds[nanoseconds.len() - 1];
  } 
  Ok(cfd_time)
}

/// Find peaks in a given time window (in ns) by 
/// comparing the waveform voltages with the 
/// given threshold. 
///
/// #Arguments:
/// * start_time     : begin to look for peaks after 
///                    this (local) waveform time 
/// * window_size    : (in ns)
/// * min_peak_width : minimum number of consequtive bins
///                    which have to be over threshold
///                    so that it is considered a peak
/// * threshold      : peaks are found when voltages go
///                    over threshold for at leas
///                    min_peak_width bins
/// * max_peaks      : stop algorithm after max_peaks are
///                    found, the rest will be ignored
/// #Returns:
/// 
/// Vec<(peak_begin_bin, peak_end_bin)>
///
pub fn find_peaks(voltages       : &Vec<f32>,
                  nanoseconds    : &Vec<f32>,
                  start_time     : f32,
                  window_size    : f32,
                  min_peak_width : usize,
                  threshold      : f32,
                  max_peaks      : usize)
-> Result<Vec<(usize,usize)>, WaveformError> {
  let mut peaks      = Vec::<(usize,usize)>::new();
  let mut start_bin  = time2bin(nanoseconds, start_time)?;
  if start_bin <= 10 {
    debug!("We deliberatly do not search for peaks within the first 10 bins! Correcting..");
    start_bin = 10;
  }
  let window_bin = time2bin(nanoseconds, start_time + window_size)? - start_bin;
  if start_bin + window_bin > voltages.len () {
    return Err(WaveformError::OutOfRangeUpperBound);
  }

  let mut pos = 0usize;
  // find the first bin when voltage
  // goes over threshold
  for k in start_bin..start_bin + window_bin {
    if voltages[k] >= threshold {
      pos = k;
      break;
    }
  }
  if pos == 0 && start_bin == 0 && voltages[pos] < threshold {
    // waveform did not cross threshold
    return Err(WaveformError::DidNotCrossThreshold)
  }
  // actual peak finding
  let mut nbins_peak   = 0usize;
  let mut begin_peak   = pos;
  let mut end_peak  : usize;
  if (pos + window_bin) > voltages.len() {
    return Err(WaveformError::OutOfRangeUpperBound);
  }
  for k in pos..(pos + window_bin) {
    if voltages[k] >= threshold {
      nbins_peak += 1;
      let mut slope = 0i16; // slope can be positive (1)
                            // or negative (-1)
                            // as soon as the slope turns, 
                            // we declare the peak over, 
                            // if it is still positive, we
                            // continue to count the bins
      if nbins_peak == min_peak_width {
        // in this case, we don't care about the slope
        begin_peak  = k - min_peak_width -1;
      } else if nbins_peak > min_peak_width {
        for j in 0..min_peak_width {
          if voltages[k -j] > voltages[k-j-1] {
            slope = 1; // still ascending
          }
        }
        if slope == 1 {
          // we consider this the same peak
          continue;
        } 
        if slope == 0 {
          // each bump counts as separate peak
          end_peak = k;
          nbins_peak = 0; // peak is done
          peaks.push((begin_peak, end_peak));
          if peaks.len() == max_peaks {
            break;
          }
        }
      } // if nbins_peak < min_peak_width, we just 
        // continue going to check if it is still 
        // over threshold
    } else {
      if nbins_peak > min_peak_width {
        end_peak = k;
        peaks.push((begin_peak, end_peak));
        if peaks.len() == max_peaks {
          break;
        }
      }
      nbins_peak = 0;
    }
  }
  // FIXME - remove invalid peaks
  let len_pks_dirty = peaks.len();
  peaks.retain(|&x| {(x.0 < NWORDS - 1) & (x.1 <= NWORDS - 1)});
  let len_pks_clean = peaks.len();
  if len_pks_clean != len_pks_dirty {
    debug!("We removed {} pks because they had values outside of 0-{}!", len_pks_dirty - len_pks_clean, NWORDS);
  }
  Ok(peaks)
}


#[cfg(feature = "advanced-algorithms")]
fn find_sequence_ranges(vec: Vec<usize>) -> Vec<(usize, usize)> {
  let mut ranges = Vec::new();
  let mut start = vec[0];
  let mut end   = vec[0];

  for &value in vec.iter().skip(1) {
    if value == end + 1 {
      // Extend the current sequence
      end = value;
    } else {
      // End of current sequence, start of a new one
      ranges.push((start, end));
      start = value;
      end = value;
    }
  }

  // Add the last sequence
  ranges.push((start, end));
  ranges
}

#[cfg(feature = "advanced-algorithms")]
/// Z-scores peak finding algorithm
///
/// Brakel, J.P.G. van (2014).
/// "Robust peak detection algorithm using z-scores". 
/// Stack Overflow.
/// Available at: <https://stackoverflow.com/questions/22583391/peak-signal-detection-in-realtime-timeseries-data/i22640362#22640362> (version: 2020-11-08).
///
/// Robust peak detection algorithm (using z-scores)
///
/// [..] algorithm that works very well for these types of datasets.
/// It is based on the principle of dispersion:
/// if a new datapoint is a given x number of standard deviations away
/// from a moving mean, the algorithm gives a signal.
/// The algorithm is very robust because it constructs a separate moving mean
/// and deviation, such that previous signals do not corrupt
/// the signalling threshold for future signals.
/// The sensitivity of the algorithm is therefore robust to previous signals.
///
/// # Arguments:
///
/// * nanoseconds   : calibrated waveform times
/// * voltages      : calibrated waveform voltages
/// * start_time    : restrict the algorithm on a 
///                   certain time window, start 
///                   at start_time
/// * window_size   : in ns
/// * lag           : The lag of the moving window that calculates the mean
///                   and standard deviation of historical data.
///                   A longer window takes more historical data in account.
///                   A shorter window is more adaptive,
///                   such that the algorithm will adapt to new information
///                   more quickly.
///                   For example, a lag of 5 will use the last 5 observations
///                   to smooth the data.
/// * threshold     : The "z-score" at which the algorithm signals.
///                   Simply put, if the distance between a new datapoint
///                   and the moving mean is larger than the threshold
///                   multiplied with the moving standard deviation of the data,
///                   the algorithm provides a signal.
///                   For example, a threshold of 3.5 will signal if a datapoint
///                   is 3.5 standard deviations away from the moving mean. 
/// * influence     : The influence (between 0 and 1) of new signals on
///                   the calculation of the moving mean and moving standard deviation.
///                   For example, an influence parameter of 0.5 gives new signals
///                   half of the influence that normal datapoints have.
///                   Likewise, an influence of 0 ignores signals completely
///                   for recalculating the new threshold.
///                   An influence of 0 is therefore the most robust option 
///                   (but assumes stationarity);
///                   putting the influence option at 1 is least robust.
///                   For non-stationary data, the influence option should
///                   therefore be put between 0 and 1.
pub fn find_peaks_zscore(nanoseconds    : &Vec<f32>,
                         voltages       : &Vec<f32>,
                         start_time     : f32,
                         window_size    : f32,
                         lag            : usize,
                         threshold      : f64,
                         influence      : f64)
-> Result<Vec<(usize,usize)>, WaveformError> {
  let mut peaks = Vec::<(usize, usize)>::new();
  let start_bin = time2bin(nanoseconds, start_time)?;
  let end_bin   = time2bin(nanoseconds, start_time + window_size)?;
  let mut ranged_voltage = Vec::<f32>::with_capacity(end_bin - start_bin);
  ranged_voltage.extend_from_slice(&voltages[start_bin..=end_bin]);
  //30, 5.0, 0.0

  let output: Vec<_> = voltages
            .into_iter()
            .enumerate()
            .peaks(PeaksDetector::new(lag, threshold, influence), |e| *e.1 as f64)
            .map(|((i, _), p)| (i, p))
            .collect();
  // we ignore low peaks
  if output.len() == 0 {
    return Ok(peaks);
  }
  let mut peak_high = Vec::<usize>::new();
  for k in output.iter() {
    if matches!(k.1, Peak::High) {
      peak_high.push(k.0);
    }
  }
  if peaks.len() > 0 {
    peaks = find_sequence_ranges(peak_high); 
  }
  Ok(peaks)
}

//---------------------------------------------------

/// Sine fit without using external libraries
pub fn fit_sine_simple<T>(volts: &[T], times: &[T]) -> (f32, f32, f32) 
  where T: Float + NumAssign + NumAssignOps + NumOps + Copy + NumCast + FloatConst {
  let start_bin = 20;
  let size_bin = 900;
  let mut data_size = T::zero();

  let mut xi_yi   = T::zero();
  let mut xi_zi   = T::zero();
  let mut yi_zi   = T::zero();
  let mut xi_xi   = T::zero();
  let mut yi_yi   = T::zero();
  let mut xi_sum  = T::zero();
  let mut yi_sum  = T::zero();
  let mut zi_sum  = T::zero();

  let c1 = T::from(2).unwrap();
  let c2 = T::from(0.02f32).unwrap();
  for i in start_bin..(start_bin + size_bin) {
      let xi = (c1 * T::PI() * c2 * times[i]).cos();
      let yi = (c1 * T::PI() * c2 * times[i]).sin();
      let zi = volts[i];

      xi_yi += xi * yi;
      xi_zi += xi * zi;
      yi_zi += yi * zi;
      xi_xi += xi * xi;
      yi_yi += yi * yi;
      xi_sum += xi;
      yi_sum += yi;
      zi_sum += zi;

      data_size += T::one();
  }

  let mut a_matrix = [[T::zero(); 3]; 3];
  a_matrix[0][0] = xi_xi;
  a_matrix[0][1] = xi_yi;
  a_matrix[0][2] = xi_sum;
  a_matrix[1][0] = xi_yi;
  a_matrix[1][1] = yi_yi;
  a_matrix[1][2] = yi_sum;
  a_matrix[2][0] = xi_sum;
  a_matrix[2][1] = yi_sum;
  a_matrix[2][2] = data_size;

  let determinant = a_matrix[0][0] * a_matrix[1][1] * a_matrix[2][2]
      + a_matrix[0][1] * a_matrix[1][2] * a_matrix[2][0]
      + a_matrix[0][2] * a_matrix[1][0] * a_matrix[2][1]
      - a_matrix[0][0] * a_matrix[1][2] * a_matrix[2][1]
      - a_matrix[0][1] * a_matrix[1][0] * a_matrix[2][2]
      - a_matrix[0][2] * a_matrix[1][1] * a_matrix[2][0];

  let inverse_factor = T::one() / determinant;

  let mut cofactor_matrix = [[T::zero(); 3]; 3];
  cofactor_matrix[0][0] = a_matrix[1][1] * a_matrix[2][2] - a_matrix[2][1] * a_matrix[1][2];
  cofactor_matrix[0][1] = (a_matrix[1][0] * a_matrix[2][2] - a_matrix[2][0] * a_matrix[1][2]) * -T::one();
  cofactor_matrix[0][2] = a_matrix[1][0] * a_matrix[2][1] - a_matrix[2][0] * a_matrix[1][1];
  cofactor_matrix[1][0] = (a_matrix[0][1] * a_matrix[2][2] - a_matrix[2][1] * a_matrix[0][2]) * -T::one();
  cofactor_matrix[1][1] = a_matrix[0][0] * a_matrix[2][2] - a_matrix[2][0] * a_matrix[0][2];
  cofactor_matrix[1][2] = (a_matrix[0][0] * a_matrix[2][1] - a_matrix[2][0] * a_matrix[0][1]) * -T::one();
  cofactor_matrix[2][0] = a_matrix[0][1] * a_matrix[1][2] - a_matrix[1][1] * a_matrix[0][2];
  cofactor_matrix[2][1] = (a_matrix[0][0] * a_matrix[1][2] - a_matrix[1][0] * a_matrix[0][2]) * -T::one();
  cofactor_matrix[2][2] = a_matrix[0][0] * a_matrix[1][1] - a_matrix[1][0] * a_matrix[0][1];

  let mut inverse_matrix = [[T::zero(); 3]; 3];
  for i in 0..3 {
      for j in 0..3 {
          inverse_matrix[i][j] = cofactor_matrix[j][i] * inverse_factor;
      }
  }

  let p = [xi_zi, yi_zi, zi_sum];
  let a = inverse_matrix[0][0] * p[0] + inverse_matrix[1][0] * p[1] + inverse_matrix[2][0] * p[2];
  let b = inverse_matrix[0][1] * p[0] + inverse_matrix[1][1] * p[1] + inverse_matrix[2][1] * p[2];

  let phi    = <f32 as NumCast>::from(a.atan2(b)).unwrap();
  let amp    = <f32 as NumCast>::from((a*a + b*b).sqrt()).unwrap();
  let freq   = 0.02 as f32;

  (amp, freq, phi)
}

#[cfg(feature="pybindings")]
#[pyfunction]
#[pyo3(name="fit_sine_simple")]
pub fn fit_sine_simple_py<'_py>(xs    : Bound<'_py,PyArray1<f32>>, ys: Bound<'_py, PyArray1<f32>>)  -> (f32,f32,f32) {
  unsafe {
    fit_sine_simple::<f32>(ys.as_slice().unwrap(), xs.as_slice().unwrap())
  }
}

