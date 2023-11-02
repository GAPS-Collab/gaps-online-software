use std::error::Error;
use std::time::{Duration, Instant};
use std::fmt;
use std::{fs::File, path::Path};
use std::io::{self, BufReader};
use std::net::{IpAddr, Ipv4Addr};
use std::collections::HashMap;
use std::net::{UdpSocket, SocketAddr};
use crossbeam_channel::Receiver;
use zmq;
use colored::{Colorize, ColoredString};

use serde_json::Value;

use log::Level;
use macaddr::MacAddr6;
use netneighbours::get_mac_to_ip_map;
use crossbeam_channel as cbc; 


use tof_dataclasses::DsiLtbRBMapping;
use tof_dataclasses::constants::NWORDS;
use tof_dataclasses::calibrations::RBCalibrations;
use tof_dataclasses::packets::{TofPacket,
                               PacketType};
use tof_dataclasses::monitoring::MtbMoniData;
use tof_dataclasses::commands::RBCommand;
use tof_dataclasses::events::MasterTriggerEvent;
use tof_dataclasses::events::master_trigger::{read_daq,
                                              read_rate,
                                              reset_daq,
                                              read_register,
                                              read_register_multiple,
                                              read_daq_word,
                                              daq_word_available,
                                              read_adc_temp_and_vccint,
                                              read_adc_vccaux_and_vccbram};

use tof_dataclasses::errors::MasterTriggerError;

pub const MT_MAX_PACKSIZE   : usize = 1024;
pub const DATAPORT : u32 = 42000;

/// Read the complete event of the MTB
///
pub fn read_mtb_event(socket  : &UdpSocket,
                      address : &str,
                      buffer  : &mut [u8;MT_MAX_PACKSIZE]) -> Result<MasterTriggerEvent, MasterTriggerError> {
  let mut mte = MasterTriggerEvent::new(0,0);
  let ntries = 100;
  let mut n_daq_words : u32;
  for _ in 0..ntries {
    // 0x13 is MT.EVENT_QUEUE>SIZE
    match read_register(socket, address, 0x13 , buffer) {
      Err(err) => {
        error!("Timeout in read_register for MTB!");
        continue;
      },
      Ok(_n_words) => {
        n_daq_words = _n_words >> 16 as u16;
        println!("Got n_daq_words {n_daq_words}");
        n_daq_words = 2;
        let foo = daq_word_available(socket, address, buffer).unwrap_or(false);
        println!("{foo}");
        loop {
          match read_register_multiple(socket, address, 0x11, buffer, n_daq_words as usize) {
            Err(err) => {
              error!("Can't read register, err {err}");
              continue;
            }
            Ok(data) => {
              for k in data.iter() {
                println!("Got {k}");
              }
              break;
            }
          }
        }
        //for _ in 0..n_daq_words {
        //  match read_daq_word(socket, address, buffer,1) {
        //    Err(err) => {},
        //    Ok(foo)  => {
        //      println!("Got word {foo}");
        //    }
        //  }
        //}
        break;
      }
    }
  }
  Ok(mte)
}


/// Connect to MTB Utp socket
///
/// This will try a number of options to bind 
/// to the local port.
/// 
/// # Arguments 
///
/// * mtb_ip    : IP Adress of the MTB
/// * mtb_port  : Port of the MTB
///
pub fn connect_to_mtb(mt_address : &String) 
  ->io::Result<UdpSocket> {
  let local_port = "0.0.0.0:50100";
  let local_addrs = [
    SocketAddr::from(([0, 0, 0, 0], 50100)),
    SocketAddr::from(([0, 0, 0, 0], 50101)),
    SocketAddr::from(([0, 0, 0, 0], 50102)),
  ];
  //let local_socket = UdpSocket::bind(local_port);
  let local_socket = UdpSocket::bind(&local_addrs[..]);
  let socket : UdpSocket;
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
      match socket.set_read_timeout(Some(Duration::from_millis(1))) {
        Err(err) => error!("Can not set read timeout for Udp socket! Error {err}"),
        Ok(_)    => ()
      }
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

/// Obtain monitoring data from the MTB.
///
/// # Arguments"
///
/// * mtb_address   : ip + port of the master trigger
/// * moni          : preallocated struct to hold monitoring 
///                   data
pub fn monitor_mtb(mtb_address : &String,
                   mtb_moni    : &mut MtbMoniData) {
  let socket = connect_to_mtb(&mtb_address); 
  let mut buffer = [0u8;MT_MAX_PACKSIZE];  
  match socket {
    Err(err) => {error!("Can not connect to MTB, error {err}")},
    Ok(sock) => {
      match read_rate(&sock, &mtb_address, &mut buffer) {
        Err(err) => {
          error!("Unable to obtain MT rate information! error {err}");
        }
        Ok(rate) => {
          info!("Got MTB rate of {rate}");
          mtb_moni.rate = rate as u16;
        }
      } // end match
      match read_adc_vccaux_and_vccbram(&sock, &mtb_address, &mut buffer) {
        Err(err) => {
          error!("Unable to obtain MT VCCAUX and VCCBRAM! error {err}");
        }
        Ok(values) => {
          mtb_moni.fpga_vccaux  = values.0;
          mtb_moni.fpga_vccbram = values.1; 
        }
      }
      match read_adc_temp_and_vccint(&sock, &mtb_address, &mut buffer) {
        Err(err) => {
          error!("Unable to obtain MT VCCAUX and VCCBRAM! error {err}");
        }
        Ok(values) => {
          mtb_moni.fpga_temp    = values.0;
          mtb_moni.fpga_vccint  = values.1; 
        }
      }
    } // end OK
  } // end match
}

/// Communications with the master trigger over Udp
///
/// The master trigger can send packets over the network.
/// These packets contain timestamps as well as the 
/// eventid and a hitmaks to identify which LTBs have
/// participated in the trigger.
/// The packet format is described
/// [here](https://gitlab.com/ucla-gaps-tof/firmware/-/tree/develop/)
///
/// # Arguments
///
/// * mt_ip       : ip address of the master trigger, most likely 
///                 something like 10.0.1.10
/// * mt_port     : 
///
/// * sender_rate : 
/// 
/// * 
///
/// * verbose     : Print "heartbeat" output 
///
pub fn master_trigger(mt_ip          : &str, 
                      mt_port        : usize,
                      dsi_j_mapping  : &DsiLtbRBMapping,
                      sender_rate    : &cbc::Sender<u32>,
                      evid_sender    : &cbc::Sender<MasterTriggerEvent>,
                      rb_request_tp  : &cbc::Sender<TofPacket>,
                      verbose        : bool) {

  let mt_address = mt_ip.to_owned() + ":" + &mt_port.to_string();
 
  let mut socket = connect_to_mtb(&mt_address).expect("Can not create local UDP socket for MTB connection!"); 
  //socket.set_nonblocking(true).unwrap();
  // we only allocate the buffer once
  // and reuse it for all operations
  let mut buffer = [0u8;MT_MAX_PACKSIZE];  
  
  //let mut event_cnt      = 0u32;
  let mut last_event_cnt = 0u32;
  let mut missing_evids  = 0usize;
  //let mut event_missing  = false;
  let mut n_events       = 0usize;
  // these are the number of expected events
  // (missing included)
  let mut n_events_expected = 0usize;
  //let mut n_paddles_expected : u32;
  let mut rate : f64;
  // for rate measurement
  let start = Instant::now();

  let mut next_beat = true;
  
  // FIXME - this is a good idea
  // limit polling rate to a maximum
  //let max_rate = 200.0; // hz
    
  // reset the master trigger before acquisiton
  info!("Resetting master trigger");
  match reset_daq(&socket, &mt_address) {
    Err(err) => error!("Can not reset DAQ, error {err}"),
    Ok(_)    => ()
  }
  // the event counter has to be reset before 
  // we connect to the readoutboards
  //reset_event_cnt(&socket, &mt_address); 
  let mut ev : MasterTriggerEvent;// = read_daq(&socket, &mt_address, &mut buffer);
  let mut timeout = Instant::now();
  //let timeout = Duration::from_secs(5);
  info!("Starting MT event loop at {:?}", timeout);
  let mut timer = Instant::now();


  loop {
    // a heartbeat every 10 s
    let elapsed = start.elapsed().as_secs();
    if (elapsed % 10 == 0) && next_beat {
      rate = n_events as f64 / elapsed as f64;
      let expected_rate = n_events_expected as f64 / elapsed as f64; 
      if verbose {
        println!("== == == == == == == == MT HEARTBEAT! {} seconds passed!", elapsed);
        println!("==> {} events recorded, trigger rate: {:.3} Hz", n_events, rate);
        println!("==> -- expected rate {:.3} Hz", expected_rate);   
        println!("== == == == == == == == END HEARTBEAT!");
      }
      next_beat = false;
    } else if elapsed % 10 != 0 {
      next_beat = true;
    }
    if timeout.elapsed().as_secs() > 10 {
      drop(socket);
      socket = connect_to_mtb(&mt_address).expect("Can not create local UDP socket for MTB connection!"); 
      timeout = Instant::now();
    }
    if timer.elapsed().as_secs() > 10 {
      match read_rate(&socket, &mt_address, &mut buffer) {
        Err(err) => {
          error!("Unable to obtain MT rate information! error {err}");
          continue;
        }
        Ok(rate) => {
          info!("Got rate from MTB {rate}");
          match sender_rate.try_send(rate) {
            Err(err) => error!("Can't send rate, error {err}"),
            Ok(_)    => ()
          }
        }
      }
      timer = Instant::now();
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
    //mt_event = read_daq(&socket, &mt_address, &mut buffer);
    //println!("Got event");
    //read_mtb_event(&socket, &mt_address, &mut buffer);
    match read_daq(&socket, &mt_address, &mut buffer) {
      Err(err) => {
        trace!("Did not get new event, Err {err}");
        continue;
      }
      Ok(new_event) => {
        ev = new_event; 
      }
    }
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
      //continue;
    }

    // FIXME
    if ev.event_id == 2863311530 {
      warn!("Magic event number! continuing! 2863311530");
      //continue;
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
      let missing = ev.event_id - last_event_cnt;
      error!("We missed {missing} eventids"); 
      // FIXME
      if missing < 200 {
        missing_evids += missing as usize;
      } else {
        warn!("We missed too many event ids from the master trigger!");
        //missing = 0;
      }
      //error!("We missed {} events!", missing);
      //event_missing = true;
    }
    let request_enabled = false; 
    if request_enabled {
      trace!("Got new event id from master trigger {}",ev.event_id);
      //println!("MTE {}", ev);
      let hits = ev.get_dsi_j_ch_for_triggered_ltbs();
      //println!("HITS {:?}", hits);
      let mut rbs_ch = HashMap::<u8, Vec<u8>>::new();
      for h in hits.iter() {
        // h is dsi,j, ch
        if !dsi_j_mapping.contains_key(&h.0) {
          error!("Don't have RB connection information for {:?}.", h);
          continue;
        }
        if !dsi_j_mapping[&h.0].contains_key(&h.1) {
          error!("Don't have RB connection information for {:?}.", h);
          continue;
        }
        if !dsi_j_mapping[&h.1].contains_key(&h.2) {
          error!("Don't have RB connection information for {:?}.", h);
          continue;
        }
        let rb  = dsi_j_mapping[&h.0][&h.1][&h.2];
        if rbs_ch.contains_key(&rb.0) {
          // unwrap is fine, bc we just checked if 
          // the key exists
          rbs_ch.get_mut(&rb.0).unwrap().push(rb.1);
        } else {
          rbs_ch.insert(rb.0, Vec::<u8>::new());
        }
      }
      //println!("RBS CH KEYS {:?}", rbs_ch.keys());
      for k in rbs_ch.keys() { 
        let mut rb_cmd = RBCommand::new();
        rb_cmd.command_code = RBCommand::REQUEST_EVENT;
        rb_cmd.payload = ev.event_id;
        rb_cmd.rb_id   = *k;
        for ch in rbs_ch[&k].iter() {
          println!("{}", ch);
          rb_cmd.channel_mask = rb_cmd.channel_mask | 2u8.pow((ch -1).into());
        }    
        let request_pk = TofPacket::from(&rb_cmd);
        match rb_request_tp.send(request_pk) {
          Ok(_) => {},
          Err(err) => {
            error!("Unable to send request packet to rb {} for event {}! Error {err}", rb_cmd.rb_id, rb_cmd.payload);
          }
        }
      } 
    }
    match evid_sender.send(ev) {
      Err(err) => trace!("Can not send event, err {err}"),
      Ok(_)    => {
        //println!("==> EVID {} master trigger event sent!", ev.event_id);
      }
    }
    last_event_cnt = ev.event_id;
    n_events += 1;
    n_events_expected = n_events + missing_evids;

    //if n_events % 1000 == 0 {
      //let pk = TofPacket::new();
      //error!("Sending of mastertrigger packets down the global data sink not supported yet!");
    //}

    let elapsed = start.elapsed().as_secs();
    // measure rate every 100 events
    if n_events % 1000 == 0 {
      rate = n_events as f64 / elapsed as f64;
      if verbose {
        println!("==> [MASTERTRIGGER] {} events recorded, trigger rate: {:.3} Hz", n_events, rate);
      }
      rate = n_events_expected as f64 / elapsed as f64;
      if verbose {
        println!("==> -- expected rate {:.3} Hz", rate);   
      }
    } 
    // end new event
  } // end loop
}

