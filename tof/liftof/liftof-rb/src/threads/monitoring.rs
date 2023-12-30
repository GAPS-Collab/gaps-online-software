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

use tof_dataclasses::monitoring::{
    RBMoniData,
    PAMoniData,
    PBMoniData,
    LTBMoniData,
};
use tof_dataclasses::packets::TofPacket;
use tof_dataclasses::threading::ThreadControl;

// Takeru's tof-control code
#[cfg(feature = "tof-control")]
use tof_control::helper::pb_type::{
    PBTemp,
    PBVcp,
};

#[cfg(feature = "tof-control")]
use tof_control::helper::preamp_type::{
    PreampTemp,
    PreampReadBias,
};

#[cfg(feature = "tof-control")]
use tof_control::helper::ltb_type::{
    LTBTemp,
    LTBThreshold,
};

#[cfg(feature = "tof-control")]
use tof_control::helper::rb_type::{
    RBTempDebug,
    RBMag,
    RBVcp,
    RBPh,
};

use crate::control::{get_board_id,
                     get_trigger_rate};


/// Gather monitoring data and pass it on over a channel
///
/// The readout of all other than the RB sensors itself
/// (RBMoniData) is locked to the readout of the RB sensors.
/// The readout interval of the other sensors has to be 
/// specified in numbers of rb_moni_interval.
///
/// # Arguments:
///
/// * board_id          -  this RB's ID. It will be used
///                        for all monitoring data as 
///                        identifiier.
/// * tp_sender         -  the resulting moni data will be 
///                        wrapped in TofPackets. Use
///                        `tp_sender` to send them to 
///                        their destination
/// * rb_moni_interval  -  Number of seconds between 2 consecutive
///                        polls of RBMoniData. 
///                        Set to 0 to disable monitoring.
/// * pa_moni_every_x   -  Get PA (preamp) moni data every x polls of RBMoniData
///                        Set to 0 to disable monitoring.
/// * pb_moni_every_x   -  Get PB (power board) moni data every x polls of RBMoniData
///                        Set to 0 to disable monitoring.
/// * ltb_moni_every_x  -  Get LTB moni data every x polls of RBMoniData
///                        Set to 0 to disable monitoring.
/// * verbose           -  print additional output for debugging
/// * thread_control    -  central thread control, e.g. kill signal
pub fn monitoring(board_id          : u8,
                  tp_sender         : &Sender<TofPacket>,
                  rb_moni_interval  : f32,
                  pa_moni_every_x   : f32,
                  pb_moni_every_x   : f32,
                  ltb_moni_every_x  : f32,
                  verbose           : bool,
                  thread_control    : Arc<Mutex<ThreadControl>>) {

  println!("[MONI] ==> Starting monitoring thread!");

  let mut rb_moni_timer  = Instant::now();
  let mut pa_moni_timer  = Instant::now();
  let mut pb_moni_timer  = Instant::now();
  let mut ltb_moni_timer = Instant::now();
 
  // we calculate some sleep time, to reduce CPU load
  // check for the smallest interfval and use that as sleep.
  let mut sleeptime_sec = rb_moni_interval;
  if pa_moni_every_x*rb_moni_interval < sleeptime_sec {
    sleeptime_sec = pa_moni_every_x*rb_moni_interval;
  }
  if pb_moni_every_x*rb_moni_interval < sleeptime_sec {
    sleeptime_sec = pb_moni_every_x*rb_moni_interval;
  }
  if ltb_moni_every_x*rb_moni_interval < sleeptime_sec {
    sleeptime_sec = ltb_moni_every_x*rb_moni_interval;
  }
  let sleeptime = Duration::from_secs_f32(sleeptime_sec);

  loop {
    match thread_control.lock() {
      Ok(tc) => {
        if tc.stop_flag {
          println!("[MONI] ==> Received STOP signal. Will end thread!");
          info!("Received stop signal. Will stop thread!");
          break;
        }
      },
      Err(err) => {
        trace!("Can't acquire lock! {err}");
      },
    }

    if rb_moni_timer.elapsed().as_secs_f32() > rb_moni_interval {
      // get tof-control data
      let mut moni_dt = RBMoniData::new();
      moni_dt.board_id = board_id; 
      cfg_if::cfg_if! {
        if #[cfg(feature = "tofcontrol")]  {
          let rb_temp = RBTempDebug::new();
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
      match tp_sender.try_send(tp) {
        Err(err) => error!("Issue sending RBMoniData {:?}", err),
        Ok(_)    => trace!("Sent RBMoniData successfully!"),
      }
      rb_moni_timer = Instant::now(); 
    }
    if pa_moni_timer.elapsed().as_secs_f32() > rb_moni_interval*pa_moni_every_x {
      let mut moni = PAMoniData::new();
      moni.board_id = board_id;
      cfg_if::cfg_if! {
        if #[cfg(feature = "tofcontrol")]  {
          // FIXME - this won't fail, however, if there
          // is an issue it will silently set all values
          // to f32::MAX
          let pa_tmp = PreampTemp::new();
          let pa_bia = PreampReadBias::new();
          moni.add_temps(&pa_tmp);
          moni.add_biases(&pa_bia);
        }
      }
      if verbose {
        println!("{}", moni);
      }
      let tp = TofPacket::from(&moni);
      match tp_sender.try_send(tp) {
        Err(err) => error!("Issue sending PAMoniData {:?}", err),
        Ok(_)    => trace!("Sent PAMoniData successfully!"),
      }
    }
    if pb_moni_timer.elapsed().as_secs_f32() > rb_moni_interval*pb_moni_every_x {
      let mut moni = PBMoniData::new();
      moni.board_id = board_id;
      cfg_if::cfg_if! {
        if #[cfg(feature = "tofcontrol")]  {
          let pb_temp = PBTemp::new();
          let pb_vcp  = PBVcp::new();
          moni.add_temps(&pb_temp);
          moni.add_vcp(&pb_vcp);
        }
      }
      if verbose {
        println!("{}", moni);
      }
      let tp = TofPacket::from(&moni);
      match tp_sender.try_send(tp) {
        Err(err) => error!("Issue sending PBMoniData {:?}", err),
        Ok(_)    => trace!("Sent PBMoniData successfully!"),
      }
    }
    if ltb_moni_timer.elapsed().as_secs_f32() > rb_moni_interval*ltb_moni_every_x {
      let mut moni = LTBMoniData::new();
      moni.board_id = board_id;
      cfg_if::cfg_if! {
        if #[cfg(feature = "tofcontrol")]  {
          let ltb_temp = LTBTemp::new();
          let ltb_thrs = LTBThreshold::new();
        }
      }
      if verbose {
        println!("{}", moni);
      }
      let tp = TofPacket::from(&moni);
      match tp_sender.try_send(tp) {
        Err(err) => error!("Issue sending LTBMoniData {:?}", err),
        Ok(_)    => debug!("Sent LTBMoniData successfully!"),
      }
    }

    thread::sleep(sleeptime);
  }
}

