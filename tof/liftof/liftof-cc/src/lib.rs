use crossbeam_channel::Sender;
use liftof_lib::{PowerStatusEnum, TofComponent};
use tof_dataclasses::packets::TofPacket;
use tof_dataclasses::commands::TofCommand;
use tof_dataclasses::constants::PAD_CMD_32BIT;

pub mod constants;
pub mod api;
pub mod flight_comms;
pub mod threads;
#[macro_use] extern crate log;
extern crate clap;
extern crate json;
extern crate colored;

extern crate local_ip_address;
extern crate crossbeam_channel;
extern crate liftof_lib;

#[cfg(feature="random")]
extern crate rand;

extern crate zmq;
extern crate tof_dataclasses;

/// Power function that targets the component specified, no ID
pub fn send_power(cmd_sender: Sender<TofPacket>,
                  component: TofComponent,
                  power_status: PowerStatusEnum) {
  // no ID in the payload
  let payload: u32 = PAD_CMD_32BIT | (component as u32) << 24 | (power_status as u32);
  let power = match power_status {
    PowerStatusEnum::OFF => TofCommand::PowerOff(payload),
    PowerStatusEnum::ON => TofCommand::PowerOn(payload),
    PowerStatusEnum::Cycle => TofCommand::PowerCycle(payload)
  };
  
  let tp = TofPacket::from(&power);
  match cmd_sender.send(tp) {
    Err(err) => error!("Unable to send command, error{err}"),
    Ok(_)    => ()
  }
}

/// Power function that targets the component specified with supplied ID
pub fn send_power_ID(cmd_sender: Sender<TofPacket>,
                     component: TofComponent,
                     power_status: PowerStatusEnum,
                     component_id: u8) {
  let payload: u32 = PAD_CMD_32BIT | (component as u32) << 24 | (component_id as u32) << 16 | (power_status as u32);
  let power_id = match power_status {
    PowerStatusEnum::OFF => TofCommand::PowerOff(payload),
    PowerStatusEnum::ON => TofCommand::PowerOn(payload),
    PowerStatusEnum::Cycle => TofCommand::PowerCycle(payload)
  };
  
  let tp = TofPacket::from(&power_id);
  match cmd_sender.send(tp) {
    Err(err) => error!("Unable to send command, error{err}"),
    Ok(_)    => ()
  }
}

/// Power function that targets the component specified with supplied ID
pub fn send_power_preamp(cmd_sender: Sender<TofPacket>,
                         power_status: PowerStatusEnum,
                         preamp_id: u8,
                         preamp_bias: u16) {
  // bias only if ON and Cycle
  let payload: u32 = match power_status {
    PowerStatusEnum::OFF => 
      (TofComponent::Preamp as u32) << 16 | (preamp_id as u32) << 8 | 0u32,
    PowerStatusEnum::ON => 
      (TofComponent::Preamp as u32) << 16 | (preamp_id as u32) << 8 | preamp_bias as u32,
    PowerStatusEnum::Cycle => 
      (TofComponent::Preamp as u32) << 16 | (preamp_id as u32) << 8 | preamp_bias as u32
  };
  let power_preamp = match power_status {
    PowerStatusEnum::OFF => TofCommand::PowerOff(payload),
    PowerStatusEnum::ON => TofCommand::PowerOn(payload),
    PowerStatusEnum::Cycle => TofCommand::PowerCycle(payload)
  };
  
  let tp = TofPacket::from(&power_preamp);
  match cmd_sender.send(tp) {
    Err(err) => error!("Unable to send command, error{err}"),
    Ok(_)    => ()
  }
}

/// Default function that starts calibration on all RBs
/// with default values.
pub fn send_default_calibration(cmd_sender: Sender<TofPacket>,
                                voltage_level: u16,
                                rb_id: u8,
                                extra: u8) {
  let payload: u32
    = (voltage_level as u32) << 16 | (rb_id as u32) << 8 | (extra as u32);
  let default_calib = TofCommand::DefaultCalibration(payload);
  let tp = TofPacket::from(&default_calib);
  match cmd_sender.send(tp) {
    Err(err) => error!("Unable to send command, error{err}"),
    Ok(_)    => ()
  }
}

/// Function that starts voltage calibration on a specific
/// RB with the specified voltage level and extras (not
/// implemented)
pub fn send_noi_calibration(cmd_sender: Sender<TofPacket>,
                            rb_id: u8,
                            extra: u8) {
  let payload: u32 = PAD_CMD_32BIT | (rb_id as u32) << 8 | (extra as u32);
  let noi_calib = TofCommand::NoiCalibration(payload);
  let tp = TofPacket::from(&noi_calib);
  match cmd_sender.send(tp) {
    Err(err) => error!("Unable to send command, error{err}"),
    Ok(_)    => ()
  }
}

/// Function that starts voltage calibration on a specific
/// RB with the specified voltage level and extras (not
/// implemented)
pub fn send_voltage_calibration(cmd_sender: Sender<TofPacket>,
                                voltage_level: u16,
                                rb_id: u8,
                                extra: u8) {
  let payload: u32
    = (voltage_level as u32) << 16 | (rb_id as u32) << 8 | (extra as u32);
  let voltage_calib = TofCommand::VoltageCalibration(payload);
  let tp = TofPacket::from(&voltage_calib);
  match cmd_sender.send(tp) {
    Err(err) => error!("Unable to send command, error{err}"),
    Ok(_)    => ()
  }
}

/// Function that starts timing calibration on a specific
/// RB with the specified voltage level and extras (not
/// implemented)
pub fn send_timing_calibration(cmd_sender: Sender<TofPacket>,
                               voltage_level: u16,
                               rb_id: u8,
                               extra: u8) {
  let payload: u32
    = (voltage_level as u32) << 16 | (rb_id as u32) << 8 | (extra as u32);
  let timing_calib = TofCommand::TimingCalibration(payload);
  let tp = TofPacket::from(&timing_calib);
  match cmd_sender.send(tp) {
    Err(err) => error!("Unable to send command, error{err}"),
    Ok(_)    => ()
  }
}

/// Default function that starts run data taking on all RBs
/// with default values.
pub fn send_run_start(cmd_sender: Sender<TofPacket>,
                run_type: u8,
                rb_id: u8,
                event_no: u8,
                time: u8) {
  let payload: u32
  = (run_type as u32) << 24 | (rb_id as u32) << 16 | (event_no as u32) << 8 | (time as u32);
  let run_start = TofCommand::DataRunStart(payload);
  let tp = TofPacket::from(&run_start);
  match cmd_sender.send(tp) {
    Err(err) => error!("Unable to send command, error{err}"),
    Ok(_)    => ()
  }
}

/// Default function that starts run data taking on all RBs
/// with default values.
pub fn send_run_stop(cmd_sender: Sender<TofPacket>,
                     rb_id: u8) {
  let payload: u32 = PAD_CMD_32BIT | (rb_id as u32);
  let run_stop = TofCommand::DataRunStop(payload);
  let tp = TofPacket::from(&run_stop);
  match cmd_sender.send(tp) {
    Err(err) => error!("Unable to send command, error{err}"),
    Ok(_)    => ()
  }
}