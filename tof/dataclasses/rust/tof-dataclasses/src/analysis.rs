//! Implementations of analysis engine
//! This is based on the original code 
//! by J.Zweerink
//!
  
use crate::errors::WaveformError;


#[cfg(feature="advanced-algorithms")]
extern crate smoothed_z_score;
#[cfg(feature="advanced-algorithms")]
use smoothed_z_score::{Peak, PeaksDetector, PeaksFilter};

// Return the bin with the maximum DC value
//
pub fn get_max_bin(voltages    : &Vec<f32>,
                   lower_bound : usize,
                   window      : usize) -> Result<usize, WaveformError> {
  if lower_bound + window > voltages.len() {
    return Err(WaveformError::OutOfRangeUpperBound);
  }
  let mut maxval = voltages[lower_bound];
  let mut maxbin = lower_bound;
  for n in lower_bound..lower_bound + window {
    if voltages[n] > maxval {
      maxval  = voltages[n];
      maxbin  = n;
    }
  } // end for
  trace!("Got maxbin {} with a value of {}", maxbin, maxval);
  Ok(maxbin)
} // end fn

///
///
///
///
pub fn interpolate_time (voltages      : &Vec<f32>,
                         nanoseconds   : &Vec<f32>, 
                         mut threshold : f32,
                         mut idx       : usize,
                         size          : usize) -> Result<f32, WaveformError> {
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
                 lower_bound  : f32,
                 size         : f32,
                 impedance    : f32) ->Result<f32, WaveformError>  {
  if lower_bound < 0.0 { 
    return Err(WaveformError::NegativeLowerBound);
  }
  let lo_bin          = time2bin(nanoseconds,lower_bound)?;
  let mut size_bin    = time2bin(nanoseconds,lower_bound + size)?;
  size_bin = size_bin - lo_bin;
  if lo_bin + size_bin > voltages.len() {
    warn!("Limiting integration range to waveform size!");
    size_bin = voltages.len() - lo_bin;
  }
  let mut sum = 0f32;
  let upper_bin = lo_bin + size_bin;
  for n in lo_bin..upper_bin {
    sum += voltages[n] * (nanoseconds[n] - nanoseconds[n-1]) ;
  }
  sum /= impedance;
  Ok(sum)
}

// Given a time in ns, find the bin most closely corresponding to that time
pub fn time2bin(nanoseconds : &Vec<f32>,
                t_ns        : f32) -> Result<usize, WaveformError> {
  for n in 0..nanoseconds.len() {
    if nanoseconds[n] > t_ns {
      return Ok(n-1);
    }
  }
  error!("Did not find a bin corresponding to the given time {}!", t_ns);
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

  let idx = get_max_bin(voltages,start_peak,end_peak-start_peak)?;
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
  if idx >= start_peak {
    error!("The index is smaller than the beginning of the peak!");
    return Err(WaveformError::OutOfRangeLowerBound);
  }
  if start_peak >= 10 {
    while n > start_peak - 10 {
    //for n in (idx..start_peak - 10).rev() {
      if f32::abs(voltages[n]) < tmp_thresh {
        lo_bin = n;
        break;
      }
      n -= 1;
    }  
  } else {
    error!("We require that the peak is at least 10 bins away from the start!");
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
/// Minimum peak width is currently hardcoded to 
/// be 3 bins in time.
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
  let mut peaks = Vec::<(usize,usize)>::new();

  let start_bin  = time2bin(nanoseconds, start_time)?;
  let window_bin = time2bin(nanoseconds, start_time + window_size)? - start_bin;

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
  Ok(peaks)
}
// 




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
/// Available at: https://stackoverflow.com/questions/
/// 22583391/peak-signal-detection-in-realtime-timeseries-data/
/// 22640362#22640362 (version: 2020-11-08).
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
  peaks = find_sequence_ranges(peak_high); 
  Ok(peaks)
}

