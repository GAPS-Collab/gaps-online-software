/****
 *
 * Communications with the 
 * mastertrigger
 *
 */ 

// to measure the rate
use std::time::{Duration, Instant};
use std::thread;
use std::net::{UdpSocket, SocketAddr};
use std::sync::mpsc::Sender;
use crossbeam_channel as cbc; 

use std::io;

use tof_dataclasses::packets::TofPacket;
use tof_dataclasses::events::master_trigger::{read_daq, reset_daq};
use tof_dataclasses::events::MasterTriggerEvent;

//const MT_MAX_PACKSIZE   : usize = 4096;
const MT_MAX_PACKSIZE   : usize = 512;


/// Connect to MTB Utp socket
pub fn connect_to_mtb(mt_ip   : &str, 
                      mt_port : &usize) 
  ->io::Result<UdpSocket> {
  let mt_address = mt_ip.to_owned() + ":" + &mt_port.to_string();
  let local_port = "0.0.0.0:50100";
  let local_addrs = [
    SocketAddr::from(([0, 0, 0, 0], 50100)),
    SocketAddr::from(([0, 0, 0, 0], 50101)),
  ];
  //let local_socket = UdpSocket::bind(local_port);
  let local_socket = UdpSocket::bind(&local_addrs[..]);
  let mut socket : UdpSocket;
  match local_socket {
    Err(err)   => {
      error!("Can not create local UDP port for master trigger connection at {}!, err {}", local_port, err);
      return Err(err);
    }
    Ok(value)  => {
      info!("Successfully bound UDP socket for master trigger communcations to {}", local_port);
      socket = value;
      // this is not strrictly necessary, but 
      // it is nice to limit communications
      match socket.connect(&mt_address) {
        Err(err) => {
          error!("Can not connect to master trigger at {}, err {}", mt_address, err);
          return Err(err);
        }
        Ok(_)    => info!("Successfully connected to the master trigger at {}", mt_address)
      }
      return Ok(socket);
    }
  } // end match
}  


///
/// Communications with the master trigger
///
///
pub fn master_trigger(mt_ip   : &str, 
                      mt_port : usize,
                      glob_data_sink : &cbc::Sender<TofPacket>,
                      evid_sender : &cbc::Sender<MasterTriggerEvent>) {

  let mt_address = mt_ip.to_owned() + ":" + &mt_port.to_string();
 
  let mut socket = connect_to_mtb(&mt_ip, &mt_port).expect("Can not create local UDP socket for MTB connection!"); 
  //socket.set_nonblocking(true).unwrap();
  
  // we only allocate the buffer once
  // and reuse it for all operations
  let mut buffer = [0u8;MT_MAX_PACKSIZE];  
  
  //let mut event_cnt      = 0u32;
  let mut last_event_cnt = 0u32;
  let mut missing_evids  = 0usize;
  let mut event_missing  = false;
  let mut n_events       = 0usize;
  // these are the number of expected events
  // (missing included)
  let mut n_events_expected = 0usize;
  let mut n_paddles_expected : u32;
  let mut rate = 0f64;
  // for rate measurement
  let start = Instant::now();

  let mut next_beat = true;
  // limit polling rate to a maximum
  let max_rate = 200.0; // hz
    
  // reset the master trigger before acquisiton
  info!("Resetting master trigger");
  reset_daq(&socket, &mt_address);  
  // the event counter has to be reset before 
  // we connect to the readoutboards
  //reset_event_cnt(&socket, &mt_address); 
  let mut mt_event = read_daq(&socket, &mt_address, &mut buffer);
  let mut timeout = Instant::now();
  //let timeout = Duration::from_secs(5);
  info!("Starting MT event loop at {:?}", timeout);

  loop {
    // a heartbeat every 10 s
    let elapsed = start.elapsed().as_secs();
    if (elapsed % 10 == 0) && next_beat {
      println!("== == == == == == == == HEARTBEAT! {} seconds passed!", elapsed);
      rate = n_events as f64 / elapsed as f64;
      println!("==> {} events recorded, trigger rate: {:.3} Hz", n_events, rate);
      rate = n_events_expected as f64 / elapsed as f64;
      println!("==> -- expected rate {:.3} Hz", rate);   
      println!("== == == == == == == == END HEARTBEAT!");
      next_beat = false;
    } else if elapsed % 10 != 0 {
      next_beat = true;
    }
    if timeout.elapsed().as_secs() > 10 {
      drop(socket);
      socket = connect_to_mtb(&mt_ip, &mt_port).expect("Can not create local UDP socket for MTB connection!"); 
      timeout = Instant::now();
    }



    //info!("Next iter...");
    // limit the max polling rate
    
    //let milli_sleep = Duration::from_millis((1000.0/max_rate) as u64);
    //thread::sleep(milli_sleep);
    

    //info!("Done sleeping..."); 
    //match socket.connect(&mt_address) {
    //  Err(err) => panic!("Can not connect to master trigger at {}, err {}", mt_address, err),
    //  Ok(_)    => info!("Successfully connected to the master trigger at {}", mt_address)
    //}
    //  let received = socket.recv_from(&mut buffer);

    //  match received {
    //    Ok((size, addr)) => println!("Received {} bytes from address {}", size, addr),
    //    Err(err)         => {
    //      println!("Received nothing! err {}", err);
    //      continue;
    //    }
    //  } // end match
    
    // daq queue states
    // 0 - full
    // 1 - something
    // 2 - empty
    //if 0 != (read_register(&socket, &mt_address, 0x12, &mut buffer) & 0x2) {
    //if read_register(&socket, &mt_address, 0x12, &mut buffer) == 2 {
    //  trace!("No new information from DAQ");
    //  //reset_daq(&socket, &mt_address);  
    //  continue;
    //}
    
    //event_cnt = read_event_cnt(&socket, &mt_address, &mut buffer);
    //println!("Will read daq");
    mt_event = read_daq(&socket, &mt_address, &mut buffer);
    //println!("Got event");
    match mt_event {
      Err(err) => {
        trace!("Did not get new event, Err {err}");
        continue;
      }
      Ok(_)    => ()
    }
    let ev = mt_event.unwrap();
    if ev.event_id == last_event_cnt {
      trace!("Same event!");
      continue;
    }

    // sometimes, the counter will just read 0
    // throw these away. 
    // FIXME - there is actually an event with ctr 0
    // but not sure how to address that yet
    if ev.event_id == 0 {
      trace!("event 0 encountered! Continuing...");
      continue;
    }

    // FIXME
    if ev.event_id == 2863311530 {
      warn!("Magic event number! continuing! 2863311530");
      continue;
    }

    // we have a new event
    //println!("** ** evid: {}",event_cnt);
    
    // if I am correct, there won't be a counter
    // overflow for a 32bit counter in 99 days 
    // for a rate of 500Hz
    if ev.event_id < last_event_cnt {
      error!("Event counter id overflow! this cntr: {} last cntr: {last_event_cnt}!", ev.event_id);
      last_event_cnt = 0;
      continue;
    }
    
    if ev.event_id - last_event_cnt > 1 {
      let mut missing = ev.event_id - last_event_cnt;
      
      // FIXME
      if missing < 200 {
        missing_evids += missing as usize;
      } else {
        warn!("We missed too many event ids from the master trigger!");
        missing = 0;
      }
      //error!("We missed {} events!", missing);
      event_missing = true;
    }
    
    info!("Got new event id from master trigger {}",ev.event_id);
    match evid_sender.send(ev) {
      Err(err) => trace!("Can not send event, err {err}"),
      Ok(_)    => ()
    }
    last_event_cnt = ev.event_id;
    n_events += 1;
    n_events_expected = n_events + missing_evids;

    if n_events % 1000 == 0 {
      let pk = TofPacket::new();
      
    }

    let elapsed = start.elapsed().as_secs();
    // measure rate every 100 events
    if n_events % 10 == 0 {
      rate = n_events as f64 / elapsed as f64;
      println!("==> {} events recorded, trigger rate: {:.3} Hz", n_events, rate);
      rate = n_events_expected as f64 / elapsed as f64;
      println!("==> -- expected rate {:.3} Hz", rate);   
    } 
    // end new event


  } // end loop
}

