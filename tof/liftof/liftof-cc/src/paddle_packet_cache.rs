///
///
///
///
///
///

use std::time::{Instant,
                Duration};
//use std::sync::mpsc::{Sender,
//                      Receiver};
use crossbeam_channel::{Sender,
                        Receiver};
use std::collections::HashMap;

use std::collections::VecDeque;

//use crate::reduced_tofevent::{PaddlePacket,
//                              TofEvent};
use tof_dataclasses::events::TofEvent;
use crate::constants::{PADDLE_PACKET_CACHE_SIZE,
                       EVENT_CACHE_SIZE,
                       EXP_N_PADDLES_PER_EVENT};

use tof_dataclasses::packets::paddle_packet::PaddlePacket;

///////struct EventCache {
///////
///////  event_cache : HashMap::<u32, TofEvent>, 
///////}
///////
///////impl EventCache {
///////
///////  pub fn new() -> EventCache {
///////    EventCache {
///////      event_cache : HashMap::<u32,TofEvent>::with_capacity(EVENT_CACHE_SIZE)
///////    }
///////  }
///////
///////  pub fn init(&mut self) {
///////    for n in 0..EVENT_CACHE_SIZE as u32 {
///////      self.event_cache.get_mut(n) = TofEvent::new(0,0);
///////    }
///////  }
///////
///////  pub fn add_pp(&mut self, pp : PaddlePacket) {
///////    match self.event_cache.get_mut(pp.event_id) {
///////      Err(_) => self.event_cache.insert
///////    }
///////        .add_paddle(pp);
///////  }
///////}
///////
///////pub fn event_cache (evid_rec    : &Receiver<Option<u32>>,
///////                    pp_rec      : &Receiver<PaddlePacket>,
///////                    pp_send     : &Sender<Option<PaddlePacket>>) {
///////}
/////////struct PaddlePacketCache {
/////////
/////////  //event_cache     : [TofEvent;PADDLE_PACKET_CACHE_SIZE],
/////////  //eventid_idx_map : [u32;PADDLE_PACKET_CACHE_SIZE],
/////////  ////packet_cache    : [Vec<PaddlePacket>;PADDLE_PACKET_CACHE_SIZE],
/////////  //idx : usize,
/////////  //first_id : u32
/////////}
/////////
/////////impl PaddlePacketCache {
/////////
/////////  pub fn new() -> PaddlePacketCache {
/////////    //const INIT : Vec::<PaddlePacket> = Vec::<PaddlePacket>::new();
/////////    //let creation_time  = SystemTime::now()
/////////    //                     .duration_since(SystemTime::UNIX_EPOCH)
/////////    //                     .unwrap().as_micros();
/////////
/////////    //const INIT : TofEvent = TofEvent { 
/////////    //  event_id       : 0,
/////////    //  n_paddles      : 0,  
/////////    //  paddle_packets : vec![],
/////////
/////////    //  n_paddles_expected : 0,
/////////
/////////    //  // This is strictly for when working
/////////    //  // with event timeouts
/////////    //  creation_time  : 0,
/////////
/////////    //  valid          : true,
/////////    //};
/////////    //PaddlePacketCache {
/////////    //  event_cache     : [INIT;PADDLE_PACKET_CACHE_SIZE],
/////////    //  eventid_idx_map : [0;PADDLE_PACKET_CACHE_SIZE],
/////////    //  //packet_cache : [INIT;PADDLE_PACKET_CACHE_SIZE],
/////////    //  idx : 0,
/////////    //  first_id       : 0
/////////    //}
/////////  }
/////////
/////////  ///! Initialize the cache
/////////  ///
/////////  ///  Warning, the initial paddle 
/////////  ///  packet will be thrown away!
/////////  ///
/////////  ///
/////////  //pub fn init(&mut self, pp : PaddlePacket) {
/////////    //for n in 0..PADDLE_PACKET_CACHE_SIZE { 
/////////    //  self.eventid_idx_map[n] = 0;
/////////    //  self.event_cache[n]     = TofEvent::new(0,0);
/////////    //}
/////////    //self.idx = 0;
/////////    //self.first_id = pp.event_id;
/////////  //}
/////////
/////////  //fn calculate_event_hash(&mut self, event_id : u32) {
/////////  //  /// assuming a max rate of 1k, we will get
/////////  //  /// 30000 events in 30 sec
/////////  //  let delta_evid  = event_id as i32 - self.first_id as i32;
/////////  //  if delta_evid
/////////  //  self.idx = (event_id - self.first_id) - 1000;
/////////
/////////  //}
/////////
/////////  //pub fn add_packet(&mut self, pp : PaddlePacket) {
/////////  //  // to calculate our event hash
/////////  //  //
/////////
/////////
/////////  //  self.idx += 1;
/////////  //  //self.packet_cache[self.idx].push(pp);
/////////  //}
/////////
/////////}
///////
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
///  # Arguments:
///
///  * 
///
pub fn paddle_packet_cache (evid_rec    : &Receiver<Option<u32>>,
                            pp_rec      : &Receiver<PaddlePacket>,
                            pp_send     : &Sender<Option<PaddlePacket>>) {

  info!("Initializing paddle packet cache!");
  let mut pp_cache           = VecDeque::<PaddlePacket>::with_capacity(PADDLE_PACKET_CACHE_SIZE);
  //// received event ids from the eventbuilder, 
  //// which have to be worked on
  ////let mut m_evid_cache = VecDeque::<MasterTriggerEvent>::with_capacity(EVENT_BUILDER_EVID_CACHE_SIZE); 
  let n_tries = 20;

  let mut start = Instant::now();
  let timeout_micro = Duration::from_micros(100);


  let mut n_iter = 0usize;
  loop {
    n_iter += 1;
    
    // every iteration, we welcome new paddle packets
    // and keep them. Let's try to receive a certain 
    // number of paddles, and then move on
    let mut try = 0;
    start = Instant::now();
    while start.elapsed() < timeout_micro {
      match pp_rec.try_recv() {
        Ok(pp) => {
          trace!("Got paddle packet for event {}", pp.event_id);
          pp_cache.push_back(pp);
        }
        Err(err) => {
          continue;
          //error!("Can not receive paddle packet!, err {}", err);
          //try += 1;
          //if try == n_tries {
          //  try = 0;
          //  continue;
          //}
        } // end Err
      } // end match
    }
    trace!("Size of paddle_cache   {}", pp_cache.len());
    
    // after we received the paddles,
    // let's try to answer event id requests.
    match evid_rec.try_recv() {
      Err(err)          => {
        //error!("Can not receive event id! {}", err);
        continue;
      },
      Ok(evid_option) => {
        match evid_option {
          None => {
            warn!("Did not get an event id!");
            // just send the first entry from the cach
            if pp_cache.len() == 0 {
              pp_send.send(None);
              continue;
            }

            let mut pp = pp_cache.pop_front().unwrap();
            if pp.valid { 
              match pp_send.send(Some(pp)) {
                Err(err) => error!("Could not send paddle package, error {err}"),
                Ok(_)    => ()
              }
              pp.invalidate();
            } else {
              match pp_send.send(None) {
                Err(err) => error!("Could not send NONE value, err {err}"),
                Ok(_)    => ()
              }
            }
            continue;
          }, // end None
        Some(evid) => {
          trace!("Received {evid} event id");
          let mut n_paddles_sent = 0;
          for pp in pp_cache.iter_mut() {
            if pp.event_id == evid {
              if pp.valid {
                match pp_send.send(Some(*pp)) {
                  Err(err) => {
                    error!("Unable to send the paddle package! Err {err}");
                  }
                  Ok(_)     => {
                    trace!("Send pp with evid {}", evid);
                    pp.invalidate();
                    n_paddles_sent += 1;
                  }
                }
              }
            } 
          } // end for
          // if we did not find it, send None
          //match pp_send.send(None) {
          //  Err(err) => trace!("We can not send that paddle packet! Err {err}"),
          //  Ok(_) => ()
          //}
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
  if n_iter % 10000 == 0 {
    let size_b4 = pp_cache.len();
    // FIXME - find something faster!
    // I saw comments that retain might be very slow
    pp_cache.retain(|&x| x.valid);
    let size_af = pp_cache.len();
    println!("==> [PADDLECACHE] Size of paddle_cache {} before and {} after clean up", size_b4, size_af);
  }
  } // end loop
}
