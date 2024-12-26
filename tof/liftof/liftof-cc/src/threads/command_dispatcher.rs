//! Command dispatcher - handle incoming requests
//!
//! Most requests will be satisfied by liftof-scheduler

use std::thread;
use std::sync::{
  Arc,
  Mutex,
};

use std::time::{
  Duration,
};

use crossbeam_channel::{
  Sender
};

use tof_dataclasses::packets::{
  TofPacket
};
use tof_dataclasses::serialization::{
  Serialization,
};

use liftof_lib::thread_control::ThreadControl;

use crate::LIFTOF_HOTWIRE;

/// Deal with incoming requests. Most will be statisfied
/// by the external liftof-scheduler. It will communicate
/// with the liftof-scheduler over a dedicated, 
/// non-configurable port
///
/// # Arguments:
///
///   * thread_control : inter-thread communications,
///                      start/stop signals.
///                      Keeps global settings.
///   * tp_to_sink     : send packets to global data sink 
pub fn command_dispatcher(thread_ctrl  : Arc<Mutex<ThreadControl>>,
                          tp_to_sink   : &Sender<TofPacket>) { 
  
  
  // socket to receive commands
  // NEW: since we have the liftof-scheduler now, this basically just 
  // listens in on the cc_pub_addr so that it can send ack packets.
  // The actual commanding is done by the liftof-scheduler
  let ctx = zmq::Context::new();
  let cmd_receiver = ctx.socket(zmq::SUB).expect("Unable to create 0MQ SUB socket!");
  cmd_receiver.set_subscribe(b"").expect("Unable to subscribe to empty topic!");
  // the "HOTWIRE" is the direct connection to the PUB socekt of the 
  // liftof-scheduler
  cmd_receiver.connect(LIFTOF_HOTWIRE).expect("Unable to subscribe to flight computer PUB");
  //info!("ZMQ SUB Socket for flight cpu listener bound to {fc_sub_addr}");
  info!("Listening on {LIFTOF_HOTWIRE}!");

  // ok to block here since we haven't started yet
  let mut sleep_time = Duration::from_secs(1);
  match thread_ctrl.lock() {
    Ok(mut tc) => {
        tc.thread_cmd_dispatch_active = true;
        sleep_time   = Duration::from_secs(tc.liftof_settings.cmd_dispatcher_settings.cmd_listener_interval_sec);
    }
    Err(err) => {
        trace!("Can't acquire lock! {err}");
    }
  }

  loop {
    // the frequency of incoming request should be
    // small, so we take the heat out and nap a bit
    thread::sleep(sleep_time);
    match cmd_receiver.connect(LIFTOF_HOTWIRE) {
      Ok(_)    => (),
      Err(err) => {
        error!("Unable to connect to {}! {}", LIFTOF_HOTWIRE, err);
      }
    }
    match cmd_receiver.recv_bytes(zmq::DONTWAIT) {
      Err(err)   => {
        trace!("ZMQ socket receiving error! {err}");
        continue;
      }
      Ok(buffer) => {
        match TofPacket::from_bytestream(&buffer, &mut 0) {
          Err(err) => {
            error!("Unable to decode bytestream for command ! {:?}", err);
            continue;  
          }
          Ok(packet) => {
            debug!("Got packet {}!", packet);
            match tp_to_sink.send(packet) {
              Err(err) => {
                error!("Unable to send ACK packet! {err}");
              }
              Ok(_)    => ()
            }
          }
        }
      }
    }
  }
}
