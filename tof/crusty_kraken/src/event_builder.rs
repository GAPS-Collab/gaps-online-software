///
///
///
///
///
///
///
///


use std::sync::mpsc::Receiver;

///
///
///
///
pub fn event_builder (master_id : &Receiver<u32>) {
  for id in master_id {
    trace!("Received master event id {}", id);
  }

}

