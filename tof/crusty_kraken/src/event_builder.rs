///
///
///
///
///
///
///
///


use std::sync::mpsc::Receiver;
use std::collections::VecDeque;
//use std::collections::HashMap;

use time::{Duration, 
           Instant};

use crate::reduced_tofevent::{PaddlePacket, TofEvent};
///
///
///
///
pub fn event_builder (master_id      : &Receiver<(u32, u32)>,
                      paddle_packets : &Receiver<PaddlePacket>) {

  let n_events_backlog = 10;

  let mut last_event_id : u32;
  //
  let mut events_backlog_prev = VecDeque::<PaddlePacket>::with_capacity(100);
  let mut events_backlog_new  = VecDeque::<PaddlePacket>::with_capacity(100);
  //let mut events_backlog : HashMap<u32,TofEvent> = HashMap::with_capacity(100);

  // FIXME make this configurable
  let wait_for_pp_timeout = Duration::from_millis(20).as_millis();

  // the first iteration of the loop waits
  // till we are in sync
  let mut first = true;
  
  let mut mt_is_behind = false;
  // the events might not be in order here
  // so we have to check the incoming event ids

  let mut n_triggers = 0usize;
  for (m_id, n_paddles) in master_id {
    n_triggers += 1;
    if n_triggers % 100 == 0 {
      println!("Got {} triggers", n_triggers);
      println!("Last trigger {}", m_id);
      println!("Length of backlog {}", events_backlog_prev.len());
      if events_backlog_prev.len() > 0 {
        println!("First bl event id {}", events_backlog_prev[0].event_id);
      }
    }

    // two scenarios 
    // 1) the mt is behind
    // 2) th pps are behind
    if n_paddles == 0 {
      error!("Received master event id {}, but there are no hit paddles with it!", m_id);
      continue;
    }
    trace!("Received master event id {} with n_paddles {}", m_id, n_paddles);
    //println!("==> Received master event id {} with n_paddles {}", m_id, n_paddles);
    events_backlog_new.clear();

    let mut event = TofEvent::new(m_id, n_paddles as u8);  


    while events_backlog_prev.len() > 0 {
      let pp = events_backlog_prev.pop_front().unwrap();
      if pp.event_id == m_id {
        event.paddle_packets.push(pp);
      } else { 
        events_backlog_new.push_back(pp);
      }
    }
    
    if event.is_complete() {
      println!("==> Event with id {} ready to be sent!", m_id);
      event.paddle_packets[0].print();
      continue;
    }

    let start = Instant::now();
    while (start.elapsed().as_millis() < wait_for_pp_timeout) || first {
      //for pp in paddle_packets {
      match paddle_packets.try_recv() {
        Ok(pp) => {  
          trace!("Got pp with id {}, m_id {}", pp.event_id, event.event_id); 
          if pp.event_id < event.event_id {
            // if we are here currently we can not do anything
            // the event is lost
            break;
          }
          if pp.event_id == m_id {
            event.paddle_packets.push(pp);    
          } else {
            events_backlog_new.push_back(pp);
          }
          if event.is_complete() {
            println!("==> Event with id {} ready to be sent!", m_id);
            event.paddle_packets[0].print();
            first = false;
            break;
          }
      
          //// alternatively, if the timeout runs out, 
          //// break here
          //if start.elapsed() > wait_for_pp_timeout {
          //  break;
          //}
        },
        Err(_) => {break;}
      }
    trace!("TIMEOUT!");
    continue;
    }

    // this is expensive!
    events_backlog_prev = events_backlog_new.clone();
  
  }
}

