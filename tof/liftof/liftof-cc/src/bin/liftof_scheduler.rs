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
};

use std::thread;
use std::io::Write;
use std::process::Command;
use std::path::{
  Path
};

use chrono::Utc;
use toml::Table;
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
use tof_dataclasses::database::{
  connect_to_db,
  ReadoutBoard,
};

use telemetry_dataclasses::packets::AckBfsw;

use liftof_cc::{
  manage_liftof_cc_service,
  ssh_command_rbs,
  run_cycler,
};

#[derive(Parser, Debug)]
#[command(author = "J.A.Stoessl", version, about, long_about = None)]
#[command(propagate_version = true)]
struct LiftofSchedArgs {
  #[arg(short, long)]
  config      : Option<String>,
  #[arg(long, default_value_t = false)]
  dry_run : bool,
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
  //let cfg_file_str    : String; 
  let dry_run         = args.dry_run;
  match args.config {
    None => panic!("No config file provided! Please provide a config file with --config or -c flag!"),
    Some(cfg_file) => {
      //cfg_file_str = cfg_file.clone();
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

  let staging_dir           = config.staging_dir; 
  let db_path               = config.db_path.clone();
  let mut conn              = connect_to_db(db_path).expect("Unable to establish a connection to the DB! CHeck db_path in the liftof settings (.toml) file!");
  // if this call does not go through, we might as well fail early.
  let mut rb_list           = ReadoutBoard::all(&mut conn).expect("Unable to retrieve RB information! Unable to continue, check db_path in the liftof settings (.toml) file and DB integrity!");
  let rb_ignorelist         = config.rb_ignorelist_always.clone();
  let rb_ignorelist_tmp     = config.rb_ignorelist_run.clone();
  for k in 0..rb_ignorelist.len() {
    let bad_rb = rb_ignorelist[k];
    rb_list.retain(|x| x.rb_id != bad_rb);
  }

  for k in 0..rb_ignorelist_tmp.len() {
    let bad_rb = rb_ignorelist_tmp[k];
    rb_list.retain(|x| x.rb_id != bad_rb);
  }

  let nboards = rb_list.len();
  println!("=> Will use {} readoutboards! Ignoring {:?} sicne they are mareked as 'ignore' in the config file!", rb_list.len(), rb_ignorelist );


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
  if !dry_run {
    cmd_sender.bind(&cc_pub_addr).expect("Unable to bind to (PUB) socket!");
  }
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
        info!("Received bytes {:?}", buffer);
        // identfiy if we have a GAPS packet
        if buffer.len() < 2 {
          error!("The received bytestring does not even have 2 bnytes for a header!");
          continue
        }
        if buffer[0] == 0xeb && buffer[1] == 0x90 && buffer[4] == 0x46 { //0x5a?
          // We have a GAPS packet -> FIXME:
          info!("Received command sent through BFSW system!");
        } 
        if buffer.len() < 8 {
          error!("Received command is too short! (Smaller than 8 bytes) {:?}", buffer);
          continue;
        }
        match TofPacket::from_bytestream(&buffer, &mut 8) {
          Err(err) => {
            error!("Unable to decode bytestream {:?} for command ! {:?}", buffer, err);
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
            println!("= => Received command {}!", cmd);
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
                if !dry_run { 
                  manage_liftof_cc_service(String::from("stop"));
                }
              },
              TofCommandCode::DataRunStart  => {
                info!("Received DataRunStart!");
                // FIXME - factor out manage_liftof_cc_service here, otherwise
                // it gets really confusing
                match run_cycler(staging_dir.clone(), dry_run) {
                  Err(err) => error!("= => Run cycler had an issue! {err}"),
                  Ok(_)    => ()
                }
              }
              TofCommandCode::ShutdownRB => {
                let cmd_rb_list  = cmd.payload.clone();
                info!("Received Shutdown RB command for RBs {:?}", cmd_rb_list);
                let cmd_args     = vec![String::from("sudo"),
                                        String::from("shutdown"),
                                        String::from("now")]; 
                match ssh_command_rbs(&cmd_rb_list, cmd_args) {
                  Err(err) => error!("SSh-ing into RBs {:?} failed! {err}", cmd_rb_list),
                  Ok(_)    => ()
                }
              }
              TofCommandCode::ShutdownRAT => {
                let cmd_rb_list  = cmd.payload.clone();
                info!("Received Shutdown RAT command for RBs {:?}", cmd_rb_list);
                let cmd_args     = vec![String::from("sudo"),
                                        String::from("shutdown"),
                                        String::from("now")]; 
                ssh_command_rbs(&cmd_rb_list, cmd_args);
              }
              TofCommandCode::ShutdownCPU => {
                let cmd_args     = vec![String::from("shutdown"),
                                        String::from("now")]; 
                info!("Received Shutdown command for CPU");
                if !args.dry_run {
                  match Command::new("sudo")
                    //.args([&rb_address, "sudo", "systemctl", "restart", "liftof"])
                    .args(cmd_args)
                    .spawn() {
                    Err(err) => {
                      error!("Unable to spawn shutdown process on TofCPU!");
                    }
                    // FIXME - timeout with try wait
                    Ok(mut child) => {
                      match child.wait() {
                        Err(err) => error!("Waiting for the shutdown process failed! {err}", err),
                        Ok(_)    => ()
                      }
                    }
                  }
                }
              }
              TofCommandCode::ChangeNextRunConfig => {
                let cfg_file = format!("{}/next/lfitof-config.toml", staging_dir.clone());
                // first check if the command is valid
                match cmd.extract_changerunconfig() {
                  None => error!("Unable to understand this command which is supposed to change the next run configuration!"),
                  Some(keys_val) => {
                    match fs::read_to_string(cfg_file.clone()) {
                      Err(err) => error!("Unable to read {}! {err}", cfg_file),
                      Ok(content) => {
                        let toml_table = content.parse::<Table>().unwrap();
                        let value = keys_val.back();
                        for k in 0..keys_val.len() - 1 {
                           let key = key_val[k];
                        }
                      }
                    }
                    //match LiftofSettings::from_toml(cfg_file) {
                    //  Err(err) => {
                    //    error!("CRITICAL! Unable to parse .toml settings file! {}", err);
                    //    //panic!("Unable to parse config file!");
                    //    continue;
                    //  }
                    //  Ok(mut config) => {

                    //    }
                    //  }
                    //}   
                  }
                }
              }
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
