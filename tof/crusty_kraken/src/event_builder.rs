///
///
///
///
///
///
///
///


use std::sync::mpsc::Receiver;

use crate::reduced_tofevent::PaddlePacket;

///
///
///
///
pub fn event_builder (master_id      : &Receiver<u32>,
                      paddle_packets : &Receiver<PaddlePacket>) {

  for pp in paddle_packets {
  }
  //for id in master_id {
  //  trace!("Received master event id {}", id);
  //    

  //}

}

