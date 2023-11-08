// read the master trigger DAQ stream (MasterTriggerEvents)
#[macro_use] extern crate log;
extern crate env_logger;
extern crate crossbeam_channel;

extern crate tof_dataclasses;
extern crate liftof_cc;
extern crate liftof_lib;

use std::collections::HashMap;


use tof_dataclasses::packets::TofPacket;
use tof_dataclasses::threading::ThreadPool;
use tof_dataclasses::DsiLtbRBMapping;
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
 

 let (tp_send, tp_rec): (cbc::Sender<TofPacket>, cbc::Receiver<TofPacket>) = cbc::unbounded(); 
 let (tp_send_req, _tp_rec_req): (cbc::Sender<TofPacket>, cbc::Receiver<TofPacket>) = cbc::unbounded(); 
  
 let ltb_rb_map : DsiLtbRBMapping = HashMap::<u8,HashMap::<u8,HashMap::<u8,(u8,u8)>>>::new();
 let master_trigger_ip   = String::from("10.0.1.10");
 let master_trigger_port = 50001usize;
 let worker_threads      = ThreadPool::new(2);

 worker_threads.execute(move || {
                        master_trigger(&master_trigger_ip, 
                                       master_trigger_port,
                                       &ltb_rb_map,
                                       &tp_send,
                                       &tp_send_req,
                                       10,
                                       60,
                                       true);
 });

 loop {
   match tp_rec.recv() {
     Err(err)  => trace!("Can not receive events! Error {err}"),
     Ok(_ev)    => {
       //if ev.n_paddles > 0 {
       //  println!("Received event {}", ev);
       //}
     }
   }
 }
}
