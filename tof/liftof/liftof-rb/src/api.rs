//! Higher level functions, to deal with events/binary reprentation of it, 
//! configure the drs4, etc.
use std::net::IpAddr;
use local_ip_address::local_ip;
use std::fs::read_to_string;


use tof_dataclasses::serialization::Serialization;
cfg_if::cfg_if! {
  if #[cfg(feature = "tofcontrol")]  {
  use tof_dataclasses::calibrations::RBCalibrations;
  use tof_dataclasses::errors::{CalibrationError,
                                RunError,
                                SetError};
  }
}
use std::path::Path;
use std::time::{Duration,
                Instant};
use std::{thread, time};
use std::env;
use crossbeam_channel::{Sender};

use crate::control::*;
use crate::memory::*;

use tof_dataclasses::events::{RBEvent,
                              DataType};
use tof_dataclasses::commands::TofCommand;
use tof_dataclasses::packets::TofPacket;
use tof_dataclasses::monitoring::RBMoniData;
use tof_dataclasses::errors::SerializationError;
use tof_dataclasses::run::RunConfig;
use tof_dataclasses::serialization::get_json_from_file;

// Takeru's tof-control
cfg_if::cfg_if! {
  if #[cfg(feature = "tofcontrol")]  {
    // for calibration
    use tof_control::rb_control::rb_mode::{select_noi_mode,
                                          select_vcal_mode,
                                          select_tcal_mode,
                                          select_sma_mode};
    // for threshold setting
    use tof_control::preamp_control::preamp_bias;

    const FIVE_SECONDS: Duration = time::Duration::from_millis(5000);
  }
}

/// The poisson self trigger mode of the board
/// triggers automatically, this means we don't 
/// have to send a forced trigger signal every
/// 1/rate.
///
/// It just sets the respective registers here
pub fn enable_poisson_self_trigger(rate : f32) {
  // we have to calculate the actual rate with Andrew's formula
  //let clk_period : f64 = 1.0/33e6;
  let max_val  : f32 = 4294967295.0;
  let reg_val = (rate/(33e6/max_val)) as u32;
  info!("Will use random self trigger with rate {reg_val} value for register, corresponding to {rate} Hz");
  match set_self_trig_rate(reg_val) {
    Err(err) => {
      error!("Setting self trigger failed! Er {err}");
      error!("To be clear, we are NOT RUNNING IN POISSON SELF-TRIGGER MODE!");
    }
    Ok(_)    => ()
  }
}


/// Wait as long as a run is active.
/// This call blocks the current thread 
/// until no run is active anymore.
///
/// Check the trigger enabled register
/// periodically to find out whether
/// a run is active or not.
///
/// if n_errors is reached, decide the
/// run to be inactive
///
/// # Arguments
///
/// * n_errors     : Unforgiveable number of errors
///                  when querying the trigger status
///                  register. If reached, break.
/// * interval     : Check the trigger register every
///                  interval
/// * n_events_exp : Don't return before we have seen
///                  this many events
pub fn wait_while_run_active(n_errors     : u32,
                             interval     : Duration,
                             n_events_exp : u32,
                             data_type    : &DataType,
                             socket       : &zmq::Socket) -> Vec<RBEvent> {
  // check if we are done
  let mut events = Vec::<RBEvent>::new();
  let mut errs : u32 = 0;
  let start = Instant::now();
  let mut triggers_have_stopped = false;
  loop {
    // listen to zmq here
    match socket.recv_bytes(0) {
      Err(err) => {
        error!("Unable to recv on socket! Err {err}");
      },
      Ok(bytes) => {
        // the first 5 bytes are the identifier, in this case
        // LOCAL
        match TofPacket::from_bytestream(&bytes, &mut 5) {
          Err(err) => {
            error!("Can't unpack TofPacket, err {err}");
          },
          Ok(tp) => {
            match RBEvent::from_bytestream(&tp.payload, &mut 0) {
              Err(err) => {
                error!("Can't unpack RBEvent, error {err}");
              },
              Ok(ev) => {
                if ev.data_type == *data_type {
                  events.push(ev);
                }
              }
            }
          }
        }
      }
    }
    if events.len() >= n_events_exp as usize {
      info!("Acquired {} events!", events.len());
      return events;
    }
    if triggers_have_stopped {
      continue;
    }
    if start.elapsed() > interval {
      match get_triggers_enabled() {
        Err(err) => {
          error!("Unable to obtain trigger status! Err {err}");
          errs += 1;
        },
        Ok(running) => {
          if !running {
            info!("Run has apparently terminated!");
            triggers_have_stopped = true;
            //break;
          } else { 
            info!("We have waited the expected time, but there are still triggers...");
            thread::sleep(interval);
          }
        }
      }
      //thread::sleep(interval);
      if errs == n_errors {
        info!("Can't wait anymore since we have seen the configured number of errors! {n_errors}");
        return events;
      }
    //start = Instant::now();
    }
  }
}

// START Calibration stuff ====================================================
// eventually, we have to rename that feature
/// A full set of RB calibration
///
/// This includes
/// - take voltage calbration data, 
///   1000 events, save to disk, but 
///   keep in memory
/// - take timing calibration data,
///   1000 events, save to disk but 
///   keep in memory
/// - no input data, 1000 events, save
///   to disk but keep in memory
/// - apply calibration script (Jamie)
///   save result in binary and in textfile,
///   send downstream
#[cfg(feature="tofcontrol")]
pub fn rb_calibration(rc_to_runner    : &Sender<RunConfig>,
                      tp_to_publisher : &Sender<TofPacket>)
-> Result<(), CalibrationError> {
  warn!("Commencing full RB calibration routine! This will take the board out of datataking for a few minutes!");
  // TODO this should become something that can be read from a local json file
  let mut run_config = RunConfig {
    nevents                 : 1300,
    is_active               : true,
    nseconds                : 0,
    stream_any              : true,
    trigger_poisson_rate    : 0,
    trigger_fixed_rate      : 100,
    latch_to_mtb            : false,
    data_type               : DataType::Unknown,
    rb_buff_size            : 1000
  }; 
  let socket = connect_to_zmq().expect("Not able to connect to socket, something REAL strange happened.");

  let mut board_id = 0u8;
  match get_board_id() {
    Err(err) => {
      error!("Unable to obtain board id. Calibration might be orphaned. Err {err}");
    },
    Ok(rb_id) => {
      board_id = rb_id as u8;
    }
  }
  let mut calibration = RBCalibrations::new(board_id);
  calibration.serialize_event_data = true;

  run_config.data_type = DataType::Noi; 
  run_noi_calibration(rc_to_runner, &socket, &mut calibration, run_config);

  run_config.data_type = DataType::VoltageCalibration; 
  run_voltage_calibration(rc_to_runner, &socket, &mut calibration, run_config);
  
  run_config.data_type = DataType::TimingCalibration;
  run_timing_calibration(rc_to_runner, &socket, &mut calibration, run_config);

  println!("==> Calibration data taking complete!"); 
  println!("Calibration : {}", calibration);
  println!("Cleaning data...");
  calibration.clean_input_data();
  println!("Calibration : {}", calibration);

  info!("Will set board to sma mode!");
  select_sma_mode();
  run_config.is_active = false;  
  match rc_to_runner.send(run_config) {
    Err(err) => warn!("Can not send runconfig!, Err {err}"),
    Ok(_)    => trace!("Success!")
  }
  thread::sleep(FIVE_SECONDS);

  // Do this only with the full calib
  calibration.calibrate()?;
  println!("Calibration : {}", calibration);

  // Send it
  let calib_pack = TofPacket::from(&calibration);
  match tp_to_publisher.send(calib_pack) {
    Err(err) => {
      error!("Unable to send RBCalibration package! Error {err}");
    },
    Ok(_) => ()
  }
  info!("Calibration done!");
  Ok(())
}

// TODO The following two functions are placeholder for subset of the
// calibration routine. It is not clear whether these make sense or not.
//
// Only no input and publish.
#[cfg(feature="tofcontrol")]
pub fn rb_noi_subcalibration(rc_to_runner    : &Sender<RunConfig>,
                             tp_to_publisher : &Sender<TofPacket>)
-> Result<(), CalibrationError> {
  warn!("Commencing RB No input sub-calibration routine! This will take the board out of datataking for a few minutes!");
  // TODO this should become something that can be read from a local json file
  let mut run_config = RunConfig {
    nevents                 : 1300,
    is_active               : true,
    nseconds                : 0,
    stream_any              : true,
    trigger_poisson_rate    : 0,
    trigger_fixed_rate      : 100,
    latch_to_mtb            : false,
    data_type               : DataType::Unknown,
    rb_buff_size            : 1000
  }; 
  let socket = connect_to_zmq().expect("Not able to connect to socket, something REAL strange happened.");

  let mut board_id = 0u8;
  match get_board_id() {
    Err(err) => {
      error!("Unable to obtain board id. Calibration might be orphaned. Err {err}");
    },
    Ok(rb_id) => {
      board_id = rb_id as u8;
    }
  }
  let mut calibration = RBCalibrations::new(board_id);
  calibration.serialize_event_data = true;

  run_config.data_type = DataType::Noi; 
  run_noi_calibration(rc_to_runner, &socket, &mut calibration, run_config);

  println!("==> No input data taking complete!"); 
  println!("Calibration : {}", calibration);
  println!("Cleaning data...");
  calibration.clean_input_data();
  println!("Calibration : {}", calibration);

  info!("Will set board to sma mode!");
  select_sma_mode();
  run_config.is_active = false;  
  match rc_to_runner.send(run_config) {
    Err(err) => warn!("Can not send runconfig!, Err {err}"),
    Ok(_)    => trace!("Success!")
  }
  thread::sleep(FIVE_SECONDS);

  println!("Calibration won't start cause the calibration datataking chain is not complete!");

  // Send it
  let calib_pack = TofPacket::from(&calibration);
  match tp_to_publisher.send(calib_pack) {
    Err(err) => {
      error!("Unable to send RBCalibration package! Error {err}");
    },
    Ok(_) => ()
  }
  info!("Calibration done!");
  Ok(())
}

// Noi -> Voltage chain and publish.
#[cfg(feature="tofcontrol")]
pub fn rb_noi_voltage_subcalibration(rc_to_runner    : &Sender<RunConfig>,
                                     tp_to_publisher : &Sender<TofPacket>,
                                     voltage_level   : u16) // where do we put this bad boi?
-> Result<(), CalibrationError> {
  warn!("Commencing RB no input + voltage sub-calibration routine! This will take the board out of datataking for a few minutes!");
  // TODO this should become something that can be read from a local json file
  let mut run_config = RunConfig {
    nevents                 : 1300,
    is_active               : true,
    nseconds                : 0,
    stream_any              : true,
    trigger_poisson_rate    : 0,
    trigger_fixed_rate      : 100,
    latch_to_mtb            : false,
    data_type               : DataType::Unknown,
    rb_buff_size            : 1000
  }; 
  let socket = connect_to_zmq().expect("Not able to connect to socket, something REAL strange happened.");

  let mut board_id = 0u8;
  match get_board_id() {
    Err(err) => {
      error!("Unable to obtain board id. Calibration might be orphaned. Err {err}");
    },
    Ok(rb_id) => {
      board_id = rb_id as u8;
    }
  }
  let mut calibration = RBCalibrations::new(board_id);
  calibration.serialize_event_data = true;

  run_config.data_type = DataType::Noi; 
  run_noi_calibration(rc_to_runner, &socket, &mut calibration, run_config);

  run_config.data_type = DataType::VoltageCalibration; 
  run_voltage_calibration(rc_to_runner, &socket, &mut calibration, run_config);

  println!("==> No input + voltage data taking complete!"); 
  println!("Calibration : {}", calibration);
  println!("Cleaning data...");
  calibration.clean_input_data();
  println!("Calibration : {}", calibration);

  info!("Will set board to sma mode!");
  select_sma_mode();
  run_config.is_active = false;  
  match rc_to_runner.send(run_config) {
    Err(err) => warn!("Can not send runconfig!, Err {err}"),
    Ok(_)    => trace!("Success!")
  }
  thread::sleep(FIVE_SECONDS);

  println!("Calibration won't start cause the calibration datataking chain is not complete!");

  // Send it
  let calib_pack = TofPacket::from(&calibration);
  match tp_to_publisher.send(calib_pack) {
    Err(err) => {
      error!("Unable to send RBCalibration package! Error {err}");
    },
    Ok(_) => ()
  }
  info!("Calibration done!");
  Ok(())
}

// Noi -> Voltage -> Timing chain and publish (no calib!).
#[cfg(feature="tofcontrol")]
pub fn rb_timing_subcalibration(rc_to_runner    : &Sender<RunConfig>,
                                tp_to_publisher : &Sender<TofPacket>,
                                voltage_level   : u16)
-> Result<(), CalibrationError> {
  warn!("Commencing RB no input + voltage + timing sub-calibration routine! This will take the board out of datataking for a few minutes!");
  // TODO this should become something that can be read from a local json file
  let mut run_config = RunConfig {
    nevents                 : 1300,
    is_active               : true,
    nseconds                : 0,
    stream_any              : true,
    trigger_poisson_rate    : 0,
    trigger_fixed_rate      : 100,
    latch_to_mtb            : false,
    data_type               : DataType::Unknown,
    rb_buff_size            : 1000
  }; 
  let socket = connect_to_zmq().expect("Not able to connect to socket, something REAL strange happened.");

  let mut board_id = 0u8;
  match get_board_id() {
    Err(err) => {
      error!("Unable to obtain board id. Calibration might be orphaned. Err {err}");
    },
    Ok(rb_id) => {
      board_id = rb_id as u8;
    }
  }
  let mut calibration = RBCalibrations::new(board_id);
  calibration.serialize_event_data = true;

  run_config.data_type = DataType::Noi; 
  run_noi_calibration(rc_to_runner, &socket, &mut calibration, run_config);

  run_config.data_type = DataType::VoltageCalibration; 
  run_voltage_calibration(rc_to_runner, &socket, &mut calibration, run_config);
  
  run_config.data_type = DataType::TimingCalibration;
  run_timing_calibration(rc_to_runner, &socket, &mut calibration, run_config);

  println!("==> No input + voltage + timing data taking complete!"); 
  println!("Calibration : {}", calibration);
  println!("Cleaning data...");
  calibration.clean_input_data();
  println!("Calibration : {}", calibration);

  info!("Will set board to sma mode!");
  select_sma_mode();
  run_config.is_active = false;  
  match rc_to_runner.send(run_config) {
    Err(err) => warn!("Can not send runconfig!, Err {err}"),
    Ok(_)    => trace!("Success!")
  }
  thread::sleep(FIVE_SECONDS);

  println!("Calibration won't start. The data taking chain is complete, but a sub-calibration routine was called!");

  // Send it
  let calib_pack = TofPacket::from(&calibration);
  match tp_to_publisher.send(calib_pack) {
    Err(err) => {
      error!("Unable to send RBCalibration package! Error {err}");
    },
    Ok(_) => ()
  }
  info!("Calibration done!");
  Ok(())
}

#[cfg(feature="tofcontrol")]
fn connect_to_zmq() -> Result<zmq::Socket, CalibrationError> {
  // here is the general idea. We connect to our own 
  // zmq socket, to gather the events and store them 
  // here locally. Then we apply the calibration 
  // and we simply have to send it back to the 
  // data publisher.
  // This saves us a mutex!!
  let this_board_ip = local_ip().expect("Unable to obtain local board IP. Something is messed up!");
  let data_address = liftof_lib::build_tcp_from_ip(this_board_ip.to_string(), DATAPORT.to_string());

  let ctx = zmq::Context::new();
  let socket : zmq::Socket; 
  match ctx.socket(zmq::SUB) {
    Err(err) => {
      error!("Unable to create zmq socket! Err {err}. This is BAD!");
      return Err(CalibrationError::CanNotConnectToMyOwnZMQSocket);
    }
    Ok(sock) => {
      socket = sock;
    }
  }
  match socket.connect(&data_address) {
    Err(err) => {
      error!("Unable to connect to data (PUB) socket {data_address}, Err {err}");
      return Err(CalibrationError::CanNotConnectToMyOwnZMQSocket);
    },
    Ok(_) => ()
  }
  
  // The packets relevant for us here in this context, will 
  // all be prefixed with "LOCAL"
  // See the respective section in data_publisher 
  // (search for prefix_local)
  let topic_local = String::from("LOCAL");
  match socket.set_subscribe(&topic_local.as_bytes()) {
    Err(err) => error!("Can not subscribe to {topic_local}, err {err}"),
    Ok(_)    => info!("Subscribing to local packages!"),
  }
  Ok(socket)
}

#[cfg(feature="tofcontrol")]
fn run_noi_calibration(rc_to_runner: &Sender<RunConfig>,
                       socket: &zmq::Socket,
                       calibration: &mut RBCalibrations,
                       run_config: RunConfig)
                       -> Result<(), CalibrationError> {
  info!("Will set board to no input mode!");
  select_noi_mode();
  match rc_to_runner.send(run_config) {
    Err(err) => warn!("Can not send runconfig!, Err {err}"),
    Ok(_)    => trace!("Success!")
  }
  let cal_dtype = DataType::Noi;
  calibration.noi_data = wait_while_run_active(10, FIVE_SECONDS, 1000, &cal_dtype, socket);
  println!("==> No input (Voltage calibration) data taken!");
  Ok(())
}

#[cfg(feature="tofcontrol")]
fn run_voltage_calibration(rc_to_runner: &Sender<RunConfig>,
                           socket: &zmq::Socket,
                           calibration: &mut RBCalibrations,
                           run_config: RunConfig)
                           -> Result<(), CalibrationError> {
  info!("Will set board to vcal mode!");
  select_vcal_mode(); 
  match rc_to_runner.send(run_config) {
    Err(err) => warn!("Can not send runconfig!, Err {err}"),
    Ok(_)    => trace!("Success!")
  }  
  let cal_dtype             = DataType::VoltageCalibration;
  calibration.vcal_data = wait_while_run_active(10, FIVE_SECONDS, 1000, &cal_dtype, socket);
  println!("==> Voltage calibration data taken!");
  Ok(())
}

#[cfg(feature="tofcontrol")]
fn run_timing_calibration(rc_to_runner: &Sender<RunConfig>,
                          socket: &zmq::Socket,
                          calibration: &mut RBCalibrations,
                          mut run_config: RunConfig)
                          -> Result<(), CalibrationError> {
  info!("Will set board to tcal mode!");
  run_config.trigger_poisson_rate  = 80;
  run_config.nevents               = 1800; // make sure we get 1000 events
  run_config.trigger_fixed_rate    = 0;
  //run_config.rb_buff_size          = 500;
  select_tcal_mode();
  match rc_to_runner.send(run_config) {
    Err(err) => warn!("Can not send runconfig!, Err {err}"),
    Ok(_)    => trace!("Success!")
  }
  let cal_dtype = DataType::TimingCalibration;
  calibration.tcal_data = wait_while_run_active(10, FIVE_SECONDS, 1000,&cal_dtype, socket);
  println!("==> Timing calibration data taken!");
  Ok(())
}
// END Calibration stuff ======================================================

// BEGIN Run stuff ============================================================
#[cfg(feature="tofcontrol")]
pub fn rb_start_run(rc_to_runner    : &Sender<RunConfig>,
                    rc_config       : RunConfig,
                    run_type        : u8,
                    rb_id           : u8,
                    event_no        : u8,
                    time            : u8) -> Result<(), RunError> {
  println!("==> Will initialize new run!");
  match rc_to_runner.send(rc_config) {
    Err(err) => error!("Error initializing run! {err}"),
    Ok(_)    => ()
  };
  println!("==> Run successfully started!");
  Ok(())
}

#[cfg(feature="tofcontrol")]
pub fn rb_stop_run(rc_to_runner    : &Sender<RunConfig>,
                   rb_id           : u8) -> Result<(), RunError> {
  println!("==> Will initialize new run!");
  println!("Received command to end run!");
  // default is not active for run config

  let  rc = RunConfig::new();
  match rc_to_runner.send(rc) {
    Err(err) => error!("Error stopping run! {err}"),
    Ok(_)    => ()
  }
  println!("==> Run successfully stopped!");
  Ok(())
}
// END Run stuff ==============================================================

const DMA_RESET_TRIES : u8 = 10;   // if we can not reset the DMA after this number
                                   // of retries, we'll panic!


// Using the same approach as the flight computer, we use
// two ports for communication/data
// 1) PUB for the data
// 2) SUB for the commands.
// - _A comment here_ while we usually would prefer REP?REQ for 
// comms, this will avoid deadlocks in any case and makes it in 
// general much easier for command servers to connect to the boards.

/// Dataport is 0MQ PUB for publishing waveform/event data
pub const DATAPORT : u32 = 42000;

// FIXME
type RamBuffer = BlobBuffer;

/// Check for the environmental 
/// variable LIFTOF_IS_SYSTEMD
/// which is set in the liftof.service file
/// to determine wether liftof is executed 
/// through systemd.
///
/// WARN - this is not elegant, but all other
/// approaches did not work!
pub fn is_systemd_process() -> bool {
  // this custom variable must be set in the 
  // liftof.service file!!
  if env::var("LIFTOF_IS_SYSTEMD").is_ok() {
    info!("Running under systemd");
    true
  } else {
    info!("Not running under systemd");
    false
  }
}

/// Get a runconfig from a file. 
///
/// FIXME - panics...
pub fn get_runconfig(rcfile : &Path) -> RunConfig {
  //match get_json_from_file(rcfile) {
  match read_to_string(rcfile) {
    Err(err) => {
      panic!("Unable to read the configuration file! Error {err}");
    }
    Ok(rc_from_file) => {
      println!("==> Found configuration file {}!", rcfile.display());
      match serde_json::from_str(&rc_from_file) {
        Err(err) => panic!("Can not read json from configuration file. Error {err}"),
        Ok(rc_json) => {
          rc_json
        }
      }
    }
  }
}

/// Get the active half of the RAM buffer
/// 
/// This uses the know regions of the RAM 
/// buffers together with the dma pointer
/// to get the correct half.
///
pub fn get_active_buffer() -> Result<RamBuffer, RegisterError> {
  let dma_ptr = get_dma_pointer()?;
  if dma_ptr >= UIO1_MAX_OCCUPANCY {
    return Ok(RamBuffer::B);
  }
  Ok(RamBuffer::A)
}

/// Add the prefix "LOCAL" to a bytestream.
///
/// This will allow for the central C&C server 
/// to ignore this packet, but the board can 
/// still send it to itself
pub fn prefix_local(input : &mut Vec<u8>) -> Vec<u8> {
  let mut bytestream : Vec::<u8>;
  let local = String::from("LOCAL");
  bytestream = local.as_bytes().to_vec();
  bytestream.append(input);
  bytestream
}

/// add the board id to the bytestream in front of the 
/// tof response
pub fn prefix_board_id(input : &mut Vec<u8>) -> Vec<u8> {
  // FIUXME - this should not panic
  let board_id = get_board_id()//
                 .unwrap_or(0);
                               //.expect("Need to be able to obtain board id!");
  let mut bytestream : Vec::<u8>;
  let board : String;
  if board_id < 10 {
    board = String::from("RB0") + &board_id.to_string();
  } else {
    board = String::from("RB")  + &board_id.to_string();
  }
  //let mut response = 
  bytestream = board.as_bytes().to_vec();
  //bytestream.append(&mut resp.to_bytestream());
  bytestream.append(input);
  bytestream
}


/// strip of the first 4 bytes of the incoming vector 
pub fn cmd_from_bytestream(bytestream : &mut Vec<u8>) ->Result<TofCommand, SerializationError>{
  //let bytestream = cmd.drain(0..4);
  // FIXME - remove expect call
  TofCommand::from_bytestream(&bytestream, &mut 4)
  //tof_command
}





/// Reset DMA pointer and buffer occupancy registers
///
/// If there are any errors, we will wait for a short
/// time and then try again
/// FIXME - this should return Result
pub fn reset_dma_and_buffers() {
  // register writing is on the order of microseconds 
  // (MHz clock) so one_milli is plenty
  let one_milli   = time::Duration::from_millis(1);
  let buf_a = BlobBuffer::A;
  let buf_b = BlobBuffer::B;
  let mut n_tries = 0u8;
  let mut failed  = true;
  loop {
    if failed && n_tries < DMA_RESET_TRIES {
      match reset_dma() {
        Ok(_)    => (),
        Err(err) => {
          error!("Resetting dma failed, err {:?}", err);
          n_tries += 1;
          thread::sleep(one_milli);
          continue;
        }
      } 
      match reset_ram_buffer_occ(&buf_a) {
        Ok(_)    => (), 
        Err(err) => {
          error!("Problem resetting buffer /dev/uio1 {:?}", err);
          n_tries += 1;
          thread::sleep(one_milli);
          continue;
        }
      }
      match reset_ram_buffer_occ(&buf_b) {
        Ok(_)    => (), 
        Err(err) => {
          error!("Problem resetting buffer /dev/uio2 {:?}", err);
          n_tries += 1;
          thread::sleep(one_milli);
          continue;
        }
      }
    failed = false;      
    } else {
      break;
    }
  }
  // in any case, relax a bit
  thread::sleep(10*one_milli);
}

/// Check if the buffers are actually filling
/// 
///  - if not, panic. We can't go on like that
pub fn run_check() {
  let buf_a = BlobBuffer::A;
  let buf_b = BlobBuffer::B;

  let interval = Duration::from_secs(5);
  let mut n_iter = 0;
  
  let mut last_occ_a = get_blob_buffer_occ(&buf_a).unwrap();
  let mut last_occ_b = get_blob_buffer_occ(&buf_b).unwrap();
  match enable_trigger() {
    Err(err) => error!("Unable to enable trigger! Err {err}"),
    Ok(_)    => info!("Triggers enabled")
  }
  loop {
    n_iter += 1;
    thread::sleep(interval);
    let occ_a = get_blob_buffer_occ(&buf_a).unwrap();
    let occ_b = get_blob_buffer_occ(&buf_b).unwrap();
    if occ_a - last_occ_a == 0 && occ_b - last_occ_b == 0 {
      panic!("We did not observe a change in occupancy for either one of the buffers!");
    }
    println!("-- buff size A {}", occ_a - last_occ_a);
    println!("-- buff size B {}", occ_b - last_occ_b);
    println!("---> Iter {n_iter}");
    last_occ_a = occ_a;
    last_occ_b = occ_b;
  }
}

///  Get the blob buffer size from occupancy register
///
///  Read out the occupancy register and compare to 
///  a previously recoreded value. 
///  Everything is u32 (the register can't hold more)
///
///  The size of the buffer can only be defined compared
///  to a start value. If the value rools over, the 
///  size then does not make no longer sense and needs
///  to be updated.
///
///  #Arguments: 
///
pub fn get_buff_size(which : &BlobBuffer) ->Result<usize, RegisterError> {
  let size : u32;
  let occ = get_blob_buffer_occ(&which)?;
  trace!("Got occupancy of {occ} for buff {which:?}");

  // the buffer sizes is UIO1_MAX_OCCUPANCY -  occ
  match which {
    BlobBuffer::A => {size = occ - UIO1_MIN_OCCUPANCY;},
    BlobBuffer::B => {size = occ - UIO2_MIN_OCCUPANCY;}
  }
  let result = size as usize;
  Ok(result)
}

/// Manage the RAM buffers for event data
///
/// This will make a decision based on the 
/// buff_trip value if a buffer is "full", 
/// and in that case, read it out, send 
/// the data over the channel elsewhere 
/// and switch to the other half of the 
/// buffer.
/// If buff_trip == DATABUF_TOTAL_SIZE, the 
/// buffer will be switched by the firmware.
///
/// # Arguments:
///
/// * buff_trip : size which triggers buffer readout.
pub fn ram_buffer_handler(buff_trip     : usize,
                          bs_sender     : &Sender<Vec<u8>>)
    -> Result<(RamBuffer, usize), RegisterError> {
  let mut switch_buff = false;
  if buff_trip < DATABUF_TOTAL_SIZE {
    switch_buff = true;
  }

  let which          = get_active_buffer()?;
  let mut buff_size  = get_buff_size(&which)?;
  if buff_size >= buff_trip {
    info!("Buff {which:?} tripped at a size of {buff_size}");  
    debug!("Buff handler switch buffers {switch_buff}");
    // 1) switch buffer
    // 2) read out
    // 3) reset
    if switch_buff {
      match switch_ram_buffer() {
        Ok(_)  => {
          info!("Ram buffer switched!");
        },
        Err(_) => error!("Unable to switch RAM buffers!") 
      }
    }
    let mut bytestream = Vec::<u8>::new(); 
    match read_data_buffer(&which, buff_size as usize) {
      Err(err) => error!("Can not read data buffer {err}"),
      Ok(bs)    => bytestream = bs,
    }
    let bs_len = bytestream.len();
    match bs_sender.send(bytestream) {
      Err(err) => error!("error sending {err}"),
      Ok(_)    => {
        info!("We are sending {} event bytes for further processing!", bs_len);
      }
    }
    match reset_ram_buffer_occ(&which) {
      Ok(_)  => debug!("Successfully reset the buffer occupancy value"),
      Err(_) => error!("Unable to reset buffer!")
    }
    buff_size = 0;
  }
  Ok((which, buff_size))
}

///  Prepare the whole readoutboard for data taking.
///
///  This sets up the drs4 and c
///  lears the memory of 
///  the data buffers.
///  
///  This will make several writes to the /dev/uio0
///  memory map.
pub fn setup_drs4() -> Result<(), RegisterError> {

  let buf_a = BlobBuffer::A;
  let buf_b = BlobBuffer::B;

  let one_milli   = time::Duration::from_millis(1);
  // DAQ defaults
  //let num_samples     : u32 = 3000;
  //let duration        : u32 = 0; // Default is 0 min (=> use events) 
  //let roi_mode        : u32 = 1;
  //let transp_mode     : u32 = 1;
  //let run_mode        : u32 = 0;
  //let run_type        : u32 = 0;        // 0 -> Events, 1 -> Time (default is Events)
  //let config_drs_flag : u32 = 1; // By default, configure the DRS chip
  // run mode info
  // 0 = free run (must be manually halted), ext. trigger
  // 1 = finite sample run, ext. trigger
  // 2 = finite sample run, software trigger
  // 3 = finite sample run, software trigger, random delays/phase for timing calibration
  let spike_clean     : bool = true;
  //let read_ch9        : u32  = 1;

  // before we do anything, set the DRS in idle mode 
  // and set the configure bit
  //idle_drs4_daq()?;
  //thread::sleep(one_milli);
  //set_drs4_configure()?;
  //thread::sleep(one_milli);

  // Sanity checking
  //let max_samples     : u32 = 65000;
  //let max_duration    : u32 = 1440; // Minutes in 1 day

  //reset_daq()?;
  //thread::sleep(one_milli);
  //reset_drs()?;
  //thread::sleep(one_milli);
  //reset_dma()?;
  //thread::sleep(one_milli);
  clear_dma_memory()?;
  thread::sleep(one_milli);
  
  
  // for some reason, sometimes it 
  // takes a bit until the blob
  // buffers reset. Let's try a 
  // few times
  info!("Resetting event memory buffers..");
  for _ in 0..5 {
    reset_ram_buffer_occ(&buf_a)?;
    thread::sleep(one_milli);
    reset_ram_buffer_occ(&buf_b)?;
    thread::sleep(one_milli);
  }

  // register 04 contains a lot of stuff:
  // roi mode, busy, adc latency
  // sample  count and spike removal
  let spike_clean_enable : u32 = 4194304; //bit 22
  if spike_clean {
    let mut value = read_control_reg(0x40).unwrap();  
    value = value | spike_clean_enable;
    write_control_reg(0x40, value)?;
    thread::sleep(one_milli);
  }
 
  // we don't want to do that anymore
  //set_readout_all_channels_and_ch9()?;
  thread::sleep(one_milli);
  set_master_trigger_mode()?;
  thread::sleep(one_milli);
  Ok(())
}

#[cfg(feature = "tofcontrol")]
pub fn send_preamp_bias_set(preamp_id: u8, bias_voltage: u16) -> Result<(), SetError> {
  preamp_bias::PreampBiasSet::set_bias_manual_id(preamp_id, bias_voltage as f32);
  Ok(())
}