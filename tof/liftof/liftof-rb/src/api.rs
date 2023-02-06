//! Higher level functions, to deal with events/binary reprentation of it, 
//!  configure the drs4, etc.

use local_ip_address::local_ip;

use tof_dataclasses::serialization::Serialization;

use std::collections::HashMap;
use std::net::IpAddr;

use std::time::{Duration,
                Instant};
use std::{thread, time};
use crossbeam_channel::{Sender,
                        Receiver};

// just for fun
use indicatif::{MultiProgress,
                ProgressBar,
                ProgressStyle};
//use indicatif::ProgressStyle;

use crate::control::*;
use crate::memory::*;
use tof_dataclasses::commands::*;

use tof_dataclasses::events::blob::{BlobData,
                                    RBEventPayload};
use tof_dataclasses::serialization::search_for_u16;
use tof_dataclasses::commands::{TofCommand,
                                TofResponse,
                                TofOperationMode};
use tof_dataclasses::packets::{TofPacket,
                               PacketType};
//use tof_dataclasses::threading::ThreadPool;
use tof_dataclasses::monitoring as moni;


/// Non-register related constants 
pub const HEARTBEAT : u64 = 5; // heartbeat in s

const SLEEP_AFTER_REG_WRITE : u32 = 1; // sleep time after register write in ms
const DMA_RESET_TRIES : u8 = 10;   // if we can not reset the DMA after this number
                                   // of retries, we'll panic!
const RESTART_TRIES : u8 = 5; // if we are not successfull, to get it going, 
                                   // panic
/// The 0MQ PUB port is defined as DATAPORT_START + readoutboard_id
const DATAPORT_START : u32 = 30000;

/// The 0MP REP port is defined as CMDPORT_START + readoutboard_id
const CMDPORT_START  : u32 = 40000;

/// Meta information for a data run
pub struct RunParams {
  pub forever   : bool,
  pub nevents   : u32,
  pub is_active : bool,
}

impl RunParams {

  pub fn new() -> RunParams {
    RunParams {
      forever   : false,
      nevents   : 0,
      is_active : false
    }
  }
}

/// Centrailized command management
/// 
/// Maintain 0MQ command connection and faciliate 
/// forwarding of commands and responses
pub fn cmd_responder(rsp_receiver     : &Receiver<TofResponse>,
                     op_mode          : &Sender<TofOperationMode>,
                     run_pars         : &Sender<RunParams>,
                     evid_to_cache    : &Sender<u32>) {
                     //cmd_sender   : &Sender<TofCommand>) {
  // create 0MQ sockedts
  let one_milli   = time::Duration::from_millis(1);
  let mut address_ip = String::from("tcp://");
  let this_board_ip = local_ip().unwrap();
  let cmd_port    = CMDPORT_START + get_board_id().unwrap();


  match this_board_ip {
    IpAddr::V4(ip) => address_ip += &ip.to_string(),
    IpAddr::V6(_) => panic!("Currently, we do not support IPV6!")
  }
  let cmd_address : String = address_ip + ":" + &cmd_port.to_string();
  let ctx = zmq::Context::new();
  let cmd_socket = ctx.socket(zmq::REP).expect("Unable to create 0MQ REP socket!");
  info!("Will set up 0MQ REP socket at address {cmd_address}");
  cmd_socket.bind(&cmd_address).expect("Unable to bind to command socket at {cmd_address}!");
  
  info!("0MQ REP socket listening at {cmd_address}");
 
  // first conenection is a ping
  // is that necesary? 
  //println!("Waiting for client to connect...");
  // block until we get a client
  //let client_response = cmd_socket.recv_bytes(0).expect("Communication to client failed!");
  println!("Client connected");
  //let resp = TofResponse::Success(0);
  //cmd_socket.send(resp.to_bytestream(), 0);
  // whatever client we got, we don't care. It just triggers the call response paatern.
  loop {

    match cmd_socket.poll(zmq::POLLIN, 1) {
      Err(err) => {
        warn!("Polling the 0MQ command socket failed! Err: {err}");
        thread::sleep(one_milli);
        continue;
      }
      Ok(in_waiting) => {
        trace!("poll successful!");
        if in_waiting == 0 {
            continue;
        }
        match cmd_socket.recv_bytes(0) {
          Err(err) => warn!("Problem receiving command over 9MQ !"),
          Ok(cmd_bytes)  => {
            info!("Received bytes {}", cmd_bytes.len());
            match TofCommand::from_bytestream(&cmd_bytes,0) {
              Err(err) => warn!("Problem decoding command {}", err),
              Ok(cmd)  => {
                // we got a valid tof command, forward it and wait for the 
                // response
                let resp_not_implemented = TofResponse::GeneralFail(RESP_ERR_NOTIMPLEMENTED);
                match cmd {
                  TofCommand::Ping (_) => {
                    info!("Received ping signal");
                    let r = TofResponse::Success(0);
                    match cmd_socket.send(r.to_bytestream(),0) {
                      Err(err) => warn!("Can not send response!"),
                      Ok(_)    => info!("Responded to Ping!")
                    }
                    continue;
                  
                  }
                  TofCommand::PowerOn   (mask) => {
                    warn!("Not implemented");
                    match cmd_socket.send(resp_not_implemented.to_bytestream(),0) {
                      Err(err) => warn!("Can not send response!"),
                      Ok(_)    => trace!("Resp sent!")
                    }
                    continue;
                  },
                  TofCommand::PowerOff  (mask) => {
                    warn!("Not implemented");
                    match cmd_socket.send(resp_not_implemented.to_bytestream(),0) {
                      Err(err) => warn!("Can not send response!"),
                      Ok(_)    => trace!("Resp sent!")
                    }
                    continue;
                  },
                  TofCommand::PowerCycle(mask) => {
                    warn!("Not implemented");
                    match cmd_socket.send(resp_not_implemented.to_bytestream(),0) {
                      Err(err) => warn!("Can not send response!"),
                      Ok(_)    => trace!("Resp sent!")
                    }
                    continue;
                  },
                  TofCommand::RBSetup   (mask) => {
                    warn!("Not implemented");
                    match cmd_socket.send(resp_not_implemented.to_bytestream(),0) {
                      Err(err) => warn!("Can not send response!"),
                      Ok(_)    => trace!("Resp sent!")
                    }
                    continue;
                  }, 
                  TofCommand::SetThresholds   (thresholds) =>  {
                    warn!("Not implemented");
                    match cmd_socket.send(resp_not_implemented.to_bytestream(),0) {
                      Err(err) => warn!("Can not send response!"),
                      Ok(_)    => trace!("Resp sent!")
                    }
                    continue;
                  },
                  TofCommand::StartValidationRun  (_) => {
                    warn!("Not implemented");
                    match cmd_socket.send(resp_not_implemented.to_bytestream(),0) {
                      Err(err) => warn!("Can not send response!"),
                      Ok(_)    => trace!("Resp sent!")
                    }
                    continue;
                  },
                  TofCommand::RequestWaveforms (eventid) => {
                    warn!("Not implemented");
                    match cmd_socket.send(resp_not_implemented.to_bytestream(),0) {
                      Err(err) => warn!("Can not send response!"),
                      Ok(_)    => trace!("Resp sent!")
                    }
                    continue;
                  },
                  TofCommand::UnspoolEventCache   (_) => {
                    warn!("Not implemented");
                    match cmd_socket.send(resp_not_implemented.to_bytestream(),0) {
                      Err(err) => warn!("Can not send response!"),
                      Ok(_)    => trace!("Resp sent!")
                    }
                    continue;
                  },
                  TofCommand::StreamOnlyRequested (_) => {
                    let mode = TofOperationMode::TofModeRequestReply;
                    
                    match op_mode.try_send(mode) {
                      Err(err) => trace!("Error sending! {err}"),
                      Ok(_)    => ()
                    }
                    let resp_good = TofResponse::Success(RESP_SUCC_FINGERS_CROSSED);
                    match cmd_socket.send(resp_good.to_bytestream(),0) {
                      Err(err) => warn!("Can not send response!"),
                      Ok(_)    => trace!("Resp sent!")
                    }
                    continue;
                  },
                  TofCommand::StreamAnyEvent      (_) => {
                    let mode = TofOperationMode::TofModeStreamAny;
                    match op_mode.try_send(mode) {
                      Err(err) => trace!("Error sending! {err}"),
                      Ok(_)    => ()
                    }
                    let resp_good = TofResponse::Success(RESP_SUCC_FINGERS_CROSSED);
                    match cmd_socket.send(resp_good.to_bytestream(),0) {
                      Err(err) => warn!("Can not send response!"),
                      Ok(_)    => trace!("Resp sent!")
                    }
                    continue;
                  },
                  TofCommand::DataRunStart (max_event) => {
                    // let's start a run. The value of the TofCommnad shall be 
                    // nevents
                    info!("Will initialize new run!");
                    let run_p = RunParams {
                      forever   : false,
                      nevents   : max_event,
                      is_active : true,
                    };
                    match run_pars.send(run_p) {
                      Err(err) => warn!("Problem initializing run!"),
                      Ok(_)    => ()
                    }
                  }, 
                  TofCommand::DataRunEnd(_)   => {
                    let run_p = RunParams {
                      forever   : false,
                      nevents   : 0,
                      is_active : false,
                    };
                    match run_pars.send(run_p) {
                      Err(err) => warn!("Problem ending run!"),
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
                    match cmd_socket.send(resp_not_implemented.to_bytestream(),0) {
                      Err(err) => warn!("Can not send response!"),
                      Ok(_)    => trace!("Resp sent!")
                    }
                    continue;
                  },
                  TofCommand::TimingCalibration  (_) => {
                    warn!("Not implemented");
                    match cmd_socket.send(resp_not_implemented.to_bytestream(),0) {
                      Err(err) => warn!("Can not send response!"),
                      Ok(_)    => trace!("Resp sent!")
                    }
                    continue;
                  },
                  TofCommand::CreateCalibrationFile (_) => {
                    warn!("Not implemented");
                    match cmd_socket.send(resp_not_implemented.to_bytestream(),0) {
                      Err(err) => warn!("Can not send response!"),
                      Ok(_)    => trace!("Resp sent!")
                    }
                    continue;
                  },
                  TofCommand::RequestEvent(eventid) => {
                    match evid_to_cache.send(eventid) {
                      Err(err) => {
                        debug!("Problem sending event id to cache! Err {err}");
                        //return Ok(TofResponse::GeneralFail(*eventid));
                      },
                      Ok(event) => (),
                    }
                    //continue;
                  },
                  TofCommand::RequestMoni (_) => {
                    warn!("Not implemented");
                    match cmd_socket.send(resp_not_implemented.to_bytestream(),0) {
                      Err(err) => warn!("Can not send response!"),
                      Ok(_)    => trace!("Resp sent!")
                    }
                    continue;
                  },
                  TofCommand::Unknown (_) => {
                    warn!("Not implemented");
                    match cmd_socket.send(resp_not_implemented.to_bytestream(),0) {
                      Err(err) => warn!("Can not send response!"),
                      Ok(_)    => trace!("Resp sent!")
                    }
                    continue;
                  }
                  _ => {
                  }
                } 
             
                // now get the response from the clients
                match rsp_receiver.recv() {
                  Err(err) => {
                    trace!("Did not recv response!");
                    warn!("Intended command receiver did not reply! Responding with Failure");
                    let resp = TofResponse::GeneralFail(RESP_ERR_CMD_STUCK);
                    match cmd_socket.send(resp.to_bytestream(), 0) {
                      Err(err) => warn!("The command likely failed and we could not send a response. This is bad!"),
                      Ok(_)    => trace!("The command likely failed, but we did not lose connection"),
                    }
                  },
                  Ok(resp) => {
                    match cmd_socket.send(resp.to_bytestream(), 0) {
                      Err(err) => warn!("The command likely went through, but we could not send a response. This is bad!"),
                      Ok(_)    => trace!("The command likely went through, but we did not lose connection"),
                    }
                  }
                }
              }
            }  
          }
        }

        //match cmd_receiver.recv() {
        //  Err(err) => {
        //    trace!("Issue receiving command!");
        //    continue;
        //  }
        //  Ok(_)    => {
        //  //match cmd_
        //  }  
        //}
      } 
    }
  }
}


/// Manage the 0MQ PUB socket and send everything 
/// which comes in over the wire as a byte 
/// payload
pub fn data_publisher(data : &Receiver<TofPacket>) {
  let mut address_ip = String::from("tcp://");
  let this_board_ip = local_ip().unwrap();
  let data_port    = DATAPORT_START + get_board_id().unwrap();


  match this_board_ip {
    IpAddr::V4(ip) => address_ip += &ip.to_string(),
    IpAddr::V6(_) => panic!("Currently, we do not support IPV6!")
  }
  let data_address : String = address_ip + ":" + &data_port.to_string();
  let ctx = zmq::Context::new();
  
  // Set up 2 ports for 0MQ communications
  // 1) control flow REP 
  // 2) data flow PUB
  let data_socket = ctx.socket(zmq::PUB).expect("Unable to create 0MQ PUB socket!");
  data_socket.bind(&data_address).expect("Unable to bind to data (PUB) socket {data_adress}");
  info!("0MQ SUB socket bound to address {data_address}");

  loop {
    match data.recv() {
      Err(err) => trace!("Error receiving TofPacket {err}"),
      Ok(packet)    => {
        // pass on the packet downstream
        // wrap the payload INTO THE 
        // FIXME - retries?
        let tp_payload = packet.to_bytestream();
        match data_socket.send(tp_payload,zmq::DONTWAIT) {
          Ok(_)  => trace!("0MQ PUB socket.send() SUCCESS!"),
          Err(err) => warn!("Not able to send over 0MQ PUB socket! Err {err}"),
        }
      }
    }
  }

}

/// Gather monitoring data and pass it on
pub fn monitoring(ch : &Sender<TofPacket>) {
  let heartbeat      = time::Duration::from_secs(HEARTBEAT);
  let mut rate: u32  = 0; 
  let mut bytestream = Vec::<u8>::new();
  bytestream.extend_from_slice(&rate.to_le_bytes());
  loop {
   //if now.elapsed().as_secs() >= HEARTBEAT {
   //}
   let mut moni_dt = moni::RBMoniData::new();
   
   let rate_query = get_trigger_rate();
   match rate_query {
     Ok(rate) => {
       debug!("Monitoring thread -> Rate: {rate}Hz ");
       moni_dt.rate = rate;
       //bytestream = Vec::<u8>::new();
       //bytestream.extend_from_slice(&rate.to_le_bytes());
       //packet.update_payload(bytestream);
     },
     Err(_)   => {
       warn!("Can not send rate monitoring packet, register problem");
     }
   }
   
   let tp = TofPacket::from(&moni_dt);
   //let payload = moni_dt.to_bytestream();
   match ch.try_send(tp) {
     Err(err) => {debug!("Issue sending RBMoniData {:?}", err)},
     Ok(_)    => {debug!("Send RBMoniData successfully!")}
   }

   thread::sleep(heartbeat);
  }
}

/// Read the data buffers when they are full and 
/// then send the stream over the channel to 
/// the thread dealing with it
///
/// # Arguments
///
///
pub fn read_data_buffers(bs_send     : Sender<Vec<u8>>,
                         buff_trip   : u32,
                         bar_a_sender : Option<Sender<u64>>,
                         bar_b_sender : Option<Sender<u64>>,
                         switch_buff : bool) {
  let buf_a = BlobBuffer::A;
  let buf_b = BlobBuffer::B;
  let sleeptime = time::Duration::from_millis(1000);

  //let mut max_buf_a : u64 = 0;
  //let mut max_buf_b : u64 = 0;
  //let mut min_buf_a : u64 = 4294967295;
  //let mut min_buf_b : u64 = 4294967295;
  // let's do some work
  loop {
    //let a_occ = get_blob_buffer_occ(&buf_a).unwrap() as u64;
    //let b_occ = get_blob_buffer_occ(&buf_b).unwrap() as u64;
    //if a_occ > max_buf_a {
    //  max_buf_a = a_occ;
    //  println!("New MAX size for A {max_buf_a}");
    //}
    //if b_occ > max_buf_b  {
    //  max_buf_b = b_occ;
    //  println!("New MAX size for B {max_buf_b}");
    //}
    //if a_occ < min_buf_a {
    //  min_buf_a = a_occ;
    //  println!("New MIN size for A {min_buf_a}");
    //}
    //if b_occ < min_buf_b  {
    //  min_buf_b = b_occ;
    //  println!("New MIN size for B {min_buf_b}");
    //}
    thread::sleep(sleeptime);
    buff_handler(&buf_a,
                 buff_trip,
                 Some(&bs_send),
                 bar_a_sender.clone(),
                 //&bar_a_op, 
                 switch_buff); 
    buff_handler(&buf_b,
                 buff_trip,
                 Some(&bs_send),
                 bar_b_sender.clone(),
                 //&bar_b_op,
                 switch_buff); 
  }
}

/// Somehow, it is not always successful to reset 
/// the DMA and the data buffers. Let's try an 
/// aggressive scheme and do it several times.
/// If we fail, something is wrong and we panic
fn reset_data_memory_aggressively() {
  let one_milli = time::Duration::from_millis(1);
  let five_milli = time::Duration::from_millis(5);
  let buf_a = BlobBuffer::A;
  let buf_b = BlobBuffer::B;
  let mut n_tries : u8 = 0;
  
  for _ in 0..DMA_RESET_TRIES {
    match reset_dma() {
      Ok(_)    => (),
      Err(err) => {
        debug!("Resetting dma failed, err {:?}", err);
        thread::sleep(five_milli);
        continue;
      }
    }
    thread::sleep(one_milli);
  }
  let mut buf_a_occ = UIO1_MAX_OCCUPANCY;
  let mut buf_b_occ = UIO2_MAX_OCCUPANCY;
  match get_blob_buffer_occ(&buf_a) {
    Err(_) => debug!("Error reseting blob buffer A"),
    Ok(val)  => {
      buf_a_occ = val;
    }
  }
  thread::sleep(one_milli);
  match get_blob_buffer_occ(&buf_b) {
    Err(_) => debug!("Error reseting blob buffer B"),
    Ok(val)  => {
      buf_b_occ = val;
    }
  }
  thread::sleep(one_milli);
  while buf_a_occ != UIO1_MIN_OCCUPANCY {
    match blob_buffer_reset(&buf_a) {
      Err(err) => warn!("Problem resetting buffer /dev/uio1 {:?}", err),
      Ok(_)    => () 
    }
    thread::sleep(five_milli);
    match get_blob_buffer_occ(&buf_a) {
      Err(_) => debug!("Error reseting blob buffer A"),
      Ok(val)  => {
        buf_a_occ = val;
        thread::sleep(five_milli);
        n_tries += 1;
        if n_tries == DMA_RESET_TRIES {
          panic!("We were unable to reset DMA and the data buffers!");
        }
        continue;
      }
    }
  }
  n_tries = 0;
  while buf_b_occ != UIO2_MIN_OCCUPANCY {
    match blob_buffer_reset(&buf_b) {
      Err(err) => warn!("Problem resetting buffer /dev/uio2 {:?}", err),
      Ok(_)    => () 
    }
    match get_blob_buffer_occ(&buf_b) {
      Err(_) => warn!("Error getting occupancey for buffer B! (/dev/uio2)"),
      Ok(val)  => {
        buf_b_occ = val;
        thread::sleep(five_milli);
        n_tries += 1;
        if n_tries == DMA_RESET_TRIES {
          panic!("We were unable to reset DMA and the data buffers!");
        }
        continue;
      }
    }
  }
}

///  Ensure the buffers are filled and everything is prepared for data
///  taking
///
///  The whole procedure takes several seconds. We have to find out
///  how much we can sacrifice from our run time.
///
///  # Arguments 
///
///  * will_panic    : The function calls itself recursively and 
///                    will panic after this many calls to itself
///
///  * force_trigger : Run in force trigger mode
///
fn make_sure_it_runs(will_panic : &mut u8,
                     force_trigger : bool) {
  let when_panic : u8 = RESTART_TRIES;
  *will_panic += 1;
  if *will_panic == when_panic {
    // it is hopeless. Let's give up.
    // Let's try to stop the DRS4 before
    // we're killing ourselves
    idle_drs4_daq().unwrap_or(());
    // FIXME - send out Alert
    panic!("I can not get this run to start. I'll kill myself!");
  }
  let five_milli = time::Duration::from_millis(5); 
  let two_secs   = time::Duration::from_secs(2);
  let five_secs  = time::Duration::from_secs(5);
  match idle_drs4_daq() {
    Err(err) => warn!("Issue setting DAQ to idle mode! {}", err),
    Ok(_)    => ()
  }
  thread::sleep(five_milli);
  match setup_drs4() { 
    Err(err) => warn!("Issue running DRS4 setup routine {}", err),
    Ok(_)    => ()
  }
  thread::sleep(five_milli);
  reset_data_memory_aggressively();
  thread::sleep(five_milli);
  if force_trigger {
    disable_master_trigger_mode();
  }

  match start_drs4_daq() {
    Err(err) => {
      debug!("Got err {:?} when trying to start the drs4 DAQ!", err);
    }
    Ok(_)  => {
      trace!("Starting DRS4..");
    }
  }
  // check that the data buffers are filling
  let buf_a = BlobBuffer::A;
  let buf_b = BlobBuffer::B;
  let buf_size_a = get_buff_size(&buf_a).unwrap_or(0);
  let buf_size_b = get_buff_size(&buf_b).unwrap_or(0); 
  thread::sleep(five_secs);
  if get_buff_size(&buf_a).unwrap_or(0) == buf_size_a &&  
      get_buff_size(&buf_b).unwrap_or(0) == buf_size_b {
    warn!("Buffers are not filling! Running setup again!");
    make_sure_it_runs(will_panic, force_trigger);
  } 
}

// palceholder
#[derive(Debug)]
pub struct FIXME {
}


/// Make sure a run stops
///
/// This will recursively call 
/// drs4_idle to stop data taking
///
/// # Arguments:
///
/// * will_panic : After this many calls to 
///                itself, kill_run will 
///                panic.
///
fn kill_run(will_panic : &mut u8) {
  let when_panic : u8 = RESTART_TRIES;
  *will_panic += 1;
  if when_panic == *will_panic {
    panic!("We can not kill the run! I'll kill myself!");
  }
  let one_milli        = time::Duration::from_millis(1);
  match idle_drs4_daq() {
    Ok(_)  => (),
    Err(_) => {
      warn!("Can not end run!");
      thread::sleep(one_milli);
      kill_run(will_panic)
    }
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
///  * max_events     : Acqyire this number of events
///  * max_seconds    : Let go for the specific runtime
///  * max_errors     : End myself when I see a certain
///                     number of errors
///  * kill_signal    : End run when this line is at bool 
///                     1
///  * prog_op_ev     : An option for a progress bar which
///                     is helpful for debugging
///  * force_trigger  : Run in forced trigger mode
///
pub fn runner(max_events          : Option<u64>,
              max_seconds         : Option<u64>,
              max_errors          : Option<u64>,
              progress            : Option<Sender<u64>>,
              run_params          : &Receiver<RunParams>,
              kill_signal         : Option<&Receiver<bool>>,
              force_trigger_rate  : u32) {
  
  let one_milli        = time::Duration::from_millis(1);
  let one_sec          = time::Duration::from_secs(1);
  let mut first_iter   = true; 
  let mut last_evt_cnt : u32 = 0;
  let mut evt_cnt      : u32;
  let mut delta_events : u64 = 0;
  let mut n_events     : u64 = 0;
  let mut n_errors     : u64 = 0;

  let mut timer        = Instant::now();
  let force_trigger    = force_trigger_rate > 0;
  let mut time_between_events : Option<f32> = None;
  if force_trigger {
    warn!("Will run in forced trigger mode with a rate of {force_trigger_rate} Hz!");
    time_between_events = Some(1.0/(force_trigger_rate as f32));
    warn!(".. this means one trigger every {} seconds...", time_between_events.unwrap());
  }

  let now = time::Instant::now();

  let mut terminate = false;
  // the runner will specifically set up the DRS4
  let mut will_panic : u8 = 0;
  let mut is_running = false;
  let mut pars = RunParams::new();
  'cmd: loop {
    if !is_running {
      match run_params.try_recv() {
        Err(err) => {
          info!("Did not receive new RunParams! Err {err}");
          thread::sleep(one_sec);
          continue;
        }
        Ok(p) => {
          info!("Received a new set of RunParams!");
          pars = p;
        }
      }
      // FIXME - the is_active switch is useless
      if pars.is_active {
        info!("Will start a new run!");
        make_sure_it_runs(&mut will_panic, force_trigger);
        info!("Begin Run!");
        is_running = true;
      } else {
        info!("Got new run params, but they don't have the active flag set. Not doing anythign!")
      }
    // as long as we did not see new 
    // run params, wait for them
    continue;
    }
    'run: loop {
      if force_trigger {
        let elapsed = timer.elapsed().as_secs_f32();
        if elapsed > time_between_events.unwrap() {
          timer = Instant::now(); 
          trigger();
        } else {
          continue;
        }
      }

      match get_event_count() {
        Err (err) => {
          debug!("Can not obtain event count! Err {:?}", err);
          thread::sleep(one_sec);    
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
            thread::sleep(one_sec);
            info!("We didn't get an updated event count!");
            continue;
          }
        }
      } // end match

      delta_events = (evt_cnt - last_evt_cnt) as u64;
      n_events += delta_events;
      last_evt_cnt = evt_cnt;
      
      match &progress { 
        None => (),
        Some(sender) => {
          match sender.try_send(delta_events) {
            Err(err) => trace!("Error sending {err}"),
            Ok(_)    => ()
          }
        }
      }
      debug!("Checking for kill signal");
      // terminate if one of the 
      // criteria is fullfilled
      match kill_signal {
        Some(ks) => {
          match ks.recv() {
            Ok(signal) => {
              warn!("Have received kill signal!");
              terminate = signal;
            },
            Err(_) => {
              info!("Did not get kill signal!");
            }
          }
        },
        None => ()
      }
      
      if !pars.forever {
        match max_events {
          None => (),
          Some(max_e) => {
            if n_events > max_e {
              terminate = true;
            }
          }
        }
        
        match max_seconds {
          None => (),
          Some(max_t) => {
            if now.elapsed().as_secs() > max_t {
              terminate = true;
            }
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
      }
      // exit loop on n event basis
      if terminate {
        break 'run;
      }
      // save cpu
      //thread::sleep(one_sec);
    } // end 'run loop 
  } // end 'cmd loop
  // if the end condition is met, we stop the run
  let mut will_panic : u8 = 0;
  kill_run(&mut will_panic);
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
pub fn event_cache_worker(recv_ev_pl    : Receiver<RBEventPayload>,
                          //cmd_from_cmdr : &Receiver<TofCommand>,
                          //send_ev_pl  : Sender<Option<RBEventPayload>>,
                          tp_to_pub    : &Sender<TofPacket>,
                          //hasit_to_cmd : &Sender<bool>,
                          resp_to_cmd  : &Sender<TofResponse>,
                          get_op_mode  : Receiver<TofOperationMode>, 
                          recv_evid    : Receiver<u32>,
                          cache_size   : usize) {

  let mut n_send_errors = 0;   
  let mut op_mode_stream = false;

  let mut oldest_event_id : u32 = 0;
  let mut event_cache : HashMap::<u32, RBEventPayload> = HashMap::new();
  loop {
    // check changes in operation mode
    match get_op_mode.try_recv() {
      Err(err) => trace!("No op mode change detected! Err {err}"),
      Ok(mode) => {
        warn!("Will change operation mode to {:?}!", mode);
        match mode {
          TofOperationMode::TofModeRequestReply => {op_mode_stream = false;},
          TofOperationMode::TofModeStreamAny    => {op_mode_stream = true;},
        }
      }
    }
    // store incoming events in the cache  
    match recv_ev_pl.try_recv() {
      Err(err) => {
        trace!("No event payload! {err}");
        //continue;
      } // end err
      Ok(event)  => {
        trace!("Received next RBEvent!");
        if oldest_event_id == 0 {
          oldest_event_id = event.event_id;
        } //endif
        // store the event in the cache
        trace!("Received payload with event id {}" ,event.event_id);
        event_cache.insert(event.event_id, event);   
        // keep track of the oldest event_id
        debug!("We have a cache size of {}", event_cache.len());
        if event_cache.len() > cache_size {
          event_cache.remove(&oldest_event_id);
          oldest_event_id += 1;
        } //endif
      }// end Ok
    } // end match
  
    // if we are in "stream_any" mode, we don't need to take care
    // of any fo the response/request.
    if op_mode_stream {
      //event_cache.as_ref().into_iter().map(|(evid, payload)| {send_ev_pl.try_send(Some(payload))});
      //let evids = event_cache.keys();
      for payload in event_cache.values() {
        // FIXME - this is bad! Too much allocation
        let tp = TofPacket::from(payload);
        //info!("{}", tp);
        match tp_to_pub.try_send(tp) {
          Err(err) => {
            trace!("Error sending! {err}");
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
          let event = event_cache.remove(&event_id).unwrap();
          let resp =  TofResponse::Success(event_id);
          match resp_to_cmd.try_send(resp) {
            Err(err) => trace!("Error informing the commander that we do have {event_id}! Err {err}"),
            Ok(_)    => ()
          }
          let tp = TofPacket::from(&event);
          //match send_ev_pl.try_send(Some(event)) {
          match tp_to_pub.try_send(tp) {
            Err(err) => trace!("Error sending! {err}"),
            Ok(_)    => ()
          }
        }
      }
    } // end match
  } // end loop
}

/// Deal with incoming commands
///
///
///
///
pub struct Commander<'a> {

  pub evid_send        : Sender<u32>,
  pub change_op_mode   : Sender<TofOperationMode>, 
  pub rb_evt_recv      : Receiver<Option<RBEventPayload>>,
  pub hasit_from_cache : &'a Receiver<bool>,
}

impl Commander<'_> {

  pub fn new<'a> (send_ev          : Sender<u32>,
                  hasit_from_cache : &'a Receiver<bool>,
                  evpl_from_cache  : Receiver<Option<RBEventPayload>>,
                  change_op_mode   : Sender<TofOperationMode>)
    -> Commander<'a> {

    Commander {
      evid_send        : send_ev,
      change_op_mode   : change_op_mode,
      rb_evt_recv      : evpl_from_cache,
      hasit_from_cache : hasit_from_cache,
    }
  }


  /// Interpret an incoming command 
  ///
  /// The command comes most likely somehow over 
  /// the wir from the tof computer
  ///
  /// Match with a list of known commands and 
  /// take action.
  ///
  /// # Arguments
  ///
  /// * command : A TofCommand instructing the 
  ///             commander what to do
  ///             Will generate a TofResponse 
  ///             
  pub fn command (&self, cmd : &TofCommand)
    -> Result<TofResponse, FIXME> {
    match cmd {
      TofCommand::PowerOn   (mask) => {
        warn!("Not implemented");
        return Ok(TofResponse::GeneralFail(RESP_ERR_NOTIMPLEMENTED));
      },
      TofCommand::PowerOff  (mask) => {
        warn!("Not implemented");
        return Ok(TofResponse::GeneralFail(RESP_ERR_NOTIMPLEMENTED));
      },
      TofCommand::PowerCycle(mask) => {
        warn!("Not implemented");
        return Ok(TofResponse::GeneralFail(RESP_ERR_NOTIMPLEMENTED));
      },
      TofCommand::RBSetup   (mask) => {
        warn!("Not implemented");
        return Ok(TofResponse::GeneralFail(RESP_ERR_NOTIMPLEMENTED));
      }, 
      TofCommand::SetThresholds   (thresholds) =>  {
        warn!("Not implemented");
        return Ok(TofResponse::GeneralFail(RESP_ERR_NOTIMPLEMENTED));
      },
      TofCommand::StartValidationRun  (_) => {
        warn!("Not implemented");
        return Ok(TofResponse::GeneralFail(RESP_ERR_NOTIMPLEMENTED));
      },
      TofCommand::RequestWaveforms (eventid) => {
        warn!("Not implemented");
        return Ok(TofResponse::GeneralFail(RESP_ERR_NOTIMPLEMENTED));
      },
      TofCommand::UnspoolEventCache   (_) => {
        warn!("Not implemented");
        return Ok(TofResponse::GeneralFail(RESP_ERR_NOTIMPLEMENTED));
      },
      TofCommand::StreamOnlyRequested (_) => {
        let op_mode = TofOperationMode::TofModeRequestReply;
        
        match self.change_op_mode.try_send(op_mode) {
          Err(err) => trace!("Error sending! {err}"),
          Ok(_)    => ()
        }
        return Ok(TofResponse::Success(RESP_SUCC_FINGERS_CROSSED));
      },
      TofCommand::StreamAnyEvent      (_) => {
        let op_mode = TofOperationMode::TofModeStreamAny;
        match self.change_op_mode.try_send(op_mode) {
          Err(err) => trace!("Error sending! {err}"),
          Ok(_)    => ()
        }
        return Ok(TofResponse::Success(RESP_SUCC_FINGERS_CROSSED));
      },
      //TofCommand::DataRunStart (max_event) => {
      //  // let's start a run. The value of the TofCommnad shall be 
      //  // nevents
      //  self.workforce.execute(move || {
      //      runner(Some(*max_event as u64),
      //             None,
      //             None,
      //             self.get_killed_chn,
      //             None);
      //  }); 
      //  return Ok(TofResponse::Success(RESP_SUCC_FINGERS_CROSSED));
      //}, 
      //TofCommand::DataRunEnd   => {
      //  if !self.run_active {
      //    return Ok(TofResponse::GeneralFail(RESP_ERR_NORUNACTIVE));
      //  }
      //  warn!("Will kill current run!");
      //  self.kill_chn.send(true);
      //  return Ok(TofResponse::Success(RESP_SUCC_FINGERS_CROSSED));
      //},
      TofCommand::VoltageCalibration (_) => {
        warn!("Not implemented");
        return Ok(TofResponse::GeneralFail(RESP_ERR_NOTIMPLEMENTED));
      },
      TofCommand::TimingCalibration  (_) => {
        warn!("Not implemented");
        return Ok(TofResponse::GeneralFail(RESP_ERR_NOTIMPLEMENTED));
      },
      TofCommand::CreateCalibrationFile (_) => {
        warn!("Not implemented");
        return Ok(TofResponse::GeneralFail(RESP_ERR_NOTIMPLEMENTED));
      },
      TofCommand::RequestEvent(eventid) => {
        match self.evid_send.send(*eventid) {
          Err(err) => {
            debug!("Problem sending event id to cache! Err {err}");
            return Ok(TofResponse::GeneralFail(*eventid));
          },
          Ok(event) => (),
        }
        match self.hasit_from_cache.recv() {
          Err(err) => {
            return Ok(TofResponse::EventNotReady(*eventid));
          }
          Ok(hasit) => {
            // FIXME - prefix topic
            if hasit {
              return Ok(TofResponse::Success(*eventid));
            } else {
              return Ok(TofResponse::EventNotReady(*eventid));
            }
            //Some(event) => {
            //  match self.zmq_pub_socket.send(event.payload, zmq::DONTWAIT) {
            //    Ok(_)  => {
            //      return Ok(TofResponse::Success(*eventid));
            //    }
            //    Err(err) => {
            //      debug!("Problem with PUB socket! Err {err}"); 
            //      return Ok(TofResponse::ZMQProblem(*eventid));
            //    }
            //  }
            //}
            //}
          }
        }
      },
      TofCommand::RequestMoni (_) => {
      },
      TofCommand::Unknown (_) => {
      }
      _ => {
      }
    } 
    let response = TofResponse::Success(1);
    Ok(response)
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
pub fn get_buff_size(which : &BlobBuffer) ->Result<u32, RegisterError> {
  let size : u32;
  let occ = get_blob_buffer_occ(&which)?;
  trace!("Got occupancy of {occ} for buff {which:?}");

  // the buffer sizes is UIO1_MAX_OCCUPANCY -  occ
  match which {
    BlobBuffer::A => {size = occ - UIO1_MIN_OCCUPANCY;},
    BlobBuffer::B => {size = occ - UIO2_MIN_OCCUPANCY;}
  }
  Ok(size)
}

///  Deal with the raw data buffers.
///
///  Read out when they exceed the 
///  tripping threshold and pass 
///  on the result.
///
///  # Arguments:
///
///  * buff_trip : size which triggers buffer readout.
pub fn buff_handler(which       : &BlobBuffer,
                    buff_trip   : u32,
                    bs_sender   : Option<&Sender<Vec<u8>>>,
                    prog_sender : Option<Sender<u64>>,
                    //prog_bar    : &Option<Box<ProgressBar>>,
                    switch_buff : bool) {
  let sleep_after_reg_write = Duration::from_millis(SLEEP_AFTER_REG_WRITE as u64);
  let buff_size : u32;
  match get_buff_size(&which) {
    Ok(bf)   => { 
      buff_size = bf;
    },
    Err(err) => { 
      debug!("Error getting buff size! {:?}", err);
      buff_size = 0;
    }
  }

  let has_tripped = buff_size >= buff_trip;

  if has_tripped {
    debug!("Buff {which:?} tripped at a size of {buff_size}");  
    debug!("Buff size {buff_size}");
    // reset the buffers
    if switch_buff {
      match switch_ram_buffer() {
        Ok(_)  => debug!("Ram buffer switched!"),
        Err(_) => warn!("Unable to switch RAM buffers!") 
      }
    }
    //thread::sleep_ms(SLEEP_AFTER_REG_WRITE);
    let bytestream = read_data_buffer(&which, buff_size as usize).unwrap();
    if bs_sender.is_some() {
      match bs_sender.unwrap().send(bytestream) {
        Err(err) => trace!("error sending {err}"),
        Ok(_)    => ()
      }
    }
    //match bs_sender {
    //  Some(snd) => snd.send(bytestream),
    //  None      => Ok(()),
    //};
    
    match blob_buffer_reset(&which) {
      Ok(_)  => debug!("Successfully reset the buffer occupancy value"),
      Err(_) => warn!("Unable to reset buffer!")
    }
    match &prog_sender {
      None => (),
      Some(up) => {
        match up.try_send(0) {
          Err(err) => trace!("Can not send!"),
          Ok(_) => ()
        }
      }
    }
    thread::sleep(sleep_after_reg_write);
  } else { // endf has tripped
    match &prog_sender {
      None => (),
      Some(up) => {
        match up.try_send(buff_size as u64) {
          Err(err) => trace!("Sending faile with error {err}"),
          Ok(_)    => ()
        }
      }
    }
    //match prog_bar {
    //  Some(bar) => bar.set_position(buff_size as u64),
    //  None      => () 
    //}
  }
}

/////! FIXME - should become a feature
//pub fn setup_progress_bar(msg : String, size : u64, format_string : String) -> ProgressBar {
//  let mut bar = ProgressBar::new(size).with_style(
//    //ProgressStyle::with_template("[{elapsed_precise}] {bar:40.cyan/blue} {pos:>7}/{len:7} {msg}")
//    ProgressStyle::with_template(&format_string)
//    .unwrap()
//    .progress_chars("##-"));
//  //);
//  bar.set_message(msg);
//  //bar.finish_and_clear();
//  ////let mut style_found = false;
//  //let style_ok = ProgressStyle::with_template("[{elapsed_precise}] {bar:40.cyan/blue} {pos:>7}/{len:7} {msg}");
//  //match style_ok {
//  //  Ok(_) => { 
//  //    style_found = true;
//  //  },
//  //  Err(ref err)  => { warn!("Can not go with chosen style! Not using any! Err {err}"); }
//  //}  
//  //if style_found { 
//  //  bar.set_style(style_ok.unwrap()
//  //                .progress_chars("##-"));
//  //}
//  bar
//}


///  Transforms raw bytestream to RBEventPayload
///
///  This allows to get the eventid from the 
///  binrary form of the RBEvent
///
///  #Arguments
/// 
///  * bs_recv   : A receiver for bytestreams. The 
///                bytestream comes directly from 
///                the data buffers.
///  * ev_sender : Send the the payload to the event cache
pub fn event_payload_worker(bs_recv   : &Receiver<Vec<u8>>,
                            ev_sender : Sender<RBEventPayload>) {
  let mut n_events : u32;
  let mut event_id : u32 = 0;
  'main : loop {
    let mut start_pos : usize = 0;
    n_events = 0;
    //let mut debug_evids = Vec::<u32>::new();
    match bs_recv.recv() {
      Ok(bytestream) => {
        'bytestream : loop {
          match search_for_u16(BlobData::HEAD, &bytestream, start_pos) {
            Ok(head_pos) => {
              let tail_pos   = head_pos + BlobData::SERIALIZED_SIZE;
              if tail_pos > bytestream.len() - 1 {
                // we are finished here
                trace!("Work on current blob complete. Extracted {n_events} events. Got last event_id! {event_id}");
                //trace!("{:?}", debug_evids);
                break 'bytestream;
              }
              event_id   = BlobData::decode_event_id(&bytestream[head_pos..tail_pos]);
              //debug_evids.push(event_id);
              //info!("Got event_id {event_id}");
              n_events += 1;
              start_pos = tail_pos;
              let mut payload = Vec::<u8>::new();
              payload.extend_from_slice(&bytestream[head_pos..tail_pos]);
              trace!("Got payload size {}", &payload.len());
              let rb_payload = RBEventPayload::new(event_id, payload); 
              match ev_sender.send(rb_payload) {
                Ok(_) => (),
                Err(err) => debug!("Problem sending RBEventPayload over channel! Err {err}"),
              }
              continue 'bytestream;
            },
            Err(err) => {
              println!("Send {n_events} events. Got last event_id! {event_id}");
              warn!("Got bytestream, but can not find HEAD bytes, err {err:?}");
              break 'bytestream;}
          } // end loop
        } // end ok
      }, // end Ok(bytestream)
      Err(err) => {
        warn!("Received Garbage! Err {err}");
        continue 'main;
      }
    }// end match 
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
  idle_drs4_daq()?;
  thread::sleep(one_milli);
  set_drs4_configure()?;
  thread::sleep(one_milli);

  // Sanity checking
  //let max_samples     : u32 = 65000;
  //let max_duration    : u32 = 1440; // Minutes in 1 day

  reset_daq()?;
  thread::sleep(one_milli);
  
  reset_dma()?;
  thread::sleep(one_milli);
  clear_dma_memory()?;
  thread::sleep(one_milli);
  
  
  // for some reason, sometimes it 
  // takes a bit until the blob
  // buffers reset. Let's try a 
  // few times
  info!("Resetting blob buffers..");
  for _ in 0..5 {
    blob_buffer_reset(&buf_a)?;
    thread::sleep(one_milli);
    blob_buffer_reset(&buf_b)?;
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

/// Show progress bars for demonstration
/// 
/// The bar is a multibar and has 3 individual
/// bars - 2 for the buffers and one for 
/// the number of events seen
/// Intended to be run in a seperate thread
///
pub fn progress_runner(max_events      : u64,
                       uio1_total_size : u64,
                       uio2_total_size : u64,
                       update_bar_a    : Receiver<u64>,
                       update_bar_b    : Receiver<u64>,
                       update_bar_ev   : Receiver<u64>,
                       finish_bars     : Receiver<bool>){
  let template_bar_a   : &str = "[{elapsed_precise}] {prefix} {msg} {spinner} {bar:60.blue/grey} {bytes:>7}/{total_bytes:7} ";
  let template_bar_b   : &str = "[{elapsed_precise}] {prefix} {msg} {spinner} {bar:60.green/grey} {bytes:>7}/{total_bytes:7} ";
  let template_bar_env : &str = "[{elapsed_precise}] {prefix} {msg} {spinner} {bar:60.red/grey} {pos:>7}/{len:7}";
  let floppy    = vec![240, 159, 146, 190];
  let floppy    = String::from_utf8(floppy).unwrap();
  let sparkles  = vec![226, 156, 168];
  let sparkles  = String::from_utf8(sparkles).unwrap();

  let label_a   = String::from("Buff A");
  let label_b   = String::from("Buff B");
  let sty_a = ProgressStyle::with_template(template_bar_a)
  .unwrap();
  //.progress_chars("##-");
  let sty_b = ProgressStyle::with_template(template_bar_b)
  .unwrap();
  //.progress_chars("##-");
  let sty_ev = ProgressStyle::with_template(template_bar_env)
  .unwrap();
  //.progress_chars("##>");
  let multi_prog = MultiProgress::new();

  let prog_a  = multi_prog
                .add(ProgressBar::new(uio1_total_size)); 
  let prog_b  = multi_prog
                .insert_after(&prog_a, ProgressBar::new(uio2_total_size)); 
  let prog_ev = multi_prog
                .insert_after(&prog_b, ProgressBar::new(max_events)); 
  
  prog_a.set_message (label_a);
  prog_a.set_prefix  (floppy.clone());
  prog_a.set_style   (sty_a);
  prog_b.set_message (label_b);
  prog_b.set_prefix  (floppy);
  prog_b.set_style   (sty_b);
  prog_ev.set_style  (sty_ev);
  prog_ev.set_prefix (sparkles);
  prog_ev.set_message("EVENTS");

  let sleep_time  = time::Duration::from_millis(1);

  loop {
    match update_bar_a.try_recv() {
      Err(err) => trace!("No update, err {err}"),
      Ok(val)  => {
        prog_a.set_position(val);
      }
    }
    match update_bar_b.try_recv() {
      Err(err) => trace!("No update, err {err}"),
      Ok(val)  => {
        prog_b.set_position(val);
        //prog_b.inc(val);
      }
    }
    match update_bar_ev.try_recv() {
      Err(err) => trace!("No update, err {err}"),
      Ok(val)  => {
        prog_ev.inc(val);
      }
    }
    match finish_bars.try_recv() {
      Err(err) => trace!("No update, err {err}"),
      Ok(val)  => {
        prog_a.finish();
        prog_b.finish();
        prog_ev.finish();
      }
    }
  thread::sleep(sleep_time);
  }
}
