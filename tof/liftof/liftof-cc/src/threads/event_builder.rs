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
use tof_dataclasses::packets::TofPacket;

/////  Walk over the event cache and check for each event
/////  if new paddles can be added.
/////
/////  # Arguments:
/////
/////
/////  * clean_up : call vec::retain to only keep events which 
/////               have not yet been sent.
//fn build_events_in_cache(event_cache   : &mut VecDeque<MasterTofEvent>,
//                         timeout_micro : u64,
//                         evid_query    : &Sender<Option<u32>>,
//                         pp_recv       : &Receiver<Option<PaddlePacket>>,
//                         clean_up      : bool,
//                         use_timeout   : bool,
//                         paddle_cache  : &mut HashMap::<u32,Vec::<PaddlePacket>>,
//                         data_sink     : &cbc::Sender<TofPacket>) { 
//                         //socket        : &zmq::Socket) {
//
//  for ev in event_cache.iter_mut() {
//    trace!("Event {} is {} sec old", ev.event_id, ev.age());
//    let start   = Instant::now();
//    let timeout = Duration::from_micros(timeout_micro);
//    if !ev.valid {
//      continue;
//    }
//    
//
//    // first check, if we have pps in the paddle_cache
//    if paddle_cache.contains_key(&ev.event_id) {
//      let pps = paddle_cache.get_mut(&ev.event_id).unwrap();
//      //for pp in pps.iter_mut() {
//      for k in 0..pps.len() {
//        if !pps[k].valid
//          {continue;}
//        match ev.add_paddle(pps[k]) {
//         Err(err) => error!("Unable to add paddle! {err}"),
//         Ok(_)    => ()
//        }
//        pps[k].invalidate();
//      }
//    }
//    // events must be valid at this point
//    if ev.is_ready_to_send(use_timeout) {
//      (*ev).valid = false;
//      let bytestream = ev.to_bytestream();
//      //error!("Event ready to send, we have {} {} {} {}", ev.n_paddles, ev.n_paddles_expected, ev.age(), ev.paddle_packets.len());
//      //println!("{:?}", ev.paddle_packets);
//      //error!("We have a bytestream len of {}", bytestream.len());
//      let mut pack = TofPacket::new();
//      pack.packet_type = PacketType::TofEvent;
//      pack.payload = bytestream;
//      if ev.n_paddles > 0 {
//        trace!("{:?}", ev);
//        trace!("{:?}", ev.paddle_packets);
//      }
//      match data_sink.send(pack) {
//        Err(err) => error!("Packet sending failed! Err {}", err),
//        Ok(_)    => debug!("Event {} with {} paddles sent and {} paddles were expected!", ev.event_id, ev.paddle_packets.len(),ev.n_paddles) 
//      }
//      continue;
//    } // end ready to send
//    match evid_query.send(Some(ev.event_id)) {
//      Err(err) => error!("Can not send even query for event {}, {}", ev.event_id, err),
//      Ok(_)    =>()
//    }
//    let mut n_received = 0;
//    let mut n_new      = 0;
//    let mut n_seen    = 0;
//    while start.elapsed() < timeout {
//      match pp_recv.try_recv() { 
//        Err(_) => {}
//        Ok(pp_option) => {
//          match pp_option {
//            None => {
//              continue;
//            },
//            Some(pp) => {
//              if pp.event_id == ev.event_id {
//                n_received += 1;
//                match ev.add_paddle(pp) {
//                  Err(err) => error!("Can not add paddle to event {}, Error {err}", ev.event_id),
//                  Ok(_)    => ()
//                }
//              } else {
//                if paddle_cache.contains_key(&pp.event_id) {
//                  paddle_cache.get_mut(&pp.event_id).unwrap().push(pp);
//                  //println!("{:?} inserting pp with", pp);
//                  n_new += 1;
//                } else {
//                  let ev_paddles = vec![pp];
//                  paddle_cache.insert(pp.event_id, ev_paddles);
//                  //println!("{:?} inserting pp with", pp);
//                  //println!("{:?}", paddle_cache[&pp.event_id].len());
//                  //println!("{:?}", paddle_cache);
//                  n_seen += 1;
//                }
//              }
//            }
//          }
//        } 
//      } // end while
//    } // end while not timeout
//    debug!("n_seen {n_seen}, n_new {n_new} n_received {n_received}");
//  } // end iter over cache
//  // clean the cache - remove all completed events
//  if clean_up {
//    let size_b4 = event_cache.len();
//    event_cache.retain(|ev| ev.valid);
//    //paddle_cache.retain(|pp| pp.valid);
//    let size_af = event_cache.len();
//    println!("==> [EVTBLD::CACHE] Size of event cache before {} and after clean up {}", size_b4, size_af);
//  }
//}



/// Settings to change the configuration of the TOF Eventbuilder on the fly
pub struct TofEventBuilderSettings {
  pub cachesize         : usize,
  pub build_interval    : usize,
  pub use_mastertrigger : bool
}

impl TofEventBuilderSettings {
  pub fn new() -> TofEventBuilderSettings {
    TofEventBuilderSettings {
      cachesize         : 100000,
      build_interval    : 1000,
      use_mastertrigger : true
    }
  }
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
/// * dsi_j_mapping  : A HashMap of some Hashmaps which makes the 
///                    following connection DSI/J/LTB_CH -> RBID, RBCH
/// * pp_query       : Send request to a paddle_packet cache to send
///                    Paddle packets with the given event id
/// * paddle_packets : Receive paddle_packets from a paddle_packet
///                    cache
/// * cmd_sender     : Sending tof commands to comander thread. This 
///                    is needed so that the event builder can request
///                    paddles from readout boards.
/// * nrb_failsafe   : This is kind of a failsafe mode. In case we are 
///                    not deciding by fw how many RBEvents we expect 
///                    for each event, we can set this number manually.
///                    THIS SHOULD NOT BE TEMPERED WITH IN NORMAL OPERATIONS
///                    AND SHOULD BE SET TO NONE!

pub fn event_builder (m_trig_ev      : &Receiver<MasterTriggerEvent>,
                      ev_from_rb     : &Receiver<RBEvent>,
                      data_sink      : &Sender<TofPacket>,
                      nrb_failsafe   : Option<usize>) { 

  //let mut event_cache = VecDeque::<TofEvent>::with_capacity(EVENT_BUILDER_EVID_CACHE_SIZE);
  let mut event_cache = HashMap::<u32, TofEvent>::new();
  let mut event_id_cache = VecDeque::<u32>::with_capacity(EVENT_BUILDER_EVID_CACHE_SIZE);
  // timeout in microsecnds
  //let timeout_micro = 100;
  //let use_timeout   = true;
  //let mut n_iter    = 0; // don't worry it'll be simply wrapped around
  // we try to receive eventids from the master trigger
  let n_mte_per_loop         = 1;
  let n_rbe_per_loop         = 40;
  let send_every_x_event     = 200usize;

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

  // Depending on the MasterTrigger Rate, 
  // we want to change the interval when 
  // we send out new events

  loop {
    n_received = 0;
    let debug_timer = Instant::now();

    while n_received < n_mte_per_loop {
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
          //event_cache.push_back(event);
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
    //first_evid = event_cache[0].mt_event.event_id; 
    
    // check this timeout
    //let mut rb_events_added   = 0usize;
    //let mut iter_ev           = 0usize;
    //let mut rb_events_dropped = 0usize;
    n_received = 0;
    'main: while !ev_from_rb.is_empty() && n_received < n_rbe_per_loop {
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
    if n_mte_received_tot % 100 == 0 {
      println!("[EVTBLDR] ==> Received {} MTE", n_mte_received_tot);
      println!("[EVTBLDR] ==> Received {n_rbe_received_tot} RBEvents!");
      println!("[EVTBLDR] ==> Delta Last MTE evid - Last RB evid  {}", last_evid - last_rb_evid);
      println!("[EVTBLDR] ==> Size of event cache {}", event_cache.len());
      println!("[EVTBLDR] ==> Size of event ID cache {}", event_id_cache.len());
      println!("[EVTBLDR] ==> Sent {n_sent} events!");
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

            match nrb_failsafe {
              None => {
                // normal operations
                if ev.is_complete() || ev_timed_out {
                  cache_it = false;
                }
              },
              Some(n_rb_ev) => {
                if ev_timed_out || ev.rb_events.len() == n_rb_ev {
                  cache_it = false;
                } else {
                  cache_it = true;
                }
              }
            }
            if cache_it {
              event_id_cache.push_back(evid);
            } else {
              let ev_to_send = event_cache.remove(&evid).unwrap();
              n_rbs_per_ev += ev_to_send.rb_events.len(); 
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

