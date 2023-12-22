use std::path::Path;
use std::sync::{
    Arc,
    Mutex,
};

//use std::time::Instant;
use crossbeam_channel::Sender;

use tof_dataclasses::commands::{TofCommand,
                                TofResponse, TofCommandResp};
use tof_dataclasses::packets::{TofPacket,
                               PacketType};
use tof_dataclasses::run::RunConfig;

use tof_dataclasses::serialization::Serialization;
use tof_dataclasses::threading::ThreadControl;

#[cfg(feature="tofcontrol")]
use tof_dataclasses::constants::{MASK_CMD_8BIT,
                                 MASK_CMD_16BIT,
                                 MASK_CMD_24BIT,
                                 MASK_CMD_32BIT};

use crate::api::{
    rb_calibration,
    get_runconfig,
    prefix_board_id,
    DATAPORT
};
#[cfg(feature="tofcontrol")]
use crate::control::get_board_id_string;
use liftof_lib::build_tcp_from_ip;

/// Centrailized command management
/// 
/// Maintain 0MQ command connection and faciliate 
/// forwarding of commands and responses
///
/// # Arguments
///
/// * cmd_server_ip             : The IP addresss of the C&C server we are listening to.
/// * run_config_file           : The default runconfig file. When we receive a simple
///                               DataRunStartCommand, we will run this configuration
/// * run_config                : A sender to send the dedicated run config to the 
///                               runner
/// * ev_request_to_cache       : When receiveing RBCommands which contain requests,
///                               forward them to event processing.
pub fn cmd_responder(cmd_server_ip             : String,
                     run_config_file           : &Path,
                     run_config                : &Sender<RunConfig>,
                     ev_request_to_cache       : &Sender<TofPacket>,
                     thread_control            : Arc<Mutex<ThreadControl>>) {
  // create 0MQ sockedts
  //let one_milli       = time::Duration::from_millis(1);
  let port = DATAPORT.to_string();
  let cmd_address = build_tcp_from_ip(cmd_server_ip,port);
  // we will subscribe to two types of messages, BRCT and RB + 2 digits 
  // of board id
  let topic_board = get_board_id_string().expect("Can not get board id!");
  let topic_broadcast = String::from("BRCT");
  let ctx = zmq::Context::new();
  // I guess expect is fine here, see above
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
    match cmd_socket.set_subscribe(&topic_broadcast.as_bytes()) {
      Err(err) => error!("Can not subscribe to {topic_broadcast}, err {err}"),
      Ok(_)    => ()
    }
    match cmd_socket.set_subscribe(&topic_board.as_bytes()) {
      Err(err) => error!("Can not subscribe to {topic_board}, err {err}"),
      Ok(_)    => ()
    }
  }
  
  //let mut heartbeat     = Instant::now();

  // I don't know if we need this, maybe the whole block can go away.
  // Originally I thought the RBs get pinged every x seconds and if we
  // don't see the ping, we reconnect to the socket. But I don't know
  // if that scenario actually occurs.
  // Paolo: instead of leaving the connection always open we might
  //  want to reopen it if its not reachable anymore (so like command-oriented)...
  //warn!("TODO: Heartbeat feature not yet implemented on C&C side");
  //let heartbeat_received = false;
  loop {
    match thread_control.lock() {
      Ok(_) => {
        info!("Received stop signal. Will stop thread!");
        break;
      },
      Err(err) => {
        trace!("Can't acquire lock! {err}");
      },
    }
    // Not sure how to deal with the connection. Poll? Or wait blocking?
    // Or don't block? Set a timeout? I guess technically since we are not doing
    // anything else here, we can block until we get something, this saves resources.
    // (in that case the DONTWAIT can go away)
    // Paolo: I would say that either blocking or setting a timeout is the best opt.
    //  Probably setting a timeout is the best practice since, else, we might die.
    //  If we wouldn't block some other commands might be sent and get stuck in the
    //  process (?).
    match cmd_socket.recv_bytes(0) {
    //match cmd_socket.recv_bytes(zmq::DONTWAIT) {
      Err(err) => trace!("Problem receiving command over 0MQ ! Err {err}"),
      Ok(cmd_bytes)  => {
        debug!("Received bytes {}", cmd_bytes.len());
        // it will always be a tof packet
        match TofPacket::from_bytestream(&cmd_bytes, &mut 4) {
          Err(err) => {
            error!("Can not decode TofPacket! bytes {:?}, error {err}", cmd_bytes);
          },
          Ok(tp) => {
            match tp.packet_type {
              PacketType::TofCommand => {
                // we have to strip off the topic
                match TofCommand::from_bytestream(&tp.payload, &mut 0) {
                  Err(err) => {
                    error!("Problem decoding command {}", err);
                  }
                  Ok(cmd)  => {
                    // we got a valid tof command, forward it and wait for the 
                    // response
                    let tof_resp  = TofResponse::GeneralFail(TofCommandResp::RespErrNotImplemented as u32);
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
                        println!("==> Will initialize new run!");
                        let rc    = get_runconfig(&run_config_file);
                        match run_config.send(rc) {
                          Err(err) => error!("Error initializing run! {err}"),
                          Ok(_)    => ()
                        };
                        let resp_good = TofResponse::Success(TofCommandResp::RespSuccFingersCrossed as u32);
                        match cmd_socket.send(resp_good.to_bytestream(),0) {
                          Err(err) => warn!("Can not send response! Err {err}"),
                          Ok(_)    => trace!("Resp sent!")
                        }
                      },
                      TofCommand::DataRunStop(_)   => {
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
                      // Voltage and timing calibration is connected now
                      TofCommand::VoltageCalibration (value) => {
                        trace!("Got voltage calibration command with {value} value");
                        cfg_if::cfg_if! {
                          if #[cfg(feature = "tofcontrol")]  {
                            // MSB first 16 bits are voltage level
                            let voltage_val: u16 = ((value | (MASK_CMD_16BIT << 16)) >> 16) as u16;
                            // MSB third 8 bits are RB ID
                            let rb_id: u8 = ((value | (MASK_CMD_8BIT << 8)) >> 8) as u8;
                            // MSB fourth 8 bits are extra (not used)
                            let extra: u8 = (value | MASK_CMD_8BIT) as u8;
                            println!("Voltage_val: {}, RB ID: {}, extra: {}",voltage_val,rb_id,extra);
                            continue;
                          } else {
                            warn!("The function is implemented, but one has to compile with --features=tofcontrol");
                            match cmd_socket.send(resp_not_implemented,0) {
                              Err(err) => warn!("Can not send response! Err {err}"),
                              Ok(_)    => trace!("Resp sent!")
                            }
                            continue;
                          }
                        }
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
              },
              PacketType::RBCommand  => {
                trace!("Received RBCommand!");
                // just forward the packet now, the cache 
                // can understand if it is an event request or not
                match ev_request_to_cache.send(tp) {
                  Err(err) => {
                    error!("Can not send event request! Err {err}");
                  },
                  Ok(_) => ()
                }
                // FIXME - notify this about TofOperation mode.
                // if the TofOperation mode is StreamAny, 
                // we won't do this.
                // It might not needed, since if we are in 
                // StreamAny mode, we should not be sending 
                // these requests from the C&C server.
              
                // FIXME - do we want to acknowledge this?
              },
              _ => {
                error!("Can not respond to {}", tp);
              }
            }
          }
        }
      }
    }
  }
}
