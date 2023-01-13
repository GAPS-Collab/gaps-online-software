mod registers;
mod memory;
mod control;
mod api;

use std::{thread, time};
use std:: {sync::mpsc::Sender,
           sync::mpsc::Receiver,
           sync::mpsc::channel};
use std::net::IpAddr;

use indicatif::{MultiProgress,
                ProgressBar,
                ProgressStyle};

use local_ip_address::local_ip;

use std::collections::HashMap;

use crate::api::*;
use crate::control::*;
use crate::memory::{BlobBuffer,
                    UIO1_MAX_OCCUPANCY,
                    UIO2_MAX_OCCUPANCY,
                    UIO1_MIN_OCCUPANCY,
                    UIO2_MIN_OCCUPANCY};
use tof_dataclasses::threading::ThreadPool;
use tof_dataclasses::packets::TofPacket;
use tof_dataclasses::packets::generic_packet::GenericPacket;
use tof_dataclasses::events::blob::RBEventPayload;
use tof_dataclasses::commands::{TofCommand,
                                TofResponse};
use tof_dataclasses::commands as cmd;
use tof_dataclasses::serialization::Serialization;
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
}


extern crate pretty_env_logger;
#[macro_use] extern crate log;

/// Non-register related constants
const TEMPLATE_BAR_A  : &str = "[{elapsed_precise}] {prefix} {msg} {spinner} {bar:60.blue/grey} {bytes:>7}/{total_bytes:7} ";
const TEMPLATE_BAR_B  : &str = "[{elapsed_precise}] {prefix} {msg} {spinner} {bar:60.green/grey} {bytes:>7}/{total_bytes:7} ";
const TEMPLATE_BAR_EV : &str = "[{elapsed_precise}] {prefix} {msg} {spinner} {bar:60.red/grey} {pos:>7}/{len:7}";

///! The 0MQ PUB port is defined as DATAPORT_START + readoutboard_id
const DATAPORT_START : u32 = 30000;

///! The 0MP REP port is defined as CMDPORT_START + readoutboard_id
const CMDPORT_START  : u32 = 40000;


///! Keep track of send/receive state of 0MQ socket

///!  Threads:
///! 
///!  - server          : comms with tof computer
///!  - monitoring      : read out environmental
///!                      data
///!  - buffer handling : check the fill level of
///!                      the buffers and switch
///!                      if necessary
///!  - data handling   : Identify event id, 
///!                      (Analyze data),
///!                      pack data
///! 
///! 


///! The actual "server" thread. Manage requests 
///  from the clients
///
///  This acts as global sync for all 
///  bytestreams.
///
///  # Arguments
///  
///  * address   : our server side address
///                we are listening on
///  * recv_bs   : A channel for receiving
///                binary payloads
///  
pub fn server(socket     : &zmq::Socket,
              recv_bs    : Receiver<Vec<u8>>,
              recv_ev_pl : Receiver<RBEventPayload>,
              cache_size : usize) {
 
  //let one_milli    = time::Duration::from_millis(1);
  //let mut message       = zmq::Message::new();
  //let mut response = zmq::Message::new();
  // a cache for the events from this specific board
  let mut event_cache : HashMap::<u32, RBEventPayload> = HashMap::new();

  // keep track of the "oldest" key
  let mut oldest_event_id : u32 = 0;

  // last moni packet
  let mut last_moni : Vec<u8> = Vec::new();
  // this works on 3 things in decreasing 
  // priority
  // 1) check if there is incoming event data
  // 3) check if a send request for cached event data
  //    has been made
  // 2) check if there is incoming monitoring data
  //    has been made
  let sock_timeout : i64 = 1;
  
  // How many events shall we receive through 
  // the channel before we try polling the 0MQ
  // socket?
  // FIXME - this might need to be 
  // configurable
  let recv_ev_per_poll : u8 = 10;
  let mut n_iter : u8 = 0;
  loop {
    let now        = time::Instant::now();
    // check for a new connection
    trace!("Server loop");
    match recv_ev_pl.recv() {
      Err(err) => {
        debug!("No event payload! {err}");
        continue;
      } // end err
      Ok(event)  => {
        if oldest_event_id == 0 {
          oldest_event_id = event.event_id;
        } //endif
        // store the event in the cache
        trace!("Adding RBEvent : {} to cache", event.event_id);
        event_cache.insert(event.event_id, event);   
        // keep track of the oldest event_id
        debug!("We have a cache size of {}", event_cache.len());
        if event_cache.len() > cache_size {
          event_cache.remove(&oldest_event_id);
          oldest_event_id += 1;
        } //endif
        n_iter += 1;
        if n_iter < recv_ev_per_poll {
          continue;
        } else {
          n_iter=0;
        }
      }// end Ok
    } // end match

    match recv_bs.recv() {
      Err(err) => debug!("Can not get bytestream payload, err {err}"),
      Ok(payload)  => {
        last_moni = payload;
      }// end Ok
    } // end match
    
    match socket.poll(zmq::PollEvents::POLLIN, sock_timeout) {
      Ok(0) => continue,
      Err(err) => warn!("0MQ socket poll failed! err {err}"),
      Ok(1) => {
        match socket.recv_bytes(zmq::DONTWAIT) {
          Err(err)  => {
            debug!("Can't receive over 0MQ, error {err}");
            continue;
          }, // end Err
          Ok(bytes) => {
            let tp = TofPacket::from_bytestream(&bytes, 0);
            match tp {
              Err(err) => {
                debug!("Got broken package! {:?}", err);
                let response = cmd::TofResponse::GeneralFail(0);
                socket.send(response.to_bytestream(), zmq::DONTWAIT);
                continue;
              },
              Ok(_) => ()
            } // end match
            let tp = tp.unwrap();
            let cmd_pk = cmd::TofCommand::from_tof_packet(&tp);
            match cmd_pk {
              None => {
                debug!("Don't understand command!");
                socket.send("[SRV] don't understand command", zmq::DONTWAIT);
                continue;
              },
              Some(cmd) => {
                match cmd {
                  TofCommand::RequestEvent(event_id) => {
                    debug!("Received request for event: {event_id}");
                    if let Some(event) = event_cache.remove(&event_id) {
                      socket.send(event.payload, zmq::DONTWAIT);
                    } else {
                      debug!{"Event {event_id} not found in cache!"};
                      let response = cmd::TofResponse::EventNotReady(event_id);
                      socket.send(response.to_bytestream(), zmq::DONTWAIT);
                      continue;
                    }
                  },
                  TofCommand::RequestMoni => {
                  },
                  _ => warn!("Currently only RequestEvent is implemented!")
                }
              }// end Some
            } // end match
          } // end Ok
        } //
      }// end ok
      Ok(_) => {
        warn!("0MQ broke it's contract. Not sure what to do. Continuig..");
        continue;
      }
    } // end poll = 1
  let time = now.elapsed().as_millis();
  println!("Server loop took {}", time);
  } // end loop
}


///! A monitoring thread, which communicates with the 
///  server program
fn monitoring(send_bs : Sender<Vec<u8>>) {
  //let mut now        = time::Instant::now();
  let heartbeat      = time::Duration::from_secs(HEARTBEAT);
  let mut rate: u32  = 0; 
  let mut bytestream = Vec::<u8>::new();
  bytestream.extend_from_slice(&rate.to_le_bytes());
  let mut packet         = GenericPacket::new(String::from("rate"),
                                              bytestream);
  loop {
   //if now.elapsed().as_secs() >= HEARTBEAT {
   //}
   let rate_query = get_trigger_rate();
   match rate_query {
     Ok(rate) => {
       debug!("Monitoring thread -> Rate: {rate}Hz ");
       bytestream = Vec::<u8>::new();
       bytestream.extend_from_slice(&rate.to_le_bytes());
       packet.update_payload(bytestream);
       let payload = packet.to_bytestream();
       match send_bs.send(payload) {
         Err(err) => {debug!("Issue sending payload {:?}", err)},
         Ok(_)    => {debug!("Send payload successfully!")}
       }
     }

     Err(_)   => {
       warn!("Can not send rate monitoring packet, register problem");
     }
   }
   thread::sleep(heartbeat);
  }
}


/// Read the data buffers when they are full and 
/// then send the stream over the channel to 
/// the thread dealing with it
///
/// # Arguments
///
///
fn read_data_buffers(bs_send     : Sender<Vec<u8>>,
                     buff_trip   : u32,
                     bar_a_op    : Option<Box<ProgressBar>>,
                     bar_b_op    : Option<Box<ProgressBar>>, 
                     switch_buff : bool) {
  let buf_a = BlobBuffer::A;
  let buf_b = BlobBuffer::B;
  let sleeptime = time::Duration::from_millis(1000);

  //let mut max_buf_a : u64 = 0;
  //let mut max_buf_b : u64 = 0;
  //let mut min_buf_a : u64 = 4294967295;
  //let mut min_buf_b : u64 = 4294967295;
  // let's do some work
  loop {
    //let a_occ = get_blob_buffer_occ(&buf_a).unwrap() as u64;
    //let b_occ = get_blob_buffer_occ(&buf_b).unwrap() as u64;
    //if a_occ > max_buf_a {
    //  max_buf_a = a_occ;
    //  println!("New MAX size for A {max_buf_a}");
    //}
    //if b_occ > max_buf_b  {
    //  max_buf_b = b_occ;
    //  println!("New MAX size for B {max_buf_b}");
    //}
    //if a_occ < min_buf_a {
    //  min_buf_a = a_occ;
    //  println!("New MIN size for A {min_buf_a}");
    //}
    //if b_occ < min_buf_b  {
    //  min_buf_b = b_occ;
    //  println!("New MIN size for B {min_buf_b}");
    //}
    thread::sleep(sleeptime);
    buff_handler(&buf_a,
                 buff_trip,
                 Some(&bs_send),
                 &bar_a_op, 
                 switch_buff); 
    buff_handler(&buf_b,
                 buff_trip,
                 Some(&bs_send),
                 &bar_b_op,
                 switch_buff); 
  }
}

/***** END THREAD IMPLEMENTATION ******/ 

fn main() {
  pretty_env_logger::init();

  let sparkle_heart         = vec![240, 159, 146, 150];
  let kraken                = vec![240, 159, 144, 153];
  let fish                  = vec![240, 159, 144, 159];
  let sparkles              = vec![226, 156, 168];

  // We know these bytes are valid, so we'll use `unwrap()`.
  let sparkle_heart    = String::from_utf8(sparkle_heart).unwrap();
  let kraken           = String::from_utf8(kraken).unwrap();
  let fish             = String::from_utf8(fish).unwrap();
  let sparkles         = String::from_utf8(sparkles).unwrap();


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

  // welcome banner!
  println!("-----------------------------------------------");
  println!(" ** Welcome to tof-kraken {} *****", kraken);
  println!(" .. TOF C&C and data acquistion suite");
  println!(" .. for the GAPS experiment {}", sparkle_heart);
  println!("-----------------------------------------------");
  println!(" => Running client for RB {}", rb_id);
  println!(" => RB had DNA {}", dna);
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
  let mut n_threads = 5;
  
  let (kill_run, run_gets_killed)    : (Sender<bool>, Receiver<bool>)       = channel();
  let (bs_send, bs_recv)             : (Sender<Vec<u8>>, Receiver<Vec<u8>>) = channel(); 
  let (moni_send, moni_recv)         : (Sender<Vec<u8>>, Receiver<Vec<u8>>) = channel(); 
  let (ev_pl_to_cache, ev_pl_from_builder) : 
      (Sender<RBEventPayload>, Receiver<RBEventPayload>) = channel();
  let (ev_pl_to_cmdr,  ev_pl_from_cache)   : 
    (Sender<Option<RBEventPayload>>, Receiver<Option<RBEventPayload>>) = channel();
  let (evid_to_cache, evid_from_cmdr)   : (Sender<u32>, Receiver<u32>) = channel();
  info!("Will start ThreadPool with {n_threads} threads");
  let workforce = ThreadPool::new(n_threads);
  
  // wait until we receive the 
  // rsponse from the server
  // This is all done by the runner now!
  //info!("Setting daq to idle mode");
  //match idle_drs4_daq() {
  //  Ok(_)    => info!("DRS4 set to idle:"),
  //  Err(_)   => panic!("Can't set DRS4 to idle!!")
  //}
  //thread::sleep(one_milli);
  //match setup_drs4() {
  //  Ok(_)    => info!("DRS4 setup routine complete!"),
  //  Err(_)   => panic!("Failed to setup DRS4!!")
  //}
  //
  //match reset_dma() {
  //  Ok(_)    => info!("DMA reset successful"),
  //  Err(_)   => {
  //    warn!("Can not reset DMA! Will run more aggressive reset procedure!");
  //  }
  //}
  thread::sleep(one_milli);
  
  if buff_trip != 66520576 {
    uio1_total_size = buff_trip as u64;
    uio2_total_size = buff_trip as u64;
  }
  
  // now we are ready to receive data 

  // set up some progress bars, so we 
  // can see what is going on 
  // this is optional
  // FIXME - feature?
  //let mut prog_op_a     : Option<&ProgressBar> = None;
  //let mut prog_op_b     : Option<&ProgressBar> = None;
  //let mut prog_op_ev    : Option<&ProgressBar> = None;
  let mut prog_op_a     : Option<Box<ProgressBar>>   = None; 
  let mut prog_op_b     : Option<Box<ProgressBar>>   = None;
  let mut prog_op_ev    : Option<Box<ProgressBar>>   = None;
  let mut multi_prog_op : Option<Box<MultiProgress>> = None;
 

  if show_progress {
    multi_prog_op = Some(Box::new(MultiProgress::new()));
    let floppy    = vec![240, 159, 146, 190];
    let floppy    = String::from_utf8(floppy).unwrap();
    let label_a   = String::from("Buff A");
    let label_b   = String::from("Buff B");
    let sty_a = ProgressStyle::with_template(TEMPLATE_BAR_A)
    .unwrap();
    //.progress_chars("##-");
    let sty_b = ProgressStyle::with_template(TEMPLATE_BAR_B)
    .unwrap();
    //.progress_chars("##-");
    let sty_ev = ProgressStyle::with_template(TEMPLATE_BAR_EV)
    .unwrap();
    //.progress_chars("##>");

    prog_op_a  = Some(Box::new(multi_prog_op
                               .as_mut()
                               .unwrap()
                               .add(ProgressBar::new(uio1_total_size)))); 
    prog_op_b  = Some(Box::new(multi_prog_op
                               .as_mut()
                               .unwrap()
                               .insert_after(&prog_op_a.as_mut().unwrap(), ProgressBar::new(uio2_total_size)))); 
    prog_op_ev = Some(Box::new(multi_prog_op
                               .as_mut()
                               .unwrap()
                               .insert_after(&prog_op_b.as_mut().unwrap(), ProgressBar::new(max_event as u64)))); 

    match prog_op_a {
      None => (),
      Some(ref bar) => {
        bar.set_message(label_a);
        bar.set_prefix(floppy.clone());
        bar.set_style(sty_a);
      }
    }
    match prog_op_b {
      None => (),
      Some(ref bar) => {
        bar.set_message(label_b);
        bar.set_prefix(floppy.clone());
        bar.set_style(sty_b);
      }
    }
    match prog_op_ev {
      None => (),
      Some(ref bar) => {
        bar.set_style(sty_ev);
        bar.set_prefix(sparkles.clone());
        bar.set_message("EVENTS");
      }
    }
  }
  // this thread deals with the bytestream and 
  // performs analysis or just sneds it over 
  // zmq
  //let pl_sender = ev_pl_send.clone();

  // Setup routines 
  // Start the individual worker threads
  // in meaningfull order
  // - higher level threads first, then 
  // the more gnarly ones.
  let moni_sender = moni_send.clone();
  workforce.execute(move || {
      monitoring(moni_sender);
  });
  workforce.execute(move || {
                    event_cache(ev_pl_from_builder,
                                ev_pl_to_cmdr,
                                evid_from_cmdr,
                                cache_size)
  });
  workforce.execute(move || {
                    event_payload_worker(&bs_recv, ev_pl_to_cache);
  });
  
  // this thread deals JUST with the data
  // buffers. It reads them and then 
  // passes on the data
  let rdb_sender = bs_send.clone();
  workforce.execute(move || {
    read_data_buffers(rdb_sender,
                      buff_trip,
                      prog_op_a,
                      prog_op_b,
                      switch_buff);
  });

  // create 0MQ sockedts
  let ctx = zmq::Context::new();
  let cmd_socket = ctx.socket(zmq::REP).expect("Unable to create 0MQ REP socket!");

  if !dont_listen {  
    info!("Will set up 0MQ REP socket at address {cmd_address}");
    cmd_socket.bind(&cmd_address);
    
    info!("0MQ REP socket listening at {cmd_address}");
    println!("Waiting for client to connect...");
    // block until we get a client
    let client_response = cmd_socket.recv_bytes(0).expect("Communication to client failed!");
    let resp =  String::from_utf8(client_response).expect("Got garbage response from client. If we start like this, I panic right away...");
    println!("Client connected! Response {resp}");
    let response = String::from("[MAIN] - connected");
    cmd_socket.send(response.as_bytes(), 0);
  } else {
    // if we are not listening to a C&C server, we have to kickstart
    // our run ourselves.
    let bar_clone = prog_op_ev.clone();
    //match prog_op_ev {
    //  None => (),
    //  Some(bar) => {
    //    &bar.suspend(|| { loop {}} );
    //  }
    //}
    workforce.execute(move || {
        runner(Some(max_event),
               None,
               None,
               None,
               bar_clone);
    });
  }

  // Now set up PUB socket 
  // The pub socket is always present, even in don't listen configuration
  // (Nobody is forced to listen to it, and it will just drop its data 
  // if it does not have any subscribers)
  let data_socket = ctx.socket(zmq::PUB).expect("Unable to create 0MQ PUB socket!");
  data_socket.bind(&data_address);
  info!("0MQ SUB socket bound to address {data_address}");

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
  let mut resp     : cmd::TofResponse;
  let executor = Commander::new(data_socket,
                                evid_to_cache,
                                ev_pl_from_cache);
  let mut run_active = false;
  loop {
    // query the command socket
    // this can block. The actual 
    // work is done by other stuff
    if dont_listen {
      continue;
    }
    let incoming = cmd_socket.recv_bytes(0);
    match incoming {
      Err(err) => {
        warn!("CMD socket error {err}");
        // sleep for a bit and see if it recovers
        thread::sleep(one_sec);
        continue;
      },
      Ok(_) => (),
    }
    let raw_command = incoming.unwrap();
    debug!("Raw command bytes : {:?}", raw_command);
    match TofCommand::from_bytestream(&raw_command,0) {
      Err(err) => {
        warn!("Can not decode Command! Err {:?}", err);
        warn!("Received {:?} ", raw_command);
        let resp = cmd::TofResponse::SerializationIssue(cmd::RESP_ERR_LEVEL_MEDIUM);
        cmd_socket.send(resp.to_bytestream(),0);
        continue;
      },
      Ok(c) => {
        let mut result = Ok(TofResponse::Unknown);

        // at this point, we have a valid command!
        info!("Received command {:?}", c);
        // intercept commands which require to spawn/kill 
        // threads
        match c {
          TofCommand::DataRunStart (max_event) => {
            // let's start a run. The value of the TofCommnad shall be 
            // nevents
            if run_active {
              warn!("Can not start a new run, stop the current first!");  
              result = Ok(TofResponse::GeneralFail(cmd::RESP_ERR_RUNACTIVE));
            } else {
              info!("Attempting to launch a new runner with {max_event} events!");
              let bar_clone = prog_op_ev.clone();
              workforce.execute(move || {
                  runner(Some(max_event as u64),
                         None,
                         None,
                         None,
                         //FIXME - maybe use crossbeam?
                         //Some(&run_gets_killed),
                         bar_clone);
              }); 
              run_active = true;
              result = Ok(TofResponse::Success(cmd::RESP_SUCC_FINGERS_CROSSED));
              cmd_socket.send(result.unwrap().to_bytestream(),0);
            }
          },
          TofCommand::DataRunEnd   => {
            if !run_active {
              warn!("Can not kill run, since there is currently none in progress!");
              result = Ok(TofResponse::GeneralFail(cmd::RESP_ERR_NORUNACTIVE));
              cmd_socket.send(result.unwrap().to_bytestream(),0);
            } else {
              warn!("Attempting to kill current run!");
              kill_run.send(true);
              result = Ok(TofResponse::Success(cmd::RESP_SUCC_FINGERS_CROSSED));
              cmd_socket.send(result.unwrap().to_bytestream(),0);
            }
          },
          _ => {
            // forward the rest to the executor
            result = executor.command(&c);
            match result {
              Err(err) => {
                warn!("Command Failed! Err {:?}", err);
                // FIXME - work on error codes
                resp = cmd::TofResponse::GeneralFail(cmd::RESP_ERR_UNEXECUTABLE);
                cmd_socket.send(resp.to_bytestream(),0);
              }
              Ok(r) =>  {
                cmd_socket.send(r.to_bytestream(),0);
              }
            }
          } // end all other commands
        } // end match
      }
    }
  } // end loop
} // end main

