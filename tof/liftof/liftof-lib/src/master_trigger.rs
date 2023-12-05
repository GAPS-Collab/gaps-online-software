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
//! Issues: I do not like the error handling with
//! Box<dyn Error> here, since the additional runtime
//! cost. This needs to have a better error handling,
//! which should not be too difficult, I guess most 
//! times it can be replaced by some Udp realted 
//! error. [See issue #21](https://uhhepvcs.phys.hawaii.edu/Achim/gaps-online-software/-/issues/21)
//! _Comment_ There is not much error handling for UDP. Most of it is that the IPAddress is wrong, 
//! in this case it is legimate (and adviced) to panic.
//! In the case, wher no data was received, this might need some thinking.
use std::error::Error;
use std::time::{Duration, Instant};
use std::fmt;
use std::io;
use std::collections::HashMap;
use std::collections::VecDeque;
use std::net::{UdpSocket, SocketAddr};
use std::thread;
use crossbeam_channel::Sender;
use colored::Colorize;


use tof_dataclasses::DsiLtbRBMapping;
use tof_dataclasses::packets::TofPacket;
use tof_dataclasses::monitoring::MtbMoniData;
use tof_dataclasses::commands::RBCommand;
use tof_dataclasses::events::MasterTriggerEvent;
use tof_dataclasses::errors::{IPBusError, MasterTriggerError};

const MT_MAX_PACKSIZE   : usize = 1024;

const N_LTBS : usize = 20;
const N_CHN_PER_LTB : usize = 16;


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
      _ => Err(IPBusError::DecodingFailed)
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
/// addr        : register addresss
/// packet_type : read/write register?
/// data        : the data value at the specific
///               register.
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



/// Read the complete event of the MTB
///
/// FIXME - this can get extended to read 
/// multiple events at once. 
/// For that, we just have to query the
/// event size register multiple times.
pub fn get_mtevent(socket  : &UdpSocket,
                   address : &str,
                   buffer  : &mut [u8;MT_MAX_PACKSIZE]) -> Result<MasterTriggerEvent, MasterTriggerError> {
  let mut mte = MasterTriggerEvent::new(0,0);
  let mut n_daq_words : u32;
  let mut hits_a       : [bool;N_CHN_PER_LTB];
  let mut hits_b       : [bool;N_CHN_PER_LTB];
  let sleeptime = Duration::from_micros(10);
  loop {
    thread::sleep(sleeptime);
    match read_register(socket, address, 0x13 , buffer) {
      Err(err) => {
        error!("Timeout in read_register for MTB! {err}");
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
                                    address,
                                    0x11,
                                    buffer,
                                    IPBusPacketType::ReadNonIncrement,
                                    n_daq_words as usize)?;
  if data[0] != 0xAAAAAAAA {
    error!("Got MTB data, but the header is incorrect {}", data[0]);
    return Err(MasterTriggerError::PackageHeaderIncorrect);
  }
  let foot_pos = (n_daq_words - 1) as usize;
  if data[foot_pos] != 0x55555555 {
    error!("Got MTB data, but the footer is incorrect {}", data[foot_pos]);
    return Err(MasterTriggerError::PackageFooterIncorrect);
  }

  // Number of words which will be always there. 
  // Min event size is +1 word for hits
  const MTB_DAQ_PACKET_FIXED_N_WORDS : u32 = 9; 
  let n_hit_packets = n_daq_words - MTB_DAQ_PACKET_FIXED_N_WORDS;

  mte.event_id      = data[1];
  mte.timestamp     = data[2];
  mte.tiu_timestamp = data[3];
  mte.tiu_gps_32    = data[4];
  mte.tiu_gps_16    = data[5] & 0x0000ffff;
  mte.board_mask    = decode_board_mask(data[6]);
  let mut hitmasks = VecDeque::<[bool;N_CHN_PER_LTB]>::new();
  for k in 0..n_hit_packets {      
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
    SocketAddr::from(([0, 0, 0, 0], 50103)),
    SocketAddr::from(([0, 0, 0, 0], 50104)),
  ];
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



/// Gather monitoring data from the Mtb
///
/// ISSUES - some values are always 0
pub fn get_mtbmonidata(socket         : &UdpSocket,
                       target_address : &str,
                       buffer         : &mut [u8;MT_MAX_PACKSIZE])
  -> Result<MtbMoniData, MasterTriggerError> {
  let mut moni = MtbMoniData::new();
  let data     = read_register_multiple(socket,
                                        target_address,
                                        0x120,
                                        buffer,
                                        IPBusPacketType::Read,
                                        4)?;
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
                                            target_address,
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
/// * mt_ip       : ip address of the master trigger, most likely 
///                 something like 10.0.1.10
/// * mt_port     : 
///
///
/// * mtb_moni_interval : time in seconds when we 
///                       are acquiring mtb moni data.
///
/// * mtb_timeout_sec   : reconnect to mtb when we don't
///                       see events in mtb_timeout seconds.
///
/// * verbose           : Print "heartbeat" output 
///
pub fn master_trigger(mt_ip             : &str, 
                      mt_port           : usize,
                      dsi_j_mapping     : &DsiLtbRBMapping,
                      mt_sender         : &Sender<MasterTriggerEvent>,
                      rb_request_tp     : &Sender<TofPacket>,
                      moni_sender       : &Sender<TofPacket>,
                      mtb_moni_interval : u64,
                      mtb_timeout_sec   : u64,
                      verbose           : bool) {

  let mt_address = mt_ip.to_owned() + ":" + &mt_port.to_string(); 

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
  
  // step 1 - reset daq
  debug!("Resetting master trigger");
  match reset_daq(&socket, &mt_address) {
    Err(err) => error!("Can not reset DAQ, error {err}"),
    Ok(_)    => ()
  }

  // step 2 - event loop
  
  // timers - when to reconnect if no 
  // events have been received in a 
  // certain timeinterval
  let mut mtb_timeout   = Instant::now();
  let mut moni_interval = Instant::now();
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
                            &mt_address,
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
    match get_mtevent(&socket, &mt_address, &mut buffer) {
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
        let request_enabled = false; //FIXME
        if request_enabled {
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
            let mut rb_cmd      = RBCommand::new();
            rb_cmd.command_code = RBCommand::REQUEST_EVENT;
            rb_cmd.payload      = _ev.event_id;
            rb_cmd.rb_id        = *k;
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

/// Remotely read out a specif register of the MTB over UDP
///
/// # Arguments
///
/// * socket      : A valid UDP socket
/// * target_addr : The IP address of the MTB
/// * reg_addr    : The address of the MTB register to 
///                 be read
/// * buffer      : pre-allocated byte array to hold the 
///                 register value
pub fn read_register(socket      : &UdpSocket,
                     target_addr : &str,
                     reg_addr    : u32,
                     buffer      : &mut [u8;MT_MAX_PACKSIZE])
  -> Result<u32, Box<dyn Error>> {
  let send_data = Vec::<u32>::from([0]);
  let message   = encode_ipbus(reg_addr,
                               IPBusPacketType::Read,
                               &send_data);
  socket.send_to(message.as_slice(), target_addr)?;
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
                              target_addr : &str,
                              reg_addr    : u32,
                              buffer      : &mut [u8;MT_MAX_PACKSIZE],
                              ptype       : IPBusPacketType,
                              nwords      : usize)
  -> Result<Vec<u32>, Box<dyn Error>> {
  let send_data = vec![0u32;nwords];
  //let send_data = Vec::<u32>::from([0]);
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
  socket.send_to(message.as_slice(), target_addr)?;
  let (number_of_bytes, _) = socket.recv_from(buffer)?;
  trace!("Received {} bytes from master trigger", number_of_bytes);
  // this one can actually succeed, but return an emtpy vector
  let data = decode_ipbus(buffer, false)?;
  if data.len() == 0 { 
    error!("Empty data!");
    return Err(Box::new(IPBusError::DecodingFailed));
  }
  // this supports up to 100 Hz
  Ok(data)
}

/// Write a register on the MTB over UDP
///
/// # Arguments
///
/// * socket      : A valid UDP socket
/// * target_addr : The IP address of the MTB
/// * reg_addr    : The address of the MTB register to 
///                 be written
/// * data        : Write this number to the specific 
///                 register
/// * buffer      : pre-allocated byte array to hold the 
///                 response from the MTB
/// FIXME - there is no verification step!
pub fn write_register(socket      : &UdpSocket,
                      target_addr : &str,
                      reg_addr    : u32,
                      data        : u32,
                      buffer      : &mut [u8;MT_MAX_PACKSIZE])
  -> Result<(), Box<dyn Error>> {
  let send_data = Vec::<u32>::from([data]);
  let message   = encode_ipbus(reg_addr,
                               IPBusPacketType::Write,
                               &send_data);
  socket.send_to(message.as_slice(), target_addr)?;
  let (number_of_bytes, _) = socket.recv_from(buffer)?;
  trace!("Received {} bytes from master trigger", number_of_bytes);
  Ok(())
}

pub fn write_register_multiple(socket      : &UdpSocket,
                               target_addr : &str,
                               reg_addr    : u32,
                               data        : &Vec<u32>,
                               buffer      : &mut [u8;MT_MAX_PACKSIZE])
  -> Result<(), Box<dyn Error>> {
  let message   = encode_ipbus(reg_addr,
                               IPBusPacketType::Write,
                               &data);
  socket.send_to(message.as_slice(), target_addr)?;
  let (number_of_bytes, _) = socket.recv_from(buffer)?;
  trace!("Received {} bytes from master trigger", number_of_bytes);
  Ok(())
}

/// Read event counter register of MTB
pub fn read_event_cnt(socket : &UdpSocket,
                  target_address : &str,
                  buffer : &mut [u8;MT_MAX_PACKSIZE])
  -> Result<u32, Box<dyn Error>> {
  let event_count = read_register(socket, target_address, 0xd, buffer)?;
  trace!("Got event count! {} ", event_count);
  Ok(event_count)
}


/// Reset the state of the MTB DAQ
pub fn reset_daq(socket : &UdpSocket,
                 target_address : &str) 
  -> Result<(), Box<dyn Error>> {
  debug!("Resetting DAQ!");
  let mut buffer = [0u8;MT_MAX_PACKSIZE];
  write_register(socket, target_address, 0x10, 1,&mut buffer)?;
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

