
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

use log::{info, LevelFilter};
use std::io::Write;

/// The 0MQ PUB port is defined as DATAPORT_START + readoutboard_id
const DATAPORT_START : u32 = 30000;

/// The 0MP REP port is defined as CMDPORT_START + readoutboard_id
const CMDPORT_START  : u32 = 40000;

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
  /// Value for wich the buffers are forced to 
  /// be read out!
  #[arg(short, long, default_value_t = 66520576)]
  buff_trip: u32,
  /// Listen to the server at the tof computer
  #[arg(short, long, default_value_t = false)]
  dont_listen: bool,
  /// Allow the software to switch buffers manually.
  /// This might be needed for custom values of buff-trip
  #[arg(short, long, default_value_t = false)]
  switch_buffers: bool,
  /// Show progress bars to indicate buffer fill values and number of acquired events
  #[arg(long, default_value_t = false)]
  show_progress: bool,
  /// Acquire this many events
  #[arg(short, long, default_value_t = 10000)]
  nevents: u64,
  /// Cache size of the internal event cache in events
  #[arg(short, long, default_value_t = 10000)]
  cache_size: usize,
  ///// A json config file with detector information
  //#[arg(short, long)]
  //json_config: Option<std::path::PathBuf>,
  /// Run without stopping. Control by remote through `TofCommand`
  #[arg(long, default_value_t = false)]
  run_forevever: bool,
  /// Activate the forced trigger. The value is the desired rate 
  #[arg(long, default_value_t = 0)]
  force_trigger: u32,
  /// Stream any eventy as soon as the software starts.
  /// Don't wait for command line.
  /// Behaviour can be controlled through `TofCommand` later
  #[arg(long, default_value_t = false)]
  stream_any : bool,
}


/// END IMPLEMENTATION OF THREADS

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

  let sparkle_heart         = vec![240, 159, 146, 150];
  let kraken                = vec![240, 159, 144, 153];
  let fish                  = vec![240, 159, 144, 159];
  let sparkles              = vec![226, 156, 168];
  let rocket                = vec![240, 159, 154, 128];
  // We know these bytes are valid, so we'll use `unwrap()`.
  let sparkle_heart    = String::from_utf8(sparkle_heart).unwrap();
  let kraken           = String::from_utf8(kraken).unwrap();
  let fish             = String::from_utf8(fish).unwrap();
  let sparkles         = String::from_utf8(sparkles).unwrap();
  let rocket           = String::from_utf8(rocket).unwrap();

  // General parameters, readout board id,, 
  // ip to tof computer

  let rb_id = get_board_id().expect("Unable to obtain board ID!");
  let dna   = get_device_dna().expect("Unable to obtain device DNA!"); 
  // this is currently not needed, since 
  // we are using the server/client setup wher
  // this is the client
  let mut address_ip = String::from("tcp://");
  let this_board_ip = local_ip().unwrap();
  match this_board_ip {
    IpAddr::V4(ip) => address_ip += &ip.to_string(),
    IpAddr::V6(_) => panic!("Currently, we do not support IPV6!")
  }
  
  // Set up 2 ports for 0MQ communications
  // 1) control flow REP 
  // 2) data flow PUB
  let cmd_port    = CMDPORT_START + get_board_id().unwrap();
  let cmd_address : String = address_ip.clone() + ":" + &cmd_port.to_string();
  
  let data_port    = DATAPORT_START + get_board_id().unwrap();
  let data_address : String = address_ip + ":" + &data_port.to_string();
  
  let args = Args::parse();                   
  let buff_trip     = args.buff_trip;         
  let switch_buff   = args.switch_buffers;    
  let max_event     = args.nevents;
  let show_progress = args.show_progress;
  let cache_size    = args.cache_size;
  let dont_listen   = args.dont_listen;
  let run_forever   = args.run_forevever;
  let stream_any    = args.stream_any;
  let force_trigger = args.force_trigger;

  // welcome banner!
  println!("-----------------------------------------------");
  println!(" ** Welcome to liftof-rb {} \u{1F388} *****", rocket);
  println!(" .. liftof if a software suite for the time-of-flight detector ");
  println!(" .. for the GAPS experiment {}", sparkle_heart);
  println!(" .. this client can be run standalone or connect to liftof-cc" );
  println!(" .. or liftof-tui for an interactive experience" );
  println!(" .. see the gitlab repository for documentation and submitting issues at" );
  println!(" **https://uhhepvcs.phys.hawaii.edu/Achim/gaps-online-software/-/tree/main/tof/liftof**");
  println!("-----------------------------------------------");
  println!(" => Running client for RB {}", rb_id);
  println!(" => ReadoutBoard DNA {}", dna);
  println!(" => Will bind local ZMQ PUB socket for data stream to {}", data_address);
  if !dont_listen { 
    println!(" => Will bind local ZMQ REP socket for control to {}"  , cmd_address);
  } 
  println!("-----------------------------------------------");
  println!("");                             
                            
  let mut uio1_total_size = (UIO1_MAX_OCCUPANCY - UIO1_MIN_OCCUPANCY) as u64;
  let mut uio2_total_size = (UIO2_MAX_OCCUPANCY - UIO2_MIN_OCCUPANCY) as u64;

  if (buff_trip > uio1_total_size as u32 ) || (buff_trip > uio2_total_size as u32) {
    println!("Invalid value for --buff-trip. Panicking!");
    panic!("Tripsize of {buff_trip} exceeds buffer sizes of A : {uio1_total_size} or B : {uio2_total_size}");
  }

  info!("Will set buffer trip size to {buff_trip}");


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
  //let (kill_bars, bar_killed        ) : (Sender<bool>, Receiver<bool>) = unbounded();  


  //thread::sleep(one_milli);
  
  if buff_trip != 66520576 {
    uio1_total_size = buff_trip as u64;
    uio2_total_size = buff_trip as u64;
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
  

  if !dont_listen {
    let set_op_mode_c = set_op_mode.clone();
    let run_params_to_main_c = run_params_to_main.clone();
    workforce.execute(move || {
                      cmd_responder(&rsp_from_client,  
                                    &set_op_mode_c,
                                    &run_params_to_main_c,
                                    &evid_to_cache )
                                    //&cmd_to_client   )  
    
    });
  }
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
  if dont_listen {  
    let mut p_op : Option<Sender<u64>> = None;
    if show_progress {
      let tmp_send = pb_ev_up_send.clone();
      p_op = Some(tmp_send); 
    }
    let run_params_from_cmdr_c = run_params_from_cmdr.clone();
    workforce.execute(move || {
        runner(Some(max_event),
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
      nevents   : max_event as u32,
      is_active : true,
    };
    match run_params_to_main.send(run_pars) {
      Err(err) => warn!("Could not initialzie Run! Err {err}"),
      Ok(_)    => info!("Run initialized!")
    }
  }

  // Now set up PUB socket 
  // The pub socket is always present, even in don't listen configuration
  // (Nobody is forced to listen to it, and it will just drop its data 
  // if it does not have any subscribers)
  //let data_socket = ctx.socket(zmq::PUB).expect("Unable to create 0MQ PUB socket!");
  //data_socket.bind(&data_address).expect("Unable to bind to data (PUB) socket {data_adress}");
  //info!("0MQ SUB socket bound to address {data_address}");
 
  workforce.execute(move || {
    data_publisher(&tp_from_client); 
  });
  // Now setup thread which require the 
  // data socket.
  workforce.execute(move || {
    monitoring(&tp_to_pub);
  });
  if show_progress {
    let kill_clone = run_gets_killed.clone();
    workforce.execute(move || { 
      progress_runner(max_event,      
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
  //let rate = get_trigger_rate().unwrap();
  //info!("Current trigger rate: {rate}Hz");
  //let mut command  : cmd::TofCommand;
  if stream_any {
    match set_op_mode.send(TofOperationMode::TofModeStreamAny) {
      Err(err) => warn!("Can not set TofOperationMode to StreamAny! Err {err}"),
      Ok(_)    => ()
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
    if dont_listen {
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

