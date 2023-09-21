//! Higher level functions, to deal with events/binary reprentation of it, 
//! configure the drs4, etc.

use local_ip_address::local_ip;
use tof_dataclasses::serialization::Serialization;
use tof_dataclasses::errors::CalibrationError;
#[cfg(feature="tofcontrol")]
use tof_dataclasses::calibrations::RBCalibrations;

use std::collections::HashMap;
use std::net::IpAddr;
use std::fs::File;
use std::path::Path;
use std::io::Write;
use std::fs::OpenOptions;
use std::fs;
use std::time::{Duration,
                Instant};
use std::ffi::OsString;
use std::{thread, time};
use std::env;
use crossbeam_channel::{Sender,
                        Receiver};

// just for fun
use indicatif::{MultiProgress,
                ProgressBar,
                ProgressStyle};

use crate::control::*;
use crate::memory::*;



use tof_dataclasses::commands::*;
use tof_dataclasses::events::{RBEventPayload,
                              RBEventHeader,
                              RBEvent,
                              RBEventMemoryView,
                              DataType,
                              DataFormat};
use tof_dataclasses::serialization::search_for_u16;
use tof_dataclasses::commands::{TofCommand,
                                TofResponse,
                                TofOperationMode};
use tof_dataclasses::packets::{TofPacket,
                               PacketType};
use tof_dataclasses::monitoring::RBMoniData;
use tof_dataclasses::errors::SerializationError;
use tof_dataclasses::run::RunConfig;
use tof_dataclasses::serialization::get_json_from_file;

// Takeru's tof-control
#[cfg(feature="tofcontrol")]
use tof_control::rb_control::rb_temp::RBtemp;
#[cfg(feature="tofcontrol")]
use tof_control::rb_control::rb_mag::RBmag;
#[cfg(feature="tofcontrol")]
use tof_control::rb_control::rb_vcp::RBvcp;
#[cfg(feature="tofcontrol")]
use tof_control::rb_control::rb_ph::RBph;

// for calibration
#[cfg(feature="tofcontrol")]
use tof_control::rb_control::rb_mode::{select_noi_mode,
                                       select_vcal_mode,
                                       select_tcal_mode,
                                       select_sma_mode};


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
/// periodically to find out wether
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
  let five_seconds   = time::Duration::from_millis(5000);
  let mut run_config = RunConfig {
    nevents                 : 1300,
    is_active               : true,
    nseconds                : 0,
    stream_any              : true,
    trigger_poisson_rate    : 0,
    trigger_fixed_rate      : 100,
    latch_to_mtb            : false,
    active_channel_mask     : 255,
    data_type               : DataType::Noi,
    data_format             : DataFormat::Default,
    rb_buff_size            : 1000
  }; 
  // here is the general idea. We connect to our own 
  // zmq socket, to gather the events and store them 
  // here locally. Then we apply the calibration 
  // and we simply have to send it back to the 
  // data publisher.
  // This saves us a mutex!!
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

  // set up zmq socket
  let mut address_ip = String::from("tcp://");
  let this_board_ip = local_ip().expect("Unable to obtainl local board IP. Something is messed up!");
  let data_port    = DATAPORT;

  match this_board_ip {
    IpAddr::V4(ip) => address_ip += &ip.to_string(),
    IpAddr::V6(_)  => panic!("Currently, we do not support IPV6!")
  }
  let data_address : String = address_ip.clone() + ":" + &data_port.to_string();

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
  // at this point, the zmq socket should be set up!
  info!("Will set board to no input mode!");
  select_noi_mode();
  match rc_to_runner.send(run_config) {
    Err(err) => warn!("Can not send runconfig!, Err {err}"),
    Ok(_)    => trace!("Success!")
  }
  let mut cal_dtype = DataType::Noi;
  calibration.noi_data = wait_while_run_active(10, five_seconds, 1000, &cal_dtype, &socket);
  println!("==> No input (Voltage calibration) data taken!");

  info!("Will set board to vcal mode!");
  select_vcal_mode();
  run_config.data_type = DataType::VoltageCalibration;  
  match rc_to_runner.send(run_config) {
    Err(err) => warn!("Can not send runconfig!, Err {err}"),
    Ok(_)    => trace!("Success!")
  }  
  cal_dtype             = DataType::VoltageCalibration;
  calibration.vcal_data = wait_while_run_active(10, five_seconds, 1000, &cal_dtype, &socket);
  
  println!("==> Voltage calibration data taken!");
  info!("Will set board to tcal mode!");
  run_config.trigger_poisson_rate  = 80;
  run_config.nevents               = 1800; // make sure we get 1000 events
  run_config.trigger_fixed_rate    = 0;
  //run_config.rb_buff_size          = 500;
  run_config.data_type = DataType::TimingCalibration;  
  select_tcal_mode();
  match rc_to_runner.send(run_config) {
    Err(err) => warn!("Can not send runconfig!, Err {err}"),
    Ok(_)    => trace!("Success!")
  }
  
  cal_dtype             = DataType::TimingCalibration;
  calibration.tcal_data = wait_while_run_active(10, five_seconds, 1000,&cal_dtype, &socket);
  println!("==> Timing calibration data taken!");
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
  thread::sleep(five_seconds);
  calibration.calibrate()?;
  println!("Calibration : {}", calibration);
  // now it just needs to be send to 
  // the publisher
  //for k in 0..10 {
  //  println!("cali vcal  {}", calibration.v_offsets[0][k]);
  //  println!("cali vincs {}", calibration.v_inc[0][k]);
  //  println!("cali vdips {}", calibration.v_dips[0][k]);
  //  println!("cali tbins {}", calibration.tbin[0][k]);
  //}
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


// this is just used for the testing case
fn find_missing_elements(nums: &[u32]) -> Vec<u32> {
  let mut missing_elements = Vec::new();
  let mut expected = nums[0];

  for &num in nums {
      while expected < num {
          missing_elements.push(expected);
          expected += 1;
      }

      if expected == num {
          expected += 1;
      }
  }
  missing_elements
}

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
  match get_json_from_file(rcfile) {
    Err(err) => {
      panic!("Unable to read the configuration file! Error {err}");
    }
    Ok(rc_from_file) => {
      println!("==> Found configuration file {}!", rcfile.display());
      println!("==> [WARN] - Currently, only the active channel mask will be parsed from the config file!");
      println!("==> [WARN/TODO] - This is WORK-IN-PROGRESS!");
      match RunConfig::from_json(&rc_from_file) {
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

/// Centrailized command management
/// 
/// Maintain 0MQ command connection and faciliate 
/// forwarding of commands and responses
///
/// # Arguments
///
/// cmd_server_ip             : The IP addresss of the C&C server we are listening to.
/// heartbeat_timeout_seconds : If we don't hear from the C&C server in this amount of 
///                             seconds, we try to reconnect.
pub fn cmd_responder(cmd_server_ip             : String,
                     heartbeat_timeout_seconds : u32,
                     //rsp_receiver              : &Receiver<TofResponse>,
                     run_config_file           : &Path,
                     run_config                : &Sender<RunConfig>,
                     evid_to_cache             : &Sender<u32>) {
                     //cmd_sender   : &Sender<TofCommand>) {
  // create 0MQ sockedts
  //let one_milli       = time::Duration::from_millis(1);
  let cmd_address = String::from("tcp://") + &cmd_server_ip + ":" + &DATAPORT.to_string() ;
  // we will subscribe to two types of messages, BDCT and RB + 2 digits 
  // of board id
  let topic_board = get_board_id().expect("Can not get board id!")
                    .to_string();
  let topic_broadcast = String::from("BRCT");
  let ctx = zmq::Context::new();
  let cmd_socket = ctx.socket(zmq::SUB).expect("Unable to create 0MQ SUB socket!");
  info!("Will set up 0MQ SUB socket to listen for commands at address {cmd_address}");
  let mut is_connected = false;
  match cmd_socket.connect(&cmd_address) {
    Err(err) => warn!("Not able to connect to {}, Error {err}", cmd_address),
    Ok(_)    => {
      info!("Connected to CnC server at {}", cmd_address);
      is_connected = true;
    }
  }
  if is_connected {
    //cmd_socket.set_subscribe(&my_topic.as_bytes());
    match cmd_socket.set_subscribe(&topic_broadcast.as_bytes()) {
      Err(err) => error!("Can not subscribe to {topic_broadcast}, err {err}"),
      Ok(_)    => ()
    }
    match cmd_socket.set_subscribe(&topic_board.as_bytes()) {
      Err(err) => error!("Can not subscribe to {topic_board}, err {err}"),
      Ok(_)    => ()
    }
  }
  
  //let my_topic = String::from("");
  //.as_bytes();
  //cmd_socket.set_subscribe(&my_topic.as_bytes());
  
  //match cmd_socket.set_subscribe(&topic_broadcast.as_bytes()) {
  //  Err(err) => error!("Unable to subscribe to {topic_broadcast}, error {err}"),
  //  Ok(_) => ()
  //}
  //match cmd_socket.set_subscribe(&topic_board.as_bytes()) {
  //  Err(err) => error!("Unable to subscribe to {topic_board}, error {err}"),
  //  Ok(_) => ()
  //}
  let mut heartbeat     = Instant::now();

  error!("TODO: Heartbeat feature not yet implemented on C&C side");
  let heartbeat_received = false;
  loop {
    if !heartbeat_received {
      trace!("No heartbeat since {}", heartbeat.elapsed().as_secs());
      if heartbeat.elapsed().as_secs() > heartbeat_timeout_seconds as u64 {
        warn!("No heartbeat received since {heartbeat_timeout_seconds}. Attempting to reconnect!");
        match cmd_socket.connect(&cmd_address) {
          Err(err) => {
            error!("Not able to connect to {}, Error {err}", cmd_address);
            is_connected = false;
          }
          Ok(_)    => {
            debug!("Connected to CnC server at {}", cmd_address);
            is_connected = true;
          }
        }
        if is_connected {
          //cmd_socket.set_subscribe(&my_topic.as_bytes());
          match cmd_socket.set_subscribe(&topic_broadcast.as_bytes()) {
            Err(err) => error!("Can not subscribe to {topic_broadcast}, err {err}"),
            Ok(_)    => ()
          }
          match cmd_socket.set_subscribe(&topic_board.as_bytes()) {
            Err(err) => error!("Can not subscribe to {topic_board}, err {err}"),
            Ok(_)    => ()
          }
        }
        heartbeat = Instant::now();
      }
    }

    if !is_connected {
      continue;
    }

    //match cmd_socket.poll(zmq::POLLIN, 1) {
    //  Err(err) => {
    //    warn!("Polling the 0MQ command socket failed! Err: {err}");
    //    thread::sleep(one_milli);
    //    continue;
    //  }
    //  Ok(in_waiting) => {
    //    trace!("poll successful!");
    //    if in_waiting == 0 {
    //        continue;
    //    }
    match cmd_socket.recv_bytes(zmq::DONTWAIT) {
      Err(err) => trace!("Problem receiving command over 0MQ ! Err {err}"),
      Ok(cmd_bytes)  => {
        info!("Received bytes {}", cmd_bytes.len());
        // we have to strip off the topic
        match TofCommand::from_bytestream(&cmd_bytes, &mut 4) {
          Err(err) => error!("Problem decoding command {}", err),
          Ok(cmd)  => {
            // we got a valid tof command, forward it and wait for the 
            // response
            let tof_resp  = TofResponse::GeneralFail(RESP_ERR_NOTIMPLEMENTED);
            let resp_not_implemented = prefix_board_id(&mut tof_resp.to_bytestream());
            //let resp_not_implemented = TofResponse::GeneralFail(RESP_ERR_NOTIMPLEMENTED);
            match cmd {
              TofCommand::Ping (_) => {
                info!("Received ping signal");
                let r = TofResponse::Success(0);
                match cmd_socket.send(r.to_bytestream(),0) {
                  Err(err) => warn!("Can not send response!, Err {err}"),
                  Ok(_)    => info!("Responded to Ping!")
                }
                continue;
              
              }
              TofCommand::PowerOn   (_mask) => {
                error!("Not implemented");
                match cmd_socket.send(resp_not_implemented,0) {
                  Err(err) => error!("Can not send response! Err {err}"),
                  Ok(_)    => trace!("Resp sent!")
                }
                continue;
              },
              TofCommand::PowerOff  (_mask) => {
                error!("Not implemented");
                match cmd_socket.send(resp_not_implemented,0) {
                  Err(err) => error!("Can not send response! {err}"),
                  Ok(_)    => trace!("Resp sent!")
                }
                continue;
              },
              TofCommand::PowerCycle(_mask) => {
                error!("Not implemented");
                match cmd_socket.send(resp_not_implemented,0) {
                  Err(err) => error!("Can not send response! {err}"),
                  Ok(_)    => trace!("Resp sent!")
                }
                continue;
              },
              TofCommand::RBSetup   (_mask) => {
                warn!("Not implemented");
                match cmd_socket.send(resp_not_implemented,0) {
                  Err(err) => warn!("Can not send response! Err {err}"),
                  Ok(_)    => trace!("Resp sent!")
                }
                continue;
              }, 
              TofCommand::SetThresholds   (_thresholds) =>  {
                warn!("Not implemented");
                match cmd_socket.send(resp_not_implemented,0) {
                  Err(err) => warn!("Can not send response! Err {err}"),
                  Ok(_)    => trace!("Resp sent!")
                }
                continue;
              },
              TofCommand::StartValidationRun  (_) => {
                warn!("Not implemented");
                match cmd_socket.send(resp_not_implemented,0) {
                  Err(err) => warn!("Can not send response! Err {err}"),
                  Ok(_)    => trace!("Resp sent!")
                }
                continue;
              },
              TofCommand::RequestWaveforms (eventid) => {
                trace!("Requesting waveforms for event {eventid}");
                error!("Not implemented");
                match cmd_socket.send(resp_not_implemented,0) {
                  Err(err) => warn!("Can not send response! Err {err}"),
                  Ok(_)    => trace!("Resp sent!")
                }
                continue;
              },
              TofCommand::UnspoolEventCache   (_) => {
                warn!("Not implemented");
                match cmd_socket.send(resp_not_implemented,0) {
                  Err(err) => warn!("Can not send response! Err {err}"),
                  Ok(_)    => trace!("Resp sent!")
                }
                continue;
              },
              TofCommand::StreamOnlyRequested (_) => {
                error!("This feature is deprecated in favor of having entire runs in a certain TofOperationMode. This simplifies configuration, while reducing flexibility. These decidsions are not easy, and we might go back to reimplementing this feature again.");
                //let mode = TofOperationMode::TofModeRequestReply;
                //
                //match op_mode.try_send(mode) {
                //  Err(err) => trace!("Error sending! {err}"),
                //  Ok(_)    => ()
                //}
                //let resp_good = TofResponse::Success(RESP_SUCC_FINGERS_CROSSED);
                //match cmd_socket.send(resp_good.to_bytestream(),0) {
                //  Err(err) => warn!("Can not send response! Err {err}"),
                //  Ok(_)    => trace!("Resp sent!")
                //}
                continue;
              },
              TofCommand::StreamAnyEvent      (_) => {
                error!("This feature is deprecated in favor of having entire runs in a certain TofOperationMode. This simplifies configuration, while reducing flexibility. These decidsions are not easy, and we might go back to reimplementing this feature again.");
                //let mode = TofOperationMode::StreamAny;
                //match op_mode.try_send(mode) {
                //  Err(err) => trace!("Error sending! {err}"),
                //  Ok(_)    => ()
                //}
                //let resp_good = TofResponse::Success(RESP_SUCC_FINGERS_CROSSED);
                //match cmd_socket.send(resp_good.to_bytestream(),0) {
                //  Err(err) => warn!("Can not send response! Err {err}"),
                //  Ok(_)    => trace!("Resp sent!")
                //}
                continue;
              },
              TofCommand::DataRunStart (_max_event) => {
                // let's start a run. The value of the TofCommnad shall be 
                // nevents
                println!("Will initialize new run!");
                let rc    = get_runconfig(&run_config_file);
                //if rc.stream_any {
                //  match op_mode.send(TofOperationMode::StreamAny) {
                //    Err(err) => error!("Can not set TofOperationMode to StreamAny! Err {err}"),
                //    Ok(_)    => info!("Using RBMode STREAM_ANY")
                //  }
                //}
                match run_config.send(rc) {
                  Err(err) => error!("Error initializing run! {err}"),
                  Ok(_)    => ()
                };
                let resp_good = TofResponse::Success(RESP_SUCC_FINGERS_CROSSED);
                match cmd_socket.send(resp_good.to_bytestream(),0) {
                  Err(err) => warn!("Can not send response! Err {err}"),
                  Ok(_)    => trace!("Resp sent!")
                }
              },
              TofCommand::DataRunEnd(_)   => {
                println!("Received command to end run!");
                // default is not active for run config

                let  rc = RunConfig::new();
                match run_config.send(rc) {
                  Err(err) => error!("Error stopping run! {err}"),
                  Ok(_)    => ()
                }
                // send response later 

              //  if !self.run_active {
              //    return Ok(TofResponse::GeneralFail(RESP_ERR_NORUNACTIVE));
              //  }
              //  warn!("Will kill current run!");
              //  self.kill_chn.send(true);
              //  return Ok(TofResponse::Success(RESP_SUCC_FINGERS_CROSSED));
              },
              TofCommand::VoltageCalibration (_) => {
                warn!("Not implemented");
                match cmd_socket.send(resp_not_implemented,0) {
                  Err(err) => warn!("Can not send response! Err {err}"),
                  Ok(_)    => trace!("Resp sent!")
                }
                continue;
              },
              TofCommand::TimingCalibration  (_) => {
                warn!("Not implemented");
                match cmd_socket.send(resp_not_implemented,0) {
                  Err(err) => warn!("Can not send response! Err {err}"),
                  Ok(_)    => trace!("Resp sent!")
                }
                continue;
              },
              TofCommand::CreateCalibrationFile (_) => {
                warn!("Not implemented");
                match cmd_socket.send(resp_not_implemented,0) {
                  Err(err) => error!("Can not send response! Err {err}"),
                  Ok(_)    => trace!("Resp sent!")
                }
                continue;
              },
              TofCommand::RequestEvent(eventid) => {
                match evid_to_cache.send(eventid) {
                  Err(err) => {
                    error!("Problem sending event id to cache! Err {err}");
                    //return Ok(TofResponse::GeneralFail(*eventid));
                  },
                  Ok(event) => {
                    error!("Noting implemented yet. Have found event {:?} though", event);
                  }
                }
                //continue;
              },
              TofCommand::RequestMoni (_) => {
                warn!("Not implemented");
                match cmd_socket.send(resp_not_implemented,0) {
                  Err(err) => error!("Can not send response! Err {err}"),
                  Ok(_)    => trace!("Resp sent!")
                }
                continue;
              },
              TofCommand::Unknown (_) => {
                warn!("Not implemented");
                match cmd_socket.send(resp_not_implemented,0) {
                  Err(err) => error!("Can not send response! Error {err}"),
                  Ok(_)    => trace!("Resp sent!")
                }
                continue;
              }
              _ => {
              match cmd_socket.send(resp_not_implemented,0) {
                Err(err) => warn!("Can not send response! Error {err}"),
                Ok(_)    => trace!("Resp sent!")
              }
              continue;
              }
            } 
         
            //// now get the response from the clients
            //match rsp_receiver.recv() {
            //  Err(err) => {
            //    trace!("Did not recv response!");
            //    warn!("Intended command receiver did not reply! Responding with Failure");
            //    let resp = TofResponse::GeneralFail(RESP_ERR_CMD_STUCK);
            //    match cmd_socket.send(resp.to_bytestream(), 0) {
            //      Err(err) => warn!("The command likely failed and we could not send a response. This is bad!"),
            //      Ok(_)    => trace!("The command likely failed, but we did not lose connection"),
            //    }
            //  },
            //  Ok(resp) => {
            //    match cmd_socket.send(resp.to_bytestream(), 0) {
            //      Err(err) => warn!("The command likely went through, but we could not send a response. This is bad!"),
            //      Ok(_)    => trace!("The command likely went through, but we did not lose connection"),
            //    }
            //  }
            //}
          }
        }  
      }
    }
  }
}


/// Manage the 0MQ PUB socket and send everything 
/// which comes in over the wire as a byte 
/// payload
///
/// # Arguments 
/// * write_to_disk : Write data to local disk (most likely
///                   a SD card). This option should be only
///                   used for diagnostic purposes.
/// * print_packets : Print outgoing packets to terminal
pub fn data_publisher(data           : &Receiver<TofPacket>,
                      write_to_disk  : bool,
                      file_suffix    : Option<&str> ,
                      testing        : bool,
                      print_packets  : bool) {
  let mut address_ip = String::from("tcp://");
  let this_board_ip = local_ip().expect("Unable to obtainl local board IP. Something is messed up!");
  let data_port    = DATAPORT;
  if testing {
    warn!("Testing mode!");
  }

  match this_board_ip {
    IpAddr::V4(ip) => address_ip += &ip.to_string(),
    IpAddr::V6(_) => panic!("Currently, we do not support IPV6!")
  }
  let data_address : String = address_ip.clone() + ":" + &data_port.to_string();
  let ctx = zmq::Context::new();
  
  let data_socket = ctx.socket(zmq::PUB).expect("Unable to create 0MQ PUB socket!");
  data_socket.bind(&data_address).expect("Unable to bind to data (PUB) socket {data_adress}");
  info!("0MQ PUB socket bound to address {data_address}");

  let board_id = address_ip.split_off(address_ip.len() -2);
  let blobfile_name = "rb_".to_owned()
                       + &board_id.to_string()
                       + file_suffix.unwrap_or(".robin");

  let blobfile_path = Path::new(&blobfile_name);
  

  let mut file_on_disk : Option<File> = None;//let mut output = File::create(path)?;
  if write_to_disk {
    // in case it is a calibration file, delete any old 
    // calibration and write it to a specific location
    let home      = env::var_os("HOME").unwrap_or(OsString::from("/home/gaps"));
    let calib_dir = home.to_string_lossy().to_string() + "/calib"; 
    if blobfile_name.ends_with("cali.tof.gaps") {
      match fs::metadata(&calib_dir) {
        Ok(metadata) => {
          // Check if the metadata is for a directory
          if !metadata.is_dir() {
            error!("The path exists, but it is not a directory.");
          }
        }
        Err(_) => {
          // An error occurred, which typically means the directory does not exist
          warn!("No calibration directory found. Will create {}", calib_dir);
          match fs::create_dir(calib_dir.clone()) {
            Ok(_) => (),
            Err(err) => {
              error!("Can not create {}!", calib_dir)
            }
          }
        }
      } // end match
      let mut calib_file = Path::new(&calib_dir);
      let local_file = calib_file.join(blobfile_name);
      //calib_file = &calib_file.join(blobfile_name);
      info!("Writing calibration to {}", local_file.display() );
      file_on_disk = OpenOptions::new().create(true).write(true).open(local_file).ok()
    } else {
      info!("Writing packets to {}", blobfile_name );
      file_on_disk = OpenOptions::new().append(true).create(true).open(blobfile_path).ok()
    }
  }
 
  // these are only required for testing
  let mut last_10k_evids = Vec::<u32>::new();
  if testing {
    last_10k_evids = Vec::<u32>::with_capacity(10000);
  }
  let mut n_tested : u32 = 0;
  loop {
    match data.recv() {
      Err(err) => trace!("Error receiving TofPacket {err}"),
      Ok(packet)    => {
        // pass on the packet downstream
        // wrap the payload INTO THE 
        // FIXME - retries?
        if write_to_disk && !packet.no_write_to_disk {
          //println!("{:?}", packet.packet_type);
          if packet.packet_type == PacketType::RBEvent || 
            packet.packet_type == PacketType::RBHeader || 
            packet.packet_type == PacketType::RBCalibration {
            // don't write individual calibrations
            //FIXME

            //println!("is rb event");
            match &mut file_on_disk {
              None => error!("We want to write data, however the file is invalid!"),
              Some(f) => {
                //println!("write to file");
                //match f.write_all(packet.payload.as_slice()) {
                match f.write_all(packet.to_bytestream().as_slice()) {
                  Err(err) => error!("Writing file to disk failed! Err {err}"),
                  Ok(()) => ()
                }
              }
            }
          }
        }
        
        if testing {
          match packet.packet_type {
            PacketType::RBEventPayload => {
              n_tested += 1;
              match RBEventMemoryView::from_bytestream(&packet.payload, &mut 0) {
                Ok(event) => {
                  last_10k_evids.push(event.event_id);
                },
                Err(err) => {
                   warn!("Error occured during testing! {err}");
                   warn!("We are seing a payload of {} bytes", packet.payload.len());
                   //warn!("Last few bytes:");
                   //for k in packet.payload.len() - 20..packet.payload.len() {
                   //  warn!("-- {}", packet.payload[k]);
                   //}
                }
              }
            },
            PacketType::RBEvent => {
              n_tested += 1;
              match RBEvent::from_bytestream(&packet.payload, &mut 0) {
                Ok(event) => {
                  last_10k_evids.push(event.header.event_id);
                },
                Err(err) => {
                   warn!("Error occured during testing! {err}");
                   warn!("We are seing a payload of {} bytes", packet.payload.len());
                   //warn!("Last few bytes:");
                   //for k in packet.payload.len() - 20..packet.payload.len() {
                   //  warn!("-- {}", packet.payload[k]);
                   //}
                }
              }
            },
            _ => ()
          }
          if n_tested == 10000 {
            println!("Testing batch complete! Will check the last 10000 events!");
            println!("-- first event id {}",  last_10k_evids[0]);
            println!("-- last event id {}", last_10k_evids[last_10k_evids.len() - 1]);
            // this is not efficient, but this is a testing case anyway
            let mut duplicates = false;
            for i in 0..last_10k_evids.len() {
              for j in (i + 1)..last_10k_evids.len() {
                if last_10k_evids[i] == last_10k_evids[j] {
                  println!("FAIL : Found eventid {} at positions {} and {}", last_10k_evids[i], i, j);
                  duplicates = true;
                }
              }
            }
            if !duplicates {
              println!("PASS - we did not observe any duplicate entries!");
            }
            let missing = find_missing_elements(&last_10k_evids);
            if missing.is_empty() {
              println!("PASS - we did not miss any event ids!");
            } else {
              println!("FAIL - we missed {} event ids ({}/100)", missing.len(), missing.len() as f32/10000.0);
              println!("MISSING {:?}", missing);
            }
            println!("----");
            println!("---- last 10k evids {:?}", last_10k_evids);
            last_10k_evids.clear();
          }
        } // end testing
        
        // prefix the board id, except for our Voltage, Timing and NOI 
        // packages. For those, we prefix wit 
        let tp_payload : Vec<u8>;
        let mut data_type = DataType::Unknown;
        if matches!(packet.packet_type, PacketType::RBEvent) {
          match RBEvent::extract_datatype(&packet.payload) {
            Ok(dtype) => {
              data_type = dtype;
            }
            Err(err) => {
              error!("Unable to extract data type! Err {err}");
            }
          }
        }
        match data_type {
          DataType::VoltageCalibration |
          DataType::TimingCalibration  | 
          DataType::Noi => {
            tp_payload = prefix_local(&mut packet.to_bytestream());
          },
          _ => {
            tp_payload = prefix_board_id(&mut packet.to_bytestream());
          }
        }
        if print_packets {
          println!("=> Tof packet type: {} with {} bytes!", packet.packet_type, packet.payload.len());
        }

        match data_socket.send(tp_payload,zmq::DONTWAIT) {
          Ok(_)    => trace!("0MQ PUB socket.send() SUCCESS!"),
          Err(err) => error!("Not able to send over 0MQ PUB socket! Err {err}"),
        }
      }
    }
  }
}

/// Gather monitoring data and pass it on
pub fn monitoring(ch : &Sender<TofPacket>,
                  verbose : bool) {
 
  let moni_interval  = 60;
  let heartbeat      = time::Duration::from_secs(moni_interval);
  let board_id = get_board_id().unwrap_or(0); 
  loop {
    // get tof-control data
    let mut moni_dt = RBMoniData::new();
    moni_dt.board_id = board_id as u8; 
    #[cfg(feature="tofcontrol")]
    let rb_temp = RBtemp::new();
    #[cfg(feature="tofcontrol")]
    let rb_mag  = RBmag::new();
    #[cfg(feature="tofcontrol")]
    let rb_vcp  = RBvcp::new();
    #[cfg(feature="tofcontrol")]
    let rb_ph   = RBph::new();
    #[cfg(feature="tofcontrol")]
    moni_dt.add_rbtemp(&rb_temp);
    #[cfg(feature="tofcontrol")]
    moni_dt.add_rbmag(&rb_mag);
    #[cfg(feature="tofcontrol")]
    moni_dt.add_rbvcp(&rb_vcp);
    #[cfg(feature="tofcontrol")]
    moni_dt.add_rbph(&rb_ph);
    
    let rate_query = get_trigger_rate();
    match rate_query {
      Ok(rate) => {
        debug!("Monitoring thread -> Rate: {rate}Hz ");
        moni_dt.rate = rate as u16;
      },
      Err(_)   => {
        warn!("Can not send rate monitoring packet, register problem");
      }
    }
   
    if verbose {
      println!("{}", moni_dt);
    }
    let tp = TofPacket::from(&moni_dt);
    match ch.try_send(tp) {
      Err(err) => {error!("Issue sending RBMoniData {:?}", err)},
      Ok(_)    => {debug!("Send RBMoniData successfully!")}
    }
    thread::sleep(heartbeat);
  }
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


// palceholder
#[derive(Debug)]
pub struct FIXME {
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

///  A simple routine which runs until 
///  a certain amoutn of events are 
///  acquired
///
///  The runner will setup the DRS4, and 
///  set it to idle state when it is 
///  finished.
///
///  To be resource friendly, this thread
///  goes with 1 second precision.
///
///  # Arguments
///
///  * max_errors     : End myself when I see a certain
///                     number of errors
///  * prog_op_ev     : An option for a progress bar which
///                     is helpful for debugging
///  * force_trigger  : Run in forced trigger mode
///
///
pub fn runner(run_config              : &Receiver<RunConfig>,
              max_errors              : Option<u64>,
              bs_sender               : &Sender<Vec<u8>>,
              dtf_to_evproc           : &Sender<(DataType,DataFormat)>,
              opmode_to_cache         : &Sender<TofOperationMode>,
              show_progress           : bool) { // FIXME deprecate this..
  
  let one_milli        = time::Duration::from_millis(1);
  let one_sec          = time::Duration::from_secs(1);
  let mut first_iter   = true; 
  let mut last_evt_cnt : u32 = 0;
  let mut evt_cnt      = 0u32;
  let mut delta_events : u64;
  let mut n_events     : u64 = 0;
  // FIXME - this is currently useless
  let     n_errors     : u64 = 0;
  
  // trigger settings. Per default, we latch to the 
  let mut latch_to_mtb = true;

  let mut timer               = Instant::now();
  // do we have to manually trigger at the desired 
  // time inberval? Then we set force_trigger.
  // The Poisson trigger triggers automatically.
  let mut force_trigger = false;
  let mut time_between_events : Option<f32> = None;
  let now = time::Instant::now();

  // run start/stop conditions
  let mut terminate = false;
  let mut is_running = false;
  let mut rc = RunConfig::new();
  
  // this are all settings for the progress bar
  let mut template_bar_env : &str;
  let mut sty_ev : ProgressStyle;
  let mut multi_prog : MultiProgress;
  let mut prog_a  = ProgressBar::hidden();
  let mut prog_b  = ProgressBar::hidden();
  let mut prog_ev = ProgressBar::hidden();
  let template_bar_a   : &str = "[{elapsed_precise}] {prefix} {msg} {spinner} {bar:60.blue/grey} {bytes:>7}/{total_bytes:7} ";
  let template_bar_b   : &str = "[{elapsed_precise}] {prefix} {msg} {spinner} {bar:60.green/grey} {bytes:>7}/{total_bytes:7} ";
  let label_a   = String::from("Buff A");
  let label_b   = String::from("Buff B");
  let sty_a = ProgressStyle::with_template(template_bar_a)
  .unwrap();
  let sty_b = ProgressStyle::with_template(template_bar_b)
  .unwrap();
  prog_a.set_position(0);
  prog_b.set_position(0);
  prog_ev.set_position(0);

  let mut which_buff  : RamBuffer;
  let mut buff_size   : usize;
  // set a default of 2000 events in the cache, 
  // but this will be defined in the run params
  let mut buffer_trip : usize = 2000*EVENT_SIZE;
  let mut uio1_total_size = DATABUF_TOTAL_SIZE;
  let mut uio2_total_size = DATABUF_TOTAL_SIZE;
  loop {
    match run_config.try_recv() {
      Err(err) => {
        trace!("Did not receive a new RunConfig! Err {err}");
        //thread::sleep(one_sec);
        //continue;
      }
      Ok(new_config) => {
        println!("=> Received a new set of RunConfig!");
        println!("{}", new_config);

        // reset some variables for the loop
        first_iter   = true; 
        last_evt_cnt = 0;
        evt_cnt      = 0;
        //delta_events = 0;
        n_events     = 0;

        rc          = new_config;
        
        // first of all, check if the new run config is active. 
        // if not, stop all triggers
        if rc.is_active { 
          terminate = false;
          //is_running = true;
        } else { 
          info!("Received runconfig is not active! Stop current run...");
          // just to be sure we set the self trigger rate to 0 
          // this is for the poisson trigger)
          match set_self_trig_rate(0) {
            Err(err) => error!("Resetting self trigger rate to 0Hz failed! Err {err}"),
            Ok(_)    => ()
          }
          match disable_trigger() {
            Err(err) => error!("Can not disable triggers, error {err}"),
            Ok(_)    => info!("Disabling triggers! Stopping current run!")
          }
          if show_progress {
            prog_ev.finish();
            prog_a.finish();
            prog_b.finish();
          }
          // do nothing else.
          is_running = false;
          terminate  = true;
          continue;
        }
        // from here on, we prepare to start 
        // a new run with this RunConfig!
        // set the channel mask
        match set_active_channel_mask_with_ch9(rc.active_channel_mask as u32) {
          Ok(_) => (),
          Err(err) => {
            error!("Unable to set channel mask! Err {err}");
          }
        }
        reset_dma_and_buffers();

        // deal with the individual settings:
        // first buffer size
        buffer_trip = (rc.rb_buff_size as usize)*EVENT_SIZE; 
        if (buffer_trip > uio1_total_size) 
        || (buffer_trip > uio2_total_size) {
          error!("Tripsize of {buffer_trip} exceeds buffer sizes of A : {uio1_total_size} or B : {uio2_total_size}. The EVENT_SIZE is {EVENT_SIZE}");
          warn!("Will set buffer_trip to {DATABUF_TOTAL_SIZE}");
          buffer_trip = DATABUF_TOTAL_SIZE;
        } else {
          uio1_total_size = buffer_trip;
          uio2_total_size = buffer_trip;
        }
        // set channel mask (if different from 255)
        //match set_active_channel_mask(rc.active_channel_mask) {
        //  Ok(_) => (),
        //  Err(err) => {
        //    error!("Setting active channel mask failed for mask {}, error {}", rc.active_channel_mask, err);
        //  }
        //}
        let mut tof_op_mode = TofOperationMode::RequestReply;
        if rc.stream_any {
          tof_op_mode = TofOperationMode::StreamAny;
        }
        match opmode_to_cache.send(tof_op_mode) {
          Err(err) => {
            error!("Unable to send TofOperationMode to the event cache! Err {err}");
          }
          Ok(_)    => ()
        }

        // send the data format/type directly to the 
        let df_c = rc.data_format.clone();
        let dt_c = rc.data_type.clone();
        match dtf_to_evproc.send((dt_c, df_c)) {
          Err(err) => {
            error!("Unable to send dataformat & type to the event processing subroutine! Err {err}");
          }
          Ok(_) => ()
        }

        // data type
        match rc.data_type {
          DataType::VoltageCalibration | 
          DataType::TimingCalibration  | 
          DataType::Noi                |
          DataType::RBTriggerPoisson   | 
          DataType::RBTriggerPeriodic =>  {
            latch_to_mtb = false;
          },
          _ => ()
        }
        if rc.trigger_poisson_rate > 0 {
          latch_to_mtb = false;
          // we also activate the poisson trigger
          enable_poisson_self_trigger(rc.trigger_poisson_rate as f32);
        }
        if rc.trigger_fixed_rate>0 {
          force_trigger = true;
          time_between_events = Some(1.0/(rc.trigger_fixed_rate as f32));
          warn!("Will run in forced trigger mode with a rate of {} Hz!", rc.trigger_fixed_rate);
          debug!("Will call trigger() every {} seconds...", time_between_events.unwrap());
          latch_to_mtb = false;
        }

        // preparations done, let's gooo
        reset_dma_and_buffers();

        if latch_to_mtb {
          match set_master_trigger_mode() {
            Err(err) => error!("Can not initialize master trigger mode, Err {err}"),
            Ok(_)    => info!("Latching to MasterTrigger")
          }
        } else {
          match disable_master_trigger_mode() {
            Err(err) => error!("Can not disable master trigger mode, Err {err}"),
            Ok(_)    => info!("Master trigger mode didsabled!")
          }
        }
        // this basically signals "RUNSTART"
        match enable_trigger() {
          Err(err) => error!("Can not enable triggers! Err {err}"),
          Ok(_)    => info!("(Forced) Triggers enabled - Run start!")
        }
        // FIXME - only if above call Ok()
        is_running = true;

        if !force_trigger {
          // we relax and let the system go 
          // for a short bit
          thread::sleep(one_sec);
          match get_trigger_rate() {
            Err(err) => error!("Unable to obtain trigger rate! Err {err}"),
            Ok(rate) => info!("Seing MTB trigger rate of {rate} Hz")
          }
        }
        if show_progress {
          if rc.runs_forever() {
            template_bar_env = "[{elapsed_precise}] {prefix} {msg} {spinner} ";
          } else {
            template_bar_env = "[{elapsed_precise}] {prefix} {msg} {spinner} {bar:60.red/grey} {pos:>7}/{len:7}";
          }
          sty_ev = ProgressStyle::with_template(template_bar_env)
          .unwrap();
          multi_prog = MultiProgress::new();
          prog_a  = multi_prog
                    .add(ProgressBar::new(uio1_total_size as u64)); 
          prog_b  = multi_prog
                    .insert_after(&prog_a, ProgressBar::new(uio2_total_size as u64)); 
          prog_ev = multi_prog
                    .insert_after(&prog_b, ProgressBar::new(rc.nevents as u64)); 
          prog_a.set_message (label_a.clone());
          prog_a.set_prefix  ("\u{1F4BE}");
          prog_a.set_style   (sty_a.clone());
          prog_a.set_position(0);
          prog_b.set_message (label_b.clone());
          prog_b.set_prefix  ("\u{1F4BE}");
          prog_b.set_style   (sty_b.clone());
          prog_b.set_position(0);
          prog_ev.set_style  (sty_ev.clone());
          prog_ev.set_prefix ("\u{2728}");
          prog_ev.set_message("EVENTS");
          prog_ev.set_position(0);
          info!("Preparations complete. Run start should be imminent.");
        }
        continue; // start loop again
      } // end Ok(RunConfig) 
    } // end run_params.try_recv()

    if is_running {
      if terminate {
        match disable_trigger() {
          Err(err) => error!("Can not disable triggers, error {err}"),
          Ok(_)    => info!("Triggers disabled!")
        }
        if show_progress {
          prog_ev.finish();
          prog_a.finish();
          prog_b.finish();
        }
        is_running = false;
        // flush the buffers
        match ram_buffer_handler(1,
                                 &bs_sender) { 
          Err(err)   => {
            error!("Can not deal with RAM buffers {err}");
          },
          Ok(_) => ()
        }
        info!("Run stopped! The runner has processed {n_events} events!");
      } // end if terminate
      
      // We did not terminate the run,
      // that means we are still going!
      if force_trigger {
        //println!("Forcing trigger!");
        //println!("Time between events {}", time_between_events.unwrap());
        let elapsed = timer.elapsed().as_secs_f32();
        //println!("Elapsed {}", elapsed);
        trace!("Forced trigger mode, {} seconds since last trigger", elapsed);
        // decide if we have to issue the trigger signal NOW!
        if elapsed > time_between_events.unwrap() {
          timer = Instant::now(); 
          match trigger() {
            Err(err) => error!("Error when triggering! {err}"),
            Ok(_)    => trace!("Firing trigger!")
          }
        } else { // not enough time has yet passed for the next trigger signal
          // FIXME - we could sleep here for a bit!
          continue;
        }
      }    

      // calculate current event count
      if !force_trigger {
        // this checks if we have seen a new event
        match get_event_count() {
          Err (err) => {
            error!("Can not obtain event count! Err {:?}", err);
            continue;
          }
          Ok (cnt) => {
            evt_cnt = cnt;
            if first_iter {
              last_evt_cnt = evt_cnt;
              first_iter = false;
              continue;
            }
            if evt_cnt == last_evt_cnt {
              thread::sleep(one_milli);
              trace!("We didn't get an updated event count!");
              continue; // only continue after we see a new event!
            }
          } // end ok
        } // end match
      } // end force trigger

      // AT THIS POINT WE KNOW WE HAVE SEEN SOMETHING!!!
      // THIS IS IMPORTANT
      match ram_buffer_handler(buffer_trip,
                               &bs_sender) { 
        Err(err)   => {
          error!("Can not deal with RAM buffers {err}");
          continue;
        }
        Ok(result) => {
          which_buff = result.0;
          buff_size  = result.1;
        }
      }
      if force_trigger {
          n_events += 1;
      } else {
        delta_events = (evt_cnt - last_evt_cnt) as u64;
        n_events    += delta_events;
        last_evt_cnt = evt_cnt;
      }
      if show_progress {
        match which_buff {
          RamBuffer::A => {
            prog_a.set_position(buff_size as u64);
            prog_b.set_position(0);
          }
          RamBuffer::B => {
            prog_b.set_position(buff_size as u64);
            prog_a.set_position(0);
          }
        }
        prog_ev.set_position(n_events);
      }

    } // end is_running
    
    // from here on, check termination 
    // conditions
    if !rc.runs_forever() {
      if rc.nevents != 0 {
        if n_events > rc.nevents as u64{
          terminate = true;
        }
      }
      
      if rc.nseconds > 0 {
          if now.elapsed().as_secs() > rc.nseconds  as u64{
            terminate = true;
          }
        }

      match max_errors {
        None => (),
        Some(max_e) => {
          if n_errors > max_e {
            terminate = true;
          }
        }
      }
      // reduce cpu load
      if !terminate {
        if !force_trigger { 
          thread::sleep(100*one_milli);
        }
      }
    }
  } // end loop
}


/// Recieve the events and hold them in a cache 
/// until they are requested
/// 
/// The function should be wired to a producer
/// of RBEventPayloads
///
/// Requests come in as event ids through `recv_evid`
/// and will be answered through `send_ev_pl`, if 
/// they are in the cache, else None
///
/// # Arguments
///
/// * control_ch : Receive operation mode instructions
///
/// * waveform_analysis : For the events requested, do the waveform processing 
///                         already
pub fn event_cache(tp_recv      : Receiver<TofPacket>,
                   tp_to_pub    : &Sender<TofPacket>,
                   resp_to_cmd  : &Sender<TofResponse>,
                   get_op_mode  : &Receiver<TofOperationMode>, 
                   recv_evid    : Receiver<u32>,
                   waveform_analysis : bool,
                   cache_size   : usize) {
  if waveform_analysis {
    warn!("Waveform analysis is not implemented, won't do it!");
  }

  let mut n_send_errors  = 0u64;   
  let mut op_mode_stream = false;

  let mut oldest_event_id : u32 = 0;
  //let mut event_cache : HashMap::<u32, RBEventPayload> = HashMap::new();
  let mut event_cache : HashMap::<u32, TofPacket> = HashMap::new();
  loop {
    // check changes in operation mode
    match get_op_mode.try_recv() {
      Err(err) => trace!("No op mode change detected! Err {err}"),
      Ok(mode) => {
        warn!("Will change operation mode to {:?}!", mode);
        match mode {
          TofOperationMode::RequestReply => {op_mode_stream = false;},
          TofOperationMode::StreamAny    => {op_mode_stream = true;},
          _ => (),
        }
      }
    }
    match tp_recv.try_recv() {
      Err(err) =>   {
        trace!("No event payload! {err}");
      }
      Ok(packet) => {
        // FIXME - there need to be checks what the 
        // packet type is
        let packet_evid = RBEventHeader::extract_eventid_from_rbheader(&packet.payload); 
        if oldest_event_id == 0 {
          oldest_event_id = packet_evid;
        } //endif
        //// store the event in the cache
        ////println!("Received payload with event id {}" ,event.event_id);
        if !event_cache.contains_key(&packet_evid) {
          event_cache.insert(packet_evid, packet);
        }
        //// keep track of the oldest event_id
        //trace!("We have a cache size of {}", event_cache.len());
        if event_cache.len() > cache_size {
          event_cache.remove(&oldest_event_id);
          oldest_event_id += 1;
        } //endif
      }
    }


    //// store incoming events in the cache  
    //match recv_ev_pl.try_recv() {
    //  Err(err) => {
    //    trace!("No event payload! {err}");
    //    //continue;
    //  } // end err
    //  Ok(event)  => {
    //    trace!("Received next RBEvent!");
    //    if oldest_event_id == 0 {
    //      oldest_event_id = event.event_id;
    //    } //endif
    //    // store the event in the cache
    //    //println!("Received payload with event id {}" ,event.event_id);
    //    if !event_cache.contains_key(&event.event_id) {
    //      event_cache.insert(event.event_id, event);
    //    }
    //    // keep track of the oldest event_id
    //    trace!("We have a cache size of {}", event_cache.len());
    //    if event_cache.len() > cache_size {
    //      event_cache.remove(&oldest_event_id);
    //      oldest_event_id += 1;
    //    } //endif
    //  }// end Ok
    //} // end match
  
    // if we are in "stream_any" mode, we don't need to take care
    // of any fo the response/request.
    if op_mode_stream {
      //event_cache.as_ref().into_iter().map(|(evid, payload)| {send_ev_pl.try_send(Some(payload))});
      //let evids = event_cache.keys();
      for tp in event_cache.values() {
        // FIXME - this is bad! Too much allocation
        //let tp = TofPacket::from(payload);
        //let tp = TofPacket::from_bytestream(payload, &mut 0).unwrap();
        //info!("{}", tp);
        match tp_to_pub.try_send(tp.clone()) {
          Err(err) => {
            error!("Error sending! {err}");
            n_send_errors += 1;
          }
          Ok(_)    => ()
        }
      }
      event_cache.clear();
      //for n in evids { 
      //  let payload = event_cache.remove(n).unwrap();
      //  send_ev_pl.try_send(Some(payload)); 
      //}
      continue;
    }
    match recv_evid.try_recv() {
      Err(err) => {
        trace!("Issue receiving event id! Err: {err}");
      },
      Ok(event_id) => {
        let has_it = event_cache.contains_key(&event_id);
        if !has_it {
          //match send_ev_pl.try_send(None) {
          let resp = TofResponse::EventNotReady(event_id);
          match resp_to_cmd.try_send(resp) {
            Err(err) => trace!("Error informing the commander that we don't have that! Err {err}"),
            Ok(_)    => ()
          }
          // hamwanich
          debug!("We don't have {event_id}!");
        } else {
          //let event = event_cache.remove(&event_id).unwrap();
          let tp = event_cache.remove(&event_id).unwrap();
          let resp =  TofResponse::Success(event_id);
          match resp_to_cmd.try_send(resp) {
            Err(err) => trace!("Error informing the commander that we do have {event_id}! Err {err}"),
            Ok(_)    => ()
          }
          //let tp = TofPacket::from(&event);
          //let tp = TofPacket::from_bytestream(&event, &mut 0).unwrap();
          match tp_to_pub.try_send(tp) {
            Err(err) => trace!("Error sending! {err}"),
            Ok(_)    => ()
          }
        }
      }
    } // end match
    if n_send_errors > 0 {
      warn!("There were {n_send_errors} errors during sending!");
    }
  } // end loop
}

///// Deal with incoming commands
/////
/////
/////
/////
//pub struct Commander<'a> {
//
//  pub evid_send        : Sender<u32>,
//  pub change_op_mode   : Sender<TofOperationMode>, 
//  pub rb_evt_recv      : Receiver<Option<RBEventPayload>>,
//  pub hasit_from_cache : &'a Receiver<bool>,
//}
//
//impl Commander<'_> {
//
//  pub fn new<'a> (send_ev          : Sender<u32>,
//                  hasit_from_cache : &'a Receiver<bool>,
//                  evpl_from_cache  : Receiver<Option<RBEventPayload>>,
//                  change_op_mode   : Sender<TofOperationMode>)
//    -> Commander<'a> {
//
//    Commander {
//      evid_send        : send_ev,
//      change_op_mode   : change_op_mode,
//      rb_evt_recv      : evpl_from_cache,
//      hasit_from_cache : hasit_from_cache,
//    }
//  }
//
//
//  /// Interpret an incoming command 
//  ///
//  /// The command comes most likely somehow over 
//  /// the wir from the tof computer
//  ///
//  /// Match with a list of known commands and 
//  /// take action.
//  ///
//  /// # Arguments
//  ///
//  /// * command : A TofCommand instructing the 
//  ///             commander what to do
//  ///             Will generate a TofResponse 
//  ///             
//  pub fn command (&self, cmd : &TofCommand)
//    -> Result<TofResponse, FIXME> {
//    match cmd {
//      TofCommand::PowerOn   (mask) => {
//        warn!("Not implemented");
//        return Ok(TofResponse::GeneralFail(RESP_ERR_NOTIMPLEMENTED));
//      },
//      TofCommand::PowerOff  (mask) => {
//        warn!("Not implemented");
//        return Ok(TofResponse::GeneralFail(RESP_ERR_NOTIMPLEMENTED));
//      },
//      TofCommand::PowerCycle(mask) => {
//        warn!("Not implemented");
//        return Ok(TofResponse::GeneralFail(RESP_ERR_NOTIMPLEMENTED));
//      },
//      TofCommand::RBSetup   (mask) => {
//        warn!("Not implemented");
//        return Ok(TofResponse::GeneralFail(RESP_ERR_NOTIMPLEMENTED));
//      }, 
//      TofCommand::SetThresholds   (thresholds) =>  {
//        warn!("Not implemented");
//        return Ok(TofResponse::GeneralFail(RESP_ERR_NOTIMPLEMENTED));
//      },
//      TofCommand::StartValidationRun  (_) => {
//        warn!("Not implemented");
//        return Ok(TofResponse::GeneralFail(RESP_ERR_NOTIMPLEMENTED));
//      },
//      TofCommand::RequestWaveforms (eventid) => {
//        warn!("Not implemented");
//        return Ok(TofResponse::GeneralFail(RESP_ERR_NOTIMPLEMENTED));
//      },
//      TofCommand::UnspoolEventCache   (_) => {
//        warn!("Not implemented");
//        return Ok(TofResponse::GeneralFail(RESP_ERR_NOTIMPLEMENTED));
//      },
//      TofCommand::StreamOnlyRequested (_) => {
//        let op_mode = TofOperationMode::TofModeRequestReply;
//        
//        match self.change_op_mode.try_send(op_mode) {
//          Err(err) => trace!("Error sending! {err}"),
//          Ok(_)    => ()
//        }
//        return Ok(TofResponse::Success(RESP_SUCC_FINGERS_CROSSED));
//      },
//      TofCommand::StreamAnyEvent      (_) => {
//        let op_mode = TofOperationMode::TofModeStreamAny;
//        match self.change_op_mode.try_send(op_mode) {
//          Err(err) => trace!("Error sending! {err}"),
//          Ok(_)    => ()
//        }
//        return Ok(TofResponse::Success(RESP_SUCC_FINGERS_CROSSED));
//      },
//      //TofCommand::DataRunStart (max_event) => {
//      //  // let's start a run. The value of the TofCommnad shall be 
//      //  // nevents
//      //  self.workforce.execute(move || {
//      //      runner(Some(*max_event as u64),
//      //             None,
//      //             None,
//      //             self.get_killed_chn,
//      //             None);
//      //  }); 
//      //  return Ok(TofResponse::Success(RESP_SUCC_FINGERS_CROSSED));
//      //}, 
//      //TofCommand::DataRunEnd   => {
//      //  if !self.run_active {
//      //    return Ok(TofResponse::GeneralFail(RESP_ERR_NORUNACTIVE));
//      //  }
//      //  warn!("Will kill current run!");
//      //  self.kill_chn.send(true);
//      //  return Ok(TofResponse::Success(RESP_SUCC_FINGERS_CROSSED));
//      //},
//      TofCommand::VoltageCalibration (_) => {
//        warn!("Not implemented");
//        return Ok(TofResponse::GeneralFail(RESP_ERR_NOTIMPLEMENTED));
//      },
//      TofCommand::TimingCalibration  (_) => {
//        warn!("Not implemented");
//        return Ok(TofResponse::GeneralFail(RESP_ERR_NOTIMPLEMENTED));
//      },
//      TofCommand::CreateCalibrationFile (_) => {
//        warn!("Not implemented");
//        return Ok(TofResponse::GeneralFail(RESP_ERR_NOTIMPLEMENTED));
//      },
//      TofCommand::RequestEvent(eventid) => {
//        match self.evid_send.send(*eventid) {
//          Err(err) => {
//            debug!("Problem sending event id to cache! Err {err}");
//            return Ok(TofResponse::GeneralFail(*eventid));
//          },
//          Ok(_) => (),
//        }
//        match self.hasit_from_cache.recv() {
//          Err(_) => {
//            return Ok(TofResponse::EventNotReady(*eventid));
//          }
//          Ok(hasit) => {
//            // FIXME - prefix topic
//            if hasit {
//              return Ok(TofResponse::Success(*eventid));
//            } else {
//              return Ok(TofResponse::EventNotReady(*eventid));
//            }
//            //Some(event) => {
//            //  match self.zmq_pub_socket.send(event.payload, zmq::DONTWAIT) {
//            //    Ok(_)  => {
//            //      return Ok(TofResponse::Success(*eventid));
//            //    }
//            //    Err(err) => {
//            //      debug!("Problem with PUB socket! Err {err}"); 
//            //      return Ok(TofResponse::ZMQProblem(*eventid));
//            //    }
//            //  }
//            //}
//            //}
//          }
//        }
//      },
//      TofCommand::RequestMoni (_) => {
//      },
//      TofCommand::Unknown (_) => {
//      }
//      _ => {
//      }
//    } 
//    let response = TofResponse::Success(1);
//    Ok(response)
//  }
//}

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





///  Transforms raw bytestream to RBEventPayload
///
///  This allows to get the eventid from the 
///  binrary form of the RBEvent
///
///  #Arguments
/// 
///  * bs_recv     : A receiver for bytestreams. The 
///                  bytestream comes directly from 
///                  the data buffers.
///  * tp_sender   : Send the resulting data product to 
///                  get processed further
///  * data_type   : If different from 0, do some processing
///                  on the data read from memory
///
pub fn event_processing(bs_recv           : &Receiver<Vec<u8>>,
                        tp_sender         : &Sender<TofPacket>,
                        dtf_fr_runner     : &Receiver<(DataType,DataFormat)>) {
  let mut n_events : u32;
  let mut event_id : u32 = 0;
  let mut last_event_id   : u32 = 0; // for checks
  let mut events_not_sent : u64 = 0;
  let mut data_type   : DataType   = DataType::Unknown;
  let mut data_format : DataFormat = DataFormat::Unknown; 
  let one_milli   = time::Duration::from_millis(1);
  'main : loop {
    let mut start_pos : usize = 0;
    n_events = 0;
    if !dtf_fr_runner.is_empty() {
      match dtf_fr_runner.try_recv() {
        Err(err) => {
          error!("Issues receiving datatype/format! Err {err}");
        }
        Ok(dtf) => {
          data_type   = dtf.0;
          data_format = dtf.1; 
          info!("Will process events for data type {}, format {}!", data_type, data_format);
        }
      }
    }
    if bs_recv.is_empty() {
      thread::sleep(5*one_milli);
      continue;
    }
    // this can't be blocking anymore, since 
    // otherwise we miss the datatype
    match bs_recv.recv() {
      Ok(bytestream) => {
        let mut packets_in_stream : u32 = 0;
        'bytestream : loop {
          //println!("Received bytestream");
          match search_for_u16(RBEventMemoryView::HEAD, &bytestream, start_pos) {
            Err(err) => {
              debug!("Send {n_events} events. Got last event_id! {event_id}");
              if start_pos == 0 {
                error!("Got bytestream, but can not find HEAD bytes, err {err:?}");
              }
              break 'bytestream;},
            Ok(head_pos) => {
              let tail_pos   = head_pos + RBEventMemoryView::SIZE;
              if tail_pos >= bytestream.len() - 1 {
                // we are finished here
                warn!("Got a trunctaed event, discarding..");
                trace!("Work on current blob complete. Extracted {n_events} events. Got last event_id! {event_id}");
                //trace!("{:?}", debug_evids);
                break 'bytestream;
              }
              //debug_evids.push(event_id);
              //info!("Got event_id {event_id}");
              n_events += 1;
              start_pos = tail_pos;
              let mut tp = TofPacket::new();
              match data_format {
                DataFormat::HeaderOnly => {
                  event_id        =  RBEventPayload::decode_event_id(&bytestream[head_pos..tail_pos]);
                  if event_id != last_event_id + 1 {
                    error!("Event id not rising continuously! This {}, last {}", event_id, last_event_id);
                  }
                  last_event_id = event_id;
                  let mut payload = Vec::<u8>::new();
                  payload.extend_from_slice(&bytestream[head_pos..tail_pos + 2]);
                  debug!("Prepared TofPacket for event {} with a payload size of {}", event_id, &payload.len());
                  let rb_payload  = RBEventPayload::new(event_id, payload); 
                  tp = TofPacket::from(&rb_payload);
                }
                DataFormat::Default => {
                  let mut pos_in_stream = head_pos;
                  match RBEvent::extract_from_rbeventmemoryview(&bytestream, &mut pos_in_stream) {
                    Err(err)   => {
                      error!("Unable to extract RBEvent from memory! Error {err}");
                      events_not_sent += 1;
                    },
                    Ok (mut event) => {
                      event.data_type = data_type;
                      tp = TofPacket::from(&event);
                    }
                  }
                },
                DataFormat::MemoryView => {
                  //let mut payload = Vec::<u8>::new();
                  //payload.extend_from_slice(&bytestream[head_pos..tail_pos+2]);
                  let mut this_event_start_pos = head_pos;
                  match RBEventHeader::extract_from_rbeventmemoryview(&bytestream, &mut this_event_start_pos) {
                    Err(err) => {
                      //let mut foo = RBEventMemoryView::new();
                      //foo.from_bytestream(&bytestream, head_pos, false);
                      //error!("{:?}", foo);
                      error!("Broken RBEventMemoryView data in memory! Err {}", err);
                      error!("-- we tried to process {} bytes!", tail_pos - head_pos);
                      error!("{:?}", &bytestream[head_pos..head_pos + 100]);
                      error!("{:?}", &bytestream[tail_pos - 10..tail_pos + 130]);
                      events_not_sent += 1;
                    }
                    Ok(event_header)    => {
                      tp = TofPacket::from(&event_header);
                    }
                  }
                }
                _ => {todo!("Dataformat != 0, 1 or 2 is not supported!");}
              } // end match
              // set flags
              match data_type {
                DataType::VoltageCalibration |
                DataType::TimingCalibration  | 
                DataType::Noi => {
                  tp.no_write_to_disk = true;
                },
                _ => ()
              }
              // send the packet
              match tp_sender.send(tp) {
                Ok(_) => {
                  packets_in_stream += 1;
                },
                Err(err) => error!("Problem sending TofPacket over channel! Err {err}"),
              }
              //continue 'bytestream;
            }
          } // end match search_for_u16 
        } // end 'bytestream loop
        info!("Send {packets_in_stream} packets for this bytestream of len {}", bytestream.len());
      }, // end OK(recv)
      Err(err) => {
        error!("Received Garbage! Err {err}");
        continue 'main;
      }
    }// end match 
    if events_not_sent > 0 {
      warn!("There were {events_not_sent} unsent events!");
    }
  } // end outer loop
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
  info!("Resetting blob buffers..");
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
  
  set_readout_all_channels_and_ch9()?;
  thread::sleep(one_milli);
  set_master_trigger_mode()?;
  thread::sleep(one_milli);
  Ok(())
}

