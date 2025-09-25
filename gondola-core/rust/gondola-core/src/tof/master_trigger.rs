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

//use crate::prelude::*;
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

use comfy_table::modifiers::UTF8_ROUND_CORNERS;
use comfy_table::presets::UTF8_FULL;
use comfy_table::*;

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


#[pyfunction]
#[pyo3(name="prescale_to_u32")]
/// Convert a prescale value in range from 0-1.0 to 
/// an u32 value so that it can be written to the 
/// MTB registers
pub fn wrap_prescale_to_u32(prescale : f32) -> u32 {
  let mut _prescale = prescale;
  prescale_to_u32(prescale)
}

//---------------------------------------
// PORT from pybidings/master_trigger.rs 

#[pyclass]
#[pyo3(name = "MasterTrigger")]
pub struct PyMasterTrigger {
  ipbus : IPBus,
}

#[pymethods]
impl PyMasterTrigger {
  #[new]
  fn new(target_address : &str) -> Self {
    let ipbus = IPBus::new(target_address).expect("Unable to connect to {target_address}");
    Self {
      ipbus : ipbus,
    }
  }

  fn reset_daq(&mut self) -> PyResult<()>{
    match self.ipbus.write(0x10,1) {
      Ok(result) => {
        return Ok(result); 
      }
      Err(err) => {
        return Err(PyValueError::new_err(err.to_string()));
      }
    }
  }

  fn get_expected_pid(&mut self) -> PyResult<u16> {
    match self.ipbus.get_target_next_expected_packet_id(){
      Ok(result) => {
        return Ok(result); 
      }
      Err(err) => {
        return Err(PyValueError::new_err(err.to_string()));
      }
    }
  }

  fn realign_packet_id(&mut self) -> PyResult<()> {
    match self.ipbus.realign_packet_id() {
      Ok(_) => {
        return Ok(()); 
      }
      Err(err) => {
        return Err(PyValueError::new_err(err.to_string()));
      }
    }
  }

  fn set_packet_id(&mut self, pid : u16) {
    self.ipbus.pid = pid;
  }

  fn get_packet_id(&mut self) -> u16 {
    self.ipbus.pid
  }
 
  #[getter]
  /// Get the global trigger rate in Hz
  fn rate(&mut self) -> PyResult<u32> {
    match TRIGGER_RATE.get(&mut self.ipbus) {
      Ok(rate) => {
        return Ok(rate);
      }
      Err(err) => {
        return Err(PyValueError::new_err(err.to_string()));
      }
    }
  }
  
  #[getter]
  /// Get the lost global trigger rate in Hz
  ///
  /// This is the rate of triggers which got 
  /// dropped due to TIU BUSY signal + those which 
  /// get dropped due to the RBs being busy
  fn lost_rate(&mut self) -> PyResult<u32> {
    match LOST_TRIGGER_RATE.get(&mut self.ipbus) {
      Ok(rate) => {
        return Ok(rate);
      }
      Err(err) => {
        return Err(PyValueError::new_err(err.to_string()));
      }
    }
  }
  
  #[getter]
  /// The lost rate which occured due to RB busy timeouts
  fn rb_lost_rate(&mut self) -> PyResult<u32> {
    match RB_LOST_TRIGGER_RATE.get(&mut self.ipbus) {
      Ok(rate) => {
        return Ok(rate);
      }
      Err(err) => {
        return Err(PyValueError::new_err(err.to_string()));
      }
    }
  }

  /// The lost rate which occured due to the tracker BUSY signal
  #[getter]
  fn tiu_lost_rate(&mut self) -> PyResult<u32> {
    match TIU_LOST_TRIGGER_RATE.get(&mut self.ipbus) {
      Ok(rate) => {
        return Ok(rate);
      }
      Err(err) => {
        return Err(PyValueError::new_err(err.to_string()));
      }
    }
  }

  #[getter]
  /// Check if the TIU emulation mode is on
  ///
  fn get_tiu_emulation_mode(&mut self) -> PyResult<u32> {
    match TIU_EMULATION_MODE.get(&mut self.ipbus) {
      Ok(mode) => {
        return Ok(mode);
      }
      Err(err) => {
        return Err(PyValueError::new_err(err.to_string()));
      }
    }
  }
 
  #[setter]
  fn set_tiu_emulation_mode(&mut self, value : u32) -> PyResult<()> {
    match TIU_EMULATION_MODE.set(&mut self.ipbus, value) {
      Ok(_) => {
        return Ok(());
      }
      Err(err) => {
        return Err(PyValueError::new_err(err.to_string()));
      }
    }
  }

  #[setter]
  fn set_tiu_emulation_mode_bsy_cnt(&mut self,  cycles : u32) -> PyResult<()> {
    match TIU_EMU_BUSY_CNT.set(&mut self.ipbus, cycles) {
      Ok(_) => {
        return Ok(());
      }
      Err(err) => {
        return Err(PyValueError::new_err(err.to_string()));
      }
    }
  }
  
  #[getter]
  fn get_tiu_emulation_mode_bsy_cnt(&mut self) -> PyResult<u32> {
    match TIU_EMU_BUSY_CNT.get(&mut self.ipbus) {
      Ok(value) => {
        return Ok(value);
      }
      Err(err) => {
        return Err(PyValueError::new_err(err.to_string()));
      }
    }
  }

  fn get_enable_cyclic_trig(&mut self) -> PyResult<bool> {
    match TRIG_CYCLIC_EN.get(&mut self.ipbus) {
      Ok(value) => {
        return Ok(value > 0);
      }
      Err(err) => {
        return Err(PyValueError::new_err(err.to_string()));
      }
    }
  }

  fn disable_cyclic_trig(&mut self) -> PyResult<()> {
    match TRIG_CYCLIC_EN.set(&mut self.ipbus, 0x0) {
      Ok(_) => {
        return Ok(());
      }
      Err(err) => {
        return Err(PyValueError::new_err(err.to_string()));
      }
    }
  }
  
  fn enable_cyclic_trig(&mut self) -> PyResult<()> {
    match TRIG_CYCLIC_EN.set(&mut self.ipbus, 0x1) {
      Ok(_) => {
        return Ok(());
      }
      Err(err) => {
        return Err(PyValueError::new_err(err.to_string()));
      }
    }
  }
  
  fn get_cyclic_trigger_interval(&mut self) -> PyResult<u32> {
    match TRIG_CYCLIC_INTERVAL.get(&mut self.ipbus) {
      Ok(interval) =>  {
        return Ok(interval);
      }
      Err(err) => {
        return Err(PyValueError::new_err(err.to_string()));
      }
    }
  } 

  fn set_cyclic_trigger_interval(&mut self, interval : u32) -> PyResult<()> {
    match TRIG_CYCLIC_INTERVAL.set(&mut self.ipbus, interval) {
      Ok(_) =>  {
        return Ok(());
      }
      Err(err) => {
        return Err(PyValueError::new_err(err.to_string()));
      }
    }
  }
  
  /// Issue a one-time forced trigger
  fn trigger(&mut self) -> PyResult<()> {
    match FORCE_TRIGGER.set(&mut self.ipbus, 1) {
      Ok(_)  => {
        return Ok(());
      }
      Err(err) => {
        return Err(PyValueError::new_err(err.to_string()));
      }
    }
  }

  fn set_poisson_trigger(&mut self, rate : u32) -> PyResult<()> {
    let clk_period = 100000000;
    let rate_val   = (u32::MAX*rate)/clk_period;//(1.0/ clk_period)).floor() as u32;
    match TRIG_GEN_RATE.set(&mut self.ipbus, rate_val) {
      Ok(_)  => {
        return Ok(());
      }
      Err(err) => {
        return Err(PyValueError::new_err(err.to_string()));
      }
    }
  }

  fn set_gaps_track_trigger(&mut self, prescale : f32, use_beta : bool) -> PyResult<()>  {
    match control::set_gaps_track_trigger(&mut self.ipbus, prescale, use_beta) {
      Ok(_) => {
        return Ok(());
      }
      Err(err) => {
        return Err(PyValueError::new_err(err.to_string()));
      }
    }
  }
  
  fn set_gaps_any_trigger(&mut self, prescale : f32, use_beta : bool) -> PyResult<()>  {
    match control::set_gaps_any_trigger(&mut self.ipbus, prescale, use_beta) {
      Ok(_) => {
        return Ok(());
      }
      Err(err) => {
        return Err(PyValueError::new_err(err.to_string()));
      }
    }
  }

  fn set_gaps_central_track_trigger(&mut self, prescale : f32, use_beta : bool) -> PyResult<()>  {
    match control::set_gaps_central_track_trigger(&mut self.ipbus, prescale, use_beta) {
      Ok(_) => {
        return Ok(());
      }
      Err(err) => {
        return Err(PyValueError::new_err(err.to_string()));
      }
    }
  }

  fn set_gaps422_central_track_trigger(&mut self, prescale : f32, use_beta : bool) -> PyResult<()>  {
    match control::set_gaps422_central_track_trigger(&mut self.ipbus, prescale, use_beta) {
      Ok(_) => {
        return Ok(());
      }
      Err(err) => {
        return Err(PyValueError::new_err(err.to_string()));
      }
    }
  }

  /// Get the status of enabling for LTBs 0-9
  fn get_lt_link_en0(&mut self) -> PyResult<u32> {
    match LT_LINK_EN0.get(&mut self.ipbus) {
      Ok(value)  => {
        return Ok(value);
      }
      Err(err) => {
        return Err(PyValueError::new_err(err.to_string()));
      }
    }
  }
  /// Get the status of enabling for LTBs 10-19
  fn get_lt_link_en1(&mut self) -> PyResult<u32> {
    match LT_LINK_EN1.get(&mut self.ipbus) {
      Ok(value)  => {
        return Ok(value);
      }
      Err(err) => {
        return Err(PyValueError::new_err(err.to_string()));
      }
    }
  }
  /// Get the status of enabling for LTBs 20-29
  fn get_lt_link_en02(&mut self) -> PyResult<u32> {
    match LT_LINK_EN2.get(&mut self.ipbus) {
      Ok(value)  => {
        return Ok(value);
      }
      Err(err) => {
        return Err(PyValueError::new_err(err.to_string()));
      }
    }
  }
  /// Get the status of enabling for LTBs 30-39
  fn get_lt_link_en3(&mut self) -> PyResult<u32> {
    match LT_LINK_EN3.get(&mut self.ipbus) {
      Ok(value)  => {
        return Ok(value);
      }
      Err(err) => {
        return Err(PyValueError::new_err(err.to_string()));
      }
    }
  }
  /// Get the status of enabling for LTBs 40-49
  fn get_lt_link_en4(&mut self) -> PyResult<u32> {
    match LT_LINK_EN4.get(&mut self.ipbus) {
      Ok(value)  => {
        return Ok(value);
      }
      Err(err) => {
        return Err(PyValueError::new_err(err.to_string()));
      }
    }
  }
  ///Set on/off link enabling for LTBs 0-9
  fn set_lt_link_en0(&mut self, value : u32) -> PyResult<u32> {
    match LT_LINK_EN0.set(&mut self.ipbus, value) {
      Ok(_) => {
        return Ok(value);
      }
      Err(err) => {
        return Err(PyValueError::new_err(err.to_string()));
      }
    }
  }
  ///Set on/off link enabling for LTBs 10-19
  fn set_lt_link_en1(&mut self, value : u32) -> PyResult<u32> {
    match LT_LINK_EN1.set(&mut self.ipbus, value) {
      Ok(_) => {
        return Ok(value);
      }
      Err(err) => {
        return Err(PyValueError::new_err(err.to_string()));
      }
    }
  }
  ///Set on/off link enabling for LTBs 20-29
  fn set_lt_link_en2(&mut self, value : u32) -> PyResult<u32> {
    match LT_LINK_EN2.set(&mut self.ipbus, value) {
      Ok(_) => {
        return Ok(value);
      }
      Err(err) => {
        return Err(PyValueError::new_err(err.to_string()));
      }
    }
  }
  ///Set on/off link enabling for LTBs 30-39
  fn set_lt_link_en3(&mut self, value : u32) -> PyResult<u32> {
    match LT_LINK_EN3.set(&mut self.ipbus, value) {
      Ok(_) => {
        return Ok(value);
      }
      Err(err) => {
        return Err(PyValueError::new_err(err.to_string()));
      }
    }
  }
  ///Set on/off link enabling for LTBs 40-49
  fn set_lt_link_en4(&mut self, value : u32) -> PyResult<u32> {
    match LT_LINK_EN4.set(&mut self.ipbus, value) {
      Ok(_) => {
        return Ok(value);
      }
      Err(err) => {
        return Err(PyValueError::new_err(err.to_string()));
      }
    }
  }
  ///get LT LINK AUTOMASK toggle status
  fn get_lt_link_automask(&mut self) -> PyResult<bool> {
    match LT_LINK_AUTOMASK.get(&mut self.ipbus) {
      Ok(value) => {
        return Ok(value != 0);
      }
      Err(err) => {
        return Err(PyValueError::new_err(err.to_string()));
      }
    }
  }
  ///set LT LINK AUTOMASK toggle status
  fn set_lt_link_automask(&mut self, toggle : bool) -> PyResult<bool> {
    match LT_LINK_AUTOMASK.set(&mut self.ipbus, toggle as u32) {
      Ok(_) => {
        return Ok(true);
      }
      Err(err) => {
        return Err(PyValueError::new_err(err.to_string()));
      }
    }
  }
fn get_gaps_trigger_prescale(&mut self) -> PyResult<f32> {
  match GAPS_TRIG_PRESCALE.get(&mut self.ipbus) {
    Ok (prescale_bus) => {
      let prescale_val = (u32::MAX as f32 * prescale_bus as f32).floor() as f32;
    return Ok(prescale_val)
    }
    Err(err) => {
      return Err(PyValueError::new_err(err.to_string()));
    }
  }
}

fn set_gaps_trigger_prescale(&mut self, prescale : f32) -> PyResult<f32> {
  let prescale_val = (f32::MAX * prescale as f32).floor() as u32;
  match GAPS_TRIG_PRESCALE.set(&mut self.ipbus, prescale_val) {
    Ok(_) => {
      return Ok(prescale)
    }
    Err(err) => {
      return Err(PyValueError::new_err(err.to_string()));
    }
  }
}

fn set_track_trigger_is_global(&mut self) -> PyResult<()> {
    match TRACK_CENTRAL_IS_GLOBAL.set(&mut self.ipbus, 1) {
      Ok(_) => {
        return Ok(());
      }
      Err(err) => {
        return Err(PyValueError::new_err(err.to_string()));
      }
    }
  }


  fn get_ltb_links_ready(&mut self) -> PyResult<HashMap<u8, u32>> {
    let registers = [LT_LINK_READY0, LT_LINK_READY1,
                     LT_LINK_READY2, LT_LINK_READY3,
                     LT_LINK_READY4];
    let mut ready = HashMap::<u8, u32>::new();
    for (k,reg) in registers.iter().enumerate() {
      match reg.get(&mut self.ipbus) {
        Err(err) => {
          return Err(PyValueError::new_err(err.to_string()));
        }
        Ok(cnt) => {
          ready.insert(k as u8, cnt);
        }
      }
    }
    Ok(ready)
  }

  fn get_ltb_event_cnts(&mut self) -> PyResult<HashMap<u8, u32>> {
    let registers = [LT0, LT1, LT2, LT3, LT4, LT5, LT6, LT7, LT8, LT9,
                     LT10, LT11, LT12, LT13, LT14, LT15, LT16, LT17, LT18, LT19,
                     LT20, LT21, LT22, LT23, LT24];
    let mut counters = HashMap::<u8, u32>::new();
    for (k,reg) in registers.iter().enumerate() {
      match reg.get(&mut self.ipbus) {
        Err(err) => {
          return Err(PyValueError::new_err(err.to_string()));
        }
        Ok(cnt) => {
          counters.insert(k as u8, cnt);
        }
      }
    }
    // print a table
    let mut table = Table::new();
    table
      .load_preset(UTF8_FULL)
      .apply_modifier(UTF8_ROUND_CORNERS)
      .set_content_arrangement(ContentArrangement::Dynamic)
      .set_width(80)
      .set_header(vec!["LT 0", "LT 1", "LT 2", "LT 3", "LT 4"])
      .add_row(vec![
          Cell::new(&(format!("{}", counters[&0]))),
          Cell::new(&(format!("{}", counters[&1]))),
          Cell::new(&(format!("{}", counters[&2]))),
          Cell::new(&(format!("{}", counters[&3]))),
          Cell::new(&(format!("{}", counters[&4]))),
          //Cell::new("Center aligned").set_alignment(CellAlignment::Center),
      ])
      .add_row(vec![
          Cell::new(String::from("LT 5")),
          Cell::new(String::from("LT 6")),
          Cell::new(String::from("LT 7")),
          Cell::new(String::from("LT 8")),
          Cell::new(String::from("LT 9")),
      ])
      .add_row(vec![
          Cell::new(&(format!("{}", counters[&5]))),
          Cell::new(&(format!("{}", counters[&6]))),
          Cell::new(&(format!("{}", counters[&7]))),
          Cell::new(&(format!("{}", counters[&8]))),
          Cell::new(&(format!("{}", counters[&9]))),
      ])
      .add_row(vec![
          Cell::new(String::from("LT 10")),
          Cell::new(String::from("LT 11")),
          Cell::new(String::from("LT 12")),
          Cell::new(String::from("LT 13")),
          Cell::new(String::from("LT 14")),
      ])
      .add_row(vec![
          Cell::new(&(format!("{}", counters[&10]))),
          Cell::new(&(format!("{}", counters[&11]))),
          Cell::new(&(format!("{}", counters[&12]))),
          Cell::new(&(format!("{}", counters[&13]))),
          Cell::new(&(format!("{}", counters[&14]))),
      ])
      .add_row(vec![
          Cell::new(String::from("LT 15")),
          Cell::new(String::from("LT 16")),
          Cell::new(String::from("LT 17")),
          Cell::new(String::from("LT 18")),
          Cell::new(String::from("LT 19")),
      ])
      .add_row(vec![
          Cell::new(&(format!("{}", counters[&15]))),
          Cell::new(&(format!("{}", counters[&16]))),
          Cell::new(&(format!("{}", counters[&17]))),
          Cell::new(&(format!("{}", counters[&18]))),
          Cell::new(&(format!("{}", counters[&19]))),
      ])
      .add_row(vec![
          Cell::new(String::from("LT 20")),
          Cell::new(String::from("LT 21")),
          Cell::new(String::from("LT 22")),
          Cell::new(String::from("LT 23")),
          Cell::new(String::from("LT 24")),
      ])
      .add_row(vec![
          Cell::new(&(format!("{}", counters[&20]))),
          Cell::new(&(format!("{}", counters[&21]))),
          Cell::new(&(format!("{}", counters[&22]))),
          Cell::new(&(format!("{}", counters[&23]))),
          Cell::new(&(format!("{}", counters[&24]))),
      ]);

    // Set the default alignment for the third column to right
    let column = table.column_mut(2).expect("Our table has three columns");
    column.set_cell_alignment(CellAlignment::Right);
    println!("{table}");
    Ok(counters)
  }
  
  /// Readout the RB event counter registers
  fn get_rb_event_cnts(&mut self) -> PyResult<HashMap<u8, u8>> {
    let registers = [RB0_CNTS, RB1_CNTS, RB2_CNTS, RB3_CNTS, RB4_CNTS,
                     RB5_CNTS, RB6_CNTS, RB7_CNTS, RB8_CNTS, RB9_CNTS,
                     RB10_CNTS, RB11_CNTS, RB12_CNTS, RB13_CNTS, RB14_CNTS,
                     RB15_CNTS, RB16_CNTS, RB17_CNTS, RB18_CNTS, RB19_CNTS,
                     RB20_CNTS, RB21_CNTS, RB22_CNTS, RB23_CNTS, RB24_CNTS,
                     RB25_CNTS, RB26_CNTS, RB27_CNTS, RB28_CNTS, RB29_CNTS,
                     RB30_CNTS, RB31_CNTS, RB32_CNTS, RB33_CNTS, RB34_CNTS,
                     RB35_CNTS, RB36_CNTS, RB37_CNTS, RB38_CNTS, RB39_CNTS,
                     RB40_CNTS, RB41_CNTS, RB42_CNTS, RB43_CNTS, RB44_CNTS,
                     RB45_CNTS, RB46_CNTS, RB47_CNTS, RB48_CNTS, RB49_CNTS];
    let mut counters = HashMap::<u8, u8>::new();
    for (k,reg) in registers.iter().enumerate() {
      match reg.get(&mut self.ipbus) {
        Err(err) => {
          return Err(PyValueError::new_err(err.to_string()));
        }
        Ok(cnt) => {
          counters.insert(k as u8, cnt as u8);
        }
      }
    }
    let mut table = Table::new();
    table
      .load_preset(UTF8_FULL)
      .apply_modifier(UTF8_ROUND_CORNERS)
      .set_content_arrangement(ContentArrangement::Dynamic)
      .set_width(60)
      .set_header(vec!["RB 0", "RB 1", "RB 2", "RB 3", "RB 4"])
      .add_row(vec![
          Cell::new(&(format!("{}", counters[&0]))),
          Cell::new(&(format!("{}", counters[&1]))),
          Cell::new(&(format!("{}", counters[&2]))),
          Cell::new(&(format!("{}", counters[&3]))),
          Cell::new(&(format!("{}", counters[&4]))),
          //Cell::new("Center aligned").set_alignment(CellAlignment::Center),
      ])
      .add_row(vec![
          Cell::new(String::from("RB 5")),
          Cell::new(String::from("RB 6")),
          Cell::new(String::from("RB 7")),
          Cell::new(String::from("RB 8")),
          Cell::new(String::from("RB 9")),
      ])
      .add_row(vec![
          Cell::new(&(format!("{}", counters[&5]))),
          Cell::new(&(format!("{}", counters[&6]))),
          Cell::new(&(format!("{}", counters[&7]))),
          Cell::new(&(format!("{}", counters[&8]))),
          Cell::new(&(format!("{}", counters[&9]))),
      ])
      .add_row(vec![
          Cell::new(String::from("RB 10")),
          Cell::new(String::from("RB 11")),
          Cell::new(String::from("RB 12")),
          Cell::new(String::from("RB 13")),
          Cell::new(String::from("RB 14")),
      ])
      .add_row(vec![
          Cell::new(&(format!("{}", counters[&10]))),
          Cell::new(&(format!("{}", counters[&11]))),
          Cell::new(&(format!("{}", counters[&12]))),
          Cell::new(&(format!("{}", counters[&13]))),
          Cell::new(&(format!("{}", counters[&14]))),
      ])
      .add_row(vec![
          Cell::new(String::from("RB 15")),
          Cell::new(String::from("RB 16")),
          Cell::new(String::from("RB 17")),
          Cell::new(String::from("RB 18")),
          Cell::new(String::from("RB 19")),
      ])
      .add_row(vec![
          Cell::new(&(format!("{}", counters[&15]))),
          Cell::new(&(format!("{}", counters[&16]))),
          Cell::new(&(format!("{}", counters[&17]))),
          Cell::new(&(format!("{}", counters[&18]))),
          Cell::new(&(format!("{}", counters[&19]))),
      ])
      .add_row(vec![
          Cell::new(String::from("RB 20")),
          Cell::new(String::from("RB 21")),
          Cell::new(String::from("RB 22")),
          Cell::new(String::from("RB 23")),
          Cell::new(String::from("RB 24")),
      ])
      .add_row(vec![
          Cell::new(&(format!("{}", counters[&20]))),
          Cell::new(&(format!("{}", counters[&21]))),
          Cell::new(&(format!("{}", counters[&22]))),
          Cell::new(&(format!("{}", counters[&23]))),
          Cell::new(&(format!("{}", counters[&24]))),
      ])
      .add_row(vec![
          Cell::new(String::from("RB 25")),
          Cell::new(String::from("RB 26")),
          Cell::new(String::from("RB 27")),
          Cell::new(String::from("RB 28")),
          Cell::new(String::from("RB 29")),
      ])
      .add_row(vec![
          Cell::new(&(format!("{}", counters[&25]))),
          Cell::new(&(format!("{}", counters[&26]))),
          Cell::new(&(format!("{}", counters[&27]))),
          Cell::new(&(format!("{}", counters[&28]))),
          Cell::new(&(format!("{}", counters[&29]))),
      ])
      .add_row(vec![
          Cell::new(String::from("RB 30")),
          Cell::new(String::from("RB 31")),
          Cell::new(String::from("RB 32")),
          Cell::new(String::from("RB 33")),
          Cell::new(String::from("RB 34")),
      ])
      .add_row(vec![
          Cell::new(&(format!("{}", counters[&30]))),
          Cell::new(&(format!("{}", counters[&31]))),
          Cell::new(&(format!("{}", counters[&32]))),
          Cell::new(&(format!("{}", counters[&33]))),
          Cell::new(&(format!("{}", counters[&34]))),
      ])
      .add_row(vec![
          Cell::new(String::from("RB 35")),
          Cell::new(String::from("RB 36")),
          Cell::new(String::from("RB 37")),
          Cell::new(String::from("RB 38")),
          Cell::new(String::from("RB 39")),
      ])
      .add_row(vec![
          Cell::new(&(format!("{}", counters[&35]))),
          Cell::new(&(format!("{}", counters[&36]))),
          Cell::new(&(format!("{}", counters[&37]))),
          Cell::new(&(format!("{}", counters[&38]))),
          Cell::new(&(format!("{}", counters[&39]))),
      ])
      .add_row(vec![
          Cell::new(String::from("RB 40")),
          Cell::new(String::from("RB 41")),
          Cell::new(String::from("RB 42")),
          Cell::new(String::from("RB 43")),
          Cell::new(String::from("RB 44")),
      ])
      .add_row(vec![
          Cell::new(&(format!("{}", counters[&40]))),
          Cell::new(&(format!("{}", counters[&41]))),
          Cell::new(&(format!("{}", counters[&42]))),
          Cell::new(&(format!("{}", counters[&43]))),
          Cell::new(&(format!("{}", counters[&44]))),
      ])
      .add_row(vec![
          Cell::new(String::from("RB 45")),
          Cell::new(String::from("RB 46")),
          Cell::new(String::from("RB 47")),
          Cell::new(String::from("RB 48")),
          Cell::new(String::from("RB 49")),
      ])
      .add_row(vec![
          Cell::new(&(format!("{}", counters[&45]))),
          Cell::new(&(format!("{}", counters[&46]))),
          Cell::new(&(format!("{}", counters[&47]))),
          Cell::new(&(format!("{}", counters[&48]))),
          Cell::new(&(format!("{}", counters[&49]))),
      ]);

    // Set the default alignment for the third column to right
    let column = table.column_mut(2).expect("Our table has three columns");
    column.set_cell_alignment(CellAlignment::Right);
    println!("{table}");
    Ok(counters)
  }
  
  /// Reset all the RB counters
  fn reset_rb_counters(&mut self) -> PyResult<()> {
    println!("{}", RB_CNTS_RESET);
    match RB_CNTS_RESET.set(&mut self.ipbus, 1) {
      Ok(_) => {
        return Ok(());
      }
      Err(err) => {
        return Err(PyValueError::new_err(err.to_string()));
      }
    }
  }

  /// Reset all the LTB counters
  fn reset_ltb_counters(&mut self) -> PyResult<()> {
    match LT_HIT_CNT_RESET.set(&mut self.ipbus, 1) {
      Ok(_) => {
        return Ok(());
      }
      Err(err) => {
        return Err(PyValueError::new_err(err.to_string()));
      }
    }
  }
  
  /// Set a channel mask for a LTB. 
  ///
  /// # Arguments
  /// * lt_link : 0-24, dsi/j connection of the LTB on the MTB
  /// * mask    : bitmask 1 = ch0 2 = ch1, etc. setting a channel
  ///             to 1 will DISABLE the channel!
  fn set_ltb_ch_mask(&mut self, lt_link : u8, mask : u8) -> PyResult<()> {
    let registers = [LT0_CHMASK, LT1_CHMASK, LT2_CHMASK, LT3_CHMASK, LT4_CHMASK,
                     LT5_CHMASK, LT6_CHMASK, LT7_CHMASK, LT8_CHMASK, LT9_CHMASK,
                     LT10_CHMASK, LT11_CHMASK, LT12_CHMASK, LT13_CHMASK, LT14_CHMASK,
                     LT15_CHMASK, LT16_CHMASK, LT17_CHMASK, LT18_CHMASK, LT19_CHMASK,
                     LT20_CHMASK, LT21_CHMASK, LT22_CHMASK, LT23_CHMASK, LT24_CHMASK];
    if lt_link as usize > registers.len() {
      return Err(PyValueError::new_err(String::from("Mask has to be in range 0-24!")));
    }

    match registers[lt_link as usize].set(&mut self.ipbus, mask as u32) {
      Ok(_) => {
        return Ok(());
      }
      Err(err) => {
        return Err(PyValueError::new_err(err.to_string()));
      }
    }
  }

  
  fn set_trace_suppression(&mut self, trace_sup : bool) -> PyResult<()> {
    let read_all_rb : u32;
    if trace_sup {
      read_all_rb = 0;
    } else {
      read_all_rb = 1;
    }
    match RB_READ_ALL_CHANNELS.set(&mut self.ipbus, read_all_rb) {
      Ok(_)  => {
        Ok(())
      }
      Err(err) => {
        return Err(PyValueError::new_err(err.to_string()));
      }
    }
  }

  fn get_trace_suppression(&mut self) -> PyResult<u32> {
    match RB_READ_ALL_CHANNELS.get(&mut self.ipbus) {
      Ok(cnt) => {
        return Ok(cnt);
      }
      Err(err) => {
        return Err(PyValueError::new_err(err.to_string()));
      }
    }
  }
  
  fn set_total_tof_thresh(&mut self, value : u32) -> PyResult<()> {
    match TOTAL_TOF_THRESH.set(&mut self.ipbus, value) {
      Ok(_)  => {
        Ok(())
      }
      Err(err) => {
        return Err(PyValueError::new_err(err.to_string()));
      }
    }
  }

  fn get_total_tof_thresh(&mut self) -> PyResult<u32> {
    match TOTAL_TOF_THRESH.get(&mut self.ipbus) {
      Ok(cnt) => {
        return Ok(cnt);
      }
      Err(err) => {
        return Err(PyValueError::new_err(err.to_string()));
      }
    }
  }
  
  fn set_inner_tof_thresh(&mut self, value : u32) -> PyResult<()> {
    match INNER_TOF_THRESH.set(&mut self.ipbus, value) {
      Ok(_) =>  {
        return Ok(());
      }
      Err(err) => {
        return Err(PyValueError::new_err(err.to_string()));
      }
    }
  }

  fn get_inner_tof_thresh(&mut self) -> PyResult<u32> {
    match INNER_TOF_THRESH.get(&mut self.ipbus) {
      Ok(cnt) => {
        return Ok(cnt);
      }
      Err(err) => {
        return Err(PyValueError::new_err(err.to_string()));
      }
    }
  }

  fn set_outer_tof_thresh(&mut self, value : u32) -> PyResult<()> {
    match OUTER_TOF_THRESH.set(&mut self.ipbus, value) {
      Ok(_) =>  {
        return Ok(());
      }
      Err(err) => {
        return Err(PyValueError::new_err(err.to_string()));
      }
    }
  }

  fn get_outer_tof_thresh(&mut self) -> PyResult<u32> {
    match OUTER_TOF_THRESH.get(&mut self.ipbus) {
      Ok(cnt) => {
        return Ok(cnt);
      }
      Err(err) => {
        return Err(PyValueError::new_err(err.to_string()));
      }
    }
  }

  fn set_cube_side_thresh(&mut self, value : u32) -> PyResult<()> {
    match CUBE_SIDE_THRESH.set(&mut self.ipbus, value) {
      Ok(_) =>  {
        return Ok(());
      }
      Err(err) => {
        return Err(PyValueError::new_err(err.to_string()));
      }
    }
  }

  fn get_cube_side_thresh(&mut self) -> PyResult<u32> {
    match CUBE_SIDE_THRESH.get(&mut self.ipbus) {
      Ok(cnt) => {
        return Ok(cnt);
      }
      Err(err) => {
        return Err(PyValueError::new_err(err.to_string()));
      }
    }
  }

  fn set_cube_top_thresh(&mut self, value : u32) -> PyResult<()> {
    match CUBE_TOP_THRESH.set(&mut self.ipbus, value) {
      Ok(_) =>  {
        return Ok(());
      }
      Err(err) => {
        return Err(PyValueError::new_err(err.to_string()));
      }
    }
  }

  fn get_cube_top_thresh(&mut self) -> PyResult<u32> {
    match CUBE_TOP_THRESH.get(&mut self.ipbus) {
      Ok(cnt) => {
        return Ok(cnt);
      }
      Err(err) => {
        return Err(PyValueError::new_err(err.to_string()));
      }
    }
  }

  fn set_cube_bot_thresh(&mut self, value : u32) -> PyResult<()> {
    match CUBE_BOT_THRESH.set(&mut self.ipbus, value) {
      Ok(_) =>  {
        return Ok(());
      }
      Err(err) => {
        return Err(PyValueError::new_err(err.to_string()));
      }
    }
  }

  fn get_cube_bot_thresh(&mut self) -> PyResult<u32> {
    match CUBE_BOT_THRESH.get(&mut self.ipbus) {
      Ok(cnt) => {
        return Ok(cnt);
      }
      Err(err) => {
        return Err(PyValueError::new_err(err.to_string()));
      }
    }
  }

  fn set_cube_corner_thresh(&mut self, value : u32) -> PyResult<()> {
    match CUBE_CORNER_THRESH.set(&mut self.ipbus, value) {
      Ok(_) =>  {
        return Ok(());
      }
      Err(err) => {
        return Err(PyValueError::new_err(err.to_string()));
      }
    }
  }

  fn get_cube_corner_thresh(&mut self) -> PyResult<u32> {
    match CUBE_CORNER_THRESH.get(&mut self.ipbus) {
      Ok(cnt) => {
        return Ok(cnt);
      }
      Err(err) => {
        return Err(PyValueError::new_err(err.to_string()));
      }
    }
  }
 
  fn set_umbrella_thresh(&mut self, value : u32) -> PyResult<()> {
    match UMBRELLA_THRESH.set(&mut self.ipbus, value) {
      Ok(_) =>  {
        return Ok(());
      }
      Err(err) => {
        return Err(PyValueError::new_err(err.to_string()));
      }
    }
  }

  fn get_umbrella_thresh(&mut self) -> PyResult<u32> {
    match UMBRELLA_THRESH.get(&mut self.ipbus) {
      Ok(cnt) => {
        return Ok(cnt);
      }
      Err(err) => {
        return Err(PyValueError::new_err(err.to_string()));
      }
    }
  }

  fn set_umbrella_center_thresh(&mut self, value : u32) -> PyResult<()> {
    match UMBRELLA_CENTER_THRESH.set(&mut self.ipbus, value) {
      Ok(_) =>  {
        return Ok(());
      }
      Err(err) => {
        return Err(PyValueError::new_err(err.to_string()));
      }
    }
  }

  fn get_umbrella_center_thresh(&mut self) -> PyResult<u32> {
    match UMBRELLA_CENTER_THRESH.get(&mut self.ipbus) {
      Ok(cnt) => {
        return Ok(cnt);
      }
      Err(err) => {
        return Err(PyValueError::new_err(err.to_string()));
      }
    }
  }

  fn set_cortina_thresh(&mut self, value : u32) -> PyResult<()> {
    match CORTINA_THRESH.set(&mut self.ipbus, value) {
      Ok(_) =>  {
        return Ok(());
      }
      Err(err) => {
        return Err(PyValueError::new_err(err.to_string()));
      }
    }
  }
  
  fn get_cortina_thresh(&mut self) -> PyResult<u32> {
    match CORTINA_THRESH.get(&mut self.ipbus) {
      Ok(cnt) => {
        return Ok(cnt);
      }
      Err(err) => {
        return Err(PyValueError::new_err(err.to_string()));
      }
    }
  }

  fn set_configurable_trigger(&mut self, value : u32) -> PyResult<()> {
    match CONFIGURABLE_TRIGGER_EN.set(&mut self.ipbus, value) {
      Ok(_) => {
        return Ok(());
      }
      Err(err) => {
        return Err(PyValueError::new_err(err.to_string()));
      }
    }
  }
  
  fn get_configurable_trigger(&mut self) -> PyResult<u32> {
    match CONFIGURABLE_TRIGGER_EN.get(&mut self.ipbus) {
      Ok(cnt) => {
        return Ok(cnt);
      }
      Err(err) => {
        return Err(PyValueError::new_err(err.to_string()));
      }
    }
  }

  fn set_any_trigger(&mut self, prescale : u32) -> PyResult<()> {
    match ANY_TRIG_PRESCALE.set(&mut self.ipbus, prescale) {
      Ok(_) =>  {
        return Ok(());
      }
      Err(err) => {
        return Err(PyValueError::new_err(err.to_string()));
      }
    }
  }

  fn set_track_trigger(&mut self, prescale : u32) -> PyResult<()> {
    match TRACK_TRIG_PRESCALE.set(&mut self.ipbus, prescale) {
      Ok(_) =>  {
        return Ok(());
      }
      Err(err) => {
        return Err(PyValueError::new_err(err.to_string()));
      }
    }
  }
  
  fn set_central_track_trigger(&mut self, prescale : u32) -> PyResult<()> {
    match TRACK_CENTRAL_PRESCALE.set(&mut self.ipbus, prescale) {
      Ok(_) =>  {
        return Ok(());
      }
      Err(err) => {
        return Err(PyValueError::new_err(err.to_string()));
      }
    }
  }

  fn use_tiu_aux_link(&mut self, use_it : bool) -> PyResult<()> {
    match control::use_tiu_aux_link(&mut self.ipbus, use_it) {
      Ok(_) => {
        return Ok(());
      }
      Err(err) => {
        return Err(PyValueError::new_err(err.to_string()));
      }
    }
  }

  fn stop_all_triggers(&mut self) -> PyResult<()> {
    match control::unset_all_triggers(&mut self.ipbus) {
      Ok(_) => {
        return Ok(());
      }
      Err(err) => {
        return Err(PyValueError::new_err(err.to_string()));
      }
    }
  }

  fn set_umbcube_trigger(&mut self) -> PyResult<()> {
    match control::set_umbcube_trigger(&mut self.ipbus) {
      Ok(_) => {
        return Ok(());
      }
      Err(err) => {
        return Err(PyValueError::new_err(err.to_string()));
      }
    }
  }
  
  fn set_umbcubez_trigger(&mut self) -> PyResult<()> {
    match control::set_umbcubez_trigger(&mut self.ipbus) {
      Ok(_) => {
        return Ok(());
      }
      Err(err) => {
        return Err(PyValueError::new_err(err.to_string()));
      }
    }
  }

  fn set_umbcorcube_trigger(&mut self) -> PyResult<()> {
    match control::set_umbcorcube_trigger(&mut self.ipbus) {
      Ok(_) => {
        return Ok(());
      }
      Err(err) => {
        return Err(PyValueError::new_err(err.to_string()));
      }
    }
  }

  fn set_corcubeside_trigger(&mut self) -> PyResult<()> {
    match control::set_corcubeside_trigger(&mut self.ipbus) {
      Ok(_) => {
        return Ok(());
      }
      Err(err) => {
        return Err(PyValueError::new_err(err.to_string()));
      }
    }
  }
  
  fn set_umb3cube_trigger(&mut self) -> PyResult<()> {
    match control::set_umb3cube_trigger(&mut self.ipbus) {
      Ok(_) => {
        return Ok(());
      }
      Err(err) => {
        return Err(PyValueError::new_err(err.to_string()));
      }
    }
  }

  #[getter]
  fn get_tiu_busy_ignore(&mut self) -> PyResult<bool> {
    match TIU_BUSY_IGNORE.get(&mut self.ipbus) {
      Ok(bsy) => {
        let res = bsy != 0;
        return Ok(res);
      }
      Err(err) => {
        return Err(PyValueError::new_err(err.to_string()));
      }
    }
  }
  
  #[setter]
  fn set_tiu_busy_ignore(&mut self, bsy : bool) -> PyResult<()> {
    match TIU_BUSY_IGNORE.set(&mut self.ipbus, bsy as u32) {
      Ok(_) => {
        return Ok(());
      }
      Err(err) => {
        return Err(PyValueError::new_err(err.to_string()));
      }
    }
  }

  #[getter]
  fn get_tiu_busy_stuck(&mut self) -> PyResult<bool> {
    match TIU_BUSY_STUCK.get(&mut self.ipbus) {
      Ok(value) => {
        return Ok(value > 0);
      }
      Err(err) => {
        return Err(PyValueError::new_err(err.to_string()));
      }
    }
  }

  #[getter]
  fn get_tiu_bad(&mut self) -> PyResult<bool> {
    match TIU_BAD.get(&mut self.ipbus) {
      Ok(value) => {
        return Ok(value > 0);
      }
      Err(err) => {
        return Err(PyValueError::new_err(err.to_string()));
      }
    }
  }

  fn get_event_cnt(&mut self) -> PyResult<u32> {
    match EVENT_CNT.get(&mut self.ipbus) {
      Ok(cnt) => {
        return Ok(cnt);
      }
      Err(err) => {
        return Err(PyValueError::new_err(err.to_string()));
      }
    }
  }
  
  fn get_event_queue_size(&mut self)
    -> PyResult<u32> {
    match EVQ_SIZE.get(&mut self.ipbus) {
      Ok(cnt) => {
        return Ok(cnt);
      }
      Err(err) => {
        return Err(PyValueError::new_err(err.to_string()));
      }
    }
  }
  
  fn get_event_queue_full(&mut self)
    -> PyResult<u32> {
    match EVQ_FULL.get(&mut self.ipbus) {
      Ok(cnt) => {
        return Ok(cnt);
      }
      Err(err) => {
        return Err(PyValueError::new_err(err.to_string()));
      }
    }
  }

  fn get_nevents_in_queue(&mut self) 
    -> PyResult<u32> {
    match EVQ_NUM_EVENTS.get(&mut self.ipbus) {
      Ok(cnt) => {
        return Ok(cnt);
      }
      Err(err) => {
        return Err(PyValueError::new_err(err.to_string()));
      }
    }
  }

  

  fn get_event(&mut self, read_until_footer : bool, verbose : bool, debug : bool)
    -> PyResult<TofEvent> {
    let use_dbg_version = debug;
    if !use_dbg_version {
      //let mut event = TofEvent::new();
      match get_event(&mut self.ipbus) {
        None => {
          // we just return an empty event!
          warn!("Did not get an event, returning empty event!");
          let event = TofEvent::new();
          Ok(event)
        }
        Some(Err(err)) => {
          //error!("Unable to obtain event from the MTB!");
          return Err(PyValueError::new_err(err.to_string()));
        }
        Some(Ok(mte)) => {
          Ok(mte)
        }
      }
    } else {
      // This can be great for debugging. However, at some point 
      // I'd like to introduce debugging features and have all 
      // the debugging at the same place
      let mut n_daq_words : u16;
      let mut n_daq_words_actual : u16;
      loop {
        match EVQ_NUM_EVENTS.get(&mut self.ipbus) {
          Err(_err) => {
            continue;
          }
          Ok(nevents_in_q) => {
            if nevents_in_q == 0 {
              if verbose {
                println!("[MasterTrigger::get_event] => EventQueue empty!!");
              }
              return Err(PyValueError::new_err(String::from("<MasterTriggerError: EventQueueEmpty>")));
            }
          }
        }
        match self.ipbus.read(0x13) { 
          Err(_err) => {
            // A timeout does not ncecessarily mean that there 
            // is no event, it can also just mean that 
            // the rate is low.
            //trace!("Timeout in read_register for MTB! {err}");
            continue;
          },
          Ok(_n_words) => {
            n_daq_words = (_n_words >> 16) as u16;
            if _n_words == 0 {
              continue;
            }
            //trace!("Got n_daq_words {n_daq_words}");
            let rest = n_daq_words % 2;
            n_daq_words /= 2 + rest; //mtb internally operates in 16bit words, but 
            //                  //registers return 32bit words.
            
            break;
          }
        }
      }
      let mut data : Vec<u32>;
      if verbose {
        println!("[MasterTrigger::get_event] => Will query DAQ for {n_daq_words} words!");
      }
      n_daq_words_actual = n_daq_words;
      match self.ipbus.read_multiple(
                                     0x11,
                                     n_daq_words as usize,
                                     false) {
        Err(err) => {
          if verbose {
            println!("[MasterTrigger::get_event] => failed! {err}");
          }
          return Err(PyValueError::new_err(err.to_string()));
        }
        Ok(_data) => {
          data = _data;
          for (i,word) in data.iter().enumerate() {
            let desc : &str;
            let desc_str : String;
            //let mut nhit_words = 0;
            match i {
              0 => desc = "HEADER",
              1 => desc = "EVENTID",
              2 => desc = "TIMESTAMP",
              3 => desc = "TIU_TIMESTAMP",
              4 => desc = "TIU_GPS32",
              5 => desc = "TIU_GPS16 + TRIG_SOURCE",
              6 => desc = "RB MASK 0",
              7 => desc = "RB MASK 1",
              8 => {
                //nhit_words = nhit_words / 2 + nhit_words % 2;
                desc_str  = format!("BOARD MASK ({} ltbs)", word.count_ones());
                desc  = &desc_str;
              },
              _ => desc = "?"
            }
            if verbose {
              println!("[MasterTrigger::get_event] => DAQ word {}    \t({:x})    \t[{}]", word, word, desc);
            }
          }
        }
      }
      if data[0] != 0xAAAAAAAA {
        if verbose {
          println!("[MasterTrigger::get_event] => Got MTB data, but the header is incorrect {}", data[0]);
        }
        return Err(PyValueError::new_err(String::from("Incorrect header value!")));
      }
      let foot_pos = (n_daq_words - 1) as usize;
      if data.len() <= foot_pos {
        if verbose {
          println!("[MasterTrigger::get_event] => Got MTB data, but the format is not correct");
        }
        return Err(PyValueError::new_err(String::from("Empty data!")));
      }
      if data[foot_pos] != 0x55555555 {
        if verbose {
          println!("[MasterTrigger::get_event] => Did not read unti footer!");
        }
        if read_until_footer {
          if verbose {
            println!("[MasterTrigger::get_event] => .. will read additional words!");
          }
          loop {
            match self.ipbus.read(0x11) {
              Err(err) => {
                if verbose {
                  println!("[MasterTrigger::get_event] => Issues reading from 0x11");
                }
                return Err(PyValueError::new_err(err.to_string()));
              },
              Ok(next_word) => {
                n_daq_words_actual += 1;
                data.push(next_word);
                if next_word == 0x55555555 {
                  break;
                }
              }
            }
          }
          if verbose {
            println!("[MasterTrigger::get_event] => We read {} additional words!", n_daq_words_actual - n_daq_words);
          }
        } else {
          if verbose {
            println!("[MasterTrigger::get_event] => Got MTB data, but the footer is incorrect {}", data[foot_pos]);
          }
          return Err(PyValueError::new_err(String::from("Footer incorrect!")));
        }
      }

      // Number of words which will be always there. 
      // Min event size is +1 word for hits
      //const MTB_DAQ_PACKET_FIXED_N_WORDS : u32 = 9; 
      //let n_hit_packets = n_daq_words as u32 - MTB_DAQ_PACKET_FIXED_N_WORDS;
      //println!("We are expecting {}", n_hit_packets);
      let mut mte          = TofEvent::new();
      mte.event_id         = data[1];
      mte.mt_timestamp     = data[2];
      mte.mt_tiu_timestamp = data[3];
      mte.mt_tiu_gps32     = data[4];
      mte.mt_tiu_gps16     = (data[5] & 0x0000ffff) as u16;
      mte.trigger_sources  = ((data[5] & 0xffff0000) >> 16) as u16;
      //mte.get_trigger_sources();
      let rbmask = (data[7] as u64) << 31 | data[6] as u64; 
      mte.mtb_link_mask  = rbmask;
      mte.dsi_j_mask     = data[8];
      let mut n_hit_words    = n_daq_words_actual - 9 - 2; // fixed part is 11 words
      if n_hit_words > n_daq_words_actual {
        n_hit_words = 0;
        println!("[MasterTrigger::get_event] N hit word calculation failed! fixing... {}", n_hit_words);
      }
      if verbose {
        println!("[MasterTrigger::get_event] => Will read {} hit word", n_hit_words);
      }
      for k in 1..n_hit_words+1 {
        if verbose {
          println!("[MasterTrigger::get_event] => Getting word {}", k);
        }
        let first  = (data[8 + k as usize] & 0x0000ffff) as u16;
        let second = ((data[8 + k as usize] & 0xffff0000) >> 16) as u16; 
        mte.channel_mask.push(first);
        if second != 0 {
          mte.channel_mask.push(second);
        }
      }
      if verbose {
        println!("[MasterTrigger::get_event] => Got MTE \n{}", mte);
      }
      let event = TofEvent::new();
      Ok(event)
    }
  }
}


