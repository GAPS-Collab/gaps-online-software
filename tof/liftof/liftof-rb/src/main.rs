//! # Radoutboard software for the GAPS experiment, TOF system
//! 
//! This software shall help with data acquisition and commandeering 
//! of the readoutboards (RB) used in the tof system of the GAPS 
//! science experiment.
//!
//!
//!
use std::{thread, time};

//extern crate ctrlc;
extern crate libc;
extern crate crossbeam_channel;
extern crate signal_hook;

use signal_hook::iterator::Signals;
use signal_hook::consts::signal::{SIGTERM, SIGINT};
use crossbeam_channel::{unbounded,
                        Sender,
                        Receiver};
use local_ip_address::local_ip;

//use std::collections::HashMap;
use std::process::exit;
use liftof_rb::api::*;
use liftof_rb::control::*;
use liftof_rb::memory::read_control_reg;
use liftof_rb::memory::{
                    EVENT_SIZE,
                    DATABUF_TOTAL_SIZE};

use tof_dataclasses::threading::ThreadPool;
use tof_dataclasses::packets::TofPacket;
use tof_dataclasses::events::blob::RBEventPayload;
use tof_dataclasses::commands::{//TofCommand,
                                TofResponse,
                                TofOperationMode};
use tof_dataclasses::run::RunConfig;
use tof_dataclasses::serialization::Serialization;
extern crate pretty_env_logger;
#[macro_use] extern crate log;

//use log::{info, LevelFilter};
//use std::io::Write;


extern crate clap;
use clap::{arg,
           command,
           //value_parser,
           //ArgAction,
           //Command,
           Parser};

#[derive(Parser, Debug)]
#[command(author = "J.A.Stoessl", version, about, long_about = None)]
struct Args {
  /// Show progress bars to indicate buffer fill values and number of acquired events
  #[arg(long, default_value_t = false)]
  show_progress: bool,
  /// Acquire this many events.
  /// If either --nevents or --run-forever options are given
  /// the board will not wait for a remote command, but start datataking as soon as 
  /// possible
  #[arg(short, long, default_value_t = 0)]
  nevents: u64,
  /// Cache size of the internal event cache in events
  #[arg(short, long, default_value_t = 10000)]
  cache_size: usize,
  /// If either --nevents or --run-forever options are given
  /// the board will not wait for a remote command, but start datataking as soon as 
  /// possible
  #[arg(long, default_value_t = false)]
  run_forever: bool,
  /// Analyze the waveforms directly on the board. We will not send
  /// waveoform data, but paddle packets instead.
  #[arg(long, default_value_t = false)]
  waveform_analysis: bool,
  /// Activate the forced trigger. The value is the desired rate 
  #[arg(long, default_value_t = 0)]
  force_trigger: u32,
  /// Activate the forced random trigger. The value is the desired rate
  #[arg(long, default_value_t = 0)]
  force_random_trigger: u32,
  /// Stream any eventy as soon as the software starts.
  /// Don't wait for command line.
  /// Behaviour can be controlled through `TofCommand` later
  #[arg(long, default_value_t = false)]
  stream_any : bool,
  /// Readoutboard testing with external trigger
  #[arg(long, default_value_t = false)]
  rb_test_ext : bool,
  /// Readoutboard testing with softare trigger, equally spaced in time
  #[arg(long, default_value_t = false)]
  rb_test_sw : bool,
  /// Take data for voltage calibration
  #[arg(long, default_value_t = false)]
  vcal : bool,
  /// Take data for timing calibration
  #[arg(long, default_value_t = false)]
  tcal : bool,
  /// Take data with no inputs [NOT IMPLEMENTED YET]
  #[arg(long, default_value_t = false)]
  noi : bool,
  ///// CnC server IP we should be listening to
  //#[arg(long, default_value_t = "10.0.1.1")]
  //cmd_server_ip : &'static str,
  /// A json run config file with a RunConfiguration
  #[arg(short, long)]
  run_config: Option<std::path::PathBuf>,
}

fn main() {

  //env_logger::Builder::new()
  //    .format(|buf, record| {
  //     writeln!(
  //     buf,
  //     "{}:{} {} [{}] - {}",
  //     record.file().unwrap_or("unknown"),
  //      record.line().unwrap_or(0),
  //     chrono::Local::now().format("%Y-%m-%dT%H:%M:%S"),
  //          record.level(),
  //       record.args()
  //     )
  //                                })
  //.filter(Some("logger_example"), LevelFilter::Debug)
  //                        .init();
  pretty_env_logger::init();
 
  let args = Args::parse();                   
  let mut n_events_run      = args.nevents;
  let mut show_progress     = args.show_progress;
  let cache_size            = args.cache_size;
  let mut run_forever       = args.run_forever;
  let mut stream_any        = args.stream_any;
  let mut force_trigger     = args.force_trigger;
  let mut force_random_trig = args.force_random_trigger;
  let wf_analysis           = args.waveform_analysis;
  let mut rb_test           = args.rb_test_ext || args.rb_test_sw;
  let vcal                  = args.vcal;
  let tcal                  = args.tcal;
  let noi                   = args.noi;
  let run_config            = args.run_config;
  let mut data_format       = 0u8;
  // active channels
  let mut ch_mask : u8 = u8::MAX;

  let mut rc_config = RunConfig::new();
  let mut rc_file_path  = std::path::PathBuf::new();
  match run_config {
    None     => (),
    Some(rcfile) => {
      rc_file_path = rcfile.clone();
      rc_config    = get_runconfig(&rcfile);
      ch_mask      = rc_config.active_channel_mask;
      data_format  = rc_config.data_format;
    }
  }

  let mut file_suffix   = String::from(".robin");

  if ( vcal && tcal ) || ( vcal && noi ) || ( tcal && noi ) {
    panic!("Can only support either of the flags --vcal --tcal --noi")
  }

  let mut end_after_run = false;
  if noi {
    file_suffix   = String::from(".noi");
    rb_test       = true;
    end_after_run = true;
  }
  if vcal {
    file_suffix   = String::from(".vcal");
    rb_test       = true;
    end_after_run = true;
  }

  if tcal {
    file_suffix = String::from(".tcal");
    force_random_trig = 100;
    show_progress     = true;
    n_events_run      = 1000;
    end_after_run = true;
  }

  //FIMXE - this needs to become part of clap
  let cmd_server_ip = String::from("10.0.1.1");
  //let cmd_server_ip     = args.cmd_server_ip;  
  if rb_test {
    show_progress = true;
    n_events_run  = 1000;
    stream_any    = true;
    if args.rb_test_sw || vcal {
      force_trigger = 100;
    }
    end_after_run = true;
  }  

  if force_trigger > 0 && force_random_trig > 0 {
    panic!("Can not use force trigger (equally spaced in time) together with random self trigger!");
  }

  if force_random_trig > 0 {
      stream_any = true;
      n_events_run = 5000;
  }

  let this_board_ip = local_ip().expect("Unable to obtainl local board IP. Something is messed up!");
  
  // General parameters, readout board id,, 
  // ip to tof computer
  let rb_id = get_board_id().expect("Unable to obtain board ID!");
  let dna   = get_device_dna().expect("Unable to obtain device DNA!"); 

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
  if rb_test {
    println!("=> We will run in rb testing mode!");
    println!("-----------------------------------------------"); 
  } 
 
  // set channel mask (if different from 255)
  match set_active_channel_mask(ch_mask) {
    Ok(_) => (),
    Err(err) => {
      error!("Setting activve channel mask failed for mask {}, error {}", ch_mask, err);
    }
  }
  let current_mask = read_control_reg(0x44).unwrap();
  println!("CURRENT MASK = {}", current_mask);
  //exit(0);

  // this resets the data buffer /dev/uio1,2 occupancy
  reset_dma_and_buffers();

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
  //FIXME - restrict to actual number of threads
  let mut n_threads = 12;
  if show_progress {
    n_threads += 1;
  }


  // FIXME - MESSAGES GET CONSUMED!!

  let (rc_to_runner, rc_from_cmdr)      : 
      (Sender<RunConfig>, Receiver<RunConfig>)                = unbounded();
  //let (cmd_to_client, cmd_from_zmq)      : 
  //    (Sender<TofCommand>, Receiver<TofCommand>)              = unbounded();
  let (rsp_to_sink, rsp_from_client)     : 
      (Sender<TofResponse>, Receiver<TofResponse>)            = unbounded();
  let (tp_to_pub, tp_from_client)        : 
      (Sender<TofPacket>, Receiver<TofPacket>)                = unbounded();
  //let (hasit_to_cmd, hasit_from_cache)   : 
  //    (Sender<bool>, Receiver<bool>)                          = unbounded();
  let (tp_to_cache, tp_from_builder) : 
      (Sender<TofPacket>, Receiver<TofPacket>)                = unbounded();


  let (set_op_mode, get_op_mode)     : 
      (Sender<TofOperationMode>, Receiver<TofOperationMode>)                = unbounded();
  let (bs_send, bs_recv)             : (Sender<Vec<u8>>, Receiver<Vec<u8>>) = unbounded(); 
  //let (moni_to_main, data_fr_moni)   : (Sender<Vec<u8>>, Receiver<Vec<u8>>) = unbounded(); 
  let (ev_pl_to_cache, ev_pl_from_worker) : 
      (Sender<RBEventPayload>, Receiver<RBEventPayload>)                    = unbounded();
  //let (ev_pl_to_cmdr,  ev_pl_from_cache)   : 
  //  (Sender<Option<RBEventPayload>>, Receiver<Option<RBEventPayload>>)      = unbounded();

  let (evid_to_cache, evid_from_cmdr)   : (Sender<u32>, Receiver<u32>)      = unbounded();
  info!("Will start ThreadPool with {n_threads} threads");
  let workforce = ThreadPool::new(n_threads);
 

  // Setup routine. Start the threads in inverse order of 
  // how far they are away from the buffers.
  let rc_from_cmdr_c = rc_from_cmdr.clone();
  let bs_send_c      = bs_send.clone();
  
  workforce.execute(move || {
    data_publisher(&tp_from_client,
                   rb_test || force_random_trig > 0,
                   Some(&file_suffix)); 
  });
  let tp_to_pub_c   = tp_to_pub.clone();
  workforce.execute(move || {
    monitoring(&tp_to_pub);
  });

  // if we don't set a rate for force_random_trig, 
  // latch to the MTB. For the other force trigger
  // modes, the runner will decide 
  // FIXME: decide everything here
  let latch_to_mtb : bool = force_random_trig == 0;

  // then the runner. It does nothing, until we send a set
  // of RunParams
  workforce.execute(move || {
      runner(&rc_from_cmdr_c,
             None, 
             &bs_send_c,
             latch_to_mtb,
             show_progress,
             force_trigger);
  });

  let rsp_to_sink_c = rsp_to_sink.clone();
  workforce.execute(move || {
                    event_cache(ev_pl_from_worker,
                                tp_from_builder,
                                &tp_to_pub_c,
                                &rsp_to_sink_c,
                                get_op_mode, 
                                evid_from_cmdr,
                                cache_size)
  });
  workforce.execute(move || {
                    event_processing(&bs_recv,
                                     tp_to_cache,
                                     data_format);
  });
  

  // Respond to commands from the C&C server
  let set_op_mode_c        = set_op_mode.clone();
  let rc_to_runner_c = rc_to_runner.clone();
  let heartbeat_timeout_seconds : u32 = 10;
  workforce.execute(move || {
                    cmd_responder(cmd_server_ip,
                                  heartbeat_timeout_seconds,
                                  &rsp_from_client,  
                                  &set_op_mode_c,
                                  &rc_file_path,
                                  &rc_to_runner_c,
                                  &evid_to_cache )
                                  //&cmd_to_client   )  
  
  });

  // if we are not listening to the C&C server,
  // we have to start the run thread here
  if stream_any {
    match set_op_mode.send(TofOperationMode::TofModeStreamAny) {
      Err(err) => error!("Can not set TofOperationMode to StreamAny! Err {err}"),
      Ok(_)    => info!("Using RBMode STREAM_ANY")
    }
  }
 
  // We can only start a run here, if this is not
  // run through systemd
  if !is_systemd_process() {
    // if we are not as systemd, 
    // always end when we are done
    println!("We are not run by systemd, sow we will stop the program when it is done");
    // we start the run by creating new RunParams
    // this is only if we give 
    if run_forever || n_events_run > 0 {
      //let mut rc = RunConfig::new();
      if run_forever {
        rc_config.nevents = 0;
      } else {
        if rc_config.nevents == 0 && n_events_run != 0 {
          rc_config.nevents = n_events_run as u32;
        }
      }
      if rc_config.nevents > 0 {
        end_after_run = true;
      }
      rc_config.is_active = true;
      //rc.rb_buff_size = 2000;
      println!("Waiting for threads to start..");
      thread::sleep(time::Duration::from_secs(5));
      println!("..done");
      match rc_to_runner.send(rc_config) {
        Err(err) => error!("Could not initialzie Run! Err {err}"),
        Ok(_)    => {
          println!("Run initialized! Attempting to start!");
        }
      }
    }
  }
  
  // if we arrive at this point and we want the random trigger, 
  // we are now ready to start it
  if force_random_trig > 0 {

    // we have to calculate the actual rate with Andrew's formulat
    //let clk_period : f64 = 1.0/33e6;
    let rate : f32 = force_random_trig as f32;
    let max_val  : f32 = 4294967295.0;
    
    //let f_trig = (33e6 * (rate/max_val)) as u32;
    //let reg_val = 1/rate = 33e6/max_val*1/f_trig
    let reg_val = (rate/(33e6/max_val)) as u32;
    info!("Will use random self trigger with rate {reg_val} value for register, corresponding to {rate} Hz");
    match set_self_trig_rate(reg_val) {
      Err(err) => {
        warn!("Setting self trigger failed! Er {err}");
        panic!("Abort!");
      }
      Ok(_)    => ()
    }
  }

  let mut signals = Signals::new(&[SIGTERM, SIGINT]).expect("Unknown signals");
  let mut end = false;
  
  // Wait until all threads are set up
  thread::sleep(5*one_sec);
  loop {
    thread::sleep(1*one_sec);
    for signal in signals.pending() {
      match signal as libc::c_int {
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
      println!("=> Finish program!");
      println!("=> Stopping triggers!");
      println!("So long and thanks for all the \u{1F41F}");
   
      match disable_trigger() {
        Err(err) => error!("Can not disable triggers, error {err}"),
        Ok(_)    => ()
      }
      if force_random_trig > 0 {
        match set_self_trig_rate(0) {
          Err(err) => {
            panic!("Could not disable random self trigger! Err {err}");
          }
          Ok(_)    => ()
        }
      }
      exit(0);
    }

  } // end loop
} // end main

