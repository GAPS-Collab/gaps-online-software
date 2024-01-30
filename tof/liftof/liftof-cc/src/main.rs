//! LIFTOF-CC - Main C&C (command and control) server application for 
//! tof datataking and control.
//!
//!
//!
//!

#[macro_use] extern crate log;
extern crate env_logger;
extern crate clap;
extern crate ctrlc;
extern crate zmq;
extern crate tof_dataclasses;
extern crate local_ip_address;
extern crate crossbeam_channel;
extern crate colored;

extern crate liftof_lib;
extern crate liftof_cc;

use std::sync::{
    Arc,
    Mutex,
};

use std::time::Instant;
//use std::collections::HashMap;
use std::io::Write;
use std::process::exit;
use std::{
    fs,
    thread,
    time
};
use std::path::{
    Path,
    PathBuf,
};

use clap::{arg,
           command,
           Parser};

use crossbeam_channel as cbc; 
//use colored::Colorize;

use tof_dataclasses::errors::CmdError;
use tof_dataclasses::events::{MasterTriggerEvent,
                              RBEvent};
use tof_dataclasses::threading::{
    ThreadControl,
};

use tof_dataclasses::packets::TofPacket;
use tof_dataclasses::manifest::{
    //ReadoutBoard,
    get_rbs_from_sqlite
};
use tof_dataclasses::DsiLtbRBMapping;
use tof_dataclasses::commands::TofCommand;
use tof_dataclasses::commands::TofCommandCode;
use liftof_lib::{
    master_trigger,
    readoutboard_commander,
    color_log,
    get_ltb_dsi_j_ch_mapping,
    LIFTOF_LOGO_SHOW,
    RunCmd, CalibrationCmd, PowerCmd, PowerStatusEnum, TofComponent, SetCmd
};
use liftof_cc::threads::{event_builder, flight_cpu_listener, global_data_sink, monitor_cpu, readoutboard_communicator};
use liftof_cc::settings::{
    LiftofCCSettings,
};
use liftof_lib::Command;

/*************************************/

#[derive(Parser, Debug)]
#[command(author = "J.A.Stoessl", version, about, long_about = None)]
#[command(propagate_version = true)]
struct LiftofCCArgs {
  /// Write the entire TofPacket Stream to a file
  #[arg(short, long, default_value_t = false)]
  write_stream: bool,
  /// Define a run id for later identification
  #[arg(short, long, default_value_t = 0)]
  run_id: usize,
  /// More detailed output for debugging
  #[arg(short, long, default_value_t = false)]
  verbose: bool,
  /// Configuration of liftof-cc. Configure analysis engine,
  /// event builder and general settings.
  #[arg(short, long)]
  config: Option<String>,
  /// A json file wit the ltb(dsi, j, ch) -> rb_id, rb_ch mapping.
  #[arg(long)]
  json_ltb_rb_map : Option<PathBuf>,
  /// List of possible commands
  #[command(subcommand)]
  command: Command,
}

/*************************************/

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

  // welcome banner!
  println!("{}", LIFTOF_LOGO_SHOW);
  println!("-----------------------------------------------");
  println!(" >> Welcome to liftof-cc \u{1F680} \u{1F388} ");
  println!(" >> liftof is a software suite for the time-of-flight detector (TOF) ");
  println!(" >> for the GAPS experiment \u{1F496}");
  println!(" >> This is the Command&Control server");
  println!(" >> It connects to the MasterTriggerBoard and the ReadoutBoards");
  
  // deal with command line arguments
  let args = LiftofCCArgs::parse();
  let verbose = args.verbose;
  // log testing
  //error!("error");
  //warn!("warn");
  //info!("info");
  //debug!("debug");
  //trace!("trace");
 
  let write_stream = args.write_stream;
  if write_stream {
    info!("Will write the entire stream to files");
  }

  let config          : LiftofCCSettings;
  let nboards         : usize;

  //let foo_settings = LiftofCCSettings::new();
  //foo_settings.to_toml(String::from("foo"));
  //exit(0);

  match args.config {
    None => panic!("No config file provided! Please provide a config file with --config or -c flag!"),
    Some(cfg_file) => {
      match LiftofCCSettings::from_toml(cfg_file) {
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
  
  println!("=> Using the following config as parsed from the config file:\n{}", config);
  //exit(0);

  let ltb_rb_map : DsiLtbRBMapping;// = HashMap::<u8,HashMap::<u8,HashMap::<u8,(u8,u8)>>>::new();
  match args.json_ltb_rb_map {
    None => {
      panic!("Will need json ltb -> rb mapping when MasterTrigger shall be used")
    },
    Some(_json_ltb_rb_map) => {
      ltb_rb_map = get_ltb_dsi_j_ch_mapping(_json_ltb_rb_map);
    }
  }

  let mtb_address           = config.mtb_address;
  info!("Will connect to the master trigger board at {}!", mtb_address);
 
  // FIXME
  let runid               = args.run_id;
  let mut write_stream_path = config.data_dir;
  let calib_file_path            = config.calibration_dir;
  let runtime_nseconds      = config.runtime_sec;
  let write_npack_file      = config.packs_per_file;
  let db_path               = Path::new(&config.db_path);
  let mtb_moni_interval     = config.mtb_moni_interval_sec;
  let cpu_moni_interval          = config.cpu_moni_interval_sec;
  let flight_address        = config.fc_pub_address;
  let mtb_trace_suppression = config.mtb_trace_suppression;
  let run_analysis_engine   = config.run_analysis_engine;
  let mut rb_list           = get_rbs_from_sqlite(db_path);
  //let mut rb_list           = vec![ReadoutBoard::new();50];
  for k in 0..rb_list.len() {
    rb_list[k].rb_id = k as u8 + 1;
  }
  let rb_ignorelist         = config.rb_ignorelist.clone();
  for k in 0..rb_ignorelist.len() {
    println!("=> We will remove RB {} due to it being marked as IGNORE in the config file!", rb_ignorelist[k]);
    let bad_rb = rb_ignorelist[k];
    rb_list.retain(|x| x.rb_id != bad_rb);
  }
  let rb_list_c = rb_list.clone();
  for k in rb_list_c {
    println!("{}", k);
  }
  nboards = rb_list.len();
  println!("=> Expecting {} readoutboards!", rb_list.len());
  info!("--> Following RBs are expected:");
  for rb in &rb_list {
    info!("     -{}", rb);
  }

  // A global kill timer
  let program_start = Instant::now();

  // Prepare outputfiles
  let mut stream_files_path = PathBuf::from(write_stream_path);
  if write_stream {
    stream_files_path.push(runid.to_string().as_str());
    // Create directory if it does not exist
    // Check if the directory exists
    if let Ok(metadata) = fs::metadata(&stream_files_path) {
      if metadata.is_dir() {
        println!("=> Directory {} for run number {} already consists and may contain files!", stream_files_path.display(), runid);
        // FILXME - in flight, we can not have interactivity.
        // But the whole system with the run ids might change 
      } 
    } else {
      match fs::create_dir(&stream_files_path) {
        Ok(())   => println!("=> Created {} to save stream data", stream_files_path.display()),
        Err(err) => panic!("Failed to create directory: {}! {}", stream_files_path.display(), err),
      }
    }
  }
  //let matches = command!() // requires `cargo` feature
  //     //.arg(arg!([name] "Optional name to operate on"))
  //     .arg(
  //         arg!(
  //             -c --json-config <FILE> "Sets a custom config file"
  //         )
  //         // We don't have syntax yet for optional options, so manually calling `required`
  //         .required(true)
  //         .value_parser(value_parser!(std::path::PathBuf)),
  //     )
  //     .arg(arg!(
  //         -d --debug ... "Turn debugging information on"
  //     ))
  //     //.subcommand(
  //     //    Command::new("test")
  //     //        .about("does testing things")
  //     //        .arg(arg!(-l --list "lists test values").action(ArgAction::SetTrue)),
  //     //)
  //     .get_matches();

  // // You can check the value provided by positional arguments, or option arguments
  // //if let Some(name) = matches.get_one::<String>("name") {
  // //    println!("Value for name: {}", name);
  // //}

  // if let Some(config_path) = matches.get_one::<std::path::PathBuf>("json-config") {
  //     println!("Value for config: {}", config_path.display());
  // }

  // // You can see how many times a particular flag or argument occurred
  // // Note, only flags can have multiple occurrences
  // match matches
  //     .get_one::<u8>("debug")
  //     .expect("Count's are defaulted")
  // {
  //     0 => println!("Debug mode is off"),
  //     1 => println!("Debug mode is kind of on"),
  //     2 => println!("Debug mode is on"),
  //     _ => println!("Don't be crazy"),
  // }

  //// You can check for the existence of subcommands, and if found use their
  //// matches just as you would the top level cmd
  //if let Some(matches) = matches.subcommand_matches("test") {
  //    // "$ myapp test" was run
  //    if *matches.get_one::<bool>("list").expect("defaulted by clap") {
  //        // "$ myapp test -l" was run
  //        println!("Printing testing lists...");
  //    } else {
  //        println!("Not printing testing lists...");
  //    }
  //}

  println!(" .. .. .. .. .. .. .. ..");
  

  // prepare channels for inter thread communications
 
  let (tp_to_sink, tp_from_client) : (cbc::Sender<TofPacket>, cbc::Receiver<TofPacket>) = cbc::unbounded();
  let mtb_moni_sender = tp_to_sink.clone();

  // send the rate from the master trigger to the main thread
  warn!("Endpoint of rate from mt channel currently not connected!");
  // master thread -> event builder ocmmuncations
  let (master_ev_send, master_ev_rec): (cbc::Sender<MasterTriggerEvent>, cbc::Receiver<MasterTriggerEvent>) = cbc::unbounded(); 
  
  // readout boards <-> paddle cache communications 
  let (ev_to_builder, ev_from_rb) : (cbc::Sender<RBEvent>, cbc::Receiver<RBEvent>) = cbc::unbounded();
  let (cmd_sender, cmd_receiver) : (cbc::Sender<TofPacket>, cbc::Receiver<TofPacket>) = cbc::unbounded();

  let ctx = zmq::Context::new();
  // I guess expect is fine here, see above
  let socket = ctx.socket(zmq::SUB).expect("Unable to create 0MQ SUB socket!");

  let thread_control = Arc::new(Mutex::new(ThreadControl::new()));

  if cpu_moni_interval > 0 {
    println!("==> Starting main monitoring thread...");
    let tp_to_sink_c = tp_to_sink.clone();
    let _thread_control_c = thread_control.clone();
    // this is anonymus, but we control the thread
    // through the thread control mechanism, so we
    // can still end it.
    let _cpu_moni_thread = thread::Builder::new()
         .name("cpu-monitoring".into())
         .spawn(move || {
           monitor_cpu(
             tp_to_sink_c,
             cpu_moni_interval,
             _thread_control_c,
             verbose)
          })
         .expect("Failed to spawn cpu-monitoring thread!");
  }

  write_stream_path = String::from(stream_files_path.into_os_string().into_string().expect("Somehow the paths are messed up very badly! So I can't help it and I quit!"));

  // this is the tailscale address
  //let flight_address = format!("tcp://100.101.96.10:{}", DATAPORT);
  // this is the address in the flight network
  // flight_address = format!("tcp://10.0.1.1:{}", DATAPORT);
  println!("==> Starting data sink thread!");
  let flight_address_c = flight_address.clone();
  let _data_sink_thread = thread::Builder::new()
       .name("data-sink".into())
       .spawn(move || {
         global_data_sink(&tp_from_client,
                          &flight_address_c,
                          write_stream,
                          write_stream_path,
                          write_npack_file,
                          runid,
                          verbose);
        })
       .expect("Failed to spawn data-sink thread!");
  println!("==> data sink thread started!");
  println!("==> Will now start rb threads..");

  for n in 0..nboards {
    let mut this_rb           = rb_list[n].clone();
    let this_tp_to_sink_clone = tp_to_sink.clone();
    this_rb.calib_file_path   = calib_file_path.clone();
    match this_rb.load_latest_calibration() {
      Err(err) => error!("Unable to load calibration for RB {}! {}", this_rb.rb_id, err),
      Ok(_)    => ()
    }
    println!("==> Starting RB thread for {}", this_rb);
    let ev_to_builder_c = ev_to_builder.clone();
    let thread_name = format!("rb-comms-{}", this_rb.rb_id);
    let _rb_comm_thread = thread::Builder::new()
         .name(thread_name)
         .spawn(move || {
      readoutboard_communicator(&ev_to_builder_c,
                                this_tp_to_sink_clone,
                                &this_rb,
                                runid,
                                false,
                                run_analysis_engine);
          })
         .expect("Failed to spawn readoutboard-communicator thread!");
  } // end for loop over nboards
  println!("==> All RB threads started!");
  
  let one_second = time::Duration::from_millis(1000);
  let cmd_receiver_c = cmd_receiver.clone();
  println!("==> Starting RB commander thread!");
    let _rb_cmd_thread = thread::Builder::new()
         .name("rb-commander".into())
         .spawn(move || {
            readoutboard_commander(&cmd_receiver_c);
          })
         .expect("Failed to spawn rb-commander thread!");
  // start the event builder thread
  println!("==> Starting event builder and master trigger threads...");
  let cmd_sender_2 = cmd_sender.clone();
  let settings = config.event_builder_settings;
    let _evb_thread = thread::Builder::new()
         .name("cpu-monitoring".into())
         .spawn(move || {
                         event_builder(&master_ev_rec,
                                       &ev_from_rb,
                                       &tp_to_sink,
                                       settings);
          })
         .expect("Failed to spawn cpu-monitoring thread!");
  // master trigger
    let _mtb_thread = thread::Builder::new()
         .name("master-trigger".into())
         .spawn(move || {
                         master_trigger(mtb_address, 
                                        &ltb_rb_map,
                                        &master_ev_send,
                                        &cmd_sender_2,
                                        &mtb_moni_sender,
                                        mtb_moni_interval,
                                        60, // allowed mtb timeout
                                        mtb_trace_suppression,
                                        false,
                                        false);
          })
         .expect("Failed to spawn cpu-monitoring thread!");

  let one_minute = time::Duration::from_millis(60000);
  
  println!("==> Sleeping 10 seconds to give the rb's a chance to fire up..");
  thread::sleep(10*one_second);
  println!("==> Sleeping done!");

  // set the handler for SIGINT
  let cmd_sender_1 = cmd_sender.clone();
  ctrlc::set_handler(move || {
    println!("==> \u{1F6D1} received Ctrl+C! We will stop triggers and end the run!");
    let end_run =
      TofCommand::from_command_code(TofCommandCode::CmdDataRunStop,0u32);
    let tp = TofPacket::from(&end_run);
    match cmd_sender_1.send(tp) {
     Err(err) => error!("Can not send end run command! {err}"),
     Ok(_)    => ()
    }
    thread::sleep(one_second);
    println!(">> So long and thanks for all the \u{1F41F} <<"); 
    exit(0);
  })
  .expect("Error setting Ctrl-C handler");

  let return_val: Result<TofCommandCode, CmdError>;
  let cmd_sender_c = cmd_sender.clone();
  match args.command {
    Command::Listen(_) => {
      let _flight_address_c = flight_address.clone();
      let _thread_control_c = thread_control.clone();
      let _cmd_interval: u64 = 1000;
      let _cmd_sender_c = cmd_sender.clone();
      let _cmd_receiver_c = cmd_receiver.clone();
      let _flight_cpu_listener = thread::Builder::new()
                    .name("flight-cpu-listener".into())
                    .spawn(move || {
                                    flight_cpu_listener(&_flight_address_c,
                                                        &_cmd_receiver_c,
                                                        &_cmd_sender_c,
                                                        _cmd_interval,
                                                        _thread_control_c);
                    })
                    .expect("Failed to spawn flight-cpu-listener thread!");
      return_val = Ok(TofCommandCode::CmdListen);
    },
    Command::Ping(ping_cmd) => {
      match ping_cmd.component {
        TofComponent::TofCpu => return_val = liftof_cc::send_ping_response(cmd_sender_c),
        TofComponent::RB  |
        TofComponent::LTB |
        TofComponent::MT     => return_val = liftof_cc::send_ping(cmd_sender_c, ping_cmd.component, ping_cmd.id),
        _                    => {
          error!("The ping command is not implemented for this TofComponent!");
          return_val = Err(CmdError::NotImplementedError);
        }
      }
    },
    Command::Moni(moni_cmd) => {
      match moni_cmd.component {
        TofComponent::TofCpu => return_val = liftof_cc::send_moni_response(cmd_sender_c),
        TofComponent::RB    |
        TofComponent::LTB   |
        TofComponent::MT     => return_val = liftof_cc::send_moni(cmd_sender_c, moni_cmd.component, moni_cmd.id),
        _                    => {
          error!("The moni command is not implemented for this TofComponent!");
          return_val = Err(CmdError::NotImplementedError);
        }
      }
    },
    Command::SystemdReboot(systemd_reboot_cmd) => {
      let rb_id = systemd_reboot_cmd.id;
      return_val = liftof_cc::send_systemd_reboot(cmd_sender_c, rb_id);
    },
    Command::Power(power_cmd) => {
      match power_cmd {
        PowerCmd::All(power_status) => {
          let power_status_enum: PowerStatusEnum = power_status.status;
          return_val = liftof_cc::send_power(cmd_sender_c, TofComponent::All, power_status_enum);
        },
        PowerCmd::MT(power_status) => {
          let power_status_enum: PowerStatusEnum = power_status.status;
          return_val = liftof_cc::send_power(cmd_sender_c, TofComponent::MT, power_status_enum);
        },
        PowerCmd::AllButMT(power_status) => {
          let power_status_enum: PowerStatusEnum = power_status.status;
          return_val = liftof_cc::send_power(cmd_sender_c, TofComponent::AllButMT, power_status_enum);
        },
        PowerCmd::LTB(ltb_power_opts) => {
          let power_status_enum: PowerStatusEnum = ltb_power_opts.status;
          let ltb_id = ltb_power_opts.id;
          return_val = liftof_cc::send_power_id(cmd_sender_c, TofComponent::LTB, power_status_enum, ltb_id);
        },
        PowerCmd::Preamp(preamp_power_opts) => {
          let power_status_enum: PowerStatusEnum = preamp_power_opts.status;
          let preamp_id = preamp_power_opts.id;
          let preamp_bias = preamp_power_opts.bias;
          return_val = liftof_cc::send_power_preamp(cmd_sender_c, power_status_enum, preamp_id, preamp_bias);
        }
      }
    },
    Command::Calibration(calibration_cmd) => {
      match calibration_cmd {
        CalibrationCmd::Default(default_opts) => {
          let voltage_level = default_opts.level;
          let rb_id = default_opts.id;
          let extra = default_opts.extra;
          return_val = liftof_cc::send_default_calibration(cmd_sender_c, voltage_level, rb_id, extra);
        },
        CalibrationCmd::Noi(noi_opts) => {
          let rb_id = noi_opts.id;
          let extra = noi_opts.extra;
          return_val = liftof_cc::send_noi_calibration(cmd_sender_c, rb_id, extra);
        },
        CalibrationCmd::Voltage(voltage_opts) => {
          let voltage_level = voltage_opts.level;
          let rb_id = voltage_opts.id;
          let extra = voltage_opts.extra;
          return_val = liftof_cc::send_voltage_calibration(cmd_sender_c, voltage_level, rb_id, extra);
        },
        CalibrationCmd::Timing(timing_opts) => {
          let voltage_level = timing_opts.level;
          let rb_id = timing_opts.id;
          let extra = timing_opts.extra;
          return_val = liftof_cc::send_timing_calibration(cmd_sender_c, voltage_level, rb_id, extra);
        }
      }
    }
    Command::Set(set_cmd) => {
      match set_cmd {
        SetCmd::LtbThreshold(ltb_threshold_opts) => {
          let ltb_id = ltb_threshold_opts.id;
          let threshold_name = ltb_threshold_opts.name;
          let threshold_level = ltb_threshold_opts.level;
          return_val = liftof_cc::send_ltb_threshold_set(cmd_sender_c, ltb_id, threshold_name, threshold_level);
        },
        SetCmd::PreampBias(preamp_bias_opts) => {
          let preamp_id = preamp_bias_opts.id;
          let preamp_bias = preamp_bias_opts.bias;
          return_val = liftof_cc::send_preamp_bias_set(cmd_sender_c, preamp_id, preamp_bias);
        }
      }
    },
    Command::Run(run_cmd) => {
      match run_cmd {
        RunCmd::Start(run_start_opts) => {
          let run_type = run_start_opts.run_type;
          let rb_id = run_start_opts.id;
          let event_no = run_start_opts.no;
          return_val = liftof_cc::send_run_start(cmd_sender_c, run_type, rb_id, event_no);
        },
        RunCmd::Stop(run_stop_opts) => {
          let rb_id = run_stop_opts.id;
          return_val = liftof_cc::send_run_stop(cmd_sender_c, rb_id);
        }
      }
    }
  }
  // deal with return values
  match return_val {
    Err(cmd_error) => {
      error!("Error in sending command {cmd_error}");
    },
    Ok(tof_command)  => {
      info!("Successfully sent command {tof_command}");
    }
  }

  println!("==> All threads initialized!");
  loop{
    // first we issue start commands until we receive
    // at least 1 positive
    //cmd_sender.send(start_run);
    thread::sleep(1*one_minute);
    println!("...");
    if program_start.elapsed().as_secs_f64() > runtime_nseconds as f64 {
      println!("=> Runtime seconds of {} have expired!", runtime_nseconds);
      println!("=> Ending program. If you don't want that behaviour, change the confifguration file.");
      exit(0);    
    }
  }
}
