mod registers;
mod memory;
mod control;
mod api;

use std::{thread, time};
use std:: {sync::mpsc::Sender,
           sync::mpsc::Receiver,
           sync::mpsc::channel};
use std::net::IpAddr;
use indicatif::MultiProgress;

use local_ip_address::local_ip;

use crate::api::*;
use crate::control::*;
use crate::memory::BlobBuffer;
use crate::registers::{UIO1_TRIP, UIO2_TRIP};
use tof_dataclasses::threading::ThreadPool;
use tof_dataclasses::packets::value_packet::ValuePacket;

extern crate pretty_env_logger;
#[macro_use] extern crate log;

/// Non-register related constants
const TEMPLATE_BAR_A  : &str = "[{elapsed_precise}] {bar:60.blue/white} {pos:>7}/{len:7} {msg}";
const TEMPLATE_BAR_B  : &str = "[{elapsed_precise}] {bar:60.orange/white} {pos:>7}/{len:7} {msg}";
const TEMPLATE_BAR_EV : &str = "[{elapsed_precise}] {bar:60.red/white} {pos:>7}/{len:7} {msg}";

const HEARTBEAT : u64 = 60; // heartbeat in s

///! A monitoring thread, which communicates with the 
///  server program
fn monitoring(socket : &zmq::Socket) {
  //let mut now        = time::Instant::now();
  let heartbeat      = time::Duration::from_secs(HEARTBEAT);
  let mut rate: u32  = 0; 
  let mut bytestream = Vec::<u8>::new();
  bytestream.extend_from_slice(&rate.to_le_bytes());
  let mut packet         = ValuePacket::new(String::from("rate"),
                                        bytestream);
  let mut message : zmq::Message;
  loop {
   //if now.elapsed().as_secs() >= HEARTBEAT {
   //}
   let rate_query = get_trigger_rate();
   match rate_query {
     Ok(rate) => {
       bytestream = Vec::<u8>::new();
       bytestream.extend_from_slice(&rate.to_le_bytes());
       packet.update_payload(bytestream);
       message = zmq::Message::from_slice(&packet
                                          .to_bytestream()
                                          .as_slice());
       socket.send_msg(message,0);
     }

     Err(_)   => {
       warn!("Can not send rate monitoring packet, register problem");
     }
   }
   thread::sleep(heartbeat);
  }
}


fn main() {
  pretty_env_logger::init();
  // some pre-defined time units for 
  // sleeping
  let two_seconds = time::Duration::from_millis(2000);
  let one_milli   = time::Duration::from_millis(1);

  info!("Setting up zmq socket");
  let mut address_ip = String::from("tcp://");

  // this is currently not needed, since 
  // we are using the server/client setup wher
  // this is the client
  //let this_board_ip = local_ip().unwrap();
  //match this_board_ip {
  //  IpAddr::V4(ip) => address_ip += &ip.to_string(),
  //  IpAddr::V6(ip) => panic!("Currently, we do not support IPV6!")

  //    //  Ipv4Addr(address) => address_ip( 
  //}
  // the port will be 38830 + board id
  address_ip += "10.0.1.1";
  let port = 38830 + get_board_id().unwrap();
  let address : String = address_ip + &port.to_string();
  info!("Will use ip address {address}");
  
  let ctx = zmq::Context::new();
  let socket = ctx.socket(zmq::REQ).unwrap();
  socket.connect(&address);

  // wait until we receive the 
  // rsponse from the server
  


  info!("Setting daq to idle mode");
  match idle_drs4_daq() {
    Ok(_)    => info!("DRS4 set to idle:"),
    Err(err) => panic!("Can't set DRS4 to idle!!")
  }
  thread::sleep(one_milli);
  match setup_drs4() {
    Ok(_)    => info!("DRS4 setup routine complete!"),
    Err(err) => panic!("Failed to setup DRS4!!")
  }

   
  
  // get the current cache sizes
  let buf_a = BlobBuffer::A;
  let buf_b = BlobBuffer::B;
  reset_dma().unwrap();
  thread::sleep(one_milli);
  let mut buf_a_start = get_blob_buffer_occ(&buf_a).unwrap();
  let mut buf_b_start = get_blob_buffer_occ(&buf_b).unwrap();
  info!("We got start values for the blob buffers at {buf_a_start} and {buf_b_start}");
  // now we are ready to receive data 
  info!("Starting daq!");
  match start_drs4_daq() {
    Ok(_)    => info!(".. successful!"),
    Err(err) => panic!("DRS4 start failed!")
  }

  // let go for a few seconds to get a 
  // rate estimate
  println!("getting rate estimate..");
  thread::sleep(two_seconds);
  let mut rate = get_trigger_rate().unwrap();
  println!("Running at a trigger rate of {rate} Hz");
  // the trigger rate defines at what intervals 
  // we want to print out stuff
  // let's print out something apprx every 2
  // seconds
  let n_evts_print : u64 = 2*rate as u64;

  // event loop
  let mut evt_cnt          : u32 = 0;
  let mut last_evt_cnt     : u32 = 0;

  let mut n_events         : u64 = 0;

  let mut skipped_events   : u64 = 0;
  let mut delta_events     : u64 = 0;

  let mut first_iter       = true;
  
  // acquire this many events
  let max_event : u64 = 10000;

  // sizes of the buffers

  // set up some progress bars, so we 
  // can see what is going on 
  let multi_bar = MultiProgress::new();
  let bar_a  = multi_bar.add(setup_progress_bar(String::from("buff A"), UIO1_TRIP as u64, String::from(TEMPLATE_BAR_A)));  
  let bar_b  = multi_bar.insert_after(&bar_a,setup_progress_bar(String::from("buff B"), UIO2_TRIP as u64, String::from(TEMPLATE_BAR_B))); 
  let mut bar_ev = multi_bar.insert_after(&bar_b,setup_progress_bar(String::from("events"), max_event, String::from(TEMPLATE_BAR_EV)));         
  let (bs_send, bs_recv): (Sender<Vec<u8>>, Receiver<Vec<u8>>) = channel(); 
  // it's a dual core cpu. Let's have 2 workers only, but make them very 
  // busy for now. Later on, if it gets more complicated, we can hire more
  let workforce = ThreadPool::new(2);
  workforce.execute(move || {
                    bytestream_worker(&bs_recv, &socket);
  });
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
    buf_a_start = buff_handler(&buf_a, buf_a_start, Some(&bs_send), Some(&bar_a)); 
    buf_b_start = buff_handler(&buf_b, buf_b_start, Some(&bs_send), Some(&bar_b)); 
    

    delta_events = (evt_cnt - last_evt_cnt) as u64;
    if delta_events > 1 {
      skipped_events += delta_events;
    }
    
    n_events += 1;
    bar_ev.inc(delta_events);   
    // exit loop on n event basis
    if n_events > max_event {
      idle_drs4_daq();
      println!("We skipped {skipped_events} events");
      thread::sleep(one_milli);
      bar_ev.finish();
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

