extern crate tof_dataclasses;
extern crate liftof_cc;
extern crate liftof_lib;
extern crate crossbeam_channel;
extern crate pretty_env_logger;

use tof_dataclasses::packets::TofPacket;
use tof_dataclasses::threading::ThreadPool;
use tof_dataclasses::events::MasterTriggerEvent; 
use liftof_lib::master_trigger;
use crossbeam_channel as cbc;

fn main() {
 pretty_env_logger::init();
 let (tp_to_sink, tp_from_client) : (cbc::Sender<TofPacket>, cbc::Receiver<TofPacket>) = cbc::unbounded();
 let (master_ev_send, master_ev_rec): (cbc::Sender<MasterTriggerEvent>, cbc::Receiver<MasterTriggerEvent>) = cbc::unbounded(); 
  
 let master_trigger_ip   = String::from("192.168.36.121");
 let master_trigger_port = 50001usize;
 let worker_threads = ThreadPool::new(2);

 worker_threads.execute(move || {
                        master_trigger(&master_trigger_ip, 
                                       master_trigger_port,
                                       &tp_to_sink,
                                       &master_ev_send);
 });

 loop {
   match master_ev_rec.recv() {
     Err(err)  => (),
     Ok(ev)    => {
       //if ev.n_paddles > 0 {
         println!("Received event {}", ev);
       //}
     }
   }
 }
}
