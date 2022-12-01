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
        // we have NCHAN-1*4 lines (no calibration data for channel 9)
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




