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
use std::time::Duration;

use std::error::Error;
use crate::errors::{IPBusError, MasterTriggerError};

use crate::serialization::{Serialization,
                           SerializationError};

//const MT_MAX_PACKSIZE   : usize = 4096;
/// Maximum packet size of packets we can 
/// receive over UDP via the IPBus protocoll
/// (arbitrary number)
const MT_MAX_PACKSIZE   : usize = 512;


const N_LTBS : usize = 20;
const N_CHN_PER_LTB : usize = 16;

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
  // indicates which LTBs have 
  // triggered
  pub board_mask    : [bool; N_LTBS],
  // one 16 bit value per LTB
  // the sorting is the same as
  // in board_mask
  pub hits          : [[bool;N_CHN_PER_LTB]; N_LTBS],
  pub crc           : u32,
  // valid is an internal flag
  // used by code working with MTEs.
  // Set it to false will mark the 
  // package for deletion.
  // Once invalidated, an event 
  // never shall be valid again.
  valid     : bool,
  pub broken   : bool
}

impl MasterTriggerEvent {
  // 21 + 4 byte board mask + 4*4 bytes hit mask
  // => 25 + 16 = 41 
  // + head + tail
  // 45
  const SIZE : usize = 45;
  const TAIL : u16 = 0x555;
  const HEAD : u16 = 0xAAAA;

  pub fn new(event_id  : u32, 
             n_paddles : u8) -> MasterTriggerEvent {
    MasterTriggerEvent {
      event_id      : event_id,
      timestamp     : 0,
      tiu_timestamp : 0,
      tiu_gps_32    : 0,
      tiu_gps_16    : 0,
      n_paddles     : n_paddles, 
      board_mask    : [false;N_LTBS],
      //ne 16 bit value per LTB
      hits          : [[false;N_CHN_PER_LTB]; N_LTBS],
      crc           : 0,
      broken    : false,
      // valid does not get serialized
      valid     : true,
    }   
  }

  pub fn is_broken(&self) -> bool {
    self.broken
  }

  pub fn to_bytestream(&self) -> Vec::<u8> {
    let mut bs = Vec::<u8>::with_capacity(MasterTriggerEvent::SIZE);
    bs.extend_from_slice(&MasterTriggerEvent::HEAD.to_le_bytes());
    bs.extend_from_slice(&self.event_id.to_le_bytes()); 
    bs.extend_from_slice(&self.timestamp.to_le_bytes());
    bs.extend_from_slice(&self.tiu_timestamp.to_le_bytes());
    bs.extend_from_slice(&self.tiu_gps_32.to_le_bytes());
    bs.extend_from_slice(&self.tiu_gps_16.to_le_bytes());
    bs.extend_from_slice(&self.n_paddles.to_le_bytes());
    let mut board_mask : u32 = 0;
    for n in 0..N_LTBS {
      if self.board_mask[n] {
        board_mask += 2_u32.pow(n as u32);
      }
    }
    bs.extend_from_slice(&board_mask.to_le_bytes());
    for n in 0..N_LTBS {
      let mut hit_mask : u32 = 0;
      for j in 0..N_CHN_PER_LTB {
        if self.hits[n][j] {
          hit_mask += 2_u32.pow(j as u32);
        }
      }
      bs.extend_from_slice(&hit_mask.to_le_bytes());
    }
    bs.extend_from_slice(&self.crc.to_le_bytes());
    bs.extend_from_slice(&MasterTriggerEvent::TAIL.to_le_bytes());
    bs
  }

  fn bitmask_to_str(mask : &[bool]) -> String {
    let mut m_str = String::from("");
    for n in mask {
      if *n {
        m_str += "1";
      } else {
        m_str += "0";
      }
    }
    m_str
  }

  pub fn boardmask_to_str(&self) -> String {
    let bm_str = MasterTriggerEvent::bitmask_to_str(&self.board_mask);
    bm_str
  }

  pub fn hits_to_str(&self) -> String {
    let mut hits_str = String::from("");
    for j in 0..self.hits.len() {
      hits_str += &(j.to_string() + ": [" + &MasterTriggerEvent::bitmask_to_str(&self.hits[j]) + "]\n");
    }
    hits_str
  }

  /// Get the number of hit paddles from 
  /// the hitmask.
  ///
  /// Now the question is 
  /// what do we consider a hit. 
  /// Currently we have for the LTB threshold
  /// 0 = no hit 
  /// 01 = thr1
  /// 10 = thr2
  /// 11 = thr3
  ///
  /// For now, we just say everything larger 
  /// than 01 is a hit
  pub fn get_hit_paddles(&self) -> u8 {
    let mut n_paddles = 0u8;
    // somehow it is messed up how we iterate over
    // the array (I think this must be reversed.
    // At least for the number of paddles it does 
    // not matter.
    for n in 0..N_LTBS { 
      for ch in (0..N_CHN_PER_LTB -1).step_by(2) {
        if self.hits[n][ch] || self.hits[n][ch+1] {
          n_paddles += 1;
        }
      }
    }
    n_paddles
  }

  pub fn invalidate(&mut self) {
    self.valid = false;
  }
}

impl Serialization for MasterTriggerEvent {

  fn from_bytestream(bytestream : &Vec<u8>,
                     start_pos  : usize)
    -> Result<Self, SerializationError> {
    let bs = bytestream;
    let pos = start_pos;
    let mt = MasterTriggerEvent::new(0,0);
    let header = u16::from_le_bytes([bs[pos],bs[pos + 1]]); 
    if header != MasterTriggerEvent::HEAD {
      return Err(SerializationError::HeadInvalid);
    }
    //bs.extend_from_slice(&MasterTriggerEvent::HEAD.to_le_bytes());
    //bs.extend_from_slice(&self.event_id.to_le_bytes()); 
    //bs.extend_from_slice(&self.timestamp.to_le_bytes());
    //bs.extend_from_slice(&self.tiu_timestamp.to_le_bytes());
    //bs.extend_from_slice(&self.tiu_gps_32.to_le_bytes());
    //bs.extend_from_slice(&self.tiu_gps_16.to_le_bytes());
    //bs.extend_from_slice(&self.n_paddles.to_le_bytes());

    Ok(mt)
  }

}

impl Default for MasterTriggerEvent {
  fn default() -> MasterTriggerEvent {
    MasterTriggerEvent::new(0,0)
  }
}

impl fmt::Display for MasterTriggerEvent {
  fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
    write!(f, "<MasterTriggerEvent\n event id\t {}\n timestamp\t {}\n tiu_timestamp\t {}\n tiu_gps_32\t {}\n tiu_gps_16\t {}\n n paddles\t {}\n boardmask\t {}\n hits {}\n crc\t {} >",
           self.event_id, self.timestamp, self.tiu_timestamp,
           self.tiu_gps_32, self.tiu_gps_16, self.get_hit_paddles(),
           self.boardmask_to_str(), self.hits_to_str(), self.crc)
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
  // this supports up to 100 Hz
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


/// Check if the MTB DAQ has new information 
pub fn daq_word_available(socket : &UdpSocket,
                          target_address : &str,
                          buffer : &mut [u8;MT_MAX_PACKSIZE]) 
    -> Result<bool, Box<dyn Error>> {
    //if 0 == (read_register(socket, target_address, 0x12) & 0x2):
    let queue = read_register(socket, target_address, 0x12, buffer)?;
    let not_empty = queue & 0x2;
    Ok(not_empty == 0)
}

/// Helper to get the number of the triggered LTB from the bitmask
pub fn decode_board_mask(board_mask : u32) -> [bool;N_LTBS] {
  let mut decoded_mask = [false;N_LTBS];
  // FIXME this implicitly asserts that the fields for non available LTBs 
  // will be 0 and all the fields will be in order 
  let mut index = N_LTBS - 1;
  for n in 0..N_LTBS {
    let mask = 1 << n;
    let bit_is_set = (mask & board_mask) > 0;
    decoded_mask[index] = bit_is_set;
    if index != 0 {
        index -= 1;
    }
  }
  decoded_mask
}

/// Helper to get the number of the triggered LTB from the bitmask
pub fn decode_hit_mask(hit_mask : u32) -> ([bool;N_CHN_PER_LTB],[bool;N_CHN_PER_LTB]) {
  let mut decoded_mask_0 = [false;N_CHN_PER_LTB];
  let mut decoded_mask_1 = [false;N_CHN_PER_LTB];
  // FIXME this implicitly asserts that the fields for non available LTBs 
  // will be 0 and all the fields will be in order
  let mut index = N_CHN_PER_LTB - 1;
  for n in 0..N_CHN_PER_LTB {
    let mask = 1 << n;
    let bit_is_set = (mask & hit_mask) > 0;
    decoded_mask_0[index] = bit_is_set;
    if index != 0 {
      index -= 1;
    }
  }
  index = N_CHN_PER_LTB -1;
  for n in N_CHN_PER_LTB..2*N_CHN_PER_LTB {
    let mask = 1 << n;
    let bit_is_set = (mask & hit_mask) > 0;
    decoded_mask_1[index] = bit_is_set;
    if index != 0 {
      index -= 1;
    }
  }
  (decoded_mask_0, decoded_mask_1)
}


/// Read a word from the DAQ package, making sure 
/// the queue is non-empty
///
pub fn read_daq_word(socket : &UdpSocket,
                     target_address : &str,
                     buffer : &mut [u8;MT_MAX_PACKSIZE])
  -> Result<u32, Box<dyn Error>> {
  let ntries = 100;
  for _ in 0..ntries {
    match daq_word_available(socket, target_address, buffer) {
      Err(err) => {
        trace!("No DAQ word available, error {err}");
        continue;
      }
      Ok(has_data) => {
        if has_data {
          let word = read_register(socket, target_address, 0x11, buffer)?;
          return Ok(word)
        } else {
          continue;
        }
      }
    }
  }
  return Err(Box::new(MasterTriggerError::DAQNotAvailable));
}


/// Read the IPBus packets from the MTB DAQ
///
/// FIXME This will only work if there is a 
/// DAQ packet ready, so it has to work 
/// together with a check that the daq queue
/// is full
///
/// # Arguments:
/// 
/// * socket         : An open Udp socket on the host side
/// * target_address : The IP address of the MTB
/// * buffer         : allocated memory for the MTB response
pub fn read_daq(socket : &UdpSocket,
                target_address : &str,
                buffer : &mut [u8;MT_MAX_PACKSIZE])
  -> Result<MasterTriggerEvent, Box<dyn Error>> {

  let board_mask           : u32;
  // board means ltb here. Hits are hits 
  // on ltbs. ltbs have 16 channels!
  let decoded_board_mask : [bool;N_LTBS];
  //let hits         = [[false;N_CHN_PER_LTB];N_LTBS];
  let mut hits_a       : [bool;N_CHN_PER_LTB];
  let mut hits_b       : [bool;N_CHN_PER_LTB];

  let n_ltbs        : u32;
  let mut trailer   : u32;
  
  // How this works is the following. We read the data register
  // until we get the header word. Then we have a new event 
  // and we fill the values of our MasterTriggerEvent by 
  // subsequently reading out the same register again
  // this will eventually determin, 
  // how often we will read the 
  // hit register
  let ntries = 100;
  let mut event = MasterTriggerEvent::new(0, 0);
  let mut head_found = false;
  for _ in 0..ntries {
    if head_found {
      // let mut paddles_rxd = 1;
      // we start a new daq package
      event.event_id        = read_daq_word(socket, target_address, buffer)?;
      if event.event_id == 0 {
        return Err(Box::new(MasterTriggerError::DAQNotAvailable));
      }
      event.timestamp         = read_daq_word(socket, target_address, buffer)?;
      event.tiu_timestamp     = read_daq_word(socket, target_address, buffer)?;
      event.tiu_gps_32        = read_daq_word(socket, target_address, buffer)?;
      event.tiu_gps_16        = read_daq_word(socket, target_address, buffer)?;
      board_mask              = read_daq_word(socket, target_address, buffer)?;
      decoded_board_mask      = decode_board_mask(board_mask);
      //println!(" decoded mask {decoded_board_mask:?}");
      event.board_mask = decoded_board_mask;
      n_ltbs = board_mask.count_ones();
      trace!("{n_ltbs} LTBs participated in this event");
      // to get the hits, we need to read the hit field.
      // Two boards can fit into a single hit field, that 
      // needs we have to read out the hit filed boards/2
      // times or boards/2 + 1 in case boards is odd.
      let mut queries_needed = n_ltbs as usize;
      let mut queried_boards = Vec::<u8>::new();
      let mut nhit_query = 0usize;
      if n_ltbs % 2 == 0 {
        queries_needed = n_ltbs as usize/2;
      } else {
        queries_needed = n_ltbs as usize/2 + 1;
      }
      //let mut queries_needed = 0;
      //let mut queried_boards = Vec::<u8>::new();
      //for n in (0..20).step_by(2) {
      //  if decoded_board_mask[n+1] || decoded_board_mask[n] {
      //    queries_needed += 1;
      //    queried_boards.push(n as u8);
      //  }
      //}
      //if queries_needed % 2 == 0 {
      //    queries_needed = queries_needed /2;
      //} else {
      //    queries_needed = queries_needed /2 + 1;
      //}

      trace!("We need {queries_needed} queries for the hitmask");
      while nhit_query < queries_needed { 
        let hitmask = read_daq_word(socket, target_address, buffer)?;
        if hitmask == 0x55555555 {
          error!("We should se a hitmask, but we saw the end of the event");
        }
        trace!("Got hitmask {hitmask}");
        (hits_a, hits_b) = decode_hit_mask(hitmask);
        let n = queried_boards[nhit_query] as usize;
        event.hits[n+1] = hits_a;
        event.hits[n] = hits_b;
        nhit_query += 1; 
      }
      //for n in (0..20).step_by(2) {
      //  println!("{n}");
      //  if decoded_board_mask[n+1] || decoded_board_mask[n] {
      //    let hitmask = read_daq_word(socket, target_address, buffer)?;
      //    if hitmask == 0x55555555 {
      //      error!("We should se a hitmask, but we saw the end of the event");
      //    }
      //    trace!("Got hitmask {hitmask}");
      //    (hits_a, hits_b) = decode_hit_mask(hitmask);
      //    event.hits[n+1] = hits_a;
      //    event.hits[n] = hits_b;
      //    nhit_query += 1; 
      //  }
      //}
      //} // end for loop
    trace!("{:?}", decoded_board_mask);

    trace!("n queries {nhit_query}");
    event.crc         = read_daq_word(socket, target_address, buffer)?;
    if event.crc == 0x55555555 {
      error!("CRC field corrupt, but we carry on!");
      event.broken = true;
      return Ok(event);
    }
    trailer     = read_daq_word(socket, target_address, buffer)?;
    if trailer != 0x55555555 {
      if trailer == 0xAAAAAAAA {
        error!("New header found while we were not done with the old event!");
      }
      event.broken = true;
      //error!("Broken package for event id {}, trailer corrupt {}", event.event_id, trailer);
      trailer     = read_daq_word(socket, target_address, buffer)?;
      if trailer == 0x55555555 {
        //println!("{:?}", decoded_board_mask);
        //for n in queried_boards.iter() {
        //    println!("{:?}", event.hits[*n as usize]);
        ////println!("{:?}", event.hits[n+1]);
        //}  
        //println!("{queries_needed}");
        //println!("{nhit_query}");
        //error!("Checking again, we found the trailer!");
      }
      return Ok(event);
      //return Err(Box::new(MasterTriggerError::BrokenPackage));
    }
    return Ok(event);
    }
    
    let word = read_daq_word(socket, target_address, buffer)?; 
    if word == 0xAAAAAAAA {
      head_found = true;
    }

  } // end loop over n-tries
  return Err(Box::new(MasterTriggerError::DAQNotAvailable));
}


