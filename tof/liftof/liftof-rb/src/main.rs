//! # Radoutboard software for the GAPS experiment, TOF system
//! 
//! This software shall help with data acquisition and commandeering 
//! of the readoutboards (RB) used in the tof system of the GAPS 
//! science experiment.
//!
//! Standalone, statically linked binary to be either run manually 
//! or to be managed by systemd

//use std::collections::HashMap;
//use std::path::PathBuf;
use std::os::raw::c_int;
use std::process::exit;
use std::{
    thread,
};
use std::sync::{
    Arc,
    Mutex,
};
use std::time::{
    Duration,
    Instant,
};
use std::io::Write;

extern crate crossbeam_channel;
extern crate signal_hook;
extern crate env_logger;

use signal_hook::iterator::Signals;
use signal_hook::consts::signal::{
    SIGTERM,
    SIGINT
};

#[macro_use] extern crate log;

extern crate clap;
use clap::{
    arg,
    command,
    Parser
};

// FIXME - think about using 
// bounded channels to not 
// create a memory leak
use crossbeam_channel::{
    unbounded,
    //bounded,
    Sender,
    Receiver
};
use colored::Colorize;

// TOF specific crates
use tof_control::helper::rb_type::RBInfo;

use tof_dataclasses::threading::{
    ThreadControl,
};
use tof_dataclasses::packets::TofPacket;
use tof_dataclasses::commands::{
    //RBCommand,
    TofOperationMode
};

use tof_dataclasses::events::DataType;
use tof_dataclasses::run::RunConfig;
use tof_dataclasses::io::{
    get_califilename,
    get_runfilename
};

use liftof_lib::{
    LIFTOF_LOGO_SHOW,
    DATAPORT,
    //Command,
    CommandRB,
    color_log,
    RunStatistics,
    CalibrationCmd,
    LiftofSettings,
    init_env_logger,
};

use liftof_rb::threads::{
    runner,
    calibration, 
    cmd_responder,
    event_processing,
    monitoring,
    data_publisher
};

use liftof_rb::api::*;
use liftof_rb::control::*;

#[derive(Parser, Debug)]
#[command(author = "J.A.Stoessl", version, about, long_about = None)]
struct Args {
  /// Configuration of liftof-rb. Configure analysis engine,
  /// event builder and general settings. This is the same
  /// config file as for liftof-cc, however, here we will 
  /// only parse the relevant sections.
  #[arg(short, long)]
  config: Option<String>,
  /// If you want to run locally, use run start
  /// and set the time of the run with this
  /// parameter
  /// The default setting of 0 will run 
  /// forever
  #[arg(long, default_value_t = 0)]
  runtime_secs : u32,
  /// Show progress bars to indicate buffer fill values and number of acquired events
  #[arg(long, default_value_t = false)]
  show_progress: bool,
  /// Print out (even more) debugging information 
  #[arg(long, default_value_t = false)]
  verbose : bool,
  /// Write the readoutboard binary data ('.robin') to the board itself
  #[arg(long, default_value_t = false)]
  to_local_file : bool,
  /// Take some events with the poisson trigger and 
  /// check the event ids for duplicates or missing ids
  #[arg(long, default_value_t = false)]
  test_eventids: bool,
  /// List of possible commands
  #[command(subcommand)]
  command: CommandRB
}

/**********************************************************/

fn main() {
  //env_logger::builder()
  //  .format(|buf, record| {
  //  writeln!( buf, "[{level}][{module_path}:{line}] {args}",
  //    level = color_log(&record.level()),
  //    module_path = record.module_path().unwrap_or("<unknown>"),
  //    //target = record.target(),
  //    line = record.line().unwrap_or(0),
  //    args = record.args()
  //    )
  //  }).init();
  init_env_logger();
  let program_start_time       = Instant::now();
  
  // welcome banner!
  println!("{}", LIFTOF_LOGO_SHOW);
  println!(" ** Welcome to liftof-rb \u{1F680} \u{1F388} *****");
  println!(" .. liftof is a software suite for the time-of-flight detector ");
  println!(" .. for the GAPS experiment \u{1F496}");
  println!(" .. this client can be run standalone or connect to liftof-cc" );
  println!(" .. or liftof-tui for an interactive experience" );
  println!(" .. this client will be publishing TofPackets on the bound port!");
  println!("-----------------------------------------------");
  
  // parse the args here, so that we can ask for the version 
  // for deployment script
  let args                     = Args::parse();                   
  
  // get board info 
  let rb_info = RBInfo::new();
  // check if it is sane. If we are not able to 
  // get the board id, we might as well panic and restart.
  if rb_info.board_id == u8::MAX {
    error!("Board ID field has been set to error state of {}", rb_info.board_id);
    panic!("Unable to obtain board id! This is a CRITICAL error! Abort!");
  }
  // we just follow the convention here. This is the local address on the 
  // RB network
  let rb_id = rb_info.board_id;
  let ip_address = format!("tcp://10.0.1.1{:02}:{}", rb_id, DATAPORT);

  let ltb_connected = rb_info.sub_board == 1;
  let pb_connected  = rb_info.sub_board == 2;
  // General parameters, readout board id,, 
  // ip to tof computer
  let rb_id         = rb_info.board_id;
  let dna           = get_device_dna().expect("Unable to obtain device DNA!"); 
  
  println!(" .. RBInfo:");
  println!(" .. .. ReadoutBoard  ID {}", rb_id);
  println!(" .. .. ReadoutBoard DNA {}", dna);
  println!(" .. .. Current Rate     {} [Hz]", rb_info.trig_rate);
  println!(" .. Connected boards:");
  if ltb_connected {
    println!("..     LTB                     - {}", String::from("YES").green());
  } else {
    println!("..     LTB                     - {}", String::from("NO").red());
  }
  if pb_connected {
    println!("..     PB (including preamps) - {}", String::from("YES").green());
  } else {
    println!("..     PB (including preamps) - {}", String::from("NO").red());
  }
  
  let config                   : LiftofSettings;
  let mut listen               = false;
  // this will hold the result of "run start"
  // and trigger to immediatly start a run
  // we don't need anything for stop. If we issue
  // stop, we will just stop all the triggers and
  // exit immediatly. I don't think there is really
  // a useful scenario for that though. Maybe 
  // calming a board.
  let mut start_run_now        = false;

  let verbose                  = args.verbose;
  let show_progress            = args.show_progress;
  let to_local_file            = args.to_local_file;
  let test_eventids            = args.test_eventids;
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
  }
  let calc_crc32            = config.rb_settings.calc_crc32;
  let only_perfect_events   = config.rb_settings.only_perfect_events;
  let cmd_server_address    = config.cc_server_address.clone();
  let mut run_config        = config.rb_settings.get_runconfig();
  run_config.nseconds       = args.runtime_secs;
  
  // FIXME - instead of passing the run config around,
  // just offer it through a mutex
  //let mut global_run_config = Arc::new(Mutex::new(run_config));
  
  // monitoring settigns
  // these are only relevant if the 
  // board features these connections
  let mut pb_moni_every_x   = f32::MAX;
  let mut pa_moni_every_x   = f32::MAX;
  let mut ltb_moni_every_x  = f32::MAX;
  
  let rb_moni_interval      = config.rb_settings.rb_moni_interval; 
  if pb_connected {
    pb_moni_every_x         = config.rb_settings.pb_moni_every_x;
    pa_moni_every_x         = config.rb_settings.pa_moni_every_x; 
  }
  if ltb_connected {
    ltb_moni_every_x        = config.rb_settings.ltb_moni_every_x;
  }
  
  println!("-----------------------------------------------");
  println!(" => We will BIND 0MQ PUB to address/port at {}", ip_address);
  println!(" => We will CONNECT to the following port on the C&C server at address: {}", cmd_server_address);
  println!(" => -- -- PORT {} (0MQ SUB) where we will be listening for commands", DATAPORT);
  println!("-----------------------------------------------");

  // check if the board has received the correct link id from the mtb
  println!("=> Performing LTB LINK ID check!");
  match get_mtb_link_id() {
    Err(err) => error!("Unable to obtain MTB link id! {err}"),
    Ok(link_id) => {
      if link_id == rb_info.board_id as u32 {
        println!("=> We received the correct link id from the MTB!");
      } else {
        error!("Received unexpected MTB link ID {}!", link_id);
        error!("Incorrect link ID. This might hint to issues with the MTB mapping!");
        error!("******************************************************************");
      }
    }
  }
  
  if test_eventids {
    warn!("Testing mode! Only for debugging!");
  }

  // should the program terminate after a run
  // or calibration is done?
  // This affects only run start and calibration
  // commands
  let mut end_after_run = false;
  let mut calibration   = false;
  // first scan of commands. Decide if we want to listen,
  // run, or calibrate
  let args_commands = args.command.clone(); // we need it later 
                                            // again

  match args_commands {
    // Matching calibration command
    CommandRB::Calibration(_) => {
      calibration   = true;
      end_after_run = true;
    }
    CommandRB::Listen(_) => {
      listen = true;
    },
    // BEGIN Matching set command
    CommandRB::Set(set_cmd) => {
      match set_cmd {
        liftof_lib::SetCmd::LtbThreshold(lbt_threshold_opts) => {
          let ltb_id = lbt_threshold_opts.id;
          let threshold_name = lbt_threshold_opts.name;
          let threshold_level: u16 = lbt_threshold_opts.level;
          match send_ltb_threshold_set(ltb_id,
                                        threshold_name,
                                        threshold_level) {
            Ok(_) => {
              println!("=> send_ltb_threshold successful!");
            },
            Err(err) => {
              error!("Unable to set LTB thresholds! Error {err}!");
            }
          }
        },
        liftof_lib::SetCmd::PreampBias(preamp_bias_opts) => {
          let preamp_id = preamp_bias_opts.id;
          let preamp_bias = preamp_bias_opts.bias;
          match send_preamp_bias_set(preamp_id, 
                                      preamp_bias) {
            Ok(_) => {
              println!("=> send_preamp_bias successful!");
            },
            Err(err) => {
              error!("Unable to set preamp bias! Error {err}!");
            }
          }
        }
      }
      println!("=> Exiting program after values have been set!");
      exit(0);
    },
    CommandRB::Run(run_cmd) => {
      match run_cmd {
        liftof_lib::RunCmd::Start(run_start_opts) => {
          start_run_now = true;
          // for the default setting, actually we 
          // don't stop after a certain time, 
          // but just when we hit Ctrl+C
          end_after_run = run_config.nseconds > 0;
        },
        liftof_lib::RunCmd::Stop(run_stop_opts) => {
          match disable_trigger() {
            Err(err) => {
              error!("Unable to stop run! {err}");
              exit(1);
            }
            Ok(_) => {
              println!("Stopped triggers successfully! Exit!");
              exit(0);
            }
          }
        }
      }
    }, 
    _ => ()
  }
  let run_stat = Arc::new(Mutex::new(RunStatistics::new()));
  let output_fname : Option<String>;
  if calibration {
    output_fname = Some(get_califilename(rb_id, false));
    println!("===================================================================");
    println!("=> Readoutboard calibration! This will override ALL other settings!");
    println!("=> We are operating on local mode (not listeing). This will save   ");
    println!("=> to {}", output_fname.clone().unwrap());
    println!("===================================================================");
  } else {
    if to_local_file {
      output_fname = Some(get_runfilename(1,1,Some(rb_id)));
    } else {
      output_fname = None;
    }
  }
  
  //***************************************/
  // THREAD CONTROL SECTION - PART I      */
  //  - spawn all necessary threads       */
  //    for calibrations                  */
  //                                      */
  //***************************************/

  // some pre-defined time units for 
  // sleeping
  let one_sec     = Duration::from_secs(1);  

  // setting up inter-thread comms
  let thread_control : Arc<Mutex<ThreadControl>> = Arc::new(Mutex::new(ThreadControl::new())); 

  let (rc_to_runner, rc_from_cmdr)      : 
      (Sender<RunConfig>, Receiver<RunConfig>)                = unbounded();
  let (tp_to_pub, tp_from_client)        : 
      (Sender<TofPacket>, Receiver<TofPacket>)                = unbounded();
  let (tp_to_cache, tp_from_builder) : 
      (Sender<TofPacket>, Receiver<TofPacket>)                = unbounded();
  let (dtf_to_evproc, dtf_from_runner) :                
      (Sender<DataType>, Receiver<DataType>)                  = unbounded();
  
  let (opmode_to_cache, opmode_from_runner)     : 
      (Sender<TofOperationMode>, Receiver<TofOperationMode>)                = unbounded();
  let (bs_send, bs_recv)             : (Sender<Vec<u8>>, Receiver<Vec<u8>>) = unbounded(); 
  
  let mut signals = Signals::new(&[SIGTERM, SIGINT]).expect("Unknown signals");
  // Setup routine. Start the threads in inverse order of 
  // how far they are away from the buffers.
  // FIXME - reduce number of threads
  // E.g while the runner is sleeping, we could 
  // check for new commands and do monitoring
  // We currently have 5 threads + main thread
  // 1) Runner
  // 2) Event processing
  // 3) Data publisher
  // 4) Commander
  // 5) Monitoring
  let rc_from_cmdr_c = rc_from_cmdr.clone();
  let ctrl_cl        = thread_control.clone();
  let address        = ip_address.clone();
  let _data_pub_thread = thread::Builder::new()
    .name("data-publisher".into())
    .spawn(move || {
       data_publisher(&tp_from_client,
                      address,
                      output_fname,
                      test_eventids,
                      verbose,
                      ctrl_cl) 
     })
    .expect("Failed to spawn data-publsher thread!");
  
  let tp_to_pub_ev     = tp_to_pub.clone();
  let rc_to_runner_cal = rc_to_runner.clone();
  let tp_to_pub_cal    = tp_to_pub.clone();
  let tp_to_pub_cmd    = tp_to_pub.clone();

  // then the runner. It does nothing, until we send a set
  // of RunParams
  let tp_to_cache_c    = tp_to_cache.clone();
  let run_control      = thread_control.clone();
  let _runner_thread = thread::Builder::new()
    .name("runner".into())
    .spawn(move || {
           runner(&rc_from_cmdr_c,
                  &bs_send,
                  &dtf_to_evproc,
                  &opmode_to_cache,
                  show_progress,
                  run_control)
    })
    .expect("Failed to spawn runner thread!");
    
  let proc_control    = thread_control.clone();
  let ev_stats        = run_stat.clone();

  let _ev_proc_thread = thread::Builder::new()
    .name("event-processing".into())
    .spawn(move || {
           event_processing(
                            rb_id,
                            &bs_recv,
                            &opmode_from_runner, 
                            &tp_to_pub_ev,
                            &dtf_from_runner,
                            args.verbose,
                            calc_crc32,
                            proc_control,
                            ev_stats,
                            only_perfect_events)
    })
    .expect("Failed to spawn event_processing thread!");
 
  // Now all threads have been spawned which are needed for 
  // the calbration
  if calibration {
    // we execute this routine first, then we 
    // can go into our loop listening for input
    match args.command {
      // BEGIN Matching calibration command
      CommandRB::Calibration(calib_cmd) => {
        match calib_cmd {
          CalibrationCmd::Default(_default_opts) => {
            match rb_calibration(&rc_to_runner_cal, 
                                 &tp_to_pub_cal,
                                 ip_address) {
              Ok(_) => (),
              Err(err) => {
                error!("Calibration failed! Error {err}!");
              }
            }
          },
          CalibrationCmd::Noi(_noi_opts) => {
            match rb_noi_subcalibration(&rc_to_runner_cal, 
                        &tp_to_pub_cal) {
              Ok(_) => (),
              Err(err) => {
                error!("Noi data taking failed! Error {err}!");
              }
            }
          },
          CalibrationCmd::Voltage(voltage_opts) => {
            let voltage_level = voltage_opts.level;
            match rb_voltage_subcalibration(&rc_to_runner_cal, 
                            &tp_to_pub_cal,
                                            voltage_level) {
              Ok(_) => (),
              Err(err) => {
                error!("Voltage calibration data taking failed! Error {err}!");
              }
            }
          },
          CalibrationCmd::Timing(timing_opts) => {
            let voltage_level = timing_opts.level;
            match rb_timing_subcalibration(&rc_to_runner_cal, 
                            &tp_to_pub_cal,
                                            voltage_level) {
              Ok(_) => (),
              Err(err) => {
                error!("Timing calibration data taking failed! Error {err}!");
              }
            }
          }
        }
      },
      _ => ()
    }
    println!("=> Calibration complete! Terminating..");
    println!("=> Terminated. So long and thanks for all the \u{1F41F}");
    exit(0);
  }
  
  //***************************************/
  // THREAD CONTROL SECTION - PART II     */
  //  - spawn all necessary threads       */
  //    for physics runs, e.g.            */
  //    housekeeping                      */
  //                                      */
  //***************************************/
  
  /// spawn the monitoring thread. From here on, 
  /// we want monitoring for whatever we are doing.
  /// If rb_moni_interval is set to a negative value,
  /// we won't do it
  if rb_moni_interval > 0.0 {
    let moni_ctrl          = thread_control.clone();
    let _monitoring_thread = thread::Builder::new()
      .name("rb-monitoring".into())
      .spawn(move || {
         monitoring(rb_id,    // board id
                    &tp_to_pub,
                    rb_moni_interval,  
                    pa_moni_every_x, 
                    pb_moni_every_x, 
                    ltb_moni_every_x, 
                    //verbose,
                    verbose,
                    moni_ctrl); 
       })
      .expect("Failed to spawn rb-monitoring thread!");
  } else {
    warn!("RB moni interval < 0 ({}). Will NOT spawn monitoring thread!", rb_moni_interval);
  }

  // Respond to commands from the C&C server
  if listen {
    println!("=> We are listening, spawning cmd-responder thread!");
    let cmd_control      = thread_control.clone(); 
    let rc_to_runner_c   = rc_to_runner.clone();
    let address          = ip_address.clone();
    let _cmd_resp_thread = thread::Builder::new()
      .name("cmd-responder".into())
      .spawn(move || {
         cmd_responder(cmd_server_address,
                       &run_config,
                       &rc_to_runner_c,
                       &tp_to_pub_cmd,
                       address,
                       cmd_control)
       })
      .expect("Failed to spawn cmd_responder thread!");
  } 

  // in case we got the "run start command", now
  // it is the time
  if start_run_now {
    if run_config.nseconds == 0 {
      println!("=> Starting run NOW! Will run until Ctrl+C is hit!");
    } else {
      println!("=> Starting run NOW! Will run for {}", run_config.nseconds);
      println!("=> The program will exit when this time has passed!");
      println!("=> {}", String::from("!If that is not what you want, check out the listen command instead of run start!").green());
    }
    match rc_to_runner.send(run_config) {
      Err(err) => {
        error!("Could not initialzie Run! {err}");
        error!("That's it. Sorry...");
        println!("=> {}", String::from("Unsuccessful termination of the program!").red().bold());
        exit(1);
      }
      Ok(_)    => ()
    }
  }
  // trigger loop break?
  let mut end = false;
  // Currently, the main thread just listens for SIGTERM and SIGINT.
  // We could give it more to do and save one of the other threads.
  // Probably, the functionality of the control thread would be 
  // a good choice
  println!("=> Initializing loop..");
  thread::sleep(2*one_sec);

  loop {
    thread::sleep(1*one_sec);
    for signal in signals.pending() {
      match signal as c_int {
        SIGTERM => {
          println!("=> {}", String::from("SIGTERM received").red().bold());
          end = true;
        }
        SIGINT  => {
          println!("=> {}", String::from("SIGINT received").red().bold());
          end = true;
        }
        _       => {
          error!("Received signal, but I don't have instructions what to do about it!");
        }
      }
    }

    match get_triggers_enabled() {
      Err(err) => error!("Can not read trigger enabled register! Error {err}"),
      Ok(enabled) => {
        if enabled {
          trace!("Trigger enabled register is asserted!");
        } else {
          debug!("Trigger enabled register is NOT asserted!");
        }
        //println!("Current trigger enabled status {}. WIll end after a run {}", enabled, end_after_run);
        if !enabled && end_after_run {
          end = true;
        }
        // in case we have nseconds and the fixed rate trigger,
        // we have end_after_run == false 
        if program_start_time.elapsed().as_secs() > run_config.nseconds as u64 && run_config.nseconds > 0 {
          end = true;
        }
      }
    }

    if end {
      println!("=> Terminating program....sending termination signal to threads.");
      // we simply generate a new run config and let the runner 
      // finish and clean up everything
      let mut rc_terminate = RunConfig::new();
      rc_terminate.is_active = false;
      match rc_to_runner.send(rc_terminate) {
        Err(err) => {
          error!("We were unable to terminate the run! Error {err}. However, we will end leaving the board in an unknown state...");
        }
        Ok(_) => ()
      }
      // send the kill signal to all threads
      match thread_control.lock() {
        Ok(mut tc) => {
          tc.stop_flag = true;
        },
        Err(err) => {
          trace!("Can't acquire lock! {err}");
        },
      }
      thread::sleep(2*one_sec);
      if verbose {
        match run_stat.lock() {
          Err(err) => error!("Can't access run statistics! {err}"),
          Ok(stat) => {
            println!("== Run summary! = == == == == == == ==");
            println!("{}", stat);
            println!("-- -- -- -- -- -- -- -- -- -- -- -- --");
            println!("-- --> Elapsed seconds since prog start {}", program_start_time.elapsed().as_secs());
          }
        }
      }
      println!("=> Terminated. So long and thanks for all the \u{1F41F}");
      exit(0);
    }
  } // end loop
} // end main

