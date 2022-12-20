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
  let wait_for_pp_timeout = Duration::from_millis(10000);

  // the events might not be in order here
  // so we have to check the incoming event ids
  for (m_id, n_paddles) in master_id {
    
    trace!("Received master event id {}", m_id);
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

    let start = Instant::now();

    for pp in paddle_packets {
      if pp.event_id == m_id {
        event.paddle_packets.push(pp);    
      } else {
        events_backlog_new.push_back(pp);
      }
      if event.is_complete() {
        break;
      }
      
      // alternatively, if the timeout runs out, 
      // break here
      if start.elapsed() > wait_for_pp_timeout {
        break;
      }
    }

    // this is expensive!
    events_backlog_prev = events_backlog_new.clone();
  
  }
}

