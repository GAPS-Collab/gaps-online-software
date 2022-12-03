/***********************************/

use std::fs::File;
use std::io::{self, BufRead, BufReader};
//use std::io::{self, BufRead};
use std::path::Path;

use crate::constants::{NWORDS, NCHN};

/***********************************/

// helper
fn read_lines<P>(filename: P) -> io::Result<io::Lines<io::BufReader<File>>>
where P: AsRef<Path>, {
    let file = File::open(filename)?;
    Ok(io::BufReader::new(file).lines())
}

/***********************************/

#[derive(Copy, Clone)]
pub struct Calibrations
{
  pub v_offsets : [f64;NWORDS], // voltage offset
  pub v_dips    : [f64;NWORDS], // voltage "dip" (time-dependent correction)
  pub v_inc     : [f64;NWORDS], // voltage increment (mV/ADC unit)
  pub tbin      : [f64;NWORDS] // cell width (ns)
}

impl Default for Calibrations {
    fn default() -> Calibrations {
        Calibrations {
           v_offsets : [0.0;NWORDS],
           v_dips    : [0.0;NWORDS],
           v_inc     : [0.0;NWORDS],
           tbin      : [0.0;NWORDS]
        }
    }
}

/***********************************/

pub fn voltage_calibration(trace_in    : &[i16;NWORDS],
                           trace_out   : &mut [f64;NWORDS],
                           t_cell      : u16,
                           cal         : &Calibrations)
{
  let mut value : f64; 
  
  for n in 0..NWORDS {
    value = trace_in[n] as f64;
    value -= cal.v_offsets[(n + (t_cell as usize)) %NWORDS];
    value -= cal.v_dips[n];
    value *= cal.v_inc[(n + (t_cell as usize)) %NWORDS];
    trace_out[n] = value;
  }

}

/***********************************/

pub fn timing_calibration(times    : &mut [f64;NWORDS],
                          t_cell   : u16,
                          cal      : &Calibrations)
{
    times[0] = 0.0;
    for n in 0..NWORDS {
      times[n] = times[n-1] + cal.tbin[(n-1+(t_cell as usize))%1024];
    }
}

/***********************************/

pub fn read_calibration_file(filename : &Path) -> [Calibrations; NCHN ]
{
    let mut cals = [Calibrations {..Default::default()}; NCHN];
    for n in 0..NCHN {
        cals[n] = Calibrations {..Default::default()};
    }

    //let mut cal =  Calibrations {..Default::default()};
    //first we count the lines to see if the file is sane
    let file = BufReader::new(File::open(filename).expect("Unable to open file"));
    let mut cnt  = 0;

    for _ in file.lines() {
        cnt += 1;
    }
    
    if cnt != NCHN*4 {panic! ("The calibration file {} does not have the proper format! It has {} lines", filename.display(), cnt);}

    cnt = 0;
    let mut vals = 0usize;

    if let Ok(lines) = read_lines(filename) {
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
                       let data : f64 = values[n].parse::<f64>().unwrap();
                       cals[cnt].v_offsets[n] = data;
                      }
                  vals += 1;
                  continue;
                  }
                  if vals == 1 {
                   for n in 0..NWORDS {
                       // this will throw an error if calibration data 
                       // is not following conventioss
                       let data : f64 = values[n].parse::<f64>().unwrap();
                       cals[cnt].v_dips[n] = data;
                      }
                  vals += 1;
                  continue;
                  }
                  if vals == 2 {
                   for n in 0..NWORDS {
                       // this will throw an error if calibration data 
                       // is not following conventioss
                       let data : f64 = values[n].parse::<f64>().unwrap();
                       cals[cnt].v_inc[n] = data;
                      }
                  vals += 1;
                  continue;
                  }
                  if vals == 3 {
                   for n in 0..NWORDS {
                       // this will throw an error if calibration data 
                       // is not following conventioss
                       let data : f64 = values[n].parse::<f64>().unwrap();
                       cals[cnt].tbin[n] = data;
                       // reset vals & cnts
                      }
                  vals = 0;
                  cnt += 1;
                  //println!("counter {} vals {}", cnt, vals);
                  continue;
                  }
                  //let strdata = values[0].parse::<String>();
                  //let intdata =  values[1].parse::<i32>();
                  //println!("Got: {:?} {:?}", strdata, intdata);
                },
                _ => panic!("Invalid input line {}", data),
              }; // end Ok lines
              vals += 1;
          }
        }
    }
    return cals;
}

/***********************************/

pub fn remove_spikes (waveform : &mut [[f64;NWORDS];NCHN],
                      t_cell : usize,
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
  let mut sp   : [[usize;NCHN];10] = [[0;NCHN];10];
  let mut n_sp : [usize;NCHN]      = [0;NCHN];

  for j in 0..NWORDS as usize {
    for i in 0..NCHN as usize {
      filter = -waveform[i][j] + waveform[i][(j + 1) % NWORDS] + waveform[i][(j + 2) % NWORDS] - waveform[i][(j + 3) % NWORDS];
      dfilter = filter + 2.0 * waveform[i][(j + 3) % NWORDS] + waveform[i][(j + 4) % NWORDS] - waveform[i][(j + 5) % NWORDS];
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
          if (sp[i][j] + sp[k][l] - 2 * t_cell) as usize % NWORDS == 1022 {
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

  for mut k in 0..n_rsp {
    spikes[k] = rsp[k];
    //for (i = 0; i < nChn; i++)
    for i in 0..NCHN {
      if (k < n_rsp && i32::abs(rsp[k] as i32 - rsp[k + 1] as i32 % NWORDS as i32) == 2)
      {
        // remove double spike 
        let j = if rsp[k] > rsp[k + 1] {rsp[k + 1] as usize}  else {rsp[k] as usize};
        x = waveform[i][(j - 1) % NWORDS];
        y = waveform[i][(j + 4) % NWORDS];
        if f64::abs(x - y) < 15.0
        {
          waveform[i][j % NWORDS] = x + 1.0 * (y - x) / 5.0;
          waveform[i][(j + 1) % NWORDS] = x + 2.0 * (y - x) / 5.0;
          waveform[i][(j + 2) % NWORDS] = x + 3.0 * (y - x) / 5.0;
          waveform[i][(j + 3) % NWORDS] = x + 4.0 * (y - x) / 5.0;
        }
        else
        {
          waveform[i][j % NWORDS] -= magic_value;
          waveform[i][(j + 1) % NWORDS] -= magic_value;
          waveform[i][(j + 2) % NWORDS] -= magic_value;
          waveform[i][(j + 3) % NWORDS] -= magic_value;
        }
      }
      else
      {
        // remove single spike 
        x = waveform[i][((rsp[k] - 1) % NWORDS as i32) as usize];
        y = waveform[i][(rsp[k] + 2) as usize % NWORDS];
        if f64::abs(x - y) < 15.0 {
          waveform[i][rsp[k] as usize] = x + 1.0 * (y - x) / 3.0;
          waveform[i][(rsp[k] + 1) as usize % NWORDS] = x + 2.0 * (y - x) / 3.0;
        }
        else
        {
          waveform[i][rsp[k] as usize] -= magic_value;
          waveform[i][(rsp[k] + 1) as usize % NWORDS] -= magic_value;
        }
      } // end loop over nchn
    } // end loop over n_rsp
    if (k < n_rsp && i32::abs(rsp[k] - rsp[k + 1] % NWORDS as i32) == 2)
      {k += 1} // skip second half of double spike
  } // end loop over k
}


/***********************************/


