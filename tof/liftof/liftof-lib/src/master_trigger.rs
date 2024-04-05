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
use std::error::Error;
use std::time::{Duration, Instant};
use std::fmt;
//use std::io;
//use std::collections::HashMap;
//use std::collections::VecDeque;
//use std::net::{
//    UdpSocket,
//    SocketAddr
//};
use std::thread;
use crossbeam_channel::Sender;
use colored::Colorize;
use serde_json::json;

//use tof_dataclasses::DsiLtbRBMapping;
use tof_dataclasses::packets::TofPacket;
use tof_dataclasses::monitoring::MtbMoniData;
//use tof_dataclasses::commands::RBCommand;
use tof_dataclasses::events::MasterTriggerEvent;
use tof_dataclasses::events::master_trigger::TriggerType;
use tof_dataclasses::errors::{
    //IPBusError,
    MasterTriggerError
};
use tof_dataclasses::ipbus::{
    IPBus,
    //IPBusPacketType,
};

//const MT_MAX_PACKSIZE   : usize = 1024;

//use tof_dataclasses::constants::{
//    // N_LTBS,
//    N_CHN_PER_LTB,
//};

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


/// Configure the trigger
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct MTBSettings {
  /// Select the trigger type for this run
  pub trigger_type           : TriggerType,
  /// Select the prescale factor for a run. The
  /// prescale factor is between 0 (no events)
  /// and 1.0 (all events). E.g. 0.1 means allow 
  /// only 10% of the events
  /// THIS DOES NOT APPLY TO THE GAPS OR POISSON 
  /// TRIGGER!
  pub trigger_prescale               : f32,
  /// in case trigger_type = "Poisson", set rate here
  pub poisson_trigger_rate           : u32,
  /// in case trigger_type = "Gaps", set if we want to use 
  /// beta
  pub gaps_trigger_use_beta     : bool,
  /// not sure
  //pub gaps_trigger_inner_thresh : u32,
  ///// not sure
  //pub gaps_trigger_outer_thresh : u32, 
  ///// not sure
  //pub gaps_trigger_total_thresh : u32, 
  ///// not sure
  //pub gaps_trigger_hit_thresh   : u32,
  /// Enable trace suppression on the MTB. If enabled, 
  /// only those RB which hits will read out waveforms.
  /// In case it is disabled, ALL RBs will readout events
  /// ALL the time. For this, we need also the eventbuilder
  /// strategy "WaitForNBoards(40)"
  pub trace_suppression  : bool,
  /// Time in seconds between housekkeping 
  /// packets
  pub mtb_moni_interval  : u64,
  /// The number of seconds we want to wait
  /// without hearing from the MTB before
  /// we attempt a reconnect
  pub mtb_timeout_sec    : u64,
  pub rb_int_window      : u8,
  pub tiu_emulation_mode : bool,
  pub tofbot_webhook     : String,}

impl MTBSettings {
  pub fn new() -> Self {
    Self {
      trigger_type           : TriggerType::Unknown,
      trigger_prescale       : 0.0,
      poisson_trigger_rate   : 0,
      gaps_trigger_use_beta  : true,
      trace_suppression      : true,
      mtb_moni_interval      : 30,
      mtb_timeout_sec        : 60,
      rb_int_window          : 1,
      tiu_emulation_mode     : false,
      tofbot_webhook         : String::from(""),
    }
  }
}

impl fmt::Display for MTBSettings {
  fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
    let disp = toml::to_string(self).unwrap_or(
      String::from("-- DESERIALIZATION ERROR! --"));
    write!(f, "<MTBSettings :\n{}>", disp)
  }
}

impl Default for MTBSettings {
  fn default() -> Self {
    Self::new()
  }
}



/// Read the complete event of the MTB
///
/// FIXME - this can get extended to read 
/// multiple events at once. 
/// For that, we just have to query the
/// event size register multiple times.
///
/// This is currently blocking until we 
/// either get a UDP timeout or a 
/// non-zero result for the MT.EVENT_QUEUE.SIZE 
/// register
///
/// # Arguments
///
/// * bus       : connected IPBus for UDP comms
pub fn get_mtevent(bus : &mut IPBus)
  -> Result<MasterTriggerEvent, MasterTriggerError> {
  let mut mte = MasterTriggerEvent::new();
  let     n_daq_words  : u32;
  //let sleeptime = Duration::from_micros(10);
  //FIXME - reduce polling rate. 10micros is the 
  //fasterst
  //let sleeptime = Duration::from_micros(1000);
  //let mut timeout = Instant::now();
  // FIXME - this is a bad workaround!!
  //match reset_daq(bus) {
  //  Err(err) => error!("Can not reset DAQ, error {err}"),
  //  Ok(_)    => ()
  //}
  loop {
    //thread::sleep(sleeptime);
    // 3 things can happen here:
    // - it returns an error. Then we end this call
    // - it returns 0 - no mt event is ready, we end 
    //   this call
    // - non 0 result -> next mt ready, continue
    // when we are reading, remember that only 
    // the first 16bits are the value we are 
    // interested in
    let nwords = bus.read(0x13)? >> 16;
    if nwords != 0 {
      n_daq_words = nwords/2 + nwords % 2;
      //println!("Read {} from SIZE register", nwords);
      break;
    } 
  }
  
  let data = bus.read_multiple(0x11, n_daq_words as usize, false)?;  
  //println!("{}", data[0]);
  if data[0] != 0xAAAAAAAA {
    error!("Got MTB data, but the header is incorrect {}", data[0]);
    match reset_daq(bus) {
      Err(err) => error!("Can not reset DAQ, error {err}"),
      Ok(_)    => ()
    }
    return Err(MasterTriggerError::PackageHeaderIncorrect);
  }
  let foot_pos = (n_daq_words - 1) as usize;
  if data.len() <= foot_pos {
    error!("Got MTB data, but the format is not correct");
    match reset_daq(bus) {
      Err(err) => error!("Can not reset DAQ, error {err}"),
      Ok(_)    => ()
    }
    return Err(MasterTriggerError::PackageHeaderIncorrect);
  }
  if data[foot_pos] != 0x55555555 {
    error!("Got MTB data, but the footer is incorrect {}", data[foot_pos]);
    match reset_daq(bus) {
      Err(err) => error!("Can not reset DAQ, error {err}"),
      Ok(_)    => ()
    }
    return Err(MasterTriggerError::PackageFooterIncorrect);
  }

  // Number of words which will be always there. 
  // Min event size is +1 word for hits
  let n_hit_words = n_daq_words - MTB_DAQ_PACKET_FIXED_N_WORDS;
  //println!("We are expecting {}", n_hit_packets);
  mte.event_id      = data[1];
  mte.timestamp     = data[2];
  mte.tiu_timestamp = data[3];
  mte.tiu_gps32     = data[4];
  mte.tiu_gps16      = (data[5] & 0x0000ffff) as u16;
  mte.trigger_source = ((data[5] & 0xffff0000) >> 16) as u16;
  //mte.get_trigger_sources();
  let rbmask = (data[7] as u64) << 31 | data[6] as u64; 
  mte.mtb_link_mask  = rbmask;
  mte.dsi_j_mask     = data[8];
  //  this can happen when the subtraction above overflows
  if n_hit_words > n_daq_words {
    error!("N hit word calculation failed! Got {} hit words!", n_hit_words);
    match reset_daq(bus) {
      Err(err) => error!("Can not reset DAQ, error {err}"),
      Ok(_)    => ()
    }
    return Err(MasterTriggerError::BrokenPackage);
  }
  for k in 1..n_hit_words+1 {
    let first  = ( data[8 + k as usize] & 0x0000ffff) as u16;
    let second = ((data[8 + k as usize] & 0xffff0000) >> 16) as u16; 
    mte.channel_mask.push(first);
    if second != 0 {
      mte.channel_mask.push(second);
    }
  }
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
  // sensors are 12 bit
  let first_word   = 0x00000fff;
  let second_word  = 0x0fff0000;
  println!("[get_mtbmonidata] => Received data from registers {:?} data", data);
  moni.calibration = ( data[0] & first_word  ) as u16;
  moni.vccpint     = ((data[0] & second_word ) >> 16) as u16;  
  moni.vccpaux     = ( data[1] & first_word  ) as u16;  
  moni.vccoddr     = ((data[1] & second_word ) >> 16) as u16;  
  moni.temp        = ( data[2] & first_word  ) as u16;  
  moni.vccint      = ((data[2] & second_word ) >> 16) as u16;  
  moni.vccaux      = ( data[3] & first_word  ) as u16;  
  moni.vccbram     = ((data[3] & second_word ) >> 16) as u16;  
 
  let rate = bus.read_multiple(0x17, 2, true)?;
  // FIXME - technically, the rate is 24bit, however, we just
  // read out 16 here (if the rate is beyond ~65kHz, we don't need 
  // to know with precision
  let mask        = 0x0000ffff;
  moni.rate       = (rate[0] & mask) as u16;
  moni.lost_rate  = (rate[1] & mask) as u16;
  Ok(moni)
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
/// * mtb_moni_interval : time in seconds when we 
///                       are acquiring mtb moni data.
///
/// * mtb_timeout_sec   : reconnect to mtb when we don't
///                       see events in mtb_timeout seconds.
///
/// * verbose           : Print "heartbeat" output 
///
pub fn master_trigger(mt_address        : String,
                      mt_sender         : &Sender<MasterTriggerEvent>,
                      moni_sender       : &Sender<TofPacket>,
                      settings          : MTBSettings,
                      verbose           : bool) {

  // missing event analysis - has to go away eventually
  //let mut event_id_test = Vec::<u32>::new();

  let mut bus : IPBus;
  match IPBus::new(mt_address.clone()) {
    // if that doesn't work, then probably the 
    // configuration is wrong, wo we might as 
    // well panic
    Err(err) => {
      error!("Can't connect to MTB! {err}");
      panic!("Without MTB, we can't proceed and might as well panic!");
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
  let tiu_emulation_mode = settings.tiu_emulation_mode;
  match set_tiu_emulation_mode(&mut bus, tiu_emulation_mode) {
    Err(err) => error!("Unable to change tiu emulation mode! {err}"),
    Ok(_) => {
      if tiu_emulation_mode {
        println!("==> Setting TIU emulation mode! This setting is useful if the TIU is NOT connected!");
      } else {
        println!("==> Not setting TIU emulation mode! TIU needs to be active and connectected!");
      }
    }
  }


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
  match settings.trigger_type {
    TriggerType::Poisson => {
      match unset_all_triggers(&mut bus) {
        Err(err) => error!("Unable to undo previous trigger settings! {err}"),
        Ok(_)    => ()
      }
      match set_poisson_trigger(&mut bus,settings.poisson_trigger_rate) {
        Err(err) => error!("Unable to set the POISSON trigger! {err}"),
        Ok(_)    => ()
      }
    }
    TriggerType::Any     => {
      match unset_all_triggers(&mut bus) {
        Err(err) => error!("Unable to undo previous trigger settings! {err}"),
        Ok(_)    => ()
      }
      match set_any_trigger(&mut bus,settings.trigger_prescale) {
        Err(err) => error!("Unable to set the ANY trigger! {err}"),
        Ok(_)    => ()
      }
    }
    TriggerType::Track   => {
      match unset_all_triggers(&mut bus) {
        Err(err) => error!("Unable to undo previous trigger settings! {err}"),
        Ok(_)    => ()
      }
      match set_track_trigger(&mut bus, settings.trigger_prescale) {
        Err(err) => error!("Unable to set the TRACK trigger! {err}"),
        Ok(_)    => ()
      }
    }
    TriggerType::TrackCentral   => {
      match unset_all_triggers(&mut bus) {
        Err(err) => error!("Unable to undo previous trigger settings! {err}"),
        Ok(_)    => ()
      }
      match set_central_track_trigger(&mut bus, settings.trigger_prescale) {
        Err(err) => error!("Unable to set the CENTRAL TRACK trigger! {err}"),
        Ok(_)    => ()
      }
    }
    TriggerType::Gaps    => {
      match unset_all_triggers(&mut bus) {
        Err(err) => error!("Unable to undo previous trigger settings! {err}"),
        Ok(_)    => ()
      }
      match set_gaps_trigger(&mut bus, settings.gaps_trigger_use_beta) {
        Err(err) => error!("Unable to set the GAPS trigger! {err}"),
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
      println!("== ==> Not setting any trigger condition. You can set it through pico_hal.py");
      warn!("Trigger condition undefined! Not setting anything!");
      error!("Trigger conditions unknown!");
    }
  }

  // step 2 - event loop
  
  // timers - when to reconnect if no 
  // events have been received in a 
  // certain timeinterval
  let mut mtb_timeout    = Instant::now();
  let mut moni_interval  = Instant::now();
  let mtb_timeout_sec    = settings.mtb_timeout_sec;
  let mtb_moni_interval  = settings.mtb_moni_interval;
  // verbose, debugging
  let mut last_event_id  = 0u32;
  let mut n_events       = 0u64;
  //let mut rate_from_reg  : Option<u32> = None;
  let mut verbose_timer  = Instant::now();
  let mut total_elapsed  = 0f64;
  let mut n_ev_unsent    = 0u64;
  let mut n_ev_missed    = 0u64;
  let mut init_reconnect = false;
  let mut first          = true;
  loop {
    if (mtb_timeout.elapsed().as_secs() > mtb_timeout_sec) || init_reconnect {
      if mtb_timeout.elapsed().as_secs() > mtb_timeout_sec {
        println!("==> [master_trigger] reconnection timer elapsed");
      } else {
        println!("==> [master_trigger] reconnection requested");
      }
      match IPBus::new(mt_address.clone()) {
        Err(err) => {
          error!("Can't connect to MTB! {err}");
          //panic!("Without MTB, we can't proceed and might as well panic!");
        }
        Ok(_bus) => {
          bus = _bus;
          thread::sleep(Duration::from_micros(1000));
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
      init_reconnect = false;
      mtb_timeout    = Instant::now();
      //match bus.reconnect() {//, &mt_address) {
      //  Err(err) => error!("Can not reconnect NTB! {err}"),
      //  Ok(_)    => ()
      //}
    }
    if moni_interval.elapsed().as_secs() > mtb_moni_interval || first {
      if first {
        first = false;
      }
      match get_mtbmonidata(&mut bus) { 
                            //&mut buffer) {
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
          //if verbose {
          //  println!("{}", _moni);
          //  rate_from_reg = Some(_moni.rate as u32);
          //}
        }
      }
      moni_interval = Instant::now();
    }
    match get_mtevent(&mut bus){ //,
      Err(err) => {
        error!("Unable to get MasterTriggerEvent! {err}");
        //init_reconnect = true;
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
            n_ev_missed += (_ev.event_id - last_event_id) as u64;
            //event_id_test.push(_ev.event_id);
          }
        }
        last_event_id = _ev.event_id;
        // we got an even successfully, so reset the 
        // connection timeout
        mtb_timeout = Instant::now();
        n_events += 1;
        match mt_sender.send(_ev) {
          Err(err) => {
            error!("Can not send MasterTriggerEvent over channel! {err}");
            n_ev_unsent += 1;
          },
          Ok(_) => ()
        }

      }
    }
    if verbose {
      let verbose_timer_elapsed = verbose_timer.elapsed().as_secs_f64();
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
      if verbose_timer_elapsed > 30.0 {
        total_elapsed += verbose_timer_elapsed;
        println!("  {:<60} <<", ">> == == == == == == ==  MT HEARTBEAT == ==  == == == == ==".bright_blue().bold());
        println!("  {:<60} <<", format!(">> ==> MET (Mission Elapsed Time) (sec) {:.1}",total_elapsed).bright_blue());
        println!("  {:<60} <<", format!(">> ==> Recorded Events                  {}", n_events).bright_blue());
        println!("  {:<60} <<", format!(">> ==> -- trigger rate (Hz)             {:.2}", n_events as f64/total_elapsed).bright_blue());
        
        if n_ev_unsent > 0 {
          println!("  {}{}{}", ">> ==> ".yellow().bold(),n_ev_unsent, " sent errors                       <<".yellow().bold());
        }
        if n_ev_missed > 0 {
          println!("  {}{}{}", ">> ==> ".yellow().bold(),n_events, " missed events                       <<".yellow().bold());
        }
        println!("  {:<60} <<", ">> == == == == == == ==  END HEARTBEAT = ==  == == == == ==".bright_blue().bold());
        verbose_timer = Instant::now();
      }
    }
  }
}


/// Read event counter register of MTB
pub fn read_event_cnt(bus : &mut IPBus) //,
                      //buffer : &mut [u8;MT_MAX_PACKSIZE])
  -> Result<u32, Box<dyn Error>> {
  let event_count = bus.read(0xd)?;
  trace!("Got event count! {} ", event_count);
  Ok(event_count)
}

/// Set the RB readout mode - either 
/// read out all channels all the time
/// or use the MTB to indicate to the RBs
/// which channels to read out 
pub fn set_trace_suppression(bus : &mut IPBus,
                             sup : bool) 
  -> Result<(), Box<dyn Error>> {
  info!("Setting MTB trace suppression {}!", sup);
  let mut value = bus.read(0xf)?;
  // bit 13 has to be 1 for read all channels
  let read_all_ch = u32::pow(2, 13);
  if sup { // sup means !read_all_ch
    value = value & !read_all_ch;
  }
  else {
    value = value | read_all_ch; 
  }
  bus.write(0xf, value)?;
  Ok(())
}

/// Reset the state of the MTB DAQ buffer
/// This can be safely issued without 
/// resetting the event id
pub fn reset_daq(bus : &mut IPBus) 
  -> Result<(), Box<dyn Error>> {
  info!("Resetting DAQ!");
  bus.write(0x10, 1)?;
  Ok(())
}

pub fn get_tiu_link_status(bus : &mut IPBus)
  -> Result<bool, Box<dyn Error>> {
  let mut tiu_good = 0x1u32;
  let value        = bus.read(0xf)?;
  tiu_good         = tiu_good & ( value & 0x1);
  Ok(tiu_good > 0)
}

/// FIXME
pub fn set_rb_int_window(bus : &mut IPBus, wind : u8)
  -> Result<(), Box<dyn Error>> {
  info!("Setting RB_INT_WINDOW to {}!", wind);
  let mut value  =  bus.read(0xf)?;
  println!("==> Retrieved {value} from register 0xf on MTB");
  let mask   = 0xffffe0ff;
  // switch the bins off
  value          = value & mask;
  let wind_bits  = (wind as u32) << 8;
  value = value | wind_bits;
  bus.write(0xf, value)?;
  trace!("++ Writing to register ++");
  value = bus.read(0xf)?;
  trace!("==> Reading back value {value} from register 0xf on MTB after writing to it!");
  Ok(())
}

/// Set the poisson trigger with a prescale
pub fn set_poisson_trigger(bus : &mut IPBus, rate : u32) 
  -> Result<(), Box<dyn Error>> {
  //let clk_period = 1e8u32; 
  let clk_period = 100000000;
  let rate_val = (u32::MAX*rate)/clk_period;//(1.0/ clk_period)).floor() as u32;
  
  //let rate_val   = (rate as f32 * u32::MAX as f32/1.0e8) as u32; 
  info!("Setting poisson trigger with rate {}!", rate);
  bus.write(0x9, rate_val)?;
  Ok(())
}

/// Set the any trigger with a prescale
pub fn set_any_trigger(bus : &mut IPBus, prescale : f32) 
  -> Result<(), Box<dyn Error>> {
  let prescale_val = (u32::MAX as f32 * prescale).floor() as u32;
  info!("Setting any trigger with prescale {}!", prescale);
  bus.write(0x40, prescale_val)?;
  Ok(())
}

/// Set the track trigger with a prescale
pub fn set_track_trigger(bus : &mut IPBus, prescale : f32) 
  -> Result<(), Box<dyn Error>> {
  let prescale_val = (u32::MAX as f32 * prescale).floor() as u32;
  info!("Setting track trigger with prescale {}!", prescale);
  bus.write(0x41, prescale_val)?;
  Ok(())
}

/// Set the CENTRAL track trigger with a prescale
pub fn set_central_track_trigger(bus : &mut IPBus, prescale : f32) 
  -> Result<(), Box<dyn Error>> {
  let prescale_val = (u32::MAX as f32 * prescale).floor() as u32;
  info!("Setting CENTRAL TRACK trigger with prescale {}!", prescale);
  bus.write(0x42, prescale_val)?;
  Ok(())
}

/// Disable all triggers
pub fn unset_all_triggers(bus : &mut IPBus) 
  -> Result<(), Box<dyn Error>> {
  // first the GAPS trigger, whcih is a more 
  // complicated register, where we only have
  // to flip 1 bit
  let mut trig_settings = bus.read(0x14)?;
  trig_settings         = trig_settings & !u32::pow(2,24);
  bus.write(0x14, trig_settings)?;
  set_poisson_trigger(bus, 0)?;
  set_any_trigger    (bus, 0.0)?;
  set_track_trigger  (bus, 0.0)?;
  set_central_track_trigger(bus, 0.0)?;
  Ok(())
}

/// Set the gaps trigger with a prescale
pub fn set_gaps_trigger(bus : &mut IPBus, use_beta : bool) 
  -> Result<(), Box<dyn Error>> {
  info!("Setting GAPS Antiparticle trigger, use beta {}!", use_beta);
  let mut trig_settings = bus.read(0x14)?;
  trig_settings = trig_settings | u32::pow(2,24);
  if use_beta {
    trig_settings = trig_settings | u32::pow(2,25);
  }
  bus.write(0x14, trig_settings)?;
  Ok(())
}


pub fn set_inner_tof_threshold(bus : &mut IPBus, thresh : u8)
  -> Result<(), Box<dyn Error>> {
  info!("Setting inner TOF threshold {}!", thresh);
  let mut trig_settings = bus.read(0x14)?;
  trig_settings = trig_settings | thresh as u32;
  bus.write(0x14, trig_settings)?;
  Ok(())
}

pub fn set_outer_tof_threshold(bus : &mut IPBus, thresh : u8)
  -> Result<(), Box<dyn Error>> {
  info!("Setting outer TOF threshold {}!", thresh);
  let mut trig_settings = bus.read(0x14)?;
  trig_settings = trig_settings | ((thresh as u32) << 8);
  bus.write(0x14, trig_settings)?;
  Ok(())
}

pub fn set_total_tof_threshold(bus : &mut IPBus, thresh : u8)
  -> Result<(), Box<dyn Error>> {
  info!("Setting total TOF threshold {}!", thresh);
  let mut trig_settings = bus.read(0x14)?;
  trig_settings = trig_settings | ((thresh as u32) << 16);
  bus.write(0x14, trig_settings)?;
  Ok(())
}

pub fn set_cube_side_threshold(bus : &mut IPBus, thresh : u8)
  -> Result<(), Box<dyn Error>> {
  info!("Setting cube side threshold {}!", thresh);
  let mut trig_settings = bus.read(0x15)?;
  trig_settings = trig_settings | thresh as u32;
  bus.write(0x15, trig_settings)?;
  Ok(())
}

pub fn set_cube_top_threshold(bus : &mut IPBus, thresh : u8)
  -> Result<(), Box<dyn Error>> {
  info!("Setting cube top threshold {}!", thresh);
  let mut trig_settings = bus.read(0x15)?;
  trig_settings = trig_settings | ((thresh as u32) << 8);
  bus.write(0x15, trig_settings)?;
  Ok(())
}

pub fn set_cube_bottom_threshold(bus : &mut IPBus, thresh : u8)
  -> Result<(), Box<dyn Error>> {
  info!("Setting cube bottom threshold {}!", thresh);
  let mut trig_settings = bus.read(0x15)?;
  trig_settings = trig_settings | ((thresh as u32) << 16);
  bus.write(0x15, trig_settings)?;
  Ok(())
}

pub fn set_cube_corner_threshold(bus : &mut IPBus, thresh : u8)
  -> Result<(), Box<dyn Error>> {
  info!("Setting cube corner threshold {}!", thresh);
  let mut trig_settings = bus.read(0x15)?;
  trig_settings = trig_settings | ((thresh as u32) << 24);
  bus.write(0x15, trig_settings)?;
  Ok(())
}

pub fn set_umbrella_threshold(bus : &mut IPBus, thresh : u8)
  -> Result<(), Box<dyn Error>> {
  info!("Setting umbrella threshold {}!", thresh);
  let mut trig_settings = bus.read(0x16)?;
  trig_settings = trig_settings | thresh as u32;
  bus.write(0x16, trig_settings)?;
  Ok(())
}

pub fn set_cortina_threshold(bus : &mut IPBus, thresh : u8)
  -> Result<(), Box<dyn Error>> {
  info!("Setting cortina threshold {}!", thresh);
  let mut trig_settings = bus.read(0x16)?;
  trig_settings = trig_settings | ((thresh as u32) << 16);
  bus.write(0x16, trig_settings)?;
  Ok(())
}


/// The TIU emulation mode setting provides the possibility to run the 
/// TOF without tracker and active TIU. If data should be taken together
/// with the tracker, we should NOT use the emulation mode and this should
/// be set to false.
pub fn set_tiu_emulation_mode(bus : &mut IPBus, set_emulation_mode : bool) 
  -> Result<(), Box<dyn Error>> {
    info!("Setting TIU Emulation mode {}", set_emulation_mode);
    let mut value = bus.read(0xe)?;
    let bitset : u32;
    if set_emulation_mode {
      bitset = 0x1;
    } else {
      bitset = 0x0;
    }
    value = value & 0xfffffffe;
    value = value | bitset;
    bus.write(0xe, value)?;
    Ok(())
}


