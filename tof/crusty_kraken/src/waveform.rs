/*****************************************/

use crate::constants::{NWORDS,
                       MAX_NUM_PEAKS,
                       WF_VOLTAGE_THRESHOLD};

/*****************************************/

#[derive(Debug)]
pub enum WaveformError {
    TimeIndexOutOfBounds,
    TimesTooSmall,
}

/*****************************************/

/// Hold calibrated (voltage) and timing
/// data for a single waveform.
///
pub struct CalibratedWaveform<'a> {
  wave  : &'a[f64;NWORDS],
  times : &'a[f64;NWORDS],
  /// peak properties
  /// bin positions
  peaks   : [usize;MAX_NUM_PEAKS],
  tdcs    : [f64;MAX_NUM_PEAKS],
  charge  : [f64;MAX_NUM_PEAKS],
  width   : [f64;MAX_NUM_PEAKS], 
  height  : [f64;MAX_NUM_PEAKS],    
  num_peaks : usize,
  begin_peak : [usize;MAX_NUM_PEAKS],
  end_peak   : [usize;MAX_NUM_PEAKS],
  spikes     : [usize;MAX_NUM_PEAKS],
}

impl CalibratedWaveform<'_> {

  pub fn new<'a>(wave: &'a[f64;NWORDS], times: &'a[f64;NWORDS]) -> CalibratedWaveform<'a> {
    CalibratedWaveform { wave    : wave,
                         times   : times,
                         peaks   : [0;  MAX_NUM_PEAKS],
                         tdcs    : [0.0;MAX_NUM_PEAKS],
                         charge  : [0.0;MAX_NUM_PEAKS],
                         width   : [0.0;MAX_NUM_PEAKS],
                         height  : [0.0;MAX_NUM_PEAKS],
                         num_peaks : 0,
                         begin_peak : [0;MAX_NUM_PEAKS],
                         end_peak   : [0;MAX_NUM_PEAKS],
                         spikes      : [0;MAX_NUM_PEAKS]
    }
  }

  fn time_2_bin(&self, t_ns : f64) -> Result<usize, WaveformError> {
    // Given a time in ns, find the bin most closely corresponding to that time
    for n in 0..NWORDS {
        if self.times[n] > t_ns {
            return Ok(n-1);
        }
    }
    println!("Did not find a bin corresponding to the given time {}", t_ns);
    return Err(WaveformError::TimesTooSmall);
  }

  fn get_max_bin(&self, lower_bound : usize, upper_bound : usize ) -> usize {
    let rel_upper_bound = NWORDS - upper_bound;
    assert!((rel_upper_bound - lower_bound) < NWORDS);
    let mut maxval = self.wave[lower_bound];
    let mut maxbin = lower_bound;
    for n in lower_bound+1..lower_bound+rel_upper_bound {
      if maxval < self.wave[n] {
        maxval = self.wave[n];
        maxbin = n;
      }
    } // end for
    return maxbin;
  } // end fn

  pub fn find_cfd_simple(&self, peak_num : usize) -> f64 {
    if peak_num > self.num_peaks {return self.wave[NWORDS];}
    let idx = self.get_max_bin(self.begin_peak[peak_num],
                               self.end_peak[peak_num]-self.begin_peak[peak_num]);

    let mut sum : f64 = 0.0;
    for n in idx-1..idx+1 {sum += self.wave[n];}
    let cfds_frac  : f64 = 0.2;
    let tmp_thresh : f64 = f64::abs(cfds_frac * (sum / 3.0));

    // Now scan through the waveform around the peak to find the bin
    // crossing the calculated threshold. Bin idx is the peak so it is
    // definitely above threshold. So let's walk backwards through the
    // trace until we find a bin value less than the threshold.
    let mut lo_bin : usize = NWORDS;
    for n in (idx..self.begin_peak[peak_num] - 10).rev() {
      if f64::abs(self.wave[n]) < tmp_thresh {
        lo_bin = n;
        break;
      }
    }

    let mut cfd_time : f64 = 0.0;
    if lo_bin < NWORDS {
      cfd_time = self.find_interpolated_time(tmp_thresh, lo_bin, 1);  
    }
    else {cfd_time = self.wave[NWORDS];} 
    return cfd_time;
  }

  pub fn find_interpolated_time (&self,
                                 //adc       : [f64;NWORDS],
                                //times     : [f64;NWORDS], 
                                mut threshold : f64,
                                mut idx       : usize,
                                size          : usize ) -> f64 
  {
    let mut time :f64;
    threshold = threshold.abs();
    let mut lval  = (self.wave[idx]).abs();
    let mut hval : f64 = 0.0; 
    if size == 1 {
      hval = (self.wave[idx+1]).abs();
    } else {
    for n in idx+1..idx+size {
      hval = self.wave[n].abs();
      if (hval>=threshold) && (threshold<=lval) { // Threshold crossing?
        idx = n-1; // Reset idx to point before crossing
        break;
        }
      lval = hval;
      }
    }
    if ( lval > threshold) && (size != 1) {
      return self.times[idx];
    } else if lval == hval {
      return self.times[idx];
    } else {
      time = self.wave[idx] + (threshold-lval)/(hval-lval) * (self.wave[idx+1]-self.wave[idx]);
      //float time = WaveTime[idx] +  
      //  (thresh-lval)/(hval-lval) * (WaveTime[idx+1]-WaveTime[idx]) ;
      return time;
      }
  }



  fn find_peaks(&mut self,
                start_time  : f64,
                window_size : f64,
                threshold   : f64) {
    // FIXME - replace unwrap calls
    let start_bin  = self.time_2_bin(start_time).unwrap();
    let window_bin = self.time_2_bin(start_time + window_size).unwrap() - start_bin;

    // minimum number of bins a peak must have
    // over threshold so that we consider it 
    // a peak
    let min_peak_width       = 3usize; 
    let mut pos              = 0usize;
    let mut peak_bins        = 0usize;
    let mut n_peaks_detected = 0usize;
    let mut peak_ctr         = 0usize;
    while ((self.wave[pos] < threshold) && (pos < NWORDS)) {
      pos += 1;
    }
    for n in pos..start_bin + window_bin {
      if self.wave[n] > threshold {
        peak_bins += 1;
        if peak_bins == min_peak_width {
          // we have a new peak
          if n_peaks_detected == MAX_NUM_PEAKS {
            println!("Max number of peaks reached in this waveform");
            break;
          }
          self.begin_peak[peak_ctr] = n - (min_peak_width - 1); 
          self.spikes    [peak_ctr] = 0;
          self.end_peak  [peak_ctr] = 0;
          peak_ctr += 1;
        } else if peak_bins > min_peak_width {
          for k in 0..3 {
            if self.wave[n-k] > self.wave[n-(k+1)]
              {continue;}
          }
          if self.end_peak[peak_ctr-1] == 0 {
            self.end_peak[peak_ctr-1] = n; // Set last bin included in peak
          }
        } else {
          peak_bins = 0;
        }
      }
    }

    
    //for pos in start_bin..NWORDS {
    //  if (self.wave > threshold) {
    //  }
        
    //}

    //((self.wave[pos] < WF_VOLTAGE_THRESHOLD) && (pos < wf_size))
    self.num_peaks = peak_ctr;
    self.begin_peak[peak_ctr] = NWORDS; // Need this to measure last peak correctly
    //peaks_found = 1;
  }


} // end imple



