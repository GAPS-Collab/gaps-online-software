//! # Radoutboard software for the GAPS experiment, TOF system
//! 
//! This software shall help with data acquisition and commandeering 
//! of the readoutboards (RB) used in the tof system of the GAPS 
//! science experiment.
//!
//!
//!
use std::{thread, time};

extern crate crossbeam_channel;
use crossbeam_channel::{unbounded,
                        Sender,
                        Receiver};

use std::net::IpAddr;

use local_ip_address::local_ip;

//use std::collections::HashMap;

use liftof_rb::api::*;
use liftof_rb::control::*;
use liftof_rb::memory::{BlobBuffer,
                    EVENT_SIZE,
                    DATABUF_TOTAL_SIZE,
                    UIO1_MAX_OCCUPANCY,
                    UIO2_MAX_OCCUPANCY,
                    UIO1_MIN_OCCUPANCY,
                    UIO2_MIN_OCCUPANCY};
use tof_dataclasses::commands::*;

use tof_dataclasses::threading::ThreadPool;
use tof_dataclasses::packets::{TofPacket,
                               PacketType};
use tof_dataclasses::packets::generic_packet::GenericPacket;
use tof_dataclasses::events::blob::RBEventPayload;
use tof_dataclasses::commands::{TofCommand,
                                TofResponse,
                                TofOperationMode};
use tof_dataclasses::commands as cmd;
use tof_dataclasses::monitoring as moni;
use tof_dataclasses::serialization::Serialization;
//use liftof_lib::misc::*;
extern crate pretty_env_logger;
#[macro_use] extern crate log;

//use log::{info, LevelFilter};
use std::io::Write;


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

  let kraken                = vec![240, 159, 144, 153];
  let fish                  = vec![240, 159, 144, 159];
  let sparkles              = vec![226, 156, 168];
  // We know these bytes are valid, so we'll use `unwrap()`.
  let kraken           = String::from_utf8(kraken).unwrap();
  let fish             = String::from_utf8(fish).unwrap();
  let sparkles         = String::from_utf8(sparkles).unwrap();

  // General parameters, readout board id,, 
  // ip to tof computer

  let rb_id = get_board_id().expect("Unable to obtain board ID!");
  let dna   = get_device_dna().expect("Unable to obtain device DNA!"); 
  
  let mut switch_buff   = false;
  
  let args = Args::parse();                   
  let buff_trip         = args.buff_trip;         
  let mut n_events_run  = args.nevents;
  let mut show_progress = args.show_progress;
  let cache_size        = args.cache_size;
  let run_forever       = args.run_forever;
  let stream_any        = args.stream_any;
  let mut force_trigger = args.force_trigger;
  let rb_test           = args.rb_test_ext || args.rb_test_sw;
  
  //FIMXE - this needs to become part of clap
  let cmd_server_ip = String::from("10.0.1.1");
  //let cmd_server_ip     = args.cmd_server_ip;  
  if rb_test {
    show_progress = true;
    n_events_run = 880;
    if args.rb_test_sw {
      force_trigger = 2000;
    }
  }  

  let this_board_ip = local_ip().expect("Unable to obtainl local board IP. Something is messed up!");
  let rate = get_trigger_rate().expect("I can not read from the get trigger rate register, this is bad!");

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
  println!(" => Currently the board sees triggers at {rate} Hz");
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
  let mut uio1_total_size = DATABUF_TOTAL_SIZE;
  let mut uio2_total_size = DATABUF_TOTAL_SIZE;

  if (buff_trip*EVENT_SIZE > uio1_total_size) || (buff_trip*EVENT_SIZE > uio2_total_size) {
    error!("Invalid value for --buff-trip. Panicking!");
    panic!("Tripsize of {buff_trip}*EVENT_SIZE exceeds buffer sizes of A : {uio1_total_size} or B : {uio2_total_size}. The EVENT_SIZE is {EVENT_SIZE}");
  }

  info!("Will set buffer trip size to an equivalent of {buff_trip} events");

  // some pre-defined time units for 
  // sleeping
  let two_seconds = time::Duration::from_millis(2000);
  let one_milli   = time::Duration::from_millis(1);
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
  let mut n_threads = 20;
  if show_progress {
    n_threads += 1;
  }
 

  let (run_params_to_main, run_params_from_cmdr)      : 
      (Sender<RunParams>, Receiver<RunParams>)                = unbounded();
  let (cmd_to_client, cmd_from_zmq)      : 
      (Sender<TofCommand>, Receiver<TofCommand>)              = unbounded();
  let (rsp_to_sink, rsp_from_client)     : 
      (Sender<TofResponse>, Receiver<TofResponse>)            = unbounded();
  let (tp_to_pub, tp_from_client)        : 
      (Sender<TofPacket>, Receiver<TofPacket>)                = unbounded();
  let (hasit_to_cmd, hasit_from_cache)   : 
      (Sender<bool>, Receiver<bool>)                          = unbounded();

  let (set_op_mode, get_op_mode)     : 
      (Sender<TofOperationMode>, Receiver<TofOperationMode>)                = unbounded();
  let (kill_run, run_gets_killed)    : (Sender<bool>, Receiver<bool>)       = unbounded();
  let (bs_send, bs_recv)             : (Sender<Vec<u8>>, Receiver<Vec<u8>>) = unbounded(); 
  let (moni_to_main, data_fr_moni)   : (Sender<Vec<u8>>, Receiver<Vec<u8>>) = unbounded(); 
  let (ev_pl_to_cache, ev_pl_from_builder) : 
      (Sender<RBEventPayload>, Receiver<RBEventPayload>)                    = unbounded();
  let (ev_pl_to_cmdr,  ev_pl_from_cache)   : 
    (Sender<Option<RBEventPayload>>, Receiver<Option<RBEventPayload>>)      = unbounded();
  let (evid_to_cache, evid_from_cmdr)   : (Sender<u32>, Receiver<u32>)      = unbounded();
  info!("Will start ThreadPool with {n_threads} threads");
  let workforce = ThreadPool::new(n_threads);
 
  // these are only for the progress bars
  let (pb_a_up_send, pb_a_up_recv   ) : (Sender<u64>, Receiver<u64>) = unbounded();  
  let (pb_b_up_send, pb_b_up_recv   ) : (Sender<u64>, Receiver<u64>) = unbounded(); 
  let (pb_ev_up_send, pb_ev_up_recv ) : (Sender<u64>, Receiver<u64>) = unbounded(); 

  if buff_trip != DATABUF_TOTAL_SIZE {
    uio1_total_size = EVENT_SIZE*buff_trip;
    uio2_total_size = EVENT_SIZE*buff_trip;
    // if we change the buff size, we HAVE to manually swith 
    // the buffers since the fw does not know about this change, 
    // it is software only
    switch_buff     = true;
  }
 
  // write a few registers - this might go to 
  // the init script
  match disable_evt_fragments() {
    Err(err) => error!("Can not disable writing of event fragments!"),
    Ok(_)    => ()
  }
  // now we are ready to receive data 

  // Setup routines 
  // Start the individual worker threads
  // in meaningfull order
  // - higher level threads first, then 
  // the more gnarly ones.
  let tp_to_pub_c = tp_to_pub.clone();
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
  

  /// Respond to commands from the C&C server
  let set_op_mode_c        = set_op_mode.clone();
  let run_params_to_main_c = run_params_to_main.clone();
  let heartbeat_timeout_seconds : u32 = 10;
  workforce.execute(move || {
                    cmd_responder(cmd_server_ip,
                                  heartbeat_timeout_seconds,
                                  &rsp_from_client,  
                                  &set_op_mode_c,
                                  &run_params_to_main_c,
                                  &evid_to_cache )
                                  //&cmd_to_client   )  
  
  });
  // this thread deals JUST with the data
  // buffers. It reads them and then 
  // passes on the data
  let rdb_sender  = bs_send.clone();
  //let prog_sender = pb_a_up_send;
  let mut pb_a = None;
  let mut pb_b = None;
  if show_progress {
    pb_a = Some(pb_a_up_send.clone());
    pb_b = Some(pb_b_up_send.clone());

  }
  workforce.execute(move || {
    read_data_buffers(rdb_sender,
                      buff_trip,
                      //prog_op_a,
                      //prog_op_b
                      pb_a,
                      pb_b,
                      switch_buff);
  });

  // if we are not listening to the C&C server,
  // we have to start the run thread here
  if n_events_run > 0 || run_forever {  
    let mut p_op : Option<Sender<u64>> = None;
    if show_progress {
      let tmp_send = pb_ev_up_send.clone();
      p_op = Some(tmp_send); 
    }
    let run_params_from_cmdr_c = run_params_from_cmdr.clone();
    workforce.execute(move || {
        runner(Some(n_events_run),
               None,
               None,
               p_op,
               &run_params_from_cmdr_c,
               None,
               force_trigger);
               //bar_clone);
    });
    // we start the run by creating new RunParams
    let run_pars = RunParams {
      forever   : run_forever,
      nevents   : n_events_run as u32,
      is_active : true,
    };
    match run_params_to_main.send(run_pars) {
      Err(err) => error!("Could not initialzie Run! Err {err}"),
      Ok(_)    => println!("Run initialized! Attempting to start!")
    }
  }
 
  workforce.execute(move || {
    data_publisher(&tp_from_client, rb_test); 
  });
  // Now setup thread which require the 
  // data socket.
  workforce.execute(move || {
    monitoring(&tp_to_pub);
  });
  if show_progress {
    let kill_clone = run_gets_killed.clone();
    workforce.execute(move || { 
      progress_runner(n_events_run,      
                      uio1_total_size,
                      uio2_total_size,
                      pb_a_up_recv ,
                      pb_b_up_recv ,
                      pb_ev_up_recv.clone(),
                      kill_clone  )
    });
  }

  //info!("Starting daq!");
  //match start_drs4_daq() {
  //  Ok(_)    => info!(".. successful!"),
  //  Err(_)   => panic!("DRS4 start failed!")
  //}

  // let go for a few seconds to get a 
  // rate estimate
  //thread::sleep(two_seconds);
  //info!("Current trigger rate: {rate}Hz");
  //let mut command  : cmd::TofCommand;
  if stream_any {
    match set_op_mode.send(TofOperationMode::TofModeStreamAny) {
      Err(err) => error!("Can not set TofOperationMode to StreamAny! Err {err}"),
      Ok(_)    => info!("Using RBMode STREAM_ANY")
    }
  }

  let mut resp     : cmd::TofResponse;
  let r_clone  = ev_pl_from_cache.clone();
  //let executor = Commander::new(evid_to_cache,
  //                              &hasit_from_cache,
  //                              r_clone,
  //                              set_op_mode);
  let mut run_active = false;
  
  loop {
    // what we are here listening to, are commands which 
    // impact threads. E.g. StartRun will start a new data run
    // which is it's own thread
    if n_events_run > 0 || run_forever {
      thread::sleep(10*one_sec);
      continue;
    }

    match run_params_from_cmdr.recv() {
      Err(err) => trace!("Did not receive a new set of run pars {err}"),
      Ok(run)    => {
        if run.is_active { 
          // start a new run. 
          // is there one active?
          if run_active {
            let resp = TofResponse::GeneralFail(RESP_ERR_RUNACTIVE);
            match rsp_to_sink.send(resp) {
              Err(err) => warn!("Unable to send response! Err {err}"),
              Ok(_)    => ()
            }
          } else {
            let run_params_from_cmdr_c = run_params_from_cmdr.clone();
            workforce.execute(move || {
                runner(Some(run.nevents as u64),
                       None,
                       None,
                       //FIXME - maybe use crossbeam?
                       //p_op,
                       None,
                       &run_params_from_cmdr_c,
                       None,
                       force_trigger);
                       //Some(&rk));
                       //bar_clone);
            });
          }
        }
      }
    }
  } // end loop
} // end main

