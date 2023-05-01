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
                              MasterTriggerMapping,
                              TofEvent};
use crate::constants::EVENT_BUILDER_EVID_CACHE_SIZE;
use tof_dataclasses::packets::{PacketType,
                               TofPacket};

use crossbeam_channel as cbc;

use tof_dataclasses::packets::paddle_packet::PaddlePacket;
use tof_dataclasses::commands::TofCommand;



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
                         paddle_cache  : &mut HashMap::<u32,Vec::<PaddlePacket>>,
                         data_sink     : &cbc::Sender<TofPacket>) { 
                         //socket        : &zmq::Socket) {

  for ev in event_cache.iter_mut() {
    trace!("Event {} is {} sec old", ev.event_id, ev.age());
    let start   = Instant::now();
    let timeout = Duration::from_micros(timeout_micro);
    if !ev.valid {
      continue;
    }
    

    // first check, if we have pps in the paddle_cache
    if paddle_cache.contains_key(&ev.event_id) {
      let pps = paddle_cache.get_mut(&ev.event_id).unwrap();
      //for pp in pps.iter_mut() {
      for k in 0..pps.len() {
        if !pps[k].valid
          {continue;}
        ev.add_paddle(pps[k]);
        pps[k].invalidate();
      }
    }
    // events must be valid at this point
    if ev.is_ready_to_send(use_timeout) {
      (*ev).valid = false;
      let bytestream = ev.to_bytestream();
      //error!("Event ready to send, we have {} {} {} {}", ev.n_paddles, ev.n_paddles_expected, ev.age(), ev.paddle_packets.len());
      //println!("{:?}", ev.paddle_packets);
      //error!("We have a bytestream len of {}", bytestream.len());
      let mut pack = TofPacket::new();
      pack.packet_type = PacketType::TofEvent;
      pack.payload = bytestream;
      if ev.n_paddles > 0 {
        trace!("{:?}", ev);
        trace!("{:?}", ev.paddle_packets);
      }
      match data_sink.send(pack) {
        Err(err) => error!("Packet sending failed! Err {}", err),
        Ok(_)    => debug!("Event {} with {} paddles sent and {} paddles were expected!", ev.event_id, ev.paddle_packets.len(),ev.n_paddles) 
      }
      continue;
    } // end ready to send
    evid_query.send(Some(ev.event_id));
    let mut n_received = 0;
    let mut n_new      = 0;
    let mut n_seen    = 0;
    while start.elapsed() < timeout {
      match pp_recv.try_recv() { 
        Err(_) => {}
        Ok(pp_option) => {
          match pp_option {
            None => {
              continue;
            },
            Some(pp) => {
              if pp.event_id == ev.event_id {
                n_received += 1;
                match ev.add_paddle(pp) {
                  Err(err) => error!("Can not add paddle to event {}, Error {err}", ev.event_id),
                  Ok(_)    => ()
                }
              } else {
                if paddle_cache.contains_key(&pp.event_id) {
                  paddle_cache.get_mut(&pp.event_id).unwrap().push(pp);
                  //println!("{:?} inserting pp with", pp);
                  n_new += 1;
                } else {
                  let ev_paddles = vec![pp];
                  paddle_cache.insert(pp.event_id, ev_paddles);
                  //println!("{:?} inserting pp with", pp);
                  //println!("{:?}", paddle_cache[&pp.event_id].len());
                  //println!("{:?}", paddle_cache);
                  n_seen += 1;
                }
              }
            }
          }
        } 
      } // end while
    } // end while not timeout
  } // end iter over cache
  // clean the cache - remove all completed events
  if clean_up {
    let size_b4 = event_cache.len();
    event_cache.retain(|ev| ev.valid);
    //paddle_cache.retain(|pp| pp.valid);
    let size_af = event_cache.len();
    println!("==> [EVTBLD::CACHE] Size of event cache before {} and after clean up {}", size_b4, size_af);
  }
}



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
/// * pp_query       : Send request to a paddle_packet cache to send
///                    Paddle packets with the given event id
/// * paddle_packets : Receive paddle_packets from a paddle_packet
///                    cache
/// * cmd_sender     : Sending tof commands to comander thread. This 
///                    is needed so that the event builder can request
///                    paddles from readout boards.
pub fn event_builder (m_trig_ev      : &cbc::Receiver<MasterTriggerEvent>,
                      mt_mapping     : MasterTriggerMapping,
                      pp_query       : &Sender<Option<u32>>,
                      pp_recv        : &Receiver<Option<PaddlePacket>>,
                      settings       : &cbc::Receiver<TofEventBuilderSettings>,
                      data_sink      : &cbc::Sender<TofPacket>,
                      cmd_sender     : &cbc::Sender<TofCommand>) {
                      //socket         : &zmq::Socket) {

  let mut event_cache = VecDeque::<TofEvent>::with_capacity(EVENT_BUILDER_EVID_CACHE_SIZE);
  let mut paddle_cache : HashMap<u32,Vec::<PaddlePacket>> = HashMap::with_capacity(100); 

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
        //println!("{:?}", mt.board_mask);
        let rbs_in_ev = mt_mapping.get_rb_ids(&mt);
        //println!("{:?}", mt);
       // println!("[EVT-BLDR] Get the following RBs in this event {:?}", rbs_in_ev);
        let mut event = TofEvent::from(&mt);
        if (event.event_id != last_evid + 1) {
          let delta_id = event.event_id - last_evid;
          error!("We skipped event ids {}", delta_id );
        }
        last_evid = event.event_id;
        event_cache.push_back(event);
        // we will push the MasterTriggerEvent down the sink
        let tp = TofPacket::from(&mt);
        data_sink.try_send(tp);
      }
    } // end match Ok(mt)
    if n_iter  == 500 {
      build_events_in_cache(&mut event_cache, timeout_micro,
                            pp_query,
                            pp_recv,
                            true,
                            use_timeout,
                            &mut paddle_cache, 
                            &data_sink);
                            //&socket);
      n_iter = 0;
    }
    n_iter += 1;
  } // end loop
}

