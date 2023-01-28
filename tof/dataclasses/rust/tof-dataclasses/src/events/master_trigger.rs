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

use std::net::UdpSocket;
use std::fmt;

use std::error::Error;
use crate::errors::IPBusError;

//const MT_MAX_PACKSIZE   : usize = 4096;
/// Maximum packet size of packets we can 
/// receive over UDP via the IPBus protocoll
/// (arbitrary number)
const MT_MAX_PACKSIZE   : usize = 512;

/// The IPBus standard encodes several packet types.
///
/// The packet type then will 
/// instruct the receiver to either 
/// write/read/etc. values from its
/// registers.
#[derive(Debug, PartialEq, Copy, Clone)]
#[repr(u8)]
pub enum IPBusPacketType {
  Read                 = 0,
  Write                = 1,
  ReadNonIncrement     = 2,
  WriteNonIncrememnt   = 3,
  RMW                  = 4
}

impl TryFrom<u8> for IPBusPacketType {
  type Error = IPBusError;
  
  fn try_from(pt : u8)
    -> Result<IPBusPacketType,IPBusError> {
    match pt {
      0 => {return Ok(IPBusPacketType::Read);},
      1 => {return Ok(IPBusPacketType::Write);},
      2 => {return Ok(IPBusPacketType::ReadNonIncrement);},
      3 => {return Ok(IPBusPacketType::WriteNonIncrememnt);},
      4 => {return Ok(IPBusPacketType::RMW);},
      _ => {return Err(IPBusError::DecodingFailed);},
    }
  }
}

impl From<IPBusPacketType> for u8 {
  fn from(pt : IPBusPacketType)
    -> u8 {
    let result : u8;
    match pt {
     IPBusPacketType::Read               => { result = 0;}, 
     IPBusPacketType::Write              => { result = 1;}, 
     IPBusPacketType::ReadNonIncrement   => { result = 2;},  
     IPBusPacketType::WriteNonIncrememnt => { result = 3;},  
     IPBusPacketType::RMW                => { result = 4;}, 
    }
    result
  }
}

/// An event as observed by the MTB
///
/// This is condensed to the most 
/// crucial information 
///
/// FIXME : implementation of absolute time
#[derive(Debug, Copy, Clone)]
pub struct MasterTriggerEvent {
  pub event_id      : u32,
  pub timestamp     : u32,
  pub tiu_timestamp : u32,
  pub tiu_gps_32    : u32,
  pub tiu_gps_16    : u32,
  pub n_paddles     : u8, 
  pub board_mask    : u32,
  // one 16 bit value per LTB
  pub hits          : [u16; 20],

  /// valid is an internal flag
  /// used by code working with MTEs.
  /// Set it to false will mark the 
  /// package for deletion.
  /// Once invalidated, an event 
  /// never shall be valid again.
  valid     : bool
}

impl MasterTriggerEvent {
  pub fn new(event_id  : u32, 
             n_paddles : u8) -> MasterTriggerEvent {
    MasterTriggerEvent {
      event_id      : event_id,
      timestamp     : 0,
      tiu_timestamp : 0,
      tiu_gps_32    : 0,
      tiu_gps_16    : 0,
      n_paddles     : n_paddles, 
      board_mask    : 0,
      //ne 16 bit value per LTB
      hits          : [0; 20],
      valid     : true
    }   
  }
  
  pub fn invalidate(&mut self) {
    self.valid = false;
  }
}

impl Default for MasterTriggerEvent {
  fn default() -> MasterTriggerEvent {
    MasterTriggerEvent::new(0,0)
  }
}

impl fmt::Display for MasterTriggerEvent {
  fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
    write!(f, "<MasterTriggerEvent\n event id\t {}\n timestamp\t {}\n tiu_timestamp\t {}\n tiu_gps_32\t {}\n tiu_gps_16\t {}\n n paddles\t {}\n board mask\t {}  >",
           self.event_id, self.timestamp, self.tiu_timestamp, self.tiu_gps_32, self.tiu_gps_16, self.n_paddles, self.board_mask)
  }
}


#[derive(Debug, Copy, Clone)]
pub struct IPBusPacket {
}

impl IPBusPacket {
  pub fn new() -> IPBusPacket {
    todo!();
    IPBusPacket {}
  }
}




/// Little helper to count the ones in a bit mask
fn count_ones(input :u32) -> u32 {
  let mut count = 0u32;
  let mut value = input;
  while value > 0 {
    count += value & 1;
    value >>= 1;
  }
  count
}


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
     || packet_type == IPBusPacketType::WriteNonIncrememnt {
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

    if matches!(packet_type, IPBusPacketType::Read)
    || matches!(packet_type, IPBusPacketType::ReadNonIncrement) {
      for i in 0..size as usize {
        data.push(  ((message[8 + i * 4]  as u32) << 24) 
                  | ((message[9 + i * 4]  as u32) << 16) 
                  | ((message[10 + i * 4] as u32) << 8)  
                  |  message[11 + i * 4]  as u32)
      }
    }

    // Write
    if matches!(packet_type, IPBusPacketType::Write) {
        data.push(0);
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
fn read_register(socket      : &UdpSocket,
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
  
  Ok(data[0])
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
  //let data = decode_ipbus(buffer, false)[0];
  //def wReg(address, data, verify=False):
  //    s.sendto(encode_ipbus(addr=address, packet_type=WRITE, data=[data]), target_ad    dress)
  //    s.recvfrom(4096)
  //    rdback = rReg(address)
  //    if (verify and rdback != data):
  //        print("Error!")
  //
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

pub fn read_rate(socket : &UdpSocket,
                 target_address : &str,
                 buffer : &mut [u8;MT_MAX_PACKSIZE])
  -> Result<u32, Box<dyn Error>> {
  let rate = read_register(socket, target_address, 0x17, buffer)?;
  trace!("Got MT rate! {} ", rate);
  Ok(rate)
}

pub fn read_lost_rate(socket : &UdpSocket,
                      target_address : &str,
                      buffer : &mut [u8;MT_MAX_PACKSIZE])
  -> Result<u32, Box<dyn Error>> {
  let lost_rate = read_register(socket, target_address, 0x18, buffer)?;
  trace!("Got MT lost rate! {} ", lost_rate);
  Ok(lost_rate)
}

/// Reset event counter on MTB
pub fn reset_event_cnt(socket : &UdpSocket,
                       target_address : &str) 
  -> Result<(), Box<dyn Error>>{
  debug!("Resetting event counter!");
  let mut buffer = [0u8;MT_MAX_PACKSIZE];
  write_register(socket, target_address, 0xc,1,&mut buffer)?;
  Ok(())
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

/// Read the IPBus packets from the MTB DAQ
///



/// # Arguments:
/// 
/// * socket         : An open Udp socket on the host side
/// * target_address : The IP address of the MTB
/// * buffer         : allocated memory for the MTB response
pub fn read_daq(socket : &UdpSocket,
                target_address : &str,
                buffer : &mut [u8;MT_MAX_PACKSIZE])
  -> Result<(u32, u32), Box<dyn Error>> {
  // check if the queue is full
  let mut event_ctr  = 0u32;
  let timestamp  : u32;
  let timecode32 : u32;
  let timecode16 : u32;
  let mask       : u32;
  //let mut hits     :  0u32;
  let crc        : u32;
  let trailer    : u32;
  let mut hit_paddles = 0u32;
  let mut paddles_rxd     = 1u32;
  let mut hits = Vec::<u32>::with_capacity(24);

  // How this works is the following. We read the data register
  // until we get the header word. Then we have a new event 
  // and we fill the values of our MasterTriggerEvent by 
  // subsequently reading out the same register again
  let word = read_register(socket, target_address, 0x11, buffer)?;
  // this will eventually determin, 
  // how often we will read the 
  // hit register
  if word == 0xAAAAAAAA {
    // we start a new daq package
    event_ctr   = read_register(socket, target_address, 0x11, buffer)?;
    timestamp   = read_register(socket, target_address, 0x11, buffer)?;
    timecode32  = read_register(socket, target_address, 0x11, buffer)?;
    timecode16  = read_register(socket, target_address, 0x11, buffer)?;
    mask        = read_register(socket, target_address, 0x11, buffer)?;
    hit_paddles = count_ones(mask);
    hits.push     (read_register(socket, target_address, 0x11, buffer)?);
    //allhits.push(hits);  
    while paddles_rxd < hit_paddles {
      hits.push(read_register(socket, target_address, 0x11, buffer)?);
      paddles_rxd += 1;
    }
    crc         = read_register(socket, target_address, 0x11, buffer)?;
    trailer     = read_register(socket, target_address, 0x11, buffer)?;

    debug!("event_ctr {}, ts {} , tc32 {}, tc16 {}, mask {}, crc {}, trailer {}", event_ctr, timestamp, timecode32, timecode16, hit_paddles, crc, trailer);
    for n in 0..hits.len() {
      debug!(" -- -- hit {}", hits[n]);
    }
  } // end header found
    //AAAAAAAA (Header)
    //0286A387 (Event cnt)
    //00000000 (Timestamp)
    //00000000 (Timecode 32 bits)
    //00000000 (Timecode 16 bits)
    //00000001 (Mask)
    //00000003 (Hits)
    //97041A48 (CRC)
    //55555555 (Trailer)
  Ok((event_ctr, hit_paddles))
}


