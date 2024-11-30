//! Readoutboard binary event formats, former denoted as BLOB (binary large object)
//! 
//! The structure is the following
//!
//!
//! - RBEventHeader  - timestamp, status, len of event, but no channel data. This
//!                    represents compression level 2
//!
//! - RBEvent        - still "raw" event, however, with modified fields 
//!                    (removed superflous ones, changed meaning of some others)
//!                    Each RBEvent has a header and a body which is the channel 
//!                    data. Data in this form represents compression level 1
//!
//! - RBWaveform     - a single waveform from a single RB. This can be used to 
//!                    deconstruct TofEvents so that the flight computer does not
//!                    struggle with the large packet size.
//!
//! 
//! * features: "random" - provides "::from_random" for all structs allowing to 
//!   populate them with random data for tests.
//!

use std::fmt;
use std::collections::HashMap;
use colored::Colorize;

use crate::packets::{
  TofPacket,
  PacketType
};
use crate::events::TofHit;
use crate::constants::{NWORDS, NCHN};
use crate::serialization::{
  u8_to_u16,
  Serialization,
  SerializationError,
  search_for_u16,
  Packable,
  parse_u8,
  parse_u16,
  parse_u32,
};

use crate::events::{
    DataType,
    EventStatus,
};
use crate::errors::{
    UserError,
    CalibrationError,
};
use crate::io::RBEventMemoryStreamer;
use crate::calibrations::{
    RBCalibrations,
    clean_spikes,
};

#[cfg(feature="database")]
use crate::database::ReadoutBoard;

cfg_if::cfg_if! {
  if #[cfg(feature = "random")]  {
    use crate::FromRandom;
    extern crate rand;
    use rand::Rng;
  }
}

/// Squeze the rb channel - paddle mapping into 5 bytes
/// for a single RB
pub struct RBPaddleID {
  /// Paddle connected to RB channel 1/2
  pub paddle_12     : u8,
  /// Paddle connected to RB channel 3/4
  pub paddle_34     : u8,
  /// Paddle connected to RB channel 5/6
  pub paddle_56     : u8,
  /// Paddle connected to RB channel 7/8
  pub paddle_78     : u8,
  /// Order - 1 if the smaller channel is the 
  /// A side, 2, if the smaller channel is the 
  /// B side
  pub channel_order : u8
}

impl Default for RBPaddleID {
  fn default() -> Self {
    Self::new()
  }
}

impl fmt::Display for RBPaddleID {
  fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
    let mut repr = String::from("<RBPaddleID:");
    for k in 1..9 {
      let pid = self.get_paddle_id(k);
    }
    //repr += &(format!(" {}", 
    write!(f, "{}", 1)
  }
}

impl RBPaddleID {
  pub fn new() -> Self {
    RBPaddleID {
      paddle_12     : 0,
      paddle_34     : 0,
      paddle_56     : 0,
      paddle_78     : 0,
      channel_order : 0
    }
  }

  pub fn to_u64(&self) -> u64 {
    let val : u64 = (self.channel_order as u64) << 32 | (self.paddle_78 as u64) << 24 | (self.paddle_56 as u64) << 16 | (self.paddle_34 as u64) << 8 |  self.paddle_12 as u64;
    val
  }

  /// Typically, the A-side will be connected to a lower channel id
  ///
  /// If the order is flipped, the lower channel will be connected to 
  /// the B-side
  ///
  /// # Arguments
  /// * channel : RB channel (1-8)
  pub fn get_order_flipped(&self, channel : u8) -> bool {
    match channel {
      1 | 2 => {
        return (self.channel_order & 0x1) == 1;
      }
      3 | 4 => {
        return (self.channel_order & 0x2) == 1;
      }
      5 | 6 => {
        return (self.channel_order & 0x4) == 1;
      }
      7 | 8 => {
        return (self.channel_order & 0x8) == 1;
      }
      _ => {
        error!("{} is not a valid RB channel!", channel);
        return false;
      }
    }
  }

  pub fn get_order_str(&self, channel : u8) -> String {
    if self.get_order_flipped(channel) {
      return String::from("BA");
    } else {
      return String::from("AB");
    }
  }

  pub fn is_a(&self, channel : u8) -> bool {
    match channel {
      1 => {
        if self.get_order_flipped(channel) {
          return false;
        } else {
          return true
        }
      }
      2 => {
        if self.get_order_flipped(channel) {
          return true;
        } else {
          return false
        }
      }
      3 => {
        if self.get_order_flipped(channel) {
          return false;
        } else {
          return true
        }
      }
      4 => {
        if self.get_order_flipped(channel) {
          return true;
        } else {
          return false
        }
      }
      5 => {
        if self.get_order_flipped(channel) {
          return false;
        } else {
          return true
        }
      }
      6 => {
        if self.get_order_flipped(channel) {
          return true;
        } else {
          return false
        }
      }
      7 => {
        if self.get_order_flipped(channel) {
          return false;
        } else {
          return true
        }
      }
      8 => {
        if self.get_order_flipped(channel) {
          return true;
        } else {
          return false
        }
      }
      _ => {
        error!("{} is not a valid RB channel!", channel);
        return false;
      }
    }
  }

  pub fn from_u64(val : u64) -> Self {
    let paddle_12     : u8 = ((val & 0xFF)) as u8;
    let paddle_34     : u8 = ((val & 0xFF00) >> 8) as u8;
    let paddle_56     : u8 = ((val & 0xFF0000) >> 16) as u8;
    let paddle_78     : u8 = ((val & 0xFF000000) >> 24) as u8; 
    let channel_order : u8 = ((val & 0xFF00000000) >> 32) as u8;
    Self {
      paddle_12,
      paddle_34,
      paddle_56,
      paddle_78,
      channel_order,
    }
  }

  #[cfg(feature="database")]
  pub fn from_rb(&self, rb : ReadoutBoard) {
  }

  /// Get the paddle id together with the information 
  /// if this is the A side
  ///
  /// channel in rb channels (starts at 1)
  pub fn get_paddle_id(&self, channel : u8) -> (u8, bool) {
    let flipped = self.get_order_flipped(channel);
    match channel {
      1 | 2 => {
        return (self.paddle_12, flipped); 
      }
      3 | 4 => {
        return (self.paddle_34, flipped); 
      }
      5 | 6 => {
        return (self.paddle_56, flipped); 
      }
      7 | 8 => {
        return (self.paddle_78, flipped); 
      }
      _ => {
        error!("{} is not a valid RB channel!", channel);
        return (0,false);
      }
    }
  }
}

///// Debug information for missing hits. 
/////
///// These hits have been seen by the MTB, but we are unable to determine where 
///// they are coming from, why they are there or we simply have lost the RB 
///// information for these hits.
//#[deprecated(since = "0.10.0", note="feature was never really used")]
//#[derive(Debug, Copy, Clone, PartialEq)]
//pub struct RBMissingHit {
//  pub event_id      : u32,
//  pub ltb_hit_index : u8,
//  pub ltb_id        : u8,
//  pub ltb_dsi       : u8,
//  pub ltb_j         : u8,
//  pub ltb_ch        : u8,
//  pub rb_id         : u8,
//  pub rb_ch         : u8,
//}
//
//#[allow(deprecated)]
//impl Serialization for RBMissingHit {
//  const HEAD               : u16    = 43690; //0xAAAA
//  const TAIL               : u16    = 21845; //0x5555
//  const SIZE               : usize  = 15; // bytes
//  
//  fn from_bytestream(stream : &Vec<u8>, pos : &mut usize)
//    -> Result<Self, SerializationError> {
//    Self::verify_fixed(stream, pos)?;
//    // verify_fixed already advances pos by 2
//    let mut miss = RBMissingHit::new();
//    miss.event_id      = parse_u32(stream, pos);
//    miss.ltb_hit_index = parse_u8(stream, pos);
//    miss.ltb_id        = parse_u8(stream, pos);
//    miss.ltb_dsi       = parse_u8(stream, pos);
//    miss.ltb_j         = parse_u8(stream, pos);
//    miss.ltb_ch        = parse_u8(stream, pos);
//    miss.rb_id         = parse_u8(stream, pos);
//    miss.rb_ch         = parse_u8(stream, pos);
//    *pos += 2; // account for header in verify_fixed
//    Ok(miss)
//  }
//
//  fn to_bytestream(&self) -> Vec<u8> {
//    let mut stream = Vec::<u8>::with_capacity(Self::SIZE);
//    stream.extend_from_slice(&Self::HEAD.to_le_bytes());
//    stream.extend_from_slice(&self.event_id.to_le_bytes());
//    stream.extend_from_slice(&self.ltb_hit_index.to_le_bytes());
//    stream.extend_from_slice(&self.ltb_id.to_le_bytes());
//    stream.extend_from_slice(&self.ltb_dsi.to_le_bytes());
//    stream.extend_from_slice(&self.ltb_j.to_le_bytes());
//    stream.extend_from_slice(&self.ltb_ch.to_le_bytes());
//    stream.extend_from_slice(&self.rb_id.to_le_bytes());
//    stream.extend_from_slice(&self.rb_ch.to_le_bytes());
//    stream.extend_from_slice(&Self::TAIL.to_le_bytes());
//    stream
//  }
//}
//
//#[allow(deprecated)]
//impl RBMissingHit {
//
//  pub fn new() -> Self {
//    RBMissingHit {
//      event_id      : 0,
//      ltb_hit_index : 0,
//      ltb_id        : 0,
//      ltb_dsi       : 0,
//      ltb_j         : 0,
//      ltb_ch        : 0,
//      rb_id         : 0,
//      rb_ch         : 0,
//    }
//  }
//}
//
//#[allow(deprecated)]
//impl fmt::Display for RBMissingHit {
//  fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
//    write!(f, "<RBMissingHit:
//           \t event ID    {},
//           \t LTB hit idx {}, 
//           \t LTB ID      {}, 
//           \t LTB DSI     {}, 
//           \t LTB J       {}, 
//           \t LTB CHN     {},   
//           \t RB ID       {}, 
//           \t RB CH {}>", 
//           self.event_id      ,
//           self.ltb_hit_index ,
//           self.ltb_id        ,
//           self.ltb_dsi       ,
//           self.ltb_j         ,
//           self.ltb_ch        ,
//           self.rb_id         ,
//           self.rb_ch         )
//  }
//}
//
//#[allow(deprecated)]
//impl Default for RBMissingHit {
//
//  fn default() -> Self {
//    Self::new()
//  }
//}
//
//#[cfg(feature = "random")]
//#[allow(deprecated)]
//impl FromRandom for RBMissingHit {
//    
//  fn from_random() -> Self {
//    let mut miss = Self::new();
//    let mut rng = rand::thread_rng();
//    miss.event_id      = rng.gen::<u32>();
//    miss.ltb_hit_index = rng.gen::<u8>();
//    miss.ltb_id        = rng.gen::<u8>();
//    miss.ltb_dsi       = rng.gen::<u8>();
//    miss.ltb_j         = rng.gen::<u8>();
//    miss.ltb_ch        = rng.gen::<u8>();
//    miss.rb_id         = rng.gen::<u8>();
//    miss.rb_ch         = rng.gen::<u8>();
//    miss
//  }
//}


/// Get the traces for a set of RBEvents
///
/// This will return a cube of 
/// The sice of this cube will be fixed
/// in two dimensions, but not the third
///
/// The rationale of this is to be able 
/// to quickly calculate means over all
/// channels.
///
/// Shape
/// \[ch:9\]\[nevents\]\[adc_bin:1024\]
///
/// # Args:
///   events - events to get the traces from
pub fn unpack_traces_f64(events : &Vec<RBEvent>)
  -> Vec<Vec<Vec<f64>>> {
  let nevents          = events.len();
  let mut traces       = Vec::<Vec::<Vec::<f64>>>::new();
  let mut trace        = Vec::<f64>::with_capacity(NWORDS);
  let mut stop_cells   = Vec::<isize>::new();
  let mut empty_events = Vec::<Vec::<f64>>::new();
  for _ in 0..nevents {
    empty_events.push(trace.clone());
  }
  for ch in 0..NCHN {
    traces.push(empty_events.clone());
    for (k,ev) in events.iter().enumerate() {
      trace.clear();
      stop_cells.push(ev.header.stop_cell as isize);
      for k in 0..NWORDS {
        trace.push(ev.adc[ch][k] as f64);
      }
      traces[ch][k] = trace.clone();
    }
  }
  traces
}

pub fn unpack_traces_f32(events : &Vec<RBEvent>)
  -> Vec<Vec<Vec<f32>>> {
  let nevents          = events.len();
  let mut traces       = Vec::<Vec::<Vec::<f32>>>::new();
  let mut trace        = Vec::<f32>::with_capacity(NWORDS);
  let mut stop_cells   = Vec::<isize>::new();
  let mut empty_events = Vec::<Vec::<f32>>::new();
  for _ in 0..nevents {
    empty_events.push(trace.clone());
  }
  for ch in 0..NCHN {
    traces.push(empty_events.clone());
    for (k,ev) in events.iter().enumerate() {
      trace.clear();
      stop_cells.push(ev.header.stop_cell as isize);
      for k in 0..NWORDS {
        trace.push(ev.adc[ch][k] as f32);
      }
      traces[ch][k] = trace.clone();
    }
  }
  traces
}


/// Event data for each individual ReadoutBoard (RB)
///
/// 
///
#[derive(Debug, Clone, PartialEq)]
pub struct RBEvent {
  pub data_type    : DataType,
  pub status       : EventStatus,
  pub header       : RBEventHeader,
  pub adc          : Vec<Vec<u16>>,
  //pub ch9_adc      : Vec<u16>,
  pub hits         : Vec<TofHit>,
}

impl RBEvent {

  pub fn new() -> Self {
    let mut adc = Vec::<Vec<u16>>::with_capacity(NCHN);
    for _ in 0..NCHN {
      adc.push(Vec::<u16>::new());
    }
    Self {
      data_type    : DataType::Unknown,
      status       : EventStatus::Unknown,
      header       : RBEventHeader::new(),
      adc          : adc,
      //ch9_adc      : Vec::<u16>::new(),
      hits         : Vec::<TofHit>::new(),
    }
  }

  /// Deconstruct the RBEvent to form RBWaveforms
  pub fn get_rbwaveforms(&self) -> Vec<RBWaveform> {
    let mut waveforms = Vec::<RBWaveform>::new();
    //for ch in self.header.get_channels() {
    //  let mut wf     = RBWaveform::new();
    //  wf.rb_id       = self.header.rb_id;
    //  wf.rb_channel  = ch;
    //  wf.event_id    = self.header.event_id;
    //  wf.stop_cell   = self.header.stop_cell;
    //  // FIXME - can we move this somehow instead of 
    //  // cloning?
    //  wf.adc         = self.adc[ch as usize].clone();
    //  waveforms.push(wf);
    //}
    waveforms
  }

  /// Get the datatype from a bytestream when we know
  /// that it is an RBEvent
  ///
  /// The data type is encoded in byte 3
  pub fn extract_datatype(stream : &Vec<u8>) -> Result<DataType, SerializationError> {
    if stream.len() < 3 {
      return Err(SerializationError::StreamTooShort);
    }
    // TODO This might panic! Is it ok?
    Ok(DataType::try_from(stream[2]).unwrap_or(DataType::Unknown))
  }
  
  /// decode the len field in the in memroy represention of 
  /// RBEventMemoryView
  pub fn get_channel_packet_len(stream : &Vec<u8>, pos : usize) -> Result<(usize, Vec::<u8>), SerializationError> {
    // len is at position 4 
    // roi is at postion 6
    if stream.len() < 8 {
      return Err(SerializationError::StreamTooShort);
    }
    let mut _pos = pos + 4;
    let packet_len = parse_u16(stream, &mut _pos) as usize * 2; // len is in 2byte words
    if packet_len < 44 {
      // There is only header data 
      error!("Event fragment - no channel data!");
      return Ok((packet_len.into(), Vec::<u8>::new()));
    }
    let nwords     = parse_u16(stream, &mut _pos) as usize + 1; // roi is max bin (first is 0)
    debug!("Got packet len of {} bytes, roi of {}", packet_len, nwords);
    let channel_packet_start = pos + 36;
    let nchan_data = packet_len - 44;
    if stream.len() < channel_packet_start + nchan_data {
      error!("We claim there should be channel data, but the event is too short!");
      return Err(SerializationError::StreamTooShort)
    }

    let mut nchan = 0usize;
    //println!("========================================");
    //println!("{} {} {}", nchan, nwords, nchan_data);
    //println!("========================================");
    while nchan * (2*nwords + 6) < nchan_data {
      nchan += 1;
    }
    if nchan * (2*nwords + 6) != nchan_data {
      error!("NCHAN consistency check failed! nchan {} , nwords {}, packet_len {}", nchan, nwords, packet_len);
    }
    let mut ch_ids = Vec::<u8>::new();
    _pos = channel_packet_start;
    for _ in 0..nchan {
      ch_ids.push(parse_u16(stream, &mut _pos) as u8);
      _pos += (nwords*2) as usize;
      _pos += 4; // trailer
    }
    debug!("Got channel ids {:?}", ch_ids);
    Ok((nchan_data.into(), ch_ids))
  }

  /// Get the event id from a RBEvent represented by bytestream
  /// without decoding the whole event
  ///
  /// This should be faster than decoding the whole event.
  pub fn extract_eventid(stream : &Vec<u8>) -> Result<u32, SerializationError> {
    if stream.len() < 30 {
      return Err(SerializationError::StreamTooShort);
    }
    // event header starts at position 7
    // in the header, it is as positon 3
    let event_id = parse_u32(stream, &mut 10);
    Ok(event_id)
  }

  pub fn get_ndatachan(&self) -> usize {
    self.adc.len()
  }

  pub fn get_channel_by_id(&self, ch : usize) -> Result<&Vec::<u16>, UserError> {
    if ch >= 9 {
      error!("channel_by_id expects numbers from 0-8!");
      return Err(UserError::IneligibleChannelLabel)
    }
    return Ok(&self.adc[ch]);
    //if ch < 8 {
    //  return Ok(&self.adc[ch]);
    //} else {
    //  if self.header.has_ch9 {
    //    return Ok(&self.ch9_adc);
    //  } else {
    //    error!("No channel 9 data for this event!");
    //    return Err(UserError::NoChannel9Data);
    //  }
    //}
  }

  pub fn get_channel_by_label(&self, ch : u8) -> Result<&Vec::<u16>, UserError>  {
    //let mut ch_adc = Vec::<u16>::new();
    if ch == 0 || ch > 9 {
      error!("channel_by_label expects numbers from 1-9!");
      return Err(UserError::IneligibleChannelLabel)
    }
    //if ch == 9 {
    //  if self.header.has_ch9 {
    //    return Ok(&self.ch9_adc);
    //  } else {
    //    error!("No channel 9 data for this event!");
    //    return Err(UserError::NoChannel9Data);
    //  }
    //}
    Ok(&self.adc[ch as usize -1])
  }

  //pub fn get_adcs(&self) -> &Vec<Vec<u16>> {
  //  self.adc
  //  //let mut adcs = Vec::<&Vec<u16>>::new();
  //  //for v in self.adc.iter() {
  //  //  adcs.push(&v);
  //  //}
  //  //if self.header.has_ch9 {
  //  //  adcs.push(&self.ch9_adc);
  //  //}
  //  adcs
  //}

  // If we know that the stream contains an RBEventMemeoryView, 
  // we can convert the stream directly to a RBEvent.
  //pub fn extract_from_rbeventmemoryview(stream : &Vec<u8>,
  //                                      pos    : &mut usize) 
  //  -> Result<Self, SerializationError> {
  //  let mut event  = Self::new();
  //  let header     = RBEventHeader::extract_from_rbeventmemoryview(stream, pos)?;
 
  //  /// calculate crc32 sums
  //  const CUSTOM_ALG: crc::Algorithm<u32> = crc::Algorithm {
  //                     width   : 32u8,
  //                     init    : 0xFFFFFFFF,
  //                     poly    : 0xEDB88320,
  //                     refin   : true,
  //                     refout  : true,
  //                     xorout  : 0xFFFFFFFF,
  //                     check   : 1,
  //                     residue : 1,
  //                     //check   : 0xcbf43926,
  //                     //residue : 0xdebb20e3,
  //                   };

  //  //let crc        = Crc::<u32>::new(&crc::CRC_32_ISO_HDLC);
  //  let crc          = Crc::<u32>::new(&CUSTOM_ALG);
  //  //let crc        = Crc::<u32>::new(&crc::CRC_32_BZIP2);
  //  let mut ch_crc       : u32;
  //  let mut ch_crc_check : u32;
  //  //if header.broken {
  //  //  error!("Broken event {}! This won't have any channel data, since the event end markes is not at the expected position. Treat the header values with caution!", &header.event_id);
  //  //  event.header   = header;
  //  //  return Ok(event);
  //  //}
  //  //let mut active_channels = header.get_active_data_channels();
  //  //let mut nchan = active_channels.len();
  //  event.header = header;
  //  // set the position marker to the start of the channel
  //  // adc data field
  //  *pos = event.header.channel_packet_start; 
  //  if stream.len() < event.header.channel_packet_len + 44 {
  //    error!("The header says there is channel data, but the event is corrupt!");
  //    return Err(SerializationError::StreamTooShort);
  //  }

  //  for ch in event.header.channel_packet_ids.iter() {
  //    let mut dig    = crc.digest_with_initial(0);
  //    *pos += 2; // ch id
  //    if ch > &9 {
  //      error!("Channel ID is messed up. Not sure if this event can be saved!");
  //      *pos += 2*event.header.nwords;
  //      *pos += 4;
  //    } else {
  //      //ch_crc_check = crc.checksum(&stream[*pos..*pos+2*event.header.nwords]); 
  //      //let mut crc_test = Crc::new();
  //      //crc_test.update(&stream[*pos..*pos+2*event.header.nwords]);
  //      //let ch_crc_check = crc_test.sum();
  //      let mut this_ch_adc = Vec::<u16>::with_capacity(event.header.nwords);
  //      for _ in 0..event.header.nwords {  
  //        //let this_bytes = [stream[*pos ], stream[*pos + 1]]; 
  //        let this_word  = [stream[*pos+1], stream[*pos]];
  //        
  //        this_ch_adc.push(0x3FFF & parse_u16(stream, pos));
  //        dig.update(&this_word);
  //      }
  //      if ch < &8 {
  //        event.adc.push(this_ch_adc);  
  //      } else {
  //        event.ch9_adc = this_ch_adc;
  //      }
  //      ch_crc = parse_u32_for_16bit_words(stream, pos);
  //      ch_crc_check = dig.finalize();
  //      //println!("==> Calculated crc32 {}, expected crc32 {}", ch_crc_check, ch_crc);
  //      //*pos += 4; // trailer
  //    }
  //  }
  //  Ok(event)
  //}
}

impl Packable for RBEvent {
  const PACKET_TYPE : PacketType = PacketType::RBEvent;
}

impl Serialization for RBEvent {
  const HEAD : u16 = 0xAAAA;
  const TAIL : u16 = 0x5555;
  
  fn from_bytestream(stream : &Vec<u8>, pos : &mut usize)
    -> Result<Self, SerializationError> {
    let mut event = Self::new();
    if parse_u16(stream, pos) != Self::HEAD {
      error!("The given position {} does not point to a valid header signature of {}", pos, Self::HEAD);
      return Err(SerializationError::HeadInvalid {});
    }
    event.data_type = DataType::try_from(parse_u8(stream, pos)).unwrap_or(DataType::Unknown);
    event.status    = EventStatus::try_from(parse_u8(stream, pos)).unwrap_or(EventStatus::Unknown);
    //let nchan_data  = parse_u8(stream, pos);
    let n_hits      = parse_u8(stream, pos);
    event.header    = RBEventHeader::from_bytestream(stream, pos)?;
    //let ch_ids      = event.header.get_active_data_channels();
    let stream_len  = stream.len();
    if event.header.is_event_fragment() {
      debug!("Fragmented event {} found!", event.header.event_id);
      let tail_pos = search_for_u16(Self::TAIL, stream, *pos)?;
      * pos = tail_pos + 2 as usize;
      // the event fragment won't have channel data, so 
      // let's move on to the next TAIL marker:ta
      return Ok(event);
    }
    if event.header.drs_lost_trigger() {
      debug!("Event {} has lost trigger!", event.header.event_id);
      let tail_pos = search_for_u16(Self::TAIL, stream, *pos)?;
      * pos = tail_pos + 2 as usize;
      return Ok(event);
    }
    let mut decoded_ch = Vec::<u8>::new();
    for ch in event.header.get_channels().iter() {
      if *pos + 2*NWORDS >= stream_len {
        error!("The channel data for event {} ch {} seems corrupt! We want to get channels {:?}, but have decoded only {:?}, because the stream ends {} bytes too early!",event.header.event_id, ch, event.header.get_channels(), decoded_ch, *pos + 2*NWORDS - stream_len);
        let tail_pos = search_for_u16(Self::TAIL, stream, *pos)?;
        * pos = tail_pos + 2 as usize;
        return Err(SerializationError::WrongByteSize {})
      }
      decoded_ch.push(*ch);
      // 2*NWORDS because stream is Vec::<u8> and it is 16 bit words.
      let data = &stream[*pos..*pos+2*NWORDS];
      //event.adc[k as usize] = u8_to_u16(data);
      event.adc[*ch as usize] = u8_to_u16(data);
      *pos += 2*NWORDS;
    }
    for _ in 0..n_hits {
      match TofHit::from_bytestream(stream, pos) {
        Err(err) => {
          error!("Can't read TofHit! Err {err}");
          let mut h = TofHit::new();
          h.valid = false;
          event.hits.push(h);
        },
        Ok(h) => {
          event.hits.push(h);
        }
      }
    }
    let tail = parse_u16(stream, pos);
    //println!("{:?}", &stream[*pos-10..*pos+2]);
    //println!("{} {}", pos, stream.len());
    if tail != Self::TAIL {
      error!("After parsing the event, we found an invalid tail signature {}", tail);
      return Err(SerializationError::TailInvalid);
    }
    Ok(event)
  }
  
  fn to_bytestream(&self) -> Vec<u8> {
    let mut stream = Vec::<u8>::with_capacity(18530);
    //let mut stream = Vec::<u8>::new();
    stream.extend_from_slice(&Self::HEAD.to_le_bytes());
    stream.push(self.data_type as u8);
    stream.push(self.status as u8);
    //let nchan_data  = self.adc.len() as u8;
    //stream.push(nchan_data);
    let n_hits      = self.hits.len() as u8;
    stream.push(n_hits);
    stream.extend_from_slice(&self.header.to_bytestream());
    // for an empty channel, we will add an empty vector
    let add_channels = !self.header.is_event_fragment() & !self.header.drs_lost_trigger();
    if add_channels {
      for n in 0..NCHN {
        for k in 0..NWORDS {
          if self.adc[n].len() == 0 {
            continue;
          }
          stream.extend_from_slice(&self.adc[n][k].to_le_bytes());  
        }
      }
      // this is way slower
      //for channel_adc in self.adc.iter() {
      //  stream.extend_from_slice(&u16_to_u8(&channel_adc)); 
      //}
    }
    //if self.ch9_adc.len() > 0 {
    //  stream.extend_from_slice(&u16_to_u8(&self.ch9_adc));
    //}
    for h in self.hits.iter() {
      stream.extend_from_slice(&h.to_bytestream());
    }
    stream.extend_from_slice(&Self::TAIL.to_le_bytes());
    stream
  }
}

impl Default for RBEvent {

  fn default () -> Self {
    Self::new()
  }
}

impl fmt::Display for RBEvent {
  fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
    let mut adc = Vec::<usize>::new();
    for k in 0..self.adc.len() -1 {
      adc.push(self.adc[k].len());
    }
    let mut ch9_str = String::from("[");
    for k in self.adc[8].iter().take(5) {
      ch9_str += &k.to_string();
      ch9_str += ","
    }
    ch9_str += " .. :";
    ch9_str += &self.adc[8].len().to_string();
    ch9_str += "]";
    let mut ch_field = String::from("[\n");
    for (ch, vals) in self.adc.iter().enumerate() {
      if ch == 8 {
        continue;
      }
      let label = (ch + 1).to_string();
      ch_field += "  [ch ";
      ch_field += &ch.to_string();
      ch_field += "('";
      ch_field += &label;
      ch_field += "') ";
      for n in vals.iter().take(5) {
        ch_field += &n.to_string();
        ch_field += ",";
      }
      ch_field += "..:";
      ch_field += &vals.len().to_string();
      ch_field += "]\n";
    }
    ch_field += "]\n";
    write!(f, "<RBEvent 
  data type     : {},
  event status  : {},
  {}
  .. .. 
  has ch9       : {},
    -> ch9      : {},
  data channels : 
    -> {},
  n hits        : {},
.. .. .. .. .. .. .. .. >",
    self.data_type,
    self.status,
    self.header,
    self.header.has_ch9(),
    ch9_str,
    ch_field,
    self.hits.len())
  }
}

#[cfg(feature = "random")]
impl FromRandom for RBEvent {
    
  fn from_random() -> Self {
    let mut event   = RBEvent::new();
    let header      = RBEventHeader::from_random();
    let mut rng     = rand::thread_rng();
    event.data_type = DataType::from_random(); 
    event.status    = EventStatus::from_random();
    event.header    = header;
    // set a good status byte. RBEvents from 
    // random will always be good
    // status_byte is tested in RBEventHeader test
    // and here we want to make sure channel data 
    // gets serialized
    // status byte of 0 means it is good
    event.header.status_byte = 0;
    //if !event.header.event_fragment && !event.header.lost_trigger {
    for ch in event.header.get_channels().iter() {
      let random_numbers: Vec<u16> = (0..NWORDS).map(|_| rng.gen()).collect();
      event.adc[*ch as usize] = random_numbers;
    }
    //}
    event
  }
}


impl From<&TofPacket> for RBEvent {
  fn from(pk : &TofPacket) -> Self {
    match pk.packet_type {
      //PacketType::RBEventMemoryView => {
      //  match RBEvent::extract_from_rbeventmemoryview(&pk.payload, &mut 0) {
      //    Ok(event) => {
      //      return event;
      //    }
      //    Err(err) => { 
      //      error!("Can not get RBEvent from RBEventMemoryView! Error {err}!");
      //      error!("Returning empty event!");
      //      return RBEvent::new();
      //    }
      //  }
      //},
      PacketType::RBEvent => {
        match RBEvent::from_bytestream(&pk.payload, &mut 0) {
          Ok(event) => {
            return event;
          }
          Err(err) => { 
            error!("Can not decode RBEvent - will return empty event! {err}");
            return RBEvent::new();
          }
        }
      },
      PacketType::RBEventMemoryView => {
        let mut streamer = RBEventMemoryStreamer::new();
        streamer.add(&pk.payload, pk.payload.len());
        match streamer.get_event_at_pos_unchecked(None) {
          None => {
            return RBEvent::new();
          },
          Some(ev) => {
            return ev;
          }
        }
      },

      _ => {
        error!("Can not deal with {}! Returning empty event", pk);
        return RBEvent::new();
      }
    }
  }
}


/// The RBEvent header gets generated once per event
/// per RB. 
/// Contains information about event id, timestamps,
/// etc.
#[derive(Debug, Clone, PartialEq)]
pub struct RBEventHeader { 
  /// Readoutboard ID - should be in the range 1-50
  /// not consecutive, there are some missing.
  /// In general, we have 40 boards
  pub rb_id                : u8   ,
  /// The event ID as sent from the MTB or self-generated
  /// if not latched to the MTB
  pub event_id             : u32  ,
  /// The DRS stop cell. This is vital information which is
  /// needed for the calibration
  pub stop_cell            : u16  , 
  // we change this by keeping the byte
  // order the same to accomodate the sine 
  // values
  pub ch9_amp               : u16,
  pub ch9_freq              : u16,
  pub ch9_phase             : u32,
  /// The adc value for the temperature
  /// of the FPGA
  pub fpga_temp             : u16, 
  /// DRS deadtime as read out from the 
  /// register
  pub drs_deadtime          : u16,
  pub timestamp32           : u32,
  pub timestamp16           : u16,
  /// Store the drs_deadtime instead 
  /// of the fpga temperature
  pub deadtime_instead_temp : bool,
  /// the mapping of rb_channel to paddle_id
  /// This will not get serialized, since it is redundant 
  /// information
  /// This is for the a side
  pub channel_pid_a_map       : Option<HashMap<u8,u8>>,
  /// the mapping of rb_channel to paddle_id
  /// This will not get serialized, since it is redundant 
  /// information
  /// This is for the b side
  pub channel_pid_b_map       : Option<HashMap<u8,u8>>,
  /// The status byte contains information about lsos of lock
  /// and event fragments and needs to be decoded
  status_byte               : u8,
  /// The channel mask is 9bit for the 9 channels.
  /// This leaves 7 bits of space so we actually 
  /// hijack that for the version information 
  /// 
  /// Bit 15 will be set 1 in case we are sending
  /// the DRS_DEADTIME instead of FPGA TEMP
  ///
  /// FIXME - make this proper and use ProtocolVersion 
  ///         instead
  channel_mask             : u16, 

}

impl RBEventHeader {

  pub fn new() -> Self {
    Self {
      rb_id                 : 0,  
      status_byte           : 0, 
      event_id              : 0,  
      channel_mask          : 0,  
      stop_cell             : 0,  
      ch9_amp               : 0,
      ch9_freq              : 0,
      ch9_phase             : 0,
      fpga_temp             : 0,  
      drs_deadtime          : 0,
      timestamp32           : 0,
      timestamp16           : 0,
      deadtime_instead_temp : false,
      channel_pid_a_map     : None,
      channel_pid_b_map     : None,
    }
  }

  /// Set the channel mask with the 9bit number
  ///
  /// Set bit 15 to either 1 or 0 depending on
  /// deadtime_instead_temp
  pub fn set_channel_mask(&mut self, channel_mask : u16) {
    if self.deadtime_instead_temp {
      self.channel_mask = 2u16.pow(15) | channel_mask;
    } else {
      self.channel_mask = channel_mask;
    }
  }

  /// Just return the channel mask and strip of 
  /// the part which contains the information about
  /// drs deadtime or fpga temp
  pub fn get_channel_mask(&self) -> u16 {
    self.channel_mask & 0x1ff 
  }

  /// Get the channel mask from a bytestream.
  /// 
  /// This takes into acount that bit 15 is 
  /// used to convey information about if we
  /// stored the drs temperature or deadtime
  pub fn parse_channel_mask(ch_mask : u16) -> (bool, u16) {
    let channel_mask          : u16;
    let deadtime_instead_temp : bool 
      = ch_mask >> 15 == 1;
    channel_mask = ch_mask & 0x1ff;
    (deadtime_instead_temp, channel_mask)
  }

  pub fn set_sine_fit(&mut self, input : (f32, f32, f32)) {
    // we have to squeze 3 f32 into 64 bit, to 
    // fit into the dataformat (we don't want to 
    // change anything. 
    // let's use an arbitrary precision for a 
    // range of -10 to 10 
    let mut amp   = (input.0 + 1090.0)*(u16::MAX as f32 / 2000.0);
    let mut freq  = (input.1 + 1090.0)*(u16::MAX as f32 / 2000.0);
    let mut phase = (input.2 + 1090.0)*(u32::MAX as f32 / 2000.0);

    if amp < 0.0 {
      warn!("amp out of range!");
      amp = 0.0;
    }
    if freq < 0.0 {
      warn!("freq out of range!");
      freq = 0.0;
    }
    if phase < 0.0 {
      warn!("phase out of range!");
      phase = 0.0;
    }
    self.ch9_amp   = amp   as u16;  
    self.ch9_freq  = freq  as u16;
    self.ch9_phase = phase as u32;
  }
  
  pub fn get_sine_fit(&self) -> (f32, f32, f32) {
    let amp    = (2000.0 * self.ch9_amp   as f32)/(u16::MAX as f32) - 1000.0;
    let freq   = (2000.0 * self.ch9_freq  as f32)/(u16::MAX as f32) - 1000.0;
    let phase  = (2000.0 * self.ch9_phase as f32)/(u16::MAX as f32) - 1000.0;
    (amp, freq, phase)
  }

  /// Only get the eventid from a binary stream
  pub fn extract_eventid_from_rbheader(stream :&Vec<u8>) -> u32 {
    // event id is 18 bytes in (including HEAD bytes)
    // event id is 3 bytes in (including HEAD bytes)
    let event_id = parse_u32(stream, &mut 3); // or should it be 5?
    event_id
  }
  
  pub fn is_event_fragment(&self) -> bool {
    self.status_byte & 1 > 0
  }
  
  pub fn drs_lost_trigger(&self) -> bool {
    (self.status_byte >> 1) & 1 > 0
  }

  pub fn lost_lock(&self) -> bool {
    (self.status_byte >> 2) & 1 > 0
  }

  pub fn lost_lock_last_sec(&self) -> bool {
    (self.status_byte >> 3) & 1 > 0
  }

  pub fn is_locked(&self) -> bool {
    !self.lost_lock()
  }
  
  pub fn is_locked_last_sec(&self) -> bool {
    !self.lost_lock_last_sec()
  }
  
  /// extract lock, drs busy and fpga temp from status field
  pub fn parse_status(&mut self, status_bytes : u16) {
    // status byte is only 4bit really
    self.status_byte        = (status_bytes & 0xf) as u8;
    self.fpga_temp = status_bytes >> 4;
  }

  /// Get the temperature value (Celsius) from the fpga_temp adc.
  pub fn get_fpga_temp(&self) -> f32 {
    let zynq_temp = (((self.fpga_temp & 4095) as f32 * 503.975) / 4096.0) - 273.15;
    zynq_temp
  }

  /// Check if the channel 9 is present in the 
  /// channel mask
  pub fn has_ch9(&self) -> bool {
    self.channel_mask & 256 > 0
  }

  /// Decode the channel mask into channel ids.
  ///
  /// The channel ids inside the memory representation
  /// of the RB Event data ("blob") are from 0-7
  ///
  /// We keep ch9 seperate.
  pub fn get_channels(&self) -> Vec<u8> {
    let mut channels = Vec::<u8>::with_capacity(8);
    for k in 0..9 {
      if self.channel_mask & (1 << k) > 0 {
        channels.push(k);
      }
    }
    channels
  }

  /// Get the number of data channels + 1 for ch9
  pub fn get_nchan(&self) -> usize {
    self.get_channels().len()
  }
  
  pub fn get_timestamp48(&self) -> u64 {
    ((self.timestamp16 as u64) << 32) | self.timestamp32 as u64
  }
}

impl Default for RBEventHeader {
  fn default() -> Self {
    Self::new()
  }
}


impl fmt::Display for RBEventHeader {
  fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
    let sfit = self.get_sine_fit();
    let mut repr = String::from("<RBEventHeader:");
    let sine_field = format!("\n    --> online fit AMP {:3} FREQ {:3} PHASE {:3}", sfit.0, sfit.1, sfit.2);
    repr += &(format!("\n  RB ID            {}",self.rb_id               )); 
    repr += &(format!("\n  event id         {}",self.event_id            ));  
    repr += &(format!("\n  ch mask          {}",self.channel_mask        ));  
    repr += &(format!("\n  has ch9          {}",self.has_ch9()           )); 
    //repr += &("\n  DRS4 temp [C]    ".to_owned() + &self.drs4_temp.to_string());  
    repr += &sine_field;
    //repr += &("\n  FPGA temp [\u{00B0}C]    ".to_owned() + &self.get_fpga_temp().to_string()); 
    if self.deadtime_instead_temp {
      repr += &(format!("\n  DRS deadtime          : {:.2}", self.drs_deadtime));
    } else {
      repr += &(format!("\n  FPGA T [\u{00B0}C]    : {:.2}", self.get_fpga_temp()));
    }
    repr += &(format!("\n  timestamp32      {}", self.timestamp32            )); 
    repr += &(format!("\n  timestamp16      {}", self.timestamp16            )); 
    repr += &(format!("\n   |-> timestamp48 {}", self.get_timestamp48()      )); 
    repr += &(format!("\n  stop cell        {}", self.stop_cell              )); 
    //repr += &("\n  dtap0            ".to_owned() + &self.dtap0.to_string()); 
    //repr += &("\n  crc32            ".to_owned() + &self.crc32.to_string()); 
    let mut perfect = true;
    if self.drs_lost_trigger() {
      repr += &"\n  !! DRS4 REPORTS LOST TRIGGER!".red().bold();
      perfect = false;
    }
    if self.is_event_fragment() {
      repr += &"\n  !! EVENT FRAGMENT!".red().bold();
      perfect = false;
    }
    if self.lost_lock() {
      repr += &"\n  !! RB CLOCK IS NOT LOCKED!".yellow().bold();
      perfect = false;
    }
    if self.lost_lock_last_sec() {
      repr += &"\n  !! RB CLOCK HAS LOST ITS LOCK WITHIN THE LAST SECOND!".yellow().bold();
      perfect = false;
    }
    if perfect {
      repr += &"\n  -- locked: YES, locked last second; YES, no event fragemnet, and no lost trigger!".green();
    }
    repr += ">";
    write!(f, "{}", repr)
  }
}

impl Serialization for RBEventHeader {
  
  const HEAD : u16   = 0xAAAA;
  const TAIL : u16   = 0x5555;
  const SIZE : usize = 30; // size in bytes with HEAD and TAIL

  fn from_bytestream(stream : &Vec<u8>, pos : &mut usize)
    -> Result<Self, SerializationError> {
    let mut header  = Self::new();
    Self::verify_fixed(stream, pos)?;
    header.rb_id                 = parse_u8 (stream, pos);  
    header.event_id              = parse_u32(stream, pos);  
    let ch_mask                  = parse_u16(stream, pos);
    let (deadtime_instead_temp, channel_mask)  
      = Self::parse_channel_mask(ch_mask);
    header.deadtime_instead_temp = deadtime_instead_temp;
    header.set_channel_mask(channel_mask);
    header.status_byte         = parse_u8 (stream, pos);
    header.stop_cell             = parse_u16(stream, pos);  
    header.ch9_amp               = parse_u16(stream, pos);
    header.ch9_freq              = parse_u16(stream, pos);
    header.ch9_phase             = parse_u32(stream, pos);
    if deadtime_instead_temp {
      header.drs_deadtime        = parse_u16(stream, pos);
    } else {
      header.fpga_temp           = parse_u16(stream, pos);
    }
    header.timestamp32           = parse_u32(stream, pos);
    header.timestamp16           = parse_u16(stream, pos);
    *pos += 2; // account for tail earlier 
    Ok(header) 
  }
  

  fn to_bytestream(&self) -> Vec<u8> {
    let mut stream = Vec::<u8>::with_capacity(Self::SIZE);
    stream.extend_from_slice(&Self::HEAD.to_le_bytes());
    stream.extend_from_slice(&self.rb_id             .to_le_bytes());
    stream.extend_from_slice(&self.event_id          .to_le_bytes());
    let ch_mask = ((self.deadtime_instead_temp as u16) << 15) | self.get_channel_mask();
    stream.extend_from_slice(&ch_mask                .to_le_bytes());
    stream.extend_from_slice(&self.status_byte       .to_le_bytes());
    stream.extend_from_slice(&self.stop_cell         .to_le_bytes());
    stream.extend_from_slice(&self.ch9_amp           .to_le_bytes());
    stream.extend_from_slice(&self.ch9_freq          .to_le_bytes());
    stream.extend_from_slice(&self.ch9_phase         .to_le_bytes());
    if self.deadtime_instead_temp {
      stream.extend_from_slice(&self.drs_deadtime    .to_le_bytes());
    } else {
      stream.extend_from_slice(&self.fpga_temp       .to_le_bytes());
    }
    stream.extend_from_slice(&self.timestamp32       .to_le_bytes());
    stream.extend_from_slice(&self.timestamp16       .to_le_bytes());
    stream.extend_from_slice(&RBEventHeader::TAIL.to_le_bytes());
    stream
  }
}

#[cfg(feature = "random")]
impl FromRandom for RBEventHeader {
    
  fn from_random() -> Self {
    let mut header = RBEventHeader::new();
    let mut rng = rand::thread_rng();

    header.rb_id                 = rng.gen::<u8>();    
    header.event_id              = rng.gen::<u32>();   
    header.status_byte           = rng.gen::<u8>();    
    header.stop_cell             = rng.gen::<u16>();   
    header.ch9_amp               = rng.gen::<u16>();
    header.ch9_freq              = rng.gen::<u16>();
    header.ch9_phase             = rng.gen::<u32>();
    header.deadtime_instead_temp = rng.gen::<bool>();
    if header.deadtime_instead_temp {
      header.drs_deadtime          = rng.gen::<u16>();
    } else {
      header.fpga_temp             = rng.gen::<u16>();  
    }
    // make sure the generated channel mask is valid!
    let ch_mask                  = rng.gen::<u16>() & 0x1ff;
    header.set_channel_mask(ch_mask);
    header.timestamp32           = rng.gen::<u32>();
    header.timestamp16           = rng.gen::<u16>();
    header
  }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RBWaveform {
  pub event_id      : u32,
  pub rb_id         : u8,
  /// FIXME - this is form 0-8, but should it be from 1-9?
  pub rb_channel_a  : u8,
  pub rb_channel_b  : u8,
  /// DRS4 stop cell
  pub stop_cell     : u16,
  pub adc_a         : Vec<u16>,
  pub adc_b         : Vec<u16>,
  pub paddle_id     : u8,
  pub voltages_a    : Vec<f32>,
  pub nanoseconds_a : Vec<f32>,
  pub voltages_b    : Vec<f32>,
  pub nanoseconds_b : Vec<f32>
}

impl RBWaveform {
  
  pub fn new() -> Self {
    Self {
      event_id       : 0,
      rb_id          : 0,
      rb_channel_a   : 0,
      rb_channel_b   : 0,
      stop_cell      : 0,
      paddle_id      : 0,
      adc_a          : Vec::<u16>::new(),
      voltages_a     : Vec::<f32>::new(),
      nanoseconds_a  : Vec::<f32>::new(),
      adc_b          : Vec::<u16>::new(),
      voltages_b     : Vec::<f32>::new(),
      nanoseconds_b  : Vec::<f32>::new()
    }
  }

  pub fn calibrate(&mut self, cali : &RBCalibrations) -> Result<(), CalibrationError>  {
    if cali.rb_id != self.rb_id {
      error!("Calibration is for board {}, but wf is for {}", cali.rb_id, self.rb_id);
      return Err(CalibrationError::WrongBoardId);
    }
    let mut voltages = vec![0.0f32;1024];
    let mut nanosecs = vec![0.0f32;1024];
    cali.voltages(self.rb_channel_a as usize + 1,
                  self.stop_cell as usize,
                  &self.adc_a,
                  &mut voltages);
    self.voltages_a = voltages.clone();
    cali.nanoseconds(self.rb_channel_a as usize + 1,
                     self.stop_cell as usize,
                     &mut nanosecs);
    self.nanoseconds_a = nanosecs.clone();
    cali.voltages(self.rb_channel_b as usize + 1,
                  self.stop_cell as usize,
                  &self.adc_b,
                  &mut voltages);
    self.voltages_b = voltages;
    cali.nanoseconds(self.rb_channel_b as usize + 1,
                     self.stop_cell as usize,
                     &mut nanosecs);
    self.nanoseconds_b = nanosecs;
    Ok(())
  }

  /// Apply Jamie's simple spike filter to the calibrated voltages
  pub fn apply_spike_filter(&mut self) {
    clean_spikes(&mut self.voltages_a, true);
    clean_spikes(&mut self.voltages_b, true);
  }
}

impl Packable for RBWaveform {
  const PACKET_TYPE : PacketType = PacketType::RBWaveform;
}

impl Serialization for RBWaveform {
  const HEAD               : u16    = 43690; //0xAAAA
  const TAIL               : u16    = 21845; //0x5555
  
  fn from_bytestream(stream : &Vec<u8>, pos : &mut usize)
    -> Result<Self, SerializationError> {
    let mut wf           = RBWaveform::new();
    if parse_u16(stream, pos) != Self::HEAD {
      error!("The given position {} does not point to a valid header signature of {}", pos, Self::HEAD);
      return Err(SerializationError::HeadInvalid {});
    }
    wf.event_id          = parse_u32(stream, pos);
    wf.rb_id             = parse_u8 (stream, pos);
    wf.rb_channel_a      = parse_u8 (stream, pos);
    wf.rb_channel_b      = parse_u8 (stream, pos);
    wf.stop_cell         = parse_u16(stream, pos);
    if stream.len() < *pos+2*NWORDS {
      return Err(SerializationError::StreamTooShort);
    }
    let data_a           = &stream[*pos..*pos+2*NWORDS];
    wf.adc_a             = u8_to_u16(data_a);
    *pos += 2*NWORDS;
    let data_b           = &stream[*pos..*pos+2*NWORDS];
    wf.adc_b             = u8_to_u16(data_b);
    *pos += 2*NWORDS;
    if parse_u16(stream, pos) != Self::TAIL {
      error!("The given position {} does not point to a tail signature of {}", pos, Self::TAIL);
      return Err(SerializationError::TailInvalid);
    }
    Ok(wf)
  }

  fn to_bytestream(&self) -> Vec<u8> {
    let mut stream = Vec::<u8>::new();
    stream.extend_from_slice(&Self::HEAD.to_le_bytes());
    stream.extend_from_slice(&self.event_id.to_le_bytes());
    stream.extend_from_slice(&self.rb_id.to_le_bytes());
    stream.extend_from_slice(&self.rb_channel_a.to_le_bytes());
    stream.extend_from_slice(&self.rb_channel_b.to_le_bytes());
    stream.extend_from_slice(&self.stop_cell.to_le_bytes());
    if self.adc_a.len() != 0 {
      for k in 0..NWORDS {
        stream.extend_from_slice(&self.adc_a[k].to_le_bytes());  
      }
    }
    if self.adc_b.len() != 0 {
      for k in 0..NWORDS {
        stream.extend_from_slice(&self.adc_b[k].to_le_bytes());  
      }
    }
    stream.extend_from_slice(&Self::TAIL.to_le_bytes());
    stream
  }
}

impl fmt::Display for RBWaveform {
  fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
    let mut repr = String::from("<RBWaveform:");
    repr += &(format!("\n  Event ID  : {}", self.event_id));
    repr += &(format!("\n  RB        : {}", self.rb_id));
    repr += &(format!("\n  ChannelA  : {}", self.rb_channel_a));
    repr += &(format!("\n  ChannelB  : {}", self.rb_channel_b));
    repr += &(format!("\n  Paddle ID : {}", self.paddle_id));
    repr += &(format!("\n  Stop cell : {}", self.stop_cell));
    if self.adc_a.len() >= 273 {
      repr += &(format!("\n  adc [{}]      : .. {} {} {} ..",self.adc_a.len(), self.adc_a[270], self.adc_a[271], self.adc_a[272]));
    } else {
      repr += &(String::from("\n  adc [EMPTY]"));
    }
    if self.adc_b.len() >= 273 {
      repr += &(format!("\n  adc [{}]      : .. {} {} {} ..",self.adc_b.len(), self.adc_b[270], self.adc_b[271], self.adc_b[272]));
    } else {
      repr += &(String::from("\n  adc [EMPTY]"));
    }
    write!(f, "{}", repr)
  }
}

#[cfg(feature = "random")]
impl FromRandom for RBWaveform {
    
  fn from_random() -> Self {
    let mut wf      = Self::new();
    let mut rng     = rand::thread_rng();
    wf.event_id     = rng.gen::<u32>();
    wf.rb_id        = rng.gen::<u8>();
    wf.rb_channel_a = rng.gen::<u8>();
    wf.rb_channel_b = rng.gen::<u8>();
    wf.stop_cell    = rng.gen::<u16>();
    let random_numbers_a: Vec<u16> = (0..NWORDS).map(|_| rng.gen()).collect();
    wf.adc_a        = random_numbers_a;
    let random_numbers_b: Vec<u16> = (0..NWORDS).map(|_| rng.gen()).collect();
    wf.adc_b        = random_numbers_b;
    wf
  }
}
  
#[test]
#[cfg(feature = "random")]
fn pack_rbwaveform() {
  for _ in 0..100 {
    let wf   = RBWaveform::from_random();
    let test : RBWaveform = wf.pack().unpack().unwrap();
    assert_eq!(wf, test);
  }
}

#[cfg(all(test,feature = "random"))]
mod test_rbevents {
  use crate::serialization::Serialization;
  use crate::FromRandom;
  use crate::events::{
      RBEvent,
      RBEventHeader,
  };
  
  #[test]
  fn serialization_rbeventheader() {
    for _ in 0..100 {
      let mut pos = 0usize;
      let head = RBEventHeader::from_random();
      println!("{}",  head);
      let stream = head.to_bytestream();
      assert_eq!(stream.len(), RBEventHeader::SIZE);
      let test = RBEventHeader::from_bytestream(&stream, &mut pos).unwrap();
      println!("{}", test);
      assert_eq!(pos, RBEventHeader::SIZE);
      assert_eq!(head, test);
      assert_eq!(head.lost_lock()         , test.lost_lock());
      assert_eq!(head.lost_lock_last_sec(), test.lost_lock_last_sec());
      assert_eq!(head.drs_lost_trigger()  , test.drs_lost_trigger());
      assert_eq!(head, test);
    }
  }
  
  #[test]
  fn serialization_rbevent() {
    for _ in 0..100 {
      let event  = RBEvent::from_random();
      let stream = event.to_bytestream();
      println!("[test rbevent] stream.len()   {:?}", stream.len());
      let test   = RBEvent::from_bytestream(&stream, &mut 0).unwrap();
      println!("[test rbevent] event frag   {:?}", event.header.is_event_fragment());
      println!("[test rbevent] lost trig    {:?}", event.header.drs_lost_trigger());
      println!("[test rbevent] event frag   {:?}", test.header.is_event_fragment());
      println!("[test rbevent] lost trig    {:?}", test.header.drs_lost_trigger());
      assert_eq!(event.header, test.header);
      assert_eq!(event.header.get_nchan(), test.header.get_nchan());
      assert_eq!(event.header.get_channels(), test.header.get_channels());
      assert_eq!(event.data_type, test.data_type);
      assert_eq!(event.status, test.status);
      assert_eq!(event.adc.len(), test.adc.len());
      assert_eq!(event.hits.len(), test.hits.len());
      println!("[test rbevent] get_channels() {:?}", event.header.get_channels());
      assert_eq!(event.adc[0].len(), test.adc[0].len());
      assert_eq!(event.adc[1].len(), test.adc[1].len());
      assert_eq!(event.adc[2].len(), test.adc[2].len());
      assert_eq!(event.adc[3].len(), test.adc[3].len());
      assert_eq!(event.adc[4].len(), test.adc[4].len());
      assert_eq!(event.adc[5].len(), test.adc[5].len());
      assert_eq!(event.adc[6].len(), test.adc[6].len());
      assert_eq!(event.adc[7].len(), test.adc[7].len());
      assert_eq!(event.adc[8].len(), test.adc[8].len());
      assert_eq!(event.adc[0], test.adc[0]);
      assert_eq!(event.adc[1], test.adc[1]);
      assert_eq!(event.adc[2], test.adc[2]);
      assert_eq!(event.adc[3], test.adc[3]);
      assert_eq!(event.adc[4], test.adc[4]);
      assert_eq!(event.adc[5], test.adc[5]);
      assert_eq!(event.adc[6], test.adc[6]);
      assert_eq!(event.adc[7], test.adc[7]);
      assert_eq!(event.adc[8], test.adc[8]);
      //for ch in (event.header.get_channels().iter()){
      //  assert_eq!(event.adc[*ch as usize], test.adc[*ch as usize]);
      //}
      //assert_eq!(event, test);
      
      //if head.header.event_fragment == test.header.event_fragment {
      //  println!("Event fragment found, no channel data available!");
      //} else {
      //  assert_eq!(head, test);
      //}
    }
  }
  
}
