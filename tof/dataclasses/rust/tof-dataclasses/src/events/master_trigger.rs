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
//use std::collections::HashMap;

use crate::serialization::{
    Serialization,
    SerializationError,
    search_for_u16,
    parse_u8,
    parse_u16,
    parse_u32,
    parse_u64,
};

//use crate::DsiLtbRBMapping;
use crate::events::EventStatus;

//use crate::events::RBMissingHit;
//use crate::constants::{
//  N_LTBS,
//  N_CHN_PER_LTB,
//};

//he default values used where thus:
//  INNER_TOF_THRESH = 3
//  OUTER_TOF_THRESH = 3
//  TOTAL_TOF_THRESH =8
//  REQUIRE_BETA =1
//
//so this corresponds to the BETA being set (required) and the loose settings for the number of hits.
//Is this correct?
//
//If so, at some point (not yet because we are not getting data through the system), I'd like us to run
//for a while with these three settings:
//
//  INNER_TOF_THRESH = 3
//  OUTER_TOF_THRESH = 3
//  TOTAL_TOF_THRESH =8
//  REQUIRE_BETA =1
//
//  INNER_TOF_THRESH = 3
//  OUTER_TOF_THRESH = 3
//  TOTAL_TOF_THRESH =8
//  REQUIRE_BETA =0
//
//  INNER_TOF_THRESH = 0
//  OUTER_TOF_THRESH = 0
//  TOTAL_TOF_THRESH =0
//  REQUIRE_BETA =1
//
//  This is from Andrew's email about Philip's debugging triggers:
//  I am proposing to just add a single new trigger, which is configured by:
//  
//  cube_side_thresh   
//  cube_top_thresh    
//  cube_bot_thresh    
//  cube_corner_thresh 
//  umbrella_thresh    
//  cortina_thresh     
//  inner_tof_thresh 
//  outer_tof_thresh
//  total_tof_thresh 
//  
//  The trigger is just
//  
//  cube_side_cnt >= cube_side_thresh AND cube_top_cnt >= cube_top_thresh AND .... etc.
//  
//  So setting thresh to zero disables a condition, and should let you implement any of these combinations except 3, which would need some new parameter.


/// masks to decode LTB hit masks
const LTB_CH0 : u16 = 0x3   ;
const LTB_CH1 : u16 = 0xc   ;
const LTB_CH2 : u16 = 0x30  ; 
const LTB_CH3 : u16 = 0xc0  ;
const LTB_CH4 : u16 = 0x300 ;
const LTB_CH5 : u16 = 0xc00 ;
const LTB_CH6 : u16 = 0x3000;
const LTB_CH7 : u16 = 0xc000;
const LTB_CHANNELS : [u16;8] = [
    LTB_CH0,
    LTB_CH1,
    LTB_CH2,
    LTB_CH3,
    LTB_CH4,
    LTB_CH5,
    LTB_CH6,
    LTB_CH7
];

#[derive(Debug, Copy, Clone, PartialEq, serde::Deserialize, serde::Serialize)]
#[repr(u8)]
pub enum TriggerType {
  Unknown      = 0u8,
  /// -> 1-10 "pysics" triggers
  Gaps         = 4u8,
  Any          = 1u8,
  Track        = 2u8,
  TrackCentral = 3u8,
  /// -> 20+ "Philip's triggers"
  /// Any paddle HIT in UMB  + any paddle HIT in CUB
  UmbCube      = 21u8,
  /// Any paddle HIT in UMB + any paddle HIT in CUB top
  UmbCubeZ     = 22u8,
  /// Any paddle HIT in UMB + any paddle hit in COR + any paddle hit in CUB 
  UmbCorCube   = 23u8,
  /// Any paddle HIT in COR + any paddle HIT in CUB SIDES
  CorCubeSide  = 24u8,
  /// Any paddle hit in UMB + any three paddles HIT in CUB
  Umb3Cube     = 25u8,
  /// > 100 -> Debug triggers
  Poisson      = 100u8,
  Forced       = 101u8, 
}

impl fmt::Display for TriggerType {
  fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
    let r = serde_json::to_string(self).unwrap_or(
      String::from("ERROR: DeserializationError!"));
    write!(f, "<TriggerType: {}>", r)
  }
}

impl From<u8> for TriggerType {
  fn from(value: u8) -> Self {
    match value {
      0   => TriggerType::Unknown,
      100 => TriggerType::Poisson,
      1   => TriggerType::Any,
      2   => TriggerType::Track,
      3   => TriggerType::TrackCentral,
      4   => TriggerType::Gaps,
      21  => TriggerType::UmbCube,
      22  => TriggerType::UmbCubeZ,
      23  => TriggerType::UmbCorCube,
      24  => TriggerType::CorCubeSide,
      25  => TriggerType::Umb3Cube,
      _   => TriggerType::Unknown
    }
  }
}

#[cfg(feature = "random")]
impl FromRandom for TriggerType {
  
  fn from_random() -> Self {
    let choices = [
      TriggerType::Unknown,
      TriggerType::Poisson,
      TriggerType::Any,
      TriggerType::Track,
      TriggerType::TrackCentral,
      TriggerType::Gaps,
      TriggerType::Forced,
      TriggerType::UmbCube,
      TriggerType::UmbCubeZ,
      TriggerType::UmbCorCube,
      TriggerType::CorCubeSide,
      TriggerType::Umb3Cube,
    ];
    let mut rng  = rand::thread_rng();
    let idx = rng.gen_range(0..choices.len());
    choices[idx]
  }
}

/////////////////////////////////////////////////

/// LTB Thresholds as passed on by the MTB
/// [See also](https://gaps1.astro.ucla.edu/wiki/gaps/images/gaps/5/52/LTB_Data_Format.pdf)
#[derive(Debug, Copy, Clone, PartialEq, serde::Deserialize, serde::Serialize)]
#[repr(u8)]
pub enum LTBThreshold {
  NoHit = 0u8,
  /// First threshold, 40mV, about 0.75 minI
  Hit   = 1u8,
  /// Second threshold, 32mV (? error in doc ?, about 2.5 minI
  Beta  = 2u8,
  /// Third threshold, 375mV about 30 minI
  Veto  = 3u8,
  /// Use u8::MAX for Unknown, since 0 is pre-determined for 
  /// "NoHit, 
  Unknown = 255u8
}

impl fmt::Display for LTBThreshold {
  fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
    let r = serde_json::to_string(self).unwrap_or(
      String::from("ERROR: DeserializationError!"));
    write!(f, "<LTBThreshold: {}>", r)
  }
}

impl From<u8> for LTBThreshold {
  fn from(value: u8) -> Self {
    match value {
      0 => LTBThreshold::NoHit,
      1 => LTBThreshold::Hit,
      2 => LTBThreshold::Beta,
      3 => LTBThreshold::Veto,
      _ => LTBThreshold::Unknown
    }
  }
}

#[cfg(feature = "random")]
impl FromRandom for LTBThreshold {
  
  fn from_random() -> Self {
    let choices = [
      LTBThreshold::NoHit,
      LTBThreshold::Hit,
      LTBThreshold::Beta,
      LTBThreshold::Veto,
      LTBThreshold::Unknown
    ];
    let mut rng  = rand::thread_rng();
    let idx = rng.gen_range(0..choices.len());
    choices[idx]
  }
}

/////////////////////////////////////////////////

/// Hold additional information about the status
/// of the registers on the MTB
/// FUTURE EXTENSION/WIP
pub struct MTBInfo {
  pub tiu_emulation_mode : bool,
  pub tiu_bad            : bool,
  pub dsi_status         : [bool;5],
  pub rb_int_window      : u32,
  pub read_all_rbchan    : bool,
  pub gaps_trig_en       : bool,
  pub require_beta       : bool,
  pub trigger_rate       : bool,
  pub lost_trigger_rate  : bool,
  pub inner_tof_thresh   : u32,
  pub outer_tof_thresh   : u32,
  pub total_tof_thresh   : u32,
  pub any_trig_is_glob   : bool,
  pub track_trig_is_glob : bool
}


/// An event as observed by the MTB
///
/// This is condensed to the most 
/// crucial information 
///
/// FIXME : implementation of absolute time
#[derive(Debug, Clone, PartialEq)]
pub struct MasterTriggerEvent {
  pub event_status   : EventStatus,
  pub event_id       : u32,
  /// Internal timestamp at the time of trigger (1 unit = 10 ns)
  /// Free running counter, rolling over every ~42 seconds
  pub timestamp      : u32,
  /// Timestamp at the edge of the TIU GPS (1 unit = 10 ns)
  pub tiu_timestamp  : u32,
  /// Second received from the TIU (format?) 
  pub tiu_gps32      : u32,
  pub tiu_gps16      : u16,
  pub crc            : u32,
  // NEW - change/extension in API for MTB fw >= 3.0.0
  /// Trigger source:  
  pub trigger_source : u16,
  pub dsi_j_mask     : u32,
  pub channel_mask   : Vec<u16>,
  pub mtb_link_mask  : u64,
}

impl MasterTriggerEvent {
  /// Implementation version, might roughly 
  /// correspond to fw version
  pub const VERSION : u8 = 3;

  pub fn new() -> Self {
    Self { 
      event_status   : EventStatus::Unknown,
      event_id       : 0,
      timestamp      : 0,
      tiu_timestamp  : 0,
      tiu_gps32      : 0,
      tiu_gps16      : 0,
      crc            : 0,
      trigger_source : 0,
      dsi_j_mask     : 0,
      channel_mask   : Vec::<u16>::new(),
      mtb_link_mask  : 0,
    }   
  }

  /// Get the RB link IDs according to the mask
  pub fn get_rb_link_ids(&self) -> Vec<u8> {
    let mut links = Vec::<u8>::new();
    for k in 0..64 {
      if (self.mtb_link_mask >> k) as u64 & 0x1 == 1 {
        links.push(k as u8);
      }
    }
    links
  }

  /// Get the combination of triggered DSI/J/CH on 
  /// the MTB which formed the trigger. This does 
  /// not include further hits which fall into the 
  /// integration window. For those, se rb_link_mask
  ///
  /// The returned values follow the TOF convention
  /// to start with 1, so that we can use them to 
  /// look up LTB ids in the db.
  ///
  /// # Returns
  ///
  ///   Vec<(hit)> where hit is (DSI, J, CH) 
  pub fn get_trigger_hits(&self) -> Vec<(u8, u8, u8, LTBThreshold)> {
    let mut hits = Vec::<(u8,u8,u8,LTBThreshold)>::new(); 
    //let n_masks_needed = self.dsi_j_mask.count_ones() / 2 + self.dsi_j_mask.count_ones() % 2;
    let n_masks_needed = self.dsi_j_mask.count_ones();
    if self.channel_mask.len() < n_masks_needed as usize {
      error!("We need {} hit masks, but only have {}! This is bad!", n_masks_needed, self.channel_mask.len());
      return hits;
    }
    let mut n_mask = 0;
    trace!("Expecting {} hit masks", n_masks_needed);
    trace!("ltb channels {:?}", self.dsi_j_mask);
    trace!("hit masks {:?}", self.channel_mask); 
    //println!("We see LTB Channels {:?} with Hit masks {:?} for {} masks requested by us!", self.dsi_j_mask, self.channel_mask, n_masks_needed);
    for k in 0..32 {
      if (self.dsi_j_mask >> k) as u32 & 0x1 == 1 {
        let dsi = (k as f32 / 4.0).floor() as u8 + 1;       
        let j   = (k % 5) as u8 + 1;
        //println!("n_mask {n_mask}");
        let channels = self.channel_mask[n_mask]; 
        for (i,ch) in LTB_CHANNELS.iter().enumerate() {
          //let chn = *ch as u8 + 1;
          let chn = i as u8 + 1;
          //println!("i,ch {}, {}", i, ch);
          let thresh_bits = ((channels & ch) >> (i*2)) as u8;
          //println!("thresh_bits {}", thresh_bits);
          if thresh_bits > 0 { // hit over threshold
            hits.push((dsi, j, chn, LTBThreshold::from(thresh_bits)));
          }
        }
        n_mask += 1;
      }
    }
    hits
  }

  ///// Compatibility with older data.
  ///// Convert deprecated array type format
  ///// to new system
  //fn get_dsi_j_mask_from_old_data(&mut self, mask : u32) {
  //  // if I am not completly mistaken, this can be saved 
  //  // directly
  //  self.dsi_j_mask = mask;
  //}

  ///// Compatiblity with older data.
  ///// Convert deprecated array type format
  ///// to new system
  //fn get_channel_mask_from_old_data(&mut self, mask : u32) {
  //  self.channel_mask.push(mask as u16); 
  //}

  /// combine the tiu gps 16 and 32bit timestamps 
  /// into a 48bit timestamp
  pub fn get_timestamp_gps48(&self) -> u64 {
    ((self.tiu_gps16 as u64) << 32) | self.tiu_gps32 as u64 
  }

  /// Get absolute timestamp as sent by the GPS
  pub fn get_timestamp_abs48(&self) -> u64 {
    let gps = self.get_timestamp_gps48();
    let mut timestamp = self.timestamp;
    if timestamp < self.tiu_timestamp {
      // it has wrapped
      timestamp += u32::MAX;
    }
    let ts  = 1_000_000_000 * gps + (timestamp - self.tiu_timestamp) as u64;
    return ts;
  }

  /// Get the trigger sources from trigger source byte
  /// FIXME! (Does not return anything)
  pub fn get_trigger_sources(&self) -> Vec<TriggerType> {
    let mut t_types    = Vec::<TriggerType>::new();
    let gaps_trigger   = self.trigger_source >> 5 & 0x1 == 1;
    if gaps_trigger {
      t_types.push(TriggerType::Gaps);
    }
    let any_trigger    = self.trigger_source >> 6 & 0x1 == 1;
    if any_trigger {
      t_types.push(TriggerType::Any);
    }
    let forced_trigger = self.trigger_source >> 7 & 0x1 == 1;
    if forced_trigger {
      t_types.push(TriggerType::Forced);
    }
    let track_trigger  = self.trigger_source >> 8 & 0x1 == 1;
    if track_trigger {
      t_types.push(TriggerType::Track);
    }
    let central_track_trigger
                       = self.trigger_source >> 9 & 0x1 == 1;
    if central_track_trigger {
      t_types.push(TriggerType::TrackCentral);
    }
    t_types
  }
}

impl Serialization for MasterTriggerEvent {
  
  /// Variable size
  const SIZE : usize = 0;
  const TAIL : u16   = 0x5555;
  const HEAD : u16   = 0xAAAA;

  fn to_bytestream(&self) -> Vec::<u8> {
    let mut bs = Vec::<u8>::with_capacity(MasterTriggerEvent::SIZE);
    bs.extend_from_slice(&MasterTriggerEvent::HEAD.to_le_bytes());
    bs.push(self.event_status as u8);
    bs.extend_from_slice(&self.event_id.to_le_bytes()); 
    bs.extend_from_slice(&self.timestamp.to_le_bytes());
    bs.extend_from_slice(&self.tiu_timestamp.to_le_bytes());
    bs.extend_from_slice(&self.tiu_gps32.to_le_bytes());
    bs.extend_from_slice(&self.tiu_gps16.to_le_bytes());
    bs.extend_from_slice(&self.crc.to_le_bytes());
    bs.extend_from_slice(&self.trigger_source.to_le_bytes());
    bs.extend_from_slice(&self.dsi_j_mask.to_le_bytes());
    let n_channel_masks = self.channel_mask.len();
    bs.push(n_channel_masks as u8);
    for k in 0..n_channel_masks {
      bs.extend_from_slice(&self.channel_mask[k].to_le_bytes());
    }
    bs.extend_from_slice(&self.mtb_link_mask.to_le_bytes());
    bs.extend_from_slice(&MasterTriggerEvent::TAIL.to_le_bytes());
    bs
  }

  fn from_bytestream(stream : &Vec<u8>,
                     pos    : &mut usize)
    -> Result<Self, SerializationError> {
    let mut mt = Self::new();
    let header = parse_u16(stream, pos); 
    if header != Self::HEAD {
      return Err(SerializationError::HeadInvalid);
    }
    mt.event_status       = parse_u8 (stream, pos).into();
    mt.event_id           = parse_u32(stream, pos);
    mt.timestamp          = parse_u32(stream, pos);
    mt.tiu_timestamp      = parse_u32(stream, pos);
    mt.tiu_gps32          = parse_u32(stream, pos);
    mt.tiu_gps16          = parse_u16(stream, pos);
    mt.crc                = parse_u32(stream, pos);
    mt.trigger_source     = parse_u16(stream, pos);
    mt.dsi_j_mask         = parse_u32(stream, pos);
    let n_channel_masks   = parse_u8(stream, pos);
    for _ in 0..n_channel_masks {
      mt.channel_mask.push(parse_u16(stream, pos));
    }
    mt.mtb_link_mask      = parse_u64(stream, pos);
    let tail              = parse_u16(stream, pos);
    if tail != Self::TAIL {
      error!("Invalid tail signature {}!", tail);
      mt.event_status = EventStatus::TailWrong;
      // PATCH - if this is old data, just skip it and
      // search the next tail
      match search_for_u16(Self::TAIL, stream, *pos) {
        Ok(tail_pos) => {
          error!("The tail was invalid, but we found a suitable end marker. The data format seems incompatible though, so the MasterTriggerEvents is probably rubbish!");
          mt.event_status = EventStatus::IncompatibleData; 
          *pos = tail_pos + 2;
        },
        Err(err) => {
          error!("Tail invalid, we assume the data format is incompatible, however, we could not do anything about it! {err}");
        }
      }
    }
    Ok(mt)
  }
}

impl Default for MasterTriggerEvent {
  fn default() -> Self {
    Self::new()
  }
}

impl fmt::Display for MasterTriggerEvent {
  fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
    let mut repr = String::from("<MasterTriggerEvent");
    repr += &(format!("\n  EventStatus     : {}", self.event_status));
    repr += &(format!("\n  EventID         : {}", self.event_id));
    repr += "\n  ** trigger sources **";
    for k in self.get_trigger_sources() {
      repr += &(format!("\n   {}", k));
    }
    repr += "\n  ** ** timestamps ** **";
    repr += &(format!("\n    timestamp     : {}", self.timestamp));
    repr += &(format!("\n    tiu_timestamp : {}", self.tiu_timestamp));
    repr += &(format!("\n    gps 48bit     : {}", self.get_timestamp_gps48()));
    repr += &(format!("\n    absolute 48bit: {}", self.get_timestamp_abs48()));
    repr += "\n  -- -- --";
    repr += &(format!("\n  crc             : {}", self.crc));
    repr += &(format!("\n  ** ** TRIGGER HITS (DSI/J/CH) [{} LTBS] ** **", self.dsi_j_mask.count_ones()));
    for k in self.get_trigger_hits() {
      repr += &(format!("\n  => {}/{}/{} ({}) ", k.0, k.1, k.2, k.3));
    }
    repr += "\n  ** ** MTB LINK IDs ** **";
    let mut mtblink_str = String::from("\n  => ");
    for k in self.get_rb_link_ids() {
      mtblink_str += &(format!("{} ", k))
    }
    repr += &mtblink_str;
    repr += &(format!("\n  == Trigger hits {}, expected RBEvents {}",
            self.get_trigger_hits().len(),
            self.get_rb_link_ids().len()));
    repr += ">";
    write!(f,"{}", repr)
  }
}

#[cfg(feature="random")]
impl FromRandom for MasterTriggerEvent {

  fn from_random() -> Self {
    let mut event        = Self::new();
    let mut rng          = rand::thread_rng();
    // FIXME - P had figured out how to this, copy his approach
    //event.event_status   = rng.gen::<u8><();
    event.event_id       = rng.gen::<u32>();
    event.timestamp      = rng.gen::<u32>();
    event.tiu_timestamp  = rng.gen::<u32>();
    event.tiu_gps32      = rng.gen::<u32>();
    event.tiu_gps16      = rng.gen::<u16>();
    event.crc            = rng.gen::<u32>();
    event.trigger_source = rng.gen::<u16>();
    event.dsi_j_mask     = rng.gen::<u32>();
    let n_channel_masks  = rng.gen::<u8>();
    for _ in 0..n_channel_masks {
      event.channel_mask.push(rng.gen::<u16>());
    }
    event.mtb_link_mask  = rng.gen::<u64>();
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

