//! Liftof scheduler - this will run as an additional process
//! on the TOF main computer to schedule run/start stop
//! through the liftof-cc main program
//!
//! Features
//!
//! * receive TofCommmandV2 from telemetry packets (flight computer)
//! * modifiy liftof-config file, recreate links to config files
//! * start/stop liftof process
//! * scedule run queue
//!
//!

#[macro_use] extern crate log;

use std::fs::{
  OpenOptions,
};

use std::path::Path;
use std::thread;
use std::io::Write;

use chrono::Utc;

use clap::{
  arg,
  command,
  Parser
};
  
use liftof_lib::{
  init_env_logger,
  LIFTOF_LOGO_SHOW,
  LiftofSettings,
};

use std::time::{
  Instant,
  Duration,
};

use tof_dataclasses::commands::{
  TofCommandV2,
  TofCommandCode
};
use tof_dataclasses::serialization::Serialization;
use tof_dataclasses::packets::{
  PacketType,
  TofPacket
};

use liftof_cc::manage_liftof_cc_service;

#[derive(Parser, Debug)]
#[command(author = "J.A.Stoessl", version, about, long_about = None)]
#[command(propagate_version = true)]
struct LiftofSchedArgs {
  #[arg(short, long)]
  config      : Option<String>,
}

fn main() {
  init_env_logger();

  // welcome banner!
  println!("{}", LIFTOF_LOGO_SHOW);
  println!("-----------------------------------------------");
  println!(" >> Welcome to liftof-cc \u{1F680} \u{1F388} ");
  println!(" >> liftof is a software suite for the time-of-flight detector (TOF) ");
  println!(" >> for the GAPS experiment \u{1F496}");
  println!(" >> This is the Command&Control server");
  println!(" >> It connects to the MasterTriggerBoard and the ReadoutBoards");
  println!("-----------------------------------------------\n\n");
  
  let args            = LiftofSchedArgs::parse();
  let config          : LiftofSettings;
  let cfg_file_str    : String; 
  match args.config {
    None => panic!("No config file provided! Please provide a config file with --config or -c flag!"),
    Some(cfg_file) => {
      cfg_file_str = cfg_file.clone();
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

  let timer = Instant::now();
  
  let sleep_time  = Duration::from_secs(config.cmd_dispatcher_settings.cmd_listener_interval_sec);
  let locked      = config.cmd_dispatcher_settings.deny_all_requests; // do not allow the reception of commands if true
  
  let fc_sub_addr = config.cmd_dispatcher_settings.fc_sub_address.clone();
  let cc_pub_addr = config.cmd_dispatcher_settings.cc_server_address.clone();
  let ctx = zmq::Context::new();
  
  // socket to receive commands
  info!("Connecting to flight computer at {}", fc_sub_addr);
  let cmd_receiver = ctx.socket(zmq::SUB).expect("Unable to create 0MQ SUB socket!");
  cmd_receiver.set_subscribe(b"").expect("Unable to subscribe to empty topic!");
  cmd_receiver.connect(&fc_sub_addr).expect("Unable to subscribe to flight computer PUB");
  info!("ZMQ SUB Socket for flight cpu listener bound to {fc_sub_addr}");

  // socket to send commands on the RB network
  info!("Binding socket for command dispatching to rb network to {}", cc_pub_addr);
  let cmd_sender = ctx.socket(zmq::PUB).expect("Unable to create 0MQ PUB socket!");
  cmd_sender.bind(&cc_pub_addr).expect("Unable to bind to (PUB) socket!");

  // open the logfile for commands
  let mut filename = config.cmd_dispatcher_settings.cmd_log_path.clone();
  if !filename.ends_with("/") {
    filename += "/";
  }
  filename        += "received-commands.log";
  let path         = Path::new(&filename);
  info!("Writing cmd log to file {filename}");
  let mut log_file = OpenOptions::new().create(true).append(true).open("received-commands.log").expect("Unable to create file!");
  match OpenOptions::new().create(true).append(true).open(path) {
    Ok(_f) => {log_file = _f;},
    Err(err) => { 
      error!("Unable to write to path {filename}! {err} Falling back to default file path");
    }
  }
  loop {
    thread::sleep(sleep_time);
    //println!("=> Cmd responder loop iteration!");
    match cmd_receiver.connect(&fc_sub_addr) {
      Ok(_)    => (),
      Err(err) => {
        error!("Unable to connect to {}! {}", fc_sub_addr, err);
      }
    }
    
    let mut cmd_packet = TofPacket::new();
    match cmd_receiver.recv_bytes(zmq::DONTWAIT) {
      Err(err)   => {
        error!("ZMQ socket receiving error! {err}");
        continue;
      }
      Ok(buffer) => {
        info!("Received command {:?}", buffer);
        // identfiy if we have a GAPS packet
        if buffer[0] == 0xeb && buffer[1] == 0x90 && buffer[4] == 0x5a {
          // We have a GAPS packet -> FIXME:
          error!("GAPS packet command receiving not supported yet! Currently, we can only process TofPackets!");
          // strip away the GAPS header!  
          continue;
        } 
        match TofPacket::from_bytestream(&buffer, &mut 8) {
          Err(err) => {
            error!("Unable to decode bytestream for command ! {:?}", err);
            continue;  
          },
          Ok(packet) => {
            cmd_packet = packet;
          }
        }
        //let mut resp = TofResponse::Unknown;
        
        debug!("Got packet {}!", cmd_packet);
        match cmd_packet.packet_type {
          PacketType::TofCommandV2 => {
            let mut cmd = TofCommandV2::new();
            match cmd_packet.unpack::<TofCommandV2>() {
              Ok(_cmd) => {cmd = _cmd;},
              Err(err) => {
                error!("Unable to decode TofCommand! {err}");
                continue;
              }
            }
            println!("= => [cmd_dispatcher] Received command {}!", cmd);
            let now = Utc::now().to_string();
            let write_to_file = format!("{:?}: {}\n",now, cmd);
            match log_file.write_all(&write_to_file.into_bytes()) {
              Err(err) => {
                error!("Writing to file to path {} failed! {}", filename, err)
              }
              Ok(_)    => ()
            }
            match log_file.sync_all() {
              Err(err) => {
                error!("Unable to sync file to disc! {err}");
              },
              Ok(_) => ()
            }
          
            // actual command tree
            match cmd.command_code {
              TofCommandCode::DataRunStop  => {
                println!("= => Received DataRunStop!");
                manage_liftof_cc_service(String::from("stop"));
              },
              TofCommandCode::DataRunStart  => {
                println!("= => Received DataRunStart!");
                manage_liftof_cc_service(String::from("start"));
              },
              _ => {
                error!("Dealing with command code {} has not been implemented yet!", cmd.command_code);
              }
            }
          },
          _ => {
            error!("Unable to deal with packet type {}!", cmd_packet.packet_type)
          }
        }
      }
    }
  }
}
