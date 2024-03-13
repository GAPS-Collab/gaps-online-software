use crossbeam_channel::Sender;
use liftof_lib::{PowerStatusEnum, TofComponent, LTBThresholdName};
use tof_dataclasses::errors::CmdError;
use tof_dataclasses::packets::{TofPacket, PacketType};
use tof_dataclasses::commands::{
    TofCommand,
    TofCommandCode,
    //TofCommandResp,
    //TofResponse
};
use tof_dataclasses::constants::PAD_CMD_32BIT;
use tof_dataclasses::serialization::Serialization;
use zmq::Socket;

#[macro_use] extern crate log;
extern crate clap;
extern crate colored;

//extern crate local_ip_address;
extern crate crossbeam_channel;
extern crate liftof_lib;

extern crate zmq;
extern crate tof_dataclasses;
extern crate tof_control;

pub mod constants;
pub mod threads;
pub mod settings;

/// Function that just replies to a moni command send to tofcpu
pub fn send_power_response(resp_socket_opt: Option<Socket>,
                           power_status: PowerStatusEnum)
                           -> Result<TofCommandCode, CmdError> {

  match power_status {
    PowerStatusEnum::ON  => (), // nothing to do here, its already on if it received
    PowerStatusEnum::OFF => {
      error!("Command not implemented"); // TODO HOW DO WE SOFT REB
      return Err(CmdError::PowerError);
    },
    _ => {
      error!("Command not supported");
      return Err(CmdError::PowerError);
    }
  }
  let mut tp = TofPacket::new();
  tp.packet_type = PacketType::CPUMoniData;
  tp.payload = vec![TofComponent::TofCpu as u8, 0u8];
  // TODO HOW TO SOF REBOOT TOFCPU
  
  match resp_socket_opt {
    None => Ok(TofCommandCode::CmdPower),
    Some(resp_socket) => {
      match resp_socket.send(tp.to_bytestream(), 0) {
        Err(err) => {
          error!("Unable to reply to command, error{err}");
          return Err(CmdError::MoniError);
        },
        Ok(_)    => {
          info!("Replied to moni command");
          return Ok(TofCommandCode::CmdMoni)
        }
      }
    }
  }
}

/// Power function that targets the component specified, no ID
pub fn send_power(resp_socket_opt: Option<Socket>,
                  outgoing: Sender<TofPacket>,
                  component: TofComponent,
                  power_status: PowerStatusEnum)
                  -> Result<TofCommandCode, CmdError> {
  // no ID in the payload
  let payload: u32 = PAD_CMD_32BIT | (component as u32) << 24 | (power_status as u32);
  let power = TofCommand::Power(payload);
  
  let tp = TofPacket::from(&power);
  let tp_c: TofPacket = tp.clone();

  match outgoing.send(tp) {
    Err(err) => {
      error!("Unable to send command, error{err}");
      return Err(CmdError::PowerError);
    },
    Ok(_)    => {
      info!("Power command sent to RBs")
    }
  }
  
  match resp_socket_opt {
    None => Ok(TofCommandCode::CmdPower),
    Some(resp_socket) => {
      match resp_socket.send(tp_c.to_bytestream(), 0) {
        Err(err) => {
          error!("Unable to reply to command, error{err}");
          return Err(CmdError::PowerError);
        },
        Ok(_)    => {
          info!("Replied to power command");
          return Ok(TofCommandCode::CmdPower)
        }
      }
    }
  }
}

/// Power function that targets the component specified with supplied ID
pub fn send_power_id(resp_socket_opt: Option<Socket>,
                     outgoing: Sender<TofPacket>,
                     component: TofComponent,
                     power_status: PowerStatusEnum,
                     component_id: u8)
                     -> Result<TofCommandCode, CmdError> {
  let payload: u32 = PAD_CMD_32BIT | (component as u32) << 24 | (component_id as u32) << 16 | (power_status as u32);
  let power_id = TofCommand::Power(payload);
  
  let tp = TofPacket::from(&power_id);
  let tp_c: TofPacket = tp.clone();
  
  match outgoing.send(tp) {
    Err(err) => {
      error!("Unable to send command, error{err}");
      return Err(CmdError::PowerError);
    },
    Ok(_)    => {
      info!("Power command sent to component")
    }
  }
  
  match resp_socket_opt {
    None => Ok(TofCommandCode::CmdPower),
    Some(resp_socket) => {
      match resp_socket.send(tp_c.to_bytestream(), 0) {
        Err(err) => {
          error!("Unable to reply to command, error{err}");
          return Err(CmdError::PowerError);
        },
        Ok(_)    => {
          info!("Replied to power command");
          return Ok(TofCommandCode::CmdPower)
        }
      }
    }
  }
}

/// Power function that targets the component specified with supplied ID
pub fn send_power_preamp(resp_socket_opt: Option<Socket>,
                         outgoing: Sender<TofPacket>,
                         power_status: PowerStatusEnum,
                         preamp_id: u8,
                         preamp_bias: u16)
                         -> Result<TofCommandCode, CmdError> {
  // bias only if ON and Cycle
  let payload: u32 = match power_status {
    PowerStatusEnum::OFF => 
      (TofComponent::Preamp as u32) << 16 | (preamp_id as u32) << 8 | 0u32,
    PowerStatusEnum::ON => 
      (TofComponent::Preamp as u32) << 16 | (preamp_id as u32) << 8 | preamp_bias as u32,
    PowerStatusEnum::Cycle => 
      (TofComponent::Preamp as u32) << 16 | (preamp_id as u32) << 8 | preamp_bias as u32,
    _ => {
      warn!("Status unknown, not doing stuff.");
      return Err(CmdError::PowerError);
    }
  };
  let power_preamp = TofCommand::Power(payload);
  
  let tp = TofPacket::from(&power_preamp);
  let tp_c: TofPacket = tp.clone();
  
  match outgoing.send(tp) {
    Err(err) => {
      error!("Unable to send command, error{err}");
      return Err(CmdError::PowerError);
    },
    Ok(_)    => {
      info!("Power command sent to Preamps")
    }
  }
  
  match resp_socket_opt {
    None => Ok(TofCommandCode::CmdPower),
    Some(resp_socket) => {
      match resp_socket.send(tp_c.to_bytestream(), 0) {
        Err(err) => {
          error!("Unable to reply to command, error{err}");
          return Err(CmdError::PowerError);
        },
        Ok(_)    => {
          info!("Replied to power command");
          return Ok(TofCommandCode::CmdPower)
        }
      }
    }
  }
}

/// Default function that starts calibration on all RBs
/// with default values.
pub fn send_default_calibration(resp_socket_opt: Option<Socket>,
                                outgoing: Sender<TofPacket>,
                                voltage_level: u16,
                                rb_id: u8,
                                extra: u8)
                                -> Result<TofCommandCode, CmdError> {
  let payload: u32
    = (voltage_level as u32) << 16 | (rb_id as u32) << 8 | (extra as u32);
  let default_calib = TofCommand::DefaultCalibration(payload);

  let tp = TofPacket::from(&default_calib);
  let tp_c: TofPacket = tp.clone();
  
  match outgoing.send(tp) {
    Err(err) => {
      error!("Unable to send command, error{err}");
      return Err(CmdError::CalibrationError);
    },
    Ok(_)    => {
      info!("Calibration command sent")
    }
  }
  
  match resp_socket_opt {
    None => Ok(TofCommandCode::CmdDefaultCalibration),
    Some(resp_socket) => {
      match resp_socket.send(tp_c.to_bytestream(), 0) {
        Err(err) => {
          error!("Unable to reply to command, error{err}");
          return Err(CmdError::CalibrationError);
        },
        Ok(_)    => {
          info!("Replied to calibration command");
          return Ok(TofCommandCode::CmdDefaultCalibration)
        }
      }
    }
  }
}

/// Function that starts voltage calibration on a specific
/// RB with the specified voltage level and extras (not
/// implemented)
pub fn send_noi_calibration(resp_socket_opt: Option<Socket>,
                            outgoing: Sender<TofPacket>,
                            rb_id: u8,
                            extra: u8)
                            -> Result<TofCommandCode, CmdError> {
  let payload: u32 = PAD_CMD_32BIT | (rb_id as u32) << 8 | (extra as u32);
  let noi_calib = TofCommand::NoiCalibration(payload);

  let tp = TofPacket::from(&noi_calib);
  let tp_c: TofPacket = tp.clone();
  
  match outgoing.send(tp) {
    Err(err) => {
      error!("Unable to send command, error{err}");
      return Err(CmdError::PowerError);
    },
    Ok(_)    => {
      info!("Calibration command sent")
    }
  }
  
  match resp_socket_opt {
    None => Ok(TofCommandCode::CmdPower),
    Some(resp_socket) => {
      match resp_socket.send(tp_c.to_bytestream(), 0) {
        Err(err) => {
          error!("Unable to reply to command, error{err}");
          return Err(CmdError::CalibrationError);
        },
        Ok(_)    => {
          info!("Replied to calibration command");
          return Ok(TofCommandCode::CmdNoiCalibration)
        }
      }
    }
  }
}

/// Function that starts voltage calibration on a specific
/// RB with the specified voltage level and extras (not
/// implemented)
pub fn send_voltage_calibration(resp_socket_opt: Option<Socket>,
                                outgoing: Sender<TofPacket>,
                                voltage_level: u16,
                                rb_id: u8,
                                extra: u8)
                                -> Result<TofCommandCode, CmdError> {
  let payload: u32
    = (voltage_level as u32) << 16 | (rb_id as u32) << 8 | (extra as u32);
  let voltage_calib = TofCommand::VoltageCalibration(payload);

  let tp = TofPacket::from(&voltage_calib);
  let tp_c: TofPacket = tp.clone();
  
  match outgoing.send(tp) {
    Err(err) => {
      error!("Unable to send command, error{err}");
      return Err(CmdError::PowerError);
    },
    Ok(_)    => {
      info!("Calibration command sent")
    }
  }
  
  match resp_socket_opt {
    None => Ok(TofCommandCode::CmdPower),
    Some(resp_socket) => {
      match resp_socket.send(tp_c.to_bytestream(), 0) {
        Err(err) => {
          error!("Unable to reply to command, error{err}");
          return Err(CmdError::CalibrationError);
        },
        Ok(_)    => {
          info!("Replied to calibration command");
          return Ok(TofCommandCode::CmdVoltageCalibration)
        }
      }
    }
  }
}

/// Function that starts timing calibration on a specific
/// RB with the specified voltage level and extras (not
/// implemented)
pub fn send_timing_calibration(resp_socket_opt: Option<Socket>,
                               outgoing: Sender<TofPacket>,
                               voltage_level: u16,
                               rb_id: u8,
                               extra: u8)
                               -> Result<TofCommandCode, CmdError> {
  let payload: u32
    = (voltage_level as u32) << 16 | (rb_id as u32) << 8 | (extra as u32);
  let timing_calib = TofCommand::TimingCalibration(payload);

  let tp = TofPacket::from(&timing_calib);
  let tp_c: TofPacket = tp.clone();
  
  match outgoing.send(tp) {
    Err(err) => {
      error!("Unable to send command, error{err}");
      return Err(CmdError::PowerError);
    },
    Ok(_)    => {
      info!("Calibration command sent")
    }
  }
  
  match resp_socket_opt {
    None => Ok(TofCommandCode::CmdPower),
    Some(resp_socket) => {
      match resp_socket.send(tp_c.to_bytestream(), 0) {
        Err(err) => {
          error!("Unable to reply to command, error{err}");
          return Err(CmdError::CalibrationError);
        },
        Ok(_)    => {
          info!("Replied to calibration command");
          return Ok(TofCommandCode::CmdTimingCalibration)
        }
      }
    }
  }
}

/// Function that sends the threshold to be set on all or
/// specific LTBs
pub fn send_ltb_threshold_set(resp_socket_opt: Option<Socket>,
                              outgoing: Sender<TofPacket>,
                              ltb_id: u8,
                              threshold_name: LTBThresholdName,
                              threshold_level: u16)
                              -> Result<TofCommandCode, CmdError> {
  let payload: u32
  = (ltb_id as u32) << 24 | (threshold_name as u32) << 16 | (threshold_level as u32);
  let ltb_threshold = TofCommand::SetThresholds(payload);

  let tp = TofPacket::from(&ltb_threshold);
  let tp_c: TofPacket = tp.clone();
  
  match outgoing.send(tp) {
    Err(err) => {
      error!("Unable to send command, error{err}");
      return Err(CmdError::PowerError);
    },
    Ok(_)    => {
      info!("Threshold set command sent")
    }
  }
  
  match resp_socket_opt {
    None => Ok(TofCommandCode::CmdPower),
    Some(resp_socket) => {
      match resp_socket.send(tp_c.to_bytestream(), 0) {
        Err(err) => {
          error!("Unable to reply to command, error{err}");
          return Err(CmdError::ThresholdSetError);
        },
        Ok(_)    => {
          info!("Replied to threshold set command");
          return Ok(TofCommandCode::CmdSetThresholds)
        }
      }
    }
  }
}

/// Function that sends the threshold to be set on all or
/// specific LTBs
pub fn send_preamp_bias_set(resp_socket_opt: Option<Socket>,
                            outgoing: Sender<TofPacket>,
                            preamp_id: u8,
                            preamp_bias: u16)
                            -> Result<TofCommandCode, CmdError> {
  let payload: u32
  = PAD_CMD_32BIT | (preamp_id as u32) << 16 | (preamp_bias as u32);
  let preamp_bias = TofCommand::SetPreampBias(payload);

  let tp = TofPacket::from(&preamp_bias);
  let tp_c: TofPacket = tp.clone();
  
  match outgoing.send(tp) {
    Err(err) => {
      error!("Unable to send command, error{err}");
      return Err(CmdError::PowerError);
    },
    Ok(_)    => {
      info!("Preamp bias set command sent")
    }
  }
  
  match resp_socket_opt {
    None => Ok(TofCommandCode::CmdPower),
    Some(resp_socket) => {
      match resp_socket.send(tp_c.to_bytestream(), 0) {
        Err(err) => {
          error!("Unable to reply to command, error{err}");
          return Err(CmdError::ThresholdSetError);
        },
        Ok(_)    => {
          info!("Replied to Preamp bias set command");
          return Ok(TofCommandCode::CmdSetPreampBias)
        }
      }
    }
  }
}

/// Default function that starts run data taking on all RBs
/// with default values.
pub fn send_run_start(resp_socket_opt: Option<Socket>,
                      outgoing: Sender<TofPacket>,
                      run_type: u8,
                      rb_id: u8,
                      event_no: u8)
                      -> Result<TofCommandCode, CmdError> {
  let payload: u32
  = PAD_CMD_32BIT | (run_type as u32) << 16 | (rb_id as u32) << 8 | (event_no as u32);
  let run_start = TofCommand::DataRunStart(payload);

  let tp = TofPacket::from(&run_start);
  let tp_c: TofPacket = tp.clone();
  
  match outgoing.send(tp) {
    Err(err) => {
      error!("Unable to send command, error{err}");
      return Err(CmdError::PowerError);
    },
    Ok(_)    => {
      info!("Start run command sent")
    }
  }
  
  match resp_socket_opt {
    None => Ok(TofCommandCode::CmdPower),
    Some(resp_socket) => {
      match resp_socket.send(tp_c.to_bytestream(), 0) {
        Err(err) => {
          error!("Unable to reply to command, error{err}");
          return Err(CmdError::RunStartError);
        },
        Ok(_)    => {
          info!("Replied to start run command");
          return Ok(TofCommandCode::CmdDataRunStart)
        }
      }
    }
  }
}

/// Default function that starts run data taking on all RBs
/// with default values.
pub fn send_run_stop(resp_socket_opt: Option<Socket>,
                     outgoing: Sender<TofPacket>,
                     rb_id: u8)
                     -> Result<TofCommandCode, CmdError> {
  let payload: u32 = PAD_CMD_32BIT | (rb_id as u32);
  let run_stop = TofCommand::DataRunStop(payload);

  let tp = TofPacket::from(&run_stop);
  let tp_c: TofPacket = tp.clone();
  
  match outgoing.send(tp) {
    Err(err) => {
      error!("Unable to send command, error{err}");
      return Err(CmdError::PowerError);
    },
    Ok(_)    => {
      info!("Stop run command sent")
    }
  }
  
  match resp_socket_opt {
    None => Ok(TofCommandCode::CmdPower),
    Some(resp_socket) => {
      match resp_socket.send(tp_c.to_bytestream(), 0) {
        Err(err) => {
          error!("Unable to reply to command, error{err}");
          return Err(CmdError::RunStopError);
        },
        Ok(_)    => {
          info!("Replied to stop run command");
          return Ok(TofCommandCode::CmdDataRunStop)
        }
      }
    }
  }
}

/// Function that manages ping commands from ground
pub fn send_ping(resp_socket_opt: Option<Socket>,
                 outgoing: Sender<TofPacket>,
                 tof_component: TofComponent,
                 id: u8)
                 -> Result<TofCommandCode, CmdError> {
  let payload: u32 = PAD_CMD_32BIT | (tof_component as u32) << 8 | (id as u32);
  let ping = TofCommand::Ping(payload);

  let tp = TofPacket::from(&ping);
  let tp_c: TofPacket = tp.clone();
  
  match outgoing.send(tp) {
    Err(err) => {
      error!("Unable to send command, error{err}");
      return Err(CmdError::PowerError);
    },
    Ok(_)    => {
      info!("Ping command sent")
    }
  }
  
  match resp_socket_opt {
    None => Ok(TofCommandCode::CmdPower),
    Some(resp_socket) => {
      match resp_socket.send(tp_c.to_bytestream(), 0) {
        Err(err) => {
          error!("Unable to reply to command, error{err}");
          return Err(CmdError::PingError);
        },
        Ok(_)    => {
          info!("Replied to ping command");
          return Ok(TofCommandCode::CmdPing)
        }
      }
    }
  }
}

/// Function that just replies to a ping command send to tofcpu
pub fn send_ping_response(resp_socket_opt: Option<Socket>)
                          -> Result<TofCommandCode, CmdError> {
  let mut tp = TofPacket::new();
  tp.packet_type = PacketType::Ping;
  tp.payload = vec![TofComponent::TofCpu as u8, 0u8];
  
  match resp_socket_opt {
    None => Ok(TofCommandCode::CmdPower),
    Some(resp_socket) => {
      match resp_socket.send(tp.to_bytestream(), 0) {
        Err(err) => {
          error!("Unable to reply to command, error{err}");
          return Err(CmdError::PingError);
        },
        Ok(_)    => {
          info!("Replied to ping command");
          return Ok(TofCommandCode::CmdPing)
        }
      }
    }
  }
}

/// Function that manages moni commands from ground
pub fn send_moni(resp_socket_opt: Option<Socket>,
                 outgoing: Sender<TofPacket>,
                 tof_component: TofComponent,
                 id: u8)
                 -> Result<TofCommandCode, CmdError> {
  let payload: u32 = PAD_CMD_32BIT | (tof_component as u32) << 8 | (id as u32);
  let moni = TofCommand::Moni(payload);

  let tp = TofPacket::from(&moni);
  let tp_c: TofPacket = tp.clone();
  
  match outgoing.send(tp) {
    Err(err) => {
      error!("Unable to send command, error{err}");
      return Err(CmdError::PowerError);
    },
    Ok(_)    => {
      info!("Moni command sent")
    }
  }
  
  match resp_socket_opt {
    None => Ok(TofCommandCode::CmdPower),
    Some(resp_socket) => {
      match resp_socket.send(tp_c.to_bytestream(), 0) {
        Err(err) => {
          error!("Unable to reply to command, error{err}");
          return Err(CmdError::MoniError);
        },
        Ok(_)    => {
          info!("Replied to moni command");
          return Ok(TofCommandCode::CmdMoni)
        }
      }
    }
  }
}

/// Function that just replies to a moni command send to tofcpu
pub fn send_moni_response(resp_socket_opt: Option<Socket>)
                          -> Result<TofCommandCode, CmdError> {
  let mut tp = TofPacket::new();
  tp.packet_type = PacketType::CPUMoniData;
  tp.payload = vec![TofComponent::TofCpu as u8, 0u8];
  
  match resp_socket_opt {
    None => Ok(TofCommandCode::CmdPower),
    Some(resp_socket) => {
      match resp_socket.send(tp.to_bytestream(), 0) {
        Err(err) => {
          error!("Unable to reply to command, error{err}");
          return Err(CmdError::MoniError);
        },
        Ok(_)    => {
          info!("Replied to moni command");
          return Ok(TofCommandCode::CmdMoni)
        }
      }
    }
  }
}

/// Function that send a restart command to RBs
pub fn send_systemd_reboot(resp_socket_opt: Option<Socket>,
                  outgoing: Sender<TofPacket>,
                           id: u8)
                           -> Result<TofCommandCode, CmdError> {
  let payload: u32 = PAD_CMD_32BIT | (id as u32);
  let systemd_reboot = TofCommand::SystemdReboot(payload);

  let tp = TofPacket::from(&systemd_reboot);
  let tp_c: TofPacket = tp.clone();
  
  match outgoing.send(tp) {
    Err(err) => {
      error!("Unable to send command, error{err}");
      return Err(CmdError::PowerError);
    },
    Ok(_)    => {
      info!("Systemd reboot command sent")
    }
  }
  
  match resp_socket_opt {
    None => Ok(TofCommandCode::CmdPower),
    Some(resp_socket) => {
      match resp_socket.send(tp_c.to_bytestream(), 0) {
        Err(err) => {
          error!("Unable to reply to command, error{err}");
          return Err(CmdError::SystemdRebootError);
        },
        Ok(_)    => {
          info!("Replied to systemd reboot command");
          return Ok(TofCommandCode::CmdSystemdReboot)
        }
      }
    }
  }
}

pub fn prefix_tof_cpu(input : &mut Vec<u8>) -> Vec<u8> {
  let mut bytestream : Vec::<u8>;
  let tof_cpu : String;
  tof_cpu = String::from("TOFCPU");
  //let mut response = 
  bytestream = tof_cpu.as_bytes().to_vec();
  //bytestream.append(&mut resp.to_bytestream());
  bytestream.append(input);
  bytestream
}
