//! # Radoutboard software for the GAPS experiment, TOF system
//! 
//! This software shall help with data acquisition and commandeering 
//! of the readoutboards (RB) used in the tof system of the GAPS 
//! science experiment.
//!
//! Standalone, statically linked binary to be either run manually 
//! or to be managed by systemd

//use std::collections::HashMap;
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
use signal_hook::consts::signal::{SIGTERM, SIGINT};

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
use local_ip_address::local_ip;
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

use liftof_lib::{
    LIFTOF_LOGO_SHOW,
    color_log,
    RunStatistics,
};

use liftof_rb::threads::{
    runner,
    //experimental_runner,
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
  /// A json run config file with a RunConfiguration. This option is essential if 
  /// this program is run manually without systemd and not controlled by a central server. 
  /// If in this configuration one wants to take data, ONE HAS TO SUPPLY A RUNCONFIG!
  #[arg(short, long)]
  run_config: Option<std::path::PathBuf>,
  /// Listen to remote input from the TOF computer at 
  /// the expected IP address
  #[arg(short, long, default_value_t = false)]
  listen: bool,
  /// Show progress bars to indicate buffer fill values and number of acquired events
  #[arg(long, default_value_t = false)]
  show_progress: bool,
  /// Print out (even more) debugging information 
  #[arg(long, default_value_t = false)]
  verbose : bool,
  /// Write the readoutboard binary data ('.robin') to the board itself
  #[arg(long, default_value_t = false)]
  to_local_file : bool,
  /// Take data for calibration. This comprises tcal, vcal and 
  /// no input data
  #[arg(long, default_value_t = false)]
  calibration : bool,
  ///// CnC server IP we should be listening to
  //#[arg(long, default_value_t = "10.0.1.1")]
  //cmd_server_ip : &'static str,
  /// Take some events with the poisson trigger and 
  /// check the event ids for duplicates or missing ids
  #[arg(long, default_value_t = false)]
  test_eventids: bool,
  /// If there is an issue with the events (if known)
  /// don't send them. This can not work when the 
  /// RB is in TofMode::RBHighThroughput
  #[arg(long, default_value_t = false)]
  only_perfect_events: bool,
  /// Calculate the crc32 checksum per channel and set 
  /// event status flag
  #[arg(long, default_value_t = false)]
  calc_crc32: bool,
}

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

  let program_start_time       = Instant::now();

  let args = Args::parse();                   
  let verbose                  = args.verbose;
  let listen                   = args.listen;
  let show_progress            = args.show_progress;
  let calibration              = args.calibration;
  let mut to_local_file        = args.to_local_file;
  let run_config               = args.run_config;
  let test_eventids            = args.test_eventids;
  let calc_crc32               = args.calc_crc32;
  let only_perfect_events      = args.only_perfect_events;

  //FIMXE - this needs to become part of clap
  let cmd_server_ip = String::from("10.0.1.1");
  //let cmd_server_ip     = args.cmd_server_ip;  
  let this_board_ip = local_ip().expect("Unable to obtainl local board IP. Something is messed up!");

  // get board info 
  let rb_info = RBInfo::new();
  // check if it is sane. If we are not able to 
  // get the board id, we might as well panic and restart.
  if rb_info.board_id == u8::MAX {
    error!("Board ID field has been set to error state of {}", rb_info.board_id);
    panic!("Unable to obtain board id! This is a CRITICAL error! Abort!");
  }
  let ltb_connected = rb_info.sub_board == 1;
  let pb_connected  = rb_info.sub_board == 2;
  // General parameters, readout board id,, 
  // ip to tof computer
  let rb_id = rb_info.board_id;
  let dna   = get_device_dna().expect("Unable to obtain device DNA!"); 

  // welcome banner!
  println!("{}", LIFTOF_LOGO_SHOW);
  println!(" ** Welcome to liftof-rb \u{1F680} \u{1F388} *****");
  println!(" .. liftof is a software suite for the time-of-flight detector ");
  println!(" .. for the GAPS experiment \u{1F496}");
  println!(" .. this client can be run standalone or connect to liftof-cc" );
  println!(" .. or liftof-tui for an interactive experience" );
  println!(" .. this client will be publishing TofPackets on the bound port!");
  println!("-----------------------------------------------");
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
  println!("-----------------------------------------------");
  println!(" => We will BIND this port to the local ip address at {}", this_board_ip);
  println!(" => -- -- PORT {} (0MQ PUB) to publish our data", DATAPORT);
  println!(" => We will CONNECT to the following port on the C&C server at address: {}", cmd_server_ip);
  println!(" => -- -- PORT {} (0MQ SUB) where we will be listening for commands", DATAPORT);
  println!("-----------------------------------------------");
  
  if test_eventids {
    warn!("Testing mode! Only for debugging!");
  }

  let mut rc_config     = RunConfig::new();
  let mut rc_file_path  = std::path::PathBuf::new();
  let mut end_after_run = false;
  let run_stat          = Arc::new(Mutex::new(RunStatistics::new()));
   
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
        // FIXME - something happened
        //println!("=> We did not get a runconfig with the -r <RUNCONFIG> commandline switch! Currently we are just listening for input on the socket. This is the desired behavior, if run by systemd. If you want to take data in standalone mode, either send a runconfig to the socket or hit CTRL+C and start the program again, this time suppling the -r <RUNCONFIG> flag or in case you want to calibrate the board, use the --calibration flag.");
      }
      config_from_shell = false;
    },
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
  let one_sec     = Duration::from_secs(1);  

  //// FIXME - this will come from future runconfig
  let rb_mon_interv   = 5.0f32; 
  let mut pb_mon_every_x  = 2.0f32;
  let mut pa_mon_every_x  = 1.0f32; 
  let mut ltb_mon_every_x = 2.0f32;
  
  // for now just set the intervals to inf, 
  // better would be to switch the whole thing
  // off.
  if !pb_connected {
    pb_mon_every_x = f32::MAX;
    pa_mon_every_x = f32::MAX;
  }
  if !ltb_connected {
    ltb_mon_every_x = f32::MAX;
  }
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
  let _data_pub_thread = thread::Builder::new()
         .name("data-publisher".into())
         .spawn(move || {
            data_publisher(&tp_from_client,
                           to_local_file,
                           Some(&file_suffix),
                           test_eventids,
                           verbose,
                           ctrl_cl) 
          })
         .expect("Failed to spawn data-publsher thread!");
  
  let tp_to_pub_ev   = tp_to_pub.clone();
  let tp_to_pub_cal  = tp_to_pub.clone();


  // then the runner. It does nothing, until we send a set
  // of RunParams
  let tp_to_cache_c        = tp_to_cache.clone();
  let run_control          = thread_control.clone();
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
  //workforce.execute(move || {
  //    //experimental_runner(&rc_from_cmdr_c,
  //    //                    None, 
  //    //                    //&bs_send,
  //    //                    &tp_to_cache,
  //    //                    &dtf_to_evproc,
  //    //                    &opmode_to_cache,
  //    //                    show_progress);
  //    runner(&rc_from_cmdr_c,
  //           None, 
  //           &bs_send,
  //           &dtf_to_evproc,
  //           &opmode_to_cache,
  //           show_progress);
  //});
    let proc_control    = thread_control.clone();
    let ev_stats        = run_stat.clone();
    let _ev_proc_thread = thread::Builder::new()
           .name("event-processing".into())
           .spawn(move || {
                  event_processing(
                                   rb_id,
                                   &tp_from_builder,
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
  

  // Respond to commands from the C&C server
  // This obviously requires that we are 
  // listening, so this needs the --listen 
  // flag
  if listen {
    let cmd_control      = thread_control.clone(); 
    let rc_to_runner_c   = rc_to_runner.clone();
    let _cmd_resp_thread = thread::Builder::new()
           .name("cmd-responder".into())
           .spawn(move || {
              cmd_responder(cmd_server_ip,
                            &rc_file_path,
                            &rc_to_runner_c,
                            &tp_to_cache_c,
                            cmd_control)
            })
           .expect("Failed to spawn cmd_responder thread!");
           //workforce.execute(move || {
           //                  cmd_responder(cmd_server_ip,
           //                                &rc_file_path,
           //                                &rc_to_runner_c,
           //                                &tp_to_cache_c)
           //
           //});
  }

  // should this program end after it is done?
  let mut end = false;
  
  let mut do_monitoring = true;
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
      match rb_calibration(&rc_to_runner, &tp_to_pub_cal) {
        Ok(_) => (),
        Err(err) => {
          error!("Calibration failed! Error {err}!");
        }
      }
      do_monitoring = false;
      end = true; // in case of we have done the calibration
                  // from shell. We finish after it is done.
    } else {
      // only do monitoring when we don't do a 
      // calibration
      do_monitoring = true;
    } 
    if config_from_shell {

      // if the runconfig does not have nevents different from 
      // 0, we will not send it right now. The commander will 
      // then take care of it and send it when it is time.
      if rc_config.nevents != 0 || rc_config.nseconds != 0 {
        println!("=> The runconfig request to take {} events or to run for {} seconds!", rc_config.nevents, rc_config.nseconds);
        println!("=> The run will be stopped when it is finished!");
        println!("=> {}", String::from("!If that is not what you want, check out the --listen flag!").green());
        end_after_run = true;
        if !rc_config.is_active {
          println!("=> The provided runconfig does not have the is_active field set to true. Won't start a run if that is what you were waiting for.");
        } 
        if !listen {
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
      }
    } // end if config from shell
  } // end if not systemd process
  if do_monitoring {
    // only do monitoring when we don't do a 
    // calibration
    let moni_ctrl          = thread_control.clone();
    let _monitoring_thread = thread::Builder::new()
           .name("rb-monitoring".into())
           .spawn(move || {
              monitoring(rb_id,    // board id
                         &tp_to_pub,
                         rb_mon_interv,  
                         pa_mon_every_x, 
                         pb_mon_every_x, 
                         ltb_mon_every_x, 
                         //verbose,
                         true,
                         moni_ctrl); 
            })
           .expect("Failed to spawn rb-monitoring thread!");
  }
  if !calibration {
    println!("=> Waiting for threads to start..");
    thread::sleep(Duration::from_secs(10));
    println!("=> ..done");
  }
  // Currently, the main thread just listens for SIGTERM and SIGINT.
  // We could give it more to do and save one of the other threads.
  // Probably, the functionality of the control thread would be 
  // a good choice

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
          debug!("Board is triggering!");
        } else {
          debug!("No triggers active!");
        }
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
      // send the kill signal to all threads
      match thread_control.lock() {
        Ok(mut tc) => {
          tc.stop_flag = true;
        },
        Err(err) => {
          trace!("Can't acquire lock! {err}");
        },
      }
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

