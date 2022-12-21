///
///
///
///
///
///


use std::sync::mpsc::{Sender,
                      Receiver};

use std::collections::VecDeque;

use crate::reduced_tofevent::PaddlePacket;
use crate::master_trigger::MasterTriggerEvent;
use crate::constants::PADDLE_PACKET_CACHE_SIZE;


///! The cache stores paddle packets as they 
///  are coming from the readoutboard threads.
///
///  It will send the paddle packets it has 
///  for every id it receives while being 
///  agnostic about if that is enough paddles
///  to complete the event.
///  This responsibility is by the 
///  event_builder 
///
pub fn paddle_packet_cache (evid_rec    : &Receiver<Option<u32>>,
                            pp_rec      : &Receiver<PaddlePacket>,
                            pp_send     : &Sender<Option<PaddlePacket>>) {
  
  let mut pp_cache           = VecDeque::<PaddlePacket>::with_capacity(PADDLE_PACKET_CACHE_SIZE);
  //// received event ids from the eventbuilder, 
  //// which have to be worked on
  ////let mut m_evid_cache = VecDeque::<MasterTriggerEvent>::with_capacity(EVENT_BUILDER_EVID_CACHE_SIZE); 
  loop {

    // every iteration, we welcome new paddle packets
    // and keep them. Let's try to receive a certain 
    // number of paddles, and then move on
    let n_tries = 20;
    let mut try = 0;
    match pp_rec.try_recv() {
      Ok(pp) => {
        trace!("Got paddle packet for event {}", pp.event_id);
        pp_cache.push_back(pp);
      }
      Err(_) => {
        try += 1;
        if try == n_tries {
          try = 0;
          continue;
        }
      } // end Err
    } // end match

    
    // after we recieved the paddles,
    // let's try to answer event id requests.
    match evid_rec.try_recv() {
      Err(_)          => {
        continue;
      },
      Ok(evid_option) => {
        match evid_option {
          None => {
            // just send the first entry from the cach
            if pp_cache.len() == 0 {
              continue;
            }

            let mut pp = pp_cache.pop_front().unwrap();
            if pp.valid { 
              pp_send.send(Some(pp));
              pp.invalidate();
            } else {
              pp_send.send(None);
            }
            continue;
          }, // end None
        Some(evid) => {
          let mut n_paddles_sent = 0;
          for pp in pp_cache.iter_mut() {
            if pp.event_id == evid {
              if pp.valid {
                pp_send.send(Some(*pp));
                pp.invalidate();
                n_paddles_sent += 1;
              }
            } 
          } // end for
          // if we did not find it, send None
          pp_send.send(None);
          trace!("We received a request from the eventbuilder to send pp for evid {} and have send {} packets", evid, n_paddles_sent);
          //if (n_paddles_sent == event.n_paddles) {
          //  // the event is complete!
          //  trace!("Send all {} packets for event {}", n_paddles_send, event.event_id);
          //} else {
          //  // we have to store the event in the cache
          //  m_evid_cache.push_back(event);
          //}
        } // end some
      } // end match
    } // end ok
  } // end match
  // FIXME - find something faster!
  // I saw comments that retain might be very slow
  pp_cache.retain(|&x| x.valid);
  //trace!("Size of event id cache {}", m_evid_cache.len());
  trace!("Size of paddle_cache   {}", pp_cache.len());
  } // end loop
}
