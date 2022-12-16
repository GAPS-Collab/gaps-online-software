/*****************************************/

use crate::constants::{NWORDS,
                       MAX_NUM_PEAKS,
                       WF_VOLTAGE_THRESHOLD};

use crate::readoutboard_blob::BlobData;

#[cfg(feature = "diagnostics")]
#[cfg(feature = "blosc")]
use hdf5::filters::blosc_set_nthreads;

#[cfg(feature = "diagnostics")]
use hdf5;


/*****************************************/

#[derive(Debug)]
pub enum WaveformError {
    TimeIndexOutOfBounds,
    TimesTooSmall,
    NegativeLowerBound
}

/*****************************************/

/// Hold calibrated (voltage) and timing
/// data for a single waveform.
///
//pub struct CalibratedWaveform<'a> {
//  wave  : &'a[f64;NWORDS],
//  times : &'a[f64;NWORDS],
#[derive(Clone, PartialEq, Debug)] // register with HDF5
#[cfg_attr(feature = "diagnostics", derive(hdf5::H5Type))]
#[cfg_attr(feature = "diagnostics", repr(C))]
pub struct CalibratedWaveform {
  pub event_ctr : u32,
  pub channel   : usize,
  wave  : [f64;NWORDS],
  times : [f64;NWORDS],
  /// peak properties
  /// bin positions
  peaks   : [usize;MAX_NUM_PEAKS],
  tdcs    : [f64;MAX_NUM_PEAKS],
  charge  : [f64;MAX_NUM_PEAKS],
  width   : [f64;MAX_NUM_PEAKS], 
  height  : [f64;MAX_NUM_PEAKS],    
  num_peaks  : usize,
  stop_cell  : u16,
  begin_peak : [usize;MAX_NUM_PEAKS],
  end_peak   : [usize;MAX_NUM_PEAKS],
  spikes     : [usize;MAX_NUM_PEAKS],

  // these values are for baseline 
  // subtraction, cfd calculation etc.
  threshold      : f64,
  cfds_fraction  : f64,
  ped_begin_bin  : usize,
  ped_bin_range  : usize,    
  pedestal       : f64,
  pedestal_sigma : f64

}


// FIXME - I think instead of borrowing it here with a livetime, I'd rather have
// it moved. 
//impl CalibratedWaveform<'_> {
impl CalibratedWaveform {

  //pub fn new<'a>(wave: &'a[f64;NWORDS], times: &'a[f64;NWORDS]) -> CalibratedWaveform<'a> {
  //pub fn new(times : [f64;NWORDS], wave : [f64;NWORDS]) ->CalibratedWaveform {
  pub fn new(blob_data : &BlobData, channel : usize) ->CalibratedWaveform {
    CalibratedWaveform { event_ctr      : blob_data.event_ctr,
                         channel        : channel,
                         wave           : blob_data.voltages[channel],
                         times          : blob_data.nanoseconds[channel],
                         peaks          : blob_data.peaks[channel],
                         tdcs           : blob_data.tdcs[channel],
                         charge         : blob_data.charge[channel],
                         width          : blob_data.width[channel],
                         height         : blob_data.height[channel],
                         num_peaks      : blob_data.num_peaks[channel],
                         stop_cell      : blob_data.stop_cell,
                         begin_peak     : blob_data.begin_peak[channel],
                         end_peak       : blob_data.end_peak[channel],
                         spikes         : blob_data.spikes[channel],
                         threshold      : blob_data.threshold[channel],
                         cfds_fraction  : blob_data.cfds_fraction[channel],
                         ped_begin_bin  : blob_data.ped_begin_bin[channel],
                         ped_bin_range  : blob_data.ped_bin_range[channel],
                         pedestal       : blob_data.pedestal[channel],
                         pedestal_sigma : blob_data.pedestal_sigma[channel]
    }
  }

  pub fn print(&self) {
    println!("<=== Calibrated waveform with {} entries ===>", NWORDS);
    println!(" .. wave:");
    for n in 0..5 {
      print!("{},", self.wave[n]);  
    }
    println!(" .. times:");
    for n in 0..5 {
      print!("{},", self.times[n]);  
    }
    println!(" .. tdcs:");
    for n in 0..5 {
      print!("{},", self.tdcs[n]);  
    }
    println!("*************************");
  }

  pub fn set_threshold(&mut self, thr : f64) {
      self.threshold = thr;
  }

  pub fn set_cfds_fraction(&mut self, fraction : f64) {
      self.cfds_fraction = fraction;
  }
  
  pub fn set_ped_begin(&mut self, time : f64) {
      match self.time_2_bin(time) {
          Err(err) => println!("Can not find bin for time {}, err FIXME", time),
          Ok(begin) => {self.ped_begin_bin = begin;}
      }
  }

  pub fn set_ped_range(&mut self, range : f64) {
    // This is a little convoluted, but we must convert the range (in
    // ns) into bins
    match self.time_2_bin(self.times[self.ped_begin_bin] + range) {
        Err(err)      => println!("Can not set pedestal range for range {}", range),
        Ok(bin_range) => {self.ped_bin_range = bin_range;}
    }
  }

  pub fn subtract_pedestal(&mut self) {
    for n in 0..NWORDS {
      self.wave[n] -= self.pedestal;
    }
  }


  pub fn calc_ped_range(&mut self) {
    let mut sum  = 0f64;
    let mut sum2 = 0f64;

    for n in self.ped_begin_bin..self.ped_begin_bin + self.ped_bin_range {
      if f64::abs(self.wave[n]) < 10.0 {
        sum  += self.wave[n];
        sum2 += self.wave[n]*self.wave[n];
      }
    }
    let average = sum/(self.ped_bin_range as f64);
    self.pedestal = average;
    self.pedestal_sigma = f64::sqrt(sum2/(self.ped_bin_range as f64 - (average*average)))

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


  // 
  // Return the bin with the maximum DC value
  //
  fn get_max_bin(&self, lower_bound : usize, upper_bound : usize ) -> usize {
    let rel_upper_bound = NWORDS - upper_bound;
    //println!("{} {} {}", lower_bound, upper_bound, rel_upper_bound);
    assert!((rel_upper_bound - lower_bound) <= NWORDS);
    let mut maxval = self.wave[lower_bound];
    let mut maxbin = lower_bound;
    for n in lower_bound+1..lower_bound+rel_upper_bound {
      // I think the - sign is bc of pmt waveforms...
      //if maxval < self.wave[n] {
      if maxval > self.wave[n] {
        maxval = self.wave[n];
        maxbin = n;
      }
    } // end for
    return maxbin;
  } // end fn

  pub fn find_cfd_simple(&self, peak_num : usize) -> f64 {
    if peak_num > self.num_peaks {return self.wave[NWORDS];}
    // FIXME
    let mut idx = self.get_max_bin(self.begin_peak[peak_num],
                                   self.end_peak[peak_num]-self.begin_peak[peak_num]);

    if idx < 1 {idx = 1;}
    let mut sum : f64 = 0.0;
    for n in idx-1..idx+1 {sum += self.wave[n];}
    let cfds_frac  : f64 = 0.2;
    let tmp_thresh : f64 = f64::abs(cfds_frac * (sum / 3.0));

    // Now scan through the waveform around the peak to find the bin
    // crossing the calculated threshold. Bin idx is the peak so it is
    // definitely above threshold. So let's walk backwards through the
    // trace until we find a bin value less than the threshold.
    let mut lo_bin : usize = NWORDS;
    let mut n = idx;
    if self.begin_peak[peak_num] >= 10 {
      while n > self.begin_peak[peak_num] - 10 {
      //for n in (idx..self.begin_peak[peak_num] - 10).rev() {
        if f64::abs(self.wave[n]) < tmp_thresh {
          lo_bin = n;
          break;
        }
        n -= 1;
      }  
    }

    let mut cfd_time : f64 = 0.0;
    if lo_bin < NWORDS {
      cfd_time = self.find_interpolated_time(tmp_thresh, lo_bin, 1);  
    }
    else {cfd_time = self.wave[NWORDS - 1];} 
    return cfd_time;
  }

  pub fn find_interpolated_time (&self,
                                 //adc       : [f64;NWORDS],
                                //times     : [f64;NWORDS], 
                                mut threshold : f64,
                                mut idx       : usize,
                                size          : usize ) -> f64 
  {
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
      return self.wave[idx] + (threshold-lval)/(hval-lval) * (self.wave[idx+1]-self.wave[idx]);
      //float time = WaveTime[idx] +  
      //  (thresh-lval)/(hval-lval) * (WaveTime[idx+1]-WaveTime[idx]) ;
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
    while (self.wave[pos] < threshold) && (pos < NWORDS) {
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


} // end impl



/*
///
/// Waveform type which owns the data. This is 
/// solely used for diagnostics (slow) and can
/// be written to an hdf file for later
/// analysis
///
#[cfg(feature = "diagnostics")]
#[derive(hdf5::H5Type, Clone, PartialEq, Debug)] // register with HDF5
#[repr(C)]
pub struct CalibratedWaveformForDiagnostics{
  pub wave  :   [f64;NWORDS],
  pub times :   [f64;NWORDS],
  /// peak properties
  /// bin positions
  pub peaks   : [usize;MAX_NUM_PEAKS],
  pub tdcs    : [f64;MAX_NUM_PEAKS],
  pub charge  : [f64;MAX_NUM_PEAKS],
  pub width   : [f64;MAX_NUM_PEAKS], 
  pub height  : [f64;MAX_NUM_PEAKS],    
  pub num_peaks  : usize,
  pub stop_cell  : u16,
  pub begin_peak : [usize;MAX_NUM_PEAKS],
  pub end_peak   : [usize;MAX_NUM_PEAKS],
  pub spikes     : [usize;MAX_NUM_PEAKS],
}

#[cfg(feature = "diagnostics")]
impl CalibratedWaveformForDiagnostics {

  pub fn new(wf : &CalibratedWaveform) -> CalibratedWaveformForDiagnostics {
    CalibratedWaveformForDiagnostics {
      //wave       : *wf.wave, //[0.0;NWORDS],
      //times      : *wf.times,
      wave       : wf.wave, //[0.0;NWORDS],
      times      : wf.times,
      peaks      : wf.peaks,
      tdcs       : wf.tdcs,
      charge     : wf.charge,
      width      : wf.width, 
      height     : wf.height,    
      num_peaks  : wf.num_peaks,
      stop_cell  : wf.stop_cell,
      begin_peak : wf.begin_peak,
      end_peak   : wf.end_peak,
      spikes     : wf.spikes
    }      
  }
  
  pub fn print(&self) {
    println!("<=== Diagnositcs waveform with {} entries ===>", NWORDS);
    println!(" .. wave: [");
    for n in 0..5 {
      print!("{},", self.wave[n]);  
    }
    println!("..]");
    println!(" .. times:");
    for n in 0..5 {
      print!("{},", self.times[n]);  
    }
    println!("..]");
    println!(" .. tdcs:");
    for n in 0..5 {
      print!("{},", self.tdcs[n]);  
    }
    println!("..]");
    println!("*************************");
  }

}

*/
