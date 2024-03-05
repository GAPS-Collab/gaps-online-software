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

use crossbeam_channel::{
    Receiver,
    Sender,
};


use tof_dataclasses::events::{MasterTriggerEvent,
                              TofEvent,
                              RBEvent};
use tof_dataclasses::packets::TofPacket;
use tof_dataclasses::manifest::get_dsi_j_ltbch_vs_rbch_map;
use tof_dataclasses::threading::ThreadControl;

use liftof_lib::settings::{
    TofEventBuilderSettings,
    BuildStrategy
};
use crate::constants::EVENT_BUILDER_EVID_CACHE_SIZE;

use colored::Colorize;

// debug the number of rb events we got from the individual boards

pub fn heartbeat_formatter() {
  let fwidth = 56;
  let n_mte_received_tot = 0;
  let mut repr = String::from("");
  repr += &String::from("  >> == == == == ==  EVTBLDR HEARTBEAT  == == == == == <<").bright_purple().bold();
  repr += &format!("  >> {:fwidth$} <<<", format!("Received MTEvents \t{}", n_mte_received_tot)).bright_purple().bold();
  //println!("  {:fwidth$}", ">> == == == == ==  EVTBLDR HEARTBEAT   == == == == == <<".bright_purple().bold());
  //println!("  {:fwidth$} <<", format!(">> ==> Received MTEvents \t{}", n_mte_received_tot).bright_purple());
  //println!("  {:fwidth$} <<", format!(">> ==> Received RBEvents \t{}", n_rbe_received_tot).bright_purple());
  //println!("  {:fwidth$} <<", format!(">> ==> Delta Last MTE evid - Last RB evid  {}", last_evid - last_rb_evid).bright_purple());
  //println!("  {:fwidth$} <<", format!(">> ==> Size of event cache    \t{}", event_cache.len()).bright_purple());
  //println!("  {:fwidth$} <<", format!(">> ==> Size of event ID cache \t{}", event_id_cache.len()).bright_purple());
  //println!("  {:fwidth$} <<", format!(">> ==> Get MTE from cache for RB ev failed {rb_ev_wo_mte} times!").bright_purple());
  //println!("  {:fwidth$} <<", format!(">> ==> Sent {} events!", n_sent).bright_purple());
  //println!("  {:fwidth$} <<", format!(">> ==> Chn len MTE receiver\t {}",m_trig_ev.len() ).bright_purple());
  //println!("  {:fwidth$} <<", format!(">> ==> Chn len RBE receiver\t {}",ev_from_rb.len()).bright_purple());
  //println!("  {:fwidth$} <<", format!(">> ==> Chn len TP  sender  \t {}",data_sink.len()).bright_purple());
  //if n_sent > 0 {
  //  let av_rb_ev = n_rbs_per_ev as f64 / n_sent as f64;
  //  println!(">> ==> Average number of RBEvents/TofEvent {:4.2}", av_rb_ev);
  //}
  //if n_mte_received_tot > 0 {
  //  let to_frac = n_timed_out as f64 / n_mte_received_tot as f64;
  //  println!(">> ==> Fraction of timed out events {:4.2}", to_frac);
  //}
  //println!(">> ==> RBEvents received overview:");
  ////let mut rbtable_repr = String::from(">> ");
  ////for k in seen_rbevents.keys() {
  ////  println!(">> >> RB {}  : {}", k, seen_rbevents[k]);
  ////}
  //
  //let mut key_value_pairs: Vec<_> = seen_rbevents.iter().collect();
  //let mut head0 = String::from(">> >> ");
  //let mut row0  = String::from(">> >> ");
  //let mut head1 = String::from(">> >> ");
  //let mut row1  = String::from(">> >> ");
  //let mut head2 = String::from(">> >> ");
  //let mut row2  = String::from(">> >> ");
  //let mut head3 = String::from(">> >> ");
  //let mut row3  = String::from(">> >> ");
  //let mut head4 = String::from(">> >> ");
  //let mut row4  = String::from(">> >> ");
  //key_value_pairs.sort_by(|a, b| a.0.cmp(b.0));
  //for (key, value) in key_value_pairs {
  //    if key < &10 {
  //      head0 += &(format!(" RB {:02}\t| ", key)); 
  //      row0  += &(format!(" {}\t| ", value));
  //      continue;
  //    }
  //    if key < &20 {
  //      head1 += &(format!(" RB {:02}\t| ", key)); 
  //      row1  += &(format!(" {}\t| ", value));
  //      continue;
  //    }
  //    if key < &30 {
  //      head2 += &(format!(" RB {:02}\t| ", key)); 
  //      row2  += &(format!(" {}\t| ", value));
  //      continue;
  //    }
  //    if key < &40 {
  //      head3 += &(format!(" RB {:02}\t| ", key)); 
  //      row3  += &(format!(" {}\t| ", value));
  //      continue;
  //    }
  //    if key < &50 {
  //      head4 += &(format!(" RB {:02}\t| ", key)); 
  //      row4  += &(format!(" {}\t| ", value));
  //      continue;
  //    }
  //    //println!(">> >> RB {} : {}", key, value);
  //}
  //println!("  {}", head0);
  //println!("  {}", row0);
  //println!("  {}", head1);
  //println!("  {}", row1);
  //println!("  {}", head2);
  //println!("  {}", row2);
  //println!("  {}", head3);
  //println!("  {}", row3);
  //println!("  {}", head4);
  //println!("  {}", row4);
  //println!("  {}",">> == == == == ==  END HEARTBEAT! == == == == == <<".bright_purple().bold());
}


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
///
/// * pp_query       : Send request to a paddle_packet cache to send
///                    Paddle packets with the given event id
/// * paddle_packets : Receive paddle_packets from a paddle_packet
///                    cache
/// * cmd_sender     : Sending tof commands to comander thread. This 
///                    is needed so that the event builder can request
///                    paddles from readout boards.
/// * settings       : Configure the event builder
pub fn event_builder (m_trig_ev      : &Receiver<MasterTriggerEvent>,
                      ev_from_rb     : &Receiver<RBEvent>,
                      data_sink      : &Sender<TofPacket>,
                      run_id         : u32,
                      db_path        : String,
                      mut settings   : TofEventBuilderSettings,
                      thread_control : Arc<Mutex<ThreadControl>>) { 
  // debug the number of rb events we have seen 
  let mut seen_rbevents      = HashMap::<u8, usize>::new();
  // 10, 12, 37,38, 43, 45 don't exist
  for k in 1..47 {
    if k == 10 || k ==12 || k == 37 || k == 38 || k == 43 || k == 45 {
      continue;
    } else {
      seen_rbevents.insert(k as u8, 0);
    }
  }
  // event caches for assembled events
  let mut event_cache        = HashMap::<u32, TofEvent>::new();
  let mut event_id_cache     = VecDeque::<u32>::with_capacity(EVENT_BUILDER_EVID_CACHE_SIZE);
  let dsi_map                = get_dsi_j_ltbch_vs_rbch_map(db_path.as_ref()); 
  let mut n_received         : usize;
  //let mut clear_cache      = 0; // clear cache every 
  //let mut event_sending    = 0;
  let mut n_mte_received_tot = 0u64;
  let mut n_rbe_received_tot = 0u64;
  let mut first_evid         : u32;
  let mut last_evid          = 0;
  let mut n_sent             = 0usize;
  let mut n_timed_out        = 0usize; 
  // debug
  let mut last_rb_evid       = 0u32;
  let mut n_rbs_per_ev       = 0usize;
  let mut rb_ev_wo_mte       = 0usize;
  let mut debug_timer = Instant::now();
    
  let mut n_receiving_errors = 0;
  let mut check_tc_update    = Instant::now();
  loop {
    if check_tc_update.elapsed().as_secs() > 5 {
      match thread_control.lock() {
        Ok(tc) => {
          if tc.calibration_active {
            thread::sleep(Duration::from_secs(1));
            continue;
          } else {
            check_tc_update = Instant::now();
          }
        },
        Err(err) => {
          error!("Can't acquire lock for ThreadControl! Unable to set calibration mode! {err}");
        },
      }
    }
    if n_receiving_errors == 1000 {
      error!("Trying to get a new MTEvent failed {}", n_receiving_errors);
      n_receiving_errors = 0;
    }
    n_received = 0;
    while n_received < settings.n_mte_per_loop {
      // every iteration, we welcome a new master event
      match m_trig_ev.try_recv() {
        Err(_) => {
          trace!("No new event ready yet!");
          n_receiving_errors += 1;
          continue;
        }   
        Ok(mt) => {
          debug!("Got master trigger for event {} with {} expected hit paddles"
                 , mt.event_id
                 , mt.n_paddles);
          // construct RB requests

          let mut event = TofEvent::from(&mt);
          event.header.run_id = run_id;
          if event.mt_event.event_id != last_evid + 1 {
            let delta_id = event.mt_event.event_id - last_evid;
            error!("We skipped event ids {}", delta_id );
          }
          last_evid = event.mt_event.event_id;
          event_cache.insert(last_evid, event);
          // use this to keep track of the order
          // of events
          event_id_cache.push_back(last_evid);
          
        }
      } // end match Ok(mt)
      n_received  += 1;
      //if n_received % 10 == 0 {
      //  println!("==> Received 10 more MasterTriggerEvents");
      //}
      n_mte_received_tot += 1;
    } // end getting MTEvents
    trace!("Debug timer MTE received! {:?}", debug_timer.elapsed());
    first_evid = event_id_cache[0]; 
    
    // check this timeout
    //let mut rb_events_added   = 0usize;
    //let mut iter_ev           = 0usize;
    //let mut rb_events_dropped = 0usize;
    n_received = 0;
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
          n_rbe_received_tot += 1;
          n_received += 1;
          match seen_rbevents.get_mut(&rb_ev.header.rb_id) {
            Some(value) => {
              *value += 1;
            }
            None => {
              error!("Unable to do bookkeeping for RB {}", rb_ev.header.rb_id);
            }
          }
          //iter_ev = 0;
          last_rb_evid = rb_ev.header.event_id;
          if last_rb_evid < first_evid {
            n_received -= 1;
            debug!(".. re-synchrnoizing ..");
            continue;
          }
          match event_cache.get_mut(&last_rb_evid) {
            None => {
              //FIXME - big issue!
              //println!("Surprisingly we don't have that!");
              // insert a new TofEvent
              //let new_ev = TofEvent::new();
              rb_ev_wo_mte += 1;
              continue 'main;
            },
            Some(ev) => {
              if settings.build_strategy == BuildStrategy::AdaptiveThorough {
                let lg_hits   = ev.mt_event.get_dsi_j_ch_for_triggered_ltbs(); 
                let mut found = false;
                let ev_rbid   = rb_ev.header.rb_id; 
                for h in &lg_hits {
                  if dsi_map[&h.0][&h.1][&h.2].0 == rb_ev.header.rb_id {
                    ev.rb_events.push(rb_ev);
                    found = true;
                    break;
                  }
                }
                if !found {
                  println!("== ==> We saw {:?}, but {} is not part of that!", lg_hits, ev_rbid);
                }
              } else {
                ev.rb_events.push(rb_ev);
              }
              //break;
            }
          }
    
          //for ev in event_cache.iter_mut() {
          //  //iter_ev += 1;
          //  //println!("==> mt event id {}, rb event id {}", ev.mt_event.event_id, rb_ev.header.event_id);
          //  //println!("==> rb event id {}", rb_ev.header.event_id);
          //  if ev.mt_event.event_id == rb_ev.header.event_id {
          //    ev.rb_events.push(rb_ev);
          //    break;
          //    //rb_events_added += 1;
          //    //println!("==> RBEvent added!");
          //    //break;
          //  }
          //}
          //if iter_ev == event_cache.len() {
          //  info!("We dropped {}", rb_ev);
          //}
          //if n_received % 10 == 0 {
          //  println!("==> Received 10 more RBEvents");
          //}
        }
      }
    }
    let av_rb_ev = n_rbs_per_ev as f64 / n_sent as f64;
    if settings.build_strategy == BuildStrategy::Adaptive || 
       settings.build_strategy == BuildStrategy::AdaptiveThorough {
      settings.n_rbe_per_loop = av_rb_ev.ceil() as usize;
      if settings.n_rbe_per_loop == 0 {
        // failsafe
        settings.n_rbe_per_loop = 40;
      }
    }
    if let BuildStrategy::AdaptiveGreedy(greediness) = settings.build_strategy {
      settings.n_rbe_per_loop = av_rb_ev.ceil() as usize + greediness;
      if settings.n_rbe_per_loop == 0 {
        // failsafe
        settings.n_rbe_per_loop = 40;
      }
    }
    let debug_timer_elapsed = debug_timer.elapsed().as_secs_f64();
    if debug_timer_elapsed > 35.0  {
      let fwidth = 100;
      println!("  {:fwidth$}", ">> == == == == ==  EVTBLDR HEARTBEAT   == == == == == <<".bright_purple().bold());
    //if n_mte_received_tot % 50 == 0 || n_rbe_received_tot % 200 == 0 {
      println!("  {:fwidth$} <<", format!(">> ==> Received MTEvents \t{}", n_mte_received_tot).bright_purple());
      println!("  {:fwidth$} <<", format!(">> ==> Received RBEvents \t{}", n_rbe_received_tot).bright_purple());
      println!("  {:fwidth$} <<", format!(">> ==> Delta Last MTE evid - Last RB evid  {}", last_evid - last_rb_evid).bright_purple());
      println!("  {:fwidth$} <<", format!(">> ==> Size of event cache    \t{}", event_cache.len()).bright_purple());
      println!("  {:fwidth$} <<", format!(">> ==> Size of event ID cache \t{}", event_id_cache.len()).bright_purple());
      println!("  {:fwidth$} <<", format!(">> ==> Get MTE from cache for RB ev failed {rb_ev_wo_mte} times!").bright_purple());
      println!("  {:fwidth$} <<", format!(">> ==> Sent {} events!", n_sent).bright_purple());
      println!("  {:fwidth$} <<", format!(">> ==> Chn len MTE receiver\t {}",m_trig_ev.len() ).bright_purple());
      println!("  {:fwidth$} <<", format!(">> ==> Chn len RBE receiver\t {}",ev_from_rb.len()).bright_purple());
      println!("  {:fwidth$} <<", format!(">> ==> Chn len TP  sender  \t {}",data_sink.len()).bright_purple());
      if n_sent > 0 {
        let av_rb_ev = n_rbs_per_ev as f64 / n_sent as f64;
        println!("  {:fwidth$} <<", format!(">> ==> Average number of RBEvents/TofEvent {:4.2}", av_rb_ev).bright_purple());
      }
      if n_mte_received_tot > 0 {
        let to_frac = n_timed_out as f64 / n_mte_received_tot as f64;
        println!("  {:fwidth$} <<", format!(">> ==> Fraction of timed out events {:4.2}", to_frac).bright_purple());
      }
      println!(">> ==> RBEvents received overview:");
      //let mut rbtable_repr = String::from(">> ");
      //for k in seen_rbevents.keys() {
      //  println!(">> >> RB {}  : {}", k, seen_rbevents[k]);
      //}
      
      let mut key_value_pairs: Vec<_> = seen_rbevents.iter().collect();
      let mut head0 = String::from(">> >> ");
      let mut row0  = String::from(">> >> ");
      let mut head1 = String::from(">> >> ");
      let mut row1  = String::from(">> >> ");
      let mut head2 = String::from(">> >> ");
      let mut row2  = String::from(">> >> ");
      let mut head3 = String::from(">> >> ");
      let mut row3  = String::from(">> >> ");
      let mut head4 = String::from(">> >> ");
      let mut row4  = String::from(">> >> ");
      key_value_pairs.sort_by(|a, b| a.0.cmp(b.0));
      for (key, value) in key_value_pairs {
          if key < &10 {
            head0 += &(format!(" RB {:02}\t| ", key)); 
            row0  += &(format!(" {}\t| ", value));
            continue;
          }
          if key < &20 {
            head1 += &(format!(" RB {:02}\t| ", key)); 
            row1  += &(format!(" {}\t| ", value));
            continue;
          }
          if key < &30 {
            head2 += &(format!(" RB {:02}\t| ", key)); 
            row2  += &(format!(" {}\t| ", value));
            continue;
          }
          if key < &40 {
            head3 += &(format!(" RB {:02}\t| ", key)); 
            row3  += &(format!(" {}\t| ", value));
            continue;
          }
          if key < &50 {
            head4 += &(format!(" RB {:02}\t| ", key)); 
            row4  += &(format!(" {}\t| ", value));
            continue;
          }
          //println!(">> >> RB {} : {}", key, value);
      }
      println!("  {}", head0);
      println!("  {}", row0);
      println!("  {}", head1);
      println!("  {}", row1);
      println!("  {}", head2);
      println!("  {}", row2);
      println!("  {}", head3);
      println!("  {}", row3);
      println!("  {}", head4);
      println!("  {}", row4);
      println!("  {}",">> == == == == ==  END HEARTBEAT! == == == == == <<".bright_purple().bold());
      //println!("[EVTBLDR] ==> Last RB evid {last_rb_evid}");
      debug_timer = Instant::now(); 
    }
    trace!("Debug timer RBE received! {:?}", debug_timer.elapsed());
    //if event_sending == send_every_x_event {
    if true {
      //for ev in event_cache.iter_mut() {
      let this_cache_size = event_id_cache.len();
      for _ in 0..this_cache_size {
        // if there wasn't a first element, size would be 0
        let evid = event_id_cache.pop_front().unwrap();
        match event_cache.get(&evid) {
          None => {
            error!("This should not happen!");
            event_id_cache.push_back(evid);
            continue;
          },
          Some(ev) => {
          //for evid in event_id_cache.iter() {
            //println!("{}",ev.age());
            let ev_timed_out = ev.age() >= settings.te_timeout_sec as u64;
            let cache_it : bool;
            if ev_timed_out {
              n_timed_out += 1;
            }
            if let BuildStrategy::WaitForNBoards(wait_nrb) = settings.build_strategy {
              if ev_timed_out || ev.rb_events.len() == wait_nrb {
                cache_it = false;
              } else {
                cache_it = true;
              }
            } else {
              // "normal" build strategy
              if ev.is_complete() || ev_timed_out {
                cache_it = false;
              } else {
                cache_it = true;
              }
            }
            if cache_it {
              event_id_cache.push_back(evid);
            } else {
              // if we don't cache it, we have to send it. 
              let ev_to_send = event_cache.remove(&evid).unwrap();
              n_rbs_per_ev  += ev_to_send.rb_events.len(); 
              if settings.send_flight_packets {
                // we have to chop it up
                let te_summary = ev_to_send.get_summary();
                let pack = TofPacket::from(&te_summary);
                match data_sink.send(pack) {
                  Err(err) => {
                    error!("Packet sending failed! {err}");
                  }
                  Ok(_)    => {
                    trace!("Event Summary for event id {} send!", evid);
                    n_sent += 1;
                  }
                }
                for rbwave in ev_to_send.get_rbwaveforms() {
                  let pack = TofPacket::from(&rbwave);
                  match data_sink.send(pack) {
                    Err(err) => {
                      error!("Packet sending failed! {err}");
                    }
                    Ok(_)    => {
                      trace!("RB waveform for event id {} send!", evid);
                    }
                  }
                }
              } else {
                let pack       = TofPacket::from(&ev_to_send);
                match data_sink.send(pack) {
                  Err(err) => {
                    error!("Packet sending failed! Err {}", err);
                  }
                  Ok(_)    => {
                    debug!("Event with id {} send!", evid);
                    n_sent += 1;
                  }
                }
              }
            }
          }
        }
      }
      //event_sending = 0;
      //event_cache.retain(|ev| ev.valid);
      debug!("Debug timer! EVT SENDING {:?}", debug_timer.elapsed());
    } 
  } // end loop
}

