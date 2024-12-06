//! Thread for rb calibration, controlled with the 
//! global shared thread control struct

use std::sync::{
    Arc,
    Mutex,
};

use std::thread;
use std::time::Duration;

use crossbeam_channel::Sender;

use tof_dataclasses::packets::TofPacket;
use tof_dataclasses::run::RunConfig;
use tof_dataclasses::threading::ThreadControl;

use crate::api::rb_calibration;


/// Perform RB calibration
///
/// This performs the Stage0 calibration 
/// - Calibration constants to convert adc to milliVolts
/// - Calibration constants to convert timing bins to nanoseconds
///
/// # Arguments
///
pub fn calibration(rc_to_runner   : &Sender<RunConfig>,
                   tp_to_pub      : &Sender<TofPacket>,
                   local_address  : String,
                   save_cali_wf   : bool,
                   thread_control : Arc<Mutex<ThreadControl>>) { 
  let sleeptime = Duration::from_secs(1);
  let mut do_it = false;
  loop {
    thread::sleep(sleeptime);
    
    match thread_control.lock() {
      Ok(tc) => {
        if tc.stop_flag {
          info!("Received stop signal. Will stop calibration thread!");
          break;
        }
        do_it = tc.calibration_active;
      },
      Err(err) => {
        trace!("Can't acquire lock! {err}");
      }
    }
    if do_it {
      // trigger calibration routine
      // this will block
      rb_calibration(rc_to_runner, tp_to_pub, save_cali_wf, local_address.clone());
      match thread_control.lock() {
        Ok(mut tc) => {
          // disable calibration flag
          tc.calibration_active = false;
        },
        Err(err) => {
          trace!("Can't acquire lock! {err}");
        }
      }
    }
  }
}
