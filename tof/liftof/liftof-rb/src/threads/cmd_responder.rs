use std::time::{
    //Instant,
    Duration,
};
use std::thread;
use std::sync::{
    Arc,
    Mutex,
};

//use std::time::Instant;
//use std::path::Path;
use crossbeam_channel::Sender;

use tof_control::helper::pa_type::PASetBias;

use tof_dataclasses::commands::{
    TofCommand,
    TofCommandV2,
    TofCommandCode,
    TofResponse,
    TofResponseCode,
};
use tof_dataclasses::config::PreampBiasConfig;

use tof_dataclasses::errors::CmdError;
use tof_dataclasses::packets::{TofPacket,
                               PacketType};
use tof_dataclasses::config::RunConfig;
use tof_dataclasses::heartbeats::RBPing;
use tof_dataclasses::serialization::Serialization;
use tof_dataclasses::serialization::Packable;

use liftof_lib::{
    //build_tcp_from_ip,
    TofComponent,
    LTBThresholdName
};

use tof_dataclasses::constants::{MASK_CMD_8BIT,
                                  MASK_CMD_16BIT};

use crate::api::{rb_calibration,
                 //set_preamp_biases,
                 //send_preamp_bias_set,
                 send_ltb_threshold_set,
};
use crate::threads::monitoring::{
    get_ltb_moni,
    get_pb_moni,
    get_rb_moni
};

use liftof_lib::constants::DEFAULT_RB_ID;
use liftof_lib::thread_control::ThreadControl;

//use tof_dataclasses::threading::ThreadControl;

use crate::control::{get_board_id_string,
                     get_board_id};

/// Centrailized command management
/// 
/// Maintain 0MQ command connection and faciliate 
/// forwarding of commands and responses
///
/// # Arguments
///
/// * cmd_server_address        : The full address string e.g. tcp://1.1.1.1:12345 
///                               where the command server is publishing commands..
/// * run_config                : The default runconfig. Defined by reading in the 
///                               config file when the code boots up.
///                               When we receive a simple DataRunStartCommand,
///                               we will run this configuration
/// * run_config_sender         : A sender to send the dedicated run config to the 
///                               runner
/// * tp_to_pub                 : Send TofPackets to the data pub.
///                               Some TOF commands might trigger
///                               additional information to get 
///                               send.
/// * address_for_cali          : The local (self) PUB address, so that the rb_calibratoin,
///                               can subscribe to it to loop itself the event packets
/// * thread_control            : Manage thread control signals, e.g. stop
pub fn cmd_responder(cmd_server_address        : String,
                     run_config                : &RunConfig,
                     run_config_sender         : &Sender<RunConfig>,
                     tp_to_pub                 : &Sender<TofPacket>,
                     address_for_cali          : String,
                     thread_control            : Arc<Mutex<ThreadControl>>) {
  // create 0MQ sockedts
  //let one_milli       = time::Duration::from_millis(1);
  //let port            = DATAPORT.to_string();
  //let cmd_address     = build_tcp_from_ip(cmd_server_ip,port);
  //// we will subscribe to two types of messages, BRCT and RB + 2 digits 
  //// of board id
  let topic_board     = get_board_id_string().expect("Can not get board id!");
  let topic_broadcast = String::from("BRCT");
  let ctx = zmq::Context::new();
  // I guess expect is fine here, see above
  let cmd_socket = ctx.socket(zmq::SUB).expect("Unable to create 0MQ SUB socket!");
  info!("Will set up 0MQ SUB socket to listen for commands at address {cmd_server_address}");
  let mut is_connected = false;
  match cmd_socket.connect(&cmd_server_address) {
    Err(err) => warn!("Not able to connect to {}, Error {err}", cmd_server_address),
    Ok(_)    => {
      info!("Connected to CnC server at {}", cmd_server_address);
      match cmd_socket.set_subscribe(&topic_broadcast.as_bytes()) {
        Err(err) => error!("Can not subscribe to {topic_broadcast}, err {err}"),
        Ok(_)    => ()
      }
      match cmd_socket.set_subscribe(&topic_board.as_bytes()) {
        Err(err) => error!("Can not subscribe to {topic_board}, err {err}"),
        Ok(_)    => ()
      }
      is_connected = true;
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
  let mut save_cali_wf = false;
  loop {
    match thread_control.lock() {
      Ok(tc) => {
        save_cali_wf = tc.liftof_settings.save_cali_wf;
        if tc.stop_flag {
          info!("Received stop signal. Will stop thread!");
          break;
        }
      },
      Err(err) => {
        trace!("Can't acquire lock! {err}");
      },
    }
    // we need to make sure to connect to our server
    // The startup is a bit tricky... FIXME
    if !is_connected {
      match cmd_socket.connect(&cmd_server_address) {
        Err(err) => {
          debug!("Not able to connect to {}! {err}", cmd_server_address);
          thread::sleep(Duration::from_millis(200));
          continue;
        }
        Ok(_)    => {
          info!("Connected to CnC server at {}", cmd_server_address);
          match cmd_socket.set_subscribe(&topic_broadcast.as_bytes()) {
            Err(err) => error!("Can not subscribe to {topic_broadcast}, err {err}"),
            Ok(_)    => ()
          }
          match cmd_socket.set_subscribe(&topic_board.as_bytes()) {
            Err(err) => error!("Can not subscribe to {topic_board}, err {err}"),
            Ok(_)    => ()
          }
          is_connected = true;
        }
      }
    }
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
              PacketType::TofCommandV2 => {
                let cmd : TofCommandV2;
                match tp.unpack::<TofCommandV2>() {
                  Ok(_cmd) => {cmd = _cmd;}
                  Err(err) => {
                    error!("Unable to unpack TofCommand! {err}");
                    continue;
                  }
                }
                match cmd.command_code {
                  TofCommandCode::SetPreampBias => {
                    match PreampBiasConfig::from_bytestream(&cmd.payload, &mut 0) {
                      Ok(pb_cfg) => { 
                        match PASetBias::set_manual_biases(pb_cfg.biases) {
                          Ok(_)    => info!("Set preamp biases to {}!", pb_cfg),
                          Err(err) => {
                            error!("Unable to set preamp biases! {err:?}");
                            continue;
                          }
                        }
                      },
                      Err(err) => {
                        error!("Unable to set preamp biases! {err}");
                        // FIXME - send response packet
                        continue;
                      }
                    }
                  }
                  _ => error!("Not able to execute command for command code {}", cmd.command_code)
                }
              }
              PacketType::TofCommand => {
                // we have to strip off the topic
                match TofCommand::from_bytestream(&tp.payload, &mut 0) {
                  Err(err) => {
                    error!("Problem decoding command {err}");
                  }
                  Ok(cmd)  => {
                    // we got a valid tof command, forward it and wait for the 
                    // response
                    //let tof_resp  = TofResponse::GeneralFail(TofResponseCode::RespErrNotImplemented as u32);
                    //let resp_not_implemented = prefix_board_id(&mut tof_resp.to_bytestream());
                    //let resp_not_implemented = TofResponse::GeneralFail(RESP_ERR_NOTIMPLEMENTED);
                    let return_val: Result<TofCommandCode, CmdError>;
                    match cmd {
                      TofCommand::Unknown (_) => {
                        info!("Received unknown command");
                        error!("Cannot interpret unknown command");
                        return_val = Err(CmdError::UnknownError);
                      },
                      TofCommand::Ping (_) => {
                        info!("Received ping command");
                        let mut ping = RBPing::new();
                        ping.rb_id = get_board_id().unwrap_or(0) as u8;
                        let tp = ping.pack();
                        match tp_to_pub.send(tp) {
                          Err(err) => error!("Unable to send ping response! {err}"),
                          Ok(_) => ()
                        }
                        return_val   = Ok(TofCommandCode::Ping);
                      },
                      TofCommand::Moni (value) => {
                        // MSB third 8 bits are 
                        let tof_component: TofComponent = TofComponent::from(((value | MASK_CMD_8BIT << 8) >> 8) as u8);
                        // MSB fourth 8 bits are 
                        let id: u8 = (value | MASK_CMD_8BIT) as u8;
                        // Function that just replies to a ping command send to tofcpu
                        // get_board_id PANICS!! TODO
                        let rb_id = get_board_id().unwrap() as u8;

                        if tof_component != TofComponent::MT &&
                           tof_component != TofComponent::TofCpu &&
                           tof_component != TofComponent::Unknown &&
                           rb_id != id {
                          // The packet was not for this RB so bye
                          continue;
                        } else {
                          match tof_component {
                            TofComponent::RB => {
                              info!("Received RB moni command");
                              let moni = get_rb_moni(id).unwrap();
                              let tp = moni.pack();

                              match tp_to_pub.send(tp) {
                                Err(err) => {
                                  error!("RB moni sending failed! Err {err}");
                                  return_val = Err(CmdError::MoniError);
                                }
                                Ok(_)    => {
                                  info!("RB moni sent");
                                  return_val = Ok(TofCommandCode::Moni);
                                }
                              };
                            },
                            TofComponent::PB  => {
                              info!("Received PB moni command");
                              let moni = get_pb_moni(id).unwrap();
                              let tp = moni.pack();
                              match tp_to_pub.send(tp) {
                                Err(err) => {
                                  error!("PB moni sending failed! Err {err}");
                                  return_val = Err(CmdError::MoniError);
                                }
                                Ok(_)    => {
                                  info!("PB moni sent");
                                  return_val = Ok(TofCommandCode::Moni);
                                }
                              };
                            },
                            TofComponent::LTB => {
                              info!("Received LTB moni command");
                              let moni = get_ltb_moni(id).unwrap();
                              let tp = moni.pack();
                              match tp_to_pub.send(tp) {
                                Err(err) => {
                                  error!("LTB moni sending failed! Err {err}");
                                  return_val = Err(CmdError::MoniError);
                                }
                                Ok(_)    => {
                                  info!("LTB moni sent");
                                  return_val = Ok(TofCommandCode::Moni);
                                }
                              };
                            },
                            _                 => {
                              return_val = Err(CmdError::MoniError);
                              error!("An RB can control just PBs and LTBs.")
                            }
                          }
                        }
                      },
                      TofCommand::SetThresholds   (value) =>  {
                        info!("Received set threshold command! Will communicate to LTBs");
                        // MSB first 8 bits are LTB ID
                        let ltb_id: u8 = ((value | (MASK_CMD_8BIT << 24)) >> 24) as u8;
                        // MSB second 8 bits are LTB ID
                        let threshold_name: LTBThresholdName = LTBThresholdName::from(((value | (MASK_CMD_8BIT << 16)) >> 16) as u8);
                        // MSB third 16 bits are extra (not used)
                        let threshold_level: u16 = (value | MASK_CMD_16BIT) as u16;
                        match send_ltb_threshold_set(ltb_id, threshold_name, threshold_level) {
                          Ok(_)    => {
                            info!("Threshold sent to LTB!");
                            return_val = Ok(TofCommandCode::SetLTBThresholds);
                          },
                          Err(err) => {
                            error!("LTB threshold sending failed! Err {err}");
                            return_val = Err(CmdError::ThresholdSetError);
                          }
                        }
                      },
                      //TofCommand::SetPreampBias   (value) =>  {
                      //  info!("Received set preamp bias command! Will communicate to preamps");
                      //  // MSB second 8 bits are LTB ID
                      //  let preamp_id: u8 = ((value | (MASK_CMD_8BIT << 16)) >> 16) as u8;
                      //  // MSB third 16 bits are extra (not used)
                      //  let preamp_bias: u16 = (value | MASK_CMD_16BIT) as u16;
                      //  match send_preamp_bias_set(preamp_id, preamp_bias) {
                      //    Ok(_)    => {
                      //      info!("Bias sent to preamp!");
                      //      return_val = Ok(TofCommandCode::SetPreampBias);
                      //    },
                      //    Err(err) => {
                      //      error!("Preamp bias sending failed! Err {err}");
                      //      return_val = Err(CmdError::PreampBiasSetError);
                      //    }
                      //  }
                      //},
                      TofCommand::DataRunStop(value)   => {
                        // MSB fourth 8 bits are RB ID
                        let rb_id: u8 = (value | MASK_CMD_8BIT) as u8;
                        println!("=> Received command to end run for board ids {rb_id}!");

                        let my_rb_id = get_board_id().unwrap() as u8;
                        // if this RB is the one then do stuff
                        if rb_id == DEFAULT_RB_ID || rb_id == my_rb_id {
                          println!("=> Received command to end run!");
                          // default is not active for run config

                          let rc = RunConfig::new();
                          match run_config_sender.send(rc) {
                            Ok(_)    => {
                              info!("Run stopped successfully!");
                              return_val = Ok(TofCommandCode::DataRunStop);
                            },
                            Err(err) => {
                              error!("Error stopping run! {err}");
                              return_val = Err(CmdError::RunStopError);
                            }
                          }
                        } else {
                          // The packet was not for this RB so bye
                          continue;
                        }
                      },
                      TofCommand::DataRunStart (value) => {
                        // MSB second 8 bits are run_type
                        //let run_type: u8 = ((value | (MASK_CMD_8BIT << 16)) >> 16) as u8;
                        // MSB third 8 bits are RB ID
                        let rb_id: u8    = ((value | (MASK_CMD_8BIT << 8)) >> 8) as u8;
                        // MSB fourth 8 bits are event number
                        //let event_no: u8 = (value | MASK_CMD_8BIT) as u8;
                        // let's start a run. The value of the TofCommnad shall be 
                        // nevents

                        let my_rb_id = get_board_id().unwrap() as u8;
                        // if this RB is the one then do stuff
                        if rb_id == DEFAULT_RB_ID || rb_id == my_rb_id {
                          println!("==> Will initialize new run!");
                          //let rc    = get_runconfig(&run_config_file);
                          let rc = run_config.clone();
                          match run_config_sender.send(rc) {
                            Ok(_)    => {
                              info!("Run started successfully!");
                              return_val = Ok(TofCommandCode::DataRunStart);
                            },
                            Err(err) => {
                              error!("Error starting run! {err}");
                              return_val = Err(CmdError::RunStartError);
                            }
                          };
                        } else {
                          // The packet was not for this RB so bye
                          continue;
                        }
                      },
                      TofCommand::DefaultCalibration  (value) => {
                        // MSB first 16 bits are voltage level
                        //let voltage_val: u16 = ((value | (MASK_CMD_16BIT << 16)) >> 16) as u16;
                        // MSB third 8 bits are RB ID
                        let rb_id: u8 = ((value | (MASK_CMD_8BIT << 8)) >> 8) as u8;
                        // MSB fourth 8 bits are extra (not used)
                        //let extra: u8 = (value | MASK_CMD_8BIT) as u8;

                        let my_rb_id = get_board_id().unwrap() as u8;
                        // if this RB is the one then do stuff
                        if rb_id == DEFAULT_RB_ID || rb_id == my_rb_id {
                          // FIXME - time delay? When we start all calibrations at the 
                          // same time, then the nw might get too busy? 
                          match rb_calibration(&run_config_sender,
                                               &tp_to_pub,
                                               save_cali_wf,
                                               address_for_cali.clone()) {
                            Ok(_) => {
                              println!("== ==> [cmd-responder] Calibration successful!");
                              info!("Default calibration data taking successful!");
                              return_val = Ok(TofCommandCode::RBCalibration);
                            },
                            Err(err) => {
                              error!("Default calibration data taking failed! Error {err}!");
                              return_val = Err(CmdError::CalibrationError);
                            }
                          }
                        } else {
                          // The packet was not for this RB so bye
                          continue;
                        }
                      },
                      TofCommand::SetRBDataBufSize   (_) => {
                        warn!("Not implemented");
                        return_val = Err(CmdError::NotImplementedError);
                      },
                      TofCommand::TriggerModeForced  (_) => {
                        warn!("Not implemented");
                        return_val = Err(CmdError::NotImplementedError);
                      },
                      TofCommand::UnspoolEventCache  (_) => {
                        warn!("Not implemented");
                        return_val = Err(CmdError::NotImplementedError);
                      },
                      TofCommand::SystemdReboot  (_) => {
                        warn!("Not implemented");
                        return_val = Err(CmdError::NotImplementedError);
                      }
                      _ => {
                        error!("{} is not implemented!", cmd);
                        return_val = Err(CmdError::NotImplementedError);
                      }
                    }
                    // deal with return values
                    match return_val {
                      Err(cmd_error) => {
                        let r = TofResponse::GeneralFail(TofResponseCode::RespErrUnexecutable as u32);
                        match cmd_socket.send(r.to_bytestream(),0) {
                          Err(err) => warn!("Can not send response!, Err {err}"),
                          Ok(_)    => info!("Responded to {cmd_error}!")
                        }
                      },
                      Ok(tof_command)  => {
                        let r = TofResponse::Success(TofResponseCode::RespSuccFingersCrossed as u32);
                        match cmd_socket.send(r.to_bytestream(),0) {
                          Err(err) => warn!("Can not send response!! {err}"),
                          Ok(_)    => info!("Responded to {tof_command}!")
                        }
                      }
                    }
                  }
                }  
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
