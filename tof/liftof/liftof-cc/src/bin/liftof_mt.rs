//! Standalone application to get events from the MTB (master trigger board)
//!
//! This will connect to the MTB over Udp and request the event queue, 
//! transforming it into MasterTriggerEvents. 
//!
//! If given the commandline option, it can automatically send requests
//! over the network to triggered readoutboards.
//!
//!

#[macro_use] extern crate log;
//extern crate env_logger;
extern crate crossbeam_channel;

extern crate tof_dataclasses;
extern crate liftof_cc;
extern crate liftof_lib;

use std::collections::HashMap;

//use std::sync::{
//    Arc,
//    Mutex
//};

use std::thread;
use std::path::PathBuf;

use tof_dataclasses::DsiLtbRBMapping;
//use tof_dataclasses::threading::ThreadControl;

//use tof_dataclasses::commands::RBCommand;
use liftof_lib::{
    get_ltb_dsi_j_ch_mapping,
    readoutboard_commander,
    init_env_logger
};

use tof_dataclasses::packets::TofPacket;
//use tof_dataclasses::threading::ThreadPool;
use tof_dataclasses::events::MasterTriggerEvent;
use liftof_lib::master_trigger;
use crossbeam_channel as cbc;
//use std::io::Write;

//use liftof_lib::color_log;
extern crate clap;
use clap::Parser;

#[derive(Parser, Default, Debug)]
#[command(author = "J.A.Stoessl", version, about, long_about = None)]
struct Args {
  /// More printout to std::out
  #[arg(long, default_value_t=false)]
  verbose : bool,
  /// Send RB request packets
  #[arg(long, default_value_t=false)]
  send_requests : bool,
  /// Apply trace suppression 
  #[arg(long, default_value_t=false)]
  trace_suppression : bool,
  /// A json file wit the ltb(dsi, j, ch) -> rb_id, rb_ch mapping.
  #[arg(long)]
  json_ltb_rb_map : Option<PathBuf>,
}


fn main() {

  init_env_logger();
  info!("Logging initialized!");
  let (mte_send, mte_rec): (cbc::Sender<MasterTriggerEvent>, cbc::Receiver<MasterTriggerEvent>) = cbc::unbounded(); 
  let (tp_send_moni, _tp_rec_moni): (cbc::Sender<TofPacket>, cbc::Receiver<TofPacket>) = cbc::unbounded(); 
  let (tp_send_req, tp_rec_req): (cbc::Sender<TofPacket>, cbc::Receiver<TofPacket>) = cbc::unbounded(); 
 
  // Create shared data wrapped in an Arc and a Mutex for synchronization
  //let thread_control = Arc::new(Mutex::new(ThreadControl::default()));

  let mut ltb_rb_map : DsiLtbRBMapping = HashMap::<u8,HashMap::<u8,HashMap::<u8,(u8,u8)>>>::new();
  let master_trigger_ip   = String::from("10.0.1.10");
  let master_trigger_port = 50001usize;
  //let worker_threads      = ThreadPool::new(2);

  let args = Args::parse();
  let verbose = args.verbose;
  //if args.trace_suppression {
  //  match set_trace_suppression(  
  //}

  if args.send_requests {
    match args.json_ltb_rb_map {
      None => {
        panic!("Will need json ltb -> rb mapping when we want to send requests!")
      },
      Some(_json_ltb_rb_map) => {
        ltb_rb_map = get_ltb_dsi_j_ch_mapping(_json_ltb_rb_map);
      }
    }
  }
  let args = Args::parse(); 
  let _worker_thread = thread::Builder::new()
         .name("master_trigger".into())
         .spawn(move || {
            master_trigger(&master_trigger_ip, 
                           master_trigger_port,
                           &ltb_rb_map,
                           &mte_send,
                           &tp_send_req,
                           &tp_send_moni,
                           10,
                           60,
                           true,
                           args.send_requests);
         })
         .expect("Failed to spawn master_trigger thread!");
  let _rbcmd_thread = thread::Builder::new()
         .name("rb_commander".into())
         .spawn(move || {
            readoutboard_commander(&tp_rec_req); 
         })
         .expect("Failed to spawn rb_commander thread!");

 let mut n_events = 0u64;
 loop {
   match mte_rec.recv() {
     Err(err)  => debug!("Can not receive events! Error {err}"),
     Ok(_ev)    => {
       //if ev.n_paddles > 0 {
       //  println!("Received event {}", ev);
       //}
       if n_events % 100 == 0 {
         if verbose {
           println!("{}", _ev);
         }
       } 
       n_events += 1;
     }
   }
   //match tp_rec_req.recv() {
   //  Err(err) => debug!("Can not receive requests! {err}"),
   //  Ok(req) => {
   //    let cmd = RBCommand::from(&req);
   //    println!("==> Received requeest {}", cmd);
   //    //match RBCommand::from(req) {
   //    //  Ok(cmd) => {
   //    //    println!("==> Received request {}", cmd);
   //    //  },
   //    //  Err(err) => {
   //    //    error!("Can't decode rb command!");
   //    //  }
   //    //}
   //  }
   //}
 }
}
