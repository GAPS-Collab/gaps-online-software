use constants::{DEFAULT_CALIB_VOLTAGE, DEFAULT_CALIB_RB, DEFAULT_CALIB_EXTRA};
use crossbeam_channel::Sender;
use tof_dataclasses::packets::TofPacket;
use tof_dataclasses::commands::TofCommand;
use tof_dataclasses::constants::{MASK_CMD_8BIT,
                                 MASK_CMD_16BIT,
                                 MASK_CMD_24BIT,
                                 MASK_CMD_32BIT,
                                 PAD_CMD_32BIT};

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
pub fn send_all_calibration(cmd_sender: Sender<TofPacket>) {
  send_voltage_calibration(cmd_sender.clone(), DEFAULT_CALIB_VOLTAGE, DEFAULT_CALIB_RB, DEFAULT_CALIB_EXTRA);
  send_voltage_calibration(cmd_sender.clone(), DEFAULT_CALIB_VOLTAGE, DEFAULT_CALIB_RB, DEFAULT_CALIB_EXTRA);
  send_timing_calibration(cmd_sender, DEFAULT_CALIB_RB, DEFAULT_CALIB_EXTRA)
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
  let timing_calib = TofCommand::TimingCalibration(payload);
  let tp = TofPacket::from(&timing_calib);
  match cmd_sender.send(tp) {
    Err(err) => error!("Unable to send command, error{err}"),
    Ok(_)    => ()
  }
}

/// Function that starts timing calibration on a specific
/// RB with the specified voltage level and extras (not
/// implemented)
pub fn send_timing_calibration(cmd_sender: Sender<TofPacket>,
                               rb_id: u8,
                               extra: u8) {
  let payload: u32 = PAD_CMD_32BIT | (rb_id as u32) << 8 | (extra as u32);
  let timing_calib = TofCommand::TimingCalibration(payload);
  let tp = TofPacket::from(&timing_calib);
  match cmd_sender.send(tp) {
    Err(err) => error!("Unable to send command, error{err}"),
    Ok(_)    => ()
  }
}