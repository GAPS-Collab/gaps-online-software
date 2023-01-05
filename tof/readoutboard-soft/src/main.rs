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

use crate::api::*;
use crate::control::*;
use crate::memory::{BlobBuffer,
                    UIO1_MAX_OCCUPANCY,
                    UIO2_MAX_OCCUPANCY,
                    UIO1_MIN_OCCUPANCY,
                    UIO2_MIN_OCCUPANCY};
use tof_dataclasses::threading::ThreadPool;
use tof_dataclasses::packets::value_packet::ValuePacket;
use tof_dataclasses::events::blob::RBEventPayload;

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

const HEARTBEAT : u64 = 5; // heartbeat in s

/**********************************************
 * Threads:
 *
 * - server          : comms with tof computer
 * - monitoring      : read out environmental
 *                     data
 * - buffer handling : check the fill level of
 *                     the buffers and switch
 *                     if necessary
 * - data handling   : Identify event id, 
 *                     (Analyze data),
 *                     pack data
 *
 ********************************************/

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
              recv_ev_pl : Receiver<RBEventPayload>) {
  
  let one_milli   = time::Duration::from_millis(1);
  let mut message = zmq::Message::new();
  
  // we basically wait for incoming stuff and 
  // send it over the network unchecked for now
  loop {
    // check for a new connection
    println!("Server loop");
    match recv_bs.recv() {
      Err(_) => (),
      Ok(payload)  => {
        println!("Received next payload!");
        message = zmq::Message::from_slice(&payload
                                          .as_slice());
        match socket.send(message, zmq::DONTWAIT) {
          Err(err) => debug!("Unable to send monitoring payload over 0MQ socket! err {err}"),
          Ok(_)    => debug!("Payload sent!")

        }
      }// end Ok
    } // end match
    println!("checking events.. ");
    match recv_ev_pl.recv() {
      Err(_) => {continue;}
      Ok(payload)  => {
        println!("Received next RBEvent!");
        message = zmq::Message::from_slice(&payload
                                           .payload
                                          .as_slice());
        match socket.send(message, zmq::DONTWAIT) {
          Err(err) => debug!("Unable to send event payload over 0MQ socket! err {err}"),
          Ok(_)    => debug!("Payload sent!")
        }
      }// end Ok
    } // end match
      //
      //
    thread::sleep(one_milli);
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
  let mut packet         = ValuePacket::new(String::from("rate"),
                                        bytestream);
  loop {
   //if now.elapsed().as_secs() >= HEARTBEAT {
   //}
   let rate_query = get_trigger_rate();
   match rate_query {
     Ok(rate) => {
       println!("Monitoring thread got {rate}");
       bytestream = Vec::<u8>::new();
       bytestream.extend_from_slice(&rate.to_le_bytes());
       packet.update_payload(bytestream);
       let payload = packet.to_bytestream();
       //message = zmq::Message::from_slice(&packet
       //                                   .to_bytestream()
       //                                   .as_slice());
       send_bs.send(payload);
     }

     Err(_)   => {
       warn!("Can not send rate monitoring packet, register problem");
     }
   }
   thread::sleep(heartbeat);
  }
}


///! Read the data buffers when they are full and 
///  then send the stream over the channel to 
///  the thread dealing with it
fn read_data_buffers(bs_send     : Sender<Vec<u8>>,
                     buff_trip   : u32,
                     bar_a       : Option<&ProgressBar>,
                     bar_b       : Option<&ProgressBar>, 
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
                 bar_a, 
                 switch_buff); 
    buff_handler(&buf_b,
                 buff_trip,
                 Some(&bs_send),
                 bar_b,
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
    IpAddr::V6(ip) => panic!("Currently, we do not support IPV6!")

      //  Ipv4Addr(address) => address_ip( 
  }
  //// the port will be 38830 + board id
  //address_ip += "10.0.1.1";
  let port = 38830 + get_board_id().unwrap();
  let address : String = address_ip + ":" + &port.to_string();
  //info!("Will use ip address {address}");

  // welcome banner!
  println!("-----------------------------------------------");
  println!(" ** Welcome to tof-kraken {} *****", kraken);
  println!(" .. TOF C&C and data acquistion suite");
  println!(" .. for the GAPS experiment {}", sparkle_heart);
  println!("-----------------------------------------------");
  println!(" => Running client for RB {}", rb_id);
  println!(" => RB had DNA {}", dna);
  println!(" => Will bind local ZMQ socket to {}", address);
  println!("-----------------------------------------------");
  println!("");                             
                                            
  let args = Args::parse();                   
  let buff_trip     = args.buff_trip;         
  let switch_buff   = args.switch_buffers;    
  let max_event     = args.nevents;
  let show_progress = args.show_progress;

  let mut uio1_total_size = (UIO1_MAX_OCCUPANCY - UIO1_MIN_OCCUPANCY) as u64;
  let mut uio2_total_size = (UIO2_MAX_OCCUPANCY - UIO2_MIN_OCCUPANCY) as u64;

  if (buff_trip > uio1_total_size as u32 ) || (buff_trip > uio2_total_size as u32) {
    println!("Invalid value for --buff-trip. Panicking!");
    panic!("Tripsize of {buff_trip} exceeds buffer sizes of A : {uio1_total_size} or B : {uio2_total_size}");
  }

  info!("Will set buffer trip size to {buff_trip}");
  let dont_listen = args.dont_listen;
  //info!("Will listen to the the tof computer");


  // some pre-defined time units for 
  // sleeping
  let two_seconds = time::Duration::from_millis(2000);
  let one_milli   = time::Duration::from_millis(1);
  
  
  // threads and inter-thread communications
  // We have
  // * server thread
  // * buffer reader thread
  // * data analysis/sender thread
  // * monitoring thread
  let mut n_threads = 3;
  if !dont_listen {
    n_threads += 1
  }
  let (bs_send, bs_recv)       : (Sender<Vec<u8>>, Receiver<Vec<u8>>) = channel(); 
  let (moni_send, moni_recv)   : (Sender<Vec<u8>>, Receiver<Vec<u8>>) = channel(); 
  let (ev_pl_send, ev_pl_recv) : (Sender<RBEventPayload>, Receiver<RBEventPayload>) = channel();
  let workforce = ThreadPool::new(n_threads);
  



  // wait until we receive the 
  // rsponse from the server

  info!("Setting daq to idle mode");
  match idle_drs4_daq() {
    Ok(_)    => info!("DRS4 set to idle:"),
    Err(_)   => panic!("Can't set DRS4 to idle!!")
  }
  thread::sleep(one_milli);
  match setup_drs4() {
    Ok(_)    => info!("DRS4 setup routine complete!"),
    Err(_)   => panic!("Failed to setup DRS4!!")
  }
  
  reset_dma().unwrap();
  thread::sleep(one_milli);
  
  // now we are ready to receive data 

  // set up some progress bars, so we 
  // can see what is going on 
  // this is optional
  // FIXME - feature?
  let mut prog_op_a  : Option<&ProgressBar> = None;
  let mut prog_op_b  : Option<&ProgressBar> = None;
  let mut prog_op_ev : Option<&ProgressBar> = None;
  let multi_bar = MultiProgress::new();
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

  if buff_trip != 66520576 {
    uio1_total_size = buff_trip as u64;
    uio2_total_size = buff_trip as u64;
  }
  let bar_a  : ProgressBar = multi_bar.add(ProgressBar::new(uio1_total_size)); 
  let bar_b  : ProgressBar = multi_bar.insert_after(&bar_a, ProgressBar::new(uio2_total_size));
  let bar_ev : ProgressBar = multi_bar.insert_after(&bar_b, ProgressBar::new(max_event as u64));         
  bar_a.set_message(label_a);
  bar_b.set_message(label_b);
  bar_ev.set_message("EVENTS");
  bar_a.set_prefix(floppy.clone());
  bar_b.set_prefix(floppy.clone());
  bar_ev.set_prefix(sparkles.clone());
  bar_a.set_style(sty_a);
  bar_b.set_style(sty_b);
  bar_ev.set_style(sty_ev);
  prog_op_a  = Some(&bar_a);
  prog_op_b  = Some(&bar_b);
  prog_op_ev = Some(&bar_ev); 
  
  if !show_progress {
    prog_op_ev = None;
    prog_op_a  = None;
    prog_op_b  = None;
  }
  // this thread deals with the bytestream and 
  // performs analysis or just sneds it over 
  // zmq
  //let mut sock_op : Option<&zmq::Socket>;
  //if !dont_listen {
  //  sock_op = Some(&socket);
  //} else {
  //  sock_op = None;
  //}
  let pl_sender = ev_pl_send.clone();
  workforce.execute(move || {
                    event_payload_worker(&bs_recv, pl_sender);
  });
  
  // this thread deals JUST with the data
  // buffers. It reads them and then 
  // passes on the data
  let rdb_sender = bs_send.clone();
  if !show_progress{
    workforce.execute(move || {
      read_data_buffers(rdb_sender,
                        buff_trip,
                        None,
                        None,
                        switch_buff);
    });
  } else {
    workforce.execute(move || {
      read_data_buffers(rdb_sender,
                        buff_trip,
                        //prog_op_a,
                        //prog_op_b, 
                        Some(&bar_a),
                        Some(&bar_b),
                        switch_buff); 
    });
  }
  if !dont_listen {
    let moni_sender = moni_send.clone();
    workforce.execute(move || {
      monitoring(moni_sender);
    });
    
    debug!("Will set up zmq socket at address {address}");
    let ctx = zmq::Context::new();
    let socket = ctx.socket(zmq::REP).expect("Unable to create 0MQ REP socket!");
    socket.bind(&address);
    
    let mut message = zmq::Message::new();

    debug!("... done");
    info!("0MQ socket listening at {address}");
    println!("Waiting for client to connect");
    let client_connected = socket.recv(&mut message,0);
    let txt = message.as_str().unwrap();
    println!("Got a client connected!");
    println!("Received {txt}");
    let response = String::from("[MAIN] - connected");
    message = zmq::Message::from(&response);
    //socket.send(message,zmq::DONTWAIT);
    println!("Executing sender thread!");
    workforce.execute(move || {
                      server(&socket, 
                             moni_recv,
                             ev_pl_recv);
    });
  }
  info!("Starting daq!");
  match start_drs4_daq() {
    Ok(_)    => info!(".. successful!"),
    Err(_)   => panic!("DRS4 start failed!")
  }

  // let go for a few seconds to get a 
  // rate estimate
  //println!("getting rate estimate..");
  thread::sleep(two_seconds);
  let rate = get_trigger_rate().unwrap();
  info!("Current trigger rate: {rate}Hz");
  // the trigger rate defines at what intervals 
  // we want to print out stuff
  // let's print out something apprx every 2
  // seconds
  let n_evts_print : u64 = 2*rate as u64;

  // event loop
  let mut evt_cnt          : u32;
  let mut last_evt_cnt     : u32 = 0;

  let mut n_events         : u64 = 0;

  let mut skipped_events   : u64 = 0;
  let mut delta_events     : u64;

  let mut first_iter       = true;

  //let bar_a  = multi_bar.add(setup_progress_bar(String::from("buff A"), UIO1_TRIP as u64, String::from(TEMPLATE_BAR_A)));  
  //let bar_b  = multi_bar.insert_after(&bar_a,setup_progress_bar(String::from("buff B"), UIO2_TRIP as u64, String::from(TEMPLATE_BAR_B))); 
  loop {
    evt_cnt = get_event_count().unwrap();
    if first_iter {
      last_evt_cnt = evt_cnt;
      first_iter = false;
    }
    if evt_cnt == last_evt_cnt {
      thread::sleep(one_milli);
      continue;
    }
    // let's do some work
    //buf_a_start = buff_handler(&buf_a, buf_a_start, Some(&bs_send), Some(&bar_a)); 
    //buf_b_start = buff_handler(&buf_b, buf_b_start, Some(&bs_send), Some(&bar_b)); 
    
    delta_events = (evt_cnt - last_evt_cnt) as u64;
    if delta_events > 1 {
      skipped_events += delta_events;
    }
    
    n_events += 1;
    match prog_op_ev {
      None => (),
      Some(bar) => {
        bar.inc(delta_events);   
      }
    }
    // exit loop on n event basis
    if n_events > max_event {
      idle_drs4_daq();
      println!("We skipped {skipped_events} events");
      thread::sleep(one_milli);
      match prog_op_ev {
        None => (),
        Some(bar) => {
          bar.finish();
        }
      }
      break;
    }

    //if n_events % n_evts_print == 0 {
    //  println!("Current event count {n_events}");
    //  println!("We skipped {skipped_events} events");
    //}
    
    //println!("Got {evt_cnt} event!");
    last_evt_cnt = evt_cnt;
  }
} // end main
  //

