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
//! Run flow/staging:
//!
//! There are 3 directories in the staging directory:
//! - current - the configuration which is run currently
//! - next    - the configuration which shall be run next. 
//!             This configuration can be edited until the 
//!             next run start.
//! - queue   - config files in here will get assesed and 
//!             sorted every new run cycle and the one with 
//!             the highest priority (number) will get 
//!             executed first.
//!
//!

#[macro_use] extern crate log;

use std::fs;
use std::fs::{
  OpenOptions,
  rename,
  //create_dir,
};

use std::thread;
use std::io::Write;

use std::path::{
  PathBuf,
  Path
};

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

use telemetry_dataclasses::packets::AckBfsw;

use liftof_cc::manage_liftof_cc_service;

#[derive(Parser, Debug)]
#[command(author = "J.A.Stoessl", version, about, long_about = None)]
#[command(propagate_version = true)]
struct LiftofSchedArgs {
  #[arg(short, long)]
  config      : Option<String>,
}

/// Get the files in the queue and sort them by number
fn get_queue(dir_path : &String) -> Vec<String> {
  let mut entries = fs::read_dir(dir_path)
    .expect("Directory might not exist!")
    .map(|entry| entry.unwrap().path())
    .collect::<Vec<PathBuf>>();
  entries.sort_by(|a, b| {
    let meta_a = fs::metadata(a).unwrap();
    let meta_b = fs::metadata(b).unwrap();
    meta_a.modified().unwrap().cmp(&meta_b.modified().unwrap())
  });
  entries.iter()
    .map(|path| path.to_str().unwrap().to_string())
    .collect()
}

fn move_file_with_name(old_path: &str, new_dir: &str) -> Result<(), std::io::Error> {
  let old_path  = Path::new(old_path);
  let file_name = old_path.file_name().unwrap().to_str().unwrap(); // Extract filename
  let new_path  = Path::new(new_dir).join(file_name); // Combine new directory with filename
  fs::rename(old_path, new_path) // Move the file
}

fn move_file_rename_liftof(old_path: &str, new_dir: &str) -> Result<(), std::io::Error> {
  let old_path  = Path::new(old_path);
  let file_name = old_path.file_name().unwrap().to_str().unwrap(); // Extract filename
  let new_path  = Path::new(new_dir).join("liftof-config.toml"); // Combine new directory with filename
  fs::rename(old_path, new_path) // Move the file
}

fn copy_file(old_path: &str, new_dir: &str) -> Result<u64, std::io::Error> {
  let old_path  = Path::new(old_path);
  let file_name = old_path.file_name().unwrap().to_str().unwrap(); // Extract filename
  let new_path  = Path::new(new_dir).join(file_name); // Combine new directory with filename
  fs::copy(old_path, new_path) 
}

fn copy_file_rename_liftof(old_path: &str, new_dir: &str) -> Result<u64, std::io::Error> {
  let old_path  = Path::new(old_path);
  let file_name = old_path.file_name().unwrap().to_str().unwrap(); // Extract filename
  let new_path  = Path::new(new_dir).join("liftof-config.toml"); // Combine new directory with filename
  fs::copy(old_path, new_path) 
}

fn delete_file(file_path: &str) -> Result<(), std::io::Error> {
  let path = Path::new(file_path);
  fs::remove_file(path) // Attempts to delete the file at the given path
}

/// Copy a config file from the queue to the current and 
/// next directories and restart liftof-cc. 
///
/// As soon as the run is started, prepare the next run
fn run_cycler() {
}


fn main() {
  init_env_logger();

  // welcome banner!
  println!("{}", LIFTOF_LOGO_SHOW);
  println!("-----------------------------------------------");
  println!(" >> Welcome to liftof-scheduler \u{1F680} \u{1F388} ");
  println!(" >> liftof is a software suite for the time-of-flight detector (TOF) ");
  println!(" >> for the GAPS experiment \u{1F496}");
  println!(" >> This is the run scheduler (liftof-scheduler)");
  println!(" >> It starts/stops the liftof-cc service and manages run configurations");
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

  //let subdirs = vec!["current", "next", "queue"]
  let staging_dir = config.staging_dir; 
  //if let Ok(metadata) = fs::metadata(&staging_dir) {
  //  if metadata.is_dir() {
  //  } 
  //} else {
  //  match fs::create_dir(&stream_files_path) {
  //    Ok(())   => println!("=> Created {} to save stream data", stream_files_path.display()),
  //    Err(err) => panic!("Failed to create directory: {}! {}", stream_files_path.display(), err),
  //  }
  //}

  let queue_dir   = format!("{}/queue", staging_dir);
  let next_dir    = format!("{}/next",  staging_dir);
  let current_dir = format!("{}/current", staging_dir);

  let sleep_time  = Duration::from_secs(config.cmd_dispatcher_settings.cmd_listener_interval_sec);
  //let locked      = config.cmd_dispatcher_settings.deny_all_requests; // do not allow the reception of commands if true
  
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
        trace!("ZMQ socket receiving error! {err}");
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
        match TofPacket::from_bytestream(&buffer, &mut 0) {
          Err(err) => {
            error!("Unable to decode bytestream for command ! {:?}", err);
            continue;  
          },
          Ok(packet) => {
            cmd_packet = packet;
          }
        }
        let mut ack = AckBfsw::new(); 
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
                let queue   = get_queue(&queue_dir);
                let current = get_queue(&current_dir);
                let next    = get_queue(&next_dir); 
                
                if current.len() == 0 {
                  // we are f***ed
                  error!("We don't have a current configuration. This is BAD!");
                  continue;
                }

                println!("= => Found {} files in run queue!", queue.len());
                if next.len() == 0 && queue.len() == 0 {
                  println!("= => Nothing staged, will jusr repeat current run setting!");
                  //manage_liftof_cc_service(String::from("restart"));
                  continue;
                }
                if next.len() == 0 && queue.len() != 0 {
                  error!("Empty next directory, but we have files in the queue!");
                  match copy_file_rename_liftof(&queue[0], &next_dir) {
                    Ok(_) => (),
                    Err(err) => {
                      error!("Unable to copy {} to {}! {}", next[0], next_dir, err);
                    }
                  }
                  match move_file_rename_liftof(&queue[0], &current_dir) {
                    Ok(_) => (),
                    Err(err) => {
                      error!("Unable to copy {} to {}! {}", queue[0], current_dir, err);
                    }
                  }
                }
                if next.len() != 0  {
                  match delete_file(&current[0]) {
                    Ok(_)    => (),
                    Err(err) => {
                      error!("Unable to delete {}! {}", current[0], err);
                    }
                  }
                  match move_file_rename_liftof(&next[0], &current_dir) {
                    Ok(_) => (),
                    Err(err) => {
                      error!("Unable to copy {} to {}! {}", next[0], current_dir, err);
                    }
                  }
                  if queue.len() != 0 {
                    match move_file_with_name(&queue[0], &next_dir) {
                      Ok(_) => (),
                      Err(err) => {
                        error!("Unable to move {} to {}! {}", queue[0], next_dir, err);
                      }
                    }
                  }
                  println!("=> Restarting liftof-cc!");
                  manage_liftof_cc_service(String::from("restart"));
                }
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
