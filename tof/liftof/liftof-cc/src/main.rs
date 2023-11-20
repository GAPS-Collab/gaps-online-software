//! LIFTOF-CC - Main C&C (command and control) server application for 
//! tof datataking and control.
//!
//!
//!
//!

#[macro_use] extern crate log;
extern crate env_logger;
extern crate clap;
extern crate json;
extern crate ctrlc;
extern crate zmq;
extern crate tof_dataclasses;
extern crate local_ip_address;
extern crate crossbeam_channel;
extern crate colored;

extern crate liftof_lib;
extern crate liftof_cc;

use std::collections::HashMap;
use std::io;
use std::io::Write;
use std::process::exit;
use std::{fs,
          thread,
          time};
use std::path::{Path, PathBuf};

use clap::{arg,
           command,
           //value_parser,
           //ArgAction,
           //Command,
           Args,
           Parser,
           Subcommand};

use crossbeam_channel as cbc; 
use colored::Colorize;

use tof_dataclasses::events::{MasterTriggerEvent,
                              RBEvent};
use tof_dataclasses::threading::ThreadPool;
use tof_dataclasses::packets::TofPacket;
use tof_dataclasses::manifest::get_rbs_from_sqlite;
use tof_dataclasses::DsiLtbRBMapping;
use tof_dataclasses::commands::TofCommand;
use tof_dataclasses::commands::TofCommandCode;
use liftof_lib::{master_trigger,
                 readoutboard_commander, RunCmd, CalibrationCmd, PowerCmd, PowerStatusEnum, TofComponent, SetCmd};
use liftof_lib::color_log;
use liftof_lib::get_ltb_dsi_j_ch_mapping;
use liftof_cc::threads::{readoutboard_communicator,
                         event_builder};
use liftof_cc::api::tofcmp_and_mtb_moni;
//use liftof_cc::paddle_packet_cache::paddle_packet_cache;
use liftof_cc::flight_comms::global_data_sink;

use liftof_cc::constants::*;

use liftof_lib::Command;


/*************************************/

#[derive(Parser, Debug)]
#[command(author = "J.A.Stoessl", version, about, long_about = None)]
#[command(propagate_version = true)]
struct LiftofCCArgs {
  /// Write the entire TofPacket Stream to a file
  #[arg(short, long, default_value_t = false)]
  write_stream: bool,
  #[arg(short, long, default_value_t = false)]
  use_master_trigger: bool,
  /// Disable monitoring features
  #[arg(short, long, default_value_t = false)]
  no_monitoring: bool,
  /// Enhance output to console
  #[arg(short, long, default_value_t = false)]
  verbose: bool,
  /// A json config file with detector information
  #[arg(short, long)]
  json_config: Option<PathBuf>,
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
  println!("-----------------------------------------------");
  println!(" ** Welcome to liftof-cc \u{1F680} \u{1F388} *****");
  println!(" .. liftof is a software suite for the time-of-flight detector (TOF) ");
  println!(" .. for the GAPS experiment \u{1F496}");
  println!(" .. This is the Command&Control server which connects to the MasterTriggerBoard and the ReadoutBoards");
  println!(" .. see the gitlab repository for documentation and submitting issues at" );
  println!(" **https://uhhepvcs.phys.hawaii.edu/Achim/gaps-online-software/-/tree/main/tof/liftof**");
  
  // deal with command line arguments
  let args = LiftofCCArgs::parse();

  let verbose = args.verbose;

  //error!("error");
  //warn!("warn");
  //info!("info");
  //debug!("debug");
  //trace!("trace");
 
  let write_stream = args.write_stream;
  if write_stream {
    info!("Will write the entire stream to files");
  }
  let no_monitoring = args.no_monitoring;
  if no_monitoring {
    warn!("All monitoring features disabled!");
  }

  let json_content  : String;
  let config        : json::JsonValue;
  
  let nboards       : usize;

  let use_master_trigger        = args.use_master_trigger;
  let mut master_trigger_ip     = String::from("");
  let mut master_trigger_port   = 0usize;
  // create copies, since we need this information
  // for 2 threads at least (moni and event)
  let mut master_trigger_ip_c   = String::from("");
  let mut master_trigger_port_c = 0usize;
  
  match args.json_config {
    None => panic!("No .json config file provided! Please provide a config file with --json-config or -j flag!"),
    Some(_) => {
      json_content = std::fs::read_to_string(args.json_config.as_ref().unwrap()).expect("Can not open json file");
      config = json::parse(&json_content).expect("Unable to parse json file");
    } // end Some
  } // end match

  let mut ltb_rb_map : DsiLtbRBMapping = HashMap::<u8,HashMap::<u8,HashMap::<u8,(u8,u8)>>>::new();
  if use_master_trigger {
    match args.json_ltb_rb_map {
      None => {
        panic!("Will need json ltb -> rb mapping when MasterTrigger shall be used")
      },
      Some(_json_ltb_rb_map) => {
        ltb_rb_map = get_ltb_dsi_j_ch_mapping(_json_ltb_rb_map);
      }
    }

    master_trigger_ip     = config["master_trigger"]["ip"].as_str().unwrap().to_owned();
    master_trigger_port   = config["master_trigger"]["port"].as_usize().unwrap();
    master_trigger_ip_c   = master_trigger_ip.clone();
    master_trigger_port_c = master_trigger_port.clone();
    info!("Will connect to the master trigger board at {}:{}", master_trigger_ip, master_trigger_port);
  } else {
    println!("==> Will NOT connect to the MTB, since -u has not been provided in the commandlline!");
  }
 
  let runid                 = config["run_id"].as_usize().unwrap(); 
  let mut write_stream_path = config["stream_savepath"].as_str().unwrap().to_owned();
  let calib_file_path       = config["calibration_file_path"].as_str().unwrap().to_owned();
  let db_path               = Path::new(config["db_path"].as_str().unwrap());
  let db_path_c             = db_path.clone();
  let mut rb_list           = get_rbs_from_sqlite(db_path_c);
  let rb_ignorelist  = &config["rb_ignorelist"];
  //exit(0);
  for k in 0..rb_ignorelist.len() {
    println!("=> We will remove RB {} due to it being marked as IGNORE in the config file!", rb_ignorelist[k]);
    let bad_rb = rb_ignorelist[k].as_u8().unwrap();
    rb_list.retain(|x| x.rb_id != bad_rb);
  }
  nboards = rb_list.len();
  println!("=> We will use the following tof manifest:");
  println!("== ==> RBs [{}]:", rb_list.len());
  for rb in &rb_list {
    println!("\t {}", rb);
  }

  // Prepare outputfiles
  let mut stream_files_path = PathBuf::from(write_stream_path);
  if write_stream {
    stream_files_path.push(runid.to_string().as_str());
    // Create directory if it does not exist
    // Check if the directory exists
    if let Ok(metadata) = fs::metadata(&stream_files_path) {
      if metadata.is_dir() {
        println!("=> Directory {} exists.", stream_files_path.display());
        // FILXME - in flight, we can not have interactivity.
        // But the whole system with the run ids might change 
        // anyway
        print!("=> You are risking overwriting files in that directory. You might have used rununmber {} before. Are you sure you want to continue? (YES/<any>): ", runid);
        io::stdout().flush().unwrap(); // Ensure the prompt is displayed
        // Read user input
        let mut input = String::new();
        io::stdin().read_line(&mut input).unwrap();
        // Trim leading/trailing whitespaces and convert to lowercase
        let input = input.trim().to_lowercase();
        // Check user input and end the program if desired
        if input == "YES" {
          println!("==> Continuing on request of user...");
        } else {
          println!("==> Abort program!");
        }
      } 
    } else {
      match fs::create_dir(&stream_files_path) {
        Ok(())   => println!("=> Created {} to save stream data", stream_files_path.display()),
        Err(err) => panic!("Failed to create directory: {}, Error {}", stream_files_path.display(), err),
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


  // prepare a thread pool. Currently we have
  // 1 thread per rb, 1 master trigger thread
  // and 1 event builder thread.
  // Also, the paddle cache is its separate 
  // thread.
  // There might
  // be a monitoring thread, too.
  // The number of threads should be fixed at 
  // runtime, but it should be possible to 
  // respawn them
  //let mut nthreads = nboards + 2; // 
  let mut nthreads = 60;
  if use_master_trigger { 
    nthreads += 1;
  }

  let worker_threads = ThreadPool::new(nthreads);

  if !no_monitoring {
    println!("==> Starting main monitoring thread...");
    let tp_to_sink_c = tp_to_sink.clone();
    let moni_interval = 10u64; // in seconds
    worker_threads.execute(move || {
                           tofcmp_and_mtb_moni(&tp_to_sink_c,
                                               &master_trigger_ip_c,
                                               master_trigger_port_c,
                                               moni_interval,
                                               false);
    });
  }

  write_stream_path = String::from(stream_files_path.into_os_string().into_string().expect("Somehow the paths are messed up very badly! So I can't help it and I quit!"));

  println!("==> Starting data sink thread!");
  worker_threads.execute(move || {
                         global_data_sink(&tp_from_client,
                                          write_stream,
                                          write_stream_path,
                                          runid,
                                          verbose);
  });
  println!("==> data sink thread started!");
  println!("==> Will now start rb threads..");
    

  for n in 0..nboards {
    let mut this_rb = rb_list[n].clone();
    let this_tp_to_sink_clone = tp_to_sink.clone();
    this_rb.infer_ip_address();
    let cali_fname = this_rb.guess_calibration_filename();
    this_rb.calib_file = calib_file_path.clone() + &cali_fname;
    //this_rb.calib_file = calib_file_path.clone() + "/" + "rb";
    //if this_rb.rb_id < 10 {
    //  this_rb.calib_file += "0";
    //}
    //this_rb.calib_file += &(this_rb.rb_id).to_string();
    //this_rb.calib_file += "_cal.txt";
    println!("==> Starting RB thread for {}", this_rb);
    let ev_to_builder_c = ev_to_builder.clone();
    worker_threads.execute(move || {
      readoutboard_communicator(&ev_to_builder_c,
                                this_tp_to_sink_clone,
                                &this_rb,
                                runid,
                                false,
                                false);
    });
  } // end for loop over nboards
  println!("==> All RB threads started!");
  
  let one_second = time::Duration::from_millis(1000);
  println!("==> Starting RB commander thread!");
  worker_threads.execute(move || {
    readoutboard_commander(&cmd_receiver);
  });
  if use_master_trigger {
    // start the event builder thread
    println!("==> Starting event builder and master trigger threads...");
    let cmd_sender_2 = cmd_sender.clone();
    worker_threads.execute(move || {
                           event_builder(&master_ev_rec,
                                         &ev_from_rb,
                                         &tp_to_sink);
    });
    // master trigger
    worker_threads.execute(move || {
                           master_trigger(&master_trigger_ip, 
                                          master_trigger_port,
                                          &ltb_rb_map,
                                          &master_ev_send,
                                          &cmd_sender_2,
                                          &mtb_moni_sender,
                                          10,
                                          60,
                                          true);
    });
  } else {
    println!("=> {}", "NOT using the MTB! This means that currently we can only save the blobfiles directly and NO EVENT data will be passed on to the flight computer!".red().bold());
    println!("=> {}", "This mode is still useful for calibration runs or to save RBBinary data locally!".italic());
  }

  let one_minute = time::Duration::from_millis(60000);
  
  println!("==> Sleeping 10 seconds to give the rb's a chance to fire up..");
  thread::sleep(10*one_second);
  println!("==> Sleeping done!");

  // set the handler for SIGINT
  let cmd_sender_c = cmd_sender.clone();
  ctrlc::set_handler(move || {
    println!("==> \u{1F6D1} received Ctrl+C! We will stop triggers and end the run!");
    let end_run =
      TofCommand::from_command_code(TofCommandCode::CmdDataRunStop,0u32);
    let tp = TofPacket::from(&end_run);
    match cmd_sender_c.send(tp) {
     Err(err) => error!("Can not send end run command! {err}"),
     Ok(_)    => ()
    }
    thread::sleep(one_second);
    println!("So long and thanks for all the \u{1F41F}"); 
    exit(0);
  })
  .expect("Error setting Ctrl-C handler");

  match args.command {
    Command::Power(power_cmd) => {
      match power_cmd {
        PowerCmd::All(power_status) => {
          let power_status_enum: PowerStatusEnum = power_status.power_status;
          liftof_cc::send_power(cmd_sender, TofComponent::All, power_status_enum);
        },
        PowerCmd::MT(power_status) => {
          let power_status_enum: PowerStatusEnum = power_status.power_status;
          liftof_cc::send_power(cmd_sender, TofComponent::MT, power_status_enum);
        },
        PowerCmd::AllButMT(power_status) => {
          let power_status_enum: PowerStatusEnum = power_status.power_status;
          liftof_cc::send_power(cmd_sender, TofComponent::AllButMT, power_status_enum);
        },
        PowerCmd::PB(pb_power_opts) => {
          let power_status_enum: PowerStatusEnum = pb_power_opts.power_status;
          let pb_id = pb_power_opts.pb_id;
          liftof_cc::send_power_ID(cmd_sender, TofComponent::PB, power_status_enum, pb_id);
        },
        PowerCmd::RB(rb_power_opts) => {
          let power_status_enum: PowerStatusEnum = rb_power_opts.power_status;
          let rb_id = rb_power_opts.rb_id;
          liftof_cc::send_power_ID(cmd_sender, TofComponent::RB, power_status_enum, rb_id);
        },
        PowerCmd::LTB(ltb_power_opts) => {
          let power_status_enum: PowerStatusEnum = ltb_power_opts.power_status;
          let ltb_id = ltb_power_opts.ltb_id;
          liftof_cc::send_power_ID(cmd_sender, TofComponent::LTB, power_status_enum, ltb_id);
        },
        PowerCmd::Preamp(preamp_power_opts) => {
          let power_status_enum: PowerStatusEnum = preamp_power_opts.power_status;
          let preamp_id = preamp_power_opts.preamp_id;
          let preamp_bias = preamp_power_opts.preamp_bias;
          liftof_cc::send_power_preamp(cmd_sender, power_status_enum, preamp_id, preamp_bias);
        }
      }
    },
    Command::Calibration(calibration_cmd) => {
      match calibration_cmd {
        CalibrationCmd::Default(default_opts) => {
          let voltage_level = default_opts.voltage_level;
          let rb_id = default_opts.rb_id;
          let extra = default_opts.extra;
          liftof_cc::send_default_calibration(cmd_sender, voltage_level, rb_id, extra);
        },
        CalibrationCmd::Noi(noi_opts) => {
          let rb_id = noi_opts.rb_id;
          let extra = noi_opts.extra;
          liftof_cc::send_noi_calibration(cmd_sender, rb_id, extra);
        },
        CalibrationCmd::Voltage(voltage_opts) => {
          let voltage_level = voltage_opts.voltage_level;
          let rb_id = voltage_opts.rb_id;
          let extra = voltage_opts.extra;
          liftof_cc::send_voltage_calibration(cmd_sender, voltage_level, rb_id, extra);
        },
        CalibrationCmd::Timing(timing_opts) => {
          let voltage_level = timing_opts.voltage_level;
          let rb_id = timing_opts.rb_id;
          let extra = timing_opts.extra;
          liftof_cc::send_timing_calibration(cmd_sender, voltage_level, rb_id, extra);
        }
      }
    }
    Command::Set(set_cmd) => {
      match set_cmd {
        SetCmd::LtbThreshold(ltb_threshold_opts) => {
          let ltb_id = ltb_threshold_opts.ltb_id;
          let threshold_level = ltb_threshold_opts.threshold_level;
          liftof_cc::send_ltb_threshold(cmd_sender, ltb_id, threshold_level);
        },
        SetCmd::PreampBias(preamp_bias_opts) => {
          let preamp_id = preamp_bias_opts.preamp_id;
          let preamp_bias = preamp_bias_opts.preamp_bias;
          liftof_cc::send_preamp_bias(cmd_sender, preamp_id, preamp_bias);
        }
      }
    },
    Command::Run(run_cmd) => {
      match run_cmd {
        RunCmd::Start(run_start_opts) => {
          let run_type = run_start_opts.run_type;
          let rb_id = run_start_opts.rb_id;
          let event_no = run_start_opts.event_no;
          let time = run_start_opts.time;
          liftof_cc::send_run_start(cmd_sender, run_type, rb_id, event_no, time);
        },
        RunCmd::Stop(run_stop_opts) => {
          let rb_id = run_stop_opts.rb_id;
          liftof_cc::send_run_stop(cmd_sender, rb_id);
        }
      }
    }
  }
  // start a new data run 
  // let start_run = TofCommand::DataRunStart(1000);
  // let tp = TofPacket::from(&start_run);
  // match cmd_sender.send(tp) {
  //   Err(err) => error!("Unable to send command, error{err}"),
  //   Ok(_)    => ()
  // }

  println!("==> All threads initialized!");
  loop{
    // first we issue start commands until we receive
    // at least 1 positive
    //cmd_sender.send(start_run);
    thread::sleep(1*one_second); 
    thread::sleep(1*one_minute);
    println!("...");
  }
  //println!("Program terminating after specified runtime! So long and thanks for all the {}", fish); 
}
