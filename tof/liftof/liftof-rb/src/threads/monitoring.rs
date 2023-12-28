use std::time::{
    Duration,
    Instant
};
use std::sync::{
    Arc,
    Mutex,
};

use std::thread;

use crossbeam_channel::Sender;

use tof_dataclasses::monitoring::RBMoniData;
use tof_dataclasses::packets::TofPacket;
use tof_dataclasses::threading::ThreadControl;

// Takeru's tof-control code
#[cfg(feature = "tof-control")]
use tof_control::helper::rb_type::{
    RBTemp,
    RBMag,
    RBVcp,
    RBPh,
};


use crate::control::{get_board_id,
                     get_trigger_rate};


/// Gather monitoring data and pass it on over a channel
///
/// # Arguments:
///
/// * ch                -  should connect to a data sync
/// * moni_interval_l1  -  rate whith which we take L1 moni
///                        data (mission critical)
/// * moni_interval_l2  -  rate for L2 (slow) moni data 
///                        (everything)
/// * verbose           -  print additional output for debugging
pub fn monitoring(ch               : &Sender<TofPacket>,
                  moni_interval_l1 : Duration,
                  moni_interval_l2 : Duration,
                  verbose          : bool,
                  thread_control   : Arc<Mutex<ThreadControl>>) {
 
  let board_id           = get_board_id().unwrap_or(0); 
  let mut moni_timer_l1  = Instant::now();
  let mut moni_timer_l2  = Instant::now();
  loop {
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

    if moni_timer_l1.elapsed() > moni_interval_l1 {
      error!("L1 monitoring not implemented yet!");
      moni_timer_l1 = Instant::now();
    }
    if moni_timer_l2.elapsed() > moni_interval_l2 {
      // get tof-control data
      let mut moni_dt = RBMoniData::new();
      moni_dt.board_id = board_id as u8; 
      cfg_if::cfg_if! {
        if #[cfg(feature = "tofcontrol")]  {
          let rb_temp = RBTemp::new();
          let rb_mag  = RBMag::new();
          let rb_vcp  = RBVcp::new();
          let rb_ph   = RBPh::new();
          moni_dt.add_rbtemp(&rb_temp);
          moni_dt.add_rbmag(&rb_mag);
          moni_dt.add_rbvcp(&rb_vcp);
          moni_dt.add_rbph(&rb_ph);
        }
      }
      
      let rate_query = get_trigger_rate();
      match rate_query {
        Ok(rate) => {
          debug!("Monitoring thread -> Rate: {rate}Hz ");
          moni_dt.rate = rate as u16;
        },
        Err(_)   => {
          warn!("Can not send rate monitoring packet, register problem");
        }
      }
   
      if verbose {
        println!("{}", moni_dt);
      }
      let tp = TofPacket::from(&moni_dt);
      match ch.try_send(tp) {
        Err(err) => {error!("Issue sending RBMoniData {:?}", err)},
        Ok(_)    => {debug!("Send RBMoniData successfully!")}
      }
      moni_timer_l2 = Instant::now(); 
    }
    thread::sleep(moni_interval_l1);
  }
}

