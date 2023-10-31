//! The ReadoutBoardCalibration is 
//! the first stage in a 3 stage 
//! calibration process for the tof
//!
//! It converts adc/bin data for the 
//! individual waveforms into mV/ns.
//!
//! This has to be done individually 
//! per board and is an automated 
//! process.
//!
use std::fs::File;
use std::io::{self, BufRead, BufReader};
use std::path::Path;
use std::fmt;

use crate::constants::{NWORDS, NCHN};
use crate::errors::{WaveformError,
                    CalibrationError};
use crate::serialization::{Serialization,
                           parse_bool,
                           parse_u16,
                           parse_f32,
                           SerializationError};
use crate::events::RBEvent;
use crate::events::rb_event::unpack_traces_f32;
use crate::packets::{PacketType,
                     TofPacket};
use crate::io::read_file;

#[cfg(feature = "random")] 
use crate::FromRandom;
#[cfg(feature = "random")]
extern crate rand;
#[cfg(feature = "random")]
use rand::Rng;

extern crate statistical;

// FIXME -fix the test!


/***********************************/

#[derive(Debug, Copy, Clone)]
pub enum Edge {
  Rising,
  Falling,
  Average,
  None
}


/***********************************/

// helper
fn read_lines<P>(filename: P) -> io::Result<io::Lines<io::BufReader<File>>>
where P: AsRef<Path>, {
  let file = File::open(filename)?;
  Ok(io::BufReader::new(file).lines())
}

/***********************************/

fn find_zero_crossings(trace: &Vec<f32>) -> Vec<usize> {
  let mut zero_crossings = Vec::new();
  for i in 1..trace.len() {
    if (trace[i - 1] > 0.0 && trace[i] < 0.0) || (trace[i - 1] < 0.0 && trace[i] > 0.0) {
      zero_crossings.push(i);
    }
  }
  zero_crossings
}

/***********************************/


///
///
/// # Arguments:
///
/// * dts : some version of tcal data? FIXME
fn get_periods(trace   : &Vec<f32>,
               dts     : &Vec<f32>,
               nperiod : f32,
               edge    : &Edge) -> (Vec<usize>, Vec<f32>) {
  let mut trace_c = trace.clone();
  let mut periods = Vec::<f32>::new();
  if trace_c.len() == 0 {
    let foo = Vec::<usize>::new();
    return (foo, periods);
  }
  let firstbin : usize = 20;
  let nskip : f32 = 0.0;
  let lastbin = firstbin + (nperiod * (900.0/nperiod).floor()).floor() as usize;
  //info!("firstbin {} lastbin {}", firstbin, lastbin);
  let mut vec_for_median = Vec::<f32>::new();
  for bin in firstbin..lastbin {
    vec_for_median.push(trace[bin]);
  }
  let median_val = median(&vec_for_median);
  for k in 0..trace_c.len() {
    trace_c[k] -= median_val;
  }
  let mut zcs = find_zero_crossings(&trace_c);
  //trace!("Found {} zero crossings!", zcs.len());
  let mut zcs_nskip = Vec::<usize>::with_capacity(zcs.len());
  for k in 0..zcs.len() {
    if zcs[k] > nskip as usize {
      zcs_nskip.push(zcs[k]);
    }
  }
  zcs = zcs_nskip;
  let mut zcs_filter = Vec::<usize>::with_capacity(zcs.len());
  for k in 0..zcs.len() {
    match edge {
      Edge::Rising => {
        if trace_c[zcs[k]] < 0.0 {
          zcs_filter.push(zcs[k]);
        }
      }
      Edge::Falling => {
        // What about the equal case?
        if trace_c[zcs[k]] > 0.0 {
          zcs_filter.push(zcs[k]);
        }
      },
      Edge::None => {
        warn!("Unsure what to do for Edge::None");
      },
      Edge::Average => {
        warn!("Unsure what to do for Edge::Average");
      }
    }
  }
  
  zcs = zcs_filter;
  if zcs.len() < 3 {
    return (zcs, periods);
  }
  
  for k in 0..zcs.len() -1 {
    let zcs_a  = &zcs[k];
    let zcs_b  = &zcs[k+1];
    // FIXME - there is an issue with the last
    // zero crossings
    if zcs_a + 2 > trace_c.len() || zcs_b + 2 > trace_c.len() {
      continue;
    }
    let tr_a   = &trace_c[*zcs_a..*zcs_a+2];
    let tr_b   = &trace_c[*zcs_b..*zcs_b+2];
    let mut period : f32 = 0.0;
    for n in zcs_a+1..*zcs_b {
      period += dts[n];
    }
    period += dts[*zcs_a]*f32::abs(tr_a[1]/(tr_a[1] - tr_a[0])); // first semi bin
    period += dts[*zcs_b]*f32::abs(tr_b[1]/(tr_b[1] - tr_b[0])); // first semi bin
    if period.is_nan() {
      warn!("NAN in period found!");
      //println!("tr_a {} {}", tr_a[1], tr_a[0]);
      //println!("tr_a {} {}", tr_b[1], tr_b[0]);
      continue;
    }
    //println!("zcs_b, zcs_a, nperiod {} {} {}", zcs_b, zcs_a, nperiod);
    //if f32::abs((zcb-zca)-nperiod as usize) > 5:
    //         zcs = zcs[:i+1]
    if f32::abs(*zcs_b as f32 - *zcs_a as f32 - nperiod) > 5.0 {
      let mut zcs_tmp = Vec::<usize>::new();
      zcs_tmp.extend_from_slice(&zcs[0..k+1]);
      zcs = zcs_tmp;
      break;
    }
    periods.push(period);
  }
  debug!("Calculated {} zero-crossings and {} periods!", zcs.len(), periods.len());
  (zcs, periods)
}

/***********************************/

fn mean(input: &Vec<f32>) -> f32 {
  if input.len() == 0 {
    error!("Vector is empty, can not calculate mean!");
    return f32::NAN;
  }
  let n_entries = input.len() as f32;
  let mut sum : f32 = 0.0;
  for k in input.iter() {
    sum += k;
  }
  sum / n_entries
}

/***********************************/

fn median(input: &Vec<f32>) -> f32 {
  let mut data = input.clone();
  
  let len = data.len();
  if len == 0 {
    error!("Vector is empty, can not calculate median!");
    return f32::NAN;
  }
  // TODO This might panic! Is it ok?
  data.sort_by(|a, b| a.partial_cmp(b).unwrap());

  if len % 2 == 0 {
    // If the vector has an even number of elements, calculate the average of the middle two
    let mid = len / 2;
    (data[mid - 1] + data[mid]) / 2.0
  } else {
    // If the vector has an odd number of elements, return the middle element
    let mid = len / 2;
    data[mid]
  }
}

/***********************************/


/// Calculate the median over axis 1
/// This is used for a single channel
/// data is row - columns
fn calculate_column_medians(data: &Vec<Vec<f32>>) -> Vec<f32> {
  // Get the number of columns (assuming all sub-vectors have the same length)
  let num_columns = data[0].len();
  let num_rows    = data.len();
  //println!("num rows {}", num_rows);
  // Initialize a Vec to store the column-wise medians
  let mut column_medians: Vec<f32> = vec![0.0; num_columns];
  debug!("Calculating medians for {} columns!", num_columns);
  debug!("Calculating medians for {} rows!", num_rows);
  // Calculate the median for each column across all sub-vectors, ignoring NaN values
  for col in 0..num_columns  {
    let mut col_vals = vec![0.0; num_rows];
    //let mut col_vals = Vec::<f32>::new();
    for k in 0..num_rows {
      col_vals[k] = data[k][col];
    }
    //println!("{:?}", col_vals);
    col_vals.retain(|x| !x.is_nan());
    //println!("len col_vals {}", col_vals.len()); 
    //info!("Col {col} has len {}", col_vals.len());
    //column_medians[col] = median(&col_vals);//.unwrap_or(f32::NAN);
    if col_vals.len() == 0 {
      column_medians[col] = f32::NAN;
    } else {
      column_medians[col] = statistical::median(col_vals.as_slice());//.unwrap_or(f32::NAN);
    }
    //if column_medians[col] == 0.0 {
    //  println!("{:?}", col_vals);
    //}
  }
  column_medians
}

/***********************************/

/// Calculate the mean over column 1
fn calculate_column_means(data: &Vec<Vec<f32>>) -> Vec<f32> {
  // Get the number of columns (assuming all sub-vectors have the same length)
  let num_columns = data[0].len();
  let num_rows    = data.len();
  // Initialize a Vec to store the column-wise medians
  let mut column_means: Vec<f32> = vec![0.0; num_columns];
  info!("Calculating means for {} columns!", num_columns);
  // Calculate the median for each column across all sub-vectors, ignoring NaN values
  for col in 0..num_columns  {
    let mut col_vals = vec![0.0; num_rows];
    for k in 0..num_rows {
      col_vals[k] = data[k][col];
    }
    col_vals.retain(|x| !x.is_nan());
    //info!("Col {col} has len {}", col_vals.len());
    column_means[col] = mean(&col_vals);//.unwrap_or(f32::NAN);
  }
  column_means
}

/***********************************/

fn roll<T: Clone>(vec: &mut Vec<T>, shift: isize) {
  let len = vec.len() as isize;
  
  // Calculate the effective shift by taking the remainder of the shift
  // operation with the length of the vector to ensure it's within bounds.
  let effective_shift = (shift % len + len) % len;
  
  // If the shift is zero, there's nothing to do.
  if effective_shift == 0 {
    return;
  }
  
  // Clone the vector and clear it.
  let temp = vec.clone();
  vec.clear();
  
  // Split the vector into two parts and swap them to achieve the roll effect.
  if effective_shift > 0 {
    let (left, right) = temp.split_at(temp.len() - effective_shift as usize);
    vec.extend_from_slice(right);
    vec.extend_from_slice(left);
  } else {
    let (left, right) = temp.split_at(-effective_shift as usize);
    vec.extend_from_slice(right);
    vec.extend_from_slice(left);
  }
}

/***********************************/

#[derive(Debug, Clone, PartialEq)]
pub struct RBCalibrations {
  pub rb_id      : u8,
  pub d_v        : f32, // input voltage difference between 
                        // vcal_data and noi data
  pub serialize_event_data : bool,
  // calibration constants
  pub v_offsets : [[f32;NWORDS];NCHN], // voltage offset
  pub v_dips    : [[f32;NWORDS];NCHN], // voltage "dip" (time-dependent correction)
  pub v_inc     : [[f32;NWORDS];NCHN], // voltage increment (mV/ADC unit)
  pub tbin      : [[f32;NWORDS];NCHN], // cell width (ns)

  // calibration data
  pub vcal_data : Vec::<RBEvent>,
  pub tcal_data : Vec::<RBEvent>,
  pub noi_data  : Vec::<RBEvent>,
}

impl RBCalibrations {
  // skip the first n cells for the 
  // voltage calibration. Historically,
  // this had been 2.
  pub const NSKIP       : usize = 2;
  pub const SINMAX      : usize = 60; // ~1000 ADC units
  pub const DVCUT       : f32   = 15.0; // ns away that should be considered
  pub const NOMINALFREQ : f32   = 2.0; // nominal sampling frequency,
                                       // GHz
  pub const CALFREQ     : f32   = 0.025; // calibration sine wave frequency (25MHz)
  
  /// Remove events with invalid traces or event fragment bits set
  pub fn clean_input_data(&mut self) {
    self.vcal_data.retain(|x| !x.header.broken & !x.header.lost_trigger & !x.header.event_fragment); 
    self.tcal_data.retain(|x| !x.header.broken & !x.header.lost_trigger & !x.header.event_fragment); 
    self.noi_data.retain(|x| !x.header.broken & !x.header.lost_trigger & !x.header.event_fragment); 
  }

  // channel is from 0-8
  pub fn apply_vcal_constants(&self,
                              adc       : &Vec<f32>,
                              channel   : usize,  
                              stop_cell : usize) -> Vec<f32> {
    let mut waveform = Vec::<f32>::with_capacity(adc.len());
    let mut value : f32;
    for k in 0..adc.len() {
      value  = adc[k] as f32;
      value -= self.v_offsets[channel][(k + (stop_cell)) %NWORDS];
      value -= self.v_dips   [channel][k];
      value *= self.v_inc    [channel][(k + (stop_cell)) %NWORDS];
      waveform.push(value);
    } 
    waveform
  }

  /// Calculate the offset and dips calibration constants 
  /// for input data. 
  ///
  /// # Return:
  ///
  /// offsets, dips
  fn voltage_offset_and_dips(input_vcal_data : &Vec<RBEvent>) 
  -> Result<(Vec<Vec<f32>>, Vec<Vec<f32>>), CalibrationError> {
    if input_vcal_data.len() == 0 {
      return Err(CalibrationError::EmptyInputData);
    }
    let mut all_v_offsets = Vec::<Vec::<f32>>::new();
    let mut all_v_dips    = Vec::<Vec::<f32>>::new();
    let nchn = input_vcal_data[0].header.get_nchan();

    info!("Found {nchn} channels!");
    for _ in 0..nchn {
      let empty_vec_off : Vec<f32> = vec![0.0;NWORDS];
      all_v_offsets.push(empty_vec_off);  
      let empty_vec_dip : Vec<f32> = vec![0.0;NWORDS];
      all_v_dips.push(empty_vec_dip);  
    }

    // we temporarily get the adc traces
    // traces are [channel][event][adc_cell]
    let mut traces        = unpack_traces_f32(&input_vcal_data);
    let mut rolled_traces = traces.clone();
    for ch in 0..nchn {
      for n in 0..input_vcal_data.len() {
        for k in 0..Self::NSKIP {
          traces[ch][n][k] = f32::NAN;
          rolled_traces[ch][n][k] = f32::NAN;
        }
        // the traces are filled and the first 2 bins
        // marked with nan, now need to get "rolled over",
        // so that they start with the stop cell
        roll(&mut rolled_traces[ch][n],
             input_vcal_data[n].header.stop_cell as isize); 
      }// first loop over events done
      let v_offsets = calculate_column_medians(&rolled_traces[ch]);
      //println!("v offsets {:?}", v_offsets);
      info!("We calculated {} voltage offset values for ch {}", v_offsets.len(), ch);
      // fill these in the prepared array structure
      for k in 0..v_offsets.len() {
        all_v_offsets[ch][k] = v_offsets[k];
      }
      for (n, ev) in input_vcal_data.iter().enumerate() {
        // now we roll the v_offsets back
        let mut v_offsets_rolled = v_offsets.clone();
        roll(&mut v_offsets_rolled, -1*ev.header.stop_cell as isize);
        for k in 0..traces[ch][n].len() {
          traces[ch][n][k] -= v_offsets_rolled[k];
        }
      }
      let v_dips = calculate_column_medians(&traces[ch]);
      for k in 0..v_dips.len() {
        if k < Self::NSKIP {
          all_v_dips[ch][k] = 0.0;
        } else {
          all_v_dips[ch][k] = v_dips[k];
        }
      }
    }
    Ok((all_v_offsets, all_v_dips))
  }
  

  pub fn timing_calibration(&self,
                        edge : &Edge) 
  -> Result<Vec<Vec<f32>>, CalibrationError> {
    if self.tcal_data.len() == 0 {
      error!("Input data for timing calibration is empty!");
      return Err(CalibrationError::EmptyInputData);
    }
    let mut all_tcal = Vec::<Vec::<f32>>::new();
    for _ in 0..NCHN {
      let nfreq_vec : Vec<f32> = vec![1.0/Self::NOMINALFREQ;NWORDS];
      all_tcal.push(nfreq_vec);  
    }

    // we temporarily get the adc traces
    // traces are [channel][event][adc_cell]
    let adc_traces = unpack_traces_f32(&self.tcal_data);
    let mut traces  = adc_traces.clone();
    let mut dtraces = traces.clone();
    let trace_len   = traces[0][0].len();
    for ch in 0..NCHN {
      for (n, ev) in self.tcal_data.iter().enumerate() {
        traces[ch][n] = self.apply_vcal_constants(&adc_traces[ch][n], ch, ev.header.stop_cell as usize);
        for k in 0..trace_len {
          if k < Self::NSKIP {
            traces[ch][n][k]  = f32::NAN;
          }
          let  trace_val =  traces[ch][n][k];
          //let dtrace_val = dtraces[ch][n][k];
          if f32::abs(trace_val) > Self::SINMAX as f32 { 
            traces[ch][n][k]  = f32::NAN;
          }// the traces are filled and the first 2 bins
        }
      }
    }
    let mut drolled_traces = traces.clone();
    //println!("{:?}", traces[0][0]);
    for ch in 0..NCHN {
      for n in 0..self.tcal_data.len() {
      //for (n, ev) in self.tcal_data.iter().enumerate() {
        // marked with nan, now need to get "rolled over",
        // so that they start with the stop cell
        roll(&mut drolled_traces[ch][n],
             self.tcal_data[n].header.stop_cell as isize); 
        //for ch in 0..NCHN { 
        //  for ev in 0..traces[ch].len() {
        for k in 0..traces[ch][n].len() {
          let mut dval : f32;
          if k == 0 {
            dval = drolled_traces[ch][n][0] - drolled_traces[ch][n][traces[ch][n].len() -1];
          } else {
            dval = drolled_traces[ch][n][k] - drolled_traces[ch][n][k-1];      
          }
          //println!("{}", dval);
          match edge {
            Edge::Rising | 
            Edge::Average => {
              if dval < 0.0 {
                dval = f32::NAN;
              }
            },
            Edge::Falling => {
              if dval > 0.0 {
                dval = f32::NAN;
              }
            },
            Edge::None => (),
          }
          dval = f32::abs(dval);
          // FIXME: check the 15
          if f32::abs(dval - 15.0) > Self::DVCUT {
            dval = f32::NAN;
          }
          if k == 0 {
            dtraces[ch][n][traces[ch][n].len() -1] = dval;
          } else {
            dtraces[ch][n][k-1] = dval;      
          }
        } 
        //println!("{:?}", dtraces[ch][n]);
      } // end loop over events
      let col_means = calculate_column_means(&dtraces[ch]);
      let ch_mean   = mean(&col_means);
      for k in 0..all_tcal[ch].len() {
        all_tcal[ch][k] *= col_means[k]/ch_mean;  
      }
    } // end loop over channels
    Ok(all_tcal)
  }

  /// Call to the calibration routine, using
  /// the set input data
  pub fn calibrate(&mut self) -> Result<(), CalibrationError>{
    if self.vcal_data.len() == 0
    || self.tcal_data.len() == 0 
    || self.noi_data.len() == 0 {
      return Err(CalibrationError::EmptyInputData);
    }
    info!("Starting voltage calibration!");
    let (v_offsets_high, _v_dips_high) 
        = Self::voltage_offset_and_dips(&self.vcal_data)?;
    let (v_offsets_low, v_dips_low) 
        = Self::voltage_offset_and_dips(&self.noi_data)?;
    // which of the v_offsets do we actually use?

    for ch in 0..NCHN {
      for k in 0..NWORDS {
        self.v_offsets[ch][k] = v_offsets_low[ch][k];
        self.v_dips[ch][k]    = v_dips_low[ch][k];
        self.v_inc[ch][k]     = self.d_v/(v_offsets_high[ch][k] - v_offsets_low[ch][k]);
      }
    }
    // at this point, the voltage calibration is complete
    info!("Voltage calibration complete!");
    info!("Begin timing calibration!");
    warn!("Calibration only supported for Edge::Average!");
    // this just suppresses a warning
    // We have to think if edge will be
    // a parameter or a constant.
    let mut edge    = Edge::None;
    if matches!(edge, Edge::None) {
      edge = Edge::Average;
    }

    let mut tcal_av = self.timing_calibration( &edge)?;
    if matches!(edge, Edge::Average) {
      edge = Edge::Falling;
      let tcal_falling = self.timing_calibration(&edge)?;
      for ch in 0..NCHN {
        for k in 0..tcal_av.len() {
          tcal_av[ch][k] += tcal_falling[ch][k];
          tcal_av[ch][k] /= 2.0;
        }
      } 
      // for further calibration procedure
      edge = Edge::Rising;
    } else {
      error!("This is not implemented for any other case yet!");
      todo!();
    }
    
    // another set of constants
    //nevents,nchan,tracelen = gbf.traces.shape
    let mut damping   : f32 = 0.1;
    let corr_limit    : f32 = 0.05;
    //let n_iter_period : f32 = 1000; //#500 or nevents #

    let mut tcal_av_cp = tcal_av.clone();
    let nperiod = Self::NOMINALFREQ/Self::CALFREQ; 
    //let mut n_correct = vec![0.0;NWORDS];

    for n in 0..self.tcal_data.len() { // this is the nIterPeriod or nevents loop
                                       // we choose nevents
      let stop_cell = self.tcal_data[n].header.stop_cell;
      for ch in 0..NCHN {
        roll(&mut tcal_av_cp[ch], -1* (stop_cell as isize));
        // tcal data per definition has all channels active...
        // FIXME - we should not just assume this for each event.
        let tcal_trace = &self.tcal_data[n].get_channel_by_id(ch).expect("Unable to get ch");
        let mut tcal_trace_f32 = Vec::<f32>::with_capacity(tcal_trace.len());
        for j in tcal_trace.iter() {
          tcal_trace_f32.push(*j as f32);
        }
        let tracelen = tcal_trace_f32.len();
        let (zcs, periods) = get_periods(&tcal_trace_f32,
                                         &tcal_av_cp[ch],
                                         nperiod,
                                         &edge);
        for (n_p,period) in periods.iter().enumerate() {
          if *period == 0.0 {
            println!("{:?}", periods);
          }
          if period.is_nan() {
            println!("{:?}", periods);
          }
          let zcs_a = zcs[n_p] + stop_cell as usize;
          let zcs_b = zcs[n_p + 1] + stop_cell as usize;
          let mut corr = (1.0/Self::CALFREQ)/period;
          if matches!(edge, Edge::None) {
            corr *= 0.5;
          }
          if f32::abs(corr - 1.0) > corr_limit {
            continue;
          }
          corr = (corr-1.0)*damping + 1.0;
          let zcs_a_tl = zcs_a%tracelen;
          let zcs_b_tl = zcs_b%tracelen;
          if zcs_a < tracelen && zcs_b > tracelen {
            for m in zcs_a..tcal_av[ch].len() {
              tcal_av[ch][m] *= corr;
            }
            for m in 0..zcs_b_tl {
              tcal_av[ch][m] *= corr;
            }
          } else {
            for m in zcs_a_tl..zcs_b_tl {
              tcal_av[ch][m] *= corr;
            }
          }
        }
        //n_correct[ch] += 1.0;
      } // end over nchannel
      damping *= 0.99;
    } // end loop over n_iter_period
    for ch in 0..NCHN {
      for k in 0..NWORDS {
        self.tbin[ch][k] = tcal_av[ch][k];
      }
    }
    Ok(())
  }

  /// Apply the spike cleaning to all channels
  pub fn spike_cleaning(voltages  : &mut Vec<Vec<f32>>,
                        stop_cell : u16) 
    -> Result<(), WaveformError> {

    let mut spikes      : [i32;10] = [0;10];
    let mut filter      : f32;
    let mut dfilter     : f32;
    let mut n_neighbor  : usize;
    let mut n_rsp       = 0usize;
    let mut rsp : [i32;10]    = [-1;10];
    // to me, this seems that should be u32
    // the 10 is for a maximum of 10 spikes (Jeff)
    let mut sp   : [[usize;10];NCHN] = [[0;10];NCHN];
    let mut n_sp : [usize;10]        = [0;10];

    for j in 0..NWORDS as usize {
      for i in 0..NCHN as usize {
        filter = -voltages[i][j] + voltages[i][(j + 1) % NWORDS] + voltages[i][(j + 2) % NWORDS] - voltages[i][(j + 3) % NWORDS];
        dfilter = filter + 2.0 * voltages[i][(j + 3) % NWORDS] + voltages[i][(j + 4) % NWORDS] - voltages[i][(j + 5) % NWORDS];
        if filter > 20.0  && filter < 100.0 {
          if n_sp[i] < 10 {   // record maximum of 10 spikes
            sp[i][n_sp[i] as usize] = (j + 1) % NWORDS ;
            n_sp[i] += 1;
          } else {
            return Err(WaveformError::TooSpiky);
          }            // too many spikes -> something wrong
        }// end of if
        else if dfilter > 40.0 && dfilter < 100.0 && filter > 10.0 {
          if n_sp[i] < 9 {  // record maximum of 10 spikes
            sp[i][n_sp[i] as usize] = (j + 1) % NWORDS ;
            sp[i][(n_sp[i] + 1) as usize] = (j + 3) % NWORDS ;
            n_sp[i] += 2;
          } else {
            return Err(WaveformError::TooSpiky);
          } // too many spikes -> something wrong
        } // end of else if

      }// end loop over NCHN
    } // end loop over NWORDS

    // go through all spikes and look for neighbors */
    for i in 0..NCHN {
      for j in 0..n_sp[i] as usize {
        //n_symmetric = 0;
        n_neighbor = 0;
        for k in 0..NCHN {
          for l in 0..n_sp[k] as usize {
          //check if this spike has a symmetric partner in any channel
            if (sp[i][j] as i32 + sp[k][l] as i32 - 2 * stop_cell as i32) as i32 % NWORDS as i32 == 1022 {
              //n_symmetric += 1;
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
    let magic_value : f32 = 14.8;
    let mut x       : f32;
    let mut y       : f32;

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
          x = voltages[i][(j - 1) % NWORDS];
          y = voltages[i][(j + 4) % NWORDS];
          if f32::abs(x - y) < 15.0 {
            voltages[i][j % NWORDS] = x + 1.0 * (y - x) / 5.0;
            voltages[i][(j + 1) % NWORDS] = x + 2.0 * (y - x) / 5.0;
            voltages[i][(j + 2) % NWORDS] = x + 3.0 * (y - x) / 5.0;
            voltages[i][(j + 3) % NWORDS] = x + 4.0 * (y - x) / 5.0;
          } else {
            voltages[i][j % NWORDS] -= magic_value;
            voltages[i][(j + 1) % NWORDS] -= magic_value;
            voltages[i][(j + 2) % NWORDS] -= magic_value;
            voltages[i][(j + 3) % NWORDS] -= magic_value;
          }
        } else {
          // remove single spike 
          x = voltages[i][((rsp[k] - 1) % NWORDS as i32) as usize];
          y = voltages[i][(rsp[k] + 2) as usize % NWORDS];
          if f32::abs(x - y) < 15.0 {
            voltages[i][rsp[k] as usize] = x + 1.0 * (y - x) / 3.0;
            voltages[i][(rsp[k] + 1) as usize % NWORDS] = x + 2.0 * (y - x) / 3.0;
          } else {
            voltages[i][rsp[k] as usize] -= magic_value;
            voltages[i][(rsp[k] + 1) as usize % NWORDS] -= magic_value;
          }
        } // end loop over nchn
      } // end loop over n_rsp
      if k < n_rsp && i32::abs(rsp[k] - rsp[k + 1] % NWORDS as i32) == 2
        {skip_next = true;} // skip second half of double spike
    } // end loop over k
  Ok(())
  }

  /// Apply the voltage calibration to a single channel 
  ///
  /// # Arguments
  ///
  /// * channel   : Channel id 1-9
  /// * stop_cell : This channels stop cell 
  /// * adc       : Uncalibrated channel data
  /// * waveform  : Pre-allocated array to hold 
  ///               calibrated waveform data.
  pub fn voltages(&self,
                  channel   : usize,
                  stop_cell : usize,
                  adc       : &Vec<u16>,
                  waveform  : &mut Vec<f32>) {
                  //waveform  : &mut [f32;NWORDS]) {
    if channel > 9 || channel == 0 {
      error!("There is no channel larger than 9 and no channel 0! Channel {channel} was requested. Can not perform voltage calibration!");
      return;
    }

    let mut value : f32; 
    for k in 0..NWORDS {
      value  = adc[k] as f32;
      value -= self.v_offsets[channel -1][(k + (stop_cell)) %NWORDS];
      value -= self.v_dips   [channel -1][k];
      value *= self.v_inc    [channel -1][(k + (stop_cell)) %NWORDS];
      waveform[k] = value;
    }
  }
  
  /// Apply the timing calibration to a single channel 
  /// 
  /// This will allocate the array for the waveform 
  /// time bins (unit is ns)
  ///
  /// # Arguments
  ///
  /// * channel   : Channel id 1-9
  /// * stop_cell : This channels stop cell 
  pub fn nanoseconds(&self,
                     channel   : usize,
                     stop_cell : usize,
                     times     : &mut Vec<f32>)
    {
    if channel > 9 || channel == 0 {
      error!("There is no channel larger than 9 and no channel 0! Channel {channel} was requested. Can not perform timing calibration!");
    }
    for k in 1..NWORDS { 
      times[k] = times[k-1] + self.tbin[channel -1][(k-1+(stop_cell))%NWORDS];
    }
  }

  pub fn new(rb_id : u8) -> Self {
    Self {
      rb_id     : rb_id,
      d_v       : 182.0, // FIXME - this needs to be a constant
      serialize_event_data : false, // per default, don't serialize the data 
      v_offsets : [[0.0;NWORDS];NCHN], 
      v_dips    : [[0.0;NWORDS];NCHN], 
      v_inc     : [[0.0;NWORDS];NCHN], 
      tbin      : [[0.0;NWORDS];NCHN],
      vcal_data : Vec::<RBEvent>::new(),
      tcal_data : Vec::<RBEvent>::new(),
      noi_data  : Vec::<RBEvent>::new()
    }
  }

  ///
  ///
  pub fn from_txtfile(filename : &Path) -> Self {
    let mut rb_cal = Self::new(0);
    rb_cal.get_id_from_filename(&filename);
    debug!("Attempting to open file {}", filename.display());
    let file = BufReader::new(File::open(filename).expect("Unable to open file {}"));
    // count lines and check if we have 4 lines per channel
    let mut cnt  = 0;
    for _ in file.lines() {
      cnt += 1;
    }
    if cnt != NCHN*4 {
      panic! ("The calibration file {} does not have the proper format! It has {} lines", filename.display(), cnt);
    }
    cnt = 0;
    let mut vals = 0usize;

    if let Ok(lines) = read_lines(filename) {
      // we have NCHN-1*4 lines (no calibration data for channel 9)
      for data in lines.flatten() {       
        let values: Vec<&str> = data.split(' ').collect();
        if values.len() == NWORDS {
          if vals == 0 {
            for n in 0..NWORDS {
              // this will throw an error if calibration data 
              // is not following conventioss
              // TODO it will actually panic!!!
              let data : f32 = values[n].parse::<f32>().unwrap();
              rb_cal.v_offsets[cnt][n] = data;
              //cals[cnt].v_offsets[n] = data;
            }
            vals += 1;
            continue;
          }
          if vals == 1 {
            for n in 0..NWORDS {
              // this will throw an error if calibration data 
              // is not following conventioss
              // TODO it will actually panic!!!
              let data : f32 = values[n].parse::<f32>().unwrap();
              rb_cal.v_dips[cnt][n] = data;
              //cals[cnt].v_dips[n] = data;
            }
            vals += 1;
            continue;
          }
          if vals == 2 {
            for n in 0..NWORDS {
              // this will throw an error if calibration data 
              // is not following conventioss
              // TODO it will actually panic!!!
              let data : f32 = values[n].parse::<f32>().unwrap();
              rb_cal.v_inc[cnt][n] = data;
              //cals[cnt].v_inc[n] = data;
            }
            vals += 1;
            continue;
          }
          if vals == 3 {
            for n in 0..NWORDS {
              // this will throw an error if calibration data 
              // is not following conventioss
              // TODO it will actually panic!!!
              let data : f32 = values[n].parse::<f32>().unwrap();
              rb_cal.tbin[cnt][n] = data;
              //cals[cnt].tbin[n] = data;
              // reset vals & cnts
            }
            vals = 0;
            cnt += 1;
            continue;
          }
        } else {
          panic!("Invalid input line {}", data)
        }
        vals += 1;
      }
    }
    rb_cal
  }

  /// Infer the readoutboard id from the filename
  ///
  /// Assuming a certain naming scheme for the filename "rbXX_cal.txt"
  /// we extract the readoutboard id
  pub fn get_id_from_filename(&mut self, filename : &Path) -> u8 {
    let rb_id : u8;
    match filename.file_name() {
      None   => {
        error!("Path {} seems non-sensical!", filename.display());
        self.rb_id = 0;
        return 0;
      }
      Some(name) => {
        // TODO This might panic! Is it ok?
        let fname = name.to_os_string().into_string().unwrap();
        let id    = &fname[2..4];
        // TODO This might panic! Is it ok?
        rb_id     = id.parse::<u8>().unwrap();
        debug!("Extracted RB ID {} from filename {}", rb_id, fname);
      }
    }
  self.rb_id = rb_id;
  rb_id
  }
}

impl Serialization for RBCalibrations {
  const SIZE            : usize = NCHN*NWORDS*4*8 + 4 + 1 + 1; 
  const HEAD            : u16   = 0xAAAA; // 43690 
  const TAIL            : u16   = 0x5555; // 21845 
  
  fn from_bytestream(bytestream : &Vec<u8>, 
                     pos        : &mut usize)
    -> Result<Self, SerializationError> { 
    let mut rb_cal = Self::new(0);
    if parse_u16(bytestream, pos) != Self::HEAD {
      return Err(SerializationError::HeadInvalid {});
    }
    let board_id = u8::from_le_bytes([bytestream[*pos]]);
    *pos += 1;
    rb_cal.rb_id = board_id;
    rb_cal.d_v   = parse_f32(bytestream, pos);
    rb_cal.serialize_event_data = parse_bool(bytestream, pos);
    for ch in 0..NCHN {
      for k in 0..NWORDS {
        let mut value = parse_f32(bytestream, pos);
        rb_cal.v_offsets[ch][k] = value;
        value         = parse_f32(bytestream, pos);
        rb_cal.v_dips[ch][k]    = value;
        value         = parse_f32(bytestream, pos);
        rb_cal.v_inc[ch][k]     = value;
        value         = parse_f32(bytestream, pos);
        rb_cal.tbin[ch][k]      = value;
      }
    }
    if rb_cal.serialize_event_data {
      let mut broken_event = RBEvent::new();
      broken_event.header.broken = true;
      let n_noi  = parse_u16(bytestream, pos);
      println!("Found {n_noi} no input data events!");
      for _ in 0..n_noi {
        //for k in 0..10 {
        //  println!("bytestream {k} {} ", bytestream[*pos+k]);
        //}
        match RBEvent::from_bytestream(bytestream, pos) {
          Ok(ev) => {
            //println!("good");
            rb_cal.noi_data.push(ev);            
          }
          Err(err) => {
            //println!("from_bytestream failed!, err {err}");
            rb_cal.noi_data.push(broken_event.clone());
          }
        }
        // FIXME - broken event won't advance the pos marker
      }
      let n_vcal = parse_u16(bytestream, pos); 
      println!("Found {n_vcal} VCal data events!");
      for _ in 0..n_vcal {
        rb_cal.vcal_data.push(RBEvent::from_bytestream(bytestream, pos).unwrap_or(broken_event.clone()));
      }
      let n_tcal = parse_u16(bytestream, pos); 
      println!("Found {n_tcal} TCal data events!");
      for _ in 0..n_tcal {
        rb_cal.tcal_data.push(RBEvent::from_bytestream(bytestream, pos).unwrap_or(broken_event.clone()));
      }
    }
    if parse_u16(bytestream, pos) != Self::TAIL {
      return Err(SerializationError::TailInvalid {});
    }
    Ok(rb_cal)
  }

  fn to_bytestream(&self) -> Vec<u8> {
    let mut bs = Vec::<u8>::with_capacity(Self::SIZE);
    bs.extend_from_slice(&Self::HEAD.to_le_bytes());
    bs.extend_from_slice(&self.rb_id.to_le_bytes());
    bs.extend_from_slice(&self.d_v.to_le_bytes());
    let serialize_event_data = self.serialize_event_data as u8;
    bs.push(serialize_event_data);
    for ch in 0..NCHN {
      for k in 0..NWORDS {
        bs.extend_from_slice(&self.v_offsets[ch][k].to_le_bytes());
        bs.extend_from_slice(&self.v_dips[ch][k]   .to_le_bytes());
        bs.extend_from_slice(&self.v_inc[ch][k]    .to_le_bytes());
        bs.extend_from_slice(&self.tbin[ch][k]     .to_le_bytes());
      }
    }
    if self.serialize_event_data {
      println!("Serializing calibration event data!");
      let n_noi  = self.noi_data.len()  as u16;
      let n_vcal = self.vcal_data.len() as u16;
      let n_tcal = self.tcal_data.len() as u16;
      bs.extend_from_slice(&n_noi.to_le_bytes());
      for ev in &self.noi_data {
        bs.extend_from_slice(&ev.to_bytestream());
      }
      bs.extend_from_slice(&n_vcal.to_le_bytes());
      for ev in &self.vcal_data {
        bs.extend_from_slice(&ev.to_bytestream());
      }
      bs.extend_from_slice(&n_tcal.to_le_bytes());
      for ev in &self.tcal_data {
        bs.extend_from_slice(&ev.to_bytestream());
      }
    }
    bs.extend_from_slice(&Self::TAIL.to_le_bytes());
    bs
  }
}

impl Default for RBCalibrations {
  fn default() -> Self {
    Self::new(0)
  }
}

impl fmt::Display for RBCalibrations {
  fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
    write!(f, 
  "<ReadoutboardCalibration:
      RB : {}
      VCalData    : {} (events)
      TCalData    : {} (events)
      NoInputData : {} (events)
      V Offsets [ch0]: .. {:?} {:?} ..
      V Incrmts [ch0]: .. {:?} {:?} ..
      V Dips    [ch0]: .. {:?} {:?} ..
      T Bins    [ch0]: .. {:?} {:?} ..>",
      self.rb_id,
      self.vcal_data.len(),
      self.tcal_data.len(),
      self.noi_data.len(),
      self.v_offsets[0][98],
      self.v_offsets[0][99],
      self.v_inc[0][98],
      self.v_inc[0][99],
      self.v_dips[0][98],
      self.v_dips[0][99],
      self.tbin[0][98],
      self.tbin[0][99])
  } 
}

#[cfg(feature = "random")]
impl FromRandom for RBCalibrations {
    
  fn from_random() -> Self {
    let mut cali   = Self::new(0);
    let mut rng    = rand::thread_rng();
    cali.rb_id     = rng.gen::<u8>();
    cali.d_v       = rng.gen::<f32>(); 
    cali.serialize_event_data = rng.gen::<bool>();
    for ch in 0..NCHN {
      for n in 0..NWORDS { 
        cali.v_offsets[ch][n] = rng.gen::<f32>();
        cali.v_dips   [ch][n] = rng.gen::<f32>(); 
        cali.v_inc    [ch][n] = rng.gen::<f32>(); 
        cali.tbin     [ch][n] = rng.gen::<f32>();
      }
    }
    if cali.serialize_event_data {
      for _ in 0..1000 {
        let mut ev = RBEvent::from_random();
        cali.vcal_data.push(ev);
        ev = RBEvent::from_random();
        cali.noi_data.push(ev);
        ev = RBEvent::from_random();
        cali.tcal_data.push(ev);
      }
    }
    cali
  }
}

impl From<&Path> for RBCalibrations {
  
  /// Read an asci text file with calibration constants.
  fn from(path : &Path) -> Self {
    let mut cali = RBCalibrations::new(0);
    if path.ends_with(".txt") {
      return RBCalibrations::from_txtfile(path);
    } else {
      match read_file(path) {
        Err(err) => {
          error!("Can not open {}! Err {err}", path.display());
        },
        Ok(stream) => {
          // assume this is wrapped in a tof packet
          match TofPacket::from_bytestream(&stream, &mut 0) {
            Err(err) => {
              error!("Can not read TofPacket, error {err}");
            }
            Ok(pk) => {
              if pk.packet_type != PacketType::RBCalibration {
                error!("TofPacket does not contain calibration data! Packet: {}", pk);
                return cali;
              }
              match RBCalibrations::from_bytestream(&pk.payload, &mut 0) {
                Err(err) => {
                  error!("Can not read calibration from binary file {}, Error {err}!", path.display());
                },
                Ok(c) => {
                  cali = c;
                }
              }
            }
          }
        }
      }
      return cali; 
    }
  }
}

#[cfg(feature = "random")]
#[test]
fn serialization_rbcalibration() {
  let mut calis = Vec::<RBCalibrations>::new();
  for n in 0..100 {
    let cali = RBCalibrations::from_random();
    if !cali.serialize_event_data {
      continue;
    }
    calis.push(cali);
    break;
  }
  let test = RBCalibrations::from_bytestream(&calis[0].to_bytestream(), &mut 0).unwrap();
  assert_eq!(calis[0], test);
}

// This test together with the 
// roll function comes directly
// from ChatGPT
// FIXME - currently it does not 
// work
//#[test]
//fn roll_with_random_vectors() {
//    const NUM_TESTS: usize = 1000;
//    const VECTOR_SIZE: usize = 10;
//
//    let mut rng = thread_rng();
//
//    for _ in 0..NUM_TESTS {
//        let mut original_vec: Vec<i32> = (0..VECTOR_SIZE).collect();
//        let shift = rng.gen_range(-VECTOR_SIZE as isize, VECTOR_SIZE as isize);
//
//        let mut rolled_vec = original_vec.clone();
//        roll(&mut rolled_vec, shift);
//
//        // Verify that the original and rolled vectors have the same elements
//        // when considering rotation (modulo VECTOR_SIZE).
//        for i in 0..VECTOR_SIZE {
//            let original_idx = (i as isize - shift + VECTOR_SIZE as isize) % VECTOR_SIZE as isize;
//            let expected_element = original_vec[original_idx as usize];
//            assert_eq!(rolled_vec[i], expected_element);
//        }
//    }
//}
