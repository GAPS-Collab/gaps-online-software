use std::sync::{
    Arc,
    Mutex,
};
use std::thread::sleep;
use std::time::{Duration, Instant};

//use std::time::Instant;
use crossbeam_channel::{Receiver,
                        Sender};

use tof_dataclasses::commands::{TofCommand, TofCommandCode, TofCommandResp, TofResponse};
use tof_dataclasses::errors::{CmdError, SetError};
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
pub fn flight_cpu_listener(flight_address_sub  : &str,
                           flight_address_pub  : &str,
                           incoming        : &Receiver<TofPacket>,
                           outgoing        : &Sender<TofPacket>,
                           cmd_interval    : u64,
                           thread_control  : Arc<Mutex<ThreadControl>>) {
  // create 0MQ sockets
  let ctx = zmq::Context::new();
  // I guess expect is fine here, see above
  let cmd_socket = ctx.socket(zmq::SUB).expect("Unable to create 0MQ SUB socket!");
  cmd_socket.set_subscribe(b"").expect("Unable to subscribe to empty topic!");
  info!("Will set up 0MQ SUB socket to listen for flight cpu commands at address {flight_address_sub}");
  cmd_socket.bind(flight_address_sub).expect("Unable to bind to data (SUB) socket {flight_address_sub}");
  info!("ZMQ SUB Socket for flight cpu listener bound to {flight_address_sub}");

  let mut timer     = Instant::now();
  let sleep_time    = Duration::from_secs(cmd_interval);
  'main: loop {
    info!("Main loop tof listener executing");
    if timer.elapsed().as_secs() >= cmd_interval {
      info!("Listening for flight CPU comms");
      timer     = Instant::now();

      match cmd_socket.recv_bytes(0) {
        Err(err) => trace!("No new packet, err {err}"),
        Ok(buffer) => {
          match TofPacket::from_bytestream(&buffer, &mut 1) {
            Err(err) => {
              error!("Unknown packet...{:?}", err);
              continue;  
            },
            Ok(pack) => {
              info!("Got new tof packet {}", pack.packet_type);
              match pack.packet_type {
                PacketType::TofCommand => {
                  // we have to strip off the topic
                  match TofCommand::from_bytestream(&pack.payload, &mut 0) {
                    Err(err) => {
                      error!("Problem decoding command {}", err);
                    }
                    Ok(cmd)  => {
                      // we got a valid tof command
                      // interpret it

                      // forward it and wait for the 
                      // response
                      let tof_resp  = TofResponse::GeneralFail(TofCommandResp::RespErrNotImplemented as u32);
                      let resp_not_implemented = crate::prefix_tof_cpu(&mut tof_resp.to_bytestream());
                      //let resp_not_implemented = TofResponse::GeneralFail(RESP_ERR_NOTIMPLEMENTED);
                      let outgoing_c = outgoing.clone();

                      let resp_socket = ctx.socket(zmq::PUB).expect("Unable to create 0MQ PUB socket!");
                      info!("Will set up 0MQ PUB socket to reply to flight cpu commands at address {flight_address_pub}");
                      resp_socket.connect(flight_address_pub).expect("Unable to bind to data (PUB) socket {data_adress}");
                      info!("ZMQ SUB Socket for flight cpu responder bound to {flight_address_pub}");

                      let return_val: Result<TofCommandCode, CmdError>;
                      match cmd {
                        TofCommand::Unknown (_) => {
                          info!("Received unknown command");
                          return_val = Err(CmdError::UnknownError);
                        },
                        TofCommand::Listen (_) => {
                          info!("Listening inception!");
                          return_val = Err(CmdError::ListenError);
                        },
                        TofCommand::Ping (value) => {
                          info!("Received ping command");
                          // MSB third 8 bits are
                          let tof_component: TofComponent = TofComponent::from(((value & (MASK_CMD_8BIT << 8)) >> 8) as u8);
                          // MSB fourth 8 bits are 
                          let id: u8 = (value & MASK_CMD_8BIT) as u8;

                          if tof_component == TofComponent::Unknown {
                            info!("The command is not valid for {}", TofComponent::Unknown);
                            // The packet was not for this RB so bye
                            continue;
                          } else {
                            match tof_component {
                              TofComponent::TofCpu => return_val = crate::send_ping_response(Some(resp_socket)),
                              TofComponent::RB  |
                              TofComponent::LTB |
                              TofComponent::MT     => return_val = crate::send_ping(Some(resp_socket), outgoing_c,  tof_component, id),
                              _                    => {
                                error!("The ping command is not implemented for this TofComponent!");
                                return_val = Err(CmdError::NotImplementedError);
                              }
                            }
                          }
                        },
                        TofCommand::Moni (value) => {
                          info!("Received moni command");
                          // MSB third 8 bits are 
                          let tof_component: TofComponent = TofComponent::from(((value & (MASK_CMD_8BIT << 8)) >> 8) as u8);
                          // MSB fourth 8 bits are 
                          let id: u8 = (value & MASK_CMD_8BIT) as u8;

                          if tof_component == TofComponent::Unknown {
                            // The packet was not for this RB so bye
                            continue;
                          } else {
                            match tof_component {
                              TofComponent::TofCpu => return_val = crate::send_moni_response(Some(resp_socket)),
                              TofComponent::RB  |
                              TofComponent::LTB |
                              TofComponent::MT     => return_val = crate::send_moni(Some(resp_socket), outgoing_c,  tof_component, id),
                              _                    => {
                                error!("The moni command is not implemented for this TofComponent!");
                                return_val = Err(CmdError::NotImplementedError);
                              }
                            }
                          }
                        },
                        TofCommand::Power   (value) => {
                          info!("Received power command");
                          // MSB second 8 bits are tof component
                          let tof_component: TofComponent = TofComponent::from(((value & (MASK_CMD_8BIT << 16)) >> 16) as u8);
                          // MSB third 8 bits are 
                          let component_id: u8 = ((value & MASK_CMD_8BIT << 8) >> 8) as u8;
                          // MSB fourth 8 bits are 
                          let power_status: PowerStatusEnum = PowerStatusEnum::from((value & MASK_CMD_8BIT) as u8);
                          // TODO implement proper routines

                          match tof_component {
                            TofComponent::All      |
                            TofComponent::MT       |
                            TofComponent::AllButMT => {
                              return_val = crate::send_power(Some(resp_socket), outgoing_c,  tof_component, power_status);
                            },
                            TofComponent::LTB      |
                            TofComponent::Preamp   => {
                              return_val = crate::send_power_id(Some(resp_socket), outgoing_c,  tof_component, power_status, component_id);
                            },
                            TofComponent::TofCpu   => {
                              return_val = crate::send_power_response(Some(resp_socket), power_status);
                            },
                            _                      => {
                              return_val = Err(CmdError::NotImplementedError);
                              error!("Power operation not implemented for Unknown!")
                            }
                          }
                        },
                        TofCommand::SetThresholds   (value) =>  {
                          info!("Received set threshold command! Will communicate to LTBs");
                          // MSB first 8 bits are LTB ID
                          let ltb_id: u8 = ((value & (MASK_CMD_8BIT << 24)) >> 24) as u8;
                          // MSB second 8 bits are LTB ID
                          let threshold_name: LTBThresholdName = LTBThresholdName::from(((value & (MASK_CMD_8BIT << 16)) >> 16) as u8);
                          // MSB third 16 bits are extra (not used)
                          let threshold_level: u16 = (value & MASK_CMD_16BIT) as u16;
                          return_val = crate::send_ltb_threshold_set(Some(resp_socket), outgoing_c,  ltb_id, threshold_name, threshold_level);
                        },
                        TofCommand::SetMTConfig  (_) => {
                          info!("Received set MT config command");
                          warn!("Not implemented");
                          return_val = Err(CmdError::NotImplementedError);
                        },
                        TofCommand::SetPreampBias   (value) =>  {
                          info!("Received set preamp bias command! Will communicate to preamps");
                          // MSB second 8 bits are LTB ID
                          let preamp_id: u8 = ((value & (MASK_CMD_8BIT << 16)) >> 16) as u8;
                          // MSB third 16 bits are extra (not used)
                          let preamp_bias: u16 = (value & MASK_CMD_16BIT) as u16;
                          return_val = crate::send_preamp_bias_set(Some(resp_socket), outgoing_c,  preamp_id, preamp_bias);
                        },
                        TofCommand::DataRunStop(value)   => {
                          info!("Received data run stop command");
                          // MSB fourth 8 bits are RB ID
                          let rb_id: u8 = (value & MASK_CMD_8BIT) as u8;

                          return_val = crate::send_run_stop(Some(resp_socket), outgoing_c,  rb_id);
                        },
                        TofCommand::DataRunStart (value) => {
                          info!("Received data run start command");
                          // MSB second 8 bits are run_type
                          let run_type: u8 = ((value & (MASK_CMD_8BIT << 16)) >> 16) as u8;
                          // MSB third 8 bits are RB ID
                          let rb_id: u8    = ((value & (MASK_CMD_8BIT << 8)) >> 8) as u8;
                          // MSB fourth 8 bits are event number
                          let event_no: u8 = (value & MASK_CMD_8BIT) as u8;
                          // let's start a run. The value of the TofCommnad shall be 
                          // nevents

                          return_val = crate::send_run_start(Some(resp_socket), outgoing_c,  run_type, rb_id, event_no);
                        },
                        TofCommand::StartValidationRun  (_) => {
                          info!("Received start validation run command");
                          warn!("Not implemented");
                          return_val = Err(CmdError::NotImplementedError);
                        },
                        TofCommand::GetFullWaveforms  (_) => {
                          info!("Received get full waveforms command");
                          warn!("Not implemented");
                          return_val = Err(CmdError::NotImplementedError);
                        },
                        // Voltage and timing calibration is connected now
                        TofCommand::NoiCalibration (value) => {
                          info!("Received no input calibration command");
                          // MSB third 8 bits are RB ID
                          let rb_id: u8 = ((value & (MASK_CMD_8BIT << 8)) >> 8) as u8;
                          // MSB fourth 8 bits are extra (not used)
                          let extra: u8 = (value & MASK_CMD_8BIT) as u8;
                          
                          return_val = crate::send_noi_calibration(Some(resp_socket), outgoing_c,  rb_id, extra);
                        },
                        TofCommand::VoltageCalibration (value) => {
                          info!("Received voltage calibration command");
                          // MSB first 16 bits are voltage level
                          let voltage_level: u16 = ((value & (MASK_CMD_16BIT << 16)) >> 16) as u16;
                          // MSB third 8 bits are RB ID
                          let rb_id: u8 = ((value & (MASK_CMD_8BIT << 8)) >> 8) as u8;
                          // MSB fourth 8 bits are extra (not used)
                          let extra: u8 = (value & MASK_CMD_8BIT) as u8;
                          
                          return_val = crate::send_voltage_calibration(Some(resp_socket), outgoing_c,  voltage_level, rb_id, extra);
                        },
                        TofCommand::TimingCalibration  (value) => {
                          info!("Received timing calibration command");
                          // MSB first 16 bits are voltage level
                          let voltage_level: u16 = ((value & (MASK_CMD_16BIT << 16)) >> 16) as u16;
                          // MSB third 8 bits are RB ID
                          let rb_id: u8 = ((value & (MASK_CMD_8BIT << 8)) >> 8) as u8;
                          // MSB fourth 8 bits are extra (not used)
                          let extra: u8 = (value & MASK_CMD_8BIT) as u8;
                          
                          return_val = crate::send_timing_calibration(Some(resp_socket), outgoing_c,  voltage_level, rb_id, extra);
                        },
                        TofCommand::DefaultCalibration  (value) => {
                          info!("Received default calibration command");
                          // MSB first 16 bits are voltage level
                          let voltage_level: u16 = ((value & (MASK_CMD_16BIT << 16)) >> 16) as u16;
                          // MSB third 8 bits are RB ID
                          let rb_id: u8 = ((value & (MASK_CMD_8BIT << 8)) >> 8) as u8;
                          // MSB fourth 8 bits are extra (not used)
                          let extra: u8 = (value & MASK_CMD_8BIT) as u8;

                          return_val = crate::send_default_calibration(Some(resp_socket), outgoing_c,  voltage_level, rb_id, extra);
                        },
                        TofCommand::SetRBDataBufSize   (_) => {
                          info!("Received set RB data buf size command");
                          warn!("Not implemented");
                          return_val = Err(CmdError::NotImplementedError);
                        },
                        TofCommand::TriggerModeForced  (_) => {
                          info!("Received trigger mode forced command");
                          warn!("Not implemented");
                          return_val = Err(CmdError::NotImplementedError);
                        },
                        TofCommand::TriggerModeForcedMTB   (_) => {
                          info!("Received trigger mode forced MTB command");
                          warn!("Not implemented");
                          return_val = Err(CmdError::NotImplementedError);
                        },
                        TofCommand::UnspoolEventCache  (_) => {
                          info!("Received unspool event cache command");
                          warn!("Not implemented");
                          return_val = Err(CmdError::NotImplementedError);
                        },
                        TofCommand::SystemdReboot  (_) => {
                          info!("Received systemd reboot command");
                          warn!("Not implemented");
                          return_val = Err(CmdError::NotImplementedError);
                        }
                      }
                      // deal with return values
                      let resp_socket = ctx.socket(zmq::PUB).expect("Unable to create 0MQ PUB socket!");
                      info!("Will set up 0MQ PUB socket to send status to flight cpu commands at address {flight_address_pub}");
                      resp_socket.connect(flight_address_pub).expect("Unable to bind to data (PUB) socket {data_adress}");
                      info!("ZMQ SUB Socket for flight cpu responder bound to {flight_address_pub}");
                      match return_val {
                        Err(cmd_error) => {
                          info!("Cmd Error: {cmd_error}");
                          let r = TofResponse::GeneralFail(TofCommandResp::RespErrUnexecutable as u32);
                          match resp_socket.send(r.to_bytestream(),0) {
                            Err(err) => warn!("Can not send response!, Err {err}"),
                            Ok(_)    => info!("Responded to {cmd_error}!")
                          }
                        },
                        Ok(tof_command)  => {
                          info!("Cmd resp: {tof_command}");
                          let r = TofResponse::Success(TofCommandResp::RespSuccFingersCrossed as u32);
                          match resp_socket.send(r.to_bytestream(),0) {
                            Err(err) => warn!("Can not send response!, Err {err}"),
                            Ok(_)    => info!("Responded to {tof_command}!")
                          }
                        }
                      }
                    }
                  }  
                },
                _ => {
                  error!("Can not respond to {}", pack);
                }
              }
            }
          }
        }
      }
    }
    // sleep most of the time
    sleep(sleep_time);
    // check for thread termination
    match thread_control.lock() {
      Ok(tc) => {
        if tc.stop_flag {
          info!("Received stop signal. Will stop thread!");
          break 'main;
        }
      },
      Err(err) => {
        trace!("Can't acquire lock! {err}");
      },
    }
  }
}
