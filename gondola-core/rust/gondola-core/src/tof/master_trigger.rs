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
// This file is part of gaps-online-software and published 
// under the GPLv3 license

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

// FIXME - whenever there are too many things, we 
//         just do this. idk if this is bad practice.
//         It might be ok since this is a private import?
use crate::prelude::*;
use crate::io::ipbus::IPBus;

use control::*;
use registers::*;

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

/// In case we get a broken DAQ package, 
/// make sure we at least read it until the next 
/// footer
fn read_until_footer(bus : &mut IPBus) 
  -> Result<Vec<u32>, MasterTriggerError> {
  let mut data = Vec::<u32>::new();
  loop {
    let val = bus.read(0x11)?;
    if val != 0xAAAAAAAA {
        data.push(val);
    }
    if (val == 0x55555555) || (val == 0xAAAAAAAA) {
      break;
    }
  }
  Ok(data)
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
  -> Option<Result<TofEvent, MasterTriggerError>> {
  let mut mte = TofEvent::new();
  let n_daq_words_fixed = 9u32;
  let mut data          : Vec<u32>;
  loop {
    // this register tells us how many times we can read out 
    // the DAQ data register. An event has at least 12 fields.
    // This is for an event with 1 LTB.
    // (see gitlab docs https://gitlab.com/ucla-gaps-tof/firmware)
    // so we definitly wait until we have at least 12. If so 
    // we read out the rest later.
    // If this reutrns an error, we quit right away.
    // FIXME: There might be a tiny bug in this register, 
    // where it is sometimes incorrect 
    // (https://gitlab.com/ucla-gaps-tof/firmware/-/issues/69) - nice!
    if EVQ_SIZE.get(bus).ok()? < 12 {
      return None;
    } else {
      break;
    }
  }
  // we read until we get ltb information, after that 
  match bus.read_multiple(0x11, n_daq_words_fixed as usize, false) {
    Ok(_data) => {
      data = _data;
    }
    Err(err) => {
      return Some(Err(err.into()));
    }
  }
  if data.len() < 9 {
    // something inconsistent happened here. We were requesting more 
    // words than we got, that is bad
    error!("Got MTB data, but the package ends before we get LTB information!");
    warn!("Resetting master trigger DAQ");
    match reset_daq(bus) {//, &mt_address) {
      Err(err) => error!("Can not reset DAQ, error {err}"),
      Ok(_)    => ()
    }
    return Some(Err(MasterTriggerError::DataTooShort));
  }
  let n_ltb = data[8].count_ones(); 
  // in case of an odd number of ltbs, 
  // there are some padding bytes
  let odd   = n_ltb % 2 != 0;
  let n_daq_words_flex : usize;
  let n_hit_words      : usize;
  // get hit fields
  if odd {
    n_hit_words = (n_ltb as usize + 1)/2;
  } else {
    n_hit_words = n_ltb as usize/2;
  }
  // the variable size part of the DAQ event
  n_daq_words_flex = n_hit_words + 2; // crc + footer
  let mut data_flex : Vec<u32>;
  match bus.read_multiple(0x11, n_daq_words_flex, false) {
    Ok(_data) => {
      data_flex = _data;
    }
    Err(err) => { 
      return Some(Err(err.into()));
    }
  }
  data.append(&mut data_flex);
  if data[0] != 0xAAAAAAAA {
    error!("Got MTB data, but the header is incorrect {:x}", data[0]);
    return Some(Err(MasterTriggerError::PackageHeaderIncorrect));
  }
 
  let n_daq_words = n_daq_words_fixed as usize + n_daq_words_flex;
  let foot_pos    = n_daq_words_fixed as usize + n_daq_words_flex - 1;
  if data.len() != foot_pos + 1{
    error!("Somehow the MTB DATA are misaligned! {}, {}", data.len(), foot_pos);
    return Some(Err(MasterTriggerError::DataTooShort));
  }
  if data[foot_pos] != 0x55555555 {
    error!("Got MTB data, but the footer is incorrect {:x}", data[foot_pos]);
    if data[foot_pos] == 0xAAAAAAAA {
      error!("Found next header, the package is TOO LONG! Attempt to fix for this event, but the next is LOST!");
      info!("If we want to fix this, this whole mechanism needs a refactor and needs to fetch more thatn a single event at a time!");
      // kill the lost event
      read_until_footer(bus).ok()?;
      // salvage from this event what is possible
      data.pop();
    } else {
      // we try to recover!
      let mut rest = read_until_footer(bus).ok()?;
      data.append(&mut rest);
      if data.len() != n_daq_words as usize + 1 {
        error!("We tried to recover the event, however, that failed! Expected size of the packet {}, actual size {}", n_daq_words, data.len());
        // get some debugging information to understand why this 
        // happened
        println!("-------------- DEBUG -------------------");
        println!("N LTBs {} ({})", data[8].count_ones(), data[8]);
        for k in data {
          println!("-- {:x} ({})", k,k);
        }
        println!("--------------------");
        return Some(Err(MasterTriggerError::PackageFooterIncorrect));
      } else {
        info!("Event recoveered!");
      }
    }
  }

  // ---------- FIll the MTBEvent now
  mte.event_id           = data[1];
  mte.mt_timestamp       = data[2];
  mte.mt_tiu_timestamp   = data[3];
  mte.mt_tiu_gps32       = data[4];
  mte.mt_tiu_gps16       =  (data[5] & 0x0000ffff) as u16;
  mte.mt_trigger_sources = ((data[5] & 0xffff0000) >> 16) as u16;
  //mte.get_trigger_sources();
  let rbmask = (data[7] as u64) << 32 | data[6] as u64; 
  mte.mtb_link_mask  = rbmask;
  mte.dsi_j_mask     = data[8];
  for k in 9..9 + n_hit_words {
    let ltb_hits = data[k as usize];
    // split them up
    let first  =  (ltb_hits & 0x0000ffff) as u16;
    let second = ((ltb_hits & 0xffff0000) >> 16) as u16;
    mte.channel_mask.push(first);
    // if this is the last hit word, only push 
    // it in case n_ltb is odd
    if k == ( 9 + n_hit_words) {
      if !odd {
         mte.channel_mask.push(second);  
      }
    } else {
      mte.channel_mask.push(second);
    }
  }
  Some(Ok(mte))
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
/// If the settings have a non-zero prescale for 
/// any of the triggers, this will cause the 
/// MTB to start triggering 
/// (if it hasn't been triggering before)
///
/// CHANGELOG - in previous versions, this reset the 
///             MTB DAQ multiple times, this is not 
///             ncessary and caused more issues than
///             actually fixed something, so these got 
///             removed.
///
/// # Arguments:
///   * bus        : IPBus connected to the MTB (UDP)
///   * settings   : configure the MTB according
///                  to these settings 
pub fn configure_mtb(bus : &mut IPBus,
                     settings   : &MTBSettings) -> Result<(), MasterTriggerError> {
  let trace_suppression = settings.trace_suppression;
  match set_trace_suppression(bus, trace_suppression) {
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
  match TIU_BUSY_IGNORE.set(bus, tiu_ignore_busy as u32) {
    Err(err) => error!("Unable to change tiu busy ignore settint! {err}"),
    Ok(_)    => {
      if tiu_ignore_busy {
        warn!("Ignoring TIU since tiu_busy_ignore is set in the config file!");
        println!("==> Ignoring TIU since tiu_busy_ignore is set in the config file!");
      }
    }
  }

  info!("Settting rb integration window!");
  let int_wind = settings.rb_int_window;
  match set_rb_int_window(bus, int_wind) {
    Err(err) => error!("Unable to set rb integration window! {err}"),
    Ok(_)    => {
      info!("rb integration window set to {}", int_wind); 
    } 
  }

  match unset_all_triggers(bus) {
    Err(err) => error!("Unable to undo previous trigger settings! {err}"),
    Ok(_)    => ()
  }
  match settings.trigger_type {
    TriggerType::Poisson => {
      match set_poisson_trigger(bus,settings.poisson_trigger_rate) {
        Err(err) => error!("Unable to set the POISSON trigger! {err}"),
        Ok(_)    => ()
      }
    }
    TriggerType::Any     => {
      match set_any_trigger(bus,settings.trigger_prescale) {
        Err(err) => error!("Unable to set the ANY trigger! {err}"),
        Ok(_)    => ()
      }
    }
    TriggerType::Track   => {
      match set_track_trigger(bus, settings.trigger_prescale) {
        Err(err) => error!("Unable to set the TRACK trigger! {err}"),
        Ok(_)    => ()
      }
    }
    TriggerType::TrackCentral   => {
      match set_central_track_trigger(bus, settings.trigger_prescale) {
        Err(err) => error!("Unable to set the CENTRAL TRACK trigger! {err}"),
        Ok(_)    => ()
      }
    }
    TriggerType::TrackUmbCentral  => {
      match set_track_umb_central_trigger(bus, settings.trigger_prescale) {
        Err(err) => error!("Unable to set the TRACK UMB CENTRAL trigger! {err}"),
        Ok(_)   => ()
      }
    }
    TriggerType::Gaps    => {
      match set_gaps_trigger(bus, settings.gaps_trigger_use_beta) {
        Err(err) => error!("Unable to set the GAPS trigger! {err}"),
        Ok(_)    => ()
      }
    }
    TriggerType::Gaps633    => {
      match set_gaps633_trigger(bus, settings.gaps_trigger_use_beta) {
        Err(err) => error!("Unable to set the GAPS trigger! {err}"),
        Ok(_)    => ()
      }
    }
    TriggerType::Gaps422    => {
      match set_gaps422_trigger(bus, settings.gaps_trigger_use_beta) {
        Err(err) => error!("Unable to set the GAPS trigger! {err}"),
        Ok(_)    => ()
      }
    }
    TriggerType::Gaps211    => {
      match set_gaps211_trigger(bus, settings.gaps_trigger_use_beta) {
        Err(err) => error!("Unable to set the GAPS trigger! {err}"),
        Ok(_)    => ()
      }
    }
    TriggerType::Gaps1044   => {
      match set_gaps1044_trigger(bus, settings.gaps_trigger_use_beta) {
        Err(err) => error!("Unable to set the GAPS trigger! {err}"),
        Ok(_)    => ()
      }
    }
    TriggerType::UmbCube => {
      match set_umbcube_trigger(bus) {
        Err(err) => error!("Unable to set UmbCube trigger! {err}"),
        Ok(_)    => ()
      }
    }
    TriggerType::UmbCubeZ => {
      match set_umbcubez_trigger(bus) {
        Err(err) => error!("Unable to set UmbCubeZ trigger! {err}"),
        Ok(_)    => ()
      }
    }
    TriggerType::UmbCorCube => {
      match set_umbcorcube_trigger(bus) {
        Err(err) => error!("Unable to set UmbCorCube trigger! {err}"),
        Ok(_)    => ()
      }
    }
    TriggerType::CorCubeSide => {
      match set_corcubeside_trigger(bus) {
        Err(err) => error!("Unable to set CorCubeSide trigger! {err}"),
        Ok(_)    => ()
      }
    }
    TriggerType::Umb3Cube => {
      match set_umb3cube_trigger(bus) {
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
    
  // combo trigger - still named "global_trigger" in settings 
  // mistakenly.
  // FIXME
  if settings.use_combo_trigger {
    let global_prescale = settings.global_trigger_prescale;
    let prescale_val    = (u32::MAX as f32 * global_prescale as f32).floor() as u32;
    println!("=> Setting an additonal trigger - using combo mode. Using prescale of {}", prescale_val as f32 / u32::MAX as f32);
    // FIXME - the "global" is wrong. We need to rename this at some point
    match settings.global_trigger_type {
      TriggerType::Any             => {
        match ANY_TRIG_PRESCALE.set(bus, prescale_val) {
          Ok(_)    => (),
          Err(err) => error!("Settting the prescale {} for the any trigger failed! {err}", prescale_val) 
        }
      }
      TriggerType::Track           => {
        match TRACK_TRIG_PRESCALE.set(bus, prescale_val) {
          Ok(_)    => (),
          Err(err) => error!("Settting the prescale {} for the any trigger failed! {err}", prescale_val) 
        }
      }
      TriggerType::TrackCentral    => {
        match TRACK_CENTRAL_PRESCALE.set(bus, prescale_val) {
          Ok(_)    => (),
          Err(err) => error!("Settting the prescale {} for the track central trigger failed! {err}", prescale_val) 
        }
      }
      TriggerType::TrackUmbCentral => {
        match TRACK_UMB_CENTRAL_PRESCALE.set(bus, prescale_val) {
          Ok(_)    => (),
          Err(err) => error!("Settting the prescale {} for the track umb central trigger failed! {err}", prescale_val) 
        }
      }
      _ => {
        error!("Unable to set {} as a global trigger type!", settings.global_trigger_type);
      }
    }
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
/// * mt_sender         : push retrieved TofEvents to 
///                       this channel
/// * mtb_timeout_sec   : reconnect to mtb when we don't
///                       see events in mtb_timeout seconds.
///
/// * verbose           : Print "heartbeat" output 
///
pub fn master_trigger(mt_address     : &str,
                      mt_sender      : &Sender<TofEvent>,
                      moni_sender    : &Sender<TofPacket>, 
                      thread_control : Arc<Mutex<ThreadControl>>,
                      verbose        : bool) {
  let mut bus            : IPBus;
  let mut heartbeat      = MasterTriggerHB::new();
  let mut mtb_timeout    = Instant::now();
  let mut moni_interval  = Instant::now();
  let mut tc_timer       = Instant::now();
  
  let mut settings       : MTBSettings;
  let mut cali_active    : bool;
  let mut holdoff        : bool;
  loop {
    match thread_control.lock() {
      Ok(tc) => {
        settings    = tc.liftof_settings.mtb_settings.clone();  
        cali_active = tc.calibration_active; 
        holdoff     = tc.holdoff_mtb_thread;
      }
      Err(err) => {
        error!("Can't acquire lock for ThreadControl! Unable to set calibration mode! {err}");
        return;
      }
    }

    if holdoff || cali_active {
      thread::sleep(Duration::from_secs(5));
    } else {
      if !holdoff {
        println!("=> Docking clamp released!");
      }
      break;
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
  let mut last_event_id       = 0u32;
  let mut first               = true;
  let mut slack_cadence       = 5; // send only one slack message 
                              // every 5 times we send moni data
  let mut evq_num_events      = 0u64;
  let mut n_iter_loop         = 0u64;
  let mut hb_timer            = Instant::now();
  let hb_interval             = Duration::from_secs(settings.hb_send_interval as u64);
  let mut n_non_recoverable   = 0usize; // events which could not be recovered. We 
                                        // can use this to reset the DAQ at a certain 
                                        // point
  let connection_timeout = Instant::now(); 
  loop { 
    match IPBus::new(mt_address) {
      Err(err) => {
        debug!("Can't connect to MTB, will try again in 10 ms! {err}");
        thread::sleep(Duration::from_millis(10));
      }
      Ok(_bus) => {
        bus = _bus;
        break
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
  
  match EVENT_CNT_RESET.set(&mut bus, 1) {
    Err(err) => error!("Unable to reset event counter! {err}"),
    Ok(_)    => println!("=> Event counter reset!")
  }

  match configure_mtb(&mut bus, &settings) {
    Err(err) => error!("Configuring the MTB failed! {err}"),
    Ok(())   => ()
  }
 
  let mut preload_cache = 1000; // preload the cache when we are starting 
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
        info!("reconnection timer elapsed");
      } else {
        info!("reconnection requested");
      }
      match IPBus::new(mt_address) {
        Err(err) => {
          error!("Can't connect to MTB! {err}");
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
          //let tp = TofPacket::from(&_moni);
          let tp = _moni.pack();
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
      None     => {
      }
      Some(Err(err)) => {
        match err {
          MasterTriggerError::PackageFooterIncorrect
          | MasterTriggerError::PackageHeaderIncorrect 
          | MasterTriggerError::DataTooShort
          | MasterTriggerError::BrokenPackage => {
            // in case we can't recover an event for x times, let's reset the DAQ
            // not sure if 10 is a good number
            if n_non_recoverable == 100 {
              error!("We have seen {} non-recoverable events, let's reset the DAQ!", n_non_recoverable);
              match reset_daq(&mut bus) {//, &mt_address) {
                Err(err) => error!("Can not reset DAQ, error {err}"),
                Ok(_)    => ()
              }
              n_non_recoverable = 0;
            } 
            n_non_recoverable += 1;
          }
          _ => ()
        }
      },
      Some(Ok(_ev)) => {
        if _ev.event_id == last_event_id {
          error!("We got a duplicate event from the MTB!");
          continue;
        }
        if _ev.event_id > last_event_id + 1 {
          if last_event_id != 0 {
            error!("We skipped {} events!", _ev.event_id - last_event_id); 
            heartbeat.n_ev_missed += (_ev.event_id - last_event_id) as u64;
          }
        }
        last_event_id = _ev.event_id;
        heartbeat.n_events += 1;
        match mt_sender.send(_ev) {
          Err(err) => {
            error!("Can not send TofEvent over channel! {err}");
            heartbeat.n_ev_unsent += 1;
          },
          Ok(_) => ()
        }
      }
    }
    
    if preload_cache > 0 {
      preload_cache -= 1;
      continue;
    }
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
      match TRACK_TRIG_PRESCALE.get(&mut bus) {
        Ok(ps) => {
          heartbeat.prescale_track = (ps as f32) / (u32::MAX as f32) ;
        }
        Err(err) => {
          error!("Unable to query {}! {err}", LOST_TRIGGER_RATE);
        }
      }
      match GAPS_TRIG_PRESCALE.get(&mut bus) {
        Ok(ps) => {
          heartbeat.prescale_gaps = (ps as f32) / (u32::MAX as f32) ;
        }
        Err(err) => {
          error!("Unable to query {}! {err}", LOST_TRIGGER_RATE);
        }
      }
      heartbeat.version = ProtocolVersion::V1; 
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
