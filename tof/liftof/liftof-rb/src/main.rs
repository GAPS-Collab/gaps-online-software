//! # Radoutboard software for the GAPS experiment, TOF system
//! 
//! This software shall help with data acquisition and commandeering 
//! of the readoutboards (RB) used in the tof system of the GAPS 
//! science experiment.
//!
//! Standalone, statically linked binary to be either run manually 
//! or to be managed by systemd
use std::{thread, time};
use std::time::Duration;
use std::io::Write;

extern crate crossbeam_channel;
extern crate signal_hook;
extern crate env_logger;

use std::os::raw::c_int;
use signal_hook::iterator::Signals;
use signal_hook::consts::signal::{SIGTERM, SIGINT};
use crossbeam_channel::{unbounded,
                        Sender,
                        Receiver};
use local_ip_address::local_ip;

//use std::collections::HashMap;
use std::process::exit;
use liftof_lib::{color_log, CalibrationCmd};

use liftof_rb::threads::{runner,
                         cmd_responder,
                         event_processing,
                         event_cache,
                         monitoring,
                         data_publisher};
use liftof_rb::api::*;
use liftof_rb::control::*;
use tof_dataclasses::threading::ThreadPool;
use tof_dataclasses::packets::TofPacket;
use tof_dataclasses::commands::{//RBCommand,
                                TofOperationMode};
use tof_dataclasses::events::DataType;
use tof_dataclasses::run::RunConfig;
#[macro_use] extern crate log;

extern crate clap;
use clap::{arg,
           command,
           Parser};

use liftof_lib::Command;

#[derive(Parser, Debug)]
#[command(author = "J.A.Stoessl", version, about, long_about = None)]
struct Args {
  /// A json run config file with a RunConfiguration. This option is essential if 
  /// this program is run manually without systemd and not controlled by a central server. 
  /// If in this configuration one wants to take data, ONE HAS TO SUPPLY A RUNCONFIG!
  #[arg(short, long)]
  run_config: Option<std::path::PathBuf>,
  /// Show progress bars to indicate buffer fill values and number of acquired events
  #[arg(long, default_value_t = false)]
  show_progress: bool,
  /// Acquire this many events. This will OVERRIDE the setting from the runconfig. 
  /// A runconfig is STILL NEEDED! However, for quick debugging, we can change the 
  /// number here (for convenience)
  #[arg(short, long, default_value_t = 0)]
  nevents: u32,
  /// Cache size of the internal event cache in events
  #[arg(short, long, default_value_t = 10000)]
  cache_size: usize,
  /// Analyze the waveforms directly on the board. We will not send
  /// waveoform data, but paddle packets instead.
  #[arg(long, default_value_t = false)]
  waveform_analysis: bool,
  /// show moni data 
  #[arg(long, default_value_t = false)]
  verbose : bool,
  /// Write the readoutboard binary data ('.robin') to the board itself
  #[arg(long, default_value_t = false)]
  to_local_file : bool,
  /// Select the monitoring interval for L1 monitoring data
  /// L1 is the fast monitoring - only critical values
  #[arg(long, default_value_t = 10)]
  moni_interval_l1 : u64,
  /// Select the monitoring interval for L2 monitoring data
  /// L2 is the slow monitoring - all values
  #[arg(long, default_value_t = 30)]
  moni_interval_l2 : u64,
  ///// CnC server IP we should be listening to
  //#[arg(long, default_value_t = "10.0.1.1")]
  //cmd_server_ip : &'static str,
  /// Take some events with the poisson trigger and 
  /// check the event ids for duplicates or missing ids
  #[arg(long, default_value_t = false)]
  test_eventids: bool,
  /// List of possible commands
  #[command(subcommand)]
  command: Command,
}

/**********************************************************/

fn main() {
  env_logger::builder()
    .format(|buf, record| {
    writeln!( buf, "[{level}][{module_path}:{line}] {args}",
      level = color_log(&record.level()),
      module_path = record.module_path().unwrap_or("<unknown>"),
      //target = record.target(),
      line = record.line().unwrap_or(0),
      args = record.args()
      )
    }).init();
 
  let args = Args::parse();                   
  let verbose                  = args.verbose;
  let n_events_run             = args.nevents;
  let show_progress            = args.show_progress;
  let cache_size               = args.cache_size;
  let wf_analysis              = args.waveform_analysis;
  let mut to_local_file        = args.to_local_file;
  let run_config               = args.run_config;
  let test_eventids            = args.test_eventids;
  
  //FIMXE - this needs to become part of clap
  let cmd_server_ip = String::from("10.0.1.1");
  //let cmd_server_ip     = args.cmd_server_ip;  
  let this_board_ip = local_ip().expect("Unable to obtainl local board IP. Something is messed up!");
  
  // General parameters, readout board id,, 
  // ip to tof computer
  let rb_id = get_board_id().expect("Unable to obtain board ID!");
  let dna   = get_device_dna().expect("Unable to obtain device DNA!"); 

  // deal with bug
  //let mut channel_reg = read_control_reg(0x44).unwrap();
  //println!("READING CHANNEL MASK {channel_reg}");
  //write_control_reg(0x44, 0x3FF).unwrap();
  //channel_reg = read_control_reg(0x44).unwrap();
  //println!("READING CHANNEL MASK {channel_reg}");

  // welcome banner!
  println!("-----------------------------------------------");
  println!(" ** Welcome to liftof-rb \u{1F680} \u{1F388} *****");
  println!(" .. liftof if a software suite for the time-of-flight detector ");
  println!(" .. for the GAPS experiment \u{1F496}");
  println!(" .. this client can be run standalone or connect to liftof-cc" );
  println!(" .. or liftof-tui for an interactive experience" );
  println!(" .. see the gitlab repository for documentation and submitting issues at" );
  println!(" **https://uhhepvcs.phys.hawaii.edu/Achim/gaps-online-software/-/tree/main/tof/liftof**");
  println!("-----------------------------------------------");
  println!(" => Running client for RB {}", rb_id);
  println!(" => ReadoutBoard DNA {}", dna);
  println!(" => We will BIND this port to the local ip address at {}", this_board_ip);
  println!(" => -- -- PORT {} (0MQ PUB) to publish our data", DATAPORT);
  println!(" => We will CONNECT to the following port on the C&C server at address: {}", cmd_server_ip);
  println!(" => -- -- PORT {} (0MQ SUB) where we will be listening for commands", DATAPORT);
  println!("-----------------------------------------------");
  
  if test_eventids {
    warn!("Testing mode! Only for debugging!");
  }
  if wf_analysis {
    todo!("--waveform-analysis is currently not implemented!");
  }

  // per default the data type should be 
  // header with all waveform data
  //let mut data_type = DataType::Physics;

  let mut rc_config     = RunConfig::new();
  let mut rc_file_path  = std::path::PathBuf::new();
  let mut end_after_run = false;
  let mut calibration = false;
  match args.command {
    // Matching calibration command
    Command::Calibration(_) => calibration = true,
    _ => ()
  }

  if calibration {
    println!("===================================================================");
    println!("=> Readoutboard calibration! This will override ALL other settings!");
    println!("===================================================================");
    end_after_run = true;
    to_local_file = true;
  }
  
  let config_from_shell : bool;
  match run_config {
    None     => {
      if !calibration {
        println!("=> We did not get a runconfig with the -r <RUNCONFIG> commandline switch! Currently we are just listening for input on the socket. This is the desired behavior, if run by systemd. If you want to take data in standalone mode, either send a runconfig to the socket or hit CTRL+C and start the program again, this time suppling the -r <RUNCONFIG> flag or in case you want to calibrate the board, use the calibration command.");
      }
      config_from_shell = false;
    }
    Some(rcfile) => {
      println!("=> Instructed to use runconfig {:?}", rcfile);
      rc_file_path   = rcfile.clone();
      rc_config      = get_runconfig(&rcfile);
      end_after_run  = rc_config.nevents > 0 || rc_config.nseconds > 0;
      config_from_shell = true;
    }
  }
  let file_suffix : String;
  if calibration {
    file_suffix = String::from(".cali.tof.gaps");
  } else {
    file_suffix = String::from(".tof.gaps");
  }
  // some pre-defined time units for 
  // sleeping
  let one_sec     = time::Duration::from_secs(1);  

  // threads and inter-thread communications
  // We have
  // * event_cache thread
  // * buffer reader thread
  // * data analysis/sender thread
  // * monitoring thread
  // * run thread
  // + main thread, which does not need a 
  //   separate thread

  // FIXME - MESSAGES GET CONSUMED!!
  // setting up inter-thread comms
  let (rc_to_runner, rc_from_cmdr)      : 
      (Sender<RunConfig>, Receiver<RunConfig>)                = unbounded();
  let (tp_to_pub, tp_from_client)        : 
      (Sender<TofPacket>, Receiver<TofPacket>)                = unbounded();
  let (tp_to_cache, tp_from_builder) : 
      (Sender<TofPacket>, Receiver<TofPacket>)                = unbounded();
  let (dtf_to_evproc, dtf_from_runner) :                
      (Sender<DataType>, Receiver<DataType>)                  = unbounded();
  
  //let (rbcalib_to_evproc, rbcalib_from_calib)   : 
  //    (Sender<RBCalibrations>, Receiver<RBCalibrations>)                    = unbounded();

  let (opmode_to_cache, opmode_from_runner)     : 
      (Sender<TofOperationMode>, Receiver<TofOperationMode>)                = unbounded();
  let (bs_send, bs_recv)             : (Sender<Vec<u8>>, Receiver<Vec<u8>>) = unbounded(); 


  
  // ucla debugging
  //match setup_drs4() {
  //  Ok(_)    => (),
  //  Err(err) => {
  //    panic!("Setup drs4 failled! Error {err}");
  //  }
  //}
  //match set_active_channel_mask_with_ch9(255) {
  //  Ok(_) => (),
  //  Err(err) => {
  //    panic!("Settint the ch9 register failed!");
  //  }
  //}
  //write_control_reg(0x44,u32::MAX);
  
  //FIXME - restrict to actual number of threads
  let n_threads = 8;
  info!("Will start ThreadPool with {n_threads} threads");
  let workforce = ThreadPool::new(n_threads);
 
  // Setup routine. Start the threads in inverse order of 
  // how far they are away from the buffers.
  let rc_from_cmdr_c = rc_from_cmdr.clone();
  
  workforce.execute(move || {
    data_publisher(&tp_from_client,
                   to_local_file,
                   Some(&file_suffix),
                   test_eventids,
                   verbose); 
  });
  let tp_to_pub_ev   = tp_to_pub.clone();
  #[cfg(feature="tofcontrol")]
  let rc_to_runner_cal  = rc_to_runner.clone();
  #[cfg(feature="tofcontrol")]
  let tp_to_pub_cal  = tp_to_pub.clone();

  // then the runner. It does nothing, until we send a set
  // of RunParams
  workforce.execute(move || {
      runner(&rc_from_cmdr_c,
             None, 
             &bs_send,
             &dtf_to_evproc,
             &opmode_to_cache,
             show_progress);
  });

  workforce.execute(move || {
                    event_cache(tp_from_builder,
                                &tp_to_pub_ev,
                                &opmode_from_runner, 
                                wf_analysis,
                                cache_size)
  });
  let tp_to_cache_c        = tp_to_cache.clone();
  workforce.execute(move || {
                    event_processing(&bs_recv,
                                     &tp_to_cache,
                                     &dtf_from_runner,
                                     args.verbose);
  });
  

  // Respond to commands from the C&C server
  let rc_to_runner_c       = rc_to_runner.clone();
  let heartbeat_timeout_seconds : u32 = 10;
  workforce.execute(move || {
                    cmd_responder(cmd_server_ip,
                                  heartbeat_timeout_seconds,
                                  &rc_file_path,
                                  &rc_to_runner_c,
                                  &tp_to_cache_c)
  
  });
  
  // should this program end after it is done?
  let mut end = false;

  // We can only start a run here, if this is not
  // run through systemd
  if is_systemd_process() {
    println!("=> Executed by systemd. Waiting for input from C&C server!");
  } else {
    // if we are not as systemd, 
    // we are either in calibration mode
    // or have started manually either with 
    // a config or not
    if calibration {
      // we execute this routine first, then we 
      // can go into our loop listening for input
      #[cfg(feature="tofcontrol")]
      match args.command {
        // BEGIN Matching calibration command
        Command::Calibration(calib_cmd) => {
          match calib_cmd {
            CalibrationCmd::Default(default_opts) => {
              match rb_calibration(&rc_to_runner_cal, &tp_to_pub_cal) {
                Ok(_) => (),
                Err(err) => {
                  error!("Calibration failed! Error {err}!");
                }
              }
            },
            CalibrationCmd::Noi(noi_opts) => {
              match rb_noi_subcalibration(&rc_to_runner_cal, &tp_to_pub_cal) {
                Ok(_) => (),
                Err(err) => {
                  error!("Noi data taking failed! Error {err}!");
                }
              }
            },
            CalibrationCmd::Voltage(voltage_opts) => {
              let voltage_level = voltage_opts.voltage_level;
              match rb_noi_voltage_subcalibration(&rc_to_runner_cal, &tp_to_pub_cal, voltage_level) {
                Ok(_) => (),
                Err(err) => {
                  error!("Voltage calibration data taking failed! Error {err}!");
                }
              }
            },
            CalibrationCmd::Timing(timing_opts) => {
              let voltage_level = timing_opts.voltage_level;
              match rb_timing_subcalibration(&rc_to_runner_cal, &tp_to_pub_cal, voltage_level) {
                Ok(_) => (),
                Err(err) => {
                  error!("Timing calibration data taking failed! Error {err}!");
                }
              }
            }
          }
        },
        // END Matching calibration command
        // BEGIN Matching run command
        Command::Run(run_cmd) => {
          match run_cmd {
            liftof_lib::RunCmd::Start(run_start_opts) => {
              let run_type = run_start_opts.run_type;
              let rb_id = run_start_opts.rb_id;
              let event_no = run_start_opts.event_no;
              let time = run_start_opts.time;
              match rb_start_run(&rc_to_runner_cal, rc_config, run_type, rb_id, event_no, time) {
                Ok(_) => (),
                Err(err) => {
                  error!("Timing calibration data taking failed! Error {err}!");
                }
              }
            },
            liftof_lib::RunCmd::Stop(run_stop_opts) => {
              let rb_id = run_stop_opts.rb_id;
              match rb_stop_run(&rc_to_runner_cal, rb_id) {
                Ok(_) => (),
                Err(err) => {
                  error!("Timing calibration data taking failed! Error {err}!");
                }
              }
            }
          }
        },
        // END Matching run commmand
        _ => ()
      }
      end = true; // in case of we have done the calibration
                      // from shell. We finish after it is done.
    } else {
      // only do monitoring when we don't do a 
      // calibration
      let moni_interval_l1 = Duration::from_secs(args.moni_interval_l1);
      let moni_interval_l2 = Duration::from_secs(args.moni_interval_l2);
      workforce.execute(move || {
        monitoring(&tp_to_pub,
                   moni_interval_l1,
                   moni_interval_l2,
                   verbose);
      });
    } 
   

    if config_from_shell {
      if n_events_run > 0 {
        println!("=> We got a nevents argument from the commandline, requesting to run for {n_events_run}. This will OVERRIDE the setting in the run config file!");
        rc_config.nevents = n_events_run;
      }

      // if the runconfig does not have nevents different from 
      // 0, we will not send it right now. The commander will 
      // then take care of it and send it when it is time.
      if rc_config.nevents != 0 {
        println!("Got a number of events to be run > 0. Will stop the run after they are done. If you want to run continuously and listen for new runconfigs from the C&C server, set nevents to 0");
        end_after_run = true;
        if !rc_config.is_active {
          println!("=> The provided runconfig does not have the is_active field set to true. Won't start a run if that is what you were waiting for.");
        } else {
          println!("=> Waiting for threads to start..");
          thread::sleep(time::Duration::from_secs(5));
          println!("=> ..done");
        }
        match rc_to_runner.send(rc_config) {
          Err(err) => error!("Could not initialzie Run! Err {err}"),
          Ok(_)    => {
            if rc_config.is_active {
              println!("=> Runner configured! Attempting to start.");
            } else {
              println!("=> Stopping run..")
            }
          }
        }
      }
    } // end if config from shell
  } // end if not systemd process
  
  // Currently, the main thread just listens for SIGTERM and SIGINT.
  // We could give it more to do and save one of the other threads.
  // Probably, the functionality of the control thread would be 
  // a good choice
  let mut signals = Signals::new(&[SIGTERM, SIGINT]).expect("Unknown signals");



  // Wait until all threads are set up
  thread::sleep(5*one_sec);
  loop {
    thread::sleep(1*one_sec);
    for signal in signals.pending() {
      match signal as c_int {
        SIGTERM => {
          println!("SIGTERM received");
          end = true;
        }
        SIGINT  => {
          println!("SIGINT received");
          end = true;
        }
        _       => ()
      }
    }

    match get_triggers_enabled() {
      Err(err) => error!("Can not read trigger enabled register! Error {err}"),
      Ok(enabled) => {
        //println!("Current trigger enabled status {}. WIll end after a run {}", enabled, end_after_run);
        if !enabled && end_after_run {
          end = true;
        }
      }
    }

    if end {
      println!("=> Terminating program....waiting 10 seconds till the threads are finished.");
      // we simply generate a new run config and let the runner 
      // finish and clean up everything
      let mut rc_terminate = RunConfig::new();
      rc_terminate.is_active = false;
      match rc_to_runner.send(rc_terminate) {
        Err(err) => {
          error!("We were unable to terminate the run! Error {err}. However, we will end leaving the board in an uknown state...");
        }
        Ok(_) => ()
      }
      thread::sleep(10*one_sec);
      println!("=> Terminated. So long and thanks for all the \u{1F41F}");
      exit(0);
    }
  } // end loop
} // end main

