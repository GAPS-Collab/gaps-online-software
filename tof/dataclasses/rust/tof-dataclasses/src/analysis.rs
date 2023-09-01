//! Implementations of analysis engine
//! This is based on the original code 
//! by J.Zweerink
//!


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
