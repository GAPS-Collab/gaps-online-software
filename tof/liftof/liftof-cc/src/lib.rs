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
//use zmq::Socket;

#[macro_use] extern crate log;
extern crate clap;
extern crate colored;

extern crate local_ip_address;
extern crate crossbeam_channel;
extern crate liftof_lib;

extern crate zmq;
extern crate tof_dataclasses;
extern crate tof_control;

pub mod constants;
pub mod threads;
pub mod settings;

/// Power function that targets the component specified, no ID
pub fn send_power(cmd_sender: Sender<TofPacket>,
                  component: TofComponent,
                  power_status: PowerStatusEnum)
                  -> Result<TofCommandCode, CmdError> {
  // no ID in the payload
  let payload: u32 = PAD_CMD_32BIT | (component as u32) << 24 | (power_status as u32);
  let power = TofCommand::Power(payload);
  
  let tp = TofPacket::from(&power);
  match cmd_sender.send(tp) {
    Err(err) => {
      error!("Unable to send command, error{err}");
      return Err(CmdError::PowerError);
    },
    Ok(_)    => {
      info!("Power command sent to RBs");
      return Ok(TofCommandCode::CmdPower)
    }
  }
}

/// Power function that targets the component specified with supplied ID
pub fn send_power_id(cmd_sender: Sender<TofPacket>,
                     component: TofComponent,
                     power_status: PowerStatusEnum,
                     component_id: u8)
                     -> Result<TofCommandCode, CmdError> {
  let payload: u32 = PAD_CMD_32BIT | (component as u32) << 24 | (component_id as u32) << 16 | (power_status as u32);
  let power_id = TofCommand::Power(payload);
  
  let tp = TofPacket::from(&power_id);
  match cmd_sender.send(tp) {
    Err(err) => {
      error!("Unable to send command, error{err}");
      return Err(CmdError::PowerError);
    },
    Ok(_)    => {
      info!("Power command sent to component");
      return Ok(TofCommandCode::CmdPower)
    }
  }
}

/// Power function that targets the component specified with supplied ID
pub fn send_power_preamp(cmd_sender: Sender<TofPacket>,
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
  match cmd_sender.send(tp) {
    Err(err) => {
      error!("Unable to send command, error{err}");
      return Err(CmdError::PowerError);
    },
    Ok(_)    => {
      info!("Power command sent to preamps");
      return Ok(TofCommandCode::CmdPower)
    }
  }
}

/// Default function that starts calibration on all RBs
/// with default values.
pub fn send_default_calibration(cmd_sender: Sender<TofPacket>,
                                voltage_level: u16,
                                rb_id: u8,
                                extra: u8)
                                -> Result<TofCommandCode, CmdError> {
  let payload: u32
    = (voltage_level as u32) << 16 | (rb_id as u32) << 8 | (extra as u32);
  let default_calib = TofCommand::DefaultCalibration(payload);

  let tp = TofPacket::from(&default_calib);
  match cmd_sender.send(tp) {
    Err(err) => {
      error!("Unable to send command, error{err}");
      return Err(CmdError::CalibrationError);
    },
    Ok(_)    => {
      info!("Calibration command sent");
      return Ok(TofCommandCode::CmdDefaultCalibration)
    }
  }
}

/// Function that starts voltage calibration on a specific
/// RB with the specified voltage level and extras (not
/// implemented)
pub fn send_noi_calibration(cmd_sender: Sender<TofPacket>,
                            rb_id: u8,
                            extra: u8)
                            -> Result<TofCommandCode, CmdError> {
  let payload: u32 = PAD_CMD_32BIT | (rb_id as u32) << 8 | (extra as u32);
  let noi_calib = TofCommand::NoiCalibration(payload);

  let tp = TofPacket::from(&noi_calib);
  match cmd_sender.send(tp) {
    Err(err) => {
      error!("Unable to send command, error{err}");
      return Err(CmdError::CalibrationError);
    },
    Ok(_)    => {
      info!("Calibration command sent");
      return Ok(TofCommandCode::CmdNoiCalibration)
    }
  }
}

/// Function that starts voltage calibration on a specific
/// RB with the specified voltage level and extras (not
/// implemented)
pub fn send_voltage_calibration(cmd_sender: Sender<TofPacket>,
                                voltage_level: u16,
                                rb_id: u8,
                                extra: u8)
                                -> Result<TofCommandCode, CmdError> {
  let payload: u32
    = (voltage_level as u32) << 16 | (rb_id as u32) << 8 | (extra as u32);
  let voltage_calib = TofCommand::VoltageCalibration(payload);

  let tp = TofPacket::from(&voltage_calib);
  match cmd_sender.send(tp) {
    Err(err) => {
      error!("Unable to send command, error{err}");
      return Err(CmdError::CalibrationError);
    },
    Ok(_)    => {
      info!("Calibration command sent");
      return Ok(TofCommandCode::CmdVoltageCalibration)
    }
  }
}

/// Function that starts timing calibration on a specific
/// RB with the specified voltage level and extras (not
/// implemented)
pub fn send_timing_calibration(cmd_sender: Sender<TofPacket>,
                               voltage_level: u16,
                               rb_id: u8,
                               extra: u8)
                               -> Result<TofCommandCode, CmdError> {
  let payload: u32
    = (voltage_level as u32) << 16 | (rb_id as u32) << 8 | (extra as u32);
  let timing_calib = TofCommand::TimingCalibration(payload);

  let tp = TofPacket::from(&timing_calib);
  match cmd_sender.send(tp) {
    Err(err) => {
      error!("Unable to send command, error{err}");
      return Err(CmdError::CalibrationError);
    },
    Ok(_)    => {
      info!("Calibration command sent");
      return Ok(TofCommandCode::CmdTimingCalibration)
    }
  }
}

/// Function that sends the threshold to be set on all or
/// specific LTBs
pub fn send_ltb_threshold_set(cmd_sender: Sender<TofPacket>,
                              ltb_id: u8,
                              threshold_name: LTBThresholdName,
                              threshold_level: u16)
                              -> Result<TofCommandCode, CmdError> {
  let payload: u32
  = (ltb_id as u32) << 24 | (threshold_name as u32) << 16 | (threshold_level as u32);
  let ltb_threshold = TofCommand::SetThresholds(payload);

  let tp = TofPacket::from(&ltb_threshold);
  match cmd_sender.send(tp) {
    Err(err) => {
      error!("Unable to send command, error{err}");
      return Err(CmdError::ThresholdSetError);
    },
    Ok(_)    => {
      info!("Threshold set command sent");
      return Ok(TofCommandCode::CmdSetThresholds)
    }
  }
}

/// Function that sends the threshold to be set on all or
/// specific LTBs
pub fn send_preamp_bias_set(cmd_sender: Sender<TofPacket>,
                            preamp_id: u8,
                            preamp_bias: u16)
                            -> Result<TofCommandCode, CmdError> {
  let payload: u32
  = PAD_CMD_32BIT | (preamp_id as u32) << 16 | (preamp_bias as u32);
  let preamp_bias = TofCommand::SetPreampBias(payload);

  let tp = TofPacket::from(&preamp_bias);
  match cmd_sender.send(tp) {
    Err(err) => {
      error!("Unable to send command, error{err}");
      return Err(CmdError::ThresholdSetError);
    },
    Ok(_)    => {
      info!("Preamp bias set command sent");
      return Ok(TofCommandCode::CmdSetPreampBias)
    }
  }
}

/// Default function that starts run data taking on all RBs
/// with default values.
pub fn send_run_start(cmd_sender: Sender<TofPacket>,
                run_type: u8,
                rb_id: u8,
                event_no: u8)
                -> Result<TofCommandCode, CmdError> {
  let payload: u32
  = PAD_CMD_32BIT | (run_type as u32) << 16 | (rb_id as u32) << 8 | (event_no as u32);
  let run_start = TofCommand::DataRunStart(payload);

  let tp = TofPacket::from(&run_start);
  match cmd_sender.send(tp) {
    Err(err) => {
      error!("Unable to send command, error{err}");
      return Err(CmdError::RunStartError);
    },
    Ok(_)    => {
      info!("Start run command sent");
      return Ok(TofCommandCode::CmdDataRunStart)
    }
  }
}

/// Default function that starts run data taking on all RBs
/// with default values.
pub fn send_run_stop(cmd_sender: Sender<TofPacket>,
                     rb_id: u8)
                     -> Result<TofCommandCode, CmdError> {
  let payload: u32 = PAD_CMD_32BIT | (rb_id as u32);
  let run_stop = TofCommand::DataRunStop(payload);

  let tp = TofPacket::from(&run_stop);
  match cmd_sender.send(tp) {
    Err(err) => {
      error!("Unable to send command, error{err}");
      return Err(CmdError::RunStopError);
    },
    Ok(_)    => {
      info!("Stop run command sent");
      return Ok(TofCommandCode::CmdDataRunStop)
    }
  }
}

/// Function that manages ping commands from ground
pub fn send_ping(cmd_sender: Sender<TofPacket>,
                 tof_component: TofComponent,
                 id: u8)
                 -> Result<TofCommandCode, CmdError> {
  let payload: u32 = PAD_CMD_32BIT | (tof_component as u32) << 8 | (id as u32);
  let ping = TofCommand::Ping(payload);

  let tp = TofPacket::from(&ping);
  match cmd_sender.send(tp) {
    Err(err) => {
      error!("Unable to send command, error{err}");
      return Err(CmdError::PingError);
    },
    Ok(_)    => {
      info!("Ping command sent");
      return Ok(TofCommandCode::CmdPing)
    }
  }
}

/// Function that just replies to a ping command send to tofcpu
pub fn send_ping_response(cmd_sender: Sender<TofPacket>)
                          -> Result<TofCommandCode, CmdError> {
  let mut tp = TofPacket::new();
  tp.packet_type = PacketType::Ping;
  tp.payload = vec![TofComponent::TofCpu as u8, 0u8];
  match cmd_sender.send(tp) {
    Err(err) => {
      error!("Unable to send command, error{err}");
      return Err(CmdError::PingError);
    },
    Ok(_)    => {
      info!("Responded to ping!");
      return Ok(TofCommandCode::CmdPing)
    }
  }
}

/// Function that manages moni commands from ground
pub fn send_moni(cmd_sender: Sender<TofPacket>,
                 tof_component: TofComponent,
                 id: u8)
                 -> Result<TofCommandCode, CmdError> {
  let payload: u32 = PAD_CMD_32BIT | (tof_component as u32) << 8 | (id as u32);
  let moni = TofCommand::Moni(payload);

  let tp = TofPacket::from(&moni);
  match cmd_sender.send(tp) {
    Err(err) => {
      error!("Unable to send command, error{err}");
      return Err(CmdError::MoniError);
    },
    Ok(_)    => {
      info!("Moni command sent");
      return Ok(TofCommandCode::CmdMoni)
    }
  }
}

/// Function that just replies to a moni command send to tofcpu
pub fn send_moni_response(cmd_sender: Sender<TofPacket>)
                          -> Result<TofCommandCode, CmdError> {
  let mut tp = TofPacket::new();
  tp.packet_type = PacketType::CPUMoniData;
  tp.payload = vec![TofComponent::TofCpu as u8, 0u8];
  match cmd_sender.send(tp) {
    Err(err) => {
      error!("Unable to send command, error{err}");
      return Err(CmdError::MoniError);
    },
    Ok(_)    => {
      info!("Responded to moni!");
      return Ok(TofCommandCode::CmdMoni)
    }
  }
}

/// Function that send a restart command to RBs
pub fn send_systemd_reboot(cmd_sender: Sender<TofPacket>,
                           id: u8)
                           -> Result<TofCommandCode, CmdError> {
  let payload: u32 = PAD_CMD_32BIT | (id as u32);
  let systemd_reboot = TofCommand::SystemdReboot(payload);

  let tp = TofPacket::from(&systemd_reboot);
  match cmd_sender.send(tp) {
    Err(err) => {
      error!("Unable to send command, error{err}");
      return Err(CmdError::SystemdRebootError);
    },
    Ok(_)    => {
      info!("Systemd reboot command sent");
      return Ok(TofCommandCode::CmdSystemdReboot)
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
