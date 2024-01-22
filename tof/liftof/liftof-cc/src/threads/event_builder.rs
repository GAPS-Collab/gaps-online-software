use std::time::Instant;

use crossbeam_channel::{
    Receiver,
    Sender,
};

use std::collections::VecDeque;
use std::collections::HashMap;

use tof_dataclasses::events::{MasterTriggerEvent,
                              TofEvent,
                              RBEvent};
use crate::constants::EVENT_BUILDER_EVID_CACHE_SIZE;
use crate::settings::{
    TofEventBuilderSettings,
    BuildStrategy
};
use tof_dataclasses::packets::TofPacket;


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
/// * dsi_j_mapping  : A HashMap of some Hashmaps which makes the 
///                    following connection DSI/J/LTB_CH -> RBID, RBCH
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
                      mut settings   : TofEventBuilderSettings) { 
  // event caches for assembled events
  let mut event_cache      = HashMap::<u32, TofEvent>::new();
  let mut event_id_cache   = VecDeque::<u32>::with_capacity(EVENT_BUILDER_EVID_CACHE_SIZE);
  
  let mut n_received         : usize;
  let mut clear_cache        = 0; // clear cache every 
  let mut event_sending      = 0;
  let mut n_mte_received_tot = 0u64;
  let mut n_rbe_received_tot = 0u64;
  let mut first_evid         = 0u32;
  let mut last_evid          = 0;
  let mut n_sent             = 0usize;
  let mut n_timed_out        = 0usize; 
  // debug
  let mut last_rb_evid       = 0u32;
  let mut n_rbs_per_ev       = 0usize;
  let mut rb_ev_wo_mte       = 0usize;
  loop {
    n_received = 0;
    let debug_timer = Instant::now();
    while n_received < settings.n_mte_per_loop {
      // every iteration, we welcome a new master event
      match m_trig_ev.try_recv() {
        Err(_) => {
          trace!("No new event ready yet!");
          continue;
        }   
        Ok(mt) => {
          debug!("Got master trigger for event {} with {} expected hit paddles"
                 , mt.event_id
                 , mt.n_paddles);
          // construct RB requests

          let event = TofEvent::from(&mt);
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
              ev.rb_events.push(rb_ev);
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
    if settings.build_strategy == BuildStrategy::Adaptive {
      settings.n_rbe_per_loop = av_rb_ev.ceil() as usize;
      if settings.n_rbe_per_loop == 0 {
        // failsafe
        settings.n_rbe_per_loop = 40;
      }
    }
    if n_mte_received_tot % 10000 == 0 {
      println!("[EVTBLDR] ==> Received {} MTE", n_mte_received_tot);
      println!("[EVTBLDR] ==> Received {n_rbe_received_tot} RBEvents!");
      println!("[EVTBLDR] ==> Delta Last MTE evid - Last RB evid  {}", last_evid - last_rb_evid);
      println!("[EVTBLDR] ==> Size of event cache {}", event_cache.len());
      println!("[EVTBLDR] ==> Size of event ID cache {}", event_id_cache.len());
      println!("[EVTBLDR] ==> Get MTE from cache for RB ev failed {rb_ev_wo_mte} times!");
      println!("[EVTBLDR] ==> Sent {n_sent} events!");
      println!("[EVTBLDR] ==> Chn len MTE receiver {}",m_trig_ev.len() );
      println!("[EVTBLDR] ==> Chn len RBE receiver {}",ev_from_rb.len());
      println!("[EVTBLDR] ==> Chn len TP  sender   {}",data_sink.len());
      
      if n_sent > 0 {
        let av_rb_ev = n_rbs_per_ev as f64 / n_sent as f64;
        println!("[EVTBLDR] ==> Average number of RBEvents/TofEvent {:4.2}", av_rb_ev);
      }
      if n_mte_received_tot > 0 {
        let to_frac = n_timed_out as f64 / n_mte_received_tot as f64;
        println!("[EVTBLDR] ==> Fraction of timed out events {:4.2}", to_frac);
      }
      //println!("[EVTBLDR] ==> Last RB evid {last_rb_evid}");
    }
    trace!("Debug timer RBE received! {:?}", debug_timer.elapsed());
    //if event_sending == send_every_x_event {
    if true {
      //for ev in event_cache.iter_mut() {
      let this_cache_size = event_id_cache.len();
      for k in 0..this_cache_size {
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
            let ev_timed_out = ev.has_timed_out();
            let mut cache_it = false;
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
              let pack = TofPacket::from(&ev_to_send);
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
      event_sending = 0;
      //event_cache.retain(|ev| ev.valid);
      debug!("Debug timer! EVT SENDING {:?}", debug_timer.elapsed());
    } 

    //// remove sent events! 
    //if clear_cache == 500 {
    //  event_cache.retain(|ev| ev.valid);
    //  clear_cache = 0;
    //}
    //clear_cache += 1
    //if n_iter  == 500 {
    //  build_events_in_cache(&mut event_cache, timeout_micro,
    //                        pp_query,
    //                        pp_recv,
    //                        true,
    //                        use_timeout,
    //                        &mut paddle_cache, 
    //                        &data_sink);
    //                        //&socket);
    //  n_iter = 0;
    //}
    //n_iter += 1;
  } // end loop
}

