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
//extern crate zmq;
extern crate tof_dataclasses;
extern crate crossbeam_channel;
extern crate colored;

extern crate liftof_lib;
extern crate liftof_cc;

use std::sync::{
    Arc,
    Mutex,
};

use std::time::{
    Instant,
    Duration,
};
//use std::collections::HashMap;
//use std::io::Write;
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
extern crate indicatif;
use indicatif::{
    ProgressBar,
    ProgressStyle,
};

use tof_dataclasses::errors::CmdError;
use tof_dataclasses::events::{MasterTriggerEvent,
                              RBEvent};
use tof_dataclasses::threading::{
    ThreadControl,
};

use tof_dataclasses::packets::TofPacket;
use tof_dataclasses::manifest::{
    //ReadoutBoard,
    get_rbs_from_sqlite,
};
use tof_dataclasses::commands::TofCommand;
use tof_dataclasses::commands::TofCommandCode;
use liftof_lib::{
    master_trigger,
    readoutboard_commander,
    init_env_logger,
    //color_log,
    LIFTOF_LOGO_SHOW,
    RunCmd,
    CalibrationCmd,
    PowerCmd,
    PowerStatusEnum,
    TofComponent,
    SetCmd
};
use liftof_cc::threads::{
    event_builder,
    flight_cpu_listener,
    global_data_sink,
    monitor_cpu,
    readoutboard_communicator
};

use liftof_lib::settings::LiftofSettings;
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
  #[arg(short, long, default_value_t=0)]
  run_id: usize,
  /// More detailed output for debugging
  #[arg(short, long, default_value_t = false)]
  verbose: bool,
  /// Configuration of liftof-cc. Configure analysis engine,
  /// event builder and general settings.
  #[arg(short, long)]
  config: Option<String>,
  /// For cmd debug purposes
  #[arg(short, long)]
  only_cmd : bool,
  /// List of possible commands
  #[command(subcommand)]
  command: Command,
}

/*************************************/

fn main() {
  init_env_logger();

  // global thread control
  let thread_control = Arc::new(Mutex::new(ThreadControl::new()));

  // welcome banner!
  println!("{}", LIFTOF_LOGO_SHOW);
  println!("-----------------------------------------------");
  println!(" >> Welcome to liftof-cc \u{1F680} \u{1F388} ");
  println!(" >> liftof is a software suite for the time-of-flight detector (TOF) ");
  println!(" >> for the GAPS experiment \u{1F496}");
  println!(" >> This is the Command&Control server");
  println!(" >> It connects to the MasterTriggerBoard and the ReadoutBoards");

  // settings 
  //let foo = LiftofSettings::new();
  //foo.to_toml(String::from("foo-settings.toml"));
  //exit(0);
  
  // log testing
  //error!("error");
  //warn!("warn");
  //info!("info");
  //debug!("debug");
  //trace!("trace");

  // deal with command line arguments
  let config          : LiftofSettings;
  let nboards         : usize;
  
  let args = LiftofCCArgs::parse();
  let verbose = args.verbose;
  let mut cali_from_cmdline = false;   
  match args.command {
    Command::Calibration(ref calibration_cmd) => {
      match calibration_cmd {
        CalibrationCmd::Default(_default_opts) => {
          cali_from_cmdline = true;
        },
        _ => ()
      }
    },
    _ => ()
  }
  
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
  
  println!("=> Using the following config as parsed from the config file:\n{}", config);

  let mtb_address           = config.mtb_address.clone();
  info!("Will connect to the master trigger board at {}!", mtb_address);
 
  // FIXME
  let runid                 = args.run_id;
  let write_stream          = args.write_stream;
  if write_stream && runid == 0 {
    panic!("Writing data to disk requires a run id != 0! Please specify runid through the --run_id parameter!");
  }
  // clone the strings, so we can save the config later
  let mut write_stream_path = config.data_publisher_settings.data_dir.clone();
  let calib_file_path       = config.calibration_dir.clone();
  let runtime_nseconds      = config.runtime_sec;
  //let write_npack_file      = config.packs_per_file;
  let db_path               = Path::new(&config.db_path);
  let cpu_moni_interval     = config.cpu_moni_interval_sec;
  //let flight_address        = config.fc_pub_address.clone();
  let flight_sub_address    = config.fc_sub_address.clone();
  let mtb_settings          = config.mtb_settings.clone();
  let mut gds_settings      = config.data_publisher_settings.clone();
  let flight_pub_address    = config.data_publisher_settings.fc_pub_address.clone();
  let cmd_listener_interval_sec    = config.cmd_listener_interval_sec;
  let run_analysis_engine   = config.run_analysis_engine;
  //let ltb_rb_map            = get_dsi_j_ltbch_vs_rbch_map(db_path);
  let mut rb_list           = get_rbs_from_sqlite(db_path);
  //let mut rb_list           = vec![ReadoutBoard::new();50];
  //for k in 0..rb_list.len() {
  //  rb_list[k].rb_id = k as u8 + 1;
  //}
  let rb_ignorelist         = config.rb_ignorelist.clone();
  for k in 0..rb_ignorelist.len() {
    let bad_rb = rb_ignorelist[k];
    println!("=> We will INGORE RB {:02}, since it is being marked as IGNORE in the config file!", bad_rb);
    rb_list.retain(|x| x.rb_id != bad_rb);
  }
  nboards = rb_list.len();
  println!("=> Expecting {} readoutboards!", rb_list.len());
  //debug!("--> Following RBs are expected:");
  match thread_control.lock() {
    Ok(mut tc) => {
      for rb in &rb_list {
        tc.finished_calibrations.insert(rb.rb_id, false); 
        //debug!("     -{}", rb);
      }
    },
    Err(err) => {
      error!("Can't acquire lock for ThreadControl! Unable to set calibration mode! {err}");
    },
  }
  for rb in &rb_list {
    //rb.load_latest_calibration();
    debug!("     -{}", rb);
    //if verbose {
    //  println!("{}", rb);
    //}
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
    // Write the settings to the directory where 
    // we want to save the run to
    let settings_fname = format!("{}/run{}.toml",stream_files_path.display(), runid); 
    println!("=> Writing data to {}!", stream_files_path.display());
    println!("=> Writing settings to {}!", settings_fname);
    config.to_toml(settings_fname);
  }
  
  // prepare channels for inter thread communications
  let (tp_to_sink, tp_from_client) : (cbc::Sender<TofPacket>, cbc::Receiver<TofPacket>) = cbc::unbounded();
  let mtb_moni_sender = tp_to_sink.clone();

  // master thread -> event builder ocmmuncations
  let (master_ev_send, master_ev_rec): (cbc::Sender<MasterTriggerEvent>, cbc::Receiver<MasterTriggerEvent>) = cbc::unbounded(); 
  
  // readout boards <-> paddle cache communications 
  let (ev_to_builder, ev_from_rb) : (cbc::Sender<RBEvent>, cbc::Receiver<RBEvent>) = cbc::unbounded();
  let (cmd_sender, cmd_receiver)  : (cbc::Sender<TofPacket>, cbc::Receiver<TofPacket>) = cbc::unbounded();

  let one_minute = time::Duration::from_millis(60000);
  let only_cmd   = args.only_cmd;
  // no cpu monitoring for cmdline calibration tasks
  let one_second = time::Duration::from_millis(1000);
  if !only_cmd {
    if !cali_from_cmdline && cpu_moni_interval > 0 {
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
              verbose) // don't print when we are calibrating
            })
          .expect("Failed to spawn cpu-monitoring thread!");
    }
    write_stream_path = String::from(stream_files_path.into_os_string().into_string().expect("Somehow the paths are messed up very badly! So I can't help it and I quit!"));
    gds_settings.data_dir = write_stream_path;

    println!("==> Starting data sink thread!");
    let thread_control_gds = thread_control.clone();
    let _data_sink_thread = thread::Builder::new()
      .name("data-sink".into())
      .spawn(move || {
        global_data_sink(&tp_from_client,
                         write_stream,
                         runid,
                         &gds_settings,
                         verbose,
                         thread_control_gds);
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
    
    println!("==> Starting RB commander thread!");
    let _cmd_receiver_rb_comms = cmd_receiver.clone();
    let _rb_cmd_thread = thread::Builder::new()
      .name("rb-commander".into())
      .spawn(move || {
         readoutboard_commander(&_cmd_receiver_rb_comms);
       })
      .expect("Failed to spawn rb-commander thread!");
    println!("==> Sleeping 5 seconds to give the rb's a chance to fire up..");
    thread::sleep(5*one_second);
    println!("==> Sleeping done!");
    // start the event builder thread
    if !cali_from_cmdline {
      println!("==> Starting event builder and master trigger threads...");
      let db_path_string    = config.db_path.clone();
      let settings          = config.event_builder_settings;
      let thread_control_eb = thread_control.clone();
      let _evb_thread = thread::Builder::new()
        .name("event-builder".into())
        .spawn(move || {
                        event_builder(&master_ev_rec,
                                      &ev_from_rb,
                                      &tp_to_sink,
                                      runid as u32,
                                      db_path_string,
                                      settings,
                                      thread_control_eb);
         })
        .expect("Failed to spawn event-builder thread!");
      // master trigger
      //let thread_control_mt = thread_control.clone();
      let _mtb_thread = thread::Builder::new()
        .name("master-trigger".into())
        .spawn(move || {
                        master_trigger(mtb_address, 
                                       &master_ev_send,
                                       &mtb_moni_sender,
                                       mtb_settings,
                                       // verbosity is currently too much 
                                       // output
                                       verbose);
        })
      .expect("Failed to spawn master-trigger thread!");
    } 

    // set the handler for SIGINT
    let cmd_sender_1 = cmd_sender.clone();
    ctrlc::set_handler(move || {
      println!("==> \u{1F6D1} Caught [SIGING] (allegedly Ctrl+C has been pressed)! Sending >>end run<< signal to all boards!");
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
  }

  let return_val: Result<TofCommandCode, CmdError>;
  let cmd_sender_c = cmd_sender.clone();
  let mut dont_stop = false;
  // let's give everything a little time to come up
  // before we issues the commands
  thread::sleep(5*one_second);
  match args.command {
    Command::Listen(_) => {
      let _flight_address_sub_c = flight_sub_address.clone();
      let _flight_address_pub_c = flight_pub_address;
      let _thread_control_c = thread_control.clone();
      let _cmd_sender_c = cmd_sender.clone();
      let _cmd_receiver_c = cmd_receiver.clone();
      let _cmd_interval_sec: u64 = cmd_listener_interval_sec;
      let _flight_cpu_listener = thread::Builder::new()
                    .name("flight-cpu-listener".into())
                    .spawn(move || {
                                    flight_cpu_listener(&_flight_address_sub_c,
                                                        &_flight_address_pub_c,
                                                        &_cmd_receiver_c,
                                                        &_cmd_sender_c,
                                                        _cmd_interval_sec,
                                                        _thread_control_c);
                    })
                    .expect("Failed to spawn flight-cpu-listener thread!");
      dont_stop = true;
      return_val = Ok(TofCommandCode::CmdListen);
    },
    Command::Ping(ping_cmd) => {
      let component = TofComponent::from(ping_cmd.component);
      match component {
        TofComponent::TofCpu => return_val = liftof_cc::send_ping_response(None),
        TofComponent::RB  |
        TofComponent::LTB |
        TofComponent::MT     => return_val = liftof_cc::send_ping(None,
                                                                  cmd_sender_c,
                                                                  component,
                                                                  ping_cmd.id),
        _                    => {
          error!("The ping command is not implemented for this TofComponent!");
          return_val = Err(CmdError::NotImplementedError);
        }
      }
    },
    Command::Moni(moni_cmd) => {
      let component = TofComponent::from(moni_cmd.component);
      match component {
        TofComponent::TofCpu => return_val = liftof_cc::send_moni_response(None),
        TofComponent::RB    |
        TofComponent::LTB   |
        TofComponent::MT     => return_val = liftof_cc::send_moni(None,
                                                                  cmd_sender_c,
                                                                  component,
                                                                  moni_cmd.id),
        _                    => {
          error!("The moni command is not implemented for this TofComponent!");
          return_val = Err(CmdError::NotImplementedError);
        }
      }
    },
    Command::SystemdReboot(systemd_reboot_cmd) => {
      let rb_id = systemd_reboot_cmd.id;
      return_val = liftof_cc::send_systemd_reboot(None,
                                                                  cmd_sender_c,
                                                                  rb_id);
    },
    Command::Power(power_cmd) => {
      match power_cmd {
        PowerCmd::All(power_status) => {
          let power_status_enum: PowerStatusEnum = power_status.status;
          return_val = liftof_cc::send_power(None,
                                                                  cmd_sender_c,
                                                                  TofComponent::AllButTofCpu,
                                                                  power_status_enum);
        },
        PowerCmd::MT(power_status) => {
          let power_status_enum: PowerStatusEnum = power_status.status;
          return_val = liftof_cc::send_power(None,
                                                                  cmd_sender_c,
                                                                  TofComponent::MT,
                                                                  power_status_enum);
        },
        PowerCmd::AllButMT(power_status) => {
          let power_status_enum: PowerStatusEnum = power_status.status;
          return_val = liftof_cc::send_power(None,
                                                                  cmd_sender_c,
                                                                  TofComponent::AllButTofCpuMT,
                                                                  power_status_enum);
        },
        PowerCmd::LTB(ltb_power_opts) => {
          let power_status_enum: PowerStatusEnum = PowerStatusEnum::from(ltb_power_opts.status);
          let ltb_id = ltb_power_opts.id;
          return_val = liftof_cc::send_power_id(None,
                                                                  cmd_sender_c,
                                                                  TofComponent::LTB,
                                                                  power_status_enum,
                                                                  ltb_id);
        },
        PowerCmd::Preamp(preamp_power_opts) => {
          let power_status_enum: PowerStatusEnum = PowerStatusEnum::from(preamp_power_opts.status);
          let preamp_id = preamp_power_opts.id;
          return_val = liftof_cc::send_power_id(None,
                                                                  cmd_sender_c,
                                                                  TofComponent::Preamp,
                                                                  power_status_enum,
                                                                  preamp_id);
        }
      }
    },
    Command::Calibration(calibration_cmd) => {
      match calibration_cmd {
        CalibrationCmd::Default(default_opts) => {
          let voltage_level = default_opts.level;
          let rb_id = default_opts.id;
          let extra = default_opts.extra;
          println!("=> Received calibration default command! Will init run start...");
          return_val = liftof_cc::send_default_calibration(None,cmd_sender_c, voltage_level, rb_id, extra);
          println!("=> calibration default command resulted in {:?}", return_val);
          println!("=> .. now we need to wait until the calibration is finished!");
          // if that is successful, we need to wait
          match thread_control.lock() {
            Ok(mut tc) => {
              tc.calibration_active = true;
            },
            Err(err) => {
              error!("Can't acquire lock for ThreadControl! Unable to set calibration mode! {err}");
            },
          }
          // now we wait until the calibrations are finished
          //let cali_wait_timer     = Instant::now();
          let mut cali_received   : u64;
          let bar_template : &str = "[{elapsed_precise}] {prefix} {msg} {spinner} {bar:60.blue/grey} {pos:>7}/{len:7}";
          let bar_label  = String::from("Acquiring RB calibration data");
          let bar_style  = ProgressStyle::with_template(bar_template).expect("Unable to set progressbar style!");
          let bar = ProgressBar::new(rb_list.len() as u64); 
          bar.set_position(0);
          bar.set_message (bar_label);
          bar.set_prefix  ("\u{2699}\u{1F4D0}");
          bar.set_style   (bar_style);

          loop {
            cali_received = 0;
            match thread_control.lock() {
              Ok(tc) => {
                for rbid in &rb_list {
                  if tc.finished_calibrations[&rbid.rb_id] {
                    cali_received += 1;
                  }
                }
              },
              Err(err) => {
                error!("Can't acquire lock for ThreadControl! Unable to set calibration mode! {err}");
              },
            }
            bar.set_position(cali_received);
            thread::sleep(5*one_second);
            if cali_received as usize == rb_list.len() {
              break;
            }
          }
          bar.finish();
          println!("=> All calibrations acquired!");
          println!(">> So long and thanks for all the \u{1F41F} <<"); 
          exit(0);
        },
        CalibrationCmd::Noi(noi_opts) => {
          let rb_id = noi_opts.id;
          let extra = noi_opts.extra;
          return_val = liftof_cc::send_noi_calibration(None,cmd_sender_c, rb_id, extra);
        },
        CalibrationCmd::Voltage(voltage_opts) => {
          let voltage_level = voltage_opts.level;
          let rb_id = voltage_opts.id;
          let extra = voltage_opts.extra;
          return_val = liftof_cc::send_voltage_calibration(None,cmd_sender_c, voltage_level, rb_id, extra);
        },
        CalibrationCmd::Timing(timing_opts) => {
          let voltage_level = timing_opts.level;
          let rb_id = timing_opts.id;
          let extra = timing_opts.extra;
          return_val = liftof_cc::send_timing_calibration(None,cmd_sender_c, voltage_level, rb_id, extra);
        }
      }
    }
    Command::Set(set_cmd) => {
      match set_cmd {
        SetCmd::LtbThreshold(ltb_threshold_opts) => {
          let ltb_id = ltb_threshold_opts.id;
          let threshold_name = ltb_threshold_opts.name;
          let threshold_level = ltb_threshold_opts.level;
          return_val = liftof_cc::send_ltb_threshold_set(None,
                                                                  cmd_sender_c,
                                                                  ltb_id,
                                                                  threshold_name,
                                                                  threshold_level);
        },
        SetCmd::PreampBias(preamp_bias_opts) => {
          let preamp_id = preamp_bias_opts.id;
          let preamp_bias = preamp_bias_opts.bias;
          return_val = liftof_cc::send_preamp_bias_set(None,
                                                                  cmd_sender_c,
                                                                  preamp_id,
                                                                  preamp_bias);
        }
      }
    },
    Command::Run(run_cmd) => {
      match run_cmd {
        RunCmd::Start(run_start_opts) => {
          let run_type = run_start_opts.run_type;
          let rb_id = run_start_opts.id;
          let event_no = run_start_opts.no;
          return_val = liftof_cc::send_run_start(None,
                                                                  cmd_sender_c,
                                                                  run_type,
                                                                  rb_id,
                                                                  event_no);
        },
        RunCmd::Stop(run_stop_opts) => {
          let rb_id = run_stop_opts.id;
          return_val = liftof_cc::send_run_stop(None,
                                                                  cmd_sender_c,
                                                                  rb_id);
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
  let pb = ProgressBar::new_spinner();
  pb.enable_steady_tick(Duration::from_millis(500));
  //pb.set_style(
  //    ProgressStyle::with_template("{spinner:.blue} {msg}")
  //        .unwrap()
  //        // For more spinners check out the cli-spinners project:
  //        // https://github.com/sindresorhus/cli-spinners/blob/master/spinners.json
  //        .tick_strings(&[
  //            "▹▹▹▹▹",
  //            "▸▹▹▹▹",
  //            "▹▸▹▹▹",
  //            "▹▹▸▹▹",
  //            "▹▹▹▸▹",
  //            "▹▹▹▹▸",
  //            "▪▪▪▪▪",
  //        ]),
  //);
  pb.set_message(".. acquiring data ..");
  loop{
    // first we issue start commands until we receive
    // at least 1 positive
    //cmd_sender.send(start_run);
    thread::sleep(1*one_minute);
    thread::sleep(Duration::from_millis(500));
    //println!("...");
    // I think the main shouldn't die if we are in listening mode
    if dont_stop {
      continue;
    } else if program_start.elapsed().as_secs_f64() > runtime_nseconds as f64 {
      pb.finish_with_message("Done");
      println!("=> Runtime seconds of {} have expired!", runtime_nseconds);
      println!("=> Ending program. If you don't want that behaviour, change the confifguration file.");
      println!(">> So long and thanks for all the \u{1F41F} <<"); 
      exit(0);    
    }
  }
}
