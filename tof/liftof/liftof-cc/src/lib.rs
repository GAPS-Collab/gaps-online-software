use crossbeam_channel::Sender;
use tof_dataclasses::packets::TofPacket;
use tof_dataclasses::commands::TofCommand;
use tof_dataclasses::constants::{PAD_CMD_32BIT};

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
  let default_calib = TofCommand::DataRunStart(payload);
  let tp = TofPacket::from(&default_calib);
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
  let default_calib = TofCommand::DataRunStop(payload);
  let tp = TofPacket::from(&default_calib);
  match cmd_sender.send(tp) {
    Err(err) => error!("Unable to send command, error{err}"),
    Ok(_)    => ()
  }
}