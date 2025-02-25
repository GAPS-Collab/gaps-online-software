pub mod master_trigger;
pub mod settings;
pub mod constants;
pub mod thread_control;
pub mod sine_fitter;

use constants::{
    DEFAULT_LTB_ID,
};

use std::thread;
use std::time::Duration;
use std::os::raw::c_int;
use std::sync::{
    Arc,
    Mutex,
};

use chrono::Utc;

#[cfg(feature="database")]
use core::f32::consts::PI;

#[cfg(feature="database")]
use half::f16;

pub use master_trigger::{
    master_trigger,
    MTBSettings,
};

pub use settings::{
    LiftofSettings,
    AnalysisEngineSettings,
};

use std::fmt;

use std::path::PathBuf;
use std::fs::read_to_string;
use std::io::{
    Write,
};

use std::collections::HashMap;
use colored::{
    Colorize,
    ColoredString
};

use serde_json::Value;

use log::Level;

#[macro_use] extern crate log;
extern crate env_logger;

use signal_hook::iterator::Signals;
use signal_hook::consts::signal::{
  SIGTERM,
  SIGINT
};

use tof_dataclasses::DsiLtbRBMapping;
#[cfg(feature="database")]
use tof_dataclasses::database::ReadoutBoard;

#[cfg(feature="database")]
use tof_dataclasses::constants::NWORDS;
#[cfg(feature="database")]
use tof_dataclasses::errors::AnalysisError;
use tof_dataclasses::errors::SetError;
#[cfg(feature="database")]
use tof_dataclasses::events::{
  RBEvent,
  TofHit,
};

#[cfg(feature="database")]
use tof_dataclasses::analysis::{
  calculate_pedestal,
  integrate,
  cfd_simple,
  find_peaks,
};

use tof_dataclasses::RBChannelPaddleEndIDMap;

use crate::thread_control::ThreadControl;

use clap::{arg,
  Args,
};

pub const MT_MAX_PACKSIZE   : usize = 512;
pub const DATAPORT          : u32   = 42000;
pub const ASSET_DIR         : &str  = "/home/gaps/assets/"; 
pub const LIFTOF_LOGO_SHOW  : &str  = "
                                  ___                         ___           ___     
                                 /\\__\\                       /\\  \\         /\\__\\    
                    ___         /:/ _/_         ___         /::\\  \\       /:/ _/_   
                   /\\__\\       /:/ /\\__\\       /\\__\\       /:/\\:\\  \\     /:/ /\\__\\  
    ___     ___   /:/__/      /:/ /:/  /      /:/  /      /:/  \\:\\  \\   /:/ /:/  /  
   /\\  \\   /\\__\\ /::\\  \\     /:/_/:/  /      /:/__/      /:/__/ \\:\\__\\ /:/_/:/  /   
   \\:\\  \\ /:/  / \\/\\:\\  \\__  \\:\\/:/  /      /::\\  \\      \\:\\  \\ /:/  / \\:\\/:/  /    
    \\:\\  /:/  /   ~~\\:\\/\\__\\  \\::/__/      /:/\\:\\  \\      \\:\\  /:/  /   \\::/__/     
     \\:\\/:/  /       \\::/  /   \\:\\  \\      \\/__\\:\\  \\      \\:\\/:/  /     \\:\\  \\     
      \\::/  /        /:/  /     \\:\\__\\          \\:\\__\\      \\::/  /       \\:\\__\\    
       \\/__/         \\/__/       \\/__/           \\/__/       \\/__/         \\/__/    

          (LIFTOF - liftof is for tof, Version 0.10 'LELEWAA', Mar 2024)
          >> with support from the Hawaiian islands \u{1f30a}\u{1f308}\u{1f965}\u{1f334}

          * Documentation
          ==> GitHub   https://github.com/GAPS-Collab/gaps-online-software/tree/LELEWAA-0.10
          ==> API docs https://gaps-collab.github.io/gaps-online-software/

  ";

///// Routine to end the liftof-cc program, finish up with current run 
///// and clean up
/////
///// FIXME - maybe this should go to liftof-cc
//pub fn end_liftof_cc(thread_control     : Arc<Mutex<ThreadControl>>) {
//  match thread_control.try_lock() {
//    Ok(mut tc) => {
//      //println!("== ==> [signal_handler] acquired thread_control lock!");
//      //println!("Tread control {:?}", tc);
//      if !tc.thread_cmd_dispatch_active 
//      && !tc.thread_data_sink_active
//      && !tc.thread_event_bldr_active 
//      && !tc.thread_master_trg_active  {
//        println!(">> So long and thanks for all the \u{1F41F} <<"); 
//        exit(0);
//      }
//      tc.stop_flag = true;
//      println!("== ==> [signal_handler] Stop flag is set, we are waiting for threads to finish...");
//      //println!("{}", tc);
//    }
//    Err(err) => {
//      error!("Can't acquire lock for ThreadControl! {err}");
//    }
//  }
//}

/// Handle incoming POSIX signals
pub fn signal_handler(thread_control     : Arc<Mutex<ThreadControl>>) {
  let sleep_time = Duration::from_millis(300);
  let mut signals = Signals::new(&[SIGTERM, SIGINT]).expect("Unknown signals");
  'main: loop {
    thread::sleep(sleep_time);

    // check pending signals and handle
    // SIGTERM and SIGINT
    for signal in signals.pending() {
      match signal as c_int {
        SIGTERM | SIGINT => {
          println!("=> {}", String::from("SIGTERM or SIGINT received. Maybe Ctrl+C has been pressed! Commencing program shutdown!").red().bold());
          match thread_control.lock() {
            Ok(mut tc) => {
              tc.sigint_recvd = true;
            }
            Err(err) => {
              error!("Can't acquire lock for ThreadControl! {err}");
            },
          }
          break 'main; // now end myself
        } 
        _ => {
          error!("Received signal, but I don't have instructions what to do about it!");
        }
      }
    }
  }
}


/// Make sure that the loglevel is in color, even though not using pretty_env logger
pub fn color_log(level : &Level) -> ColoredString {
  match level {
    Level::Error    => String::from(" ERROR!").red(),
    Level::Warn     => String::from(" WARN  ").yellow(),
    Level::Info     => String::from(" Info  ").green(),
    Level::Debug    => String::from(" debug ").blue(),
    Level::Trace    => String::from(" trace ").cyan(),
  }
}

/// Set up the environmental (env) logger
/// with our format
///
/// Ensure that the lines and module paths
/// are printed in the logging output
pub fn init_env_logger() {
  env_logger::builder()
    .format(|buf, record| {
    writeln!( buf, "[{ts} - {level}][{module_path}:{line}] {args}",
      ts    = Utc::now().format("%Y/%m/%d-%H:%M:%SUTC"), 
      level = color_log(&record.level()),
      module_path = record.module_path().unwrap_or("<unknown>"),
      line  = record.line().unwrap_or(0),
      args  = record.args()
      )
    }).init();
}

/// Keep track of run related statistics, errors
#[derive(Debug, Copy, Clone)]
pub struct RunStatistics {
  /// The number of events we have recorded
  pub n_events_rec      : usize,
  /// The number of packets going through 
  /// the event processing
  pub evproc_npack      : usize,
  /// The first event id we saw
  pub first_evid        : u32,
  /// The last event id we saw
  pub last_evid         : u32,
  /// The number of times we encountered 
  /// a deserialization issue
  pub n_err_deser       : usize,
  /// The number of times we encountered 
  /// an issue while sending over zmq
  pub n_err_zmq_send    : usize,
  /// The number of times we encountered
  /// an issue with a wrong channel identifier
  pub n_err_chid_wrong  : usize,
  /// How many times did we read out an incorrect
  /// tail?
  pub n_err_tail_wrong  : usize,
  /// The number of times we failed a crc32 check
  pub n_err_crc32_wrong : usize,
}

impl RunStatistics {
  
  pub fn new() -> Self {
    Self {
      n_events_rec      : 0,
      evproc_npack      : 0,
      first_evid        : 0,
      last_evid         : 0,
      n_err_deser       : 0,
      n_err_zmq_send    : 0,
      n_err_chid_wrong  : 0,
      n_err_tail_wrong  : 0,
      n_err_crc32_wrong : 0,
    }
  }

  pub fn get_n_anticipated(&self) -> i32 {
    self.last_evid as i32 - self.first_evid as i32
  }
}

impl fmt::Display for RunStatistics {
  fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
    let mut resp = String::from("<RunStatistics:\n");
    resp += &(format!("  first event id : {}\n", self.first_evid));
    resp += &(format!("  last  event id : {}\n", self.last_evid));
    resp += &(format!("  --> expected {} event (ids)\n", self.get_n_anticipated()));
    resp += &(format!("  event_processing #packets : {}\n", self.evproc_npack));
    if self.get_n_anticipated() != self.evproc_npack as i32 {
      resp += &(format!("  --> discrepancy of {} event (ids)\n", self.get_n_anticipated() - self.evproc_npack as i32))
    }
    resp += &(format!("  event_processing n tail err : {}\n", self.n_err_tail_wrong));
    resp += &(format!("  event_processing n chid err : {}\n", self.n_err_chid_wrong));
    write!(f, "{}", resp)
  }
}

#[cfg(feature="database")]
/// Sine fit without using external libraries
pub fn fit_sine_sydney(volts: &Vec<f32>, times: &Vec<f32>) -> (f32, f32, f32) {
  let start_bin = 20;
  let size_bin = 900;
  let pi = PI;
  let mut data_size = 0;

  let mut xi_yi = 0.0;
  let mut xi_zi = 0.0;
  let mut yi_zi = 0.0;
  let mut xi_xi = 0.0;
  let mut yi_yi = 0.0;
  let mut xi_sum = 0.0;
  let mut yi_sum = 0.0;
  let mut zi_sum = 0.0;

  for i in start_bin..(start_bin + size_bin) {
      let xi = (2.0 * pi * 0.02 * times[i]).cos();
      let yi = (2.0 * pi * 0.02 * times[i]).sin();
      let zi = volts[i];

      xi_yi += xi * yi;
      xi_zi += xi * zi;
      yi_zi += yi * zi;
      xi_xi += xi * xi;
      yi_yi += yi * yi;
      xi_sum += xi;
      yi_sum += yi;
      zi_sum += zi;

      data_size += 1;
  }

  let mut a_matrix = [[0.0; 3]; 3];
  a_matrix[0][0] = xi_xi;
  a_matrix[0][1] = xi_yi;
  a_matrix[0][2] = xi_sum;
  a_matrix[1][0] = xi_yi;
  a_matrix[1][1] = yi_yi;
  a_matrix[1][2] = yi_sum;
  a_matrix[2][0] = xi_sum;
  a_matrix[2][1] = yi_sum;
  a_matrix[2][2] = data_size as f32;

  let determinant = a_matrix[0][0] * a_matrix[1][1] * a_matrix[2][2]
      + a_matrix[0][1] * a_matrix[1][2] * a_matrix[2][0]
      + a_matrix[0][2] * a_matrix[1][0] * a_matrix[2][1]
      - a_matrix[0][0] * a_matrix[1][2] * a_matrix[2][1]
      - a_matrix[0][1] * a_matrix[1][0] * a_matrix[2][2]
      - a_matrix[0][2] * a_matrix[1][1] * a_matrix[2][0];

  let inverse_factor = 1.0 / determinant;

  let mut cofactor_matrix = [[0.0; 3]; 3];
  cofactor_matrix[0][0] = a_matrix[1][1] * a_matrix[2][2] - a_matrix[2][1] * a_matrix[1][2];
  cofactor_matrix[0][1] = (a_matrix[1][0] * a_matrix[2][2] - a_matrix[2][0] * a_matrix[1][2]) * -1.0;
  cofactor_matrix[0][2] = a_matrix[1][0] * a_matrix[2][1] - a_matrix[2][0] * a_matrix[1][1];
  cofactor_matrix[1][0] = (a_matrix[0][1] * a_matrix[2][2] - a_matrix[2][1] * a_matrix[0][2]) * -1.0;
  cofactor_matrix[1][1] = a_matrix[0][0] * a_matrix[2][2] - a_matrix[2][0] * a_matrix[0][2];
  cofactor_matrix[1][2] = (a_matrix[0][0] * a_matrix[2][1] - a_matrix[2][0] * a_matrix[0][1]) * -1.0;
  cofactor_matrix[2][0] = a_matrix[0][1] * a_matrix[1][2] - a_matrix[1][1] * a_matrix[0][2];
  cofactor_matrix[2][1] = (a_matrix[0][0] * a_matrix[1][2] - a_matrix[1][0] * a_matrix[0][2]) * -1.0;
  cofactor_matrix[2][2] = a_matrix[0][0] * a_matrix[1][1] - a_matrix[1][0] * a_matrix[0][1];

  let mut inverse_matrix = [[0.0; 3]; 3];
  for i in 0..3 {
      for j in 0..3 {
          inverse_matrix[i][j] = cofactor_matrix[j][i] * inverse_factor;
      }
  }

  let p = [xi_zi, yi_zi, zi_sum];
  let a = inverse_matrix[0][0] * p[0] + inverse_matrix[1][0] * p[1] + inverse_matrix[2][0] * p[2];
  let b = inverse_matrix[0][1] * p[0] + inverse_matrix[1][1] * p[1] + inverse_matrix[2][1] * p[2];

  let phi    = a.atan2(b);
  let amp    = (a*a + b*b).sqrt();
  let freq   = 0.02 as f32;

  (amp, freq, phi)
}

//*************************************************
// I/O - read/write (general purpose) files
//
//
//pub fn read_value_from_file(file_path: &str) -> io::Result<u32> {
//  let mut file = File::open(file_path)?;
//  let mut contents = String::new();
//  file.read_to_string(&mut contents)?;
//  let value: u32 = contents.trim().parse().map_err(|err| {
//    io::Error::new(io::ErrorKind::InvalidData, err)
//  })?;
//  Ok(value)
//}

/**************************************************/


/// Helper function to generate a proper tcp string starting
/// from the ip one.
pub fn build_tcp_from_ip(ip: String, port: String) -> String {
  //String::from("tcp://") + &ip + ":" + &port
  format!("tcp://{}:{}", ip, port)
}


//**********************************************
//
// Analysis
//

/// Waveform analysis engine - identify waveform variables
///
/// This will populate the TofHits in an RBEvent
///
/// TofHits contain information about peak location,
/// charge, timing.
///
/// FIXME - I think this should take a HashMap with 
/// algorithm settings, which we can load from a 
/// json file
///
/// # Arguments
///
/// * event       : current RBEvent with waveforms to 
///                 work on
/// * rb          : ReadoutBoard as loaded from the DB, 
///                 with latest calibration attached
/// * settings    : Parameters to configure the waveform
///                 analysis & peak finding
#[cfg(feature="database")]
pub fn waveform_analysis(event         : &mut RBEvent,
                         rb            : &ReadoutBoard,
                         settings      : AnalysisEngineSettings)
-> Result<(), AnalysisError> {
  // Don't do analysis for mangled events!
  if event.has_any_mangling_flag() {
    warn!("Event for RB {} has data mangling! Not doing analysis!", rb.rb_id);
    return Err(AnalysisError::DataMangling);
  }
  match event.self_check() {
    Err(_err) => {
      // Phlip want to ahve all hits even if they are broken
    },
    Ok(_)    => ()
  }
  let active_channels = event.header.get_channels();
  // will become a parameter
  let fit_sinus       = true;
  // allocate memory for the calbration results
  let mut voltages    : Vec<f32>= vec![0.0; NWORDS];
  let mut times       : Vec<f32>= vec![0.0; NWORDS];

  // Step 0 : If desired, fit sine
  let mut fit_result = (0.0f32, 0.0f32, 0.0f32);
  if fit_sinus {
    if !active_channels.contains(&8) {
      warn!("RB {} does not have ch9 data!", rb.rb_id);
      //println!("{}", event.header);
      return Err(AnalysisError::NoChannel9);
    }
    rb.calibration.voltages(9,
                            event.header.stop_cell as usize,
                            &event.adc[8],
                            &mut voltages);
    //warn!("We have to rework the spike cleaning!");
    //match RBCalibrations::spike_cleaning(&mut ch_voltages,
    //                                     event.header.stop_cell) {
    //  Err(err) => {
    //    error!("Spike cleaning failed! {err}");
    //  }
    //  Ok(_)    => ()
    //}
    rb.calibration.nanoseconds(9,
                               event.header.stop_cell as usize,
                               &mut times);
    fit_result                = fit_sine_sydney(&voltages, &times);

    //println!("FIT RESULT = {:?}", fit_result);
    //event.header.set_sine_fit(fit_result);
  }

  // structure to store final result
  // extend with Vec<TofHit> in case
  // we want to have multiple hits
  let mut paddles    = HashMap::<u8, TofHit>::new();
  //println!("RBID {}, Paddles {:?}", rb.rb_id ,rb.get_paddle_ids());
  for pid in rb.get_paddle_ids() {
    // cant' fail by constructon of pid
    let ch_a = rb.get_pid_rbchA(pid).unwrap() as usize;
    let ch_b = rb.get_pid_rbchB(pid).unwrap() as usize;
    let mut hit = TofHit::new();
    hit.paddle_id = pid;
    //println!("{ch_a}, {ch_b}, active_channels {:?}", active_channels);
    for (k, ch) in [ch_a, ch_b].iter().enumerate() {
      // Step 1: Calibration
      //println!("Ch {}, event {}", ch, event);
      //println!("---------------------------");
      //println!("pid {}, active channels : {:?}, ch {}",pid, active_channels, ch);
      if !active_channels.contains(&(*ch as u8 -1)) {
        trace!("Skipping channel {} because it is not marked to be readout in the event header channel mask!", ch);
        continue;
      }
      //println!("Will do waveform analysis for ch {}", ch);
      rb.calibration.voltages(*ch,
                              event.header.stop_cell as usize,
                              &event.adc[*ch as usize -1],
                              &mut voltages);
      //FIXME - spike cleaning!
      //match RBCalibrations::spike_cleaning(&mut ch_voltages,
      //                                     event.header.stop_cell) {
      //  Err(err) => {
      //    error!("Spike cleaning failed! {err}");
      //  }
      //  Ok(_)    => ()
      //}
      rb.calibration.nanoseconds(*ch,
                                 event.header.stop_cell as usize,
                                 &mut times);
      // Step 2: Pedestal subtraction
      let (ped, ped_err) = calculate_pedestal(&voltages,
                                              settings.pedestal_thresh,
                                              settings.pedestal_begin_bin,
                                              settings.pedestal_win_bins);
      trace!("Calculated pedestal of {} +- {}", ped, ped_err);
      for n in 0..voltages.len() {
        voltages[n] -= ped;
      }
      let mut charge : f32 = 0.0;
      //let peaks : Vec::<(usize, usize)>;
      let mut cfd_times = Vec::<f32>::new();
      let mut max_volts = 0.0f32;
      // Step 4 : Find peaks
      // FIXME - what do we do for multiple peaks?
      // Currently we basically throw them away
      match find_peaks(&voltages ,
                       &times    ,
                       settings.find_pks_t_start , 
                       settings.find_pks_t_window,
                       settings.min_peak_size    ,
                       settings.find_pks_thresh  ,
                       settings.max_peaks      ) {
        Err(err) => {
          // FIXME - if this happens, most likely the channel is dead. 
          debug!("Unable to find peaks for RB{:02} ch {ch}! Ignoring this channel!", rb.rb_id);
          debug!("We won't be able to calculate timing information for this channel! Err {err}");
        },
        Ok(peaks)  => {
          //peaks = pks;
          // Step 5 : Find tdcs
          //println!("Found {} peaks for ch {}! {:?}", peaks.len(), raw_ch, peaks);
          for pk in peaks.iter() {
            match cfd_simple(&voltages,
                             &times,
                             settings.cfd_fraction,
                             pk.0, pk.1) {
              Err(err) => {
                debug!("Unable to calculate cfd for peak {} {}! {}", pk.0, pk.1, err);
              }
              Ok(cfd) => {
                cfd_times.push(cfd);
              }
            }
            let pk_height = voltages[pk.0..pk.1].iter().max_by(|a,b| a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Less)).unwrap(); 
            max_volts = *pk_height;
            let max_index = voltages.iter().position(|element| *element == max_volts).unwrap();

            let (start_q_int, stop_q_int) = if max_index - 40 < 10 {
              (10, 210)
            } else {
              (max_index - 40, max_index + 160)
            };
          

            //debug!("Check impedance value! Just using 50 [Ohm]");
            // Step 3 : charge integration
            // FIXME - make impedance a settings parameter
            match integrate(&voltages,
                            &times,
                            //settings.integration_start,
                            //settings.integration_window,
                            //pk.0, 
                            //pk.1,
                            start_q_int,
                            stop_q_int,
                            50.0) {
              Err(err) => {
                error!("Integration failed! Err {err}");
              }
              Ok(chrg)   => {
                charge = chrg;
              }
            }
            // // just do the first peak for now
            // let pk_height = voltages[pk.0..pk.1].iter().max_by(|a,b| a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Less)).unwrap(); 
            // max_volts = *pk_height; 
            // //debug!("Check impedance value! Just using 50 [Ohm]");
            // // Step 3 : charge integration
            // // FIXME - make impedance a settings parameter
            // match integrate(&voltages,
            //                 &times,
            //                 //settings.integration_start,
            //                 //settings.integration_window,
            //                 pk.0, 
            //                 pk.1,
            //                 50.0) {
            //   Err(err) => {
            //     error!("Integration failed! Err {err}");
            //   }
              
            break;
          }
        }// end OK
      } // end match find_peaks 
      let mut tdc : f32 = 0.0; 
      if cfd_times.len() > 0 {
        tdc = cfd_times[0];
      }
      //println!("Calucalated tdc {}, charge {}, max {} for ch {}!", tdc, charge, max_volts, ch); 
      //if rb.channel_to_paddle_end_id[*raw_ch as usize] > 2000 {
      if k == 0 {
        hit.ftime_a      = tdc;
        hit.fpeak_a      = max_volts;
        hit.set_time_a(tdc);
        hit.set_charge_a(charge);
        hit.set_peak_a(max_volts);
        hit.baseline_a     = f16::from_f32(ped);
        hit.baseline_a_rms = f16::from_f32(ped_err);
      } else {
        hit.ftime_b = tdc;
        hit.fpeak_b = max_volts;
        hit.set_time_b(tdc);
        hit.set_charge_b(charge);
        hit.set_peak_b(max_volts);
        hit.baseline_b     = f16::from_f32(ped);
        hit.baseline_b_rms = f16::from_f32(ped_err);
        // this is the seoond iteration,
        // we are done!
        hit.phase = f16::from_f32(fit_result.2);
        paddles.insert(pid, hit);
      }
    }
  }
  let result = paddles.into_values().collect();
  event.hits = result;
  //print ("EVENT {}", event);
  Ok(())
}

//**********************************************

/// Load the rb channel vs paddle end id mapping
///
/// The map file is expected to have information for 
/// all rbs, rb_id is used to grab the section for 
/// the specific rb.
pub fn get_rb_ch_pid_map(map_file : PathBuf, rb_id : u8) -> RBChannelPaddleEndIDMap {
  let mut mapping = RBChannelPaddleEndIDMap::new();
  let json_content : String;
  match read_to_string(&map_file) {
    Ok(_json_content) => {
      json_content = _json_content;
    },
    Err(err) => { 
      error!("Unable to parse json file {}. Error {err}", map_file.display());
      return mapping;
    }      
  }
  let json : Value;
  match serde_json::from_str(&json_content) {
    Ok(_json) => {
      json = _json;
    },
    Err(err) => { 
      error!("Unable to parse json file {}. Error {err}", map_file.display());
      return mapping;
    }
  }
  for ch in 0..8 {
    let tmp_val = &json[rb_id.to_string()][(ch +1).to_string()];
    let val = tmp_val.to_string().parse::<u16>().unwrap_or(0);
    mapping.insert(ch as u8 + 1, val);
  }
  mapping
}

pub fn get_ltb_dsi_j_ch_mapping(mapping_file : PathBuf) -> DsiLtbRBMapping {
  let mut mapping = HashMap::<u8,HashMap::<u8,HashMap::<u8,(u8,u8)>>>::new();
  for dsi in 1..6 {
    mapping.insert(dsi, HashMap::<u8,HashMap::<u8, (u8, u8)>>::new());
    for j in 1..6 {
      mapping.get_mut(&dsi).unwrap().insert(j, HashMap::<u8,(u8, u8)>::new());
      for ch in 1..17 {
        mapping.get_mut(&dsi).unwrap().get_mut(&j).unwrap().insert(ch, (0,0));
      }
    }
  }
  let json_content : String;
  match read_to_string(&mapping_file) {
    Ok(_json_content) => {
      json_content = _json_content;
    },
    Err(err) => { 
      error!("Unable to parse json file {}. Error {err}", mapping_file.display());
      return mapping;
    }      
  }
  let json : Value;
  match serde_json::from_str(&json_content) {
    Ok(_json) => {
      json = _json;
    },
    Err(err) => { 
      error!("Unable to parse json file {}. Error {err}", mapping_file.display());
      return mapping;
    }
  }
  for dsi in 1..6 { 
    for j in 1..6 {
      for ch in 1..17 {
        let val = mapping.get_mut(&dsi).unwrap().get_mut(&j).unwrap().get_mut(&ch).unwrap();
        //println!("Checking {} {} {}", dsi, j, ch);
        let tmp_val = &json[dsi.to_string()][j.to_string()][ch.to_string()];
        *val = (tmp_val[0].to_string().parse::<u8>().unwrap_or(0), tmp_val[1].to_string().parse::<u8>().unwrap_or(0));
      }
    }
  }
  debug!("Mapping {:?}", mapping);
  mapping
}

/// Convert an int value to the board ID string.
pub fn to_board_id_string(rb_id: u32) -> String {

  //String::from("RB") + &format!("{:02}", rb_id)
  format!("RB{:02}", rb_id)
}

/**********************************************************/

#[derive(Debug, Clone, Args, PartialEq)]
pub struct LtbThresholdOpts {
  /// ID of the LTB to target
  #[arg(short, long, default_value_t = DEFAULT_LTB_ID)]
  pub id: u8,
  /// Name of the threshold to be set
  #[arg(required = true)]
  pub name: LTBThresholdName,
  /// Threshold level to be set
  #[arg(required = true)]
  pub level: u16
}

impl LtbThresholdOpts {
  pub fn new(id: u8, name: LTBThresholdName, level: u16) -> Self {
    Self { 
      id,
      name,
      level
    }
  }
}

// repr is u16 in order to leave room for preamp bias
#[derive(Debug, Copy, Clone, PartialEq, serde::Deserialize, serde::Serialize, clap::ValueEnum)]
#[repr(u8)]
pub enum LTBThresholdName {
  Unknown  = 0u8,
  Hit      = 10u8,
  Beta     = 20u8,
  Veto     = 30u8,
}

impl LTBThresholdName {
  pub fn get_ch_number(threshold_name: LTBThresholdName) -> Result<u8, SetError> {
    match threshold_name {
      LTBThresholdName::Hit     => Ok(0u8),
      LTBThresholdName::Beta    => Ok(1u8),
      LTBThresholdName::Veto    => Ok(2u8),
      LTBThresholdName::Unknown => {
        error!("Not able to get a LTB threshold from Unknown");
        Err(SetError::EmptyInputData)
      }
    }
  }
}

impl fmt::Display for LTBThresholdName {
  fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
    let r = serde_json::to_string(self).unwrap_or(
      String::from("Error: cannot unwrap this PowerStatusEnum"));
    write!(f, "<PowerStatusEnum: {}>", r)
  }
}

impl From<u8> for LTBThresholdName {
  fn from(value: u8) -> Self {
    match value {
      0u8  => LTBThresholdName::Unknown,
      10u8 => LTBThresholdName::Hit,
      20u8 => LTBThresholdName::Beta,
      30u8 => LTBThresholdName::Veto,
      _    => LTBThresholdName::Unknown
    }
  }
}

#[derive(Debug, Copy, Clone, PartialEq, serde::Deserialize, serde::Serialize, clap::ValueEnum)]
#[repr(u8)]
pub enum TofComponent {
  Unknown   = 0u8,
  /// everything (LTB + preamps + MT)
  All       = 1u8,
  /// everything but MT (LTB + preamps)
  AllButMT  = 2u8,
  /// TOF CPU
  TofCpu    = 3u8,
  /// MT alone
  MT        = 10u8,
  /// all or specific RBs
  RB        = 20u8,
  /// all or specific PBs
  PB        = 30u8,
  /// all or specific LTBs
  LTB       = 40u8,
  /// all or specific preamp
  Preamp    = 50u8
}

impl fmt::Display for TofComponent {
  fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
    let r = serde_json::to_string(self).unwrap_or(
      String::from("Error: cannot unwrap this TofComponent"));
    write!(f, "<TofComponent: {}>", r)
  }
}

impl From<u8> for TofComponent {
  fn from(value: u8) -> Self {
    match value {
      0u8  => TofComponent::Unknown,
      1u8  => TofComponent::All,
      2u8  => TofComponent::AllButMT,
      3u8  => TofComponent::TofCpu,
      10u8 => TofComponent::MT,
      20u8 => TofComponent::RB,
      30u8 => TofComponent::PB,
      40u8 => TofComponent::LTB,
      50u8 => TofComponent::Preamp,
      _    => TofComponent::Unknown
    }
  }
}

impl From<TofComponent> for clap::builder::Str {
  fn from(value: TofComponent) -> Self {
    match value {
      TofComponent::Unknown  => clap::builder::Str::from("Unknown"),
      TofComponent::All      => clap::builder::Str::from("All"),
      TofComponent::AllButMT => clap::builder::Str::from("AllButMT"),
      TofComponent::TofCpu   => clap::builder::Str::from("TofCpu"),
      TofComponent::MT       => clap::builder::Str::from("MT"),
      TofComponent::RB       => clap::builder::Str::from("RB"),
      TofComponent::PB       => clap::builder::Str::from("PB"),
      TofComponent::LTB      => clap::builder::Str::from("LTB"),
      TofComponent::Preamp   => clap::builder::Str::from("Preamp")
    }
  }
}

