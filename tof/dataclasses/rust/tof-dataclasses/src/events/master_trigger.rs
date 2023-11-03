//! MasterTriggerEvent

cfg_if::cfg_if! {
  if #[cfg(feature = "random")]  {
    use crate::FromRandom;
    extern crate rand;
    use rand::Rng;
  }
}

use std::fmt;
//use std::time::Duration;
use std::collections::HashMap;

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
const N_LTBS : usize = 20;
const N_CHN_PER_LTB : usize = 16;


/////////////////////////////////////////////////

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
  pub fn get_ltb_ids(&self) 
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
  valid        : bool,
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

  pub fn get_n_rbs_expected(&self) -> u8 {
    let mut n_rbs = 0u8;
    //println!("SELF HITS : {:?}", self.hits);
    for h in self.hits {
      let count = h.iter().filter(|&&x| x).count();
      //println!("COUNT! {count}"); 
      if count >= 1 {
        n_rbs += 1;
      } 
      if count > 8 {
        n_rbs += 1;
      }
    }
    n_rbs
  }

  /// Make the connection between the triggered
  /// boards in the boardmask and convert that
  /// to DSI/J
  pub fn get_dsi_j_ch_for_triggered_ltbs(&self) -> Vec<(u8,u8,u8)> {
    let mut dsi_js = Vec::<(u8,u8,u8)>::new();
    let mut dsi = 1u8;
    let mut j   = 1u8;
    let mut ch  : u8;
    for k in 0..N_LTBS {
      if self.board_mask[k] {
        ch = 1;
        for n in 0..self.hits[k].len() {
          if self.hits[k][n] {
            dsi_js.push((dsi, j, ch));
          }
          ch += 1;
        }
      }
      j += 1;
      if j > 5 {
        j = 1;
        dsi += 1;
      }
    }
    dsi_js
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
    let mut repr = String::from("<MasterTriggerEvent");

    repr += "\n\t event_id                    ";
    repr += &self.event_id.to_string(); 
    repr += "\n\t timestamp                   ";
    repr += &self.timestamp.to_string(); 
    repr += "\n\t tiu_timestamp               ";
    repr += &self.tiu_timestamp.to_string(); 
    repr += "\n\t tiu_gps_32                  ";
    repr += &self.tiu_gps_32.to_string(); 
    repr += "\n\t tiu_gps_16                  ";
    repr += &self.tiu_gps_16.to_string(); 
    repr += "\n\t n_paddles                   ";
    repr += &self.n_paddles.to_string(); 
    repr += "\n\t crc                         ";
    repr += &self.crc.to_string();
    repr += "\n\t broken                      ";
    repr += &self.broken.to_string();
    repr += "\n\t valid                       ";
    repr += &self.valid.to_string();
    repr += "\n -- hit mask --";
    repr += "\n [DSI/J]";
    repr += "\n 1/1 - 1/2 - 1/3 - 1/4 - 1/5 - 2/1 - 2/2 - 2/3 - 2/4 - 2/5 - 3/1 - 3/2 - 3/3 - 3/4 - 3/5 - 4/1 - 4/2 - 4/3 - 4/4 - 4/5 \n";
    let mut hit_boards = Vec::<u8>::with_capacity(20);
    let mut dsi_j = HashMap::<u8, &str>::new();
    dsi_j.insert(0  , "1/1");
    dsi_j.insert(1  , "1/2");
    dsi_j.insert(2  , "1/3");
    dsi_j.insert(3  , "1/4");
    dsi_j.insert(4  , "1/5");
    dsi_j.insert(5  , "2/1");
    dsi_j.insert(6  , "2/2");
    dsi_j.insert(7  , "2/3");
    dsi_j.insert(8  , "2/4");
    dsi_j.insert(9  , "2/5");
    dsi_j.insert(10 , "3/1");
    dsi_j.insert(11 , "3/2");
    dsi_j.insert(12 , "3/3");
    dsi_j.insert(13 , "3/4");
    dsi_j.insert(14 , "3/5");
    dsi_j.insert(15 , "4/1");
    dsi_j.insert(16 , "4/2");
    dsi_j.insert(16 , "4/3");
    dsi_j.insert(17 , "4/4");
    dsi_j.insert(19 , "4/5");
    repr += " ";
    println!("SELFBOARDMASK {:?}", self.board_mask);
    for k in 0..N_LTBS {
      if self.board_mask[k] {
        repr += "-X-   ";
        hit_boards.push(k as u8);
      } else {
        repr += "-0-   ";
      }
    }
    repr += "\n\t == == LTB HITS [BRD CH] == ==\n";
    for  k in hit_boards.iter() {
      repr += "\t DSI/J ";
      repr += dsi_j[k];
      repr += "\t=> ";
      for j in 0..N_CHN_PER_LTB {
        if self.hits[*k as usize][j] {
          repr += " ";
          repr += &(j + 1).to_string();
          repr += " ";
        } else {
          continue;
          //repr += " N.A. ";
        } 
      }
      repr += "\n";
    }  
    repr += ">";
    write!(f,"{}", repr)
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

#[cfg(all(test,feature = "random"))]
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

