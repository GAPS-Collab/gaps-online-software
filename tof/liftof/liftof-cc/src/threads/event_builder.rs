//! Assemble paddle packets to full events
//!
//!
//!

use crossbeam_channel::{Sender,
                        Receiver};

use std::collections::VecDeque;
use std::collections::HashMap;

use std::time::{Duration, 
                Instant};

use tof_dataclasses::events::{MasterTriggerEvent,
                              MasterTofEvent,
                              RBEvent};
use crate::constants::EVENT_BUILDER_EVID_CACHE_SIZE;
use tof_dataclasses::packets::{PacketType,
                               TofPacket};
use crossbeam_channel as cbc;

use tof_dataclasses::packets::paddle_packet::PaddlePacket;
use tof_dataclasses::serialization::Serialization;


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

/// The event builder, assembling events from an id given by the 
/// master trigger
///
/// This requires the master trigger sending `MasterTriggerEvents`
/// over the channel. The event builder then will querey the 
/// paddle_packet cache for paddle packets with the same event id
/// Queries which can not be satisfied will lead to events being 
/// cached until they can be completed, or discarded after 
/// a timeout.
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
pub fn event_builder (m_trig_ev      : &cbc::Receiver<MasterTriggerEvent>,
                      ev_from_rb     : &Receiver<RBEvent>,
                      data_sink      : &cbc::Sender<TofPacket>) { 

  let mut event_cache = VecDeque::<MasterTofEvent>::with_capacity(EVENT_BUILDER_EVID_CACHE_SIZE);

  // timeout in microsecnds
  let timeout_micro = 100;
  let use_timeout   = true;
  let mut n_iter = 0; // don't worry it'll be simply wrapped around
  // we try to receive eventids from the master trigger
  let mut last_evid = 0;
  loop {

    // every iteration, we welcome a new master event
    match m_trig_ev.try_recv() {
      Err(_) => {
        trace!("No new event ready yet!");
        continue;
      }   
      Ok(mt) => {
        trace!("Got master trigger for event {} with {} expected hit paddles"
               , mt.event_id
               , mt.n_paddles);
        // construct RB requests

        let event = MasterTofEvent::from(&mt);
        if event.mt_event.event_id != last_evid + 1 {
          let delta_id = event.mt_event.event_id - last_evid;
          error!("We skipped event ids {}", delta_id );
        }
        last_evid = event.mt_event.event_id;
        event_cache.push_back(event);
        //// we will push the MasterTriggerEvent down the sink
        //let tp = TofPacket::from(&mt);
        //match data_sink.try_send(tp) {
        //  Err(err) => {
        //    error!("Unable to send tof packet to data sink! {err}");
        //  },
        //  Ok(_)    => ()
        //}
      }
    } // end match Ok(mt)
    // check this timeout
    let mut n_received = 0usize;
    while !ev_from_rb.is_empty() || n_received < 20 {
      match ev_from_rb.recv() {
        Err(err) => {
          error!("Can't receive RBEvent! Err {err}");
        },
        // FIXME - remove the clone!!
        Ok(rb_ev) => {
          for ev in event_cache.iter_mut() {
            if ev.mt_event.event_id == rb_ev.header.event_id {
              ev.rb_events.push(rb_ev.clone());
            }
          }
          n_received += 1;
        }
      }
    }
    for ev in event_cache.iter() {
      if ev.is_complete() || ev.has_timed_out() {
        let pack = TofPacket::from(ev);
        match data_sink.send(pack) {
          Err(err) => {
            error!("Packet sending failed! Err {}", err);
          }
          Ok(_)    => {
            debug!("Event with id {} send!", ev.mt_event.event_id);
          }
        }
      }
    }

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

