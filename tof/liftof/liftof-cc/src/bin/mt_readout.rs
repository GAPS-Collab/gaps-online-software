// read the master trigger DAQ stream (MasterTriggerEvents)
#[macro_use] extern crate log;
extern crate env_logger;
extern crate crossbeam_channel;

extern crate tof_dataclasses;
extern crate liftof_cc;
extern crate liftof_lib;

//use tof_dataclasses::packets::TofPacket;
use tof_dataclasses::threading::ThreadPool;
use tof_dataclasses::events::MasterTriggerEvent; 
use liftof_lib::master_trigger;
use crossbeam_channel as cbc;
use std::io::Write;

use liftof_lib::color_log;

fn main() {

  env_logger::builder()
    .format(|buf, record| {
    writeln!( buf, "[{level}][{module_path}:{line}] {args}",
      level = color_log(&record.level()),
      module_path = record.module_path().unwrap_or("<unknown>"),
      line = record.line().unwrap_or(0),
      args = record.args()
      )
    }).init();
 

 let (master_ev_send, master_ev_rec): (cbc::Sender<MasterTriggerEvent>, cbc::Receiver<MasterTriggerEvent>) = cbc::unbounded(); 
  
 let master_trigger_ip   = String::from("192.168.36.121");
 let master_trigger_port = 50001usize;
 let worker_threads = ThreadPool::new(2);
 let (rate_to_main, rate_from_mt) : (cbc::Sender<u32>, cbc::Receiver<u32>) = cbc::unbounded();

 worker_threads.execute(move || {
                        master_trigger(&master_trigger_ip, 
                                       master_trigger_port,
                                       &rate_to_main,
                                       &master_ev_send,
                                       true);
 });

 loop {
   match master_ev_rec.recv() {
     Err(err)  => error!("Can not receive events! Error {err}"),
     Ok(ev)    => {
       //if ev.n_paddles > 0 {
         println!("Received event {}", ev);
       //}
     }
   }
 }
}
