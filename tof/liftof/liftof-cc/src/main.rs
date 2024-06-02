//! LIFTOF-CC - Main C&C (command and control) server application for 
//! tof datataking and control.
//!
//! This is meant to be run as a systemd service on the main tof computer.
//!
//!

#[macro_use] extern crate log;
extern crate env_logger;
extern crate clap;
extern crate tof_dataclasses;
extern crate crossbeam_channel;
extern crate colored;
extern crate signal_hook;

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
    //Path,
    PathBuf,
};
use std::os::raw::c_int;

use signal_hook::iterator::Signals;

use clap::{
    arg,
    command,
    Parser
};

use colored::Colorize;

use crossbeam_channel::{
    Sender,
    Receiver,
    unbounded,
};

//use colored::Colorize;
extern crate indicatif;
use indicatif::{
    ProgressBar,
    ProgressStyle,
};

use tof_dataclasses::events::{
    MasterTriggerEvent,
    RBEvent
};

use tof_dataclasses::threading::{
    ThreadControl,
};
use tof_dataclasses::serialization::{
    Serialization,
    Packable
};

use tof_dataclasses::packets::TofPacket;
use tof_dataclasses::database::{
    connect_to_db,
    get_linkid_rbid_map,
    ReadoutBoard,
};

use tof_dataclasses::constants::PAD_CMD_32BIT;
use tof_dataclasses::commands::{
    TofCommand,
    //TofCommandCode,
    TofResponse,
};

use liftof_lib::{
    signal_handler,
    init_env_logger,
    //color_log,
    LIFTOF_LOGO_SHOW,
    master_trigger,
    LiftofSettings,
    CommandCC,
};

use liftof_lib::constants::{
    DEFAULT_CALIB_VOLTAGE,
    DEFAULT_RB_ID,
    DEFAULT_CALIB_EXTRA
};

use liftof_cc::threads::{
    event_builder,
    //flight_cpu_listener,
    command_dispatcher,
    global_data_sink,
    monitor_cpu,
    readoutboard_communicator
};

/*************************************/



/// Command line arguments for calling 
/// liftof-cc directly from the command line
#[derive(Debug, Parser, PartialEq)]
pub enum CommandLineCommand {
  /// Listen for flight CPU commands.
  Listen,
  /// Staging mode - work through all .toml files
  /// in the staging area
  Staging,
  /// Ping a TOF sub-system.
  Ping,
  ///// Monitor a TOF sub-system.
  //Moni(MoniCmd),
  ///// Restart RB systemd
  //SystemdReboot(SystemdRebootCmd),
  /// Power control of TOF sub-systems.
  /// Remotely trigger the readoutboards to run the calibration routines (tcal, vcal).
  Calibration,
  /// Start/stop data taking run.
  Run
}

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
  run_id      : usize,
  /// More detailed output for debugging
  #[arg(short, long, default_value_t = false)]
  verbose     : bool,
  /// Configuration of liftof-cc. Configure analysis engine,
  /// event builder and general settings.
  #[arg(short, long)]
  config      : Option<String>,
  /// List of possible commands
  #[command(subcommand)]
  command     : CommandCC,
}

/*************************************/

/// Deal with the "result" of the command
/// inquiry
//use liftof_lib::{
//    StartRunOpts,
//    DefaultOpts,
//};

/// Little helper, just makes sure that all the 
/// channels are of same type
fn init_channels<T>() -> (Sender<T>, Receiver<T>) {
  let channels : (Sender<T>, Receiver<T>) = unbounded(); 
  channels
}

/*************************************/

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
  // global thread control
  let thread_control = Arc::new(Mutex::new(ThreadControl::new()));
  let one_second = time::Duration::from_millis(1000);

  // deal with command line arguments
  let config          : LiftofSettings;
  let nboards         : usize;
  let args              = LiftofCCArgs::parse();
  let verbose           = args.verbose;
  
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
  
  //println!("=> Using the following config as parsed from the config file:\n{}", config);

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
  let db_path               = config.db_path.clone();
  let cpu_moni_interval     = config.cpu_moni_interval_sec;
  //let flight_address        = config.fc_pub_address.clone();
  //let flight_sub_address    = config.fc_sub_address.clone();
  let cmd_dispatch_settings = config.cmd_dispatcher_settings.clone();
  let mtb_settings          = config.mtb_settings.clone();
  let mut gds_settings      = config.data_publisher_settings.clone();
  let run_analysis_engine   = config.run_analysis_engine;
  
  let mut conn              = connect_to_db(db_path).expect("Unable to establish a connection to the DB! CHeck db_path in the liftof settings (.toml) file!");
  // if this call does not go through, we might as well fail early.
  let mut rb_list           = ReadoutBoard::all(&mut conn).expect("Unable to retrieve RB information! Unable to continue, check db_path in the liftof settings (.toml) file and DB integrity!");
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
    debug!("     -{}", rb);
    if verbose {
      //println!("{}", rb);
    }
  }
  let mtb_link_id_map = get_linkid_rbid_map(&rb_list);
  
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

  /*******************************************************
   * Channels (crossbeam, unbounded) for inter-thread
   * communications.
   *
   * FIXME - do we need to use bounded channels
   * just in case?
   *
   */ 

  // all threads who send TofPackets to the global data sink, can clone this receiver
  let (tp_to_sink, tp_from_threads)   = init_channels::<TofPacket>();

  // master thread -> event builder MasterTriggerEvent transmission
  let (master_ev_send, master_ev_rec) = init_channels::<MasterTriggerEvent>(); 
  
  // readout boards -> event builder RBEvent transmission 
  let (ev_to_builder, ev_from_rb)     = init_channels::<RBEvent>();
  let (ack_to_cmd_disp, ack_from_rb)  = init_channels::<TofResponse>();   

  //let one_minute = time::Duration::from_millis(60000);

  // set up signal handline
  //let mut signals = Signals::new(&[SIGTERM, SIGINT]).expect("Unknown signals");

  // no cpu monitoring for cmdline calibration tasks
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
            false) 
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
      global_data_sink(&tp_from_threads,
                       write_stream,
                       runid,
                       &gds_settings,
                       false,
                       thread_control_gds);
    })
    .expect("Failed to spawn data-sink thread!");
  println!("==> data sink thread started!");
  let thread_control_sh = thread_control.clone();
  let _signal_handler_thread = thread::Builder::new()
    .name("signal_handler".into())
    .spawn(move || {
      signal_handler(
        thread_control_sh) 
      })
    .expect("Failed to spawn signal-handler thread!");
  println!("==> signal handler thread started!");

  println!("==> Starting event builder and master trigger threads...");
  //let db_path_string    = config.db_path.clone();
  let evb_settings      = config.event_builder_settings.clone();
  let thread_control_eb = thread_control.clone();
  let tp_to_sink_c      = tp_to_sink.clone();
  let _evb_thread = thread::Builder::new()
    .name("event-builder".into())
    .spawn(move || {
                    event_builder(&master_ev_rec,
                                  &ev_from_rb,
                                  &tp_to_sink_c,
                                  runid as u32,
                                  //db_path_string,
                                  mtb_link_id_map,
                                  evb_settings,
                                  thread_control_eb);
     })
    .expect("Failed to spawn event-builder thread!");
  // master trigger
  //let thread_control_mt = thread_control.clone();
  let mtb_moni_sender = tp_to_sink.clone(); 
  let thread_control_mt = thread_control.clone();
  let _mtb_thread = thread::Builder::new()
    .name("master-trigger".into())
    .spawn(move || {
                    master_trigger(mtb_address, 
                                   &master_ev_send,
                                   &mtb_moni_sender,
                                   mtb_settings,
                                   thread_control_mt,
                                   // verbosity is currently too much 
                                   // output
                                   verbose);
    })
  .expect("Failed to spawn master-trigger thread!");
  
  thread::sleep(one_second);
  println!("==> Will now start rb threads..");
  for n in 0..nboards {
    let mut this_rb           = rb_list[n].clone();
    let this_tp_to_sink_clone = tp_to_sink.clone();
    this_rb.calib_file_path   = calib_file_path.clone();
    match this_rb.load_latest_calibration() {
      Err(err) => panic!("Unable to load calibration for RB {}! {}", this_rb.rb_id, err),
      Ok(_)    => ()
    }
    println!("==> Starting RB thread for {}", this_rb.rb_id);
    let ev_to_builder_c = ev_to_builder.clone();
    let thread_name     = format!("rb-comms-{}", this_rb.rb_id);
    let settings        = config.analysis_engine_settings.clone();
    let ack_sender      = ack_to_cmd_disp.clone();
    let tc_rb_comm      = thread_control.clone();
    let _rb_comm_thread = thread::Builder::new()
      .name(thread_name)
      .spawn(move || {
        readoutboard_communicator(ev_to_builder_c,
                                  this_tp_to_sink_clone,
                                  this_rb,
                                  false,
                                  run_analysis_engine,
                                  settings,
                                  ack_sender,
                                  tc_rb_comm);
      })
      .expect("Failed to spawn readoutboard-communicator thread!");
  } // end for loop over nboards
  println!("==> All RB threads started!");
  println!("==> All threads initialized!");
  
  // Now we are ready. Let's decide what to do!
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
 
  //----------------------------------------------------
  //  Now we have a bunch of scenarios, depending on the 
  //  input. Most of this might go away, but we keep it 
  //  for now.
  // 
  //  1) If listening - we start the event builder and 
  //     master trigger and cpu moni threads as 
  //     well as the command dispatcher and continue 
  //     to the main program loop
  // 
  //  2) Staging. This requires we load ANOTHER configuration
  //     from the staging directory and work through them. 
  //     We do have to kill/restart threads with updated settings.
  //     TODO.
  //     FIXME: When we are in staging mode, do we want the cmd 
  //     dispatcher?
  //  3) Run - we just immediatly start a run.
  // 

  // possible progress bar
  let bar_template : &str = "[{elapsed_precise}] {prefix} {msg} {spinner} {bar:60.blue/grey} {pos:>7}/{len:7}";
  let bar_style  = ProgressStyle::with_template(bar_template).expect("Unable to set progressbar style!");
  let mut bar    = ProgressBar::hidden();

  // default  behavriour is to stop
  // when we are done
  let mut dont_stop = false;
  match args.command {
    CommandCC::Listen => {
      dont_stop = true;
      // start command dispatcher thread
      let tc = thread_control.clone();
      let ts = tp_to_sink.clone();
      let _cmd_dispatcher = thread::Builder::new()
        .name("command-dispatcher".into())
        .spawn(move || {
          command_dispatcher(
            cmd_dispatch_settings,
            tc,
            ts,
            ack_from_rb
          )
        })
      .expect("Failed to spawn cpu-monitoring thread!");
    },
    CommandCC::Calibration => {
      let voltage_level = DEFAULT_CALIB_VOLTAGE;
      let rb_id         = DEFAULT_RB_ID;
      let extra         = DEFAULT_CALIB_EXTRA;
      println!("=> Received calibration default command! Will init calibration run of all RBs...");
      let cmd_payload: u32
        = (voltage_level as u32) << 16 | (rb_id as u32) << 8 | (extra as u32);
      let default_calib = TofCommand::DefaultCalibration(cmd_payload);
      let tp = default_calib.pack();
      let mut payload  = String::from("BRCT").into_bytes();
      payload.append(&mut tp.to_bytestream());
      
      // open 0MQ socket here
      let ctx = zmq::Context::new();
      let cmd_sender  = ctx.socket(zmq::PUB).expect("Unable to create 0MQ PUB socket!");
      let cc_pub_addr = config.cmd_dispatcher_settings.cc_server_address.clone();
      cmd_sender.bind(&cc_pub_addr).expect("Unable to bind to (PUB) socket!");
      println!("=> Give the RBs a chance to connect and wait a bit..");
      thread::sleep(10*one_second);

      match cmd_sender.send(&payload, 0) {
        Err(err) => {
          error!("Unable to send command, error{err}");
          exit(1);
        },
        Ok(_) => {
          println!("=> Calibration  initialized!");
        }
      }
      println!("=> .. now we need to wait until the calibration is finished!");
      let bar_label  = String::from("Acquiring RB calibration data");

      bar = ProgressBar::new(rb_list.len() as u64); 
      bar.set_position(0);
      bar.set_message (bar_label);
      bar.set_prefix  ("\u{2699}\u{1F4D0}");
      bar.set_style   (bar_style);
      // if that is successful, we need to wait
      match thread_control.lock() {
        Ok(mut tc) => {
          // deactivate the master trigger thread
          tc.thread_master_trg_active =false;
          tc.calibration_active = true;
        },
        Err(err) => {
          error!("Can't acquire lock for ThreadControl! Unable to set calibration mode! {err}");
        },
      }
    },
    CommandCC::Staging => {
      error!("Staging sequence not implemented!");
    }
    CommandCC::Run => {
      // in this scenario, we want to end
      // after we are done
      dont_stop = false;
      // technically, it is run_typ, rb_id, event number
      // all to the max means run start for all
      // We don't need this - just need to make sure it gets broadcasted
      let cmd_payload: u32 =  PAD_CMD_32BIT | (255u32) << 16 | (255u32) << 8 | (255u32);
      let cmd          = TofCommand::DataRunStart(cmd_payload);
      let packet       = cmd.pack();
      let mut payload  = String::from("BRCT").into_bytes();
      payload.append(&mut packet.to_bytestream());
      
      // open 0MQ socket here
      let ctx = zmq::Context::new();
      let cmd_sender  = ctx.socket(zmq::PUB).expect("Unable to create 0MQ PUB socket!");
      let cc_pub_addr = config.cmd_dispatcher_settings.cc_server_address.clone();
      cmd_sender.bind(&cc_pub_addr).expect("Unable to bind to (PUB) socket!");
      // after we opened the socket, give the RBs a chance to connect
      println!("=> Give the RBs a chance to connect and wait a bit..");
      thread::sleep(10*one_second);

      println!("=> Initializing Run Start!");
      match cmd_sender.send(&payload, 0) {
        Err(err) => {
          error!("Unable to send command, error{err}");
        },
        Ok(_) => {
          debug!("We sent {:?}", payload);
        }
      }
      let run_start_timeout  = Instant::now();
      // let's wait 20 seconds here
      let mut n_rb_ack_rcved = 0;
      while run_start_timeout.elapsed().as_secs() < 20 {
        //println!("{}", run_start_timeout.elapsed().as_secs());
        match ack_from_rb.try_recv() {
          Err(_) => {
            continue;
          }
          Ok(_ack_pack) => {
            //FIXME - do something with it
            n_rb_ack_rcved += 1;
          }
        }
        if n_rb_ack_rcved == rb_list.len() {
          break; 
        }
      }
      println!("Run initialized!");
      bar = ProgressBar::new_spinner();
      bar.enable_steady_tick(Duration::from_millis(500));
      bar.set_message(".. acquiring data ..");
    }
    _ => {
      panic!("Unable to execute request for this command!");
    }
  }

  //---------------------------------------------------------
  // 
  // Program main loop. Remember, most work is done in the 
  // individual threads. Here we have to check for ongoing
  // calibrations
  // 


  // a counter for the number 
  // or RBCalibrations we have
  // received
  let mut end_program   = false;
  // a counter for number of RBCalibrations
  // received to understand when a calibration 
  // procedure is finished
  let mut cali_received  = 0u64;
  loop {
    // take out the heat a bit
    thread::sleep(1*one_second);

    // check pending signals and handle
    // SIGTERM and SIGINT
    //for signal in signals.pending() {
    //  match signal as c_int {
    //    SIGTERM => {
    //      println!("=> {}", String::from("SIGINT received. Maybe Ctrl+C has been pressed!").red().bold());
    //      end_program = true;
    //    } 
    //    SIGINT => {
    //      println!("=> {}", String::from("SIGTERM received").red().bold());
    //      end_program = true;
    //    }
    //    _ => {
    //      error!("Received signal, but I don't have instructions what to do about it!");
    //    }
    //  }
    //}
    if end_program {
      println!("=> Shutting down threads...");
      match thread_control.lock() {
        Ok(mut tc) => {
          tc.stop_flag = true;
        },
        Err(err) => {
          error!("Can't acquire lock for ThreadControl! Unable to set calibration mode! {err}");
        },
      }
      // wait actually until all threads have been finished
      let timeout = Instant::now();
      loop {
        match thread_control.lock() {
          Ok(mut tc) => {
            tc.stop_flag = true;
            // each thread will report here if
            // it is done
            if !tc.thread_cmd_dispatch_active 
            && !tc.thread_data_sink_active
            && !tc.thread_event_bldr_active 
            && !tc.thread_master_trg_active {
              break;
            }
          },
          Err(err) => {
            error!("Can't acquire lock for ThreadControl! Unable to set calibration mode! {err}");
          },
        }
        // in any case, break after timeout
        if timeout.elapsed() > 5*one_second {
          break;
        }
      }
      println!(">> So long and thanks for all the \u{1F41F} <<"); 
      exit(0);
    }

    // check thread control - this is useful 
    // for everything

    match thread_control.try_lock() {
      Ok(mut tc) => {
        if tc.stop_flag {
          end_program = true;
        }
        if tc.calibration_active {
          for rbid in &rb_list {
            // the global data sink sets these flags
            if tc.finished_calibrations[&rbid.rb_id] {
              cali_received += 1;
              bar.set_position(cali_received);
            }
          }
          // FIXME - this or a timer
          if cali_received as usize == rb_list.len() {
            cali_received = 0;
            // if we want to redo a calibration, 
            // somebody else has to set this 
            // flag again.
            tc.calibration_active = false;
            // reset the counters
            for rbid in &rb_list {
              *tc.finished_calibrations.get_mut(&rbid.rb_id).unwrap() = false; 
            }
            bar.finish_with_message("Done");
            if !dont_stop {
              end_program = true;
            }
          }
        }
      }
      Err(err) => {
        error!("Can't acquire lock for ThreadControl at this time! Unable to set calibration mode! {err}");
      }
    }
    // in case the runtime seconds are over, we can end the program
    if program_start.elapsed().as_secs_f64() > runtime_nseconds as f64 && !dont_stop {
      println!("=> Runtime seconds of {} have expired!", runtime_nseconds);
      println!("=> Ending program. If you don't want that behaviour, change the confifguration file.");
      end_program = true;
    }
  }
}
