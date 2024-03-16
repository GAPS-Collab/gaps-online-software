//! Standalone application to get events from the MTB (master trigger board)
//!
//! This will connect to the MTB over Udp and request the event queue, 
//! transforming it into MasterTriggerEvents. 
//!
//! If given the commandline option, it can automatically send requests
//! over the network to triggered readoutboards.
//!
//!

extern crate zmq;

#[macro_use] extern crate log;
extern crate crossbeam_channel;

extern crate tof_dataclasses;
extern crate liftof_cc;
extern crate liftof_lib;

use std::thread;

//use tof_dataclasses::threading::ThreadControl;

//use tof_dataclasses::commands::RBCommand;
use liftof_lib::{
    init_env_logger,
    LiftofSettings
};

use tof_dataclasses::packets::TofPacket;
use tof_dataclasses::events::MasterTriggerEvent;
use tof_dataclasses::serialization::Serialization;
use liftof_lib::{
    master_trigger,
};


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
  /// Relay RB network traffic through 
  /// open poart
  #[arg(long, default_value_t=false)]
  relay_rbs : bool,
  /// Publish TofPackets at port 42000
  #[arg(long, default_value_t=false)]
  publish_packets : bool,
  /// Configuration of liftof-mt. 
  /// Use the same config 
  #[arg(short, long)]
  config: Option<String>,

}

fn rb_relay() {
  let ctx = zmq::Context::new();
  let socket = ctx.socket(zmq::SUB).expect("Unable to create 0MQ SUB socket!");
  let socket_out = ctx.socket(zmq::PUB).expect("Unable to create 0MQ PUB socket!");
  for rb_id in 1..51 {
    let address = format!("tcp://10.0.1.1{:02}:42000", rb_id);
    socket.connect(&address).expect("Unable to bind to data (PUB) socket {adress}");
    println!("==> 0MQ PUB socket bound to address {address}");
  }
  match socket.set_subscribe(b"") {
    Err(err) => error!("Unable to subscribe to any message! {err}"),
    Ok(_)    => ()
  }
  let address_out : &str = "tcp://100.96.207.91:42001";
  match socket_out.bind(address_out) {
    Err(err) => error!("Unable to bind to PUB socket at {}! {err}", address_out),
    Ok(_)    => ()
  }
  loop {
    match socket.recv_bytes(0) {
      Err(_err) => (),
      Ok(data)  => {
        match socket_out.send(data, 0) {
          Err(_err) => (),
          Ok(_)     => ()
        }
      }
    }
  }
}


fn main() {

  init_env_logger();
  info!("Logging initialized!");
  let (mte_send, mte_rec): (cbc::Sender<MasterTriggerEvent>, cbc::Receiver<MasterTriggerEvent>) = cbc::unbounded(); 
  let (tp_send_moni, tp_rec_moni): (cbc::Sender<TofPacket>, cbc::Receiver<TofPacket>) = cbc::unbounded(); 
 
  // Create shared data wrapped in an Arc and a Mutex for synchronization
  //let thread_control = Arc::new(Mutex::new(ThreadControl::default()));
  // deal with command line arguments
  let config          : LiftofSettings;
  let args    = Args::parse();
  match args.config {
    None => panic!("No config file provided! Please provide a config file with --config or -c flag!"),
    Some(cfg_file) => {
      match LiftofSettings::from_toml(cfg_file) {
        Err(err) => {
          error!("CRITICAL! Unable to parse .toml settings file! {}", err);
          panic!("Unable to parse config file!");
        }
        Ok(_cfg) => {
          config = _cfg;
        }
      }
    } // end Some
  } // end match
  let mtb_settings          = config.mtb_settings.clone();
  
  println!("=> Using the following config as parsed from the config file:\n{}", config);

  let mt_address          = String::from("10.0.1.10:50001");

  let args                = Args::parse();
  let verbose             = args.verbose;
  let publish_packets     = args.publish_packets;
  let relay_rbs           = args.relay_rbs;

  let _worker_thread = thread::Builder::new()
         .name("master_trigger".into())
         .spawn(move || {
            master_trigger(mt_address, 
                           &mte_send,
                           &tp_send_moni,
                           mtb_settings,
                           verbose);
         })
         .expect("Failed to spawn master_trigger thread!");
 if relay_rbs {
   let _relay_thread = thread::Builder::new()
                       .name("rb_relay".into())
                       .spawn(move || {
                         rb_relay(); 
                        })
                       .expect("Failed to spawn rb_relay thread!");
 
 }
 let mut n_events = 0u64;
 let ctx = zmq::Context::new();
 let address : &str = "tcp://100.96.207.91:42000";
 let data_socket = ctx.socket(zmq::PUB).expect("Unable to create 0MQ PUB socket!");
 if publish_packets {
   data_socket.bind(address).expect("Unable to bind to data (PUB) socket {adress}");
   println!("==> 0MQ PUB socket bound to address {address}");
 }
 loop {
   match mte_rec.recv() {
     Err(err)  => debug!("Can not receive events! Error {err}"),
     Ok(_ev)    => {
       //if ev.n_paddles > 0 {
       //  println!("Received event {}", ev);
       //}
       if publish_packets {
         let tp = TofPacket::from(&_ev);
         match data_socket.send(tp.to_bytestream(), 0) {
           Err(err) => error!("Can't send TofPacket! {err}"),
           Ok(_)    => ()
         }
       }
       if n_events % 1000 == 0 {
         if verbose {
           println!("{}", _ev);
         }
       } 
       n_events += 1;
     }
   }
   match tp_rec_moni.try_recv() {
     Err(err)  => debug!("Can not receive events! Error {err}"),
     Ok(_moni)    => {
       if publish_packets {
         //let tp = TofPacket::from(&_moni);
         match data_socket.send(_moni.to_bytestream(), 0) {
           Err(err) => error!("Can't send TofPacket! {err}"),
           Ok(_)    => ()
         }
       }
       if verbose {
         println!("{}", _moni);
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
