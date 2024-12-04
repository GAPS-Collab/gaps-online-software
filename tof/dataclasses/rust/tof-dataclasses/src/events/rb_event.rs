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
//use std::collections::HashMap;
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
#[derive(Debug, Copy, Clone, PartialEq)]
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
      let ord = self.get_order_str(k);
      repr += &(format!("\n  {k} -> {} ({ord})", pid.0)) 
    }
    repr += ">";
    write!(f, "{}", repr)
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
  pub fn from_rb( rb : &ReadoutBoard) -> Self {
    let mut rb_pid = RBPaddleID::new();
    rb_pid.paddle_12 = rb.paddle12.paddle_id as u8;    
    rb_pid.paddle_34 = rb.paddle34.paddle_id as u8;    
    rb_pid.paddle_56 = rb.paddle56.paddle_id as u8;    
    rb_pid.paddle_78 = rb.paddle78.paddle_id as u8;    
    let mut flipped  = 0u8 ;
    if rb.paddle12_chA != 1 {
      flipped = flipped | 0x1;
    }
    if rb.paddle34_chA != 3 {
      flipped = flipped | 0x2;
    }
    if rb.paddle56_chA != 5 {
      flipped = flipped | 0x4;
    }
    if rb.paddle78_chA != 7 {
      flipped = flipped | 0x8;
    }
    rb_pid.channel_order = flipped;
    rb_pid
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

#[cfg(feature = "random")]
impl FromRandom for RBPaddleID {
    
  fn from_random() -> Self {
    let mut rb_pid  = Self::new();
    let mut rng = rand::thread_rng();
    rb_pid.paddle_12   = rng.gen::<u8>();
    rb_pid.paddle_34   = rng.gen::<u8>();
    rb_pid.paddle_56   = rng.gen::<u8>();
    rb_pid.paddle_78   = rng.gen::<u8>();
    rb_pid.channel_order = rng.gen::<u8>();
    rb_pid
  }
}

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
      hits         : Vec::<TofHit>::new(),
    }
  }

  /// Deconstruct the RBEvent to form RBWaveforms
  pub fn get_rbwaveforms(&self) -> Vec<RBWaveform> {
    // FIXME - fix it, this drives me crazy
    let mut waveforms   = Vec::<RBWaveform>::new();
    // at max, we can have 4 waveform packets
    let active_channels = self.header.get_channels();
    let pid             = self.header.get_rbpaddleid();
    if active_channels.contains(&0) || active_channels.contains(&1) {
      let paddle_id  = pid.get_paddle_id(1);
      let mut wf     = RBWaveform::new();
      wf.rb_id       = self.header.rb_id;
      wf.event_id    = self.header.event_id;
      wf.stop_cell   = self.header.stop_cell;
      wf.paddle_id   = paddle_id.0;
      if paddle_id.1 {
        // then b is channel 1 (or 0)
        wf.adc_b   = self.adc[0].clone();
        wf.adc_a   = self.adc[1].clone();
        wf.rb_channel_b = 0;
        wf.rb_channel_a = 1;
      } else {
        wf.adc_a   = self.adc[0].clone();
        wf.adc_b   = self.adc[1].clone();
        wf.rb_channel_b = 1;
        wf.rb_channel_a = 0;
      }
      waveforms.push(wf);
    }
    if active_channels.contains(&2) || active_channels.contains(&3) {
      let paddle_id  = pid.get_paddle_id(3);
      let mut wf     = RBWaveform::new();
      wf.rb_id       = self.header.rb_id;
      wf.event_id    = self.header.event_id;
      wf.stop_cell   = self.header.stop_cell;
      wf.paddle_id   = paddle_id.0;
      if paddle_id.1 {
        // channel order flipped!
        wf.adc_b   = self.adc[2].clone();
        wf.adc_a   = self.adc[3].clone();
        wf.rb_channel_b = 2;
        wf.rb_channel_a = 3;
      } else {
        wf.adc_a   = self.adc[2].clone();
        wf.adc_b   = self.adc[3].clone();
        wf.rb_channel_b = 3;
        wf.rb_channel_a = 2;
      }
    }
    if active_channels.contains(&4) || active_channels.contains(&5) {
      let paddle_id  = pid.get_paddle_id(5);
      let mut wf     = RBWaveform::new();
      wf.rb_id       = self.header.rb_id;
      wf.event_id    = self.header.event_id;
      wf.stop_cell   = self.header.stop_cell;
      wf.paddle_id   = paddle_id.0;
      if paddle_id.1 {
        // then b is channel 1 (or 0)
        wf.adc_b   = self.adc[4].clone();
        wf.adc_a   = self.adc[5].clone();
        wf.rb_channel_b = 4;
        wf.rb_channel_a = 5;
      } else {
        wf.adc_a   = self.adc[4].clone();
        wf.adc_b   = self.adc[5].clone();
        wf.rb_channel_b = 5;
        wf.rb_channel_a = 4;
      }
    }
    if active_channels.contains(&6) || active_channels.contains(&7) {
      let paddle_id  = pid.get_paddle_id(7);
      let mut wf     = RBWaveform::new();
      wf.rb_id       = self.header.rb_id;
      wf.event_id    = self.header.event_id;
      wf.stop_cell   = self.header.stop_cell;
      wf.paddle_id   = paddle_id.0;
      if paddle_id.1 {
        // then b is channel 1 (or 0)
        wf.adc_b   = self.adc[6].clone();
        wf.adc_a   = self.adc[7].clone();
        wf.rb_channel_b = 6;
        wf.rb_channel_a = 7;
      } else {
        wf.adc_a   = self.adc[6].clone();
        wf.adc_b   = self.adc[7].clone();
        wf.rb_channel_b = 6;
        wf.rb_channel_a = 7;
      }
    }
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
  }

  pub fn get_channel_by_label(&self, ch : u8) -> Result<&Vec::<u16>, UserError>  {
    if ch == 0 || ch > 9 {
      error!("channel_by_label expects numbers from 1-9!");
      return Err(UserError::IneligibleChannelLabel)
    }
    Ok(&self.adc[ch as usize -1])
  }
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
  /// RBPaddleID - component 
  pub pid_ch12             : u8,
  /// RBPaddleID - component
  pub pid_ch34             : u8,
  /// RBPaddleID - component
  pub pid_ch56             : u8,
  /// RBPaddleID - component
  pub pid_ch78             : u8,
  /// RBPaddleID - component
  pub pid_ch_order         : u8,
  /// Reserved
  pub rsvd1                : u8,
  /// Reserved
  pub rsvd2                : u8,
  /// Reserved
  pub rsvd3                : u8,
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
      pid_ch12              : 0,
      pid_ch34              : 0,
      pid_ch56              : 0,
      pid_ch78              : 0,
      pid_ch_order          : 0,
      rsvd1                 : 0,
      rsvd2                 : 0,
      rsvd3                 : 0,
      fpga_temp             : 0,  
      drs_deadtime          : 0,
      timestamp32           : 0,
      timestamp16           : 0,
      deadtime_instead_temp : false,
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

  pub fn get_rbpaddleid(&self) -> RBPaddleID {
    let mut pid = RBPaddleID::new();
    pid.paddle_12     = self.pid_ch12;
    pid.paddle_34     = self.pid_ch34;
    pid.paddle_56     = self.pid_ch56;
    pid.paddle_78     = self.pid_ch78;
    pid.channel_order = self.pid_ch_order;
    pid                    
  }
  
  pub fn set_rbpaddleid(&mut self, pid : &RBPaddleID) {
    self.pid_ch12     = pid.paddle_12;
    self.pid_ch12     = pid.paddle_34;
    self.pid_ch12     = pid.paddle_56;
    self.pid_ch12     = pid.paddle_78;
    self.pid_ch_order = pid.channel_order;
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

  /// Get the active paddles
  pub fn get_active_paddles(&self) -> Vec<(u8,bool)> {
    // FIXME - help. Make this nicer. My brain is fried 
    // at this point. Please. I'll be thankful.
    let mut active_paddles = Vec::<(u8,bool)>::new();
    let active_channels = self.get_channels();
    let pid             = self.get_rbpaddleid();
    let mut ch0_pair_done = false;
    let mut ch2_pair_done = false;
    let mut ch4_pair_done = false;
    let mut ch6_pair_done = false;
    for ch in active_channels {
      if (ch == 0 || ch == 1) && !ch0_pair_done {
        active_paddles.push(pid.get_paddle_id(ch));
        ch0_pair_done = true;
      }
      if (ch == 2 || ch == 3) && !ch2_pair_done {
        active_paddles.push(pid.get_paddle_id(ch));
        ch2_pair_done = true;
      }
      if (ch == 4 || ch == 5) && !ch4_pair_done {
        active_paddles.push(pid.get_paddle_id(ch));
        ch4_pair_done = true;
      }
      if (ch == 6 || ch == 7) && !ch6_pair_done {
        active_paddles.push(pid.get_paddle_id(ch));
        ch6_pair_done = true;
      }
    }
    active_paddles
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
    let mut repr = String::from("<RBEventHeader:");
    repr += &(format!("\n  RB ID            {}",self.rb_id               )); 
    repr += &(format!("\n  event id         {}",self.event_id            ));  
    repr += &(format!("\n  ch mask          {}",self.channel_mask        ));  
    repr += &(format!("\n  has ch9          {}",self.has_ch9()           )); 
    repr += &(format!("\n  ch mapping       {}",self.get_rbpaddleid()    ));
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
    header.pid_ch12              = parse_u8(stream, pos);
    header.pid_ch34              = parse_u8(stream, pos);
    header.pid_ch56              = parse_u8(stream, pos);
    header.pid_ch78              = parse_u8(stream, pos);
    header.pid_ch_order          = parse_u8(stream, pos);
    header.rsvd1                 = parse_u8(stream, pos);
    header.rsvd2                 = parse_u8(stream, pos);
    header.rsvd3                 = parse_u8(stream, pos);
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
    stream.push(self.pid_ch12    );
    stream.push(self.pid_ch34    );
    stream.push(self.pid_ch56    );
    stream.push(self.pid_ch78    );
    stream.push(self.pid_ch_order);
    stream.push(self.rsvd1       );
    stream.push(self.rsvd2       );
    stream.push(self.rsvd3       );
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
    header.pid_ch12              = rng.gen::<u8>();
    header.pid_ch34              = rng.gen::<u8>();
    header.pid_ch56              = rng.gen::<u8>();
    header.pid_ch78              = rng.gen::<u8>();
    header.pid_ch_order          = rng.gen::<u8>();
    header.rsvd1                 = rng.gen::<u8>();
    header.rsvd2                 = rng.gen::<u8>();
    header.rsvd3                 = rng.gen::<u8>();
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

#[test]
#[cfg(feature = "random")]
fn pack_rbpaddleid() {
  for _ in 0..100 {
    let pid = RBPaddleID::from_random();
    let test = pid.to_u64();
    let pid_back = RBPaddleID::from_u64(test);
    assert_eq!(pid, pid_back);
  }
}

#[test]
fn rbpaddleid_from_rb() {
let mut rng = rand::thread_rng();
  let channels = vec![1u8, 2u8, 3u8, 4u8, 5u8, 6u8, 7u8, 7u8];
  for _ in 0..100 {
    let mut rb = ReadoutBoard::new();
    rb.paddle12.paddle_id   = rng.gen::<u8>() as i16;
    let mut idx = rng.gen_range(0..2);
    rb.paddle12_chA         = channels[idx];
    idx = rng.gen_range(2..4);
    rb.paddle34.paddle_id   = rng.gen::<u8>() as i16;
    rb.paddle34_chA         = channels[idx];
    idx = rng.gen_range(4..6);
    rb.paddle56.paddle_id   = rng.gen::<u8>() as i16;
    rb.paddle56_chA         = channels[idx];
    idx = rng.gen_range(6..8);
    rb.paddle78.paddle_id   = rng.gen::<u8>() as i16;
    rb.paddle78_chA         = channels[idx];

    let pid                 = RBPaddleID::from_rb(&rb);
    assert_eq!(pid.paddle_12, rb.paddle12.paddle_id as u8);
    assert_eq!(pid.paddle_34, rb.paddle34.paddle_id as u8);
    assert_eq!(pid.paddle_56, rb.paddle56.paddle_id as u8);
    assert_eq!(pid.paddle_78, rb.paddle78.paddle_id as u8);
    for ch in &channels {
      if pid.get_order_flipped(*ch) {
        if *ch == 1 || *ch == 2 {
          assert_eq!(rb.paddle12_chA,2);
        }
        if *ch == 3 || *ch == 4 {
          assert_eq!(rb.paddle12_chA,4);
        }
        if *ch == 5 || *ch == 6 {
          assert_eq!(rb.paddle12_chA,6);
        }
        if *ch == 7 || *ch == 8 {
          assert_eq!(rb.paddle12_chA,8);
        }
      }
    }
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
