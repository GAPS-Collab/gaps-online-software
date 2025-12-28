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

use crossbeam_channel::{
  Receiver,
  Sender,
};

use gondola_core::prelude::*;

use crate::constants::EVENT_BUILDER_EVID_CACHE_SIZE;

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
pub fn event_builder (m_trig_ev      : &Receiver<TofEvent>,
                      ev_from_rb     : &Receiver<RBEvent>,
                      orphanage      : &Sender<RBEvent>,
                      data_sink      : &Sender<TofPacket>,
                      data_sink_ev   : &Sender<TofEvent>,
                      mtb_link_map   : HashMap<u8,u8>,
                      dead_rb_ids    : Vec<u8>,
                      thread_control : Arc<Mutex<ThreadControl>>) { 
  // set up the event builder. Since we are now doing settings only at run 
  // start, it is fine to do this outside of the loop
  //let mut send_tev_sum    : bool;
  //let mut send_rbwaveform : bool;
  //let mut send_rbwf_freq  : u32;
  //let mut rbwf_ctr        = 0u64;
  let mut settings        : TofEventBuilderSettings;
  let mut run_id          : u32;
  // this can block it is fine bc it is only 
  // happening once at init
  let mut cali_active : bool;
  // two trigger types are possible, make the time out depending on them 
  //let mut primary_trigger   : TriggerType;
  let mut combo_trigger : TriggerType;
  // these are for debugging as long as we don't have the correct link ids working 
  // these should be +1 for orphans when they are dropped or in case of the link_ids 
  // when the events time out
  let mut weird_orphan_rbids    = HashMap::<u8, usize>::new();
  //let mut weird_rbe_link_ids    = HashMap::<u8, usize>::new();
  let mut dead_rbs    : Option<(&Vec<u8>, &DsiJChRbMapping)> = None;
  let mut no_expect_dead_rbs    : bool;
  loop {
    match thread_control.lock() {
      Ok(tc) => {
        settings           = tc.liftof_settings.event_builder_settings.clone();
        run_id             = tc.run_id;
        cali_active        = tc.calibration_active;
        //primary_trigger   = tc.liftof_settings.mtb_settings.trigger_type;
        combo_trigger      = tc.liftof_settings.mtb_settings.global_trigger_type;
        no_expect_dead_rbs = tc.liftof_settings.event_builder_settings.no_expect_dead_rbs.unwrap_or(false);
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
  let paddles   = TofPaddle::all().unwrap_or(Vec::<TofPaddle>::new());
  let dsijrbmap = get_dsi_j_ch_rb_map(&paddles); 
  //let all_rbs   = ReadoutBoard::all().unwrap_or(Vec::<ReadoutBoard>::new()); 
  //let exprbmap  = get_linkid_rbid_map(&all_rbs);
  if no_expect_dead_rbs {  
    dead_rbs = Some((&dead_rb_ids, &dsijrbmap));
  }

  // event caches for assembled events
  let mut heartbeat            = EventBuilderHB::new();
  let mut event_cache          = HashMap::<u32, TofEvent>::new();
  let mut event_id_cache       = VecDeque::<u32>::with_capacity(EVENT_BUILDER_EVID_CACHE_SIZE);
  let mut n_received           : usize;
  let mut last_evid            = 0;
  //let mut n_sent               = 0usize;
  // debug
  let mut last_rb_evid         : u32;
  let mut n_rbe_per_te         = 0usize;
  //let mut debug_timer          = Instant::now();
  let mut check_tc_update      = Instant::now();
  let daq_reset_cooldown       = Instant::now();
  let reset_daq_flag           = false;
  let mut retire               = false;
  let mut hb_timer             = Instant::now(); 
  let hb_interval              = Duration::from_secs(settings.hb_send_interval as u64);

  //let mut debug_orphans        = Vec::<RBEvent>::new();

  // holdoff, just empty the channels, until we are confident to start 
  if let Some(ho) = settings.holdoff {
    // orphans, stfu
    while hb_timer.elapsed().as_secs() < ho as u64 {
      while !m_trig_ev.is_empty() {
        let _foo = m_trig_ev.try_recv();
      }
      while !ev_from_rb.is_empty() {
        let _bar = ev_from_rb.try_recv();
      }
    }
    println!("=> EvtBldr starting m_trig_ev  len {}", m_trig_ev.len());
    println!("=> EvtBldr starting ev_from_rb len {}", ev_from_rb.len());
    println!("=> EvtBldr passed holdoff time of {}", ho);
  }
  //------- DEBUG -- Measure the timing of the different parts 
  //------- of the loop
  //let mut mt_loop_time     = Instant::now();
  //let mut avg_mt_loop_time = 0u128;
  //let mut n_iter_mt_loop   = 0usize;
  //let mut rb_loop_time     = Instant::now();
  //let mut avg_rb_loop_time = 0u128;
  //let mut n_iter_rbe_loop  = 0usize;
  let n_rbe_per_loop_default = settings.n_rbe_per_loop; 
  loop {
    if check_tc_update.elapsed().as_secs() > 2 {
      //println!("= => [evt_builder] checkling tc..");

      let mut cali_still_active = false;
      match thread_control.try_lock() {
        Ok(mut tc) => {
          if !tc.thread_event_bldr_active {
            //println!("= => [evt_builder] (thread_event_bldr_active == false) shutting down...");
            continue; 
          }
          //println!("= => [evt_builder] {}", tc);
          if tc.stop_flag {
            // end myself
            println!("= => [evt_builder] (stop_flag == true) shutting down...");
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
      //mt_loop_time     = Instant::now(); 
      // every iteration, we welcome a new master event
      //mt_loop_time = Instant::now(); 
      //if m_trig_ev.is_empty() {
      //  continue;
      //}
      // have that deliberatly blocking
      match m_trig_ev.try_recv() {
      //match m_trig_ev.try_recv() {
        Err(_) => {
          trace!("No new event ready yet!");
          //n_receiving_errors += 1;
          continue;
        }   
        Ok(mut event) => {
          debug!("Received MasterTriggerEvent {}!", event);
          event.run_id = run_id as u16; // FIXME - might be too big
          if last_evid != 0 {
            if event.event_id != last_evid + 1 {
              if event.event_id > last_evid {
                heartbeat.n_mte_skipped += (event.event_id - last_evid - 1) as u32;
              }
            }
          }
          last_evid = event.event_id;
          event_cache.insert(last_evid, event);
          // use this to keep track of the order
          // of events
          event_id_cache.push_back(last_evid);
          n_received  += 1;
          heartbeat.n_mte_received_tot += 1;
        }
      } // end match Ok(mt)
      //avg_mt_loop_time += mt_loop_time.elapsed().as_nanos();
      //n_iter_mt_loop += 1;
    } // end getting MTEvents
    //trace!("Debug timer MTE received! {:?}", debug_timer.elapsed());
    // recycle that variable for the rb events as well
    n_received = 0;
    // The second receiver gets RBEvents from all ReadoutBoards. ReadoutBoard events are 
    // NOT cached by design. The assumption here is that due to caching on the RBs and the 
    // longer pathway (harting cable + ethernet cables) and DRS and user time, RBEvents are 
    // ALWAYS later than the MTEvents.
    'main: while !ev_from_rb.is_empty() && n_received < settings.n_rbe_per_loop as usize {
      
      //rb_loop_time = Instant::now();
      match ev_from_rb.try_recv() {
        Err(err) => {
          error!("Can't receive RBEvent! Err {err}");
        },
        Ok(rb_ev) => {
          heartbeat.n_rbe_received_tot += 1;
          n_received += 1;
          if rb_ev.status == EventStatus::RBEventWacky {
            continue;
          }

          //match seen_rbevents.get_mut(&rb_ev.header.rb_id) {
          //  Some(value) => {
          //    *value += 1;
          //  }
          //  None => {
          //    warn!("Unable to do bookkeeping for RB {}", rb_ev.header.rb_id);
          //  }
          //}
          //iter_ev = 0;
          last_rb_evid = rb_ev.header.event_id;
          // try to asscociate the rb events with the mtb events
          // the event ids from the RBEvents have to be in the 
          // range of the MTB Event
          // The event_id_cache is sorted, that is why it works
          if last_rb_evid < event_id_cache[0] {
            // this is the first check. If this fails, then the event is for 
            // sure not in the event_cache and we can dismiss it right away,
            // knowing that it is from the past
            n_received -= 1;
            debug!("The received RBEvent {} is from the ancient past! Currently, we don't have a way to deal with that and this event will be DISCARDED! The RBEvent queue will be re-synchronized...", last_rb_evid);
            heartbeat.n_rbe_discarded_tot += 1;
            heartbeat.n_rbe_from_past     += 1;
            //*too_early_rbevents.get_mut(&rb_ev.header.rb_id).unwrap() += 1;
            continue;
          }
          // Now try to get the master trigger event for 
          // this RBEvent
          match event_cache.get_mut(&last_rb_evid) {
            None => {
              if let Some(backend_evid) = event_id_cache.back() { 
                if last_rb_evid < *backend_evid {
                  // we know that this is neither too late nor too early!
                  heartbeat.rbe_wo_mte          += 1;
                }
                //debug_orphans.push(rb_ev);
                //let orphan_pack = rb_ev.pack();
                //writer.add_tof_packet(&orphan_pack);
                if rb_ev.creation_time.is_some() {
                  if rb_ev.creation_time.unwrap().elapsed().as_secs() > 300 {
                    let delta_evid = last_rb_evid - backend_evid;
                    error!("We can't associate event id {} from RB {} with a MTEvent in range {} .. {}. It is {} event ids ahead !", last_rb_evid, rb_ev.header.rb_id, event_id_cache[0], backend_evid, delta_evid);
                    debug!("{}", rb_ev);
                    warn!("Orphan could not be adopted within 5 mins. Kicking them out!");
                    heartbeat.n_rbe_discarded_tot += 1;
                    heartbeat.n_rbe_orphan        += 1;
                    continue 'main
                  }
                }
                match orphanage.send(rb_ev) {
                  Ok(_) => (),
                  Err(err) => {
                    error! ("Orphanage does not accept this orphan. They are dying in the gutter all by themselves. What a said world! {err}");
                  }
                }
              }
              continue 'main;
            },
            Some(ev) => {
              if settings.build_strategy == BuildStrategy::AdaptiveThorough {
                match mtb_link_map.get(&rb_ev.header.rb_id) {
                  None => {
                    error!("Don't know MTB Link ID for {}", rb_ev.header.rb_id);
                    error!("This RBEvent gets discarded!");
                  }
                  Some(link_id) => {
                    if ev.get_rb_link_ids().contains(link_id) {
                      ev.rb_events.push(rb_ev);
                    } else {
                      error!("RBEvent {} has the same event id, but does not show up in MTB Link ID mask!", rb_ev);
                    }
                  }
                }
              } else {
                // Just ad it without questioning
                //println!("PUSHING NEW RG EVENT WITH {} HITS", rb_ev.hits.len());
                ev.rb_events.push(rb_ev);
                //println!("[EVTBUILDER] DEBUG n rb expected : {}, n rbs {}",ev.mt_event.get_n_rbs_expected(), ev.rb_events.len());
              }
              //break;
            }
          }
        }
      }
      //thread::sleep(Duration::from_nanos(200)); 
      //avg_rb_loop_time += rb_loop_time.elapsed().as_nanos();
      //n_iter_rbe_loop  += 1;
    }
    // FIXME - timing debugging
    //let debug_timer_elapsed = debug_timer.elapsed().as_secs_f64();
    ////println!("Debug timer elapsed {}", debug_timer_elapsed);
    //if debug_timer_elapsed > 90.0  {
    //  debug_timer = Instant::now(); 
    //  let mut file = File::create("event_id_cache.txt").unwrap();
    //  let mut file2 = File::create("orphans.txt").unwrap();
    //  let content = format!("{:?}", event_id_cache);
    //  //let content = "This is a line of text.\nAnother line.";
    //  file.write_all(content.as_bytes()); // write_all expects a byte slice
    //  let mut content_rbs = String::new();
    //  for k in &debug_orphans {
    //    content_rbs += &format!("{}", k);
    //  }
    //  file2.write_all(content_rbs.as_bytes()); // write_al
    //
    //}
    //trace!("Debug timer RBE received! {:?}", debug_timer.elapsed());

    // -----------------------------------------------------
    // ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^
    // This concludes the actually "event building" part
    // -----------------------------------------------------

    let av_rb_ev = n_rbe_per_te as f64 / heartbeat.n_sent as f64;
    if settings.build_strategy == BuildStrategy::Adaptive || 
      settings.build_strategy  == BuildStrategy::AdaptiveThorough {
      //settings.n_rbe_per_loop  = av_rb_ev.ceil() as u32;
      settings.n_rbe_per_loop  = av_rb_ev.floor() as u32;
    }
    if let BuildStrategy::AdaptiveGreedy = settings.build_strategy {
      settings.n_rbe_per_loop = av_rb_ev.ceil() as u32 + settings.greediness as u32;
      if settings.n_rbe_per_loop == 0 {
        // failsafe
        settings.n_rbe_per_loop = 40;
      }
    }


    //-----------------------------------------
    // From here on, we are checking all events
    // in the cache, deciding which ones are 
    // ready to be passed on
    // ----------------------------------------

    let mut prior_ev_sent = 0u32;
    let mut first_ev_sent = false;
    let timeout           = settings.te_timeout_sec as u64;  
    // For the combo trigger, we can have a different timeout
    // if it is not set, we use the same timeout as for the primary trigger
    let combo_timeout = settings.te_timeout_sec_combo.unwrap_or(settings.te_timeout_sec) as u64;
    let n_iter_max = event_id_cache.len() / 2;
    for idx in 0..n_iter_max {
      // if there wasn't a first element, size would be 0
      let evid = event_id_cache.pop_front().unwrap();
      match event_cache.get_mut(&evid) {
        None => {
          error!("Event id and event caches are misaligned for event id {}, idx {} and sizes {} {}! This is BAD and most likely a BUG!", evid, idx, event_cache.len(), event_id_cache.len());
          continue;
        },
        Some(ev) => {
          let mut ev_timed_out = ev.age() >= timeout;
          let mut is_combo     = false;
          for trg in &ev.get_trigger_sources() {
            // the combo trigger should superseed the primary trigger and only if 
            // we have the secondary trigger, we will apply the other timeout 
            if trg == &combo_trigger {
              ev_timed_out = ev.age() >= combo_timeout;
              is_combo     = true;
              break;
            }
          }
          // timed out events should be sent in any case
          let is_complete = ev.is_complete(dead_rbs);
          //let mut ready_to_send = ev_timed_out || is_complete;
          let mut ready_to_send : bool;
          // mangled events will be treated seperatly
          if ev_timed_out && !is_complete && !ev.has_any_mangling() {
            //if !ev.is_complete(dead_rbs) {
              if is_combo {
                heartbeat.n_timed_out_combo += 1;
                //println!("Ev {} timed out! ({} total) Seen rb events {}, expected {} link ids", ev.event_id, heartbeat.n_timed_out_combo, ev.rb_events.len(), ev.get_rb_link_ids().len());
              } else {
                heartbeat.n_timed_out += 1;
                //println!("GAPS Ev {} timed out! ({} total) Seen rb events {}, expected {} link ids! Expected RBs {:?}", ev.event_id, heartbeat.n_timed_out, ev.rb_events.len(), ev.get_rb_link_ids().len(), ev.get_expected_rbs(&exprbmap));
              }
              // let's check the rb_link_ids here and let's check which one is missing
              //for l_id in ev.get_rb_link_ids() {
              //  if weird_rbe_link_ids.contains_key(&l_id) {
              //    *weird_rbe_link_ids.get_mut(&l_id).unwrap() += 1;
              //  } else {
              //    weird_rbe_link_ids.insert(l_id, 1);
              //  }
              //}
            //}
          } // else {
          // we are earlier than our time out, maybe the 
          // event is already complete
          ready_to_send = ev_timed_out;
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
              if ev.is_complete(dead_rbs) {
                //println!("EV HAS {} RBEVENTS", ev.rb_events.len());
                ready_to_send = true;
              }
            }
          }
          //} 
          // this feature tries to sort the events which are getting sent
          // by id. This might lead to timed out events and more resources needed
          if settings.sort_events {
            if ready_to_send && !ev_timed_out {
              if idx == 0 {
                first_ev_sent = true;
                prior_ev_sent = ev.event_id;
              } else {
                if idx == 1 {
                  if !first_ev_sent {
                    // we wait and check the others too see if something 
                    // else timed out
                    ready_to_send = false;
                  }
                }
                if ev.event_id != (prior_ev_sent + 1) {
                  // we wait and check the others too see if something 
                  // else timed out
                  ready_to_send = false;
                }
                prior_ev_sent = ev.event_id;
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
            let is_mangled     = ev_to_send.has_any_mangling(); 
            if is_mangled {
              heartbeat.data_mangled_ev += 1;
            }
            if ev_timed_out && !is_mangled {
              ev_to_send.status = EventStatus::EventTimeOut;
            }
            //if send_rbwaveform {
            //  if rbwf_ctr == send_rbwf_freq as u64 {
            //    for wf in ev_to_send.get_waveforms() {
            //      let mut pack = wf.pack();
            //      pack.no_write_to_disk = true;
            //      match data_sink.send(pack) {
            //        Err(err) => {
            //          error!("Packet sending failed! Err {}", err);
            //        }
            //        Ok(_)    => {
            //          debug!("Event with id {} sent!", evid);
            //        }
            //      }
            //    }
            //    rbwf_ctr = 0;
            //  }
            //  rbwf_ctr += 1; // increase for every event, not wf
            //}
            // update event status, so that we will also see in an 
            // (optionally) produced tof event summary if the 
            // event has isuses
            n_rbe_per_te  += ev_to_send.rb_events.len();
            // sum up lost hits due to drs4 deadtime
            heartbeat.drs_bsy_lost_hg_hits += ev_to_send.get_lost_hits() as u32;

            //println!("GOT {} ", ev_to_send.hits.len());
            heartbeat.n_sent += 1;
            if is_combo {
              heartbeat.n_sent_combo_trigger += 1;
            } else {
              heartbeat.n_sent_trigger += 1;
            }
            //let mut no_write_to_disk = false;
            //let no_send_over_nw  = false; 
            //if send_tev_sum {
            //let tes  = ev_to_send.get_summary();
            // FIXME - these might be all zero!
            //if settings.only_save_interesting {
            //  no_write_to_disk = true;
            //  if ev_to_send.n_hits_umb   >= settings.thr_n_hits_umb 
            //  && ev_to_send.n_hits_cbe   >= settings.thr_n_hits_cbe
            //  && ev_to_send.n_hits_cor   >= settings.thr_n_hits_cor
            //  && ev_to_send.tot_edep_umb >= settings.thr_tot_edep_umb
            //  && ev_to_send.tot_edep_cbe >= settings.thr_tot_edep_cbe
            //  && ev_to_send.tot_edep_cor >= settings.thr_tot_edep_cor {
            //    no_write_to_disk = false;
            //  }
            //}
            //for wf in ev_to_send.get_waveforms() {
            //  println!( "FS {} ", wf);
            //}
            //for ev in &ev_to_send.rb_events {
            //   println!("{:?}", ev.hits);
            //}
            // Right now we want version V3 (which should already 
            // be set), so it saves waveforms 
            // FIXME - this all should go to the data_publsher
            ev_to_send.version = ProtocolVersion::V3;
            //let mut pack = ev_to_send.pack();
            //pack.no_write_to_disk = no_write_to_disk;
            match data_sink_ev.send(ev_to_send) {
              Err(err) => {
                error!("Packet sending failed! Err {}", err);
              }
              Ok(_)    => {
                
                debug!("Event with id {} sent!", evid);
              }
            //}
            }

            //if 
            
          // this happens when we are NOT ready to send -> cache it!
          } else { 
            event_id_cache.push_front(evid);
          }
        }
      }
    } // end loop over event_id_cache
    if hb_timer.elapsed() >= hb_interval {
      // print the statistics for the weird orphans 
      //println!("=> Weird orphan statistics. Here is a map of rb_id -> N_orphans (which ended up in the gutter)");
      //println!("=> {:?}", weird_orphan_rbids);
      //println!("-- -- -- -- -- -- -- -- -- -- -- --");

      // make sure the heartbeat has the latest mission elapsed time
      heartbeat.met_seconds         += hb_timer.elapsed().as_secs_f64() as u64;// as usize;
      // get the length of the various caches at this moment in time
      heartbeat.event_cache_size     = event_cache.len()    as u32;
      heartbeat.event_id_cache_size  = event_id_cache.len() as u32;
      heartbeat.mte_receiver_cbc_len = m_trig_ev.len()      as u32;
      heartbeat.rbe_receiver_cbc_len = ev_from_rb.len()     as u32;
      heartbeat.tp_sender_cbc_len    = data_sink.len()      as u32;

      let pack         = heartbeat.pack();
      //println!("Avg mt loop time {}", avg_mt_loop_time as f64/n_iter_mt_loop as f64);
      //println!("Avg rb loop time {}", avg_rb_loop_time as f64/n_iter_rbe_loop as f64);

      match data_sink.send(pack) {
        Err(err) => {
          error!("Packet sending failed! Err {}", err);
        }
        Ok(_)    => {
        }
      }
      hb_timer = Instant::now();
    }

    // mitigation for full channels 
    if ev_from_rb.is_full() {
    // flush rb events from the rb receiver channel 
    // limit level 1 
    //if let Some(pl)  = settings.rbe_purge_limit1 {
      //settings.n_rbe_per_loop = n_rbe_per_loop_default;
      //let purge       = settings.rbe_purge_limit1.unwrap_or(ev_from_rb.len() as u32);
      //let expired_sec = settings.rbe_purge_ev_time1.unwrap_or(30) as u64;
      //if ev_from_rb.len() > pl as usize {
      settings.n_rbe_per_loop = n_rbe_per_loop_default;
      for _k in 0..ev_from_rb.capacity().unwrap_or(20000) {
        match ev_from_rb.try_recv() {
          Err(_) => (), 
          Ok(_ev) => {
            ////FIXME - what to do with these?
            // We should probalby tell the heartbeat how many we have purged?
            //if ev.creation_time.is_some() {
            //  if ev.creation_time.unwrap().elapsed().as_secs() < expired_sec {
            //    match orphanage.send(ev) {
            //      Ok(_) => (),
            //      Err(err) => {
            //        error! ("Orphanage does not accept this orphan. They are dying in the gutter all by themselves. What a said world! {err}");
            //      }
            //    } 
            //  } else {
            //    //// just do statistics 
            //    //if weird_orphan_rbids.contains_key(&ev.header.rb_id) {
            //    //  *weird_orphan_rbids.get_mut(&ev.header.rb_id).unwrap() += 1;
            //    //} else {
            //    //  weird_orphan_rbids.insert(ev.header.rb_id,1);
            //    //}
            //  }
            //}
          }
        }
      }
      //}
    }

    // flush rb events from the rb receiver channel 
    // limit level 1 
    if let Some(pl)  = settings.rbe_purge_limit1 {
      //settings.n_rbe_per_loop = n_rbe_per_loop_default;
      //let purge       = settings.rbe_purge_limit1.unwrap_or(ev_from_rb.len() as u32);
      let purge      = ev_from_rb.len();
      let expired_sec = settings.rbe_purge_ev_time1.unwrap_or(30) as u64;
      if ev_from_rb.len() > pl as usize {
        settings.n_rbe_per_loop = n_rbe_per_loop_default;
        for _k in 0..purge {
          match ev_from_rb.recv() {
            Err(_) => (), 
            Ok(ev) => {
              //FIXME - what to do with these?
              if ev.creation_time.is_some() {
                if ev.creation_time.unwrap().elapsed().as_secs() < expired_sec {
                  match orphanage.send(ev) {
                    Ok(_) => (),
                    Err(err) => {
                      error! ("Orphanage does not accept this orphan. They are dying in the gutter all by themselves. What a said world! {err}");
                    }
                  } 
                } else {
                  // just do statistics 
                  if weird_orphan_rbids.contains_key(&ev.header.rb_id) {
                    *weird_orphan_rbids.get_mut(&ev.header.rb_id).unwrap() += 1;
                  } else {
                    weird_orphan_rbids.insert(ev.header.rb_id,1);
                  }
                }
              }
            }
          }
        }
      }
    }
 
    //// flush rb events from the rb receiver channel 
    //// limit level 2 
    //if let Some(pl)  = settings.rbe_purge_limit2 {
    //  let purge       = settings.rbe_purge_limit2.unwrap_or(ev_from_rb.len() as u32);
    //  let expired_sec = settings.rbe_purge_ev_time2.unwrap_or(30) as u64;
    //  if ev_from_rb.len() > pl as usize {
    //    settings.n_rbe_per_loop = n_rbe_per_loop_default;
    //    for k in 0..purge {
    //      match ev_from_rb.recv() {
    //        Err(_) => (), 
    //        Ok(ev) => {
    //          //FIXME - what to do with these?
    //          if ev.creation_time.is_some() {
    //            if ev.creation_time.unwrap().elapsed().as_secs() < expired_sec {
    //              match orphanage.send(ev) {
    //                Ok(_) => (),
    //                Err(err) => {
    //                  error! ("Orphanage does not accept this orphan. They are dying in the gutter all by themselves. What a said world! {err}");
    //                }
    //              } 
    //            } else {
    //              // just do statistics 
    //              if weird_orphan_rbids.contains_key(&ev.header.rb_id) {
    //                *weird_orphan_rbids.get_mut(&ev.header.rb_id).unwrap() += 1;
    //              } else {
    //                weird_orphan_rbids.insert(ev.header.rb_id,1);
    //              }
    //            }
    //          } 
    //        }
    //      }
    //    }
    //  }
    //}
    //// flush rb events from the rb receiver channel 
    //// limit level 3 
    //if let Some(pl)  = settings.rbe_purge_limit3 {
    //  let purge       = settings.rbe_purge_limit3.unwrap_or(ev_from_rb.len() as u32);
    //  let expired_sec = settings.rbe_purge_ev_time3.unwrap_or(30) as u64;
    //  if ev_from_rb.len() > pl as usize {
    //    settings.n_rbe_per_loop = n_rbe_per_loop_default;
    //    for k in 0..purge {
    //      match ev_from_rb.recv() {
    //        Err(_) => (), 
    //        Ok(ev) => {
    //          //FIXME - what to do with these?
    //          if ev.creation_time.is_some() {
    //            if ev.creation_time.unwrap().elapsed().as_secs() < expired_sec {
    //              match orphanage.send(ev) {
    //                Ok(_) => (),
    //                Err(err) => {
    //                  error! ("Orphanage does not accept this orphan. They are dying in the gutter all by themselves. What a said world! {err}");
    //                }
    //              } 
    //            }
    //          } else {
    //            // just do statistics 
    //            if weird_orphan_rbids.contains_key(&ev.header.rb_id) {
    //              *weird_orphan_rbids.get_mut(&ev.header.rb_id).unwrap() += 1;
    //            } else {
    //              weird_orphan_rbids.insert(ev.header.rb_id,1);
    //            }
    //          }
    //        }
    //      }
    //    }
    //  }
    //}
    
    ////if ev_from_rb.len() > 1000 {
    ////  for k in 0..1000 {
    ////    //while !ev_from_rb.is_empty() {
    ////    match ev_from_rb.recv() {
    ////      Err(_) => (), 
    ////      Ok(ev) => {
    ////        //FIXME - what to do with these?
    ////        if ev.creation_time.is_some() {
    ////          if ev.creation_time.unwrap().elapsed().as_secs() < 90 {
    ////            match orphanage.send(ev) {
    ////              Ok(_) => (),
    ////              Err(err) => {
    ////                error! ("Orphanage does not accept this orphan. They are dying in the gutter all by themselves. What a said world! {err}");
    ////              }
    ////            } 
    ////          }
    ////        }
    ////      }
    ////    }        
    ////  }
    ////  settings.n_rbe_per_loop = n_rbe_per_loop_default;
    ////}
    if settings.n_rbe_per_loop == 0 {
      // failsafe
      settings.n_rbe_per_loop = 40;
    }
    heartbeat.n_rbe_per_loop = settings.n_rbe_per_loop as u32;
  } // end loop
}  

