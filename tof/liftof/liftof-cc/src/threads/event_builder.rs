//! The Heart of lfitof-cc. The event builder assembles all 
//! events coming from the Readoutboards in a single event

use std::thread;
use std::time::{
  Instant,
  Duration
};

use std::sync::{
  Arc,
  Mutex
};

use std::collections::VecDeque;
use std::collections::HashMap;
//use std::path::Path;
//
use crossbeam_channel::{
  Receiver,
  Sender,
};


use tof_dataclasses::events::{
  MasterTriggerEvent,
  TofEvent,
  RBEvent
};

use tof_dataclasses::serialization::Packable;

use tof_dataclasses::packets::TofPacket;
use tof_dataclasses::events::EventStatus;

//use liftof_lib::heartbeat_printer;
use liftof_lib::settings::{
  TofEventBuilderSettings,
};
use tof_dataclasses::commands::config::BuildStrategy;
use liftof_lib::thread_control::ThreadControl;

use crate::constants::EVENT_BUILDER_EVID_CACHE_SIZE;

use tof_dataclasses::heartbeats::EVTBLDRHeartbeat;



/// Events ... assemble! 
///
/// The event_builder collects all available event information,
/// beginning with the MasterTriggerEvent defining the event 
/// id. It collects the requested number of RBEvents.
/// The final product then will be a TofEvent
///
/// The event_builder is the heart of this software and crucial
/// to all operations.
///
/// # Arguments
///
/// * m_trig_ev      : Receive a `MasterTriggerEvent` over this 
///                    channel. The event will be either build 
///                    immediatly, or cached. 
/// * ev_from_rb     : Receive a number of `RBEvents` over this channel.
///                    The events here shall be associated with the 
///                    MasterTriggerEvent
/// * data_sink      : Send assembled events (and everything else in 
///                    the form of TofPackets to the data sink/
/// * mtb_link_map   : Map of MTB Link ID - RB ID. Maybe in the future
///                    RBs will know their link id themselves?
///                    This is currently only needed for the build strategy
///                    "AdaptiveThorough"
/// * settings       : Configure the event builder
pub fn event_builder (m_trig_ev      : &Receiver<MasterTriggerEvent>,
                      ev_from_rb     : &Receiver<RBEvent>,
                      data_sink      : &Sender<TofPacket>,
                      mtb_link_map   : HashMap<u8,u8>,
                      thread_control : Arc<Mutex<ThreadControl>>) { 
  // missing event analysis
  //let mut event_id_test = Vec::<u32>::new();

  // debug the number of rb events we have seen 
  // in production mode, these features should go away
  // FIXEM - add debug flags to features
  let mut seen_rbevents      = HashMap::<u8, u64>::new();
  let mut too_early_rbevents = HashMap::<u8, u64>::new(); 
  // 10, 12, 37,38, 43, 45 don't exist
  for k in 1..51 {
    if k == 10 || k ==12 || k == 37 || k == 38 || k == 43 || k == 45 {
      continue;
    } else {
      seen_rbevents.insert(k as u8, 0);
      too_early_rbevents.insert(k as u8, 0);
    }
  }
  
  // set up the event builder. Since we are now doing settings only at run 
  // start, it is fine to do this outside of the loop
  let mut send_tev_sum    : bool;
  let mut send_rbwaveform : bool;
  let mut send_rbwf_freq  : u32;
  let mut rbwf_ctr          = 0u64;
  let mut settings          : TofEventBuilderSettings;
  let mut run_id             : u32;
  // this can block it is fine bc it is only 
  // happening once at init
  let mut cali_active : bool;
  loop {
    match thread_control.lock() {
      Ok(tc) => {
        send_rbwaveform   = tc.liftof_settings.data_publisher_settings.send_rbwaveform_packets;
        send_tev_sum      = tc.liftof_settings.data_publisher_settings.send_tof_summary_packets;
        send_rbwf_freq    = tc.liftof_settings.data_publisher_settings.send_rbwf_every_x_event;
        settings          = tc.liftof_settings.event_builder_settings.clone();
        run_id            = tc.run_id;
        cali_active       = tc.calibration_active;
      }
      Err(err) => {
        error!("Can't acquire lock for ThreadControl! {err}");
        error!("CRITICAL: Unable to configure event builder thread! Aborting!");
        return;
      }
    }
    if !cali_active {
      break;
    } else {
      thread::sleep(Duration::from_secs(4));
    }
  }
  info!("Will assign run id {} to events!", run_id);

  // event caches for assembled events
  let mut heartbeat            = EVTBLDRHeartbeat::new();
  let mut event_cache          = HashMap::<u32, TofEvent>::new();
  let mut event_id_cache       = VecDeque::<u32>::with_capacity(EVENT_BUILDER_EVID_CACHE_SIZE);
  let mut n_received           : usize;
  let mut n_mte_skipped        = 0u32;
  let mut first_evid           : u32;
  let mut last_evid            = 0;
  let mut n_sent               = 0usize;
  // debug
  let mut last_rb_evid         : u32;
  let mut n_rbe_per_te         = 0usize;
  let mut debug_timer          = Instant::now();
  let met_seconds              = 0f64;  
  //let mut n_receiving_errors  = 0;
  let mut check_tc_update      = Instant::now();
  let daq_reset_cooldown       = Instant::now();
  let reset_daq_flag           = false;
  let mut retire               = false;
  let mut hb_timer             = Instant::now(); 
  let hb_interval              = Duration::from_secs(settings.hb_send_interval as u64);
  

  loop {
    if check_tc_update.elapsed().as_secs() > 2 {
      //println!("= => [evt_builder] checkling tc..");

      let mut cali_still_active = false;
      match thread_control.try_lock() {
        Ok(mut tc) => {
          if tc.thread_event_bldr_active {
            println!("= => [evt_builder] shutting down...");
            break; 
          }
          //println!("= => [evt_builder] {}", tc);
          if tc.stop_flag {
            // end myself
            println!("= => [evt_builder] shutting down...");
            retire = true;
          }
          //println!("== ==> [evt_builder] tc lock acquired!");
          if tc.calibration_active {
            cali_still_active = true;
          } else {
            cali_still_active = false;  
          }
          if daq_reset_cooldown.elapsed().as_secs_f32() > 120.0 && reset_daq_flag {
            warn!("Resetttign MTB DAQ queue!");
            tc.reset_mtb_daq = true;
          }
        },
        Err(err) => {
          error!("Can't acquire lock for ThreadControl! Unable to set calibration mode! {err}");
        },
      }
      check_tc_update = Instant::now();
      if cali_still_active {
        thread::sleep(Duration::from_secs(1));
        continue;
      }
    }
    if retire {
      //thread::sleep(Duration::from_secs(2));
      break;
    }
    n_received = 0;
    while n_received < settings.n_mte_per_loop as usize {
      // every iteration, we welcome a new master event
      match m_trig_ev.try_recv() {
        Err(_) => {
          trace!("No new event ready yet!");
          //n_receiving_errors += 1;
          continue;
        }   
        Ok(mt) => {
          debug!("Received MasterTriggerEvent {}!", mt);
          let mut event = TofEvent::from(mt);
          event.header.run_id = run_id;
          if last_evid != 0 {
            if event.mt_event.event_id != last_evid + 1 {
              //let delta_id = event.mt_event.event_id - last_evid;
              if event.mt_event.event_id > last_evid {
                n_mte_skipped += event.mt_event.event_id - last_evid - 1;
              }
              //error!("We skipped event ids {}", delta_id );
            }
          }
          heartbeat.n_mte_skipped = n_mte_skipped as usize;
          last_evid = event.mt_event.event_id;
          event_cache.insert(last_evid, event);
          // use this to keep track of the order
          // of events
          event_id_cache.push_back(last_evid);
          n_received  += 1;
          heartbeat.n_mte_received_tot += 1;
        }
      } // end match Ok(mt)
      //if n_received % 10 == 0 {
      //  println!("==> Received 10 more MasterTriggerEvents");
      //}
    } // end getting MTEvents
    trace!("Debug timer MTE received! {:?}", debug_timer.elapsed());
    first_evid = event_id_cache[0]; 
    // recycle that variable for the rb events as well
    n_received = 0;
    'main: while !ev_from_rb.is_empty() && n_received < settings.n_rbe_per_loop as usize {
      match ev_from_rb.try_recv() {
        Err(err) => {
          error!("Can't receive RBEvent! Err {err}");
        },
        Ok(rb_ev) => {
          // FIXME - this is technically a bit risky, but 
          // mt event should arrive so much earlier (seconds)
          // I hope it won't be a problem. Otherwise we have
          // to add another cache.
          //println!("==> Len evt cache {}", event_cache.len());
          heartbeat.event_cache_size    = event_cache.len();
          heartbeat.event_id_cache_size = event_id_cache.len();
          heartbeat.n_rbe_received_tot += 1;
          n_received += 1;
          match seen_rbevents.get_mut(&rb_ev.header.rb_id) {
            Some(value) => {
              *value += 1;
            }
            None => {
              warn!("Unable to do bookkeeping for RB {}", rb_ev.header.rb_id);
            }
          }
          //iter_ev = 0;
          last_rb_evid = rb_ev.header.event_id;
          if last_rb_evid < first_evid {
            n_received -= 1;
            debug!("The received RBEvent {} is from the ancient past! Currently, we don't have a way to deal with that and this event will be DISCARDED! The RBEvent queue will be re-synchronized...", last_rb_evid);
            //attempts += 1;
            heartbeat.n_rbe_discarded_tot += 1;
            heartbeat.n_rbe_from_past += 1;
            *too_early_rbevents.get_mut(&rb_ev.header.rb_id).unwrap() += 1;
            continue;
          }
          match event_cache.get_mut(&last_rb_evid) {
            None => {
              //FIXME - big issue!
              //println!("Surprisingly we don't have that!");
              // insert a new TofEvent
              //let new_ev = TofEvent::new();
              heartbeat.rbe_wo_mte += 1;
              heartbeat.n_rbe_discarded_tot += 1;
              heartbeat.n_rbe_orphan += 1;
              //error!("No MTEvent for RBEvent. rb event id {}, first mte {}, last mte {}", last_rb_evid, first_evid, last_cache_evid);
              continue 'main;
            },
            Some(ev) => {
              if settings.build_strategy == BuildStrategy::AdaptiveThorough {
                match mtb_link_map.get(&rb_ev.header.rb_id) {
                  None => {
                    error!("Don't know MTB Link ID for {}", rb_ev.header.rb_id)
                  }
                  Some(link_id) => {
                    if ev.mt_event.get_rb_link_ids().contains(link_id) {
                      ev.rb_events.push(rb_ev);
                    } else {
                      error!("MT Event {}", ev.mt_event);
                      error!("RBEvent {} has the same event id, but does not show up in MTB Link ID mask!", rb_ev);
                    }
                  }
                }
              } else {
                // Just ad it without questioning
                ev.rb_events.push(rb_ev);
                //println!("[EVTBUILDER] DEBUG n rb expected : {}, n rbs {}",ev.mt_event.get_n_rbs_expected(), ev.rb_events.len());
              }
              //break;
            }
          }
        }
      }
    }
    let av_rb_ev = n_rbe_per_te as f64 / n_sent as f64;
    if settings.build_strategy == BuildStrategy::Adaptive || 
      settings.build_strategy  == BuildStrategy::AdaptiveThorough {
      settings.n_rbe_per_loop  = av_rb_ev.ceil() as u32;
      // if the rb in the pipeline get too long, catch up
      // and drain it a bit
      if ev_from_rb.len() > 1000 {
        settings.n_rbe_per_loop = ev_from_rb.len() as u32 - 500;
      }
      if settings.n_rbe_per_loop == 0 {
        // failsafe
        settings.n_rbe_per_loop = 40;
      }
    }
    if let BuildStrategy::AdaptiveGreedy = settings.build_strategy {
      settings.n_rbe_per_loop = av_rb_ev.ceil() as u32 + settings.greediness as u32;
      if settings.n_rbe_per_loop == 0 {
        // failsafe
        settings.n_rbe_per_loop = 40;
      }
    }
    let debug_timer_elapsed = debug_timer.elapsed().as_secs_f64();
    //println!("Debug timer elapsed {}", debug_timer_elapsed);
    if debug_timer_elapsed > 35.0  {
      debug_timer = Instant::now(); 
    }
    trace!("Debug timer RBE received! {:?}", debug_timer.elapsed());
    //if event_sending == send_every_x_event {
    let mut prior_ev_sent = 0u32;
    let mut first_ev_sent = false;
   

    for idx in 0..event_id_cache.len() {
      // if there wasn't a first element, size would be 0
      let evid = event_id_cache.pop_front().unwrap();
      match event_cache.get(&evid) {
        None => {
          error!("Event id and event caches are misaligned for event id {}, idx {} and sizes {} {}! This is BAD and most likely a BUG!", evid, idx, event_cache.len(), event_id_cache.len());
          continue;
        },
        Some(ev) => {
          let ev_timed_out = ev.age() >= settings.te_timeout_sec as u64;
          // timed out events should be sent in any case
          let mut ready_to_send = ev_timed_out;
          if ev_timed_out {
            if !ev.is_complete() {
              heartbeat.n_timed_out += 1;
            }
          } else {
            // we are earlier than our time out, maybe the 
            // event is already complete
            match settings.build_strategy {
              BuildStrategy::WaitForNBoards => {
                // we will always wait for the expected number of boards, 
                // except the event times out
                if ev.rb_events.len() as u8 == settings.wait_nrb {
                  ready_to_send = true;
                } // else ready_to_send is still false 
              },
              BuildStrategy::Adaptive 
              | BuildStrategy::AdaptiveThorough
              | BuildStrategy::AdaptiveGreedy
              | BuildStrategy::Smart 
              | BuildStrategy::Unknown => {
                if ev.is_complete() {
                  ready_to_send = true;
                }
              }
            }
          } 
          // this feature tries to sort the events which are getting sent
          // by id. This might lead to timed out events and more resources needed
          if settings.sort_events {
            if ready_to_send && !ev_timed_out {
              if idx == 0 {
                first_ev_sent = true;
                prior_ev_sent = ev.header.event_id;
              } else {
                if idx == 1 {
                  if !first_ev_sent {
                    // we wait and check the others too see if something 
                    // else timed out
                    ready_to_send = false;
                  }
                }
                if ev.header.event_id != (prior_ev_sent + 1) {
                  // we wait and check the others too see if something 
                  // else timed out
                  ready_to_send = false;
                }
                prior_ev_sent = ev.header.event_id;
              }
            }
          }
          if ready_to_send {
            // if we don't cache it, we have to send it. 
            //let ev_to_send = ev.clone();
            // so the idea here is that this happens way less 
            // often (a few times per main loop iteration)
            // than the cache it case, so we rather do something
            // here even if it might require re-allocating memory
            // we should have an eye on performance though
            //idx_to_remove.push(idx);
            let mut ev_to_send = event_cache.remove(&evid).unwrap();
            if ev_timed_out {
              ev_to_send.mt_event.event_status = EventStatus::EventTimeOut;
            }
            // update event status, so that we will also see in an 
            // (optionally) produced tof event summary if the 
            // event has isuses
            n_rbe_per_te  += ev_to_send.rb_events.len();
            if ev_to_send.has_any_mangling() {
              heartbeat.data_mangled_ev += 1;
            }
            // sum up lost hits due to drs4 deadtime
            heartbeat.drs_bsy_lost_hg_hits += ev_to_send.get_lost_hits() as usize;

            // This only works if we are sending TofEventSummary
            // since it relies on quantities only calculated 
            // in the TofEventSummary packet 
            let mut save_to_disk = true;
            n_sent += 1;
            heartbeat.n_sent += 1;
            if send_tev_sum {
              let tes  = ev_to_send.get_summary();
              if settings.only_save_interesting {
                save_to_disk = false;
                if tes.n_hits_umb   >= settings.thr_n_hits_umb 
                && tes.n_hits_cbe   >= settings.thr_n_hits_cbe
                && tes.n_hits_cor   >= settings.thr_n_hits_cor
                && tes.tot_edep_umb >= settings.thr_tot_edep_umb
                && tes.tot_edep_cbe >= settings.thr_tot_edep_cbe
                && tes.tot_edep_cor >= settings.thr_tot_edep_cor {
                  save_to_disk = true;
                }
              }
              let pack = tes.pack();
              match data_sink.send(pack) {
                Err(err) => {
                  error!("Packet sending failed! Err {}", err);
                }
                Ok(_)    => {
                  debug!("Event with id {} sent!", evid);
                }
              }
            }

            //if 
            if send_rbwaveform {
              if rbwf_ctr == send_rbwf_freq as u64 {
                for wf in ev_to_send.get_rbwaveforms() {
                  let pack = wf.pack();
                  match data_sink.send(pack) {
                    Err(err) => {
                      error!("Packet sending failed! Err {}", err);
                    }
                    Ok(_)    => {
                      debug!("Event with id {} sent!", evid);
                    }
                  }
                }
                rbwf_ctr = 0;
              }
              rbwf_ctr += 1; // increase for every event, not wf
            }
            
            // always sent TofEvents, so they get written to disk.
            // There is one exception though, in case we have 
            // "interesting" event cuts in place, then this can 
            // be restricted.
            if save_to_disk {
              let pack = ev_to_send.pack();
              match data_sink.send(pack) {
                Err(err) => {
                  error!("Packet sending failed! Err {}", err);
                }
                Ok(_)    => {
                  debug!("Event with id {} sent!", evid);
                }
              }
            } 
          // this happens when we are NOT ready to send -> cache it!
          } else { 
            event_id_cache.push_front(evid);
          }
        }
      }
    } 
    // end loop over event_id_cache
    let met_elapsed = hb_timer.elapsed();
    if hb_timer.elapsed() >= hb_interval {
      heartbeat.mte_receiver_cbc_len = m_trig_ev.len();
      heartbeat.rbe_receiver_cbc_len = ev_from_rb.len();
      heartbeat.tp_sender_cbc_len    = data_sink.len();

      //println!("{}", heartbeat);
      let mut counters = HashMap::<u8,u64>::new();
      if met_seconds > 0.0 {
        for k in seen_rbevents.keys() {
          counters.insert(*k, (seen_rbevents[&k] as f64/met_seconds as f64).floor() as u64);
        }
      }
      //let mut table = rb_table(&counters, true); 
      //println!("{table}");
      //
      //table = rb_table(&too_early_rbevents, false);
      //println!("{table}");
      
      heartbeat.met_seconds += met_elapsed.as_secs_f64() as usize;
      let pack         = heartbeat.pack();
      match data_sink.send(pack) {
        Err(err) => {
          error!("Packet sending failed! Err {}", err);
        }
        Ok(_)    => {
        }
      }
      hb_timer = Instant::now();
    } 
  } // end loop
}  

