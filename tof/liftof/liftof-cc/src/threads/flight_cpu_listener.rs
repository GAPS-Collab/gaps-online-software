use std::sync::{
    Arc,
    Mutex,
};
use std::thread::sleep;
use std::time::{Duration, Instant};

//use std::time::Instant;
use crossbeam_channel::{Receiver,
                        Sender};

use tof_dataclasses::commands::{TofCommand,
                                TofResponse, TofCommandResp};
use tof_dataclasses::errors::SetError;
use tof_dataclasses::packets::{TofPacket,
                               PacketType};
use tof_dataclasses::run::RunConfig;

use tof_dataclasses::serialization::Serialization;

use liftof_lib::{TofComponent, PowerStatusEnum, LTBThresholdName};

use tof_dataclasses::constants::{MASK_CMD_8BIT,
                                  MASK_CMD_16BIT};

use tof_dataclasses::threading::ThreadControl;

/// Command management for tof cpu
///
/// # Arguments
///
/// * flight_address       : The address the flight computer
///                          (or whomever) wants to listen.
///                          A 0MQ PUB socket will be bound 
///                          to this address.
/// * incoming             : Bytestream to be unpacked sent by flight cpu
/// * write_npack_file     : Write this many TofPackets to a 
///                          single file before starting a 
///                          new one.
pub fn flight_cpu_listener(flight_address  : &str,
                           incoming        : &Receiver<TofPacket>,
                           outgoing        : &Sender<TofPacket>,
                           cmd_interval    : u64,
                           thread_control  : Arc<Mutex<ThreadControl>>) {
  // create 0MQ sockets
  let ctx = zmq::Context::new();
  // I guess expect is fine here, see above
  let cmd_socket = ctx.socket(zmq::SUB).expect("Unable to create 0MQ SUB socket!");
  info!("Will set up 0MQ SUB socket to listen for flight cpu commands at address {flight_address}");
  match cmd_socket.connect(flight_address) {
    Err(err) => panic!("Can not bind to address {}! {}", flight_address, err),
    Ok(_)    => ()
  }
  info!("ZMQ SUB Socket for flight cpu listener bound to {flight_address}");

  let mut timer     = Instant::now();
  let sleep_time    = Duration::from_secs(cmd_interval);
  loop {
    if timer.elapsed().as_secs() >= cmd_interval {
      timer     = Instant::now();
      match incoming.recv() {
        Err(err) => trace!("No new packet, err {err}"),
        Ok(pack) => {
          debug!("Got new tof packet {}", pack.packet_type);
          match pack.packet_type {
            PacketType::TofCommand => {
              // we have to strip off the topic
              match TofCommand::from_bytestream(&pack.payload, &mut 0) {
                Err(err) => {
                  error!("Problem decoding command {}", err);
                }
                Ok(cmd)  => {
                  // we got a valid tof command, forward it and wait for the 
                  // response
                  let tof_resp  = TofResponse::GeneralFail(TofCommandResp::RespErrNotImplemented as u32);
                  let resp_not_implemented = crate::prefix_tof_cpu(&mut tof_resp.to_bytestream());
                  //let resp_not_implemented = TofResponse::GeneralFail(RESP_ERR_NOTIMPLEMENTED);
                  let outgoing_c = outgoing.clone();
                  match cmd {
                    TofCommand::Unknown (_) => {
                      info!("Received unknown command");
                      let r = TofResponse::GeneralFail(TofCommandResp::RespErrUnexecutable as u32);
                      match cmd_socket.send(r.to_bytestream(),0) {
                        Err(err) => warn!("Can not send response!, Err {err}"),
                        Ok(_)    => info!("Responded to Unknown!")
                      }
                      warn!("Can not interpret Unknown command!");
                      continue;
                    },
                    TofCommand::Ping (value) => {
                      info!("Received ping command");
                      // MSB third 8 bits are 
                      let tof_component: TofComponent = TofComponent::from(((value | MASK_CMD_8BIT << 8) >> 8) as u8);
                      // MSB fourth 8 bits are 
                      let id: u8 = (value | MASK_CMD_8BIT) as u8;

                      let return_val;
                      if tof_component == TofComponent::Unknown {
                        info!("The command is not valid for {}", TofComponent::Unknown);
                        // The packet was not for this RB so bye
                        continue;
                      } else {
                        match tof_component {
                          TofComponent::TofCpu => crate::send_ping_response(outgoing_c, cmd_socket),
                          TofComponent::RB  |
                          TofComponent::LTB |
                          TofComponent::MT     => crate::send_ping(outgoing_c, tof_component, id),
                          _                    => error!("The ping command is not implemented for this TofComponent!")
                        }
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

                            match outgoing.send(tp) {
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

                            match outgoing.send(tp) {
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

                            match outgoing.send(tp) {
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
                        match rb_noi_subcalibration(&run_config, &outgoing) {
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
                        match rb_voltage_subcalibration(&run_config, &outgoing, voltage_val) {
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
                        match rb_timing_subcalibration(&run_config, &outgoing, voltage_val) {
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
                        match rb_calibration(&run_config, &outgoing) {
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
              match outgoing.send(tp) {
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
    // check for thread termination
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
  }
  // sleep most of the time
  sleep(sleep_time);
}
