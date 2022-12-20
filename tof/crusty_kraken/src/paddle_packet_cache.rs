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
use crate::constants::{PADDLE_PACKET_CACHE_SIZE,
                       EVENT_BUILDER_EVID_CACHE_SIZE};
///
///
///
///
pub fn paddle_packet_cache (evid_rec : &Receiver<u32>,
                            pp_rec   : &Receiver<PaddlePacket>,
                            pp_send  : &Sender<PaddlePacket>) {
  let mut pp_cache           = VecDeque::<PaddlePacket>::with_capacity(PADDLE_PACKET_CACHE_SIZE);
  // received event ids from the eventbuilder, 
  // which have to be worked on
  let mut m_evid_cache = VecDeque::<u32>::with_capacity(EVENT_BUILDER_EVID_CACHE_SIZE); 
  loop {
    match pp_rec.try_recv() {
      Ok(pp) => {
        trace!("Got paddle packet for event {}", pp.event_id);
        pp_cache.push_back(pp);
      }
      Err(_) => {continue;}
    } // end match
    match evid_rec.try_recv() {
      Ok(evid) => {
        trace!("We received a request from the eventbuilder to send pp for evid {}", evid);
        let mut n_paddles_sent = 0;
        for pp in pp_cache.iter_mut() {
          if pp.event_id == evid {
            pp_send.send(*pp);
            pp.invalidate();
            n_paddles_sent += 1;
          } 
        }
      }
      Err(_) => {continue;}
    }
  // FIXME - find something faster!
  // I saw comments that retain might be very slow
  pp_cache.retain(|&x| x.valid);
  trace!("Size of event id cache {}", m_evid_cache.len());
  trace!("Size of paddle_cache   {}", pp_cache.len());
  } // end loop
}
