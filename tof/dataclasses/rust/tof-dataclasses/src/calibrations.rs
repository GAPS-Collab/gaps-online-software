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
  //NCHAN*NWORDS*4 + 1 

  /// Apply the voltage calibration to a single channel 
  ///
  /// # Arguments
  ///
  /// * channel   : Channel id 1-9
  /// * stop_cell : This channels stop cell 
  /// * adc       : Uncalibrated channel data
  /// * waveform  : Pre-allocated array to hold 
  ///               calibrated waveform data.
  pub fn apply_vcal_ch(&self,
                       channel   : usize,
                       stop_cell : usize,
                       adc       : &[u16;NWORDS],
                       waveform  : &mut [f32;NWORDS]) {
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
  pub fn apply_tcal_ch(&self,
                       channel   : usize,
                       stop_cell : usize)
    -> [f32;NWORDS] {
    
    // allocate the timing array
    let mut times : [f32;NWORDS] = [0.0;NWORDS];
    
    if channel > 9 || channel == 0 {
      error!("There is no channel larger than 9 and no channel 0! Channel {channel} was requested. Can not perform voltage calibration!");
      return times;
    }
    

    for k in 1..NWORDS { 
      times[k] = times[k-1] + self.tbin[channel -1][(k-1+(stop_cell))%NWORDS];
    }

    times
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

  pub fn to_bytestream(&self) -> Vec<u8> {
    let mut bs = Vec::<u8>::with_capacity(ReadoutBoardCalibrations::SIZE);
    bs.extend_from_slice(&ReadoutBoardCalibrations::HEAD.to_le_bytes());
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


