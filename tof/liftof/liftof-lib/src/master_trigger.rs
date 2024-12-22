//! MasterTriggerBoard communications
//!
//! The MTB (MasterTriggerBoard) is currently
//! (Jan 2023) connected to the ethernet 
//! via UDP sockets and sends out its 
//! own datapackets per each triggered 
//! event.
//!
//! The packet format contains the event id
//! as well as number of hits and a mask 
//! which encodes the hit channels.
//!
//! The data is encoded in IPBus packets.
//! [see docs here](https://ipbus.web.cern.ch/doc/user/html/)
//! 

pub mod control;
pub mod registers;

use std::sync::{
  Arc,
  Mutex,
};

use std::time::{
  Duration,
  Instant
};

use std::thread;
use crossbeam_channel::Sender;
use serde_json::json;

use tof_dataclasses::packets::TofPacket;
use tof_dataclasses::monitoring::MtbMoniData;
use tof_dataclasses::events::MasterTriggerEvent;
use tof_dataclasses::events::master_trigger::TriggerType;

use tof_dataclasses::errors::{
  MasterTriggerError
};

use tof_dataclasses::ipbus::{
  IPBus,
  //IPBusPacketType,
};

use tof_dataclasses::heartbeats::MTBHeartbeat;
use tof_dataclasses::serialization::Packable;

use crate::thread_control::ThreadControl;

// make this public to not brake liftof-cc
pub use crate::settings::MTBSettings;

use control::*;
use registers::*;

/// The DAQ packet from the MTB has a flexible size, but it will
/// be at least this number of words long.
const MTB_DAQ_PACKET_FIXED_N_WORDS : u32 = 11; 

/// helper function to parse output for TofBot
fn remove_from_word(s: String, word: &str) -> String {
  if let Some(index) = s.find(word) {
    // Keep everything up to the found index (not including the word itself)
    s[..index].to_string()
  } else {
    // If the word isn't found, return the original string
    s
  }
}

/// Read the complete event of the MTB
///
/// FIXME - this can get extended to read 
/// multiple events at once. 
/// For that, we just have to query the
/// event size register multiple times.
///
/// <div class="warning"> Blocki until a UDP timeout error occurs or a non-zero result for MT.EVENT_QUEUE.SIZE register has been obtained.</div>
///
/// # Arguments
///
/// * bus       : connected IPBus for UDP comms
pub fn get_event(bus                     : &mut IPBus)
  -> Result<MasterTriggerEvent, MasterTriggerError> {
  let mut mte = MasterTriggerEvent::new();
  let     n_daq_words : u32;
  loop {
    //thread::sleep(sleeptime);
    //if nevents == 0 {
    //  // In case there is no events in the queue, return 
    //  // immediatly
    //  return Err(MasterTriggerError::EventQueueEmpty);
    //}
    // 3 things can happen here:
    // - it returns an error. Then we end this call
    // - it returns 0 - no mt event is ready, we end 
    //   this call
    // - non 0 result -> next mt ready, continue
    // when we are reading, remember that only 
    // the first 16bits are the value we are 
    // interested in
    let nwords  = EVQ_SIZE.get(bus)?;
    if nwords != 0 {
      n_daq_words = nwords/2 + nwords % 2;
      //println!("Read {} from SIZE register", nwords);
      break;
    } 
  }
  
  let data = bus.read_multiple(0x11, n_daq_words as usize, false)?;  
  //println!("{}", data[0]);
  if data[0] != 0xAAAAAAAA {
    error!("Got MTB data, but the header is incorrect {:x}", data[0]);
    return Err(MasterTriggerError::PackageHeaderIncorrect);
  }
  let foot_pos = (n_daq_words - 1) as usize;
  if data.len() <= foot_pos {
    error!("Got MTB data, but the package ends too early!");
    return Err(MasterTriggerError::DataTooShort);
  }
  if data.len() > foot_pos + 1 {
    error!("The MTB event packets has {} fields, when {} are expected!", data.len(), n_daq_words);
  }
  if data[foot_pos] != 0x55555555 {
    error!("Got MTB data, but the footer is incorrect {:x}", data[foot_pos]);
    if data[foot_pos] == 0xAAAAAAAA {
      println!("Found next header, printing the whole package!");
      println!("N LTBs {} ({})", data[8].count_ones(), data[8]);
      for k in data {
        println!("-- {:x}", k);
      }
    }
    return Err(MasterTriggerError::PackageFooterIncorrect);
  }

  // Number of words which will be always there. 
  // Min event size is +1 word for hits
  let n_hit_words    = n_daq_words - MTB_DAQ_PACKET_FIXED_N_WORDS;
  //  this can happen when the subtraction above overflows
  if n_hit_words > n_daq_words {
    error!("N hit word calculation failed! Got {} hit words!", n_hit_words);
    return Err(MasterTriggerError::BrokenPackage);
  }
  //println!("We are expecting {}", n_hit_packets);
  mte.event_id       = data[1];
  mte.timestamp      = data[2];
  mte.tiu_timestamp  = data[3];
  mte.tiu_gps32      = data[4];
  mte.tiu_gps16      = (data[5] & 0x0000ffff) as u16;
  mte.trigger_source = ((data[5] & 0xffff0000) >> 16) as u16;
  //mte.get_trigger_sources();
  let rbmask = (data[7] as u64) << 32 | data[6] as u64; 
  mte.mtb_link_mask  = rbmask;
  mte.dsi_j_mask     = data[8];
  /*** NEW ***/
  // we try ;)
  let n_trig_boards  = data[8].count_ones();
  let n_hit_fields   : u32;
  let mut odd_boards = false;
  if n_trig_boards % 2 == 0 {
    n_hit_fields = n_trig_boards/2;
  } else {
    n_hit_fields = n_trig_boards/2 + 1;
    odd_boards   = true;
  }
  for k in 9..9 + n_hit_fields {
    let ltb_hits = data[k as usize];
    // split them up
    let first  =  (ltb_hits & 0x0000ffff) as u16;
    let second = ((ltb_hits & 0xffff0000) >> 16) as u16;
  //for k in 1..n_hit_words+1 {
  //  let first  = ( data[8 + k as usize] & 0x0000ffff) as u16;
  //  let second = ((data[8 + k as usize] & 0xffff0000) >> 16) as u16; 
    mte.channel_mask.push(first);
    if !odd_boards {
      mte.channel_mask.push(second);
    } else {
      if k != (9 + n_hit_fields - 1) {
        mte.channel_mask.push(second);
      }
    }
  }
  //println!("{:?}", data);
  //println!("{:?}", mte.channel_mask);
  Ok(mte)
}

/// Gather monitoring data from the Mtb
///
/// ISSUES - some values are always 0
pub fn get_mtbmonidata(bus : &mut IPBus) 
  -> Result<MtbMoniData, MasterTriggerError> {
  let mut moni = MtbMoniData::new();
  let data = bus.read_multiple(0x120, 4, true)?;
  if data.len() < 4 {
    return Err(MasterTriggerError::BrokenPackage);
  }
  let tiu_busy_len    = TIU_BUSY_LENGTH.get(bus)?;
  let tiu_aux_link    = (TIU_USE_AUX_LINK.get(bus)? != 0) as u8;
  let tiu_emu_mode    = (TIU_EMULATION_MODE.get(bus)? != 0) as u8;
  //let tiu_bad         = TIU_BAD.get(bus)? as u8;
  //let tiu_busy_stuck  = (TIU_BUSY_STUCK.get(bus)? != 0) as u8;
  //let tiu_link_bad    = TIU_BAD.get(bus)?;
  //let tiu_busy_ign    = (TIU_BUSY_IGNORE.get(bus)? != 0) as u8;
  let aggr_tiu        = TIU_LT_AND_RB_MULT.get(bus)?;
  let tiu_link_bad    = (aggr_tiu & 0x1) as u8;
  let tiu_busy_stuck  = ((aggr_tiu & 0x2) >> 1) as u8;
  let tiu_busy_ign    = ((aggr_tiu & 0x4) >> 2) as u8;
  let mut tiu_status  = 0u8;
  tiu_status          = tiu_status | (tiu_emu_mode);
  tiu_status          = tiu_status | (tiu_aux_link << 1);
  tiu_status          = tiu_status | ((tiu_link_bad as u8) << 2);
  tiu_status          = tiu_status | (tiu_busy_stuck << 3);
  tiu_status          = tiu_status | (tiu_busy_ign << 4);
  let daq_queue_len   = EVQ_NUM_EVENTS.get(bus)? as u16;
  moni.tiu_status     = tiu_status;
  moni.tiu_busy_len   = tiu_busy_len;
  moni.daq_queue_len  = daq_queue_len;
  // sensors are 12 bit
  let first_word     = 0x00000fff;
  let second_word    = 0x0fff0000;
  //println!("[get_mtbmonidata] => Received data from registers {:?} data", data);
  moni.temp          = ( data[2] & first_word  ) as u16;  
  moni.vccint        = ((data[2] & second_word ) >> 16) as u16;  
  moni.vccaux        = ( data[3] & first_word  ) as u16;  
  moni.vccbram       = ((data[3] & second_word ) >> 16) as u16;  
 
  let rate           = bus.read_multiple(0x17, 2, true)?;
  // FIXME - technically, the rate is 24bit, however, we just
  // read out 16 here (if the rate is beyond ~65kHz, we don't need 
  // to know with precision
  let mask           = 0x0000ffff;
  moni.rate          = (rate[0] & mask) as u16;
  moni.lost_rate     = (rate[1] & mask) as u16;
  let rb_lost_rate  = RB_LOST_TRIGGER_RATE.get(bus)?;
  if rb_lost_rate > 255 {
    moni.rb_lost_rate = 255;
  } else {
    moni.rb_lost_rate = rb_lost_rate as u8;
  }
  Ok(moni)
}

/// Configure the MTB according to lifot settings.
///
/// # Arguments:
///   * mt_address : udp address of the MTB
///   * settings   : configure the MTB according
///                  to these settings 
pub fn configure_mtb(mt_address : &str,
                     settings   : &MTBSettings) -> Result<(), MasterTriggerError> {
  let mut bus : IPBus;
  match IPBus::new(mt_address) {
    // if that doesn't work, then probably the 
    // configuration is wrong, wo we might as 
    // well panic
    Err(err) => {
      error!("Can't connect to MTB! {err}");
      return Err(MasterTriggerError::UdpTimeOut);
    }
    Ok(_bus) => {
      bus = _bus;
    }
  }

  // configure MTB here
  let trace_suppression = settings.trace_suppression;
  match set_trace_suppression(&mut bus, trace_suppression) {
    Err(err) => error!("Unable to set trace suppression mode! {err}"),
    Ok(_)    => {
      if trace_suppression {
        println!("==> Setting MTB to trace suppression mode!");
      } else {
        println!("==> Setting MTB to ALL_RB_READOUT mode!");
        warn!("Reading out all events from all RBs! Data might be very large!");
      }
    }
  }

  let tiu_ignore_busy    = settings.tiu_ignore_busy;
  match TIU_BUSY_IGNORE.set(&mut bus, tiu_ignore_busy as u32) {
    Err(err) => error!("Unable to change tiu busy ignore settint! {err}"),
    Ok(_)    => {
      if tiu_ignore_busy {
        warn!("Ignoring TIU since tiu_busy_ignore is set in the config file!");
        println!("==> Ignroing TIU since tiu_busy_ignore is set in the config file!");
      }
    }
  }

  // disable broken emulation mode!!
  //let tiu_emulation_mode = settings.tiu_emulation_mode;
  //match set_tiu_emulation_mode(&mut bus, tiu_emulation_mode) {
  //  Err(err) => error!("Unable to change tiu emulation mode! {err}"),
  //  Ok(_) => {
  //    if tiu_emulation_mode {
  //      println!("==> Setting TIU emulation mode! This will emulate a TIU. However, this is (usually) not a good run setting for taking data together with the tracker!");
  //    } else {
  //      println!("==> Not setting TIU emulation mode! Good setting for combined runs with tracker! \u{1F4AF}");
  //    }
  //  }
  //}
  
  info!("Settting rb integration window!");
  let int_wind = settings.rb_int_window;
  match set_rb_int_window(&mut bus, int_wind) {
    Err(err) => error!("Unable to set rb integration window! {err}"),
    Ok(_)    => {
      info!("rb integration window set to {}", int_wind); 
    } 
  }

  debug!("Resetting master trigger DAQ");
  match reset_daq(&mut bus) {//, &mt_address) {
    Err(err) => error!("Can not reset DAQ, error {err}"),
    Ok(_)    => ()
  }
  
  match unset_all_triggers(&mut bus) {
    Err(err) => error!("Unable to undo previous trigger settings! {err}"),
    Ok(_)    => ()
  }
  match settings.trigger_type {
    TriggerType::Poisson => {
      match set_poisson_trigger(&mut bus,settings.poisson_trigger_rate) {
        Err(err) => error!("Unable to set the POISSON trigger! {err}"),
        Ok(_)    => ()
      }
    }
    TriggerType::Any     => {
      match set_any_trigger(&mut bus,settings.trigger_prescale) {
        Err(err) => error!("Unable to set the ANY trigger! {err}"),
        Ok(_)    => ()
      }
    }
    TriggerType::Track   => {
      match set_track_trigger(&mut bus, settings.trigger_prescale) {
        Err(err) => error!("Unable to set the TRACK trigger! {err}"),
        Ok(_)    => ()
      }
    }
    TriggerType::TrackCentral   => {
      match set_central_track_trigger(&mut bus, settings.trigger_prescale) {
        Err(err) => error!("Unable to set the CENTRAL TRACK trigger! {err}"),
        Ok(_)    => ()
      }
    }
    TriggerType::TrackUmbCentral  => {
      match set_track_umb_central_trigger(&mut bus, settings.trigger_prescale) {
        Err(err) => error!("Unable to set the TRACK UMB CENTRAL trigger! {err}"),
        Ok(_)   => ()
      }
    }
    TriggerType::Gaps    => {
      match set_gaps_trigger(&mut bus, settings.gaps_trigger_use_beta) {
        Err(err) => error!("Unable to set the GAPS trigger! {err}"),
        Ok(_)    => ()
      }
    }
    TriggerType::Gaps633    => {
      match set_gaps633_trigger(&mut bus, settings.gaps_trigger_use_beta) {
        Err(err) => error!("Unable to set the GAPS trigger! {err}"),
        Ok(_)    => ()
      }
    }
    TriggerType::Gaps422    => {
      match set_gaps422_trigger(&mut bus, settings.gaps_trigger_use_beta) {
        Err(err) => error!("Unable to set the GAPS trigger! {err}"),
        Ok(_)    => ()
      }
    }
    TriggerType::Gaps211    => {
      match set_gaps211_trigger(&mut bus, settings.gaps_trigger_use_beta) {
        Err(err) => error!("Unable to set the GAPS trigger! {err}"),
        Ok(_)    => ()
      }
    }
    TriggerType::UmbCube => {
      match set_umbcube_trigger(&mut bus) {
        Err(err) => error!("Unable to set UmbCube trigger! {err}"),
        Ok(_)    => ()
      }
    }
    TriggerType::UmbCubeZ => {
      match set_umbcubez_trigger(&mut bus) {
        Err(err) => error!("Unable to set UmbCubeZ trigger! {err}"),
        Ok(_)    => ()
      }
    }
    TriggerType::UmbCorCube => {
      match set_umbcorcube_trigger(&mut bus) {
        Err(err) => error!("Unable to set UmbCorCube trigger! {err}"),
        Ok(_)    => ()
      }
    }
    TriggerType::CorCubeSide => {
      match set_corcubeside_trigger(&mut bus) {
        Err(err) => error!("Unable to set CorCubeSide trigger! {err}"),
        Ok(_)    => ()
      }
    }
    TriggerType::Umb3Cube => {
      match set_umb3cube_trigger(&mut bus) {
        Err(err) => error!("Unable to set Umb3Cube trigger! {err}"), 
        Ok(_)    => ()
      }
    }
    TriggerType::Unknown => {
      println!("== ==> Not setting any trigger condition. You can set it through pico_hal.py");
      warn!("Trigger condition undefined! Not setting anything!");
      error!("Trigger conditions unknown!");
    }
    _ => {
      error!("Trigger type {} not covered!", settings.trigger_type);
      println!("= => Not setting any trigger condition. You can set it through pico_hal.py");
      warn!("Trigger condition undefined! Not setting anything!");
      error!("Trigger conditions unknown!");
    }
  }
    
  // global trigger type
  if settings.use_combo_trigger {
    let global_prescale = settings.global_trigger_prescale;
    let prescale_val    = (u32::MAX as f32 * global_prescale as f32).floor() as u32;
    println!("=> Setting an additonal trigger - using combo mode. Using prescale of {prescale_val}");
    // FIXME - the "global" is wrong. We need to rename this at some point
    match settings.global_trigger_type {
      TriggerType::Any             => {
        match ANY_TRIG_PRESCALE.set(&mut bus, prescale_val) {
          Ok(_)    => (),
          Err(err) => error!("Settting the prescale {} for the any trigger failed! {err}", prescale_val) 
        }
      }
      TriggerType::Track           => {
        match TRACK_TRIG_PRESCALE.set(&mut bus, prescale_val) {
          Ok(_)    => (),
          Err(err) => error!("Settting the prescale {} for the any trigger failed! {err}", prescale_val) 
        }
      }
      TriggerType::TrackCentral    => {
        match TRACK_CENTRAL_PRESCALE.set(&mut bus, prescale_val) {
          Ok(_)    => (),
          Err(err) => error!("Settting the prescale {} for the track central trigger failed! {err}", prescale_val) 
        }
      }
      TriggerType::TrackUmbCentral => {
        match TRACK_UMB_CENTRAL_PRESCALE.set(&mut bus, prescale_val) {
          Ok(_)    => (),
          Err(err) => error!("Settting the prescale {} for the track umb central trigger failed! {err}", prescale_val) 
        }
      }
      _ => {
        error!("Unable to set {} as a global trigger type!", settings.global_trigger_type);
      }
    }
  }
  
  // reset the DAQ event queue before start
  match reset_daq(&mut bus) {//, &mt_address) {
    Err(err) => {
      error!("Can not reset DAQ! {err}");
    }
    Ok(_)    => ()
  }
  Ok(())
}

/// Communications with the master trigger over Udp
///
/// The master trigger can send packets over the network.
/// These packets contain timestamps as well as the 
/// eventid and a hitmaks to identify which LTBs have
/// participated in the trigger.
/// The packet format is described
/// [here](https://gitlab.com/ucla-gaps-tof/firmware/-/tree/develop/)
///
/// # Arguments
///
/// * mt_address        : Udp address of the MasterTriggerBoard
///
/// * mt_sender         : push retrieved MasterTriggerEvents to 
///                       this channel
/// * mtb_timeout_sec   : reconnect to mtb when we don't
///                       see events in mtb_timeout seconds.
///
/// * verbose           : Print "heartbeat" output 
///
pub fn master_trigger(mt_address     : &str,
                      mt_sender      : &Sender<MasterTriggerEvent>,
                      moni_sender    : &Sender<TofPacket>, 
                      thread_control : Arc<Mutex<ThreadControl>>,
                      verbose        : bool) {

  // missing event analysis - has to go away eventually
  //let mut event_id_test = Vec::<u32>::new();

  let mut bus : IPBus;

  // timers - when to reconnect if no 
  // events have been received in a 
  // certain timeinterval
  let mut heartbeat      = MTBHeartbeat::new();
  let mut mtb_timeout    = Instant::now();
  let mut moni_interval  = Instant::now();
  let mut tc_timer       = Instant::now();
  
  let mut settings       : MTBSettings;
  let mut cali_active    : bool;
  loop {
    match thread_control.lock() {
      Ok(tc) => {
        settings    = tc.liftof_settings.mtb_settings.clone();  
        cali_active = tc.calibration_active; 
      }
      Err(err) => {
        error!("Can't acquire lock for ThreadControl! Unable to set calibration mode! {err}");
        return;
      }
    }
    if !cali_active {
      break;
    } else {
      thread::sleep(Duration::from_secs(5));
    }
    if moni_interval.elapsed().as_secs() > settings.mtb_moni_interval {
      match IPBus::new(mt_address) {
        Err(err) => {
          debug!("Can't connect to MTB, will try again in 10 ms! {err}");
          continue;
        }
        Ok(mut moni_bus) => {
          match get_mtbmonidata(&mut moni_bus) { 
            Err(err) => {
              error!("Can not get MtbMoniData! {err}");
            },
            Ok(moni) => {
              let tp = moni.pack();
              match moni_sender.send(tp) {
                Err(err) => {
                  error!("Can not send MtbMoniData over channel! {err}");
                },
                Ok(_) => ()
              }
            }
          }
        }
      }
      moni_interval = Instant::now();
    }
  } 
  let mtb_timeout_sec    = settings.mtb_timeout_sec;
  let mtb_moni_interval  = settings.mtb_moni_interval;
  
  // verbose, debugging
  let mut last_event_id           = 0u32;
  //let mut n_events                   = 0u64;
  //let mut rate_from_reg  : Option<u32> = None;
  //let mut verbose_timer       = Instant::now();
  //let mut total_elapsed              = 0f64;
  //let mut n_ev_unsent                = 0u64;
  //let mut n_ev_missed                = 0u64;
  let mut first                  = true;
  let mut slack_cadence           = 5; // send only one slack message 
                              // every 5 times we send moni data
  let mut evq_num_events      = 0u64;
  let mut n_iter_loop         = 0u64;
  let mut hb_timer            = Instant::now();
  let hb_interval             = Duration::from_secs(settings.hb_send_interval as u64);

  match configure_mtb(mt_address, &settings) {
    Err(err) => error!("Configuring the MTB failed! {err}"),
    Ok(())   => ()
  }

  let connection_timeout = Instant::now(); 
  loop { 
    match IPBus::new(mt_address) {
      Err(err) => {
        debug!("Can't connect to MTB, will try again in 10 ms! {err}");
        //panic!("Without MTB, we can't proceed and might as well panic!");
        thread::sleep(Duration::from_millis(10));
      }
      Ok(_bus) => {
        bus = _bus;
        break
        //thread::sleep(Duration::from_micros(1000));
      }
    }
    if connection_timeout.elapsed().as_secs() > 10 {
      error!("Unable to connect to MTB after 10 seconds!");
      match thread_control.lock() {
        Ok(mut tc) => {
          tc.thread_master_trg_active = false;
        }
        Err(err) => {
          error!("Can't acquire lock for ThreadControl! Unable to set calibration mode! {err}");
        },
      }
      return;
    }
  }
  
  debug!("Resetting master trigger DAQ");
  // We'll reset the pid as well
  bus.pid = 0;
  match bus.realign_packet_id() {
    Err(err) => error!("Can not realign packet ID! {err}"),
    Ok(_)    => ()
  }
  
  match reset_daq(&mut bus) {//, &mt_address) {
    Err(err) => error!("Can not reset DAQ! {err}"),
    Ok(_)    => ()
  }
  loop {
    // Check thread control and what to do
    // Deactivate this for now
    if tc_timer.elapsed().as_secs_f32() > 2.5 {
      match thread_control.try_lock() {
        Ok(mut tc) => {
          if tc.stop_flag || tc.sigint_recvd {
            tc.end_all_rb_threads = true;
            break;
          }
        
        },
        Err(err) => {
          error!("Can't acquire lock for ThreadControl! Unable to set calibration mode! {err}");
        },
      }
      tc_timer = Instant::now();
    }
    // This is a recovery mechanism. In case we don't see an event
    // for mtb_timeout_sec, we attempt to reconnect to the MTB
    if mtb_timeout.elapsed().as_secs() > mtb_timeout_sec {
      if mtb_timeout.elapsed().as_secs() > mtb_timeout_sec {
        println!("= => [master_trigger] reconnection timer elapsed");
      } else {
        println!("= => [master_trigger] reconnection requested");
      }
      match IPBus::new(mt_address) {
        Err(err) => {
          error!("Can't connect to MTB! {err}");
          //panic!("Without MTB, we can't proceed and might as well panic!");
          continue; // try again
        }
        Ok(_bus) => {
          bus = _bus;
          //thread::sleep(Duration::from_micros(1000));
          debug!("Resetting master trigger DAQ");
          // We'll reset the pid as well
          bus.pid = 0;
          match bus.realign_packet_id() {
            Err(err) => error!("Can not realign packet ID! {err}"),
            Ok(_)    => ()
          }
          match reset_daq(&mut bus) {//, &mt_address) {
            Err(err) => error!("Can not reset DAQ! {err}"),
            Ok(_)    => ()
          }
        }
      }
      //match bus.reconnect() {//, &mt_address) {
      //  Err(err) => error!("Can not reconnect NTB! {err}"),
      //  Ok(_)    => ()
      //}
      mtb_timeout    = Instant::now();
    }
    if moni_interval.elapsed().as_secs() > mtb_moni_interval || first {
      if first {
        first = false;
      }
      match get_mtbmonidata(&mut bus) { 
        Err(err) => {
          error!("Can not get MtbMoniData! {err}");
        },
        Ok(_moni) => {
          if settings.tofbot_webhook != String::from("")  {
            let url  = &settings.tofbot_webhook;
            let message = format!("\u{1F916}\u{1F680}\u{1F388} [LIFTOF (Bot)]\n rate - {}[Hz]\n {}", _moni.rate, settings);
            let clean_message = remove_from_word(message, "tofbot_webhook");
            let data = json!({
              "text" : clean_message
            });
            match serde_json::to_string(&data) {
              Ok(data_string) => {
                if slack_cadence == 0 {
                  match ureq::post(url)
                      .set("Content-Type", "application/json")
                      .send_string(&data_string) {
                    Err(err) => { 
                      error!("Unable to send {} to TofBot! {err}", data_string);
                    }
                    Ok(response) => {
                      match response.into_string() {
                        Err(err) => {
                          error!("Not able to read response! {err}");
                        }
                        Ok(body) => {
                          if verbose {
                            println!("[master_trigger] - TofBot responded with {}", body);
                          }
                        }
                      }
                    }
                  }
                } else {
                  slack_cadence -= 1;
                }
                if slack_cadence == 0 {
                  slack_cadence = 5;
                }
              }
              Err(err) => {
                error!("Can not convert .json to string! {err}");
              }
            }
          }
          let tp = TofPacket::from(&_moni);
          match moni_sender.send(tp) {
            Err(err) => {
              error!("Can not send MtbMoniData over channel! {err}");
            },
            Ok(_) => ()
          }
        }
      }
      moni_interval = Instant::now();
    }
    
    match get_event(&mut bus){ //,
      Err(err) => {
        match err {
          MasterTriggerError::PackageFooterIncorrect
          | MasterTriggerError::PackageHeaderIncorrect 
          | MasterTriggerError::DataTooShort
          | MasterTriggerError::BrokenPackage => {
            error!("MasterTriggerEventPackage not adhering to expected format! {err}");
            warn!("Resetting DAQ Event Queue!");
            match reset_daq(&mut bus) {
              Err(err) => error!("Can not reset DAQ, error {err}"),
              Ok(_)    => ()
            }
          }
          _ => ()
        }
        continue;
      },
      Ok(_ev) => {
        if _ev.event_id == last_event_id {
          error!("We got a duplicate event from the MTB!");
          continue;
        }
        if _ev.event_id > last_event_id + 1 {
          if last_event_id != 0 {
            error!("We skipped {} events!", _ev.event_id - last_event_id); 
            heartbeat.n_ev_missed += (_ev.event_id - last_event_id) as u64;
            //event_id_test.push(_ev.event_id);
          }
        }
        last_event_id = _ev.event_id;
        // we got an even successfully, so reset the 
        // connection timeout
        //mtb_timeout = Instant::now();
        
        heartbeat.n_events += 1;
        match mt_sender.send(_ev) {
          Err(err) => {
            error!("Can not send MasterTriggerEvent over channel! {err}");
            heartbeat.n_ev_unsent += 1;
          },
          Ok(_) => ()
        }
      }
    }

    //let verbose_timer_elapsed = verbose_timer.elapsed().as_secs_f64();
    //let mut missing = 0usize;
    //if event_id_test.len() > 0 {
    //  let mut evid = event_id_test[0];
    //  for _ in 0..event_id_test.len() {
    //    if !event_id_test.contains(&evid) {
    //      missing += 1;
    //    }
    //    evid += 1;
    //  }
    //}
    //let evid_check_str = format!(">> ==> In a chunk of {} events, we missed {} ({}%) <<", event_id_test.len(), missing, 100.0*(missing as f64)/event_id_test.len() as f64);
    //event_id_test.clear();
    if hb_timer.elapsed() >= hb_interval {
      match EVQ_NUM_EVENTS.get(&mut bus) {
        Err(err) => {
          error!("Unable to query {}! {err}", EVQ_NUM_EVENTS);
        }
        Ok(num_ev) => {
          evq_num_events += num_ev as u64;
          heartbeat.evq_num_events_last = num_ev as u64;
          n_iter_loop    += 1;
          heartbeat.evq_num_events_avg = (evq_num_events as u64)/(n_iter_loop as u64);
        }
      }
      heartbeat.total_elapsed += hb_timer.elapsed().as_secs() as u64;
      match TRIGGER_RATE.get(&mut bus) {
        Ok(trate) => {
          heartbeat.trate = trate as u64;
        }
        Err(err) => {
          error!("Unable to query {}! {err}", TRIGGER_RATE);
        }
      }
      match LOST_TRIGGER_RATE.get(&mut bus) {
        Ok(lost_trate) => {
          heartbeat.lost_trate = lost_trate as u64;
        }
      
        Err(err) => {
          error!("Unable to query {}! {err}", LOST_TRIGGER_RATE);
        }
      }
      
      if verbose {
        println!("{}", heartbeat);
      }

      let pack = heartbeat.pack();
      match moni_sender.send(pack) {
        Err(err) => {
          error!("Can not send MTB Heartbeat over channel! {err}");
        },
        Ok(_) => ()
      }
      
      hb_timer = Instant::now();
    } 
  }
}
