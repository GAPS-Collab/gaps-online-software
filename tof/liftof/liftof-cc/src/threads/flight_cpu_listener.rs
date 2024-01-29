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
      info!("Listening for flight CPU comms");
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
                      let tof_component: TofComponent = TofComponent::from(((value | MASK_CMD_8BIT << 8) >> 8) as u8);
                      // MSB fourth 8 bits are 
                      let id: u8 = (value | MASK_CMD_8BIT) as u8;

                      if tof_component == TofComponent::Unknown {
                        info!("The command is not valid for {}", TofComponent::Unknown);
                        // The packet was not for this RB so bye
                        continue;
                      } else {
                        match tof_component {
                          TofComponent::TofCpu => return_val = crate::send_ping_response(outgoing_c),
                          TofComponent::RB  |
                          TofComponent::LTB |
                          TofComponent::MT     => return_val = crate::send_ping(outgoing_c, tof_component, id),
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
                      let tof_component: TofComponent = TofComponent::from(((value | MASK_CMD_8BIT << 8) >> 8) as u8);
                      // MSB fourth 8 bits are 
                      let id: u8 = (value | MASK_CMD_8BIT) as u8;

                      if tof_component == TofComponent::Unknown {
                        // The packet was not for this RB so bye
                        continue;
                      } else {
                        match tof_component {
                          TofComponent::TofCpu => return_val = crate::send_moni_response(outgoing_c),
                          TofComponent::RB  |
                          TofComponent::LTB |
                          TofComponent::MT     => return_val = crate::send_moni(outgoing_c, tof_component, id),
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
                      let tof_component: TofComponent = TofComponent::from(((value | (MASK_CMD_8BIT << 16)) >> 16) as u8);
                      // MSB third 8 bits are 
                      let component_id: u8 = ((value | MASK_CMD_8BIT << 8) >> 8) as u8;
                      // MSB fourth 8 bits are 
                      let power_status: PowerStatusEnum = PowerStatusEnum::from((value | MASK_CMD_8BIT) as u8);
                      // TODO implement proper routines

                      match tof_component {
                        TofComponent::All      => {
                          return_val = crate::send_power(outgoing_c, TofComponent::All, power_status);
                        }, //power_all(cmd_socket, component_id, status),
                        TofComponent::MT       => {
                          return_val = crate::send_power(outgoing_c, TofComponent::MT, power_status);
                        }, //power_mt(cmd_socket, component_id, status),
                        TofComponent::AllButMT => {
                          return_val = crate::send_power(outgoing_c, TofComponent::AllButMT, power_status);
                        }, //power_allbutmt(cmd_socket, component_id, status),
                        TofComponent::LTB      => {
                          return_val = crate::send_power_id(outgoing_c, TofComponent::LTB, power_status, component_id);
                        },
                        TofComponent::Preamp   => {
                          return_val = crate::send_power_id(outgoing_c, TofComponent::Preamp, power_status, component_id);
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
                      let ltb_id: u8 = ((value | (MASK_CMD_8BIT << 24)) >> 24) as u8;
                      // MSB second 8 bits are LTB ID
                      let threshold_name: LTBThresholdName = LTBThresholdName::from(((value | (MASK_CMD_8BIT << 16)) >> 16) as u8);
                      // MSB third 16 bits are extra (not used)
                      let threshold_level: u16 = (value | MASK_CMD_16BIT) as u16;
                      return_val = crate::send_ltb_threshold_set(outgoing_c, ltb_id, threshold_name, threshold_level);
                    },
                    TofCommand::SetMTConfig  (_) => {
                      info!("Received set MT config command");
                      warn!("Not implemented");
                      return_val = Err(CmdError::NotImplementedError);
                    },
                    TofCommand::SetPreampBias   (value) =>  {
                      info!("Received set preamp bias command! Will communicate to preamps");
                      // MSB second 8 bits are LTB ID
                      let preamp_id: u8 = ((value | (MASK_CMD_8BIT << 16)) >> 16) as u8;
                      // MSB third 16 bits are extra (not used)
                      let preamp_bias: u16 = (value | MASK_CMD_16BIT) as u16;
                      return_val = crate::send_preamp_bias_set(outgoing_c, preamp_id, preamp_bias);
                    },
                    TofCommand::DataRunStop(value)   => {
                      info!("Received data run stop command");
                      // MSB fourth 8 bits are RB ID
                      let rb_id: u8 = (value | MASK_CMD_8BIT) as u8;

                      return_val = crate::send_run_stop(outgoing_c, rb_id);
                    },
                    TofCommand::DataRunStart (value) => {
                      info!("Received data run start command");
                      // MSB second 8 bits are run_type
                      let run_type: u8 = ((value | (MASK_CMD_8BIT << 16)) >> 16) as u8;
                      // MSB third 8 bits are RB ID
                      let rb_id: u8    = ((value | (MASK_CMD_8BIT << 8)) >> 8) as u8;
                      // MSB fourth 8 bits are event number
                      let event_no: u8 = (value | MASK_CMD_8BIT) as u8;
                      // let's start a run. The value of the TofCommnad shall be 
                      // nevents

                      return_val = crate::send_run_start(outgoing_c, run_type, rb_id, event_no);
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
                      let rb_id: u8 = ((value | (MASK_CMD_8BIT << 8)) >> 8) as u8;
                      // MSB fourth 8 bits are extra (not used)
                      let extra: u8 = (value | MASK_CMD_8BIT) as u8;
                      
                      return_val = crate::send_noi_calibration(outgoing_c, rb_id, extra);
                    },
                    TofCommand::VoltageCalibration (value) => {
                      info!("Received voltage calibration command");
                      // MSB first 16 bits are voltage level
                      let voltage_level: u16 = ((value | (MASK_CMD_16BIT << 16)) >> 16) as u16;
                      // MSB third 8 bits are RB ID
                      let rb_id: u8 = ((value | (MASK_CMD_8BIT << 8)) >> 8) as u8;
                      // MSB fourth 8 bits are extra (not used)
                      let extra: u8 = (value | MASK_CMD_8BIT) as u8;
                      
                      return_val = crate::send_voltage_calibration(outgoing_c, voltage_level, rb_id, extra);
                    },
                    TofCommand::TimingCalibration  (value) => {
                      info!("Received timing calibration command");
                      // MSB first 16 bits are voltage level
                      let voltage_level: u16 = ((value | (MASK_CMD_16BIT << 16)) >> 16) as u16;
                      // MSB third 8 bits are RB ID
                      let rb_id: u8 = ((value | (MASK_CMD_8BIT << 8)) >> 8) as u8;
                      // MSB fourth 8 bits are extra (not used)
                      let extra: u8 = (value | MASK_CMD_8BIT) as u8;
                      
                      return_val = crate::send_timing_calibration(outgoing_c, voltage_level, rb_id, extra);
                    },
                    TofCommand::DefaultCalibration  (value) => {
                      info!("Received default calibration command");
                      // MSB first 16 bits are voltage level
                      let voltage_level: u16 = ((value | (MASK_CMD_16BIT << 16)) >> 16) as u16;
                      // MSB third 8 bits are RB ID
                      let rb_id: u8 = ((value | (MASK_CMD_8BIT << 8)) >> 8) as u8;
                      // MSB fourth 8 bits are extra (not used)
                      let extra: u8 = (value | MASK_CMD_8BIT) as u8;

                      return_val = crate::send_default_calibration(outgoing_c, voltage_level, rb_id, extra);
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
                  match return_val {
                    Err(cmd_error) => {
                      let r = TofResponse::GeneralFail(TofCommandResp::RespErrUnexecutable as u32);
                      match cmd_socket.send(r.to_bytestream(),0) {
                        Err(err) => warn!("Can not send response!, Err {err}"),
                        Ok(_)    => info!("Responded to {cmd_error}!")
                      }
                    },
                    Ok(tof_command)  => {
                      let r = TofResponse::Success(TofCommandResp::RespSuccFingersCrossed as u32);
                      match cmd_socket.send(r.to_bytestream(),0) {
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
