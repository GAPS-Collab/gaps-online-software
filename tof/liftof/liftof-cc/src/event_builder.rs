//! Assemble paddle packets to full events
//!
//!
//!


use std::sync::mpsc::{Sender,
                      Receiver};
use std::collections::VecDeque;
//use std::collections::HashMap;

use std::time::{Duration, 
                Instant};

use crate::reduced_tofevent::TofEvent;
use crate::master_trigger::MasterTriggerEvent;
use crate::constants::EVENT_BUILDER_EVID_CACHE_SIZE;
use tof_dataclasses::packets::{PacketType,
                               TofPacket};

use crossbeam_channel as cbc;

use tof_dataclasses::packets::paddle_packet::PaddlePacket;

///  Walk over the event cache and check for each event
///  if new paddles can be added.
///
///  # Arguments:
///
///
///  * clean_up : call vec::retain to only keep events which 
///               have not yet been sent.
fn build_events_in_cache(event_cache   : &mut VecDeque<TofEvent>,
                         timeout_micro : u64,
                         evid_query    : &Sender<Option<u32>>,
                         pp_recv       : &Receiver<Option<PaddlePacket>>,
                         clean_up      : bool,
                         use_timeout   : bool, 
                         data_sink     : &cbc::Sender<TofPacket>) { 
                         //socket        : &zmq::Socket) {

  for ev in event_cache.iter_mut() {
    let start   = Instant::now();
    let timeout = Duration::from_micros(timeout_micro)
                  .as_micros();
    if !ev.valid {
      continue;
    }
    evid_query.send(Some(ev.event_id));
    while start.elapsed().as_micros() < timeout {
      match pp_recv.try_recv() { 
        Err(_) => {}
        Ok(pp_option) => {
          match pp_option {
            None => {
              continue;
            },
            Some(pp) => {
              ev.add_paddle(pp);
            }
          }
        }
      } // end match

      if ev.is_ready_to_send(use_timeout) {
        (*ev).valid = false;
        let bytestream = ev.to_bytestream();
        let mut pack = TofPacket::new();
        pack.packet_type = PacketType::TofEvent;
        pack.payload = bytestream;

        match data_sink.send(pack) {
          Err(err) => warn!("Packet sending failed! Err {}", err),
          Ok(_)    => trace!("Event {} sent!", ev.event_id) 
        }
      }
      //error!("Not implemented!! We have to do something with the event, but we don!");
      //break;
    } // end while not timeout
  } // end iter over cache
  // clean the cache - remove all completed events
  if clean_up {
    let size_b4 = event_cache.len();
    event_cache.retain(|ev| ev.valid);
    let size_af = event_cache.len();
    info!("Size of event cache before {} and after clean up {}", size_b4, size_af);
  }
}



//fn paddle_query (timeout_in_mus : u64,
//                 event_id : u32,
//                 event    : &mut TofEvent,
//                 pp_query : &Receiver<PaddlePacket>) {
//
//  let start = Instant::now();
//  let timeout = Duration::from_micros(timeout_in_mus)
//                .as_micros();
//  pp_query.send(event.event_id);
//  while (start.elapsed().as_millis() < timeout) {
//    let mut n_pad = 0;
//    match pp_query.try_recv() { 
//      Err(_) => {}
//      Ok(pp) => {
//        event.paddle_packets.push(pp);
//        n_pad += 1
//      }
//    } // end match
//    if event.is_complete() {
//      trace!("Event {} building complete!", event.n_paddles);
//      break;
//    }
//  } // end while not timeout
//} // end fn

///  An event builder which works without the 
///  master trigger. 
///
///  The event id is coming from the readout board 
///  blobs, as these might not be coming in sequence,
///  Paddle packets have to be received and ordered.
///  The event is declared as "finished" when we have
///  incoming blobls from all readout boards and we 
///  can check that the time has passed
///
///  # Arguments
///
///  * pp_query  - a std::net::Sender, which is expected
///                to have its receiver with a paddle_cache.
///                The pp_query will be used to ask the 
///                cache to send the packets for a certain
///                event id it has in store
///
//pub fn event_builder_no_master(evid_query : &Sender<Option<u32>>,
//                               pp_recv    : &Receiver<Option<PaddlePacket>>,
//                               socket     : &zmq::Socket) {
//
//  info!("Initializing event builder without master trigger support!");
//  let clock = Instant::now();
//
//  let mut event_cache = VecDeque::<TofEvent>::with_capacity(EVENT_BUILDER_EVID_CACHE_SIZE);
//  let timeout_micro : u64 = 2000;
//
//  let mut n_packets = 0usize;
//  let max_packets   : usize  = 10;
//  let mut n_iter = 0; // don't worry, it'll simply get wrapped around
//
//  //// how many new events per 
//  //// iteration do we want to allow?
//  //let max_new_events : usize = 100;
//  //let evid_seen = [0
//
//  loop {
//    let mut event = TofEvent::new(0,0);
//    // we set the number of expected paddles to 
//    // unrealistic high number (max u8)
//    event.n_paddles_expected = 255; 
//    match evid_query.send(None) {
//      Err(_) => {continue;},
//      Ok(_) => {
//        match pp_recv.recv() {
//          Err(err) => {
//            error!("Connection error or nothing in channel!");
//            continue;
//          },
//          Ok(pp_option) => {
//            match pp_option {
//              None => {
//                continue;
//              },
//              Some(pp) => {
//                event.event_id = pp.event_id;
//                event.add_paddle(pp);
//                trace!("Have event with event id {}", event.event_id);
//                n_packets += 1;
//              }
//            } // end inner match
//          } // end ok
//        }// end match
//      } // end outer ok
//    } // end match
//    //if n_packets == max_packets {
//    //  break;
//    //}
//    event_cache.push_back(event);
//    if n_iter % 100 == 0 {
//      build_events_in_cache(&mut event_cache,
//                            timeout_micro,
//                            evid_query,
//                            pp_recv,
//                            true,
//                            true,
//                            &socket);
//    } else {
//      build_events_in_cache(&mut event_cache,
//                            timeout_micro,
//                            evid_query,
//                            pp_recv,
//                            false,
//                            true,
//                            &socket);
//    }
//    n_iter += 1;
//  } // end loop
//  //  event.event_id = pp.event_id;
//  //  event.paddle_packets.push(pp);
//  //  event.timeout = clock.elapsed().as_micros();
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
/// * master_id : Receive a `MasterTriggerEvent` over this 
///               channel. The event will be either build 
///               immediatly, or cached. 
///
/// * pp_query       : Send request to a paddle_packet cache to send
///                    Paddle packets with the given event id
/// * paddle_packets : Receive paddle_packets from a paddle_packet
///                    cache
///
pub fn event_builder (master_id      : &Receiver<MasterTriggerEvent>,
                      pp_query       : &Sender<Option<u32>>,
                      pp_recv        : &Receiver<Option<PaddlePacket>>,
                      settings       : &cbc::Receiver<TofEventBuilderSettings>,
                      data_sink      : &cbc::Sender<TofPacket>) {
                      //socket         : &zmq::Socket) {

  let mut event_cache = VecDeque::<TofEvent>::with_capacity(EVENT_BUILDER_EVID_CACHE_SIZE);

  // timeout in microsecnds
  let timeout_micro = 100;
  let mut n_iter = 0; // don't worry it'll be simply wrapped around
  // we try to receive eventids from the master trigger
  loop {
   //// let's work on our backlog and check if we can complete 
   //// events
   //for ev in event_cache.iter_mut() {
   //  let start = Instant::now();
   //  let timeout = Duration::from_micros(timeout_micro)
   //                .as_micros();
   //  pp_query.send(Some(ev.event_id));
   //  while start.elapsed().as_micros() < timeout {
   //    let mut n_pad = 0;
   //    match paddle_packets.try_recv() { 
   //      Err(_) => {}
   //      Ok(pp_or_none) => {
   //        match pp_or_none {
   //          Some(pp) => {
   //            ev.paddle_packets.push(pp);
   //            n_pad += 1
   //          },
   //          None => {
   //            break;
   //          }
   //        }
   //      }
   //    } // end match
   //    if ev.is_complete() {
   //      trace!("Event {} building complete!", ev.event_id);
   //      break; // on to the next event in cache
   //    }
   //  } // end while not timeout
   //}
   //// clean the cache - remove all completed events
   //event_cache.retain(|ev| !ev.is_complete());

   // every iteration, we welcome a new master event
   match master_id.try_recv() {
     Err(_) => {
       trace!("No new event ready yet!");
       continue;
     }   
     Ok(mt) => {
       trace!("Got trigger for event {} with {} expected hit paddles"
              , mt.event_id
              , mt.n_paddles);
       let mut event = TofEvent::new(mt.event_id, mt.n_paddles);
       // let's query the paddle packet cache for a certain time
       // and then move on and try later again      
       let start = Instant::now();
       let timeout = Duration::from_micros(timeout_micro)
                     .as_micros();
       pp_query.send(Some(mt.event_id));
       while start.elapsed().as_micros() < timeout {
         let mut n_pad = 0;
         match pp_recv.try_recv() { 
           Err(_) => {}
           Ok(pp_or_none) => {
             match pp_or_none {
               Some(pp) => {
                 event.add_paddle(pp);
                 n_pad += 1
               }
               None => {
                 break;
               }
             } 
           }
         } // end match
         if event.is_complete() {
           trace!("Event {} building complete!", mt.event_id);
           break;
         }
       } // end while not timeout
       event_cache.push_back(event);
       //if event.paddle_packets.len() == mt.n_paddles as usize {
       //  trace!("Event {} building complete!", mt.event_id);
       //  continue; // on to the next mt event
       //} else {
       //  // we have to put the event on the stack and try
       //  // again later
       //}
     }
    } // end match Ok(mt)
    if n_iter % 1000 == 0 {
      build_events_in_cache(&mut event_cache, timeout_micro,
                            pp_query,
                            pp_recv,
                            true,
                            false,
                            &data_sink);
                            //&socket);
    }
    n_iter += 1;
  } // end loop


//  let n_events_backlog = 10;
//
//  let mut last_event_id : u32;
//  //
//  let mut events_backlog_prev = VecDeque::<PaddlePacket>::with_capacity(100);
//  let mut events_backlog_new  = VecDeque::<PaddlePacket>::with_capacity(100);
//  //let mut events_backlog : HashMap<u32,TofEvent> = HashMap::with_capacity(100);
//
//  // FIXME make this configurable
//  let wait_for_pp_timeout = Duration::from_millis(20).as_millis();
//
//  // the first iteration of the loop waits
//  // till we are in sync
//  let mut first = true;
//  
//  let mut mt_is_behind = false;
//  // the events might not be in order here
//  // so we have to check the incoming event ids
//
//  let mut n_triggers = 0usize;
//  for master_event in master_id {
//    let m_id = master_event.event_id;
//    let n_paddles = master_event.n_paddles;
//    n_triggers += 1;
//    if n_triggers % 100 == 0 {
//      println!("Got {} triggers", n_triggers);
//      println!("Last trigger {}", m_id);
//      println!("Length of backlog {}", events_backlog_prev.len());
//      if events_backlog_prev.len() > 0 {
//        println!("First bl event id {}", events_backlog_prev[0].event_id);
//      }
//    }
//
//    // two scenarios 
//    // 1) the mt is behind
//    // 2) th pps are behind
//    if n_paddles == 0 {
//      error!("Received master event id {}, but there are no hit paddles with it!", m_id);
//      continue;
//    }
//    trace!("Received master event id {} with n_paddles {}", m_id, n_paddles);
//    //println!("==> Received master event id {} with n_paddles {}", m_id, n_paddles);
//    events_backlog_new.clear();
//
//    let mut event = TofEvent::new(m_id, n_paddles as u8);  
//
//
//    while events_backlog_prev.len() > 0 {
//      let pp = events_backlog_prev.pop_front().unwrap();
//      if pp.event_id == m_id {
//        event.paddle_packets.push(pp);
//      } else { 
//        events_backlog_new.push_back(pp);
//      }
//    }
//    
//    if event.is_complete() {
//      println!("==> Event with id {} ready to be sent!", m_id);
//      event.paddle_packets[0].print();
//      continue;
//    }
//
//    let start = Instant::now();
//    while (start.elapsed().as_millis() < wait_for_pp_timeout) || first {
//      //for pp in paddle_packets {
//      match paddle_packets.try_recv() {
//        Ok(pp) => {  
//          trace!("Got pp with id {}, m_id {}", pp.event_id, event.event_id); 
//          if pp.event_id < event.event_id {
//            // if we are here currently we can not do anything
//            // the event is lost
//            break;
//          }
//          if pp.event_id == m_id {
//            event.paddle_packets.push(pp);    
//          } else {
//            events_backlog_new.push_back(pp);
//          }
//          if event.is_complete() {
//            println!("==> Event with id {} ready to be sent!", m_id);
//            event.paddle_packets[0].print();
//            first = false;
//            break;
//          }
//      
//          //// alternatively, if the timeout runs out, 
//          //// break here
//          //if start.elapsed() > wait_for_pp_timeout {
//          //  break;
//          //}
//        },
//        Err(_) => {break;}
//      }
//    trace!("TIMEOUT!");
//    continue;
//    }
//
//    // this is expensive!
//    events_backlog_prev = events_backlog_new.clone();
//  
//  }
}

