//! LIFTOF-CC - Main C&C (command and control) server application for 
//! tof datataking and control.
//!
//! This is meant to be run as a systemd service on the main tof computer.
//!
//!

#[macro_use] extern crate log;

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
use std::process::{
  Command,
  Child,
  exit
};

use std::{
  //fs,
  thread,
  time
};

use std::path::{
  //Path,
  PathBuf,
};

use clap::{
  arg,
  command,
  Parser
};

use crossbeam_channel::{
  Sender,
  Receiver,
  unbounded,
};

//use colored::Colorize;
use indicatif::{
  ProgressBar,
};

use tof_dataclasses::events::{
  MasterTriggerEvent,
  RBEvent
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
};

use liftof_lib::thread_control::ThreadControl;
use liftof_lib::constants::{
  DEFAULT_RB_ID,
};

use liftof_cc::{
  prepare_run,
  calibrate_tof,
  verification_run,
  restart_liftof_rb,
};

use liftof_cc::threads::{
  event_builder,
  command_dispatcher,
  global_data_sink,
  readoutboard_communicator
};

#[cfg(feature="tof-ctrl")]
use liftof_cc::threads::monitor_cpu;

/*************************************/

#[derive(Debug, Parser, PartialEq)]
pub enum CommandCC {
  /// Listen for flight CPU commands.
  Listen,
  /// Staging mode - work through all .toml files
  /// in the staging area
  Staging,
  /// Run full Readoutboard calibration.
  Calibration,
  /// Start/stop data taking run.
  Run
}

/*************************************/

#[derive(Parser, Debug)]
#[command(author = "J.A.Stoessl", version, about, long_about = None)]
#[command(propagate_version = true)]
struct LiftofCCArgs {
  /// Explicetly suppress writing to disk (e.g. for debugging)
  #[arg(long, default_value_t = false)]
  no_write_to_disk: bool,
  /// Define a run id for later identification
  /// If this is not given, we will check the 
  /// data path and assign the next folowing 
  /// id which has not been used as new run id
  /// If this argument is given, it *overrides* 
  /// this behaviour.
  #[arg(short, long)]
  run_id      : Option<u32>,
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
  println!("-----------------------------------------------\n\n");

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
  

  // program execution control
  let thread_control = Arc::new(Mutex::new(ThreadControl::new()));
  let mut end_program   = false;

  // there seems to be now way to create handles without thread
  //let mut evtbldr_handle   : thread::JoinHandle<_> = thread::spawn(||{});
  //let mut data_sink_handle : thread::JoinHandle<_> = thread::spawn(||{});
  //let mut mtb_handle       : thread::JoinHandle<_> = thread::spawn(||{});
  //let mut cmd_handle       : thread::JoinHandle<_> = thread::spawn(||{});
  //#[cfg(feature="tof-ctrl")]
  //let mut cpu_moni_handle  : thread::JoinHandle<_> = thread::spawn(||{});
  //let mut sig_handle       : thread::JoinHandle<_> = thread::spawn(||{});
  let mut rb_handles       = Vec::<thread::JoinHandle<_>>::new();

  let one_second = time::Duration::from_millis(1000);

  // deal with command line arguments
  let mut config      : LiftofSettings;
  let nboards         : usize;
  let args              = LiftofCCArgs::parse();
  let verbose           = args.verbose;
  let cfg_file_str   : String; 
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
  
  let mtb_address           = config.mtb_address.clone();
  info!("Will connect to the master trigger board at {}!", mtb_address);
 
  // FIXME
  let runid                 = args.run_id;
  let write_stream          = !args.no_write_to_disk;
  // clone the strings, so we can save the config later
  let mut write_stream_path = config.data_publisher_settings.data_dir.clone();
  let calib_file_path       = config.calibration_dir.clone();
  let runtime_nseconds      = config.runtime_sec;
  let db_path               = config.db_path.clone();
  #[cfg(feature="tof-ctrl")]
  let cpu_moni_interval     = config.cpu_moni_interval_sec;
  let cmd_dispatch_settings = config.cmd_dispatcher_settings.clone();
  let mtb_settings          = config.mtb_settings.clone();
  let mut gds_settings      = config.data_publisher_settings.clone();
  let run_analysis_engine   = config.run_analysis_engine;
  let pre_run_calibration   = config.pre_run_calibration; 
  let verification_rt_sec   = config.verification_runtime_sec;
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

  nboards = rb_list.len();
  println!("=> Will use {} readoutboards! Ignoring {:?} sicne they are mareked as 'ignore' in the config file!", rb_list.len(), rb_ignorelist );
  //debug!("--> Following RBs are expected:");
  // init thread control
  match thread_control.lock() {
    Ok(mut tc) => {
      for rb in &rb_list {
        tc.finished_calibrations.insert(rb.rb_id, false); 
        //debug!("     -{}", rb);
      }
      tc.n_rbs = rb_list.len() as u32;
      tc.thread_data_sink_active = true;
      tc.liftof_settings         = config.clone();
      //tc.rb_list                 = rb_list.clone(); 
    },
    Err(err) => {
      error!("Can't acquire lock for ThreadControl! Unable to set calibration mode! {err}");
    },
  }
  
  println!("=> Copying config to all RBs!");
  let mut children = Vec::<(u8,Child)>::new();
  for rb in &rb_list {
    // also populate the rb thread nandles
    rb_handles.push(thread::spawn(||{}));
    
    let rb_address = format!("tof-rb{:02}:config/liftof-config.toml", rb.rb_id);
    match Command::new("scp")
      .args([&cfg_file_str, &rb_address])
      .spawn() {
      Err(err) => {
        error!("Unable to spawn ssh process to copy config on RB {}! {}", rb.rb_id, err);
      }
      Ok(child) => {
        children.push((rb.rb_id,child));
      }
    }
  }
  let mut issues = Vec::<u8>::new();
  for rb_child in &mut children {
    match rb_child.1.wait() {
      Err(err) => {
        error!("Child process failed with stderr {:?}! {}", rb_child.1.stderr, err);
      }
      Ok(status) => {
        if status.success() {
          info!("Copied config to RB {} successfully!", rb_child.0);
        } else {
          error!("Copy config to RB {} failed with exit code {:?}!", rb_child.0, status.code());
          issues.push(rb_child.0);
        }
      }
    }
  }
  if issues.len() == 0 {
    println!("=> Copied config to all RBs successfully \u{1F389}!");
    info!("Copied config to all RBs successfully!");
  }
 
  restart_liftof_rb(&rb_list); 
  // FIXME - this needs to be a function
  // copy the current config file on all RBs
  //rb_handles.clear();
  //println!("=> Restarting liftof-rb clients on all RBs!");
  //let mut children = Vec::<(u8,Child)>::new();
  //for rb in &rb_list {
  //  // also populate the rb thread nandles
  //  rb_handles.push(thread::spawn(||{}));
  //  
  //  let rb_address = format!("tof-rb{:02}", rb.rb_id);
  //  match Command::new("ssh")
  //    .args([&rb_address, "sudo", "systemctl", "restart", "liftof"])
  //    .spawn() {
  //    Err(err) => {
  //      error!("Unable to spawn ssh process to restart liftoof-rb on RB {}! {}", rb.rb_id, err);
  //    }
  //    Ok(child) => {
  //      children.push((rb.rb_id,child));
  //    }
  //  }
  //}
  //let mut issues = Vec::<u8>::new();
  //for rb_child in &mut children {
  //  match rb_child.1.wait() {
  //    Err(err) => {
  //      error!("Child process failed with stderr {:?}! {}", rb_child.1.stderr, err);
  //    }
  //    Ok(status) => {
  //      if status.success() {
  //        info!("Restarted liftof-rb on {} successfully!", rb_child.0);
  //        //println!("=> Restarted liftof-rb on {} successfully \u{1F389}!", rb_child.0)
  //      } else {
  //        error!("Restart of liftof-rb on {} failed with exit code {:?}!", rb_child.0, status.code());
  //        issues.push(rb_child.0);
  //      }
  //    }
  //  }
  //}
  //if issues.len() == 0 {
  //  println!("=> Restarted liftof-rb on all RBs successfully \u{1F389}!");
  //  info!("=> Restarted liftof-rb on all RBs successfully!");
  //}

  let mtb_link_id_map = get_linkid_rbid_map(&rb_list);
  // A global kill timer
  let program_start = Instant::now();

  // Prepare outputfiles
  let mut new_run_id = 0u32;
  let mut stream_files_path = PathBuf::from(write_stream_path.clone());
  match args.command { 
    CommandCC::Run => {
      if write_stream {
        match prepare_run(write_stream_path.clone(), &config, runid, write_stream) {
          None => {
            error!("Unable to assign new run id, falling back to 0!");
          }
          Some(_rid) => {
            new_run_id = _rid;
            info!("Will use new run id {}!", new_run_id);
          }
        }
        // FIXME - ugly
        stream_files_path.push(new_run_id.to_string().as_str());
 
        // Now as we have the .toml file copied to our run location, we reload it
        // and reset the config settings in thread_control
        let cfg_file = format!("{}/run{}.toml", stream_files_path.display(), new_run_id);
        match LiftofSettings::from_toml(cfg_file) {
          Err(err) => {
            error!("CRITICAL! Unable to parse .toml settings file! {}", err);
            panic!("Unable to parse config file!");
          }
          Ok(_cfg) => {
            config = _cfg;
          }
        }
        // as well as upadte the shared memory
        match thread_control.lock() {
          Ok(mut tc) => {
            tc.liftof_settings    = config.clone();
            tc.run_id             = new_run_id;
            tc.new_run_start_flag = true;
          },
          Err(err) => {
            error!("Can't acquire lock for ThreadControl! Unable to set calibration mode! {err}");
          },
        }
      }
    }
    _ => ()
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


  // no cpu monitoring for cmdline calibration tasks
  #[cfg(feature="tof-ctrl")]
  if cpu_moni_interval > 0 {
    debug!("Starting main monitoring thread...");
    let _thread_control_c = Arc::clone(&thread_control);
    // this is anonymus, but we control the thread
    // through the thread control mechanism, so we
    // can still end it.
    let tp_to_sink_c = tp_to_sink.clone();
    let _cpu_moni_handle = thread::Builder::new()
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

  debug!("Starting data sink thread!");
  let thread_control_gds = Arc::clone(&thread_control);
  let dp_settings      = config.data_publisher_settings.clone();
  let _data_sink_handle = thread::Builder::new()
    .name("data-sink".into())
    .spawn(move || {
      global_data_sink(&tp_from_threads,
                       thread_control_gds, 
                       dp_settings);
    })
    .expect("Failed to spawn data-sink thread!");
  debug!("Data sink thread started!");
  let thread_control_sh = Arc::clone(&thread_control);
  let _sig_handle = thread::Builder::new()
    .name("signal_handler".into())
    .spawn(move || {
      signal_handler(
        thread_control_sh) 
      })
    .expect("Failed to spawn signal-handler thread!");
  debug!("Signal handler thread started!");

  debug!("Starting event builder and master trigger threads...");
  //let db_path_string    = config.db_path.clone();
  let evb_settings      = config.event_builder_settings.clone();
  let thread_control_eb = Arc::clone(&thread_control);
  let tp_to_sink_c      = tp_to_sink.clone();
  let _evtbldr_handle = thread::Builder::new()
    .name("event-builder".into())
    .spawn(move || {
                    event_builder(&master_ev_rec,
                                  &ev_from_rb,
                                  &tp_to_sink_c,
                                  new_run_id as u32,
                                  //db_path_string,
                                  mtb_link_id_map,
                                  evb_settings,
                                  thread_control_eb);
     })
    .expect("Failed to spawn event-builder thread!");
  // master trigger
  let mtb_moni_sender = tp_to_sink.clone(); 
  let thread_control_mt = Arc::clone(&thread_control);
  let _mtb_handle = thread::Builder::new()
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
  //println!("==> Will now start rb threads..");
  for n in 0..nboards {
    let mut this_rb           = rb_list[n].clone();
    let this_tp_to_sink_clone = tp_to_sink.clone();
    this_rb.calib_file_path   = calib_file_path.clone() + "latest";
    //match this_rb.load_latest_calibration() {
    //  Err(err) => panic!("Unable to load calibration for RB {}! {}", this_rb.rb_id, err),
    //  Ok(_)    => ()
    //}
    debug!("Starting RB thread for {}", this_rb.rb_id);
    let ev_to_builder_c = ev_to_builder.clone();
    let thread_name     = format!("rb-comms-{}", this_rb.rb_id);
    let settings        = config.analysis_engine_settings.clone();
    let ack_sender      = ack_to_cmd_disp.clone();
    let tc_rb_comm      = Arc::clone(&thread_control);
    let rb_comm_thread = thread::Builder::new()
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
    rb_handles.push(rb_comm_thread);
  } // end for loop over nboards
  //println!("=> All RB threads started!");
  println!("=> All threads initialized!");
  
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

  let mut bar = ProgressBar::hidden();

  // default  behaviour is to stop
  // when we are done
  let mut dont_stop = false;
  
  let mut command_socket : Option<zmq::Socket> = None;
  match args.command {
    CommandCC::Listen => {
      dont_stop = true;
      // start command dispatcher thread
      let tc = Arc::clone(&thread_control);
      let ts = tp_to_sink.clone();
      let _cmd_handle = thread::Builder::new()
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
     let tc_cali = thread_control.clone();
     calibrate_tof(tc_cali, &rb_list, true);
     end_program = true;
    },
    CommandCC::Staging => {
      error!("Staging sequence not implemented!");
    }
    CommandCC::Run => {
      if pre_run_calibration {
        let tc_cali = thread_control.clone();
        calibrate_tof(tc_cali, &rb_list, true);
        restart_liftof_rb(&rb_list);
      }
      if verification_rt_sec > 0 {
        let tc_verification = thread_control.clone();
        verification_run(verification_rt_sec, tc_verification);
        restart_liftof_rb(&rb_list);
      }
      thread::sleep(5*one_second);
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
      match thread_control.lock() {
        Ok(mut tc) => {
          // deactivate the master trigger thread
          tc.thread_master_trg_active = true;
          tc.calibration_active       = false;
          tc.thread_event_bldr_active = true;
          if write_stream {
            tc.write_data_to_disk       = true;
          }
          tc.run_id                   = new_run_id as u32;
          tc.new_run_start_flag       = true;
        },
        Err(err) => {
          error!("Can't acquire lock for ThreadControl! Unable to set calibration mode! {err}");
        },
      }
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
      bar.enable_steady_tick(Duration::from_secs(1));
      bar.set_message(".. acquiring data ..");
      // move the socket out of here for further use
      command_socket = Some(cmd_sender);
    }
  }

  //---------------------------------------------------------
  // 
  // Program main loop. Remember, most work is done in the 
  // individual threads. Here we have to check for ongoing
  // calibrations
  // 


  loop {
    // take out the heat a bit
    thread::sleep(5*one_second);

    if end_program {
      bar.finish();
      println!("=> Ending program!");
      println!("=> Sending run termination command to the RBs");
      let cmd          = TofCommand::DataRunStop(DEFAULT_RB_ID as u32);
      let packet       = cmd.pack();
      let mut payload  = String::from("BRCT").into_bytes();
      payload.append(&mut packet.to_bytestream());
      
      match command_socket {
        None => {
          warn!("=> No command socket available! Can not shut down RBs..!");
          // open 0MQ socket here
          let ctx = zmq::Context::new();
          let cmd_sender  = ctx.socket(zmq::PUB).expect("Unable to create 0MQ PUB socket!");
          let cc_pub_addr = config.cmd_dispatcher_settings.cc_server_address.clone();
          cmd_sender.bind(&cc_pub_addr).expect("Unable to bind to (PUB) socket!");
          // after we opened the socket, give the RBs a chance to connect
          println!("=> Give the RBs a chance to connect and wait a bit..");
          thread::sleep(10*one_second);
          match cmd_sender.send(&payload, 0) {
            Err(err) => {
              error!("Unable to send command! {err}");
            },
            Ok(_) => {
              debug!("We sent {:?}", payload);
            }
          }
        }
        Some(_sock) => {
          match _sock.send(&payload, 0) {
            Err(err) => {
              error!("Unable to send command! {err}");
            },
            Ok(_) => {
              debug!("We sent {:?}", payload);
            }
          }
        }
      }
      println!("=> Give the RBs a chance to connect and wait a bit..");
      thread::sleep(10*one_second);

      // for now, we end brutally
      // FIXME
      println!(">> So long and thanks for all the \u{1F41F} <<"); 
      exit(0);
    
      // FIXME - this all needs debugging. The goal is to shut down 
      // the threads in order
      //println!("=> Shutting down signal handler...");
      //// event builder first, to avoid a lot of error messages
      //match thread_control.lock() {
      //  Ok(mut tc) => {
      //    tc.thread_signal_hdlr_active = false;
      //  }
      //  Err(err) => {
      //    error!("Can't acquire lock for ThreadControl! Unable to set calibration mode! {err}");
      //  }
      //}
      //let _ = sig_handle.join();
      ////thread::sleep(2*one_second);
    
      //// end RB threads
      //println!("=> Shutting down rb threads...");
      //match thread_control.lock() {
      //  Ok(mut tc) => {
      //    for rb in &rb_list {
      //      if tc.thread_rbcomm_active.contains_key(&rb.rb_id) {
      //        *tc.thread_rbcomm_active.get_mut(&rb.rb_id).unwrap() = false;
      //      }
      //    }
      //  }
      //  Err(err) => {
      //    error!("Can't acquire lock for ThreadControl! Unable to set calibration mode! {err}");
      //  }
      //}

      //for k in rb_handles {
      //  let _ = k.join();
      //}

      //// event builder first, to avoid a lot of error messages
      //println!("=> Shutting down event builder...");
      //match thread_control.lock() {
      //  Ok(mut tc) => {
      //    tc.thread_event_bldr_active = false;
      //    println!("tc {}", tc);
      //  }
      //  Err(err) => {
      //    error!("Can't acquire lock for ThreadControl! Unable to set calibration mode! {err}");
      //  }
      //} 
      //println!("=> Waiting for event builder thread to finish up...");
      //println!("=> evt builder thread is finsihed: {}", evtbldr_handle.is_finished());
      //let _ = evtbldr_handle.join();
      //println!("=> .. done!");

      //match thread_control.lock() {
      //  Ok(mut tc) => {
      //    tc.stop_flag = true;
      //  },
      //  Err(err) => {
      //    error!("Can't acquire lock for ThreadControl! Unable to set calibration mode! {err}");
      //  },
      //}
      //// wait actually until all threads have been finished
      //let timeout = Instant::now();
      //loop {
      //  match thread_control.lock() {
      //    Ok(mut tc) => {
      //      tc.stop_flag = true;
      //      // each thread will report here if
      //      // it is done
      //      if !tc.thread_cmd_dispatch_active 
      //      && !tc.thread_data_sink_active
      //      && !tc.thread_event_bldr_active 
      //      && !tc.thread_master_trg_active {
      //        break;
      //      }
      //    },
      //    Err(err) => {
      //      error!("Can't acquire lock for ThreadControl! Unable to set calibration mode! {err}");
      //    },
      //  }
      //  // in any case, break after timeout
      //  if timeout.elapsed() > 5*one_second {
      //    break;
      //  }
      //}
      //println!(">> So long and thanks for all the \u{1F41F} <<"); 
      //exit(0);
    }

    // check thread control - this is useful 
    // for everything

    match thread_control.try_lock() {
      Ok(tc) => {
        if tc.stop_flag {
          end_program = true;
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
