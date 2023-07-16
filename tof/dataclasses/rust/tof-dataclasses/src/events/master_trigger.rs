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
#[cfg(feature="random")]
use crate::FromRandom;
#[cfg(feature = "random")]
use rand::Rng;

use std::net::UdpSocket;
use std::fmt;
//use std::time::Duration;

use std::error::Error;
use crate::errors::{IPBusError, MasterTriggerError};

use crate::serialization::{Serialization,
                           SerializationError,
                           parse_u8,
                           parse_u16,
                           parse_u32};

use crate::events::RBMissingHit;

use crate::manifest::{LocalTriggerBoard,
                      ReadoutBoard};
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

/// MasterTrigger related mapping
///
/// Caches ltb/rb relevant information 
/// and can generate rb/ltb id lists
#[derive(Debug, Clone)]
pub struct MasterTriggerMapping {
  pub ltb_list : Vec<LocalTriggerBoard>,
  pub rb_list  : Vec<ReadoutBoard>,

  /// Holds RB id at the position where 
  /// it is supposed to be in the MTB 
  /// trigger mask
  pub ltb_mapping  : [LocalTriggerBoard;N_LTBS]

  // 
  //ltb_rb_mapping : HashMap<u8;ReadoutBoard>;
}

impl MasterTriggerMapping {

  pub fn new(ltb_list : Vec<LocalTriggerBoard>, rb_list : Vec<ReadoutBoard>) 
    -> MasterTriggerMapping {
    let mut mtm = MasterTriggerMapping {
      ltb_list,
      rb_list,
      ltb_mapping : [LocalTriggerBoard::new();N_LTBS]
    };
    mtm.construct_ltb_mapping();
    mtm
  }



  /// Map LTBs to the internal mask to identify which 
  /// LTB has been hit 
  ///
  /// LTB id = RAT id
  pub fn construct_ltb_mapping(&mut self) {
    info!("Construction LTB mapping for {} LTBs", self.ltb_list.len());
    for ltb in &self.ltb_list {
      if ltb.ltb_dsi == 0 && ltb.ltb_j == 0 {
        error!("Found ltb with invalid connection information! {:?}", ltb);
        continue;
      }
      let index = ((ltb.ltb_dsi - 1) * 5) + (ltb.ltb_j - 1);
      self.ltb_mapping[index as usize] = *ltb;
    }
    for k in 0..N_LTBS {
      debug! ("{k} -> {}", self.ltb_mapping[k]);
    }
    //panic!("Uff");
  }

  /// Mapping trigger LTB board mask - LTB ids
  ///
  /// # Arguments:
  ///
  /// * board_mask : The board mask as it comes from the 
  ///                MasterTriggerEvent. Each entry corresponds
  ///                to one LocalTriggerBoard. They are sorted
  ///                by DSI and J, e.g
  ///                [.., DSI_{ltb_1} +J_{ltb_1}]
  pub fn get_ltb_ids(&self, board_mask : &[bool; N_LTBS] ) 
    -> Vec<u8> {
    let mut ltbs = Vec::<u8>::new();
    for k in 0..N_LTBS {
      if self.ltb_mapping[k].ltb_id > 0 {
        ltbs.push(self.ltb_mapping[k].ltb_id)
      }
    }
    ltbs
  }


  /// Get the rb ids for the hits in the MasterTrigger Event
  ///
  /// This is the same as ::get_rb_ids, however, with additional
  /// debug information.
  /// It will also emit missing hits.
  ///
  /// FIXME - in the future, include a "debug" feature.
  pub fn get_rb_ids_debug(&self, 
                          mt_ev   : &MasterTriggerEvent,
                          verbose : bool)
    -> (Vec::<(u8,u8)>,Vec::<RBMissingHit>) {
    let mut missing_hits = Vec::<RBMissingHit>::new();
    let mut rb_ids       = Vec::<(u8,u8)>::new();
    if verbose {
      println!("-- DEBUG - get_rb_ids_debug --");
      println!("--> MTEV BRD MASK {:?}", mt_ev.board_mask);
    }
    let mut board_has_hits : bool;
    for k in 0..mt_ev.board_mask.len() {
      board_has_hits = false;
      if mt_ev.board_mask[k] {
        let hits = mt_ev.hits[k];//ltb_hit_index];
        if verbose {
          println!("--> MTEV HITS {:?}", hits);
        }
        // search for corresponding hits in the hit mask
        for ltb_ch in 0..hits.len() {
          if hits[ltb_ch] {
            board_has_hits = true;
            if verbose {
              println!("--> Found hit at {ltb_ch} [+1] [LTB CH]");
              println!("--> LTB REGISTERED FOR THIS HIT {k} {:?} with id {}", self.ltb_mapping[k], self.ltb_mapping[k].ltb_id);
            }
            let rb_id = self.ltb_mapping[k].get_rb_id(ltb_ch as u8 +1);
            if rb_id == 0 {
              let mut missing       = RBMissingHit::new();
              missing.event_id      = mt_ev.event_id;
              missing.ltb_hit_index = k as u8;
              missing.ltb_id        = self.ltb_mapping[k].ltb_id;
              missing.ltb_dsi       = self.ltb_mapping[k].ltb_dsi;
              missing.ltb_j         = self.ltb_mapping[k].ltb_j;
              missing_hits.push(missing);
              error!("Got invalid rb_id 0, LTB {} at index {k}", self.ltb_mapping[k]);
              continue;
            }
            let rb_ch = self.ltb_mapping[k].get_rb_ch(ltb_ch as u8 +1);
            let id_ch = (rb_id, rb_ch);
            if rb_ids.contains(&id_ch) {
              continue;
            } else {
              rb_ids.push(id_ch);
            }
          }
        }
        if !board_has_hits {
          if verbose {
            println!("--> {}", mt_ev);
          }
          let mut missing       = RBMissingHit::new();
          missing.event_id      = mt_ev.event_id;
          missing.ltb_hit_index = k as u8;
          missing.ltb_id        = self.ltb_mapping[k].ltb_id;
          missing.ltb_dsi       = self.ltb_mapping[k].ltb_dsi;
          missing.ltb_j         = self.ltb_mapping[k].ltb_j;
          missing_hits.push(missing);
          error!("We were expecting hits for LTB {}, but we did not see any!", self.ltb_mapping[k]);
        }
      }
    }
    (rb_ids,missing_hits)
  }

  /// Return the ids of the readoutboards which are connected to the LTBs which participated in the
  /// trigger.
  pub fn get_rb_ids(&self,
                    mt_ev    : &MasterTriggerEvent)
                    //ltb_mask : &[bool; N_LTBS],
                    //hit_mask : [[bool; N_CHN_PER_LTB]; N_LTBS])
    -> Vec<(u8, u8)> {
    let mut rb_ids = Vec::<(u8,u8)>::new();
    let mut board_has_hits : bool;
    for k in 0..mt_ev.board_mask.len() {
      board_has_hits = false;
      if mt_ev.board_mask[k] {
        let hits = mt_ev.hits[k];//ltb_hit_index];
        // search for corresponding hits in the hit mask
        for ltb_ch in 0..hits.len() {
          if hits[ltb_ch] {
            board_has_hits = true;
            let rb_id = self.ltb_mapping[k].get_rb_id(ltb_ch as u8 +1);
            if rb_id == 0 {
              error!("Got invalid rb_id 0, LTB {} at index {k}", self.ltb_mapping[k]);
              continue;
            }
            let rb_ch = self.ltb_mapping[k].get_rb_ch(ltb_ch as u8 +1);
            let id_ch = (rb_id, rb_ch);
            if rb_ids.contains(&id_ch) {
              continue;
            } else {
              rb_ids.push(id_ch);
            }
          }
        }
        if !board_has_hits {
          error!("We were expecting hits for LTB {}, but we did not see any!", self.ltb_mapping[k]);
        }
      }
    }
    rb_ids
  } 
}

/// An event as observed by the MTB
///
/// This is condensed to the most 
/// crucial information 
///
/// FIXME : implementation of absolute time
#[derive(Debug, Copy, Clone, PartialEq)]
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
  const TAIL : u16 = 0x5555;
  const HEAD : u16 = 0xAAAA;

  pub fn new(event_id  : u32, 
             n_paddles : u8) -> Self {
    Self {
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

  pub fn get_triggered_ltb_ids(&self) -> Vec<u8> {
    let mut ltbs = Vec::<u8>::new();
    todo!();
    ltbs
    //for k in 0..N_LTBS {
    //  if board_mask[k] {
    //    ltbs.push(2
    //  }
    //}
  }
  
  pub fn decode_board_mask(&mut self, mask : u32) {
    // FIXME -> This basically inverses the order of the LTBs
    // so bit 0 (rightmost in the mask is the leftmost in the 
    // array
    for i in 0..N_LTBS {
      self.board_mask[i] = (mask & (1 << i)) != 0;
    }
  }

  pub fn decode_hit_mask(&mut self, ltb_idx : usize, mask : u32) {
    for i in 0..N_CHN_PER_LTB {
      self.hits[ltb_idx][i] = (mask & (1 << i)) != 0;
    }
  }

  pub fn is_broken(&self) -> bool {
    self.broken
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

  pub fn n_ltbs(&self) -> u32 {
    let mut nboards = 0;
    for n in self.board_mask { 
      if n {
        nboards += 1;
      }
    }
    nboards
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

  pub fn check(&self) -> bool {
    let good = self.n_paddles == self.get_hit_paddles();
    if !good {
      error!("Missmatch in expected and registered hit paddles! Expected : {}, seen {}", self.n_paddles, self.get_hit_paddles());
    }
    good
  }

  pub fn invalidate(&mut self) {
    self.valid = false;
  }
}

impl Serialization for MasterTriggerEvent {
  
  // 21 + 4 byte board mask + 4*4 bytes hit mask
  // => 25 + 16 = 41 
  // + head + tail
  // 45
  const SIZE : usize = 45;
  const TAIL : u16 = 0x5555;
  const HEAD : u16 = 0xAAAA;

  fn to_bytestream(&self) -> Vec::<u8> {
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


  fn from_bytestream(bytestream : &Vec<u8>,
                     pos        : &mut usize)
    -> Result<Self, SerializationError> {
    let bs     = bytestream;
    let mut mt = Self::new(0,0);
    let header = parse_u16(bs, pos); 
    if header != Self::HEAD {
      return Err(SerializationError::HeadInvalid);
    }
    mt.event_id           = parse_u32(bs, pos);
    mt.timestamp          = parse_u32(bs, pos);
    mt.tiu_timestamp      = parse_u32(bs, pos);
    mt.tiu_gps_32         = parse_u32(bs, pos);
    mt.tiu_gps_16         = parse_u32(bs, pos);
    mt.n_paddles          = parse_u8(bs, pos);
    let board_mask        = parse_u32(bs, pos);
    mt.decode_board_mask(board_mask);
    let mut hit_mask : u32;
    for n in 0..N_LTBS {
      hit_mask = parse_u32(bs, pos);
      mt.decode_hit_mask(n, hit_mask);
    }
    mt.crc                = parse_u32(bs, pos);
    warn!("This is specific to data written with <= 0.6.0 KIHIKIHI! This is a BUG! It needs to be fixed in future versions! Version 0.6.1 should already fix ::to_bytestream, but leaves a modded ::from_bytestream for current analysis.");
    let tail_a              = parse_u8(bs, pos);
    let tail_b              = parse_u8(bs, pos);
    if tail_a == 85 && tail_b == 85 {
      debug!("Correct tail found!");
    }
    else if tail_a == 85 && tail_b == 5 {
      debug!("Tail for version 0.6.0/0.6.1 found");  
    } else {
      error!("Tail is messed up. See comment for version 0.6.0/0.6.1 in CHANGELOG! We got {} {} but were expecting 85 5", tail_a, tail_b);
      //error!("Got {} for MTE tail signature! Expecting {}", tail, MasterTriggerEvent::TAIL);
      return Err(SerializationError::TailInvalid);
    }
    //let tail              = parse_u16(bs, pos);
    //if tail != MasterTriggerEvent::TAIL {
    //  error!("Got {} for MTE tail signature! Expecting {}", tail, MasterTriggerEvent::TAIL);
    //  return Err(SerializationError::TailInvalid);
    //}
    //let hit_mask          = 

    //mt.n_paddles          = parse_u8(bs, pos);
    //bs.extend_from_slice(&self.n_paddles.to_le_bytes());

    Ok(mt)
  }
}

impl Default for MasterTriggerEvent {
  fn default() -> Self {
    Self::new(0,0)
  }
}

impl fmt::Display for MasterTriggerEvent {
  fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
    write!(f, "<MasterTriggerEvent\n event id\t {}\n n boards\t {}\n n paddles\t {}\n hits\t {}\n timestamp\t {}\n tiu_timestamp\t {}\n tiu_gps_32\t {}\n tiu_gps_16\t {}\n boardmask\t {}\n crc\t {} >",
           self.event_id, self.n_ltbs(), self.get_hit_paddles(), self.hits_to_str(),  self.timestamp, self.tiu_timestamp,
           self.tiu_gps_32, self.tiu_gps_16, 
           self.boardmask_to_str(), self.crc)
  }
}

#[cfg(feature="random")]
impl FromRandom for MasterTriggerEvent {

  fn from_random() -> Self {
    let mut event   = Self::new(0,0);
    let mut rng = rand::thread_rng();
    event.event_id      = rng.gen::<u32>();
    event.timestamp     = rng.gen::<u32>();
    event.tiu_timestamp = rng.gen::<u32>();
    event.tiu_gps_32    = rng.gen::<u32>();
    event.tiu_gps_16    = rng.gen::<u32>();
    event.n_paddles     = rng.gen::<u8>(); 
    // broken will not get serializad, so this won't
    // be set randomly here
    for k in 0..N_LTBS {
      event.board_mask[k] = rng.gen::<bool>();
      for j in 0..N_CHN_PER_LTB {
        event.hits[k][j]  = rng.gen::<bool>();
      }
    }
    event
  }
}

//#[derive(Debug, Copy, Clone)]
//pub struct IPBusPacket {
//}
//
//impl IPBusPacket {
//  pub fn new() -> IPBusPacket {
//    todo!();
//    IPBusPacket {}
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

/// Helper function to separate a u32 into two u15
fn extract_values_from_32bit(number: u32) -> (u16, u16) {
  let lower_bits = number as u16;
  let upper_bits = (number >> 16) as u16;
  (lower_bits, upper_bits)
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


/// Read the MTB rate counter
pub fn read_rate(socket : &UdpSocket,
                 target_address : &str,
                 buffer : &mut [u8;MT_MAX_PACKSIZE])
  -> Result<u32, Box<dyn Error>> {
  let rate = read_register(socket, target_address, 0x17, buffer)?;
  trace!("Got MT rate! {} ", rate);
  Ok(rate)
}


/// Convert ADC temp from adc values to Celsius
fn convert_adc_temp(data : u16) -> f32 {
  data as f32 * 503.975 / 4096.0 - 273.15
}

// Convert ADC VCCINT from adc values to Voltage
fn convert_adc_vccint(data : u16) -> f32 {
  3.0 * data as f32 / (2_u32.pow(12-1)) as f32
}

/// Read the ADC temp
pub fn read_adc_temp_and_vccint(socket : &UdpSocket,
                                target_address : &str,
                                buffer : &mut [u8;MT_MAX_PACKSIZE])
  -> Result<(f32, f32), Box<dyn Error>> {
  let value = read_register(socket, target_address, 0x122, buffer)?;
  let (mut adc_temp, vccint) = extract_values_from_32bit(value); 
  // only 12 bit temp
  adc_temp &= 0x0fff;
  let temp_c   = convert_adc_temp(adc_temp);
  let vccint_v = convert_adc_vccint(vccint);
  //let value_bytes = value.to_le_bytes(); 
  trace!("Got ADC temp! {} ", temp_c);
  trace!("Got VCCINT    {} ", vccint_v);
  Ok((temp_c, vccint_v))
}

pub fn read_adc_vccaux_and_vccbram(socket : &UdpSocket,
                                   target_address : &str,
                                   buffer : &mut [u8;MT_MAX_PACKSIZE])
  -> Result<(f32, f32), Box<dyn Error>> {
  let value = read_register(socket, target_address, 0x123, buffer)?;
  let (vccaux, vccbram) = extract_values_from_32bit(value); 
  let vccaux_v  = convert_adc_vccint(vccaux);
  let vccbram_v = convert_adc_vccint(vccbram);
  //let value_bytes = value.to_le_bytes(); 
  trace!("Got VCCAUX  [V]  {} ", vccaux_v);
  trace!("Got VCCBRAM [V]  {} ", vccbram_v);
  Ok((vccaux_v, vccbram_v))
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
    //println!("MASK {:?}", mask);
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
      //println!("{:?}", event.board_mask);
      trace!("{n_ltbs} LTBs participated in this event");
      // to get the hits, we need to read the hit field.
      // Two boards can fit into a single hit field, that 
      // needs we have to read out the hit filed boards/2
      // times or boards/2 + 1 in case boards is odd.
      let queries_needed : usize;//= n_ltbs as usize;
      //let queried_boards = Vec::<u8>::new();
      let mut nhit_query = 0usize;
      if n_ltbs % 2 == 0 {
        queries_needed = n_ltbs as usize/2;
      } else {
        queries_needed = n_ltbs as usize/2 + 1;
      }
      trace!("We need {queries_needed} queries for the hitmask");
      let mut hitmasks = Vec::<[bool;N_CHN_PER_LTB]>::new();
      //println!("NEW HITS");
      while nhit_query < queries_needed { 
        let hitmask = read_daq_word(socket, target_address, buffer)?;
        //println!("HITMASK {:?}", hitmask);

        (hits_a, hits_b) = decode_hit_mask(hitmask);
        // hit mask in reverse order than in the encoded word.    
        hitmasks.push(hits_a);
        hitmasks.push(hits_b);
        //println!("HITMASKS_VEC {:?}", hitmasks);
        nhit_query += 1;
      }
      for k in 0..event.board_mask.len() {
        if event.board_mask[k] {
          let thishits = hitmasks.pop().unwrap();
          //println!("Will assign {:?} for {k}", thishits);
          event.hits[k] = thishits;
          //println!("EVENT HAS HITS ASSIGNED : {:?}", event.hits[k]);
      
        }
      }
    //println!("EVENT HAS HITS ASSIGNED : {:?}", event.hits);
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

#[cfg(test)]
mod test_mastertriggerevent {
  use crate::serialization::Serialization;
  use crate::FromRandom;
  use crate::events::MasterTriggerEvent;
  
  #[test]
  fn serialization_mastertriggerevent() {
    let data = MasterTriggerEvent::from_random();
    let test = MasterTriggerEvent::from_bytestream(&data.to_bytestream(), &mut 0).unwrap();
    assert_eq!(data, test);
  }
}

