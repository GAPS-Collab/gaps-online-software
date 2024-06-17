/// TOF CPU related monitoring tasks 
///
/// Monitors environmental data from the 
/// MTB as well as the TOF CPU

use std::time::{
    Instant,
    Duration
};
use std::thread::sleep;
use std::sync::{
    Arc,
    Mutex
};

use crossbeam_channel::Sender;
use tof_dataclasses::monitoring::{
    CPUMoniData,
};
use tof_dataclasses::packets::TofPacket;
//use tof_dataclasses::threading::ThreadControl;

use tof_control::helper::cpu_type::{
    CPUTempDebug,
    CPUInfoDebug,
};
use liftof_lib::thread_control::ThreadControl;


/// Monitor the main tof computer (sysinfo)
///
/// Get cpu usage, disk usage and temperature
/// information for the main TOF CPU
///
/// Thread to be used with liftof-cc and friends
///
/// # Arguments
///
/// * thread control - start/stop/halt/revive thread
///                    externally
/// * verbose        - print monitoring information 
///                    to the terminal
pub fn monitor_cpu(tp_sender      : Sender<TofPacket>,
                   moni_interval  : u64,
                   thread_control : Arc<Mutex<ThreadControl>>,
                   verbose        : bool) {
  let mut moni_data = CPUMoniData::new();
  let mut timer     = Instant::now();
  let sleep_time    = Duration::from_secs(moni_interval);
  'main: loop {
    let cpu_info    = CPUInfoDebug::new();
    let cpu_temp    = CPUTempDebug::new();
    if timer.elapsed().as_secs() >= moni_interval {
      moni_data.add_temps(&cpu_temp);
      moni_data.add_info(&cpu_info);
      let tp = TofPacket::from(&moni_data);
      match tp_sender.send(tp) {
        Err(err) => error!("Can't send CPUMoniData over channel1 {err}"),
        Ok(_)    => ()
      }
      timer = Instant::now();
      if verbose {
        println!("{}", moni_data);
      }
    }
    sleep(sleep_time);
    // FIXME - technically we should look for the 
    // stop signal on a shorter timescale.
    // But this saves CPU cycles
    match thread_control.try_lock() {
      Err(err) => error!("Unable to lock shared memory! {err}"),
      Ok(tc)   => {
        //println!("== ==> [monitoring] tc lock ackquired!");
        if tc.stop_flag {
          println!("==> Stopping monitoring thread, stop signal received!");
          break 'main;
        }
      }
    }
  }
}
