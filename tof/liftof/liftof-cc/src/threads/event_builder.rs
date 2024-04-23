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

use tof_dataclasses::packets::TofPacket;
use tof_dataclasses::threading::ThreadControl;

//use liftof_lib::heartbeat_printer;
use liftof_lib::settings::{
    TofEventBuilderSettings,
    BuildStrategy,
};
use crate::constants::EVENT_BUILDER_EVID_CACHE_SIZE;

use colored::{
    Colorize,
    ColoredString
};

#[derive(Debug, Clone, PartialEq)]
pub struct EventBuilderHeartBeat {
  pub met_seconds           : usize,
  pub n_mte_received_tot    : usize,
  pub n_rbe_received_tot    : usize,
  pub n_rbe_per_te          : usize,
  pub n_rbe_discarded_tot   : usize,
  pub n_mte_skipped         : usize,
  pub n_timed_out           : usize,
  pub n_sent                : usize,
  pub delta_mte_rbe         : usize,
  pub event_cache_size      : usize,
  pub event_id_cache_size   : usize, 
  pub rbe_wo_mte            : usize,
  pub n_sent_events         : usize,
  pub mte_receiver_cbc_len  : usize,
  pub rbe_receiver_cbc_len  : usize,
  pub tp_sender_cbc_len     : usize,
  pub seen_rbevents         : HashMap<u8, usize>,
}

impl EventBuilderHeartBeat {
  pub fn new() -> Self {
    let mut seen_rbevents = HashMap::<u8, usize>::new();
    for k in 1..47 {
      if k == 10 || k ==12 || k == 37 || k == 38 || k == 43 || k == 45 {
        continue;
      } else {
        seen_rbevents.insert(k as u8, 0);
      }
    }
    Self {
      met_seconds          : 0,
      n_mte_received_tot   : 0,
      n_rbe_received_tot   : 0,
      n_rbe_per_te         : 0,
      n_rbe_discarded_tot  : 0,
      n_mte_skipped        : 0,
      n_timed_out          : 0,
      n_sent               : 0,
      delta_mte_rbe        : 0,
      event_cache_size     : 0,
      event_id_cache_size  : 0,
      rbe_wo_mte           : 0,
      n_sent_events        : 0,
      mte_receiver_cbc_len : 0,
      rbe_receiver_cbc_len : 0,
      tp_sender_cbc_len    : 0,
      seen_rbevents        : seen_rbevents, 
    }
 }

 pub fn get_average_rbe_te(&self) -> f64 {
   if self.n_sent > 0 {
     return self.n_rbe_per_te as f64 / self.n_sent as f64;
   }
   0.0
 }

 pub fn get_timed_out_frac(&self) -> f64 {
   if self.n_sent > 0 {
     return self.n_timed_out as f64 / self.n_sent as f64;
   }
   0.0
 }

 pub fn add_rbevent(&mut self, rb_id : u8) {
   *self.seen_rbevents.get_mut(&rb_id).unwrap() += 1;
 }

 pub fn get_incoming_vs_outgoing_mte(&self) -> f64 {
   if self.n_sent > 0 {
     return self.n_mte_received_tot as f64 /  self.n_sent as f64;
   }
   0.0
 }

 pub fn get_nrbe_discarded_frac(&self) -> f64 {
   if self.n_rbe_received_tot > 0 {
     return self.n_rbe_discarded_tot as f64 / self.n_rbe_received_tot as f64;
   }
   0.0
 }

 pub fn get_string(&self) -> ColoredString {
   let mut string_field = Vec::<ColoredString>::new();
   let mut hb = String::from("== == == == == == EVENTBUILDER HEARTBTEAT == == == == == ==").bright_purple();
   string_field.push(hb);
   //if n_mte_received_tot % 50 == 0 || n_rbe_received_tot % 200 == 0 {
   hb = format!("==> Received MTEvents \t\t{}", self.n_mte_received_tot).bright_purple();
   string_field.push(hb.clone());
   hb = format!("==> Received RBEvents \t\t{}", self.n_rbe_received_tot).bright_purple();
   string_field.push(hb.clone());
   hb = format!("==> Skipped MTEvents  \t\t{}", self.n_mte_skipped     ).bright_purple();
    //hb = format!("==> Delta Last MTE evid - Last RB evid  {}", last_evid - last_rb_evid).bright_purple());
   string_field.push(hb.clone());
   hb = format!("==> Size of event cache    \t{}", self.event_cache_size).bright_purple();
   string_field.push(hb.clone());
   hb = format!("==> Size of event ID cache \t{}", self.event_id_cache_size).bright_purple();
   string_field.push(hb.clone());
   hb = format!("==> Get MTE from cache for RB ev failed {} times!", self.rbe_wo_mte).bright_purple();
   string_field.push(hb.clone());
   hb = format!("==> Sent {} events!", self.n_sent).bright_purple();
   string_field.push(hb.clone());
   //hb = format!("==> Chn len MTE receiver\t{}",self.m_trig_ev.len() ).bright_purple();
   //hb = format!("==> Chn len RBE receiver\t{}",ev_from_rb.len()).bright_purple();
   //hb = format!("==> Chn len TP  sender  \t{}",data_sink.len()).bright_purple();
    //  if n_sent > 0 {
    //    let av_rb_ev = n_rbs_per_ev as f64 / n_sent as f64;
    //    println!("  {:fwidth$} <<", format!(">> ==> Average number of RBEvents/TofEvent {:4.2}", av_rb_ev).bright_purple());
    //    let to_frac = 100.0 * n_timed_out as f64 / n_sent as f64;
    //    println!("  {:fwidth$} <<", format!(">> ==> Fraction of timed out events {:4.2}%", to_frac).bright_purple());
    //    let recv_sent_frac = 100.0* n_mte_received_tot as f64 / n_sent as f64;
    //    println!("  {:fwidth$} <<", format!(">> ==> Fraction of incoming vs outgoing MTEvents {:4.2}%", recv_sent_frac).bright_purple());
    //  }
    //  if n_rbe_received_tot > 0 {
    //    let rbe_discarded_frac = 100.0 * n_rbe_discarded_tot as f64 / n_rbe_received_tot as f64;
    //    println!("  {:fwidth$} <<", format!(">> ==> Fraction of discarded RBEvents {:4.2}%", rbe_discarded_frac).bright_purple());
    //  }
    //  println!(">> ==> RBEvents received overview:");
    //  //let mut rbtable_repr = String::from(">> ");
    //  //for k in seen_rbevents.keys() {
    //  //  println!(">> >> RB {}  : {}", k, seen_rbevents[k]);
    //  //}
   hb
 }
 //     }
 //     println!(">> ==> RBEvents received overview:");
 //     //let mut rbtable_repr = String::from(">> ");
 //     //for k in seen_rbevents.keys() {
 //     //  println!(">> >> RB {}  : {}", k, seen_rbevents[k]);
 //     //}
 //     
 //     let mut key_value_pairs: Vec<_> = seen_rbevents.iter().collect();
 //     let mut head0 = String::from(">> >> ");
 //     let mut row0  = String::from(">> >> ");
 //     let mut head1 = String::from(">> >> ");
 //     let mut row1  = String::from(">> >> ");
 //     let mut head2 = String::from(">> >> ");
 //     let mut row2  = String::from(">> >> ");
 //     let mut head3 = String::from(">> >> ");
 //     let mut row3  = String::from(">> >> ");
 //     let mut head4 = String::from(">> >> ");
 //     let mut row4  = String::from(">> >> ");
 //     key_value_pairs.sort_by(|a, b| a.0.cmp(b.0));
 //     for (key, value) in key_value_pairs {
 //         if key < &10 {
 //           head0 += &(format!(" RB {:02}\t| ", key)); 
 //           row0  += &(format!(" {}\t| ", value));
 //           continue;
 //         }
 //         if key < &20 {
 //           head1 += &(format!(" RB {:02}\t| ", key)); 
 //           row1  += &(format!(" {}\t| ", value));
 //           continue;
 //         }
 //         if key < &30 {
 //           head2 += &(format!(" RB {:02}\t| ", key)); 
 //           row2  += &(format!(" {}\t| ", value));
 //           continue;
 //         }
 //         if key < &40 {
 //           head3 += &(format!(" RB {:02}\t| ", key)); 
 //           row3  += &(format!(" {}\t| ", value));
 //           continue;
 //         }
 //         if key < &50 {
 //           head4 += &(format!(" RB {:02}\t| ", key)); 
 //           row4  += &(format!(" {}\t| ", value));
 //           continue;
 //         }
 //         //println!(">> >> RB {} : {}", key, value);
 //     }
 //     println!("  {}", head0);
 //     println!("  {}", row0);
 //     println!("  {}", head1);
 //     println!("  {}", row1);
 //     println!("  {}", head2);
 //     println!("  {}", row2);
 //     println!("  {}", head3);
 //     println!("  {}", row3);
 //     println!("  {}", head4);
 //     println!("  {}", row4);
 //     println!("  {}",">> == == == == ==  END HEARTBEAT! == == == == == <<".bright_purple().bold());
 //   }
 // }
}

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
  let mut seen_rbevents = HashMap::<u8, usize>::new();
  // 10, 12, 37,38, 43, 45 don't exist
  for k in 1..47 {
    if k == 10 || k ==12 || k == 37 || k == 38 || k == 43 || k == 45 {
      continue;
    } else {
      seen_rbevents.insert(k as u8, 0);
    }
  }
  // event caches for assembled events
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
  let mut n_rbe_received_tot   = 0u64;
  let mut n_rbe_discarded_tot  = 0u64;
  let mut first_evid           : u32;
  let mut last_evid            = 0;
  let mut n_sent               = 0usize;
  let mut n_sent_ch_err        = 0usize;
  let mut n_timed_out          = 0usize; 
  // debug
  let mut last_rb_evid         = 0u32;
  let mut n_rbs_per_ev         = 0usize;
  let mut rb_ev_wo_mte         = 0usize;
  let mut n_rbe_from_past      = 0usize;
  let mut n_rbe_orphan         = 0usize;
  let mut debug_timer          = Instant::now();
  let mut met_total_sec        = 0f64;  
  //let mut n_receiving_errors  = 0;
  let mut check_tc_update      = Instant::now();
  let mut n_gathered_fr_cache  = 0usize;
  let mut misaligned_cache_err = 0usize; 
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
          last_evid = event.mt_event.event_id;
          event_cache.insert(last_evid, event);
          // use this to keep track of the order
          // of events
          event_id_cache.push_back(last_evid);
          n_received  += 1;
          n_mte_received_tot += 1;
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
            debug!("The received RBEvent {} is from the ancient past! Currently, we don't have a way to deal with that and this event will be DISCARDED! The RBEvent queue will be re-synchronized...", last_rb_evid);
            //attempts += 1;
            n_rbe_discarded_tot += 1;
            n_rbe_from_past += 1;
            continue;
          }
          match event_cache.get_mut(&last_rb_evid) {
            None => {
              //FIXME - big issue!
              //println!("Surprisingly we don't have that!");
              // insert a new TofEvent
              //let new_ev = TofEvent::new();
              rb_ev_wo_mte += 1;
              n_rbe_discarded_tot += 1;
              n_rbe_orphan += 1;
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
    let av_rb_ev = n_rbs_per_ev as f64 / n_sent as f64;
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
    if let BuildStrategy::AdaptiveGreedy(greediness) = settings.build_strategy {
      settings.n_rbe_per_loop = av_rb_ev.ceil() as usize + greediness;
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
      let mut missing = 0usize;
      for _ in 0..event_id_cache.len() {
        if !event_id_cache.contains(&evid_check) {
          missing += 1;
        }
        evid_check += 1;
      }
      //event_id_test.dedup();
      //println!("DEBUG 1");
      //let evid_id_test_len = event_id_test.len();
      //println!("DEBUG .1.");
      //let evid_test_len = event_id_cache.len();
      //let mut evid_test_missing = 0usize;
      //if evid_test_len > 0 {
      //  let mut evid = event_id_cache[0];
      //  //println!("DEBUG 1.5");
      //  //println!("len of evid_id_test {}", evid_id_test_len);
      //  for _ in 0..evid_test_len {
      //    if !event_id_cache.contains(&evid) {
      //      evid_test_missing += 1;
      //    }
      //    evid += 1;
      //  }
      //}
      ////println!("DEBUG 2");
      //event_id_test.clear();
      //println!("DEBUG 3");
      
      //let mut hbs : Vec<String>;
      //let mut line :String;
      //let title = String::from("== == == == == == EVENTBUILDER HEARTBTEAT == == == == == ==");
      //hbs.push(title);
      //line = format!("==> Received MTEvents \t{}", n_mte_received_tot).bright_purple();
      //hbs.push(line);
      //let end   = String::from("== == == == == == == == == == == == == == == == == == == ==");
      //hbs.push(end);
      met_total_sec += debug_timer_elapsed;
      println!("  {:<70} <<", ">> == == == == == == ==  EVTBLDR HEARTBEAT == ==  == == == == ==".bright_purple().bold());
    //if n_mte_received_tot % 50 == 0 || n_rbe_received_tot % 200 == 0 {
      println!("  {:<70} <<", format!(">> ==> Received MTEvents {}", n_mte_received_tot).bright_purple());
      println!("  {:<70} <<", format!(">> ==> Received RBEvents {}", n_rbe_received_tot).bright_purple());
      println!("  {:<70} <<", format!(">> ==> Skipped MTEvents  {}", n_mte_skipped).bright_purple());
      //println!("  {:<80}", format!(">> ==> Missing evid analysis:  {} of {} events missing ({:.2}%)<<", ev_id_test_missing, ev_id_test_len, 100.0*(ev_id_test_missing as f64/evid_test_len as f64)).bright_purple());
      println!("  {:<70} <<", format!(">> ==> Delta Last MTE evid - Last RB evid  {}", last_evid - last_rb_evid).bright_purple());
      println!("  {:<70} <<", format!(">> ==> Size of event cache    {}", event_cache.len()).bright_purple());
      println!("  {:<70} <<", format!(">> ==> Size of event ID cache {}", event_id_cache.len()).bright_purple());
      println!("  {:<70} <<", format!(">> ==> Gathered events from cache, last iter {}", n_gathered_fr_cache).bright_purple());
      println!("  {:<70} <<", format!(">> ==> Misaligned cache errs  {}", misaligned_cache_err).bright_purple());
      println!("  {:<70} <<", format!(">> ==> Get MTE from cache for RB ev failed {rb_ev_wo_mte} times!").bright_purple());
      println!("  {:<70} <<", format!(">> ==> Sent {} events! rate   {:4.2} Hz", n_sent, n_sent as f64/met_total_sec).bright_purple());
      println!("  {:<70} <<", format!(">> ==> Failed in sending {} events!", n_sent_ch_err).bright_purple());
      println!("  {:<70} <<", format!(">> ==> Chn len MTE receiver   {}",m_trig_ev.len() ).bright_purple());
      println!("  {:<70} <<", format!(">> ==> Chn len RBE receiver   {}",ev_from_rb.len()).bright_purple());
      println!("  {:<70} <<", format!(">> ==> Chn len TP  sender     {}",data_sink.len()).bright_purple());
      println!("  {:<70} <<", format!(">> ==> Event id check: {:4.2}% of events missing in the event ID cache", missing as f64 / event_cache.len() as f64).bright_purple());
      if n_sent > 0 {
        let av_rb_ev = n_rbs_per_ev as f64 / n_sent as f64;
        println!("  {:<70} <<", format!(">> ==> Average number of RBEvents/TofEvent {:4.2}", av_rb_ev).bright_purple());
        let to_frac = 100.0 * n_timed_out as f64 / n_sent as f64;
        println!("  {:<70} <<", format!(">> ==> Fraction of timed out events {:4.2}%", to_frac).bright_purple());
        let recv_sent_frac = 100.0* n_mte_received_tot as f64 / n_sent as f64;
        println!("  {:<70} <<", format!(">> ==> Fraction of incoming vs outgoing MTEvents {:4.2}%", recv_sent_frac).bright_purple());
      }
      if n_rbe_received_tot > 0 {
        let rbe_discarded_frac = 100.0 * n_rbe_discarded_tot as f64 / n_rbe_received_tot as f64;
        let rbe_fpast_frac     = 100.0 * n_rbe_from_past   as f64 / n_rbe_received_tot as f64;
        let rbe_orphaned_frac  = 100.0 * n_rbe_orphan        as f64 / n_rbe_received_tot as f64;
        println!("  {:<70} <<", format!(">> ==> Fraction of discarded RBEvents {:4.2}%", rbe_discarded_frac).bright_purple());
        println!("  {:<70} <<", format!(">> ==> RBEvents discarded (too early) {} ({:4.2}%)",n_rbe_from_past, rbe_fpast_frac).bright_purple());
        println!("  {:<70} <<", format!(">> ==> RBEvents discarded (orphaned, too late?) {} ({:4.2}%)",n_rbe_orphan, rbe_orphaned_frac).bright_purple());
      }
      println!("RBEvents received overview (rate/RB [Hz]):");

      //let mut key_value_pairs: Vec<_> = seen_rbevents.iter().collect();
      //key_value_pairs.sort_by(|a, b| a.0.cmp(b.0));
      //for (key, value) in key_value_pairs {
      //    if key < &10 {
      //      head0 += &(format!(" RB {:02}\t| ", key)); 
      //      row0  += &(format!(" {:.1}\t| ", *value as f64/met_total_sec as f64));
      //      continue;
      let mut counters = HashMap::<u8,f64>::new();
      for k in seen_rbevents.keys() {
        counters.insert(*k, seen_rbevents[&k] as f64/met_total_sec as f64);
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
              n_timed_out += 1;
            }
          }
          // always ready when the event is timed out
          let mut ready_to_send = ev_timed_out;
          if !ev_timed_out {
            // we are earlier than our time out, maybe the 
            // event is already complete
            match settings.build_strategy {
              BuildStrategy::WaitForNBoards(wait_nrb) => {
                // we will always wait for the expected number of boards, 
                // except the event times out
                if ev.rb_events.len() == wait_nrb {
                  ready_to_send = true;
                  //n_gathered_fr_cache += 1;
                } // else ready_to_send is still false 
              },
              BuildStrategy::Adaptive 
              | BuildStrategy::AdaptiveThorough
              | BuildStrategy::AdaptiveGreedy(_)
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
            let ev_to_send = event_cache.remove(&evid).unwrap();
            n_rbs_per_ev  += ev_to_send.rb_events.len(); 
            // can we avoid unpacking and repacking?
            let pack       = TofPacket::from(&ev_to_send);
            match data_sink.send(pack) {
              Err(err) => {
                error!("Packet sending failed! Err {}", err);
                n_sent_ch_err += 1; 
              }
              Ok(_)    => {
                debug!("Event with id {} send!", evid);
                n_sent += 1;
              }
            }
          } else {
            event_id_cache.push_front(evid);
          }
        }
      }
    } // end loop over event_id_cache
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
}

