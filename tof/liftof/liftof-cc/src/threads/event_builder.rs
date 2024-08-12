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

use comfy_table::modifiers::{
    UTF8_ROUND_CORNERS,
    UTF8_SOLID_INNER_BORDERS,
};
use comfy_table::presets::UTF8_FULL;
use comfy_table::*;

use tof_dataclasses::events::{
    MasterTriggerEvent,
    TofEvent,
    RBEvent
};

use tof_dataclasses::serialization::Packable;

use tof_dataclasses::packets::TofPacket;
//use tof_dataclasses::threading::ThreadControl;
use tof_dataclasses::events::EventStatus;



//use liftof_lib::heartbeat_printer;
use liftof_lib::settings::{
    TofEventBuilderSettings,
    //BuildStrategy,
};
use tof_dataclasses::config::BuildStrategy;
use liftof_lib::thread_control::ThreadControl;

use crate::constants::EVENT_BUILDER_EVID_CACHE_SIZE;

use colored::{
    Colorize,
};

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
                      run_id         : u32,
                      mtb_link_map   : HashMap<u8,u8>,
                      mut settings   : TofEventBuilderSettings,
                      thread_control : Arc<Mutex<ThreadControl>>) { 
  // missing event analysis
  //let mut event_id_test = Vec::<u32>::new();


  // debug the number of rb events we have seen 
  // in production mode, these features should go away
  // FIXEM - add debug flags to features
  let mut seen_rbevents      = HashMap::<u8, usize>::new();
  let mut too_early_rbevents = HashMap::<u8, usize>::new(); 
  // 10, 12, 37,38, 43, 45 don't exist
  for k in 1..47 {
    if k == 10 || k ==12 || k == 37 || k == 38 || k == 43 || k == 45 {
      continue;
    } else {
      seen_rbevents.insert(k as u8, 0);
      too_early_rbevents.insert(k as u8, 0);
    }
  }

  // event caches for assembled events
  let mut heartbeat            = EVTBLDRHeartbeat::new();
  let mut event_cache          = HashMap::<u32, TofEvent>::new();
  let mut event_id_cache       = VecDeque::<u32>::with_capacity(EVENT_BUILDER_EVID_CACHE_SIZE);
  //let mut idx_to_remove = Vec::<usize>::with_capacity(20);
  //let mut event_id_cache_a    = VecDeque:
  //let dsi_map                 = get_dsi_j_ltbch_vs_rbch_map(db_path.as_ref()); 
  let mut n_received           : usize;
  //let mut clear_cache        = 0; // clear cache every 
  //let mut event_sending      = 0;
  let mut n_mte_received_tot   = 0u64;
  let mut n_mte_skipped        = 0u32;
  let n_rbe_received_tot   = 0u64;
  let n_rbe_discarded_tot  = 0u64;
  let mut first_evid           : u32;
  let mut last_evid            = 0;
  let mut n_sent               = 0usize;
  let mut n_sent_ch_err        = 0usize;
  let n_timed_out          = 0usize; 
  // debug
  let mut last_rb_evid         = 0u32;
  let mut n_rbe_per_te         = 0usize;
  let rb_ev_wo_mte         = 0usize;
  let n_rbe_from_past      = 0usize;
  let n_rbe_orphan         = 0usize;
  let mut debug_timer          = Instant::now();
  let met_seconds          = 0f64;  
  //let mut n_receiving_errors  = 0;
  let mut check_tc_update      = Instant::now();
  let mut n_gathered_fr_cache  = 0usize;
  let mut misaligned_cache_err = 0usize; 
  let daq_reset_cooldown   = Instant::now();
  let reset_daq_flag       = false;
  let mut retire               = false;
  let mut hb_timer               = Instant::now(); 
  let mut hb_interval         = Duration::from_secs(settings.hb_send_interval as u64);
  loop {
    if check_tc_update.elapsed().as_secs() > 2 {
      //println!("= => [evt_builder] checkling tc..");

      let mut cali_still_active = false;
      match thread_control.try_lock() {
        Ok(mut tc) => {
          //println!("= => [evt_builder] {}", tc);
          if (!tc.thread_event_bldr_active) || tc.stop_flag {
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
      thread::sleep(Duration::from_secs(2));
      break;
    }
    n_received = 0;
    while n_received < settings.n_mte_per_loop {
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
          n_mte_received_tot += 1;
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
    //let mut attempts = 0usize;
    'main: while !ev_from_rb.is_empty() && n_received < settings.n_rbe_per_loop {
    // try to catch up
    //while !ev_from_rb.is_empty() && last_rb_evid < last_evid {
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
      settings.build_strategy == BuildStrategy::AdaptiveThorough {
      settings.n_rbe_per_loop = av_rb_ev.ceil() as usize;
      // if the rb in the pipeline get too long, catch up
      // and drain it a bit
      if ev_from_rb.len() > 1000 {
        settings.n_rbe_per_loop = ev_from_rb.len() - 500;
      }
      if settings.n_rbe_per_loop == 0 {
        // failsafe
        settings.n_rbe_per_loop = 40;
      }
    }
    if let BuildStrategy::AdaptiveGreedy = settings.build_strategy {
      settings.n_rbe_per_loop = av_rb_ev.ceil() as usize + settings.greediness as usize;
      if settings.n_rbe_per_loop == 0 {
        // failsafe
        settings.n_rbe_per_loop = 40;
      }
    }
    let debug_timer_elapsed = debug_timer.elapsed().as_secs_f64();
    //println!("Debug timer elapsed {}", debug_timer_elapsed);
    if debug_timer_elapsed > 35.0  {
      // missing event id check
      let mut evid_check = event_id_cache[0];
      let mut n_ev_wo_evid = 0usize;
      for _ in 0..event_id_cache.len() {
        if !event_id_cache.contains(&evid_check) {
          n_ev_wo_evid += 1;
        }
        evid_check += 1;
      }
      heartbeat.met_seconds += debug_timer_elapsed as usize;
      heartbeat.mte_receiver_cbc_len = m_trig_ev.len();
      heartbeat.rbe_receiver_cbc_len = ev_from_rb.len();
      heartbeat.tp_sender_cbc_len = data_sink.len();

      while hb_timer.elapsed() < hb_interval {};
      }

      while hb_timer.elapsed() >= hb_interval {
      let pack = heartbeat.pack();
      match data_sink.send(pack) {
        Err(err) => {
          error!("EVTBLDR Heartbeat sending failed! Err {}", err);
        }
        Ok(_)    => {
          debug!("Heartbeat sent <3 <3 <3");
        }
      }
      println!("{}", heartbeat);
      hb_timer = Instant::now();

      while hb_timer.elapsed() < hb_interval {};
    }
      let mut counters = HashMap::<u8,f64>::new();
      for k in seen_rbevents.keys() {
        counters.insert(*k, seen_rbevents[&k] as f64/met_seconds as f64);
      }
      let mut table = Table::new();
      table
        .load_preset(UTF8_FULL)
        .apply_modifier(UTF8_ROUND_CORNERS)
        .apply_modifier(UTF8_SOLID_INNER_BORDERS)
        .set_content_arrangement(ContentArrangement::Dynamic)
        .set_width(80)
        //.set_header(vec!["Readoutboard Rates:"])
        .add_row(vec![
            Cell::new(&(format!("RB01 {:.1} Hz", counters[&1]))),
            Cell::new(&(format!("RB02 {:.1} Hz", counters[&2]))),
            Cell::new(&(format!("RB03 {:.1} Hz", counters[&3]))),
            Cell::new(&(format!("RB04 {:.1} Hz", counters[&4]))),
            Cell::new(&(format!("RB05 {:.1} Hz", counters[&5]))),
            //Cell::new("Center aligned").set_alignment(CellAlignment::Center),
        ])
        .add_row(vec![
            Cell::new(&(format!("RB06 {:.1} Hz", counters[&6]))),
            Cell::new(&(format!("RB07 {:.1} Hz", counters[&7]))),
            Cell::new(&(format!("RB08 {:.1} Hz", counters[&8]))),
            Cell::new(&(format!("RB09 {:.1} Hz", counters[&9]))),
            Cell::new(&(format!("RB10 {}", "N.A."))),
        ])
        .add_row(vec![
            Cell::new(&(format!("RB11 {:.1} Hz", counters[&11]))),
            Cell::new(&(format!("RB12 {}", "N.A."))),
            Cell::new(&(format!("RB13 {:.1} Hz", counters[&13]))),
            Cell::new(&(format!("RB14 {:.1} Hz", counters[&14]))),
            Cell::new(&(format!("RB15 {:.1} Hz", counters[&15]))),
        ])
        .add_row(vec![
            Cell::new(&(format!("RB16 {:.1} Hz", counters[&16]))),
            Cell::new(&(format!("RB17 {:.1} Hz", counters[&17]))),
            Cell::new(&(format!("RB18 {:.1} Hz", counters[&18]))),
            Cell::new(&(format!("RB19 {:.1} Hz", counters[&19]))),
            Cell::new(&(format!("RB20 {:.1} Hz", counters[&20]))),
        ])
        .add_row(vec![
            Cell::new(&(format!("RB21 {:.1} Hz", counters[&21]))),
            Cell::new(&(format!("RB22 {:.1} Hz", counters[&22]))),
            Cell::new(&(format!("RB23 {:.1} Hz", counters[&23]))),
            Cell::new(&(format!("RB24 {:.1} Hz", counters[&24]))),
            Cell::new(&(format!("RB25 {:.1} Hz", counters[&25]))),
        ])
        .add_row(vec![
            Cell::new(&(format!("RB26 {:.1} Hz", counters[&26]))),
            Cell::new(&(format!("RB27 {:.1} Hz", counters[&27]))),
            Cell::new(&(format!("RB28 {:.1} Hz", counters[&28]))),
            Cell::new(&(format!("RB29 {:.1} Hz", counters[&29]))),
            Cell::new(&(format!("RB30 {:.1} Hz", counters[&30]))),
        ])
        .add_row(vec![
            Cell::new(&(format!("RB31 {:.1} Hz", counters[&31]))),
            Cell::new(&(format!("RB32 {:.1} Hz", counters[&32]))),
            Cell::new(&(format!("RB33 {:.1} Hz", counters[&33]))),
            Cell::new(&(format!("RB34 {:.1} Hz", counters[&34]))),
            Cell::new(&(format!("RB35 {:.1} Hz", counters[&35]))),
        ])
        .add_row(vec![
            Cell::new(&(format!("RB36 {:.1}", counters[&36]))),
            Cell::new(&(format!("RB37 {}", "N.A."))),
            Cell::new(&(format!("RB38 {}", "N.A."))),
            Cell::new(&(format!("RB39 {:.1}", counters[&39]))),
            Cell::new(&(format!("RB40 {:.1}", counters[&40]))),
        ])
        .add_row(vec![
            Cell::new(&(format!("RB41 {:.1}", counters[&41]))),
            Cell::new(&(format!("RB43 {:.1}", counters[&42]))),
            Cell::new(&(format!("RB42 {}", "N.A."))),
            Cell::new(&(format!("RB44 {:.1}", counters[&44]))),
            Cell::new(&(format!("RB45 {}", "N.A."))),
        ])
        .add_row(vec![
            Cell::new(&(format!("RB46 {:.1} Hz", counters[&46]))),
            Cell::new(&(format!("{}", "N.A."))),
            Cell::new(&(format!("{}", "N.A."))),
            Cell::new(&(format!("{}", "N.A."))),
            Cell::new(&(format!("{}", "N.A."))),
        ]);

      // Set the default alignment for the third column to right
      //let column = table.column_mut(2).expect("Our table has three columns");
      //column.set_cell_alignment(CellAlignment::Right);
      println!("{table}");
      println!("  {}",">> == == == == ==  END HEARTBEAT! == == == == == <<".bright_purple().bold());
      let mut table = Table::new();
      table
        .load_preset(UTF8_FULL)
        .apply_modifier(UTF8_ROUND_CORNERS)
        .apply_modifier(UTF8_SOLID_INNER_BORDERS)
        .set_content_arrangement(ContentArrangement::Dynamic)
        .set_width(80)
        //.set_header(vec!["Readoutboard Rates:"])
        .add_row(vec![
            Cell::new(&(format!("RB01 {:.1} Hz", too_early_rbevents[&1]))),
            Cell::new(&(format!("RB02 {:.1} Hz", too_early_rbevents[&2]))),
            Cell::new(&(format!("RB03 {:.1} Hz", too_early_rbevents[&3]))),
            Cell::new(&(format!("RB04 {:.1} Hz", too_early_rbevents[&4]))),
            Cell::new(&(format!("RB05 {:.1} Hz", too_early_rbevents[&5]))),
            //Cell::new("Center aligned").set_alitoo_early_rbeventsgnment(CellAlignment::Center),
        ])
        .add_row(vec![
            Cell::new(&(format!("RB06 {:.1} Hz", too_early_rbevents[&6]))),
            Cell::new(&(format!("RB07 {:.1} Hz", too_early_rbevents[&7]))),
            Cell::new(&(format!("RB08 {:.1} Hz", too_early_rbevents[&8]))),
            Cell::new(&(format!("RB09 {:.1} Hz", too_early_rbevents[&9]))),
            Cell::new(&(format!("RB10 {}", "N.A."))),
        ])
        .add_row(vec![
            Cell::new(&(format!("RB11 {:.1} Hz", too_early_rbevents[&11]))),
            Cell::new(&(format!("RB12 {}", "N.A."))),
            Cell::new(&(format!("RB13 {:.1} Hz", too_early_rbevents[&13]))),
            Cell::new(&(format!("RB14 {:.1} Hz", too_early_rbevents[&14]))),
            Cell::new(&(format!("RB15 {:.1} Hz", too_early_rbevents[&15]))),
        ])
        .add_row(vec![
            Cell::new(&(format!("RB16 {:.1} Hz", too_early_rbevents[&16]))),
            Cell::new(&(format!("RB17 {:.1} Hz", too_early_rbevents[&17]))),
            Cell::new(&(format!("RB18 {:.1} Hz", too_early_rbevents[&18]))),
            Cell::new(&(format!("RB19 {:.1} Hz", too_early_rbevents[&19]))),
            Cell::new(&(format!("RB20 {:.1} Hz", too_early_rbevents[&20]))),
        ])
        .add_row(vec![
            Cell::new(&(format!("RB21 {:.1} Hz", too_early_rbevents[&21]))),
            Cell::new(&(format!("RB22 {:.1} Hz", too_early_rbevents[&22]))),
            Cell::new(&(format!("RB23 {:.1} Hz", too_early_rbevents[&23]))),
            Cell::new(&(format!("RB24 {:.1} Hz", too_early_rbevents[&24]))),
            Cell::new(&(format!("RB25 {:.1} Hz", too_early_rbevents[&25]))),
        ])
        .add_row(vec![
            Cell::new(&(format!("RB26 {:.1} Hz", too_early_rbevents[&26]))),
            Cell::new(&(format!("RB27 {:.1} Hz", too_early_rbevents[&27]))),
            Cell::new(&(format!("RB28 {:.1} Hz", too_early_rbevents[&28]))),
            Cell::new(&(format!("RB29 {:.1} Hz", too_early_rbevents[&29]))),
            Cell::new(&(format!("RB30 {:.1} Hz", too_early_rbevents[&30]))),
        ])
        .add_row(vec![
            Cell::new(&(format!("RB31 {:.1} Hz", too_early_rbevents[&31]))),
            Cell::new(&(format!("RB32 {:.1} Hz", too_early_rbevents[&32]))),
            Cell::new(&(format!("RB33 {:.1} Hz", too_early_rbevents[&33]))),
            Cell::new(&(format!("RB34 {:.1} Hz", too_early_rbevents[&34]))),
            Cell::new(&(format!("RB35 {:.1} Hz", too_early_rbevents[&35]))),
        ])
        .add_row(vec![
            Cell::new(&(format!("RB36 {:.1}", too_early_rbevents[&36]))),
            Cell::new(&(format!("RB37 {}", "N.A."))),
            Cell::new(&(format!("RB38 {}", "N.A."))),
            Cell::new(&(format!("RB39 {:.1}", too_early_rbevents[&39]))),
            Cell::new(&(format!("RB40 {:.1}", too_early_rbevents[&40]))),
        ])
        .add_row(vec![
            Cell::new(&(format!("RB41 {:.1}", too_early_rbevents[&41]))),
            Cell::new(&(format!("RB43 {:.1}", too_early_rbevents[&42]))),
            Cell::new(&(format!("RB42 {}", "N.A."))),
            Cell::new(&(format!("RB44 {:.1}", too_early_rbevents[&44]))),
            Cell::new(&(format!("RB45 {}", "N.A."))),
        ])
        .add_row(vec![
            Cell::new(&(format!("RB46 {:.1} Hz", too_early_rbevents[&46]))),
            Cell::new(&(format!("{}", "N.A."))),
            Cell::new(&(format!("{}", "N.A."))),
            Cell::new(&(format!("{}", "N.A."))),
            Cell::new(&(format!("{}", "N.A."))),
        ]);

      // Set the default alignment for the third column to right
      //let column = table.column_mut(2).expect("Our table has three columns");
      //column.set_cell_alignment(CellAlignment::Right);
      println!("{table}");
      //println!("[EVTBLDR] ==> Last RB evid {last_rb_evid}");
      debug_timer = Instant::now(); 
    }
    trace!("Debug timer RBE received! {:?}", debug_timer.elapsed());
    //if event_sending == send_every_x_event {
    n_gathered_fr_cache = 0;
    let mut prior_ev_sent = 0u32;
    let mut first_ev_sent = false;
    
    for idx in 0..event_id_cache.len() {
      // if there wasn't a first element, size would be 0
      let evid = event_id_cache.pop_front().unwrap();
      // this is an alternative approach, but it seems much slower
      //let evid : u32;
      //match event_id_cache.get(idx) {
      //  None => {
      //    error!("Unable to get index {} from event_id_cache with len {}", idx, event_id_cache.len());
      //    continue;
      //  },
      //  Some(_evid) => {
      //    evid = *_evid;
      //  }
      //}
      match event_cache.get(&evid) {
        None => {
          error!("Event id and event caches are misaligned for event id {}, idx {} and sizes {} {}! This is BAD and most likely a BUG!", evid, idx, event_cache.len(), event_id_cache.len());
          //event_id_cache.push_back(evid);
          misaligned_cache_err += 1;
          continue;
        },
        Some(ev) => {
          let ev_timed_out = ev.age() >= settings.te_timeout_sec as u64;
          //let cache_it : bool;
          if ev_timed_out {
            if !ev.is_complete() {
              heartbeat.n_timed_out += 1;
            }
          }
          // always ready when the event is timed out
          let mut ready_to_send = ev_timed_out;
          if !ev_timed_out {
            // we are earlier than our time out, maybe the 
            // event is already complete
            match settings.build_strategy {
              BuildStrategy::WaitForNBoards => {
                // we will always wait for the expected number of boards, 
                // FIXME - make this a member of settings
                let _wait_nrb : usize = 40;
                // except the event times out
                if ev.rb_events.len() as u8 == settings.wait_nrb {
                  ready_to_send = true;
                  //n_gathered_fr_cache += 1;
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
            // update event status, so that we will also see in an 
            // (optionally) produced tof event summary if the 
            // event has isuses
            n_rbe_per_te  += ev_to_send.rb_events.len();
            heartbeat.data_mangled_ev = 69;
            let _ev_satus  = ev_to_send.mt_event.event_status;
            for ev in &ev_to_send.rb_events {
              if ev.status == EventStatus::CellSyncErrors || ev.status == EventStatus::ChnSyncErrors {
                ev_to_send.mt_event.event_status = EventStatus::AnyDataMangling {
                };
                heartbeat.data_mangled_ev += 1;
              }
            }
            // can we avoid unpacking and repacking?
            
            while hb_timer.elapsed() < hb_interval {
              // Do nothing, wait for the next heartbeat cycle
          }
          while hb_timer.elapsed() >= hb_interval {
            let pack       = TofPacket::from(&ev_to_send);
            match data_sink.send(pack) {
              Err(err) => {
                error!("Packet sending failed! Err {}", err);
                n_sent_ch_err += 1; 
              }
              Ok(_)    => {
                debug!("Event with id {} sent!", evid);
                n_sent += 1;
                heartbeat.n_sent += 1;
              }
            }
            hb_timer = Instant::now();
            // Wait until the configured interval has passed
            while hb_timer.elapsed() < hb_interval {};
          } 
        } else {
            event_id_cache.push_front(evid);
          }
        }
      }
    } 
    // end loop over event_id_cache
    // this is related to the above way to deal with the
    // event_id_cache. But it might be too slow.
    //let mut to_remove : usize;
    //let mut idx_mod   = 0usize;
    //for idx in &idx_to_remove {
    //  to_remove = idx - idx_mod;
    //  event_id_cache.remove(to_remove);
    //  idx_mod += 1;
    //}
    //idx_to_remove.clear();
    //event_sending = 0;
    //event_cache.retain(|ev| ev.valid);
    debug!("Debug timer! EVT SENDING {:?}", debug_timer.elapsed());
    } // end loop
  

