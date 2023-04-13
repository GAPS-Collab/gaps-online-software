//! Higher level functions, to deal with events/binary reprentation of it, 
//!  configure the drs4, etc.

use local_ip_address::local_ip;

use tof_dataclasses::serialization::Serialization;

use std::collections::HashMap;
use std::net::IpAddr;
use std::fs::File;
use std::path::Path;

use std::io::Write;
use std::fs::OpenOptions;

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
use tof_dataclasses::errors::SerializationError;
use liftof_lib::RunParams;


/// Non-register related constants 
pub const HEARTBEAT : u64 = 5; // heartbeat in s

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
/// Commandport is 0MQ SUB for receiving commands from a C&C server
pub const CMDPORT  : u32 = 32000;


// FIXME
type RamBuffer = BlobBuffer;

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
  TofCommand::from_bytestream(&bytestream, 4)
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
                     rsp_receiver              : &Receiver<TofResponse>,
                     op_mode                   : &Sender<TofOperationMode>,
                     run_pars                  : &Sender<RunParams>,
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
  cmd_socket.connect(&cmd_address).expect("Unable to bind to command socket at {cmd_address}!");
  //let my_topic = String::from("");
  //.as_bytes();
  //cmd_socket.set_subscribe(&my_topic.as_bytes());
  match cmd_socket.set_subscribe(&topic_broadcast.as_bytes()) {
    Err(err) => error!("Unable to subscribe to {topic_broadcast}, error {err}"),
    Ok(_) => ()
  }
  match cmd_socket.set_subscribe(&topic_board.as_bytes()) {
    Err(err) => error!("Unable to subscribe to {topic_board}, error {err}"),
    Ok(_) => ()
  }
  let mut heartbeat     = Instant::now();

  error!("TODO: Heartbeat feature not yet implemented on C&C side");
  let heartbeat_received = false;
  loop {
    if !heartbeat_received {
      if heartbeat.elapsed().as_secs() > heartbeat_timeout_seconds as u64 {
        warn!("No heartbeat received since {heartbeat_timeout_seconds}. Attempting to reconnect!");
        cmd_socket.connect(&cmd_address).expect("Unable to bind to command socket at {cmd_address}!");
        //cmd_socket.set_subscribe(&my_topic.as_bytes());
        match cmd_socket.set_subscribe(&topic_broadcast.as_bytes()) {
          Err(err) => error!("Can not subscribe to {topic_broadcast}, err {err}"),
          Ok(_)    => ()
        }
        match cmd_socket.set_subscribe(&topic_board.as_bytes()) {
          Err(err) => error!("Can not subscribe to {topic_board}, err {err}"),
          Ok(_)    => ()
        }
        heartbeat = Instant::now();
      }
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
    match cmd_socket.recv_bytes(0) {
      Err(err) => error!("Problem receiving command over 0MQ ! Err {err}"),
      Ok(cmd_bytes)  => {
        info!("Received bytes {}", cmd_bytes.len());
        match TofCommand::from_bytestream(&cmd_bytes,4) {
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
              TofCommand::PowerOn   (mask) => {
                warn!("Not implemented");
                match cmd_socket.send(resp_not_implemented,0) {
                  Err(err) => warn!("Can not send response! Err {err}"),
                  Ok(_)    => trace!("Resp sent!")
                }
                continue;
              },
              TofCommand::PowerOff  (mask) => {
                warn!("Not implemented");
                match cmd_socket.send(resp_not_implemented,0) {
                  Err(err) => warn!("Can not send response! {err}"),
                  Ok(_)    => trace!("Resp sent!")
                }
                continue;
              },
              TofCommand::PowerCycle(mask) => {
                warn!("Not implemented");
                match cmd_socket.send(resp_not_implemented,0) {
                  Err(err) => warn!("Can not send response! {err}"),
                  Ok(_)    => trace!("Resp sent!")
                }
                continue;
              },
              TofCommand::RBSetup   (mask) => {
                warn!("Not implemented");
                match cmd_socket.send(resp_not_implemented,0) {
                  Err(err) => warn!("Can not send response! Err {err}"),
                  Ok(_)    => trace!("Resp sent!")
                }
                continue;
              }, 
              TofCommand::SetThresholds   (thresholds) =>  {
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
                  Err(err) => warn!("Can not send response!"),
                  Ok(_)    => trace!("Resp sent!")
                }
                continue;
              },
              TofCommand::RequestWaveforms (eventid) => {
                trace!("Requesting waveforms for event {eventid}");
                error!("Not implemented");
                match cmd_socket.send(resp_not_implemented,0) {
                  Err(err) => warn!("Can not send response!"),
                  Ok(_)    => trace!("Resp sent!")
                }
                continue;
              },
              TofCommand::UnspoolEventCache   (_) => {
                warn!("Not implemented");
                match cmd_socket.send(resp_not_implemented,0) {
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
                  forever   : true,
                  nevents   : 0,
                  is_active : true,
                  nseconds  : 0,
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
                  nseconds : 0,
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
                match cmd_socket.send(resp_not_implemented,0) {
                  Err(err) => warn!("Can not send response!"),
                  Ok(_)    => trace!("Resp sent!")
                }
                continue;
              },
              TofCommand::TimingCalibration  (_) => {
                warn!("Not implemented");
                match cmd_socket.send(resp_not_implemented,0) {
                  Err(err) => warn!("Can not send response!"),
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
                    debug!("Problem sending event id to cache! Err {err}");
                    //return Ok(TofResponse::GeneralFail(*eventid));
                  },
                  Ok(event) => (),
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
pub fn data_publisher(data : &Receiver<TofPacket>,
                      write_blob  : bool,
                      file_suffix : Option<&str> ) {
  let mut address_ip = String::from("tcp://");
  let this_board_ip = local_ip().expect("Unable to obtainl local board IP. Something is messed up!");
  let data_port    = DATAPORT;

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
  let blobfile_name = "tof-rb".to_owned()
                       + &board_id.to_string()
                       + file_suffix.unwrap_or(".blob");

  let blobfile_path = Path::new(&blobfile_name);
  

  let mut file_on_disc : Option<File> = None;//let mut output = File::create(path)?;
  if write_blob {
    info!("Writing blobs to {}", blobfile_name );
    file_on_disc = OpenOptions::new().append(true).create(true).open(blobfile_path).ok()
  }


  loop {
    match data.recv() {
      Err(err) => trace!("Error receiving TofPacket {err}"),
      Ok(packet)    => {
        // pass on the packet downstream
        // wrap the payload INTO THE 
        // FIXME - retries?
        if write_blob {
          //println!("{:?}", packet.packet_type);
          if packet.packet_type == PacketType::RBEvent {
            //println!("is rb event");
            match &mut file_on_disc {
              None => error!("We want to write data, however the file is invalid!"),
              Some(f) => {
                //println!("write to file");
                match f.write_all(packet.payload.as_slice()) {
                  Err(err) => error!("Writing file to disk failed! Err {err}"),
                  Ok(()) => ()
                }
              }
            }
          }
        }
        let tp_payload = prefix_board_id(&mut packet.to_bytestream());
        match data_socket.send(tp_payload,zmq::DONTWAIT) {
          Ok(_)    => trace!("0MQ PUB socket.send() SUCCESS!"),
          Err(err) => error!("Not able to send over 0MQ PUB socket! Err {err}"),
        }
      }
    }
  }

}

/// Gather monitoring data and pass it on
pub fn monitoring(ch : &Sender<TofPacket>) {
  let heartbeat      = time::Duration::from_secs(HEARTBEAT);
  loop {

   let mut moni_dt = moni::RBMoniData::new();
   
   let rate_query = get_trigger_rate();
   match rate_query {
     Ok(rate) => {
       debug!("Monitoring thread -> Rate: {rate}Hz ");
       moni_dt.rate = rate;
     },
     Err(_)   => {
       warn!("Can not send rate monitoring packet, register problem");
     }
   }
   
   let tp = TofPacket::from(&moni_dt);
   match ch.try_send(tp) {
     Err(err) => {debug!("Issue sending RBMoniData {:?}", err)},
     Ok(_)    => {debug!("Send RBMoniData successfully!")}
   }

   thread::sleep(heartbeat);
  }
}

/// Reset DMA pointer and buffer occupancy registers
///
/// If there are any errors, we will wait for a short
/// time and then try again
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
    }
    failed = false;
  } else {
    break;
  }
}

///// Somehow, it is not always successful to reset 
///// the DMA and the data buffers. Let's try an 
///// aggressive scheme and do it several times.
///// If we fail, something is wrong and we panic
//pub fn reset_data_memory_aggressively() {
//  //let one_milli = time::Duration::from_millis(1);
//  let five_milli = time::Duration::from_millis(5);
//  let buf_a = BlobBuffer::A;
//  let buf_b = BlobBuffer::B;
//  let n_tries : u8 = 0;
//  
//  for _ in 0..DMA_RESET_TRIES {
//    match reset_dma() {
//      Ok(_)    => break,
//      Err(err) => {
//        debug!("Resetting dma failed, err {:?}", err);
//        thread::sleep(five_milli);
//        continue;
//      }
//    }
//  }
//  //let mut buf_a_occ = UIO1_MAX_OCCUPANCY;
//  //let mut buf_b_occ = UIO2_MAX_OCCUPANCY;
//  match reset_ram_buffer_occ(&buf_a) {
//    Err(err) => warn!("Problem resetting buffer /dev/uio1 {:?}", err),
//    Ok(_)    => () 
//  }
//  match reset_ram_buffer_occ(&buf_b) {
//    Err(err) => warn!("Problem resetting buffer /dev/uio1 {:?}", err),
//    Ok(_)    => () 
//  }
//  //match get_blob_buffer_occ(&buf_a) {
//  //  Err(err) => debug!("Error getting blob buffer A occupnacy {err}"),
//  //  Ok(val)  => {
//  //    debug!("Got a value for the buffer A of {val}");
//  //    buf_a_occ = val;
//  //  }
//  //}
//  //thread::sleep(one_milli);
//  //match get_blob_buffer_occ(&buf_b) {
//  //  Err(err) => {
//  //    warn!("Error getting blob buffer B occupancy {err}");
//  //  }
//  //  Ok(val)  => {
//  //    debug!("Got a value for the buffer B of {val}");
//  //    buf_b_occ = val;
//  //  }
//
//  //}
//  //thread::sleep(one_milli);
//  //while buf_a_occ != UIO1_MIN_OCCUPANCY {
//
//  //  match reset_ram_buffer_occ(&buf_a) {
//  //    Err(err) => warn!("Problem resetting buffer /dev/uio1 {:?}", err),
//  //    Ok(_)    => () 
//  //  }
//  //  thread::sleep(five_milli);
//  //  match get_blob_buffer_occ(&buf_a) {
//  //    Err(_) => {
//  //      warn!("Error reseting blob buffer A");
//  //      thread::sleep(five_milli);
//  //      }
//  //      continue;
//  //    }
//  //    Ok(val)  => {
//  //      buf_a_occ = val;
//  //    }
//  //  }
//  //}
//  //n_tries = 0;
//  //while buf_b_occ != UIO2_MIN_OCCUPANCY {
//  //  match reset_ram_buffer_occ(&buf_b) {
//  //    Err(err) => warn!("Problem resetting buffer /dev/uio2 {:?}", err),
//  //    Ok(_)    => () 
//  //  }
//  //  match get_blob_buffer_occ(&buf_b) {
//  //    Err(_) => {
//  //      warn!("Error getting occupancey for buffer B! (/dev/uio2)");
//  //      thread::sleep(five_milli);
//  //      n_tries += 1;
//  //      if n_tries == DMA_RESET_TRIES {
//  //        panic!("We were unable to reset DMA and the data buffers!");
//  //      }
//  //      continue;
//  //    }
//  //    Ok(val)  => {
//  //      buf_b_occ = val;
//  //    }
//  //  }
//  //}
//}

/////  Ensure the buffers are filled and everything is prepared for data
/////  taking
/////
/////  The whole procedure takes several seconds. We have to find out
/////  how much we can sacrifice from our run time.
/////
/////  # Arguments 
/////
/////  * will_panic    : The function calls itself recursively and 
/////                    will panic after this many calls to itself
/////
/////  * force_trigger : Run in force trigger mode
/////
//fn make_sure_it_runs(will_panic : &mut u8,
//                     force_trigger : bool) {
//  let when_panic : u8 = RESTART_TRIES;
//  *will_panic += 1;
//  if *will_panic == when_panic {
//    // it is hopeless. Let's give up.
//    // Let's try to stop the DRS4 before
//    // we're killing ourselves
//    disable_trigger();
//    //idle_drs4_daq().unwrap_or(());
//    // FIXME - send out Alert
//    panic!("I can not get this run to start. I'll kill myself!");
//  }
//
//
//  let five_milli = time::Duration::from_millis(5); 
//  let one_sec    = time::Duration::from_secs(1);
//  let two_secs   = time::Duration::from_secs(2);
//  let five_secs  = time::Duration::from_secs(5);
//  thread::sleep(five_milli);
//  if force_trigger {
//    match disable_master_trigger_mode() {
//      Err(err) => error!("Can not disable master trigger mode, Err {err}"),
//      Ok(_)    => info!("Master trigger mode didsabled!")
//    }
//  }
//
//  match enable_trigger() {
//    Err(err) => error!("Can not enable triggers! Err {err}"),
//    Ok(_)    => trace!("Triggers enabled")
//  }
//  //println!("triggers enabled");
//  // check that the data buffers are filling
//  let buf_a = BlobBuffer::A;
//  let buf_b = BlobBuffer::B;
//  let buf_size_a = get_buff_size(&buf_a).unwrap_or(0);
//  let buf_size_b = get_buff_size(&buf_b).unwrap_or(0); 
//  thread::sleep(one_sec);
//  if get_buff_size(&buf_a).unwrap_or(0) == buf_size_a &&  
//      get_buff_size(&buf_b).unwrap_or(0) == buf_size_b {
//    error!("Buffers are not filling! Running setup again!");
//    make_sure_it_runs(will_panic, force_trigger);
//  } 
//}

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
pub fn runner(run_params          : &Receiver<RunParams>,
              buffer_trip         : usize,
              max_errors          : Option<u64>,
              bs_sender           : &Sender<Vec<u8>>,
              uio1_total_size     : usize,
              uio2_total_size     : usize,
              mut latch_to_mtb    : bool,
              show_progress       : bool,
              force_trigger_rate  : u32) {
  
  let one_milli        = time::Duration::from_millis(1);
  let one_sec          = time::Duration::from_secs(1);
  let mut first_iter   = true; 
  let mut last_evt_cnt : u32 = 0;
  let mut evt_cnt      = 0u32;
  let mut delta_events : u64;
  let mut n_events     : u64 = 0;
  let     n_errors     : u64 = 0;
  // per default, latch to the mtb trigger.
  // for testting/calibration that gets switched off
  // below
  //latch_to_mtb = true;

  let mut timer        = Instant::now();
  let force_trigger    = force_trigger_rate > 0;
  let mut time_between_events : Option<f32> = None;
  if force_trigger {
    warn!("Will run in forced trigger mode with a rate of {force_trigger_rate} Hz!");
    time_between_events = Some(1.0/(force_trigger_rate as f32));
    warn!(".. this means one trigger every {} seconds...", time_between_events.unwrap());
    latch_to_mtb = false;
  }

  let now = time::Instant::now();

  let mut terminate = false;
  // the runner will specifically set up the DRS4
  let mut is_running = false;
  let mut pars = RunParams::new();

  // this is the progress bars
  //let mut template_bar_env : &str = "[{elapsed_precise}] {prefix} {msg} {spinner} {bar:60.red/grey} {pos:>7}/{len:7}";
  //let sty_ev = ProgressStyle::with_template(template_bar_env)
  //.unwrap();
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

  let mut which_buff : RamBuffer;
  let mut buff_size  : usize;
  loop {
    if !is_running {
      match run_params.try_recv() {
        Err(err) => {
          trace!("Did not receive new RunParams! Err {err}");
          thread::sleep(one_sec);
          continue;
        }
        Ok(p) => {
          info!("Received a new set of RunParams! {:?}", p);
          pars = p;
          if pars.is_active {
            info!("Will start a new run!");
            info!("Initializing board, starting up...");
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
            if force_trigger {
              match enable_trigger() {
                Err(err) => error!("Can not enable triggers! Err {err}"),
                Ok(_)    => info!("Triggers enabled - Run start!")
              }
            } else {
              match enable_trigger() {
                Err(err) => error!("Can not enable triggers! Err {err}"),
                Ok(_)    => info!("Triggers enabled - Run start!")
              }
              thread::sleep(one_sec);
              match get_trigger_rate() {
                Err(err) => error!("Unable to obtain trigger rate! Err {err}"),
                Ok(rate) => info!("Seing MTB trigger rate of {rate} Hz")
              }
            }
            is_running = true;
            if show_progress {
              if pars.forever {
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
                            .insert_after(&prog_b, ProgressBar::new(pars.nevents as u64)); 
              prog_a.set_message (label_a.clone());
              prog_a.set_prefix  ("\u{1F4BE}");
              prog_a.set_style   (sty_a.clone());
              prog_b.set_message (label_b.clone());
              prog_b.set_prefix  ("\u{1F4BE}");
              prog_b.set_style   (sty_b.clone());
              prog_ev.set_style  (sty_ev.clone());
              prog_ev.set_prefix ("\u{2728}");
              prog_ev.set_message("EVENTS");
            }
            continue; // start loop again
          } else {
            info!("Got signal to end stop run");
            is_running = false;
            match disable_trigger() {
              Err(err) => error!("Can not disable triggers, error {err}"),
              Ok(_)    => ()
            }
            if show_progress {
              prog_ev.finish();
              prog_a.finish();
              prog_b.finish();
            }
          }
        }
      }
    } // this is the !is_running branch

    if force_trigger {
      //println!("Forcing trigger!");
      //println!("Time between events {}", time_between_events.unwrap());
      let elapsed = timer.elapsed().as_secs_f32();
      //println!("Elapsed {}", elapsed);
      if elapsed > time_between_events.unwrap() {
        timer = Instant::now(); 
        match trigger() {
          Err(err) => error!("Error when triggering! {err}"),
          Ok(_)    => ()//println!("Firing trigger!")
        }
      } else {
        // FIXME - we could sleep here for a bit!
        continue;
      }
    }    

    // calculate current event count
    if !force_trigger {
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
            continue;
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
        RamBuffer::A => prog_a.set_position(buff_size as u64),
        RamBuffer::B => prog_b.set_position(buff_size as u64),
      }
      prog_ev.set_position(n_events);
    }

    if !pars.forever {
      if pars.nevents != 0 {
        if n_events > pars.nevents as u64{
          terminate = true;
        }
      }
      
      if pars.nseconds > 0 {
          if now.elapsed().as_secs() > pars.nseconds  as u64{
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
      // exit loop on n event basis
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
        println!("Run stopped! We have seen {n_events}. If this process has been started manually, you can kill it with CTRL+C");
      } else {
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
pub fn event_cache_worker(recv_ev_pl    : Receiver<RBEventPayload>,
                          //cmd_from_cmdr : &Receiver<TofCommand>,
                          //send_ev_pl  : Sender<Option<RBEventPayload>>,
                          tp_to_pub    : &Sender<TofPacket>,
                          //hasit_to_cmd : &Sender<bool>,
                          resp_to_cmd  : &Sender<TofResponse>,
                          get_op_mode  : Receiver<TofOperationMode>, 
                          recv_evid    : Receiver<u32>,
                          cache_size   : usize) {

  let mut n_send_errors  = 0u64;   
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
        //println!("Received payload with event id {}" ,event.event_id);
        if !event_cache.contains_key(&event.event_id) {
          event_cache.insert(event.event_id, event);
        }
        // keep track of the oldest event_id
        trace!("We have a cache size of {}", event_cache.len());
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
    info!("Buff handler switch buffers {switch_buff}");
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
        info!("We are sending {} bytes", bs_len);
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
///  * bs_recv   : A receiver for bytestreams. The 
///                bytestream comes directly from 
///                the data buffers.
///  * ev_sender : Send the the payload to the event cache
pub fn event_payload_worker(bs_recv   : &Receiver<Vec<u8>>,
                            ev_sender : Sender<RBEventPayload>) {
  let mut n_events : u32;
  let mut event_id : u32 = 0;
  //println!("[EVENT PAYLOAD WORKER] Start..");
  'main : loop {
    let mut start_pos : usize = 0;
    n_events = 0;
    //let mut debug_evids = Vec::<u32>::new();
    match bs_recv.recv() {
      Ok(bytestream) => {
        'bytestream : loop {
          //println!("Received bytestream");
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
                Err(err) => error!("Problem sending RBEventPayload over channel! Err {err}"),
              }
              continue 'bytestream;
            },
            Err(err) => {
              debug!("Send {n_events} events. Got last event_id! {event_id}");
              debug!("Got bytestream, but can not find HEAD bytes, err {err:?}");
              break 'bytestream;}
          } // end loop
        } // end ok
      }, // end Ok(bytestream)
      Err(err) => {
        error!("Received Garbage! Err {err}");
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

