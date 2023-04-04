//! # Radoutboard software for the GAPS experiment, TOF system
//! 
//! This software shall help with data acquisition and commandeering 
//! of the readoutboards (RB) used in the tof system of the GAPS 
//! science experiment.
//!
//!
//!
use std::{thread, time};

extern crate ctrlc;

extern crate crossbeam_channel;
use crossbeam_channel::{unbounded,
                        Sender,
                        Receiver};
use local_ip_address::local_ip;

//use std::collections::HashMap;
use std::process::exit;
use liftof_rb::api::*;
use liftof_rb::control::*;
use liftof_rb::memory::{
                    EVENT_SIZE,
                    DATABUF_TOTAL_SIZE};

use tof_dataclasses::threading::ThreadPool;
use tof_dataclasses::packets::TofPacket;
use tof_dataclasses::events::blob::RBEventPayload;
use tof_dataclasses::commands::{//TofCommand,
                                TofResponse,
                                TofOperationMode};

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
  /// Size of the internal eventbuffers which are mapped to /dev/uio1 
  /// and /dev/uio2. These buffers are maximum of about 64 MBytes.
  /// Depending on the event rate, this means that the events might
  /// sit quit a while in the buffers (~10s of seconds)
  /// To mitigate that waiting time, we can choose a smaller buffer
  /// The size of the buffer here is in <number_of_events_in_buffer>
  /// [! The default value is in bytes, since per default the buffers 
  /// don't hold an integer number of events]
  #[arg(short, long, default_value_t = 66524928)]
  buff_trip: usize,
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
  /// Readoutboard testing with internal trigger
  #[arg(long, default_value_t = false)]
  rb_test_ext : bool,
  /// Readoutboard testing with softare trigger
  #[arg(long, default_value_t = false)]
  rb_test_sw : bool,
  ///// CnC server IP we should be listening to
  //#[arg(long, default_value_t = "10.0.1.1")]
  //cmd_server_ip : &'static str,
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

  //let kraken                = vec![240, 159, 144, 153];
  // We know these bytes are valid, so we'll use `unwrap()`.
  //let kraken           = String::from_utf8(kraken).unwrap();

  // General parameters, readout board id,, 
  // ip to tof computer

  let rb_id = get_board_id().expect("Unable to obtain board ID!");
  let dna   = get_device_dna().expect("Unable to obtain device DNA!"); 
  
  
  let args = Args::parse();                   
  let mut buff_trip     = args.buff_trip;         
  let mut n_events_run  = args.nevents;
  let mut show_progress = args.show_progress;
  let cache_size        = args.cache_size;
  let run_forever       = args.run_forever;
  let mut stream_any    = args.stream_any;
  let mut force_trigger = args.force_trigger;
  let force_random_trig = args.force_random_trigger;
  let rb_test           = args.rb_test_ext || args.rb_test_sw;
  
  //FIMXE - this needs to become part of clap
  let cmd_server_ip = String::from("10.0.1.1");
  //let cmd_server_ip     = args.cmd_server_ip;  
  if rb_test {
    show_progress = true;
    n_events_run  = 1000;
    buff_trip     = 200;
    stream_any    = true;
    if args.rb_test_sw {
      force_trigger = 100;
    }
  }  

  if force_trigger > 0 && force_random_trig > 0 {
    panic!("Can not use force trigger (equally spaced in time) together with random self trigger!");
  }

  if force_random_trig > 0 {
      stream_any = true;
      buff_trip  = 2000;
      n_events_run = 5000;
  }

  let this_board_ip = local_ip().expect("Unable to obtainl local board IP. Something is messed up!");

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
  reset_data_memory_aggressively();
  reset_data_memory_aggressively();
  reset_data_memory_aggressively();
  //reset_data_memory_aggressively();
  //reset_data_memory_aggressively();
  let mut uio1_total_size = DATABUF_TOTAL_SIZE;
  let mut uio2_total_size = DATABUF_TOTAL_SIZE;

  if (buff_trip*EVENT_SIZE > uio1_total_size) || (buff_trip*EVENT_SIZE > uio2_total_size) {
    error!("Invalid value for --buff-trip. Panicking!");
    panic!("Tripsize of {buff_trip}*EVENT_SIZE exceeds buffer sizes of A : {uio1_total_size} or B : {uio2_total_size}. The EVENT_SIZE is {EVENT_SIZE}");
  }
  if buff_trip == DATABUF_TOTAL_SIZE {
    info!("Will set buffer trip size to an equivalent of {} events", buff_trip/EVENT_SIZE);
  } else {
    info!("Will set buffer trip size to an equivalent of {buff_trip} events");
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
  //FIXME - restrict to actual number of threads
  let mut n_threads = 12;
  if show_progress {
    n_threads += 1;
  }


  // FIXME - MESSAGES GET CONSUMED!!

  let (run_params_to_runner, run_params_from_cmdr)      : 
      (Sender<RunParams>, Receiver<RunParams>)                = unbounded();
  //let (cmd_to_client, cmd_from_zmq)      : 
  //    (Sender<TofCommand>, Receiver<TofCommand>)              = unbounded();
  let (rsp_to_sink, rsp_from_client)     : 
      (Sender<TofResponse>, Receiver<TofResponse>)            = unbounded();
  let (tp_to_pub, tp_from_client)        : 
      (Sender<TofPacket>, Receiver<TofPacket>)                = unbounded();
  //let (hasit_to_cmd, hasit_from_cache)   : 
  //    (Sender<bool>, Receiver<bool>)                          = unbounded();

  let (set_op_mode, get_op_mode)     : 
      (Sender<TofOperationMode>, Receiver<TofOperationMode>)                = unbounded();
  let (bs_send, bs_recv)             : (Sender<Vec<u8>>, Receiver<Vec<u8>>) = unbounded(); 
  //let (moni_to_main, data_fr_moni)   : (Sender<Vec<u8>>, Receiver<Vec<u8>>) = unbounded(); 
  let (ev_pl_to_cache, ev_pl_from_builder) : 
      (Sender<RBEventPayload>, Receiver<RBEventPayload>)                    = unbounded();
  //let (ev_pl_to_cmdr,  ev_pl_from_cache)   : 
  //  (Sender<Option<RBEventPayload>>, Receiver<Option<RBEventPayload>>)      = unbounded();
  let (evid_to_cache, evid_from_cmdr)   : (Sender<u32>, Receiver<u32>)      = unbounded();
  info!("Will start ThreadPool with {n_threads} threads");
  let workforce = ThreadPool::new(n_threads);
 

  if buff_trip != DATABUF_TOTAL_SIZE {
    uio1_total_size = EVENT_SIZE*buff_trip;
    uio2_total_size = EVENT_SIZE*buff_trip;
    buff_trip = EVENT_SIZE*buff_trip;
    // if we change the buff size, we HAVE to manually swith 
    // the buffers since the fw does not know about this change, 
    // it is software only
    info!("We set a value for buff_trip of {buff_trip}");
  }
 
  // write a few registers - this might go to 
  // the init script
  //match disable_evt_fragments() {
  //  Err(err) => error!("Can not disable writing of event fragments!"),
  //  Ok(_)    => ()
  //}
  // now we are ready to receive data 

  // Setup routine. Start the threads in inverse order of 
  // how far they are away from the buffers.
  
  
  let run_params_from_cmdr_c = run_params_from_cmdr.clone();
  let rdb_sender_a  = bs_send.clone();
  
  workforce.execute(move || {
    data_publisher(&tp_from_client, rb_test || force_random_trig > 0); 
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
      runner(&run_params_from_cmdr_c,
             buff_trip,
             None, 
             &rdb_sender_a,
             uio1_total_size,
             uio2_total_size,
             latch_to_mtb,
             show_progress,
             force_trigger);
  });

  let rsp_to_sink_c = rsp_to_sink.clone();
  workforce.execute(move || {
                    event_cache_worker(ev_pl_from_builder,
                                       //&cmd_from_zmq,
                                       //ev_pl_to_cmdr,
                                       &tp_to_pub_c,
                                       //&hasit_to_cmd,
                                       &rsp_to_sink_c,
                                       get_op_mode, 
                                       evid_from_cmdr,
                                       cache_size)
  });
  workforce.execute(move || {
                    event_payload_worker(&bs_recv, ev_pl_to_cache);
  });
  

  // Respond to commands from the C&C server
  let set_op_mode_c        = set_op_mode.clone();
  let run_params_to_runner_c = run_params_to_runner.clone();
  let heartbeat_timeout_seconds : u32 = 10;
  workforce.execute(move || {
                    cmd_responder(cmd_server_ip,
                                  heartbeat_timeout_seconds,
                                  &rsp_from_client,  
                                  &set_op_mode_c,
                                  &run_params_to_runner_c,
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
    
  //let run_params_from_cmdr_c = run_params_from_cmdr.clone();
  // we start the run by creating new RunParams
  if run_forever || n_events_run > 0 {
    let run_pars = RunParams {
      forever   : run_forever,
      nevents   : n_events_run as u32,
      is_active : true,
      nseconds  : 0
    };
    println!("Waiting for threads to start..");
    thread::sleep(time::Duration::from_secs(5));
    println!("..done");
    match run_params_to_runner.send(run_pars) {
      Err(err) => error!("Could not initialzie Run! Err {err}"),
      Ok(_)    => {
        println!("Run initialized! Attempting to start!");
      }
    }
  }
  //let mut resp     : cmd::TofResponse;
  //let r_clone  = ev_pl_from_cache.clone();
  //let executor = Commander::new(evid_to_cache,
  //                              &hasit_from_cache,
  //                              r_clone,
  //                              set_op_mode);

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

  ctrlc::set_handler(move || {
    println!("received Ctrl+C! We will stop triggers and end the run!");
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
  })
  .expect("Error setting Ctrl-C handler");


  loop {
    thread::sleep(10*one_sec);
  } // end loop
} // end main

