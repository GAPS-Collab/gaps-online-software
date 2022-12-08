/***********************************
 * Readoutboard data, calibration and 
 * waveform analysis.
 *
 * Basically a translation of the 
 * tof library written by 
 * J. Zweerink
 *
 *
 ***********************************/

use crate::constants::{NWORDS, NCHN, MAX_NUM_PEAKS};
use crate::waveform::WaveformError;

use crate::calibrations::Calibrations;

pub fn get_constant_blobeventsize() -> usize {
  let size = 36 + (NCHN*2) + (NCHN*NWORDS*2) + (NCHN*4) + 8;
  return size;
}

// for diagnostics, we use hdf5 files
#[cfg(feature = "diagnostics")]
#[cfg(feature = "blosc")]
use hdf5::filters::blosc_set_nthreads;

#[cfg(feature = "diagnostics")]
use hdf5;

/***********************************/

#[derive(Debug, Clone, Copy, PartialEq)]
#[cfg_attr(feature = "diagnostics", derive(hdf5::H5Type))]
#[cfg_attr(feature = "diagnostics", repr(C))] 
pub struct BlobData
{
  pub head            : u16, // Head of event marker
  pub status          : u16,
  pub len             : u16,
  pub roi             : u16,
  pub dna             : u64, 
  pub fw_hash         : u16,
  pub id              : u16,   
  pub ch_mask         : u16,
  pub event_ctr       : u32,
  pub dtap0           : u16,
  pub dtap1           : u16,
  pub timestamp       : u64,
  pub ch_head         : [u16; NCHN],
  pub ch_adc          : [[i16; NWORDS];NCHN], 
  pub ch_trail        : [u32; NCHN],
  pub stop_cell       : u16,
  pub crc32           : u32,
  pub tail            : u16, // End of event marker

  // these are NOT in the official blob format
  // these will NOT be able to be deserialized from
  // a standard readoutboard blob file
  pub voltages           : [[f64;NWORDS];NCHN],
  pub nanoseconds        : [[f64;NWORDS];NCHN],
  
  // these values are for baseline 
  // subtraction, cfd calculation etc.
  pub threshold      : [f64;NCHN],
  pub cfds_fraction  : [f64;NCHN],
  pub ped_begin_bin  : [usize;NCHN],
  pub ped_bin_range  : [usize;NCHN],    
  pub pedestal       : [f64;NCHN],
  pub pedestal_sigma : [f64;NCHN],

  // fields used for internal calculations
  pub peaks      : [[usize;MAX_NUM_PEAKS];NCHN],
  pub tdcs       : [[f64;MAX_NUM_PEAKS];NCHN],
  pub charge     : [[f64;MAX_NUM_PEAKS];NCHN],
  pub width      : [[f64;MAX_NUM_PEAKS];NCHN], 
  pub height     : [[f64;MAX_NUM_PEAKS];NCHN],    
  pub num_peaks  : [usize;NCHN],
  //pub stop_cell  : [u16;NCHN],
  pub begin_peak : [[usize;MAX_NUM_PEAKS];NCHN],
  pub end_peak   : [[usize;MAX_NUM_PEAKS];NCHN],
  pub spikes     : [[usize;MAX_NUM_PEAKS];NCHN],
} 

/***********************************/

impl BlobData {

  ///
  ///FIXME - more correctly, this should be 
  ///"deserialize from readoutboard_blob

  pub fn from_bytestream(&mut self, bytestream : &Vec<u8>, start_pos : usize ) -> usize {
    let mut pos = start_pos;
    let mut raw_bytes_2  = [bytestream[pos],bytestream[pos + 1]];
    pos   += 2;
    self.head    = u16::from_le_bytes(raw_bytes_2);
    
    raw_bytes_2  = [bytestream[pos],bytestream[pos + 1]];
    pos   += 2;
    self.status  = u16::from_le_bytes(raw_bytes_2); 
    
    raw_bytes_2  = [bytestream[pos],bytestream[pos + 1]];
    pos   += 2;
    self.len     = u16::from_le_bytes(raw_bytes_2); 
    
    raw_bytes_2  = [bytestream[pos],bytestream[pos + 1]];
    pos   += 2;
    self.roi     = u16::from_le_bytes(raw_bytes_2); 

    let mut raw_bytes_8  = [bytestream[pos + 1],
                            bytestream[pos + 0],
                            bytestream[pos + 3],
                            bytestream[pos + 2],
                            bytestream[pos + 5],
                            bytestream[pos + 4],
                            bytestream[pos + 7],
                            bytestream[pos + 6]];
    pos   += 8;
    self.dna     = u64::from_be_bytes(raw_bytes_8);

    raw_bytes_2  = [bytestream[pos],bytestream[pos + 1]];
    pos   += 2;
    self.fw_hash = u16::from_le_bytes(raw_bytes_2); 
    
    raw_bytes_2  = [bytestream[pos],bytestream[pos + 1]];
    pos   += 2;
    self.id      = u16::from_le_bytes(raw_bytes_2);    
    
    raw_bytes_2  = [bytestream[pos],bytestream[pos + 1]];
    pos   += 2;
    self.ch_mask = u16::from_le_bytes(raw_bytes_2); 
   
    let mut raw_bytes_4  = [bytestream[pos + 1],
                            bytestream[pos + 0],
                            bytestream[pos + 3],
                            bytestream[pos + 2]];
    pos   += 4; 
    self.event_ctr = u32::from_be_bytes(raw_bytes_4); 


    raw_bytes_2  = [bytestream[pos],bytestream[pos + 1]];
    pos   += 2;
    self.dtap0   = u16::from_le_bytes(raw_bytes_2); 

    raw_bytes_2  = [bytestream[pos],bytestream[pos + 1]];
    pos   += 2;
    self.dtap1   = u16::from_le_bytes(raw_bytes_2); 
    
    raw_bytes_8  = [0,0,bytestream[pos+1],
                    bytestream[pos + 0],
                    bytestream[pos + 3],
                    bytestream[pos + 2],
                    bytestream[pos + 5],
                    bytestream[pos + 4]];
    pos += 6;
    self.timestamp  = u64::from_be_bytes(raw_bytes_8); 
    for n in 0..NCHN {
        raw_bytes_2  = [bytestream[pos],bytestream[pos + 1]];
        self.ch_head[n] = u16::from_le_bytes(raw_bytes_2);
        pos   += 2;
        for k in 0..NWORDS {
            raw_bytes_2  = [bytestream[pos],bytestream[pos + 1]];
            self.ch_adc[n][k] = i16::from_le_bytes(raw_bytes_2);
            pos += 2;
        }
        raw_bytes_4  = [bytestream[pos + 1],
                        bytestream[pos + 0],
                        bytestream[pos + 3],
                        bytestream[pos + 2]];
        pos   += 4; 
        self.ch_trail[n] = u32::from_be_bytes(raw_bytes_4); 
    }

    raw_bytes_2  = [bytestream[pos+0],bytestream[pos + 1]];
    pos   += 2;
    self.stop_cell       = u16::from_le_bytes(raw_bytes_2); 
    raw_bytes_4  = [bytestream[pos + 1],
                    bytestream[pos + 0],
                    bytestream[pos + 3],
                    bytestream[pos + 2]];
    pos   += 4; 
    self.crc32   = u32::from_be_bytes(raw_bytes_4); 

    raw_bytes_2  = [bytestream[pos],bytestream[pos + 1]];
    pos   += 2;
    self.tail    = u16::from_le_bytes(raw_bytes_2);  // End of event marker
    return pos;
  }


  /// Apply the calibration for time
  /// and voltage.


  pub fn calibrate (&mut self, cal : &[Calibrations;NCHN]) {
    self.voltage_calibration(&cal);
    self.timing_calibration(&cal);
  }

  fn voltage_calibration(&mut self, cal : &[Calibrations;NCHN]) {
    let mut value : f64; 
    for n in 0..NCHN {
      for m in 0..NWORDS {
        value  = self.ch_adc[n][m] as f64;
        value -= cal[n].v_offsets[(n + (self.stop_cell as usize)) %NWORDS];
        value -= cal[n].v_dips[n];
        value *= cal[n].v_inc[(n + (self.stop_cell as usize)) %NWORDS];
        self.voltages[n][m] = value;
        }
      }
    }

  fn timing_calibration( &mut self, cal : &[Calibrations;NCHN]){
    for n in 0..NCHN {
      self.nanoseconds[n][0] = 0.0;
      for m in 1..NWORDS { 
        self.nanoseconds[n][m] = self.nanoseconds[n][m-1] + cal[n].tbin[(m-1+(self.stop_cell as usize))%NWORDS];
      }
    }
  }

  pub fn remove_spikes (&mut self,
                        spikes : &mut [i32;10]) {

  //let mut spikes  : [i32;10] = [0;10];
  let mut filter  : f64;
  let mut dfilter : f64;
  let mut n_symmetric : usize;
  let mut n_neighbor  : usize;

  let mut n_rsp      = 0usize;

  let mut rsp : [i32;10]    = [-1;10];
  //let mut spikes : [i32;10] = [-1;10
  // to me, this seems that should be u32
  // the 10 is for a maximum of 10 spikes (Jeff)
  let mut sp   : [[usize;10];NCHN] = [[0;10];NCHN];
  let mut n_sp : [usize;10]      = [0;10];

  for j in 0..NWORDS as usize {
    for i in 0..NCHN as usize {
      filter = -self.voltages[i][j] + self.voltages[i][(j + 1) % NWORDS] + self.voltages[i][(j + 2) % NWORDS] - self.voltages[i][(j + 3) % NWORDS];
      dfilter = filter + 2.0 * self.voltages[i][(j + 3) % NWORDS] + self.voltages[i][(j + 4) % NWORDS] - self.voltages[i][(j + 5) % NWORDS];
      if filter > 20.0  && filter < 100.0 {
        if n_sp[i] < 10 {   // record maximum of 10 spikes
          sp[i][n_sp[i] as usize] = (j + 1) % NWORDS ;
          n_sp[i] += 1;
        // FIXME - error checking
        } else {return;}            // too many spikes -> something wrong
      }// end of if
      else if dfilter > 40.0 && dfilter < 100.0 && filter > 10.0 {
        if n_sp[i] < 9 {  // record maximum of 10 spikes
          sp[i][n_sp[i] as usize] = (j + 1) % NWORDS ;
          sp[i][(n_sp[i] + 1) as usize] = (j + 3) % NWORDS ;
          n_sp[i] += 2;
        } else { return;} // too many spikes -> something wrong
      } // end of else if

    }// end loop over NCHN
  } // end loop over NWORDS

  // go through all spikes and look for neighbors */
  for i in 0..NCHN {
    for j in 0..n_sp[i] as usize {
      n_symmetric = 0;
      n_neighbor = 0;
      for k in 0..NCHN {
        for l in 0..n_sp[k] as usize {
        //check if this spike has a symmetric partner in any channel
          if (sp[i][j] as i32 + sp[k][l] as i32 - 2 * self.stop_cell as i32) as i32 % NWORDS as i32 == 1022 {
            n_symmetric += 1;
            break;
          }
        }
      } // end loop over k
      // check if this spike has same spike is in any other channels */
      //for (k = 0; k < nChn; k++) {
      for k in 0..NCHN {
        if i != k {
          for l in 0..n_sp[k] {
            if sp[i][j] == sp[k][l] {
            n_neighbor += 1;
            break;
            }
          } // end loop over l   
        } // end if
      } // end loop over k

      if n_neighbor >= 2 {
        for k in 0..n_rsp {
          if rsp[k] == sp[i][j] as i32 {break;} // ignore repeats
          if n_rsp < 10 && k == n_rsp {
            rsp[n_rsp] = sp[i][j] as i32;
            n_rsp += 1;
          }
        }  
      }

    } // end loop over j
  } // end loop over i

  // recognize spikes if at least one channel has it */
  //for (k = 0; k < n_rsp; k++)
  let magic_value : f64 = 14.8;
  let mut x : f64;
  let mut y : f64;

  let mut skip_next : bool = false;
  for k in 0..n_rsp {
    if skip_next {
      skip_next = false;
      continue;
    }
    spikes[k] = rsp[k];
    //for (i = 0; i < nChn; i++)
    for i in 0..NCHN {
      if k < n_rsp && i32::abs(rsp[k] as i32 - rsp[k + 1] as i32 % NWORDS as i32) == 2
      {
        // remove double spike 
        let j = if rsp[k] > rsp[k + 1] {rsp[k + 1] as usize}  else {rsp[k] as usize};
        x = self.voltages[i][(j - 1) % NWORDS];
        y = self.voltages[i][(j + 4) % NWORDS];
        if f64::abs(x - y) < 15.0
        {
          self.voltages[i][j % NWORDS] = x + 1.0 * (y - x) / 5.0;
          self.voltages[i][(j + 1) % NWORDS] = x + 2.0 * (y - x) / 5.0;
          self.voltages[i][(j + 2) % NWORDS] = x + 3.0 * (y - x) / 5.0;
          self.voltages[i][(j + 3) % NWORDS] = x + 4.0 * (y - x) / 5.0;
        }
        else
        {
          self.voltages[i][j % NWORDS] -= magic_value;
          self.voltages[i][(j + 1) % NWORDS] -= magic_value;
          self.voltages[i][(j + 2) % NWORDS] -= magic_value;
          self.voltages[i][(j + 3) % NWORDS] -= magic_value;
        }
      }
      else
      {
        // remove single spike 
        x = self.voltages[i][((rsp[k] - 1) % NWORDS as i32) as usize];
        y = self.voltages[i][(rsp[k] + 2) as usize % NWORDS];
        if f64::abs(x - y) < 15.0 {
          self.voltages[i][rsp[k] as usize] = x + 1.0 * (y - x) / 3.0;
          self.voltages[i][(rsp[k] + 1) as usize % NWORDS] = x + 2.0 * (y - x) / 3.0;
        }
        else
        {
          self.voltages[i][rsp[k] as usize] -= magic_value;
          self.voltages[i][(rsp[k] + 1) as usize % NWORDS] -= magic_value;
        }
      } // end loop over nchn
    } // end loop over n_rsp
    if k < n_rsp && i32::abs(rsp[k] - rsp[k + 1] % NWORDS as i32) == 2
      {skip_next = true;} // skip second half of double spike
    } // end loop over k
  }
  
  pub fn set_threshold(&mut self, thr : f64, ch : usize) {
      self.threshold[ch] = thr;
  }

  pub fn set_cfds_fraction(&mut self, fraction : f64, ch : usize) {
      self.cfds_fraction[ch] = fraction;
  }
  
  pub fn set_ped_begin(&mut self, time : f64, ch : usize) {
      match self.time_2_bin(time, ch) {
          Err(err) => println!("Can not find bin for time {}, ch {}, err FIXME", time, ch),
          Ok(begin) => {self.ped_begin_bin[ch] = begin;}
      }
  }

  pub fn set_ped_range(&mut self, range : f64, ch : usize) {
    // This is a little convoluted, but we must convert the range (in
    // ns) into bins
    match self.time_2_bin(self.nanoseconds[ch][self.ped_begin_bin[ch]] + range, ch) {
        Err(err)      => println!("Can not set pedestal range for range {} for ch {}", range, ch),
        Ok(bin_range) => {self.ped_bin_range[ch] = bin_range;}
    }
  }

  pub fn subtract_pedestal(&mut self, ch : usize) {
    for n in 0..NWORDS {
      self.voltages[ch][n] -= self.pedestal[ch];
    }
  }

  pub fn calc_ped_range(&mut self, ch : usize) {
    let mut sum  = 0f64;
    let mut sum2 = 0f64;

    for n in self.ped_begin_bin[ch]..self.ped_begin_bin[ch] + self.ped_bin_range[ch] {
      if f64::abs(self.voltages[ch][n]) < 10.0 {
        sum  += self.voltages[ch][n];
        sum2 += self.voltages[ch][n]*self.voltages[ch][n];
      }
    }
    let average = sum/(self.ped_bin_range[ch] as f64);
    self.pedestal[ch] = average;
    self.pedestal_sigma[ch] = f64::sqrt(sum2/(self.ped_bin_range[ch] as f64 - (average*average)))

  }

  fn time_2_bin(&self, t_ns : f64, ch : usize) -> Result<usize, WaveformError> {
    // Given a time in ns, find the bin most closely corresponding to that time
    for n in 0..NWORDS {
      if self.nanoseconds[ch][n] > t_ns {
        return Ok(n-1);
      }
    }
    println!("Did not find a bin corresponding to the given time {} for ch {}", t_ns, ch);
    return Err(WaveformError::TimesTooSmall);
  }


  // 
  // Return the bin with the maximum DC value
  //
  fn get_max_bin(&self, lower_bound : usize, upper_bound : usize, ch : usize ) -> usize {
    let rel_upper_bound = NWORDS - upper_bound;
    //println!("{} {} {}", lower_bound, upper_bound, rel_upper_bound);
    assert!((rel_upper_bound - lower_bound) <= NWORDS);
    let mut maxval = self.voltages[ch][lower_bound];
    let mut maxbin = lower_bound;
    for n in lower_bound+1..lower_bound+rel_upper_bound {
      // I think the - sign is bc of pmt waveforms...
      //if maxval < self.wave[n] {
      if maxval > self.voltages[ch][n] {
        maxval  = self.voltages[ch][n];
        maxbin  = n;
      }
    } // end for
    return maxbin;
  } // end fn

  pub fn find_cfd_simple(&mut self, peak_num : usize, ch : usize) -> f64 {
    if peak_num > self.num_peaks[ch] {return self.voltages[ch][NWORDS];}
    // FIXME
    let mut idx = self.get_max_bin(self.begin_peak[ch][peak_num],
                                   self.end_peak[ch][peak_num]-self.begin_peak[ch][peak_num],
                                   ch);

    if idx < 1 {idx = 1;}
    let mut sum : f64 = 0.0;
    for n in idx-1..idx+1 {sum += self.voltages[ch][n];}
    let cfds_frac  : f64 = 0.2;
    let tmp_thresh : f64 = f64::abs(cfds_frac * (sum / 3.0));

    // Now scan through the waveform around the peak to find the bin
    // crossing the calculated threshold. Bin idx is the peak so it is
    // definitely above threshold. So let's walk backwards through the
    // trace until we find a bin value less than the threshold.
    let mut lo_bin : usize = NWORDS;
    let mut n = idx;
    if self.begin_peak[ch][peak_num] >= 10 {
      while n > self.begin_peak[ch][peak_num] - 10 {
      //for n in (idx..self.begin_peak[peak_num] - 10).rev() {
        if f64::abs(self.voltages[ch][n]) < tmp_thresh {
          lo_bin = n;
          break;
        }
        n -= 1;
      }  
    }

    let mut cfd_time : f64 = 0.0;
    if lo_bin < NWORDS {
      cfd_time = self.find_interpolated_time(tmp_thresh, lo_bin, 1, ch);  
    }
    else {cfd_time = self.voltages[ch][NWORDS - 1];} 

    // save it in member variable
    self.tdcs[ch][peak_num] = cfd_time;
    return cfd_time;
  }

  pub fn find_interpolated_time (&self,
                                 //adc       : [f64;NWORDS],
                                 //times     : [f64;NWORDS], 
                                 mut threshold : f64,
                                 mut idx       : usize,
                                 size          : usize, 
                                 ch            : usize) -> f64 
  {
    threshold = threshold.abs();
    let mut lval  = (self.voltages[ch][idx]).abs();
    let mut hval : f64 = 0.0; 
    if size == 1 {
      hval = (self.voltages[ch][idx+1]).abs();
    } else {
    for n in idx+1..idx+size {
      hval = self.voltages[ch][n].abs();
      if (hval>=threshold) && (threshold<=lval) { // Threshold crossing?
        idx = n-1; // Reset idx to point before crossing
        break;
        }
      lval = hval;
      }
    }
    if ( lval > threshold) && (size != 1) {
      return self.nanoseconds[ch][idx];
    } else if lval == hval {
      return self.nanoseconds[ch][idx];
    } else {
      return self.voltages[ch][idx] 
          + (threshold-lval)/(hval-lval) * (self.voltages[ch][idx+1]
          - self.voltages[ch][idx]);
      //float time = WaveTime[idx] +  
      //  (thresh-lval)/(hval-lval) * (WaveTime[idx+1]-WaveTime[idx]) ;
      }
  }

  fn find_peaks(&mut self,
                start_time  : f64,
                window_size : f64,
                threshold   : f64,
                ch          : usize) {
    // FIXME - replace unwrap calls
    let start_bin  = self.time_2_bin(start_time, ch).unwrap();
    let window_bin = self.time_2_bin(start_time + window_size, ch).unwrap() - start_bin;

    // minimum number of bins a peak must have
    // over threshold so that we consider it 
    // a peak
    let min_peak_width       = 3usize; 
    let mut pos              = 0usize;
    let mut peak_bins        = 0usize;
    let mut n_peaks_detected = 0usize;
    let mut peak_ctr         = 0usize;
    while (self.voltages[ch][pos] < threshold) && (pos < NWORDS) {
      pos += 1;
    }
    for n in pos..start_bin + window_bin {
      if self.voltages[ch][n] > threshold {
        peak_bins += 1;
        if peak_bins == min_peak_width {
          // we have a new peak
          if n_peaks_detected == MAX_NUM_PEAKS {
            println!("Max number of peaks reached in this waveform");
            break;
          }
          self.begin_peak[ch][peak_ctr] = n - (min_peak_width - 1); 
          self.spikes    [ch][peak_ctr] = 0;
          self.end_peak  [ch][peak_ctr] = 0;
          peak_ctr += 1;
        } else if peak_bins > min_peak_width {
          for k in 0..3 {
            if self.voltages[ch][n-k] > self.voltages[ch][n-(k+1)]
              {continue;}
          }
          if self.end_peak[ch][peak_ctr-1] == 0 {
            self.end_peak[ch][peak_ctr-1] = n; // Set last bin included in peak
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
    self.num_peaks[ch] = peak_ctr;
    self.begin_peak[ch][peak_ctr] = NWORDS; // Need this to measure last peak correctly
    //peaks_found = 1;
  }

  pub fn print (&self) {
    println!("======");
    println!("==> HEAD       {} ", self.head);
    println!("==> STATUS     {} ", self.status);
    println!("==> LEN        {} ", self.len);
    println!("==> ROI        {} ", self.roi);
    println!("==> DNA        {} ", self.dna);
    println!("==> FW_HASH    {} ", self.fw_hash);
    println!("==> ID         {} ", self.id);
    println!("==> CH_MASK    {} ", self.ch_mask);
    println!("==> EVT_CTR    {} ", self.event_ctr);
    println!("==> DTAP0      {} ", self.dtap0);
    println!("==> DTAP1      {} ", self.dtap1);
    println!("==> TIMESTAMP  {} ", self.timestamp);
    println!("==> STOP_CELL  {} ", self.stop_cell);
    println!("==> CRC32      {} ", self.crc32);
    println!("==> TAIL       {} ", self.tail);
    println!("======");

  }
}    

/***********************************/

impl Default for BlobData {
    fn default() -> BlobData {
        BlobData {
            head            : 0, // Head of event marker
            status          : 0,
            len             : 0,
            roi             : 0,
            dna             : 0, 
            fw_hash         : 0,
            id              : 0,   
            ch_mask         : 0,
            event_ctr       : 0,
            dtap0           : 0,
            dtap1           : 0,
            timestamp       : 0,
            ch_head         : [0; NCHN],
            ch_adc          : [[0; NWORDS]; NCHN],
            ch_trail        : [0; NCHN],
            stop_cell       : 0,
            crc32           : 0,
            tail            : 0, // End of event marker

            voltages        : [[0.0; NWORDS]; NCHN],
            nanoseconds     : [[0.0; NWORDS]; NCHN],
  
            threshold      : [0.0;NCHN],
            cfds_fraction  : [0.0;NCHN],
            ped_begin_bin  : [0;NCHN],
            ped_bin_range  : [0;NCHN],    
            pedestal       : [0.0;NCHN],
            pedestal_sigma : [0.0;NCHN],
  
            peaks      : [[0;MAX_NUM_PEAKS];NCHN],
            tdcs       : [[0.0;MAX_NUM_PEAKS];NCHN],
            charge     : [[0.0;MAX_NUM_PEAKS];NCHN],
            width      : [[0.0;MAX_NUM_PEAKS];NCHN], 
            height     : [[0.0;MAX_NUM_PEAKS];NCHN],    
            num_peaks  : [0;NCHN],
            //stop_cell  : [u16;NCHN],
            begin_peak : [[0;MAX_NUM_PEAKS];NCHN],
            end_peak   : [[0;MAX_NUM_PEAKS];NCHN],
            spikes     : [[0;MAX_NUM_PEAKS];NCHN],
        }
    }
}

/***********************************/



