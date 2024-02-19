//! MasterTriggerBoard communications
//!
//! The MTB (MasterTriggerBoard) is currently
//! (Jan 2023) connected to the ethernet 
//! via UDP sockets and sends out its 
//! own datapackets per each triggered 
//! event.
//!
//! The packet format contains the event id
//! as well as number of hits and a mask 
//! which encodes the hit channels.
//!
//! The data is encoded in IPBus packets.
//! [see docs here](https://ipbus.web.cern.ch/doc/user/html/)
//! 
use std::error::Error;
use std::time::{Duration, Instant};
use std::fmt;
use std::io;
use std::collections::HashMap;
use std::collections::VecDeque;
use std::net::{
    UdpSocket,
    SocketAddr
};
use std::thread;
use crossbeam_channel::Sender;
use colored::Colorize;


use tof_dataclasses::DsiLtbRBMapping;
use tof_dataclasses::packets::TofPacket;
use tof_dataclasses::monitoring::MtbMoniData;
use tof_dataclasses::commands::RBCommand;
use tof_dataclasses::events::MasterTriggerEvent;
use tof_dataclasses::events::master_trigger::TriggerType;
use tof_dataclasses::errors::{IPBusError, MasterTriggerError};

const MT_MAX_PACKSIZE   : usize = 1024;

use tof_dataclasses::constants::{
    N_LTBS,
    N_CHN_PER_LTB,
};

/// The IPBus standard encodes several packet types.
///
/// The packet type then will 
/// instruct the receiver to either 
/// write/read/etc. values from its
/// registers.
#[derive(Debug, PartialEq, Copy, Clone, serde::Deserialize, serde::Serialize)]
#[repr(u8)]
pub enum IPBusPacketType {
  Read                 = 0,
  Write                = 1,
  ReadNonIncrement     = 2,
  WriteNonIncrement    = 3,
  RMW                  = 4
}

impl fmt::Display for IPBusPacketType {
  fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
    let r = serde_json::to_string(self).unwrap_or(
      String::from("Error: cannot unwrap this IPBusPacketType"));
    write!(f, "<IPBusPacketType: {}>", r)
  }
}

impl TryFrom<u8> for IPBusPacketType {
  type Error = IPBusError;
  
  fn try_from(pt : u8)
    -> Result<IPBusPacketType,IPBusError> {
    match pt {
      0 => Ok(IPBusPacketType::Read),
      1 => Ok(IPBusPacketType::Write),
      2 => Ok(IPBusPacketType::ReadNonIncrement),
      3 => Ok(IPBusPacketType::WriteNonIncrement),
      4 => Ok(IPBusPacketType::RMW),
      _ => {
        error!("Unable to decode packet type {}", pt);
        return Err(IPBusError::DecodingFailed);
      }
    }
  }
}

//#[cfg(feature = "random")]
//impl FromRandom for IPBusPacketType {
//  
//  fn from_random() -> Self {
//    let choices = [
//      IPBusPacketType::Read,
//      IPBusPacketType::Write,
//      IPBusPacketType::ReadNonIncrement,
//      IPBusPacketType::WriteNonIncrement,
//      IPBusPacketType::RMW,
//    ];
//    let mut rng  = rand::thread_rng();
//    let idx = rng.gen_range(0..choices.len());
//    choices[idx]
//  }
//}


/// Encode register addresses and values in IPBus packet
///
/// # Arguments:
///
/// * addr        : register addresss
/// * packet_type : read/write register?
/// * data        : the data value at the specific
///                 register.
///
pub fn encode_ipbus(addr        : u32,
                    packet_type : IPBusPacketType,
                    data        : &Vec<u32>) -> Vec<u8> {
  // this will silently overflow, but 
  // if the message is that long, then 
  // most likely there will be a 
  // problem elsewhere, so we 
  // don't care
  let size = data.len() as u8;

  let packet_id = 0u8;
  let mut udp_data = Vec::<u8>::from([
    // Transaction Header
    0x20 as u8, // Protocol version & RSVD
    0x00 as u8, // Transaction ID (0 or bug)
    0x00 as u8, // Transaction ID (0 or bug)
    0xf0 as u8, // Packet order & packet_type
    // Packet Header
    //
    // FIXME - in the original python script, 
    // the 0xf0 is a 0xf00, but this does not
    // make any sense in my eyes...
    (0x20 as u8 | ((packet_id & 0xf0 as u8) as u32 >> 8) as u8), // Protocol version & Packet ID MSB
    (packet_id & 0xff as u8), // Packet ID LSB,
    size, // Words
    (((packet_type as u8 & 0xf as u8) << 4) | 0xf as u8), // Packet_Type & Info code
    // Address
    ((addr & 0xff000000 as u32) >> 24) as u8,

    ((addr & 0x00ff0000 as u32) >> 16) as u8,
    ((addr & 0x0000ff00 as u32) >> 8)  as u8,
    (addr  & 0x000000ff as u32) as u8]);

  if packet_type    == IPBusPacketType::Write
     || packet_type == IPBusPacketType::WriteNonIncrement {
    for i in 0..size as usize {
      udp_data.push (((data[i] & 0xff000000 as u32) >> 24) as u8);
      udp_data.push (((data[i] & 0x00ff0000 as u32) >> 16) as u8);
      udp_data.push (((data[i] & 0x0000ff00 as u32) >> 8)  as u8);
      udp_data.push ( (data[i] & 0x000000ff as u32)        as u8);
    }
  }
  //for n in 0..udp_data.len() {
  //    println!("-- -- {}",udp_data[n]);
  //}
  udp_data
}

/// Unpack a binary representation of an IPBusPacket
///
///
/// # Arguments:
///
/// * message : The binary representation following 
///             the specs of IPBus protocoll
/// * verbose : print information for debugging.
///
/// FIXME - currently this is always successful.
/// Should we check for garbage?
pub fn decode_ipbus( message : &[u8;MT_MAX_PACKSIZE],
                     verbose : bool)
    -> Result<Vec<u32>, IPBusError> {

    // Response
    let ipbus_version = message[0] >> 4;
    let id            = (((message[4] & 0xf as u8) as u32) << 8) as u8 | message[5];
    let size          = message[6];
    let pt_val        = (message[7] & 0xf0 as u8) >> 4;
    let info_code     = message[7] & 0xf as u8;
    let mut data      = Vec::<u32>::new(); //[None]*size

    let packet_type = IPBusPacketType::try_from(pt_val)?;
    // Read

    match packet_type {
      IPBusPacketType::Read |
      IPBusPacketType::ReadNonIncrement => {
        for i in 0..size as usize {
          data.push(  ((message[8 + i * 4]  as u32) << 24) 
                    | ((message[9 + i * 4]  as u32) << 16) 
                    | ((message[10 + i * 4] as u32) << 8)  
                    |  message[11 + i * 4]  as u32)
        }
      },
      IPBusPacketType::Write => data.push(0),
      IPBusPacketType::WriteNonIncrement
        => error!("I am sorry, I don't know what to do with this packet!"),
      IPBusPacketType::RMW
        => error!("I am sorry, I don't know what to do with this packet!")
    }

    if verbose { 
      println!("Decoding IPBus Packet:");
      println!(" > Msg = {:?}", message);
      println!(" > IPBus version = {}", ipbus_version);
      println!(" > ID = {}", id);
      println!(" > Size = {}", size);
      println!(" > Type = {:?}", packet_type);
      println!(" > Info = {}", info_code);
      println!(" > data = {:?}", data);
    }
    Ok(data)
}

/// Configure the trigger
#[derive(Debug, Copy, Clone, serde::Serialize, serde::Deserialize)]
pub struct MTBSettings {
  /// Select the trigger type for this run
  pub trigger_type           : TriggerType,
  /// Select the prescale factor for a run. The
  /// prescale factor is between 0 (no events)
  /// and 1.0 (all events). E.g. 0.1 means allow 
  /// only 10% of the events
  /// THIS DOES NOT APPLY TO THE GAPS OR POISSON 
  /// TRIGGER!
  pub trigger_prescale               : f32,
  /// in case trigger_type = "Poisson", set rate here
  pub poisson_trigger_rate           : u32,
  /// in case trigger_type = "Gaps", set if we want to use 
  /// beta
  pub gaps_trigger_use_beta     : bool,
  /// not sure
  //pub gaps_trigger_inner_thresh : u32,
  ///// not sure
  //pub gaps_trigger_outer_thresh : u32, 
  ///// not sure
  //pub gaps_trigger_total_thresh : u32, 
  ///// not sure
  //pub gaps_trigger_hit_thresh   : u32,
  /// Enable trace suppression on the MTB. If enabled, 
  /// only those RB which hits will read out waveforms.
  /// In case it is disabled, ALL RBs will readout events
  /// ALL the time. For this, we need also the eventbuilder
  /// strategy "WaitForNBoards(40)"
  pub trace_suppression  : bool,
  /// Time in seconds between housekkeping 
  /// packets
  pub mtb_moni_interval  : u64,
  /// The number of seconds we want to wait
  /// without hearing from the MTB before
  /// we attempt a reconnect
  pub mtb_timeout_sec    : u64,
  pub rb_int_window      : u8,
  pub tiu_emulation_mode : bool,
}

impl MTBSettings {
  pub fn new() -> Self {
    Self {
      trigger_type           : TriggerType::Unknown,
      trigger_prescale       : 0.0,
      poisson_trigger_rate   : 0,
      gaps_trigger_use_beta  : true,
      trace_suppression      : true,
      mtb_moni_interval      : 30,
      mtb_timeout_sec        : 60,
      rb_int_window          : 1,
      tiu_emulation_mode     : false,
    }
  }
}

impl fmt::Display for MTBSettings {
  fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
    let disp = toml::to_string(self).unwrap_or(
      String::from("-- DESERIALIZATION ERROR! --"));
    write!(f, "<MTBSettings :\n{}>", disp)
  }
}

impl Default for MTBSettings {
  fn default() -> Self {
    Self::new()
  }
}



/// Read the complete event of the MTB
///
/// FIXME - this can get extended to read 
/// multiple events at once. 
/// For that, we just have to query the
/// event size register multiple times.
///
/// # Arguments
///
/// * socket    : local sockat for Udp comms
/// * buffer    : location in memory where we
///               can write what we received
///               over Udp
pub fn get_mtevent(socket  : &UdpSocket,
                   buffer  : &mut [u8;MT_MAX_PACKSIZE]) -> Result<MasterTriggerEvent, MasterTriggerError> {
  let mut mte = MasterTriggerEvent::new(0,0);
  let mut n_daq_words  : u32;
  let mut hits_a       : [bool;N_CHN_PER_LTB];
  let mut hits_b       : [bool;N_CHN_PER_LTB];
  let sleeptime = Duration::from_micros(10);
  loop {
    thread::sleep(sleeptime);
    match read_register(socket, 
                        //address,
                        0x13 , buffer) {
      Err(err) => {
        // A timeout does not ncecessarily mean that there 
        // is no event, it can also just mean that 
        // the rate is low.
        debug!("Timeout in read_register for MTB! {err}");
        continue;
      },
      Ok(_n_words) => {
        n_daq_words = _n_words >> 16 as u16;
        if _n_words == 0 {
          continue;
        }
        debug!("Got n_daq_words {n_daq_words}");
        n_daq_words /= 2; //mtb internally operates in 16bit words, but 
                          //registers return 32bit words.
        break;
      }
    }
  }
  let data = read_register_multiple(socket,
                                    //address,
                                    0x11,
                                    buffer,
                                    IPBusPacketType::ReadNonIncrement,
                                    n_daq_words as usize)?;
  if data[0] != 0xAAAAAAAA {
    error!("Got MTB data, but the header is incorrect {}", data[0]);
    return Err(MasterTriggerError::PackageHeaderIncorrect);
  }
  let foot_pos = (n_daq_words - 1) as usize;
  if data.len() <= foot_pos {
    error!("Got MTB data, but the format is not correct");
    return Err(MasterTriggerError::PackageHeaderIncorrect);
  }
  if data[foot_pos] != 0x55555555 {
    error!("Got MTB data, but the footer is incorrect {}", data[foot_pos]);
    return Err(MasterTriggerError::PackageFooterIncorrect);
  }

  // Number of words which will be always there. 
  // Min event size is +1 word for hits
  const MTB_DAQ_PACKET_FIXED_N_WORDS : u32 = 9; 
  let n_hit_packets = n_daq_words - MTB_DAQ_PACKET_FIXED_N_WORDS;
  //println!("We are expecting {}", n_hit_packets);
  mte.event_id      = data[1];
  mte.timestamp     = data[2];
  mte.tiu_timestamp = data[3];
  mte.tiu_gps_32    = data[4];
  mte.tiu_gps_16    = data[5] & 0x0000ffff;
  mte.board_mask    = decode_board_mask(data[6]);
  let mut hitmasks = VecDeque::<[bool;N_CHN_PER_LTB]>::new();
  for k in 0..n_hit_packets {
    //println!("hit packet {:?}", data[7usize + k as usize]);
    (hits_a, hits_b) = decode_hit_mask(data[7usize + k as usize]);
    hitmasks.push_back(hits_a);
    hitmasks.push_back(hits_b);
  }
  for k in 0..mte.board_mask.len() {
    if mte.board_mask[k] {
      match hitmasks.pop_front() { 
        None => {
          error!("MTE hit assignment wrong. We expect hits for a certain LTB, but we don't see any!");
        },
        Some(_hits) => {
          mte.hits[k] = _hits;
        }
      }
    }
  }
  mte.n_paddles = mte.get_hit_paddles(); 
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
  // provide a number of local ports to try
  let local_addrs = [
    SocketAddr::from(([0, 0, 0, 0], 50100)),
    SocketAddr::from(([0, 0, 0, 0], 50101)),
    SocketAddr::from(([0, 0, 0, 0], 50102)),
    SocketAddr::from(([0, 0, 0, 0], 50103)),
    SocketAddr::from(([0, 0, 0, 0], 50104)),
  ];
  let local_socket = UdpSocket::bind(&local_addrs[..]);
  let socket : UdpSocket;
  match local_socket {
    Err(err)   => {
      error!("Can not create local UDP socket for master trigger connection!, err {}", err);
      return Err(err);
    }
    Ok(value)  => {
      info!("Successfully bound UDP socket for master trigger communcations to {:?}", value);
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



/// Gather monitoring data from the Mtb
///
/// ISSUES - some values are always 0
pub fn get_mtbmonidata(socket         : &UdpSocket,
                       buffer         : &mut [u8;MT_MAX_PACKSIZE])
  -> Result<MtbMoniData, MasterTriggerError> {
  let mut moni = MtbMoniData::new();
  let data     = read_register_multiple(socket,
                                        //target_address,
                                        0x120,
                                        buffer,
                                        IPBusPacketType::Read,
                                        4)?;
  if data.len() < 4 {
    return Err(MasterTriggerError::BrokenPackage);
  }
  let first_word   = 0x00000fff;
  let second_word  = 0x0fff0000;
  //println!("{:?} data", data);
  moni.calibration = ( data[0] & first_word  ) as u16;
  moni.vccpint     = ((data[0] & second_word ) >> 16) as u16;  
  moni.vccpaux     = ( data[1] & first_word  ) as u16;  
  moni.vccoddr     = ((data[1] & second_word ) >> 16) as u16;  
  moni.temp        = ( data[2] & first_word  ) as u16;  
  moni.vccint      = ((data[2] & second_word ) >> 16) as u16;  
  moni.vccaux      = ( data[3] & first_word  ) as u16;  
  moni.vccbram     = ((data[3] & second_word ) >> 16) as u16;  
  
  let rate         = read_register_multiple(socket, 
                                            //target_address,
                                            0x17,
                                            buffer,
                                            IPBusPacketType::Read,
                                            2)?;
  // FIXME - technically, the rate is 24bit, however, we just
  // read out 16 here (if the rate is beyond ~65kHz, we don't need 
  // to know with precision
  let mask        = 0x0000ffff;
  moni.rate       = (rate[0] & mask) as u16;
  moni.lost_rate  = (rate[1] & mask) as u16;
  Ok(moni)
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
/// * mt_address        : Udp address of the MasterTriggerBoard
///
/// * dsi_j_mapping     : A DsiLtbRBMapping containing mapping 
///                       information for DSI/J/CH (LTB) -> RBID/RBCH (RB)
///                       to identify high gain channels rb ids for 
///                       low gain channels
/// * mt_sender         : push retrieved MasterTriggerEvents to 
///                       this channel
/// * mtb_moni_interval : time in seconds when we 
///                       are acquiring mtb moni data.
///
/// * mtb_timeout_sec   : reconnect to mtb when we don't
///                       see events in mtb_timeout seconds.
///
/// * verbose           : Print "heartbeat" output 
///
/// * send_requests     : Send RBCommand instances with event 
///                       requests in TofPackets to the 
///                       rb_reqeust_tp Sender
pub fn master_trigger(mt_address        : String,
                      dsi_j_mapping     : &DsiLtbRBMapping,
                      mt_sender         : &Sender<MasterTriggerEvent>,
                      rb_request_tp     : &Sender<TofPacket>,
                      moni_sender       : &Sender<TofPacket>,
                      settings          : MTBSettings,
                      verbose           : bool,
                      send_requests     : bool) {

  // data buffer for MTB readout - allocate once and reuse
  let mut buffer = [0u8;MT_MAX_PACKSIZE];  
  
  // FIXME - this panics. However, it seems there is no way to init an UdpSocket 
  // without binding it. And if it can't bind, it panics.
  let mut socket = connect_to_mtb(&mt_address).expect("Can not establish initial connection to MTB");
  // unfortunatly something like this won't compile
  //let mut socket : UdpSocket; 
  //while !connected {
  //  match connect_to_mtb(&mt_address) {
  //    Err(err) => {
  //      error!("Can not create local UDP socket fro MTB connections!, {err}");
  //      thread::sleep(connection_timeout);
  //      continue;
  //    },
  //    Ok(_sock) => {
  //      info!("Successfully created local UDP socket for MTB connection!");
  //      socket = _sock;
  //      connected = true;
  //    }
  //  }
  //}
  

  // configure MTB here
  let trace_suppression = settings.trace_suppression;
  match set_trace_suppression(&socket, trace_suppression) {
    Err(err) => error!("Unable to set trace suppression mode! {err}"),
    Ok(_)    => {
      if trace_suppression {
        println!("==> Setting MTB to trace suppression mode!");
      } else {
        println!("==> Setting MTB to ALL_RB_READOUT mode!");
        warn!("Reading out all events from all RBs! Data might be very large!");
      }
    }
  }
  info!("Settting rb integration window!");
  let int_wind = settings.rb_int_window;
  match set_rb_int_window(&socket, int_wind) {
    Err(err) => error!("Unable to set rb integration window! {err}"),
    Ok(_)    => {
      info!("rb integration window set to {}", int_wind); 
    } 
  }

  debug!("Resetting master trigger DAQ");
  match reset_daq(&socket) {//, &mt_address) {
    Err(err) => error!("Can not reset DAQ, error {err}"),
    Ok(_)    => ()
  }
  match settings.trigger_type {
    TriggerType::Poisson => {
      unset_all_triggers(&socket); 
      set_poisson_trigger(&socket,settings.poisson_trigger_rate);
    }
    TriggerType::Any     => {
      unset_all_triggers(&socket); 
      set_any_trigger(&socket,settings.trigger_prescale); 
    }
    TriggerType::Track   => {
      unset_all_triggers(&socket); 
      set_track_trigger(&socket, settings.trigger_prescale);
    }
    TriggerType::Gaps    => {
      unset_all_triggers(&socket); 
      set_gaps_trigger(&socket, settings.gaps_trigger_use_beta);
    }
    TriggerType::Unknown => {
      println!("== ==> Not setting any trigger condition. You can set it through pico_hal.py");
      warn!("Trigger condition undefined! Not setting anything!");
      error!("Trigger conditions unknown!");
    }
    _ => {
      // actually panic here since this does not make sense
      panic!("Can't set trigger for trigger type {}", settings.trigger_type);
    }
  }

  // step 2 - event loop
  
  // timers - when to reconnect if no 
  // events have been received in a 
  // certain timeinterval
  let mut mtb_timeout   = Instant::now();
  let mut moni_interval = Instant::now();
  let mtb_timeout_sec   = settings.mtb_timeout_sec;
  let mtb_moni_interval = settings.mtb_moni_interval;
  // verbose, debugging
  let mut last_event_id = 0u32;
  let mut n_events      = 0u64;
  let mut rate_from_reg : Option<u32> = None;
  let mut verbose_timer = Instant::now();
  let mut total_elapsed = 0f64;
  let mut n_ev_unsent   = 0u64;
  let mut n_ev_missed   = 0u64;
  loop {
    if mtb_timeout.elapsed().as_secs() > mtb_timeout_sec {
      error!("MTB timed out! Attempting to reconnnect...");
      match connect_to_mtb(&mt_address) {
        Err(err) => {
          error!("Can not establish initial connection to MTB! {err}");
        }
        Ok(_sock) => {
          info!(".. connected!");
          socket = _sock;
        }
      }
    }
    if moni_interval.elapsed().as_secs() > mtb_moni_interval {
      match get_mtbmonidata(&socket, 
                            //&mt_address,
                            &mut buffer) {
        Err(err) => {
          error!("Can not get MtbMoniData! {err}");
        },
        Ok(_moni) => {
          let tp = TofPacket::from(&_moni);
          match moni_sender.send(tp) {
            Err(err) => {
              error!("Can not send MtbMoniData over channel! {err}");
            },
            Ok(_) => ()
          }
          if verbose {
            println!("{}", _moni);
            rate_from_reg = Some(_moni.rate as u32);
          }
        }
      }
      moni_interval = Instant::now();
    }
    match get_mtevent(&socket,
                      //&mt_address,
                      &mut buffer) {
      Err(err) => {
        error!("Unable to get MasterTriggerEvent! {err}");
        continue;
      },
      Ok(_ev) => {
        if _ev.event_id == last_event_id {
          error!("We got a duplicate event from the MTB!");
          continue;
        }
        if _ev.event_id > last_event_id + 1 {
          if last_event_id != 0 {
            error!("We skipped {} events!", _ev.event_id - last_event_id); 
            n_ev_missed += (_ev.event_id - last_event_id) as u64;
          }
        }
        last_event_id = _ev.event_id;
        //let tp = TofPacket::from(&_ev);
        // we got an even successfully, so reset the 
        // connection timeout
        mtb_timeout = Instant::now();
        n_events += 1;
        if send_requests {
          trace!("Got new event id from master trigger {}",_ev.event_id);
          let hits = _ev.get_dsi_j_ch_for_triggered_ltbs();
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
            if rb.1 < 1 {
              warn!("Invalid channel 0 found in DSI/J/CH (LTB->RB) map!");
              continue
            }
            if rbs_ch.contains_key(&rb.0) {
              // unwrap is fine, bc we just checked if 
              // the key exists
              rbs_ch.get_mut(&rb.0).unwrap().push(rb.1);
            } else {
              let mut new_vec = Vec::<u8>::new();
              new_vec.push(rb.1);
              rbs_ch.insert(rb.0, new_vec);
            }
          }
          //println!("RBS CH KEYS {:?}", rbs_ch.keys());
          for k in rbs_ch.keys() { 
            let mut rb_cmd      = RBCommand::new();
            rb_cmd.command_code = RBCommand::REQUEST_EVENT;
            rb_cmd.payload      = _ev.event_id;
            rb_cmd.rb_id        = *k;
            for ch in rbs_ch[&k].iter() {
              //println!("{}", ch);
              // here we have to subtract 1 because in the db and 
              // also in the json RB channels will be 1-9
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
        } // end if request
        match mt_sender.send(_ev) {
          Err(err) => {
            error!("Can not send MasterTriggerEvent over channel! {err}");
            n_ev_unsent += 1;
          },
          Ok(_) => ()
        }

      }
    }
    if verbose {
      let verbose_timer_elapsed = verbose_timer.elapsed().as_secs_f64();
      if verbose_timer_elapsed > 20.0 {
        total_elapsed += verbose_timer_elapsed;
        println!("  {}", ">> == == == == ==  MT HEARTBEAT   == == == == == <<".bright_blue().bold());
        println!("  {}{:.1}{}", ">> ==> ".bright_blue(),total_elapsed, " mission elapsed time (sec)           <<".bright_blue());
        println!("  {}{}{}", ">> ==> ".bright_blue(),n_events, " events recorded                       <<".bright_blue());
        if n_ev_unsent > 0 {
          println!("  {}{}{}", ">> ==> ".yellow().bold(),n_ev_unsent, " sent errors                       <<".yellow().bold());
        }
        if n_ev_missed > 0 {
          println!("  {}{}{}", ">> ==> ".yellow().bold(),n_events, " missed events                       <<".yellow().bold());
        }
        println!("  {}{:.2}{}", ">> ==> -- trigger rate: ".bright_blue(), n_events as f64/total_elapsed, " Hz                 <<".bright_blue());
        match rate_from_reg {
          None => (),
          Some(_rate) => {
            println!("  {}{:.3}{}",">> ==> -- expected rate ".bright_blue(),_rate," Hz (from register)    <<".bright_blue());   
          }
        }
        println!("  {}",">> == == == == ==  END HEARTBEAT! == == == == == <<".bright_blue().bold());
        verbose_timer = Instant::now();
      }
    }
  }
}

/// Remotely read out a specific register of the MTB over UDP
///
/// # Arguments
///
/// * socket      : A valid UDP socket, "connected" to 
///                 the MTB through UdpSocket::connect
/// * reg_addr    : The address of the MTB register to 
///                 be read
/// * buffer      : pre-allocated byte array to hold the 
///                 register value
pub fn read_register(socket      : &UdpSocket,
                     reg_addr    : u32,
                     buffer      : &mut [u8;MT_MAX_PACKSIZE])
  -> Result<u32, Box<dyn Error>> {
  let send_data = Vec::<u32>::from([0]);
  let message   = encode_ipbus(reg_addr,
                               IPBusPacketType::Read,
                               &send_data);
  socket.send(message.as_slice())?;
  let (number_of_bytes, _) = socket.recv_from(buffer)?;
  trace!("Received {} bytes from master trigger", number_of_bytes);
  // this one can actually succeed, but return an emtpy vector
  let data = decode_ipbus(buffer, false)?;
  if data.len() == 0 
    { return Err(Box::new(IPBusError::DecodingFailed));}
  // this supports up to 100 Hz
  Ok(data[0])
}

pub fn read_register_multiple(socket      : &UdpSocket,
                              reg_addr    : u32,
                              buffer      : &mut [u8;MT_MAX_PACKSIZE],
                              ptype       : IPBusPacketType,
                              nwords      : usize)
  -> Result<Vec<u32>, Box<dyn Error>> {
  let send_data = vec![0u32;nwords];
  let message : Vec<u8>;
  if send_data.len() > 1 {
    message = encode_ipbus(reg_addr,
                           ptype,
                           &send_data);
  } else {
    message   = encode_ipbus(reg_addr,
                             IPBusPacketType::Read,
                             &send_data);
  }
  socket.send(message.as_slice())?;
  let (number_of_bytes, _) = socket.recv_from(buffer)?;
  trace!("Received {} bytes from master trigger", number_of_bytes);
  // this one can actually succeed, but return an emtpy vector
  let data = decode_ipbus(buffer, false)?;
  if data.len() == 0 { 
    error!("Empty data!");
    return Err(Box::new(IPBusError::DecodingFailed));
  }
  if data.len() != nwords {
    error!("Data too short!");
    return Err(Box::new(MasterTriggerError::DataTooShort));
  }
  // this supports up to 100 Hz
  Ok(data)
}

/// Write a register on the MTB over UDP
///
/// # Arguments
///
/// * socket      : A valid UDP socket, "connected" to
///                 the MTB through UdpSocket::connect
/// * reg_addr    : The address of the MTB register to 
///                 be written
/// * data        : Write this number to the specific 
///                 register
/// * buffer      : pre-allocated byte array to hold the 
///                 response from the MTB
/// FIXME - there is no verification step!
pub fn write_register(socket      : &UdpSocket,
                      reg_addr    : u32,
                      data        : u32,
                      buffer      : &mut [u8;MT_MAX_PACKSIZE])
  -> Result<(), Box<dyn Error>> {
  let send_data = Vec::<u32>::from([data]);
  let message   = encode_ipbus(reg_addr,
                               IPBusPacketType::Write,
                               &send_data);
  socket.send(message.as_slice())?;
  let (number_of_bytes, _) = socket.recv_from(buffer)?;
  trace!("Received {} bytes from master trigger", number_of_bytes);
  Ok(())
}

pub fn write_register_multiple(socket      : &UdpSocket,
                               reg_addr    : u32,
                               data        : &Vec<u32>,
                               buffer      : &mut [u8;MT_MAX_PACKSIZE])
  -> Result<(), Box<dyn Error>> {
  let message   = encode_ipbus(reg_addr,
                               IPBusPacketType::Write,
                               &data);
  socket.send(message.as_slice())?;
  let (number_of_bytes, _) = socket.recv_from(buffer)?;
  trace!("Received {} bytes from master trigger", number_of_bytes);
  Ok(())
}

/// Read event counter register of MTB
pub fn read_event_cnt(socket : &UdpSocket,
                      buffer : &mut [u8;MT_MAX_PACKSIZE])
  -> Result<u32, Box<dyn Error>> {
  let event_count = read_register(socket,
                                  //target_address,
                                  0xd, buffer)?;
  trace!("Got event count! {} ", event_count);
  Ok(event_count)
}

/// Set the RB readout mode - either 
/// read out all channels all the time
/// or use the MTB to indicate to the RBs
/// which channels to read out 
pub fn set_trace_suppression(socket : &UdpSocket,
                             sup    : bool) 
  -> Result<(), Box<dyn Error>> {
  info!("Setting MTB trace suppression {}!", sup);
  let mut buffer = [0u8;MT_MAX_PACKSIZE];
  let mut value = read_register(socket, 0xf, &mut buffer)?;
  // bit 13 has to be 1 for read all channels
  let mut read_all_ch = u32::pow(2, 13);
  if sup { // sup means !read_all_ch
    value = value & !read_all_ch;
  }
  else {
    value = value | read_all_ch; 
  }
  //let val = !sup;
  //value = value | (val as u32) << 13;
  write_register(socket,
                 0xf,
                 value,
                 &mut buffer)?;
  Ok(())
}

/// Reset the state of the MTB DAQ
pub fn reset_daq(socket : &UdpSocket) 
  -> Result<(), Box<dyn Error>> {
  info!("Resetting DAQ!");
  let mut buffer = [0u8;MT_MAX_PACKSIZE];
  write_register(socket,
                 0x10, 1,&mut buffer)?;
  Ok(())
}

pub fn get_tiu_link_status(socket : &UdpSocket)
  -> Result<bool, Box<dyn Error>> {
  let mut tiu_good = 0x1u32;
  let mut buffer   = [0u8;MT_MAX_PACKSIZE];
  let mut value    = read_register(socket, 0xf, &mut buffer)?;
  tiu_good         = tiu_good & ( value & 0x1);
  Ok(tiu_good > 0)
}

/// FIXME
pub fn set_rb_int_window(socket : &UdpSocket, wind : u8)
  -> Result<(), Box<dyn Error>> {
  info!("Setting RB_INT_WINDOW to {}!", wind);
  let mut buffer = [0u8;MT_MAX_PACKSIZE];
  let mut value =  read_register(socket, 0xf , &mut buffer)?;
  let mut mask = 0xfffff0ff;
  // switch the bins off
  value = value & mask;
  let wind_bits = (wind as u32) << 8;
  value = value | wind_bits;
  write_register(socket,
                 0xf,
                 value,
                 &mut buffer)?;
  Ok(())
}

/// Set the poisson trigger with a prescale
pub fn set_poisson_trigger(socket : &UdpSocket, rate : u32) 
  -> Result<(), Box<dyn Error>> {
  //let clk_period = 1e8u32; 
  let clk_period = 100000000;
  let rate_val = (u32::MAX*rate)/clk_period;//(1.0/ clk_period)).floor() as u32;
  
  //let rate_val   = (rate as f32 * u32::MAX as f32/1.0e8) as u32; 
  info!("Setting poisson trigger with rate {}!", rate);
  let mut buffer = [0u8;MT_MAX_PACKSIZE];
  write_register(socket,
                 0x9,
                 rate_val,
                 &mut buffer)?;
  Ok(())
}

/// Set the any trigger with a prescale
pub fn set_any_trigger(socket : &UdpSocket, prescale : f32) 
  -> Result<(), Box<dyn Error>> {
  let prescale_val = (u32::MAX as f32 * prescale).floor() as u32;
  info!("Setting any trigger with prescale {}!", prescale);
  let mut buffer = [0u8;MT_MAX_PACKSIZE];
  write_register(socket,
                 0x40,
                 prescale_val,
                 &mut buffer)?;
  Ok(())
}

/// Set the track trigger with a prescale
pub fn set_track_trigger(socket : &UdpSocket, prescale : f32) 
  -> Result<(), Box<dyn Error>> {
  let prescale_val = (u32::MAX as f32 * prescale).floor() as u32;
  info!("Setting track trigger with prescale {}!", prescale);
  let mut buffer = [0u8;MT_MAX_PACKSIZE];
  write_register(socket,
                 0x41,
                 prescale_val,
                 &mut buffer)?;
  Ok(())
}

/// Disable all triggers
pub fn unset_all_triggers(socket : &UdpSocket) 
  -> Result<(), Box<dyn Error>> {
  let mut buffer = [0u8;MT_MAX_PACKSIZE];
  // first the GAPS trigger, whcih is a more 
  // complicated register, where we only have
  // to flip 1 bit
  let mut trig_settings = read_register(socket, 0x14 , &mut buffer)?;
  trig_settings         = trig_settings & !u32::pow(2,24);
  write_register(socket,
                 0x14,
                 trig_settings,
                 &mut buffer)?;
  set_poisson_trigger(socket, 0);
  set_any_trigger    (socket, 0.0);
  set_track_trigger  (socket, 0.0);
  Ok(())
}

/// Set the gaps trigger with a prescale
pub fn set_gaps_trigger(socket : &UdpSocket, use_beta : bool) 
  -> Result<(), Box<dyn Error>> {
  info!("Setting GAPS Antiparticle trigger, use beta {}!", use_beta);
  let mut buffer = [0u8;MT_MAX_PACKSIZE];
  let mut trig_settings =  read_register(socket, 0x14 , &mut buffer)?;
  trig_settings = trig_settings | u32::pow(2,24);
  if use_beta {
    trig_settings = trig_settings | u32::pow(2,25);
  }
  write_register(socket,
                 0x14,
                 trig_settings,
                 &mut buffer)?;
  Ok(())
}

/// Helper to get the number of the triggered LTB from the bitmask
pub fn decode_board_mask(board_mask : u32) -> [bool;N_LTBS] {
  let mut decoded_mask = [false;N_LTBS];
  // FIXME this implicitly asserts that the fields for non available LTBs 
  // will be 0 and all the fields will be in order
  //println!("BOARD MASK {}", board_mask);
  let mut index = N_LTBS - 1;
  for n in 0..N_LTBS {
    let mask = 1 << n;
    let bit_is_set = (mask & board_mask) > 0;
    decoded_mask[index] = bit_is_set;
    if index != 0 {
        index -= 1;
    }
    //decoded_mask[N_LTBS-1 - n] = bit_is_set;
  }
  //println!("DECODED MASK {:?}", decoded_mask);
  // reverse the mask, so actually RAT0 is at position 0
  decoded_mask.reverse();
  decoded_mask
}

/// Helper to get the number of the triggered LTB from the bitmask
pub fn decode_hit_mask(hit_mask : u32) -> ([bool;N_CHN_PER_LTB],[bool;N_CHN_PER_LTB]) {
  //println!("HITMASK NON DECODED :{}", hit_mask);
  let mut decoded_mask_0 = [false;N_CHN_PER_LTB];
  let mut decoded_mask_1 = [false;N_CHN_PER_LTB];
  // FIXME this implicitly asserts that the fields for non available LTBs 
  // will be 0 and all the fields will be in order
  let mut index = N_CHN_PER_LTB - 1;
  for n in 0..N_CHN_PER_LTB {
    let mask = 1 << n;
    //println!("MASK {:?}", mask);
    let bit_is_set = (mask & hit_mask) > 0;
    //println!("bit is set {}, index {}", bit_is_set, index);
    //decoded_mask_0[N_CHN_PER_LTB-1 - n] = bit_is_set;
    decoded_mask_0[index] = bit_is_set;
    if index != 0 {
      index -= 1;
    }
  }
  index = N_CHN_PER_LTB -1;
  for n in N_CHN_PER_LTB..2*N_CHN_PER_LTB {
    let mask = 1 << n;
    let bit_is_set = (mask & hit_mask) > 0;
    //FIXME - this is buggy and panics. Until this is fixed,
    //I'll revive my cringy way to do things.
    //decoded_mask_1[N_CHN_PER_LTB-1 - n] = bit_is_set;
    decoded_mask_1[index] = bit_is_set;
    if index != 0 {
      index -= 1;
    }
  }
  //println!("DECODED HITMASK 0 {:?}", decoded_mask_0);
  //println!("DECODED HITMASK 1 {:?}", decoded_mask_1);
  // reverse everything 
  // so decoded_mask_0 is still for the first board, but 
  // let's do the channels so that they count up
  decoded_mask_0.reverse();
  decoded_mask_1.reverse();
  (decoded_mask_0, decoded_mask_1)
}



