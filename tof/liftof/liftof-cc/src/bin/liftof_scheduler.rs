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
  Duration,
};

use tof_dataclasses::commands::{
  TofCommandV2,
  TofCommandCode,
  TofReturnCode
};
use tof_dataclasses::serialization::{
  Serialization,
  Packable
};
use tof_dataclasses::packets::{
  PacketType,
  TofPacket
};
use tof_dataclasses::database::{
  connect_to_db,
  ReadoutBoard,
};
use tof_dataclasses::commands::config::{
  TriggerConfig,
  TOFEventBuilderConfig,
  DataPublisherConfig,
  TofRunConfig,
  TofRBConfig
};

use telemetry_dataclasses::packets::AckBfsw;

use liftof_cc::{
  manage_liftof_cc_service,
  ssh_command_rbs,
  copy_file_rename_liftof,
  LIFTOF_HOTWIRE,
};



#[derive(Parser, Debug)]
#[command(author = "J.A.Stoessl", version, about, long_about = None)]
#[command(propagate_version = true)]
struct LiftofSchedArgs {
  #[arg(short, long)]
  config      : Option<String>,
  /// Don't do anything, just tell us what 
  /// would happen
  #[arg(long, default_value_t = false)]
  dry_run : bool,
  /// Don't send ACK packets
  #[arg(long, default_value_t = false)]
  no_ack  : bool,
}

/// Send an ack packet to liftof-cc
///
/// Matroshka! Literally Ack in Pack, recursive packaging
/// - I love this!
///
/// The purpose of this is to sneak an Bfsw ack packet 
/// through the bfsw system. Well, that's how broken 
/// we all are
fn send_ack_packet(cc       : TofCommandCode,
                   ret_code : TofReturnCode,
                   socket   : &zmq::Socket) {
  let mut ack = AckBfsw::new(); 
  ack.ret_code1 = ret_code as u8;
  ack.ret_code2 = cc as u8;
  let tp = ack.pack();
  match socket.send(tp.to_bytestream(), 0) {
    Ok(_)    => (),
    Err(err) => error!("Unable to send ACK! {err}")
  }
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
  let dry_run         = args.dry_run;
  let no_ack          = args.no_ack;
  match args.config {
    None => panic!("No config file provided! Please provide a config file with --config or -c flag!"),
    Some(cfg_file) => {
      //cfg_file_str = cfg_file.clone();
      match LiftofSettings::from_toml(&cfg_file) {
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

  let staging_dir = config.staging_dir; 
  // This is the file we will edit 
  let cfg_file         = format!("{}/next/liftof-config.toml", staging_dir.clone());
  let next_dir         = format!("{}/next", staging_dir.clone());
  let current_dir      = format!("{}/current", staging_dir.clone());
  let default_cfg_file = format!("{}/default/liftof-config-default.toml", staging_dir.clone());
  let db_path     = config.db_path.clone();
  let mut conn    = connect_to_db(db_path).expect("Unable to establish a connection to the DB! CHeck db_path in the liftof settings (.toml) file!");
  // if this call does not go through, we might as well fail early.
  let rb_list     = ReadoutBoard::all(&mut conn).expect("Unable to retrieve RB information! Unable to continue, check db_path in the liftof settings (.toml) file and DB integrity!");
  let mut all_rb_ids  = Vec::<u8>::new();
  for rb in rb_list {
    all_rb_ids.push(rb.rb_id as u8);
  }

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
  if !dry_run || no_ack {
    cmd_sender.bind(LIFTOF_HOTWIRE).expect("Unable to bind to (PUB) socket!");
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

  // All RBs

  loop {
    thread::sleep(sleep_time);
    //println!("=> Cmd responder loop iteration!");
    let mut success = TofReturnCode::Unknown;
    match cmd_receiver.connect(&fc_sub_addr) {
      Ok(_)    => (),
      Err(err) => {
        error!("Unable to connect to {}! {}", fc_sub_addr, err);
        continue;
      }
    }
    
    let cmd_packet : TofPacket;
    match cmd_receiver.recv_bytes(zmq::DONTWAIT) {
      Err(err)   => {
        trace!("ZMQ socket receiving error! {err}");
        continue;
      }
      Ok(buffer) => {
        info!("Received bytes {:?}", buffer);
        // identfiy if we have a GAPS packet
        if buffer.len() < 4 {
          error!("Can't deal with commands shorter than 4 bytes@");
          continue
        }
        // check on the buffer
        if buffer[0] == 0x90 && buffer[1] == 0xeb {
          if buffer[4] != 0x46 { //0x5a?
            // We have a GAPS packet -> FIXME:
            info!("We received something, but it does not seem to be address to us! We are only listening to address {} right now!", 0x46);
            continue;
          } else {
            info!("Received command sent through (Cra-)BFSW system!");
            if buffer.len() < 8 {
              error!("Received command is too short! (Smaller than 8 bytes) {:?}", buffer);
              success = TofReturnCode::GarbledCommand;
              send_ack_packet(TofCommandCode::Unknown, success, &cmd_sender);
              continue;
            }
            match TofPacket::from_bytestream(&buffer, &mut 8) {
              Err(err) => {
                error!("Unable to decode bytestream {:?} for command ! {:?}", buffer, err);
                success = TofReturnCode::GarbledCommand;
                send_ack_packet(TofCommandCode::Unknown, success, &cmd_sender);
                continue;  
              },
              Ok(packet) => {
                cmd_packet = packet;
              }
            }
          }
        } else if  buffer[0] == 170 && buffer[1] == 170 {
          info!("Got a TofPacket!");
          match TofPacket::from_bytestream(&buffer, &mut 0) {
            Err(err) => {
              error!("Unable to decode bytestream {:?} for command ! {:?}", buffer, err);
              success = TofReturnCode::GarbledCommand;
              send_ack_packet(TofCommandCode::Unknown, success, &cmd_sender);
              continue;  
            },
            Ok(packet) => {
              cmd_packet = packet;
            }
          }
        } else {
          error!("Received bytestream, but don't know how to deal with it!");
          continue;
        }
        debug!("Got packet {}!", cmd_packet);
        match cmd_packet.packet_type {
          PacketType::TofCommandV2 => {
            let cmd : TofCommandV2;
            match cmd_packet.unpack::<TofCommandV2>() {
              Ok(_cmd) => {cmd = _cmd;},
              Err(err) => {
                error!("Unable to decode TofCommand! {err}");
                success = TofReturnCode::GarbledCommand;
                send_ack_packet(TofCommandCode::Unknown, success, &cmd_sender);
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
                  success =  manage_liftof_cc_service("stop"); 
                }
              },
              TofCommandCode::DataRunStart  => {
                info!("Received DataRunStart!");
                // FIXME - factor out manage_liftof_cc_service here, otherwise
                if !dry_run { 
                  success =  manage_liftof_cc_service("restart");
                }
              }
              TofCommandCode::ShutdownRB => {
                let mut cmd_rb_list  = cmd.payload.clone();
                if cmd_rb_list.is_empty() {
                  cmd_rb_list = all_rb_ids.clone();
                }
                info!("Received Shutdown RB command for RBs {:?}", cmd_rb_list);
                let cmd_args     = vec![String::from("sudo"),
                                        String::from("shutdown"),
                                        String::from("now")]; 
                if !args.dry_run {
                  match ssh_command_rbs(&cmd_rb_list, cmd_args) {
                    Err(err) => {
                      error!("SSh-ing into RBs {:?} failed! {err}", cmd_rb_list);
                      success = TofReturnCode::GeneralFail;
                    }
                    Ok(_)    => {
                      success = TofReturnCode::Success;
                    }
                  }
                }
              }
              TofCommandCode::ResetConfigWDefault => {
                info!("Will reset {} with {}", cfg_file, default_cfg_file);
                match copy_file_rename_liftof(&default_cfg_file, &next_dir) {
                  Ok(_)    => {
                    info!("Copy successful!");
                    success = TofReturnCode::Success;
                  }
                  Err(err) => {
                    error!("Unable to copy! {err}");
                    success = TofReturnCode::GeneralFail;
                  }
                }
              }
              TofCommandCode::SubmitConfig => {
                info!("Submitting the worked on config!");
                match copy_file_rename_liftof(&cfg_file, &current_dir) {
                  Ok(_)    => {
                    info!("Copy successful!");
                    success = TofReturnCode::Success;
                  }
                  Err(err) => { 
                    error!("Unable to copy! {err}");
                    success = TofReturnCode::GeneralFail;
                  }
                }
              }
              TofCommandCode::ShutdownRAT => {
                let cmd_rb_list  = cmd.payload.clone();
                info!("Received Shutdown RAT command for RBs {:?}", cmd_rb_list);
                let cmd_args     = vec![String::from("sudo"),
                                        String::from("shutdown"),
                                        String::from("now")]; 
                if !args.dry_run {
                  match ssh_command_rbs(&cmd_rb_list, cmd_args) {
                    Err(err) => {
                      error!("SSh-ing into RBs {:?} failed! {err}", cmd_rb_list);
                      success = TofReturnCode::GeneralFail;
                    }
                    Ok(_)    => {
                      success = TofReturnCode::Success;
                    }
                  }
                }
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
                      error!("Unable to spawn shutdown process on TofCPU! {err}");
                      success = TofReturnCode::GeneralFail;
                    }
                    // FIXME - timeout with try wait
                    Ok(mut child) => {
                      match child.wait() {
                        Err(err) => error!("Waiting for the shutdown process failed! {err}"),
                        Ok(_)    => ()
                      }
                    }
                  }
                }
              }
              TofCommandCode::RBCalibration => {
                info!("Received RBCalibration command!");
                if cmd.payload.len() < 3 {
                  error!("Broken RBCalibration command!");
                  continue;
                }
                let pre_run_cali   = cmd.payload[0] != 0;
                let send_packets   = cmd.payload[1] != 0;  
                let save_waveforms = cmd.payload[2] != 0;
                match LiftofSettings::from_toml(&cfg_file) {
                  Err(err) => {
                    error!("CRITICAL! Unable to parse .toml settings file! {}", err);
                    //panic!("Unable to parse config file!");
                    success = TofReturnCode::GeneralFail;
                  }
                  Ok(mut config) => {
                    config.data_publisher_settings.send_cali_packets = send_packets;
                    config.save_cali_wf                              = save_waveforms;
                    config.pre_run_calibration = pre_run_cali;
                    config.to_toml(String::from(cfg_file.clone()));
                    info!("We changed the data publisher settings to be this {}",config.data_publisher_settings);
                    
                    success = TofReturnCode::Success;
                  }
                }   
              }
              TofCommandCode::SetMTConfig => {
                info!("Will change trigger config for next run!");
                match TriggerConfig::from_bytestream(&cmd.payload, &mut 0) {
                  Err(err) => error!("Unable to extract TriggerConfig from command! {err}"),
                  Ok(tcf)  => {
                    match LiftofSettings::from_toml(&cfg_file) {
                      Err(err) => {
                        error!("CRITICAL! Unable to parse .toml settings file! {}", err);
                        success = TofReturnCode::GeneralFail;
                      }
                      Ok(mut config) => {
                        println!("=> We received the following trigger config {}", tcf);
                        config.mtb_settings.from_triggerconfig(&tcf);
                        println!("=> We changed the mtb settings to be this {}",config.mtb_settings);
                        config.to_toml(String::from(cfg_file.clone()));
                        success = TofReturnCode::Success;
                      }
                    }   
                  }
                }
              }
              TofCommandCode::SetTOFEventBuilderConfig => {
                info!("Will change tof event builder config for next run!");
                match TOFEventBuilderConfig::from_bytestream(&cmd.payload, &mut 0) {
                  Err(err) => error!("Unable to extract TofEventBuilderConfig from command! {err}"),
                  Ok(tcf)  => {
                    info!("Received config {}",tcf);
                    match LiftofSettings::from_toml(&cfg_file) {
                      Err(err) => {
                        error!("CRITICAL! Unable to parse .toml settings file! {}", err);
                        success = TofReturnCode::GeneralFail;
                      }
                      Ok(mut config) => {
                        config.event_builder_settings.from_tofeventbuilderconfig(&tcf);
                        info!("We changed the event builder settings to be this {}",config.event_builder_settings);
                        config.to_toml(String::from(cfg_file.clone()));
                        success = TofReturnCode::Success;
                      }
                    }   
                  }
                }
              }
              TofCommandCode::SetTofRunConfig => {
                info!("Will change tof run config for next run!");
                match TofRunConfig::from_bytestream(&cmd.payload, &mut 0) {
                  Err(err) => error!("Unable to extract TofEventBuilderConfig from command! {err}"),
                  Ok(tcf)  => {
                    info!("Received config {}",tcf);
                    match LiftofSettings::from_toml(&cfg_file) {
                      Err(err) => {
                        error!("CRITICAL! Unable to parse .toml settings file! {}", err);
                        success = TofReturnCode::GeneralFail;
                      }
                      Ok(mut config) => {
                        config.from_tofrunconfig(&tcf);
                        info!("We changed the run config to be this {}",config);
                        config.to_toml(String::from(cfg_file.clone()));
                        success = TofReturnCode::Success;
                      }
                    }   
                  }
                }
              }
              TofCommandCode::SetTofRBConfig => {
                info!("Will change tof rb config for next run!");
                match TofRBConfig::from_bytestream(&cmd.payload, &mut 0) {
                  Err(err) => error!("Unable to extract TofEventBuilderConfig from command! {err}"),
                  Ok(tcf)  => {
                    info!("Received config {}",tcf);
                    match LiftofSettings::from_toml(&cfg_file) {
                      Err(err) => {
                        error!("CRITICAL! Unable to parse .toml settings file! {}", err);
                        success = TofReturnCode::GeneralFail;
                      }
                      Ok(mut config) => {
                        config.rb_settings.from_tofrbconfig(&tcf);
                        info!("We changed the run config to be this {}",config);
                        config.to_toml(String::from(cfg_file.clone()));
                        success = TofReturnCode::Success;
                      }
                    }   
                  }
                }
              }
              TofCommandCode::SetDataPublisherConfig => {
                info!("Will change data publisher config for next run!");
                let cfg_file = format!("{}/next/liftof-config.toml", staging_dir.clone());
                match DataPublisherConfig::from_bytestream(&cmd.payload, &mut 0) {
                  Err(err) => error!("Unable to extract TofEventBuilderConfig from command! {err}"),
                  Ok(tcf)  => {
                    info!("Received config {}",tcf);
                    match LiftofSettings::from_toml(&cfg_file) {
                      Err(err) => {
                        error!("CRITICAL! Unable to parse .toml settings file! {}", err);
                        success = TofReturnCode::GeneralFail;
                      }
                      Ok(mut config) => {
                        config.data_publisher_settings.from_datapublisherconfig(&tcf);
                        info!("We changed the event builder settings to be this {}",config.data_publisher_settings);
                        config.to_toml(String::from(cfg_file));
                        success = TofReturnCode::Success;
                      }
                    }   
                  }
                }
              }
              _ => {
                error!("Dealing with command code {} has not been implemented yet!", cmd.command_code);
                success = TofReturnCode::GeneralFail;
              }
            }
            if !args.no_ack {
              send_ack_packet(cmd.command_code, success, &cmd_sender);
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
