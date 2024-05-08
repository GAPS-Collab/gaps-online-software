pub mod master_trigger;
pub mod settings;
pub mod constants;

use constants::{
    DEFAULT_CALIB_VOLTAGE,
    DEFAULT_CALIB_EXTRA,
    DEFAULT_RB_ID,
    //DEFAULT_PB_ID,
    DEFAULT_LTB_ID,
    DEFAULT_PREAMP_ID,
    DEFAULT_PREAMP_BIAS,
    //DEFAULT_POWER_STATUS,
    DEFAULT_RUN_TYPE,
    DEFAULT_RUN_EVENT_NO,
    //DEFAULT_RUN_TIME,
    PREAMP_MIN_BIAS,
    PREAMP_MAX_BIAS
};

use std::thread;
use std::time::Duration;
use std::os::raw::c_int;
use std::sync::{
    Arc,
    Mutex,
};
use std::process::exit;


pub use master_trigger::{
    master_trigger,
    MTBSettings,
};

pub use settings::{
    LiftofSettings,
    AnalysisEngineSettings,
};

use std::error::Error;
use std::fmt;

use std::{
    fs::File,
};
use std::path::PathBuf;
use std::fs::read_to_string;
use std::io::{
    self,
    Read,
    Write,
};
use std::collections::HashMap;
use std::net::IpAddr;
use std::net::Ipv4Addr;
use crossbeam_channel::Receiver;
use zmq;
use colored::{Colorize, ColoredString};

use serde_json::Value;

use log::Level;

#[macro_use] extern crate log;
extern crate env_logger;

use signal_hook::iterator::Signals;
use signal_hook::consts::signal::{
    SIGTERM,
    SIGINT
};
//use ndarray::{array, Array1};
//use nlopt::{Algorithm, Objective, Optimization, Result};

use tof_dataclasses::DsiLtbRBMapping;
use tof_dataclasses::database::ReadoutBoard;
use tof_dataclasses::threading::{
    ThreadControl,
};

use tof_dataclasses::constants::NWORDS;
use tof_dataclasses::calibrations::{
    //RBCalibrations,
    find_zero_crossings,
};
use tof_dataclasses::packets::{
    TofPacket,
    PacketType,
};
use tof_dataclasses::errors::{
    AnalysisError,
    SetError
};
use tof_dataclasses::serialization::Serialization;
use tof_dataclasses::commands::RBCommand;
use tof_dataclasses::events::{
    RBEvent,
    TofHit,
};
use tof_dataclasses::events::tof_hit::Peak;

use tof_dataclasses::analysis::{
    calculate_pedestal,
    integrate,
    cfd_simple,
    find_peaks,
    get_paddle_t0,
    pos_across
};

use tof_dataclasses::RBChannelPaddleEndIDMap;

use clap::{arg,
  //value_parser,
  //ArgAction,
  //Command,
  Parser,
  Args,
  Subcommand
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

          * Documentation
          ==> GitHub   https://github.com/GAPS-Collab/gaps-online-software/tree/LELEWAA-0.10
          ==> API docs https://gaps-collab.github.io/gaps-online-software/

  ";

/// Handle incoming POSIX signals
pub fn signal_handler(thread_control     : Arc<Mutex<ThreadControl>>) {
  let one_second = Duration::from_millis(1000);

  let mut end_program = false;
  let mut signals = Signals::new(&[SIGTERM, SIGINT]).expect("Unknown signals");
  loop {
    thread::sleep(1*one_second);
    match thread_control.lock() {
      Ok(tc) => {
        if !tc.thread_cmd_dispatch_active 
        && !tc.thread_data_sink_active
        && !tc.thread_event_bldr_active 
        && !tc.thread_master_trg_active {
          exit(0);
        }
      }
      Err(err) => {
        error!("Can't acquire lock for ThreadControl! Unable to set calibration mode! {err}");
      },
    }

    // check pending signals and handle
    // SIGTERM and SIGINT
    for signal in signals.pending() {
      match signal as c_int {
        SIGTERM => {
          println!("=> {}", String::from("SIGINT received. Maybe Ctrl+C has been pressed!").red().bold());
          end_program = true;
        } 
        SIGINT => {
          println!("=> {}", String::from("SIGTERM received").red().bold());
          end_program = true;
        }
        _ => {
          error!("Received signal, but I don't have instructions what to do about it!");
        }
      }
    }
    if end_program {
      println!("=> Shutting down threads...");
      match thread_control.lock() {
        Ok(mut tc) => {
          tc.stop_flag = true;
        },
        Err(err) => {
          error!("Can't acquire lock for ThreadControl! Unable to set calibration mode! {err}");
        },
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
    writeln!( buf, "[{level}][{module_path}:{line}] {args}",
      level = color_log(&record.level()),
      module_path = record.module_path().unwrap_or("<unknown>"),
      line = record.line().unwrap_or(0),
      args = record.args()
      )
    }).init();
}

/// Nicer output for thread "heartbeats" to terminal
pub fn heartbeat_printer(strings: Vec<String>) {
    // Determine the maximum length of the strings to ensure consistent length
    let max_length = strings.iter().map(|s| s.len()).max().unwrap_or(0);
    // Calculate total width including ">>" and "<<" markers
    let total_width = max_length + 4; // 4 extra characters for ">>" and "<<"

    for s in strings {
        // Use the calculated total_width for consistent formatting
        println!(">>{: <width$}<<", s, width = total_width - 4);
    }
}

/// Common settings for apps, e.g. liftof-tui
#[derive(Debug, Clone)]
pub struct AppSettings {
  pub cali_master_path : String,
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

//fn sine_to_fit(amp : f32, freq : f32, phase : f32, time : &Vec<f32>, ys : &mut Vec<f32>) {
//  //let ys = Vec::<f32>::with_capacity(time.len());
//  for k in 0..time.len() {
//    ys[k] = amp * (freq * time[k] + phase).sin(); 
//  }
//}
//
//fn cost_function(amp : f32, freq : f32, phase : f32, time : &Vec<f32>, volts : &Vec<f32>) -> f32 {
//  //let 
//  //let fitted_values = amplitude * (2.0 * std::f32::consts::PI * frequency * time + phase).sin();
//  let fit_volts = Vec::<f32>::with_capacity(time.len());
//  sine_to_fit(amp, freq, phase, &time, &mut fit_volts);
//  let mut chi_square = 0f32;
//  for k in 0..fit_volts.len() {
//    chi_square += (volts[k] - fit_volts[k]).powi(2);
//    // FIXME - error
//  }
//  chi_square
//}


/// FIXME - proper fitting algorithm
/// This here is bad, because it does not interpolate between 
/// the bins
fn fit_sine(time: &Vec<f32>, data: &Vec<f32>) -> (f32, f32, f32) {
  let z_cross   = find_zero_crossings(&data);
  let mut y_max = f32::MIN;
  let mut y_min = f32::MAX;
  for y in data {
    if *y > y_max {
      y_max = *y;
    }
    if *y < y_min {
      y_min = *y;
    }
  }
  let amp   = f32::abs(y_max - y_min)/2.0;
  let mut phase = 0.0;
  let mut freq  = 0.0;
  if z_cross.len() >= 3 {
    phase = time[z_cross[0]];
    freq  = 1.0/(time[z_cross[2]] - time[z_cross[0]]);
  }
  (amp,freq,phase)
}

//*************************************************
// I/O - read/write (general purpose) files
//
//
pub fn read_value_from_file(file_path: &str) -> io::Result<u32> {
  let mut file = File::open(file_path)?;
  let mut contents = String::new();
  file.read_to_string(&mut contents)?;
  let value: u32 = contents.trim().parse().map_err(|err| {
    io::Error::new(io::ErrorKind::InvalidData, err)
  })?;
  Ok(value)
}





/**************************************************/


/// Helper function to generate a proper tcp string starting
/// from the ip one.
pub fn build_tcp_from_ip(ip: String, port: String) -> String {
  //String::from("tcp://") + &ip + ":" + &port
  format!("tcp://{}:{}", ip, port)
}

/// Broadcast commands over the tof-computer network
/// socket via zmq::PUB to the rb network.
/// Currently, the only participants in the rb network
/// are the readoutboards.
///
/// After the reception of a TofCommand, this will currently be 
/// broadcasted to all readout boards.
///
/// ISSUE/FIXME  : Send commands only to specific addresses.
///
/// # Arguments 
///
/// * cmd        : a \[crossbeam\] receiver, to receive 
///                TofCommands.
pub fn readoutboard_commander(cmd : &Receiver<TofPacket>){
  debug!(".. started!");
  let ctx = zmq::Context::new();
  let this_board_ip = IpAddr::V4(Ipv4Addr::new(10, 0, 1, 1));

  let address_ip;
  match this_board_ip {
    IpAddr::V4(ip) => address_ip = ip.to_string().clone(),
    IpAddr::V6(_) => panic!("Currently, we do not support IPV6!")
  }
  let data_address : String = build_tcp_from_ip(address_ip,DATAPORT.to_string());
  let data_socket = ctx.socket(zmq::PUB).expect("Unable to create 0MQ PUB socket!");
  data_socket.bind(&data_address).expect("Unable to bind to data (PUB) socket {data_adress}");
  println!("==> 0MQ PUB socket bound to address {data_address}");
  loop {
    // check if we get a command from the main 
    // thread
    match cmd.try_recv() {
      Err(err) => trace!("Did not receive a new command, error {err}"),
      Ok(packet) => {
        // now we have several options
        match packet.packet_type {
          PacketType::TofCommand => {
            info!("Received TofCommand! Broadcasting to all TOF entities who are listening!");
            let mut payload  = String::from("BRCT").into_bytes();
            payload.append(&mut packet.to_bytestream());
            match data_socket.send(&payload,0) {
              Err(err) => error!("Unable to send command! Error {err}"),
              Ok(_)    => info!("BRCT command sent!")
            }
          },
          PacketType::RBCommand => {
            debug!("Received RBCommand!");
            let mut payload_str  = String::from("RB");
            match RBCommand::from_bytestream(&packet.payload, &mut 0) {
              Ok(rb_cmd) => {
                let to_rb_id = rb_cmd.rb_id;
                if rb_cmd.rb_id < 10 {
                  payload_str += &String::from("0");
                  payload_str += &to_rb_id.to_string();
                } else {
                  payload_str += &to_rb_id.to_string();
                }

                let mut payload = payload_str.into_bytes();
                payload.append(&mut packet.to_bytestream());
                match data_socket.send(&payload,0) {
                  Err(err) => error!("Unable to send command {}! Error {err}", rb_cmd),
                  Ok(_)    => debug!("Making event request! {}", rb_cmd)
                }
              }
              Err(err) => {
                error!("Can not construct RBCommand, error {err}");
              }
            }
          },
          _ => {
            error!("Received garbage package! {}", packet);
          }
        }// end match
      }
    }
  }
}

//**********************************************
//
// Analysis
//

/// Extract peaks from waveforms
///
/// Helper for waveform analysis
pub fn get_peaks() -> Vec<Peak> {
  let peaks = Vec::<Peak>::new();
  peaks
}


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
pub fn waveform_analysis(event         : &mut RBEvent,
                         rb            : &ReadoutBoard,
                         settings      : AnalysisEngineSettings)
-> Result<(), AnalysisError> {
  //if event.status != EventStatus::Perfect {
  //if event.header.broken {
  //  // just return the analysis error, there 
  //  // is probably nothing else we can do?
  //  return Err(AnalysisError::InputBroken);
  //}
  // ch -> pid
  // pid -> (ch, ch) (for the two paddle ends)
  //let mut pid_vs_chs = HashMap::<u8, (PaddleEndIdentifier,[u8;2])>::new();
  let active_channels = event.header.get_channels();
  // will become a parameter
  let fit_sinus       = true;
  // allocate memory for the calbration results
  let mut voltages    : Vec<f32>= vec![0.0; NWORDS];
  let mut times       : Vec<f32>= vec![0.0; NWORDS];

  // Step 0 : If desired, fit sine
  if fit_sinus {
    if !active_channels.contains(&8) {
      error!("RB {} does not have ch9 data!", rb.rb_id);
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
    let fit_result = fit_sine(&times, &voltages);
    //println!("FIT RESULT = {:?}", fit_result);
    event.header.set_sine_fit(fit_result);
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
          error!("Unable to find peaks for RB{:02} ch {ch}! Ignoring this channel!", rb.rb_id);
          error!("We won't be able to calculate timing information for this channel! Err {err}");
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
            // just do the first peak for now
            let pk_height = voltages[pk.0..pk.1].iter().max_by(|a,b| a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Less)).unwrap(); 
            max_volts = *pk_height; 
            //debug!("Check impedance value! Just using 50 [Ohm]");
            // Step 3 : charge integration
            // FIXME - make impedance a settings parameter
            match integrate(&voltages,
                            &times,
                            //settings.integration_start,
                            //settings.integration_window,
                            pk.0, 
                            pk.1,
                            50.0) {
              Err(err) => {
                error!("Integration failed! Err {err}");
              }
              Ok(chrg)   => {
                charge = chrg;
              }
            }
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
        hit.ftime_b      = tdc;
        hit.fpeak_b      = max_volts;
        hit.set_time_b(tdc);
        hit.set_charge_b(charge);
        hit.set_peak_b(max_volts);
      } else {
        hit.ftime_a = tdc;
        hit.fpeak_b = max_volts;
        hit.set_time_a(tdc);
        hit.set_charge_a(charge);
        hit.set_peak_a(max_volts);
        // this is the seoond iteration,
        // we are done!
        paddles.insert(pid, hit);
      }
    }
  }
  //println!("Paddles {:?}", paddles);
  for (_, hit) in paddles.iter_mut() {
    // unwrap should not fail by construction
    let t0 = get_paddle_t0(hit.ftime_a, hit.ftime_b, rb.get_paddle_length(hit.paddle_id).unwrap_or(1.0));
    //println!("Cot t0 :{}", t0);
    //println!("Hit : {}", hit);
    //println!("Hit.ftime_a {}", hit.ftime_a);
    let pa = pos_across(hit.ftime_a, t0);
    //println!("pa : {}", pa);
    hit.set_t0(t0);
    hit.set_pos_across(pa);
    hit.set_edep((hit.fpeak_a + hit.fpeak_b) / 2.0);
    //println!("caluclated {} {} for {}",t0, pa, hit);
  }
  let result = paddles.into_values().collect();
  event.hits = result;
  //print ("EVENT {}", event);
  Ok(())
}

//**********************************************
//
// Subsystem communication
//


/// construct a request string which can be broadcast over 0MQ to all the boards
/// ///
/// /// Boards will only send paddle information when this request string is received
pub fn construct_event_request(rb_id : u8) -> String {
  let mut request = String::from("RB");
  if rb_id < 10 {
    request += "0";
  }
  request += &rb_id.to_string();
  request
}

#[derive(Debug, serde::Deserialize, serde::Serialize)]
#[repr(u8)]
pub enum ReadoutBoardError {
  NoConnectionInfo,
  NoResponse,
}

impl fmt::Display for ReadoutBoardError {
  fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
    let r = serde_json::to_string(self).unwrap_or(
      String::from("Error: cannot unwrap this ReadoutBoardError"));
    write!(f, "<ReadoutBoardError: {}>", r)
  }
}

impl Error for ReadoutBoardError {
}


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
/// Command Enums and stucts
#[derive(Debug, Parser, PartialEq)]
pub enum CommandCC {
  /// Listen for flight CPU commands.
  Listen(ListenCmd),
  /// Staging mode - work through all .toml files
  /// in the staging area
  Staging(StagingCmd),
  /// Ping a TOF sub-system.
  Ping(PingCmd),
  /// Monitor a TOF sub-system.
  Moni(MoniCmd),
  /// Restart RB systemd
  SystemdReboot(SystemdRebootCmd),
  /// Power control of TOF sub-systems.
  #[command(subcommand)]
  Power(PowerCmd),
  /// Remotely trigger the readoutboards to run the calibration routines (tcal, vcal).
  #[command(subcommand)]
  Calibration(CalibrationCmd),
  /// Remotely set LTB thresholds or preamp bias.
  #[command(subcommand)]
  Set(SetCmd),
  /// Start/stop data taking run.
  #[command(subcommand)]
  Run(RunCmd)
}

/// Command Enums and stucts
#[derive(Debug, Clone, Parser, PartialEq)]
pub enum CommandRB {
  /// Remotely trigger the readoutboards to run the calibration routines (tcal, vcal).
  #[command(subcommand)]
  Calibration(CalibrationCmd),
  /// Remotely set LTB thresholds or preamp bias.
  #[command(subcommand)]
  Set(SetCmd),
  /// Start/stop data taking run.
  #[command(subcommand)]
  Run(RunCmd),
  /// Listen to commands from the central C&C server (liftof-cc).
  Listen(ListenCmd),
}

/// TOF SW cmds ====================================================
#[derive(Debug, Copy, Clone, Args, PartialEq)]
pub struct ListenCmd { }

#[derive(Debug, Copy, Clone, Args, PartialEq)]
pub struct StagingCmd { }

#[derive(Debug, Args, PartialEq)]
pub struct PingCmd {
  /// Component to target
  #[arg(value_parser = clap::builder::PossibleValuesParser::new([
          TofComponent::TofCpu,
          TofComponent::MT,
          TofComponent::RB,
          TofComponent::LTB
        ]),
        required = true)]
  pub component: TofComponent,
  /// Component ID
  #[arg(required = true)]
  pub id: u8
}

#[derive(Debug, Args, PartialEq)]
pub struct MoniCmd {
  /// Component to target
  #[arg(value_parser = clap::builder::PossibleValuesParser::new([
          TofComponent::TofCpu,
          TofComponent::MT,
          TofComponent::RB,
          TofComponent::LTB
        ]),
        required = true)]
  pub component: TofComponent,
  /// Component ID
  #[arg(required = true)]
  pub id: u8
}

#[derive(Debug, Args, PartialEq)]
pub struct SystemdRebootCmd {
  /// RB ID
  #[arg(required = true)]
  pub id: u8
}
/// END TOF SW cmds ================================================


/// Set cmds ====================================================
#[derive(Debug, Clone, Subcommand, PartialEq)]
pub enum SetCmd {
  /// Set MT configuration (WHAT SHOULD I DO WITH THIS TODO)
  //MTConfig(MTConfigOpts),
  /// Set threshold level on all LTBs or a single LTB
  LtbThreshold(LtbThresholdOpts),
  /// Set bias level on all preamps or a single preamp
  PreampBias(PreampBiasOpts)
}

// #[derive(Debug, Args, PartialEq)]
// pub struct MTConfigOpts {
//   /// RB to target in voltage calibration run.
//   #[arg(short, long, default_value_t = DEFAULT_RB_ID)]
//   pub id: u8,
//   /// Theshold level to be set
//   #[arg(required = true, 
//         value_parser = clap::value_parser!(i64).range(PREAMP_MIN_BIAS..=PREAMP_MAX_BIAS))]
//   pub bias: u16
// }

// impl MTConfigOpts {
//   pub fn new(id: u8, bias: u16) -> Self {
//     Self { 
//       id,
//       bias
//     }
//   }
// }

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

#[derive(Debug, Clone, Args, PartialEq)]
pub struct PreampBiasOpts {
  /// RB to target in voltage calibration run.
  #[arg(short, long, default_value_t = DEFAULT_RB_ID)]
  pub id: u8,
  /// Theshold level to be set
  #[arg(required = true, 
        value_parser = clap::value_parser!(i64).range(PREAMP_MIN_BIAS..=PREAMP_MAX_BIAS))]
  pub bias: u16
}

impl PreampBiasOpts {
  pub fn new(id: u8, bias: u16) -> Self {
    Self { 
      id,
      bias
    }
  }
}
/// END Set cmds ================================================

/// Calibration cmds ====================================================
#[derive(Debug, Clone, Subcommand, PartialEq)]
pub enum CalibrationCmd {
  /// Default calibration run, meaning 2 voltage calibrations and one timing calibration on all RBs with the default values.
  Default(DefaultOpts),
  /// No input data taking run. All RB are targeted are default ones if nothing else is specified.
  Noi(NoiOpts),
  /// Voltage data taking run. All RB are targeted and voltage are default ones if nothing else is specified.
  Voltage(VoltageOpts),
  /// Timing data taking run. All RB are targeted and voltage are default ones if nothing else is specified.
  Timing(TimingOpts)
}

#[derive(Debug, Clone, Args, PartialEq)]
pub struct DefaultOpts {
  /// Voltage level to be set in default calibration run.
  #[arg(short, long, default_value_t = DEFAULT_CALIB_VOLTAGE)]
  pub level: u16,
  /// ID of the RB to target in default calibration run.
  #[arg(short, long, default_value_t = DEFAULT_RB_ID)]
  pub id: u8,
  /// Extra arguments in default calibration run (not implemented).
  #[arg(short, long, default_value_t = DEFAULT_CALIB_EXTRA)]
  pub extra: u8,
}

impl DefaultOpts {
  pub fn new(level: u16, id: u8, extra: u8) -> Self {
    Self { 
      level,
      id,
      extra
    }
  }
}

#[derive(Debug, Clone, Args, PartialEq)]
pub struct NoiOpts {
  /// ID of the RB to target in no input calibration run.
  #[arg(short, long, default_value_t = DEFAULT_RB_ID)]
  pub id: u8,
  /// Extra arguments in no input calibration run (not implemented).
  #[arg(short, long, default_value_t = DEFAULT_CALIB_EXTRA)]
  pub extra: u8,
}

impl NoiOpts {
  pub fn new(id: u8, extra: u8) -> Self {
    Self { 
      id,
      extra
    }
  }
}

#[derive(Debug, Copy, Clone, Args, PartialEq)]
pub struct VoltageOpts {
  /// Voltage level to be set in voltage calibration run.
  #[arg(short, long, default_value_t = DEFAULT_CALIB_VOLTAGE)]
  pub level: u16,
  /// RB to target in voltage calibration run.
  #[arg(short, long, default_value_t = DEFAULT_RB_ID)]
  pub id: u8,
  /// Extra arguments in voltage calibration run (not implemented).
  #[arg(short, long, default_value_t = DEFAULT_CALIB_EXTRA)]
  pub extra: u8,
}

impl VoltageOpts {
  pub fn new(level: u16, id: u8, extra: u8) -> Self {
    Self { 
      level,
      id,
      extra
    }
  }
}

#[derive(Debug, Copy, Clone, Args, PartialEq)]
pub struct TimingOpts {
  /// Voltage level to be set in voltage calibration run.
  #[arg(short, long, default_value_t = DEFAULT_CALIB_VOLTAGE)]
  pub level: u16,
  /// RB to target in voltage calibration run.
  #[arg(short, long, default_value_t = DEFAULT_RB_ID)]
  pub id: u8,
  /// Extra arguments in voltage calibration run (not implemented).
  #[arg(short, long, default_value_t = DEFAULT_CALIB_EXTRA)]
  pub extra: u8,
}

impl TimingOpts {
  pub fn new(level: u16, id: u8, extra: u8) -> Self {
    Self { 
      level,
      id,
      extra
    }
  }
}
/// END Calibration cmds ================================================

/// Power cmds ====================================================
#[derive(Debug, Subcommand, PartialEq)]
pub enum PowerCmd {
  /// Power up everything (LTB + preamps + MT)
  All(PowerStatus),
  /// Power up MT alone
  MT(PowerStatus),
  /// Power up everything but MT (LTB + preamps)
  AllButMT(PowerStatus),
  /// Power up all or specific LTBs (changes threshold)
  LTB(LTBPowerOpts),
  /// Power up all or specific preamp (changes bias)
  Preamp(PreampPowerOpts)
}

#[derive(Debug, Args, PartialEq)]
pub struct PowerStatus {
  /// Which power status one wants to achieve
  #[arg(value_parser = clap::builder::PossibleValuesParser::new([
          PowerStatusEnum::OFF,
          PowerStatusEnum::ON,
          PowerStatusEnum::Cycle
        ]),
        required = true)]
  pub status: PowerStatusEnum
}

impl PowerStatus {
  pub fn new(status: PowerStatusEnum) -> Self {
    Self { 
      status
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

// repr is u16 in order to leave room for preamp bias
#[derive(Debug, Copy, Clone, PartialEq, serde::Deserialize, serde::Serialize, clap::ValueEnum)]
#[repr(u8)]
pub enum PowerStatusEnum {
  Unknown   = 0u8,
  OFF       = 10u8,
  ON        = 20u8,
  Cycle     = 30u8
}

impl fmt::Display for PowerStatusEnum {
  fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
    let r = serde_json::to_string(self).unwrap_or(
      String::from("Error: cannot unwrap this PowerStatusEnum"));
    write!(f, "<PowerStatusEnum: {}>", r)
  }
}

impl From<u8> for PowerStatusEnum {
  fn from(value: u8) -> Self {
    match value {
      0u8  => PowerStatusEnum::Unknown,
      10u8 => PowerStatusEnum::OFF,
      20u8 => PowerStatusEnum::ON,
      30u8 => PowerStatusEnum::Cycle,
      _    => PowerStatusEnum::Unknown
    }
  }
}

impl From<PowerStatusEnum> for clap::builder::Str {
  fn from(value: PowerStatusEnum) -> Self {
    match value {
      PowerStatusEnum::Unknown => clap::builder::Str::from("Unknown"),
      PowerStatusEnum::OFF     => clap::builder::Str::from("OFF"),
      PowerStatusEnum::ON      => clap::builder::Str::from("ON"),
      PowerStatusEnum::Cycle   => clap::builder::Str::from("Cycle")
    }
  }
}

#[derive(Debug, Args, PartialEq)]
pub struct PBPowerOpts {
  /// Which power status one wants to achieve
  #[arg(long)]
  pub status: PowerStatusEnum,
  /// ID of the PB to be powered up
  #[arg(long)]
  pub id: u8
}

impl PBPowerOpts {
  pub fn new(status: PowerStatusEnum, id: u8) -> Self {
    Self { 
      status,
      id
    }
  }
}

#[derive(Debug, Args, PartialEq)]
pub struct RBPowerOpts {
  /// Which power status one wants to achieve
  #[arg(short, long)]
  pub status: PowerStatusEnum,
  /// ID of the RB to be powered up
  #[arg(short, long)]
  pub id: u8
}

impl RBPowerOpts {
  pub fn new(status: PowerStatusEnum, id: u8) -> Self {
    Self {
      status,
      id
    }
  }
}

#[derive(Debug, Args, PartialEq)]
pub struct LTBPowerOpts {
  /// Which power status one wants to achieve
  #[arg(value_parser = clap::builder::PossibleValuesParser::new([
          PowerStatusEnum::OFF,
          PowerStatusEnum::ON,
          PowerStatusEnum::Cycle
        ]),
        required = true)]
  pub status: PowerStatusEnum,
  /// ID of the LTB to be powered up
  #[arg(short, long, default_value_t = DEFAULT_LTB_ID)]
  pub id: u8
}

impl LTBPowerOpts {
  pub fn new(status: PowerStatusEnum, id: u8) -> Self {
    Self {
      status,
      id
    }
  }
}

#[derive(Debug, Args, PartialEq)]
pub struct PreampPowerOpts {
  /// Which power status one wants to achieve
  #[arg(value_parser = clap::builder::PossibleValuesParser::new([
          PowerStatusEnum::OFF,
          PowerStatusEnum::ON,
          PowerStatusEnum::Cycle
        ]),
        required = true)]
  pub status: PowerStatusEnum,
  /// ID of the preamp to be powered up
  #[arg(short, long, default_value_t = DEFAULT_PREAMP_ID)]
  pub id: u8,
  /// Turn on bias of the preamp specified
  #[arg(short, long, default_value_t = DEFAULT_PREAMP_BIAS)]
  pub bias: u16
}

impl PreampPowerOpts {
  pub fn new(status: PowerStatusEnum, id: u8, bias: u16) -> Self {
    Self {
      status,
      id,
      bias
    }
  }
}
/// END Power cmds ================================================

/// Run cmds ======================================================
#[derive(Debug, Clone, Subcommand, PartialEq)]
pub enum RunCmd {
  /// Start data taking
  Start(StartRunOpts),
  /// Stop data taking
  Stop(StopRunOpts)
}

#[derive(Debug, Clone, Args, PartialEq)]
pub struct StartRunOpts {
  /// Which kind of run is to be launched
  #[arg(short, long, default_value_t = DEFAULT_RUN_TYPE)]
  pub run_type: u8,
  /// ID of the RB where to run data taking
  #[arg(short, long, default_value_t = DEFAULT_RB_ID)]
  pub id: u8,
  /// Number of events in the run
  #[arg(short, long, default_value_t = DEFAULT_RUN_EVENT_NO)]
  pub no: u8
}

impl StartRunOpts {
  pub fn new(run_type: u8, id: u8, no: u8) -> Self {
    Self {
      run_type,
      id,
      no
    }
  }
}

#[derive(Debug, Clone, Args, PartialEq)]
pub struct StopRunOpts {
  /// ID of the RB where to run data taking
  #[arg(short, long, default_value_t = DEFAULT_RB_ID)]
  pub id: u8
}

impl StopRunOpts {
  pub fn new(id: u8) -> Self {
    Self {
      id
    }
  }
}
/// END Run cmds ==================================================

#[test]
fn test_display() {
  let rb = ReadoutBoard::default();
  println!("Readout board {}", rb);
  assert_eq!(1,1);
}


