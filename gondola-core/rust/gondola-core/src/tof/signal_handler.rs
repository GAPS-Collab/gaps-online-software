// This file is part of gaps-online-software and published 
// under the GPLv3 license

use std::thread;
use std::sync::{
  Arc,
  Mutex
};
use colored::{
    Colorize,
    //ColoredString
};

use signal_hook::iterator::Signals;
use signal_hook::consts::signal::{
  SIGTERM,
  SIGINT
};
use std::os::raw::c_int;
use std::time::Duration;
use crate::tof::ThreadControl;


/// Handle incoming POSIX signals and inform threads about 
/// the state.
///
/// Allows to terminate multithreaded application safevly 
/// when CTRL+C is pressed
pub fn signal_handler(thread_control     : Arc<Mutex<ThreadControl>>) {
  let sleep_time = Duration::from_millis(300);
  let mut signals = Signals::new(&[SIGTERM, SIGINT]).expect("Unknown signals");
  'main: loop {
    thread::sleep(sleep_time);

    // check pending signals and handle
    // SIGTERM and SIGINT
    for signal in signals.pending() {
      match signal as c_int {
        SIGTERM | SIGINT => {
          println!("=> {}", String::from("SIGTERM or SIGINT received. Maybe Ctrl+C has been pressed! Commencing program shutdown!").red().bold());
          match thread_control.lock() {
            Ok(mut tc) => {
              tc.sigint_recvd = true;
            }
            Err(err) => {
              error!("Can't acquire lock for ThreadControl! {err}");
            },
          }
          break 'main; // now end myself
        } 
        _ => {
          error!("Received signal, but I don't have instructions what to do about it!");
        }
      }
    }
  }
}
