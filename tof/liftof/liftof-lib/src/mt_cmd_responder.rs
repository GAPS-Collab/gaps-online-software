use std::path::Path;
use std::sync::{
    Arc,
    Mutex,
};

//use std::time::Instant;
use crossbeam_channel::Sender;

use tof_dataclasses::commands::{TofCommand,
                                TofResponse, TofCommandResp};
use tof_dataclasses::errors::SetError;
use tof_dataclasses::packets::{TofPacket,
                               PacketType};
use tof_dataclasses::run::RunConfig;

use tof_dataclasses::serialization::Serialization;

use liftof_lib::{build_tcp_from_ip, TofComponent, PowerStatusEnum, LTBThresholdName};

use tof_dataclasses::constants::{MASK_CMD_8BIT,
                                  MASK_CMD_16BIT,
                                  MASK_CMD_24BIT,
                                  MASK_CMD_32BIT};

use crate::api::{rb_calibration,
                 rb_noi_subcalibration,
                 rb_voltage_subcalibration,
                 rb_timing_subcalibration,
                 send_preamp_bias_set,
                 send_ltb_threshold_set,
                 power_preamp,
                 power_ltb,
                 get_runconfig,
                 prefix_board_id,
                 DATAPORT};
use crate::threads::monitoring::{get_ltb_moni, get_pb_moni, get_rb_moni};

use liftof_lib::constants::DEFAULT_RB_ID;

use tof_dataclasses::threading::ThreadControl;

use crate::control::{get_board_id_string,
                     get_board_id};

/// Centrailized command management
/// 
/// Maintain 0MQ command connection and faciliate 
/// forwarding of commands and responses
///
/// # Arguments
///
/// * cmd_server_ip             : The IP addresss of the C&C server we are listening to.
/// * ev_request_to_cache       : When receiveing RBCommands which contain requests,
///                               forward them to event processing.
pub fn cmd_responder(cmd_server_ip             : String,
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
  
  loop {
    match thread_control.lock() {
      Ok(tc) => {
        if tc.stop_flag {
          info!("Received stop signal. Will stop thread!");
          break;
        }
      },
      Err(err) => {
        trace!("Can't acquire lock! {err}");
      },
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
                      TofCommand::Unknown (_) => {
                        info!("Received unknown command");
                        let r = TofResponse::GeneralFail(TofCommandResp::RespErrUnexecutable as u32);
                        match cmd_socket.send(r.to_bytestream(),0) {
                          Err(err) => warn!("Can not send response!, Err {err}"),
                          Ok(_)    => info!("Responded to Uknown!")
                        }
                        warn!("Can not interpret Unknown command!");
                        continue;
                      },
                      TofCommand::Ping (value) => {
                        // MSB third 8 bits are 
                        let tof_component: TofComponent = TofComponent::from(((value | MASK_CMD_8BIT << 8) >> 8) as u8);
                        // MSB fourth 8 bits are 
                        let id: u8 = (value | MASK_CMD_8BIT) as u8;

                        let return_val;
                        if tof_component != TofComponent::MT {
                          // The packet was not for the MT so bye
                          continue;
                        } else {
                          match tof_component {
                            TofComponent::RB => {
                              info!("Received moni command");
                              let mut tp = TofPacket::new();
                              tp.packet_type = PacketType::Ping;
                              // TODO what do we want here
                              tp.payload = vec![TofComponent::RB as u8, rb_id];

                              match ev_request_to_cache.send(tp) {
                                Err(err) => {
                                  error!("TofCpu moni sending failed! Err {}", err);
                                  return_val = Err(SetError::CanNotConnectToMyOwnZMQSocket);
                                }
                                Ok(_)    => {
                                  return_val = Ok(());
                                }
                              };
                            },
                            TofComponent::PB  => {
                              return_val = Err(SetError::EmptyInputData);
                              warn!("Not implemented for PB yet")
                            },
                            TofComponent::LTB => {
                              return_val = Err(SetError::EmptyInputData);
                              warn!("Not implemented for LTB yet")
                            },
                            _                 => {
                              return_val = Err(SetError::EmptyInputData);
                              error!("An RB can control just PBs and LTBs.")
                            }
                          }

                          match return_val {
                            Err(_) => {
                              let r = TofResponse::GeneralFail(TofCommandResp::RespErrUnexecutable as u32);
                              match cmd_socket.send(r.to_bytestream(),0) {
                                Err(err) => warn!("Can not send response!, Err {err}"),
                                Ok(_)    => info!("Responded to Power!")
                              }
                            },
                            Ok(_)  => {
                              let r = TofResponse::Success(TofCommandResp::RespSuccFingersCrossed as u32);
                              match cmd_socket.send(r.to_bytestream(),0) {
                                Err(err) => warn!("Can not send response!, Err {err}"),
                                Ok(_)    => info!("Responded to Moni!")
                              }
                            }
                          }
                          continue;
                        }
                      },
                      TofCommand::Moni (value) => {
                        // MSB third 8 bits are 
                        let tof_component: TofComponent = TofComponent::from(((value | MASK_CMD_8BIT << 8) >> 8) as u8);
                        // MSB fourth 8 bits are 
                        let id: u8 = (value | MASK_CMD_8BIT) as u8;

                        // Function that just replies to a ping command send to tofcpu
                        // get_board_id PANICS!! TODO
                        let rb_id = get_board_id().unwrap() as u8;

                        let return_val;
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
                              let tp = TofPacket::from(&moni);

                              match ev_request_to_cache.send(tp) {
                                Err(err) => {
                                  error!("RB moni sending failed! Err {}", err);
                                  return_val = Err(SetError::CanNotConnectToMyOwnZMQSocket);
                                }
                                Ok(_)    => {
                                  return_val = Ok(());
                                }
                              };
                            },
                            TofComponent::PB  => {
                              info!("Received PB moni command");
                              let moni = get_pb_moni(id).unwrap();
                              let tp = TofPacket::from(&moni);

                              match ev_request_to_cache.send(tp) {
                                Err(err) => {
                                  error!("PB moni sending failed! Err {}", err);
                                  return_val = Err(SetError::CanNotConnectToMyOwnZMQSocket);
                                }
                                Ok(_)    => {
                                  return_val = Ok(());
                                }
                              };
                            },
                            TofComponent::LTB => {
                              info!("Received LTB moni command");
                              let moni = get_ltb_moni(id).unwrap();
                              let tp = TofPacket::from(&moni);

                              match ev_request_to_cache.send(tp) {
                                Err(err) => {
                                  error!("LTB moni sending failed! Err {}", err);
                                  return_val = Err(SetError::CanNotConnectToMyOwnZMQSocket);
                                }
                                Ok(_)    => {
                                  return_val = Ok(());
                                }
                              };
                            },
                            _                 => {
                              return_val = Err(SetError::EmptyInputData);
                              error!("An RB can control just PBs and LTBs.")
                            }
                          }

                          match return_val {
                            Err(_) => {
                              let r = TofResponse::GeneralFail(TofCommandResp::RespErrUnexecutable as u32);
                              match cmd_socket.send(r.to_bytestream(),0) {
                                Err(err) => warn!("Can not send response!, Err {err}"),
                                Ok(_)    => info!("Responded to Power!")
                              }
                            },
                            Ok(_)    => {
                              let r = TofResponse::Success(TofCommandResp::RespSuccFingersCrossed as u32);
                              match cmd_socket.send(r.to_bytestream(),0) {
                                Err(err) => warn!("Can not send response!, Err {err}"),
                                Ok(_)    => info!("Responded to Moni!")
                              }
                            }
                          }
                          continue;
                        }
                      },
                      TofCommand::Power   (value) => {
                        info!("Received set threshold command! Will communicate to LTBs");
                        // MSB second 8 bits are tof component
                        let tof_component: TofComponent = TofComponent::from(((value | (MASK_CMD_8BIT << 16)) >> 16) as u8);
                        // MSB third 8 bits are 
                        let component_id: u8 = ((value | MASK_CMD_8BIT << 8) >> 8) as u8;
                        // MSB fourth 8 bits are 
                        let status: PowerStatusEnum = PowerStatusEnum::from((value | MASK_CMD_8BIT) as u8);
                        // TODO implement proper routines
                        let return_val;
                        match tof_component {
                          TofComponent::All      => {
                            return_val = Err(SetError::EmptyInputData);
                            warn!("Not implemented for All yet")
                          }, //power_all(cmd_socket, component_id, status),
                          TofComponent::MT       => {
                            return_val = Err(SetError::EmptyInputData);
                            warn!("Not implemented for MT yet")
                          }, //power_mt(cmd_socket, component_id, status),
                          TofComponent::AllButMT => {
                            return_val = Err(SetError::EmptyInputData);
                            warn!("Not implemented for AllButMT yet")
                          }, //power_allbutmt(cmd_socket, component_id, status),
                          TofComponent::LTB      => {
                            return_val = power_ltb(component_id, status);
                            match return_val {
                              Ok(_)  => trace!("LTB powered up!"),
                              Err(_) => warn!("Not able to power up LTB!")
                            };
                          },
                          TofComponent::Preamp   => {
                            return_val = power_preamp(component_id, status);
                            match return_val {
                              Ok(_)  => trace!("Preamp powered up!"),
                              Err(_) => warn!("Not able to power up Preamp!")
                            };
                          },
                          _                      => {
                            return_val = Err(SetError::EmptyInputData);
                            error!("Power operation not implemented for Unknown!")
                          }
                        }
                        match return_val {
                          Err(_) => {
                            let r = TofResponse::GeneralFail(TofCommandResp::RespErrUnexecutable as u32);
                            match cmd_socket.send(r.to_bytestream(),0) {
                              Err(err) => warn!("Can not send response!, Err {err}"),
                              Ok(_)    => info!("Responded to Power!")
                            }
                          },
                          Ok(_)    => {
                            let r = TofResponse::Success(TofCommandResp::RespSuccFingersCrossed as u32);
                            match cmd_socket.send(r.to_bytestream(),0) {
                              Err(err) => warn!("Can not send response!, Err {err}"),
                              Ok(_)    => info!("Responded to Power!")
                            }
                          }
                        }
                        continue;
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
                          Err(err) => {
                            let r = TofResponse::GeneralFail(TofCommandResp::RespErrUnexecutable as u32);
                            match cmd_socket.send(r.to_bytestream(),0) {
                              Err(err) => warn!("Can not send response!, Err {err}"),
                              Ok(_)    => info!("Responded to SetThreshold!")
                            }
                            warn!("Can not set ltb threshold! Err {err}")
                          },
                          Ok(_)    => {
                            let r = TofResponse::Success(TofCommandResp::RespSuccFingersCrossed as u32);
                            match cmd_socket.send(r.to_bytestream(),0) {
                              Err(err) => warn!("Can not send response!, Err {err}"),
                              Ok(_)    => info!("Responded to SetThreshold!")
                            }
                            trace!("Resp sent!")
                          }
                        }
                        continue;
                      },
                      TofCommand::SetMTConfig  (_) => {
                        warn!("Not implemented");
                        match cmd_socket.send(resp_not_implemented,0) {
                          Err(err) => warn!("Can not send response! Err {err}"),
                          Ok(_)    => trace!("Resp sent!")
                        }
                        continue;
                      },
                      TofCommand::SetPreampBias   (value) =>  {
                        info!("Received set preamp bias command! Will communicate to preamps");
                        // MSB second 8 bits are LTB ID
                        let preamp_id: u8 = ((value | (MASK_CMD_8BIT << 16)) >> 16) as u8;
                        // MSB third 16 bits are extra (not used)
                        let preamp_bias: u16 = (value | MASK_CMD_16BIT) as u16;
                        match send_preamp_bias_set(preamp_id, preamp_bias) {
                          Err(err) => {
                            let r = TofResponse::GeneralFail(TofCommandResp::RespErrUnexecutable as u32);
                            match cmd_socket.send(r.to_bytestream(),0) {
                              Err(err) => warn!("Can not send response!, Err {err}"),
                              Ok(_)    => info!("Responded to SetPreampBias!")
                            }
                            warn!("Can not set ltb threshold! Err {err}")
                          },
                          Ok(_)    => {
                            let r = TofResponse::Success(TofCommandResp::RespSuccFingersCrossed as u32);
                            match cmd_socket.send(r.to_bytestream(),0) {
                              Err(err) => warn!("Can not send response!, Err {err}"),
                              Ok(_)    => info!("Responded to SetPreampBias!")
                            }
                            trace!("Resp sent!")
                          }
                        }
                        continue;
                      },
                      TofCommand::DataRunStop(value)   => {
                        // MSB fourth 8 bits are RB ID
                        let rb_id: u8 = (value | MASK_CMD_8BIT) as u8;

                        let my_rb_id = get_board_id().unwrap() as u8;
                        // if this RB is the one then do stuff
                        if rb_id == DEFAULT_RB_ID || rb_id == my_rb_id {
                          println!("Received command to end run!");
                          // default is not active for run config

                          let rc = RunConfig::new();
                          match run_config.send(rc) {
                            Err(err) => error!("Error stopping run! {err}"),
                            Ok(_)    => ()
                          }
                          let resp_good = TofResponse::Success(TofCommandResp::RespSuccFingersCrossed as u32);
                          match cmd_socket.send(resp_good.to_bytestream(),0) {
                            Err(err) => warn!("Can not send response! Err {err}"),
                            Ok(_)    => trace!("Resp sent!")
                          }
                        }
                        continue;
                      },
                      TofCommand::DataRunStart (value) => {
                        // MSB second 8 bits are run_type
                        let run_type: u8 = ((value | (MASK_CMD_8BIT << 16)) >> 16) as u8;
                        // MSB third 8 bits are RB ID
                        let rb_id: u8    = ((value | (MASK_CMD_8BIT << 8)) >> 8) as u8;
                        // MSB fourth 8 bits are event number
                        let event_no: u8 = (value | MASK_CMD_8BIT) as u8;
                        // let's start a run. The value of the TofCommnad shall be 
                        // nevents

                        let my_rb_id = get_board_id().unwrap() as u8;
                        // if this RB is the one then do stuff
                        if rb_id == DEFAULT_RB_ID || rb_id == my_rb_id {
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
                      TofCommand::GetFullWaveforms  (_) => {
                        warn!("Not implemented");
                        match cmd_socket.send(resp_not_implemented,0) {
                          Err(err) => warn!("Can not send response! Err {err}"),
                          Ok(_)    => trace!("Resp sent!")
                        }
                        continue;
                      },
                      // Voltage and timing calibration is connected now
                      TofCommand::NoiCalibration (value) => {
                        // MSB third 8 bits are RB ID
                        let rb_id: u8 = ((value | (MASK_CMD_8BIT << 8)) >> 8) as u8;
                        // MSB fourth 8 bits are extra (not used)
                        let extra: u8 = (value | MASK_CMD_8BIT) as u8;
                        
                        let my_rb_id = get_board_id().unwrap() as u8;
                        // if this RB is the one then do stuff
                        if rb_id == DEFAULT_RB_ID || rb_id == my_rb_id {
                          match rb_noi_subcalibration(&run_config, &ev_request_to_cache) {
                            Ok(_) => (),
                            Err(err) => {
                              error!("Noi data taking failed! Error {err}!");
                            }
                          }

                          let resp_good = TofResponse::Success(TofCommandResp::RespSuccFingersCrossed as u32);
                          match cmd_socket.send(resp_good.to_bytestream(),0) {
                            Err(err) => warn!("Can not send response! Err {err}"),
                            Ok(_)    => trace!("Resp sent!")
                          }
                        }
                        continue;
                      },
                      TofCommand::VoltageCalibration (value) => {
                        trace!("Got voltage calibration command with {value} value");
                        // MSB first 16 bits are voltage level
                        let voltage_val: u16 = ((value | (MASK_CMD_16BIT << 16)) >> 16) as u16;
                        // MSB third 8 bits are RB ID
                        let rb_id: u8 = ((value | (MASK_CMD_8BIT << 8)) >> 8) as u8;
                        // MSB fourth 8 bits are extra (not used)
                        let extra: u8 = (value | MASK_CMD_8BIT) as u8;
                        
                        let my_rb_id = get_board_id().unwrap() as u8;
                        // if this RB is the one then do stuff
                        if rb_id == DEFAULT_RB_ID || rb_id == my_rb_id {
                          match rb_voltage_subcalibration(&run_config, &ev_request_to_cache, voltage_val) {
                            Ok(_) => (),
                            Err(err) => {
                              error!("Noi data taking failed! Error {err}!");
                            }
                          }

                          let resp_good = TofResponse::Success(TofCommandResp::RespSuccFingersCrossed as u32);
                          match cmd_socket.send(resp_good.to_bytestream(),0) {
                            Err(err) => warn!("Can not send response! Err {err}"),
                            Ok(_)    => trace!("Resp sent!")
                          }
                        }
                        continue;
                      },
                      TofCommand::TimingCalibration  (value) => {
                        // MSB first 16 bits are voltage level
                        let voltage_val: u16 = ((value | (MASK_CMD_16BIT << 16)) >> 16) as u16;
                        // MSB third 8 bits are RB ID
                        let rb_id: u8 = ((value | (MASK_CMD_8BIT << 8)) >> 8) as u8;
                        // MSB fourth 8 bits are extra (not used)
                        let extra: u8 = (value | MASK_CMD_8BIT) as u8;
                        
                        let my_rb_id = get_board_id().unwrap() as u8;
                        // if this RB is the one then do stuff
                        if rb_id == DEFAULT_RB_ID || rb_id == my_rb_id {
                          match rb_timing_subcalibration(&run_config, &ev_request_to_cache, voltage_val) {
                            Ok(_) => (),
                            Err(err) => {
                              error!("Noi data taking failed! Error {err}!");
                            }
                          }

                          let resp_good = TofResponse::Success(TofCommandResp::RespSuccFingersCrossed as u32);
                          match cmd_socket.send(resp_good.to_bytestream(),0) {
                            Err(err) => warn!("Can not send response! Err {err}"),
                            Ok(_)    => trace!("Resp sent!")
                          }
                        }
                        continue;
                      },
                      TofCommand::DefaultCalibration  (value) => {
                        // MSB first 16 bits are voltage level
                        let voltage_val: u16 = ((value | (MASK_CMD_16BIT << 16)) >> 16) as u16;
                        // MSB third 8 bits are RB ID
                        let rb_id: u8 = ((value | (MASK_CMD_8BIT << 8)) >> 8) as u8;
                        // MSB fourth 8 bits are extra (not used)
                        let extra: u8 = (value | MASK_CMD_8BIT) as u8;

                        let my_rb_id = get_board_id().unwrap() as u8;
                        // if this RB is the one then do stuff
                        if rb_id == DEFAULT_RB_ID || rb_id == my_rb_id {
                          match rb_calibration(&run_config, &ev_request_to_cache) {
                            Ok(_) => (),
                            Err(err) => {
                              error!("Calibration failed! Error {err}!");
                            }
                          }

                          let resp_good = TofResponse::Success(TofCommandResp::RespSuccFingersCrossed as u32);
                          match cmd_socket.send(resp_good.to_bytestream(),0) {
                            Err(err) => warn!("Can not send response! Err {err}"),
                            Ok(_)    => trace!("Resp sent!")
                          }
                        }
                        continue;
                      },
                      TofCommand::SetRBDataBufSize   (_) => {
                        warn!("Not implemented");
                        match cmd_socket.send(resp_not_implemented,0) {
                          Err(err) => warn!("Can not send response! Err {err}"),
                          Ok(_)    => trace!("Resp sent!")
                        }
                        continue;
                      },
                      TofCommand::TriggerModeForced  (_) => {
                        warn!("Not implemented");
                        match cmd_socket.send(resp_not_implemented,0) {
                          Err(err) => warn!("Can not send response! Err {err}"),
                          Ok(_)    => trace!("Resp sent!")
                        }
                        continue;
                      },
                      TofCommand::TriggerModeForcedMTB   (_) => {
                        warn!("Not implemented");
                        match cmd_socket.send(resp_not_implemented,0) {
                          Err(err) => warn!("Can not send response! Err {err}"),
                          Ok(_)    => trace!("Resp sent!")
                        }
                        continue;
                      },
                      TofCommand::UnspoolEventCache  (_) => {
                        warn!("Not implemented");
                        match cmd_socket.send(resp_not_implemented,0) {
                          Err(err) => warn!("Can not send response! Err {err}"),
                          Ok(_)    => trace!("Resp sent!")
                        }
                        continue;
                      },
                      TofCommand::SystemdReboot  (_) => {
                        warn!("Not implemented");
                        match cmd_socket.send(resp_not_implemented,0) {
                          Err(err) => warn!("Can not send response! Err {err}"),
                          Ok(_)    => trace!("Resp sent!")
                        }
                        continue;
                      }
                    }
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
