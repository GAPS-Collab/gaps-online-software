//! Work with calibration files
//!
//! Read out calibration files.
//! 
//! The `Calibration` class then 
//! holds the results for a single 
//! channel
//!
//!
//!
//!
//!
//!

use std::fs::File;
use std::io::{self, BufRead, BufReader};
use std::path::Path;
use std::fmt;

use crate::constants::{NWORDS, NCHN};

use crate::serialization::{Serialization,
                           parse_u16,
                           parse_f32,
                           SerializationError};

/***********************************/

// helper
fn read_lines<P>(filename: P) -> io::Result<io::Lines<io::BufReader<File>>>
where P: AsRef<Path>, {
    let file = File::open(filename)?;
    Ok(io::BufReader::new(file).lines())
}

/***********************************/

#[derive(Copy, Clone, Debug)]
pub struct ReadoutBoardCalibrations {
  pub v_offsets : [[f32;NWORDS];NCHN], // voltage offset
  pub v_dips    : [[f32;NWORDS];NCHN], // voltage "dip" (time-dependent correction)
  pub v_inc     : [[f32;NWORDS];NCHN], // voltage increment (mV/ADC unit)
  pub tbin      : [[f32;NWORDS];NCHN], // cell width (ns)
  pub rb_id     : u8
}

impl ReadoutBoardCalibrations {

  /// Apply the spike cleaning to all channels
  pub fn spike_cleaning(_adcs : &mut Vec<Vec<f32>> ) {

  ////let mut spikes  : [i32;10] = [0;10];
  //let mut filter      : f64;
  //let mut dfilter     : f64;
  //let mut n_neighbor  : usize;
  //let mut n_rsp      = 0usize;
  //let mut rsp : [i32;10]    = [-1;10];
  ////let mut spikes : [i32;10] = [-1;10
  //// to me, this seems that should be u32
  //// the 10 is for a maximum of 10 spikes (Jeff)
  //let mut sp   : [[usize;10];NCHN] = [[0;10];NCHN];
  //let mut n_sp : [usize;10]        = [0;10];

  //for j in 0..NWORDS as usize {
  //  for i in 0..NCHN as usize {
  //    filter = -self.voltages[i][j] + self.voltages[i][(j + 1) % NWORDS] + self.voltages[i][(j + 2) % NWORDS] - self.voltages[i][(j + 3) % NWORDS];
  //    dfilter = filter + 2.0 * self.voltages[i][(j + 3) % NWORDS] + self.voltages[i][(j + 4) % NWORDS] - self.voltages[i][(j + 5) % NWORDS];
  //    if filter > 20.0  && filter < 100.0 {
  //      if n_sp[i] < 10 {   // record maximum of 10 spikes
  //        sp[i][n_sp[i] as usize] = (j + 1) % NWORDS ;
  //        n_sp[i] += 1;
  //      // FIXME - error checking
  //      } else {return;}            // too many spikes -> something wrong
  //    }// end of if
  //    else if dfilter > 40.0 && dfilter < 100.0 && filter > 10.0 {
  //      if n_sp[i] < 9 {  // record maximum of 10 spikes
  //        sp[i][n_sp[i] as usize] = (j + 1) % NWORDS ;
  //        sp[i][(n_sp[i] + 1) as usize] = (j + 3) % NWORDS ;
  //        n_sp[i] += 2;
  //      } else { return;} // too many spikes -> something wrong
  //    } // end of else if

  //  }// end loop over NCHN
  //} // end loop over NWORDS

  //// go through all spikes and look for neighbors */
  //for i in 0..NCHN {
  //  for j in 0..n_sp[i] as usize {
  //    //n_symmetric = 0;
  //    n_neighbor = 0;
  //    for k in 0..NCHN {
  //      for l in 0..n_sp[k] as usize {
  //      //check if this spike has a symmetric partner in any channel
  //        if (sp[i][j] as i32 + sp[k][l] as i32 - 2 * self.stop_cell as i32) as i32 % NWORDS as i32 == 1022 {
  //          //n_symmetric += 1;
  //          break;
  //        }
  //      }
  //    } // end loop over k
  //    // check if this spike has same spike is in any other channels */
  //    //for (k = 0; k < nChn; k++) {
  //    for k in 0..NCHN {
  //      if i != k {
  //        for l in 0..n_sp[k] {
  //          if sp[i][j] == sp[k][l] {
  //          n_neighbor += 1;
  //          break;
  //          }
  //        } // end loop over l   
  //      } // end if
  //    } // end loop over k

  //    if n_neighbor >= 2 {
  //      for k in 0..n_rsp {
  //        if rsp[k] == sp[i][j] as i32 {break;} // ignore repeats
  //        if n_rsp < 10 && k == n_rsp {
  //          rsp[n_rsp] = sp[i][j] as i32;
  //          n_rsp += 1;
  //        }
  //      }  
  //    }

  //  } // end loop over j
  //} // end loop over i

  //// recognize spikes if at least one channel has it */
  ////for (k = 0; k < n_rsp; k++)
  //let magic_value : f64 = 14.8;
  //let mut x : f64;
  //let mut y : f64;

  //let mut skip_next : bool = false;
  //for k in 0..n_rsp {
  //  if skip_next {
  //    skip_next = false;
  //    continue;
  //  }
  //  spikes[k] = rsp[k];
  //  //for (i = 0; i < nChn; i++)
  //  for i in 0..NCHN {
  //    if k < n_rsp && i32::abs(rsp[k] as i32 - rsp[k + 1] as i32 % NWORDS as i32) == 2
  //    {
  //      // remove double spike 
  //      let j = if rsp[k] > rsp[k + 1] {rsp[k + 1] as usize}  else {rsp[k] as usize};
  //      x = self.voltages[i][(j - 1) % NWORDS];
  //      y = self.voltages[i][(j + 4) % NWORDS];
  //      if f64::abs(x - y) < 15.0
  //      {
  //        self.voltages[i][j % NWORDS] = x + 1.0 * (y - x) / 5.0;
  //        self.voltages[i][(j + 1) % NWORDS] = x + 2.0 * (y - x) / 5.0;
  //        self.voltages[i][(j + 2) % NWORDS] = x + 3.0 * (y - x) / 5.0;
  //        self.voltages[i][(j + 3) % NWORDS] = x + 4.0 * (y - x) / 5.0;
  //      }
  //      else
  //      {
  //        self.voltages[i][j % NWORDS] -= magic_value;
  //        self.voltages[i][(j + 1) % NWORDS] -= magic_value;
  //        self.voltages[i][(j + 2) % NWORDS] -= magic_value;
  //        self.voltages[i][(j + 3) % NWORDS] -= magic_value;
  //      }
  //    }
  //    else
  //    {
  //      // remove single spike 
  //      x = self.voltages[i][((rsp[k] - 1) % NWORDS as i32) as usize];
  //      y = self.voltages[i][(rsp[k] + 2) as usize % NWORDS];
  //      if f64::abs(x - y) < 15.0 {
  //        self.voltages[i][rsp[k] as usize] = x + 1.0 * (y - x) / 3.0;
  //        self.voltages[i][(rsp[k] + 1) as usize % NWORDS] = x + 2.0 * (y - x) / 3.0;
  //      }
  //      else
  //      {
  //        self.voltages[i][rsp[k] as usize] -= magic_value;
  //        self.voltages[i][(rsp[k] + 1) as usize % NWORDS] -= magic_value;
  //      }
  //    } // end loop over nchn
  //  } // end loop over n_rsp
  //  if k < n_rsp && i32::abs(rsp[k] - rsp[k + 1] % NWORDS as i32) == 2
  //    {skip_next = true;} // skip second half of double spike
  //} // end loop over k
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
                  adc       : Vec<u16>,
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

  pub fn new(rb_id : u8) -> ReadoutBoardCalibrations {
    ReadoutBoardCalibrations {
      v_offsets : [[0.0;NWORDS];NCHN], 
      v_dips    : [[0.0;NWORDS];NCHN], 
      v_inc     : [[0.0;NWORDS];NCHN], 
      tbin      : [[0.0;NWORDS];NCHN],
      rb_id     : rb_id,
    }
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
        let fname = name.to_os_string().into_string().unwrap();
        let id    = &fname[2..4];
        rb_id     = id.parse::<u8>().unwrap();
        debug!("Extracted RB ID {} from filename {}", rb_id, fname);
      }
    }
  self.rb_id = rb_id;
  rb_id
  }
}

impl Serialization for ReadoutBoardCalibrations {
  const SIZE            : usize = NCHN*NWORDS*4*8 + 4 + 1; 
  const HEAD            : u16   = 0xAAAA; // 43690 
  const TAIL            : u16   = 0x5555; // 21845 
  
  /// Decode a serializable from a bytestream  
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
    if parse_u16(bytestream, pos) != ReadoutBoardCalibrations::TAIL {
      return Err(SerializationError::TailInvalid {});
    }
    Ok(rb_cal)
  }

  fn to_bytestream(&self) -> Vec<u8> {
    let mut bs = Vec::<u8>::with_capacity(Self::SIZE);
    bs.extend_from_slice(&Self::HEAD.to_le_bytes());
    bs.extend_from_slice(&self.rb_id.to_le_bytes());
    for ch in 0..NCHN {
      for k in 0..NWORDS {
        bs.extend_from_slice(&self.v_offsets[ch][k].to_le_bytes());
        bs.extend_from_slice(&self.v_dips[ch][k]   .to_le_bytes());
        bs.extend_from_slice(&self.v_inc[ch][k]    .to_le_bytes());
        bs.extend_from_slice(&self.tbin[ch][k]     .to_le_bytes());
      }
    }
    bs
  }
}

impl Default for ReadoutBoardCalibrations {
  fn default() -> ReadoutBoardCalibrations {
    ReadoutBoardCalibrations::new(0)
  }
}

impl fmt::Display for ReadoutBoardCalibrations {
  fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
    write!(f, "<ReadoutboardCalibrations: RB {}>", self.rb_id)
  } 
}

impl From<&Path> for ReadoutBoardCalibrations {
  
  /// Read an asci text file with calibration constants.
  fn from(path : &Path) -> ReadoutBoardCalibrations {
    let mut rb_cal = ReadoutBoardCalibrations::new(0);
    rb_cal.get_id_from_filename(&path);
    debug!("Attempting to open file {}", path.display());
    let file = BufReader::new(File::open(path).expect("Unable to open file {}"));
    // count lines and check if we have 4 lines per channel
    let mut cnt  = 0;
    for _ in file.lines() {
      cnt += 1;
    }
    if cnt != NCHN*4 {panic! ("The calibration file {} does not have the proper format! It has {} lines", path.display(), cnt);}
    cnt = 0;
    let mut vals = 0usize;

    if let Ok(lines) = read_lines(path) {
      // we have NCHN-1*4 lines (no calibration data for channel 9)
      for line in lines {
        if let Ok(data) = line {        
          let values: Vec<&str> = data.split(' ').collect();
          match values.len() {
            NWORDS => {
              if vals == 0 {
                for n in 0..NWORDS {
                  // this will throw an error if calibration data 
                  // is not following conventioss
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
                  let data : f32 = values[n].parse::<f32>().unwrap();
                  rb_cal.tbin[cnt][n] = data;
                  //cals[cnt].tbin[n] = data;
                  // reset vals & cnts
                }
                vals = 0;
                cnt += 1;
                continue;
              }
            },
            _ => panic!("Invalid input line {}", data),
          }; // end Ok lines
          vals += 1;
        }
      }
    }
    rb_cal
  }
}


