//! Readoutboard binary event formats, former denoted as BLOB (binary large object)
//! 
//! The structure is the following
//! FIXME - come up with more descriptive names, e.g. RoBinDataL0
//!
//! - RBEventMemoryView   - the raw "orignal" blob, written to the memory of the 
//!                    RB's. This corresponds to compression level 0.
//!
//! - RBEventPayload - the raw "original" data, but the event id is extracted
//!                    and available as a separate field.
//!
//! - RBEvent        - still "raw" event, however, with modified fields 
//!                    (removed superflous ones, changed meaning of some others)
//!                    Each RBEvent has a header and a body which is the channel 
//!                    data. Data in this form represents compression level 1
//!
//! - RBEventHeader  - timestamp, status, len of event, but no channel data. This
//!                    represents compression level 2
//!
//! - RBMissingHit   - a placeholder for debugging. If the MTB claims there is a hit,
//!                    but we do not see it, RBMissingHit accounts for the fact
//! 
//! * features: "random" - provides "::from_random" for all structs allowing to 
//!   populate them with random data for tests.
//!
use std::fmt;
use std::path::Path;

use crate::packets::{TofPacket, PacketType, PaddlePacket};
use crate::constants::{NWORDS, NCHN};
use crate::serialization::{u16_to_u8,
                           u8_to_u16,
                           search_for_u16,
                           Serialization,
                           SerializationError,
                           parse_bool,
                           parse_u8,
                           parse_u16,
                           parse_u32,
                           parse_u32_for_16bit_words,
                           parse_u48_for_16bit_words,
                           parse_u64};

use crate::events::DataType;

#[cfg(feature = "random")] 
use crate::FromRandom;
#[cfg(feature = "random")]
extern crate rand;
#[cfg(feature = "random")]
use rand::Rng;

/// Debug information for missing hits. 
///
/// These hits have been seen by the MTB, but we are unable to determine where 
/// they are coming from, why they are there or we simply have lost the RB 
/// information for these hits.
#[derive(Debug, Copy, Clone, PartialEq)]
pub struct RBMissingHit {
  pub event_id      : u32,
  pub ltb_hit_index : u8,
  pub ltb_id        : u8,
  pub ltb_dsi       : u8,
  pub ltb_j         : u8,
  pub ltb_ch        : u8,
  pub rb_id         : u8,
  pub rb_ch         : u8,
}

impl Serialization for RBMissingHit {
  const HEAD               : u16    = 43690; //0xAAAA
  const TAIL               : u16    = 21845; //0x5555
  const SIZE               : usize  = 15; // bytes
  
  fn from_bytestream(stream : &Vec<u8>, pos : &mut usize)
    -> Result<Self, SerializationError> {
    Self::verify_fixed(stream, pos)?;
    // verify_fixed already advances pos by 2
    let mut miss = RBMissingHit::new();
    miss.event_id      = parse_u32(stream, pos);
    miss.ltb_hit_index = parse_u8(stream, pos);
    miss.ltb_id        = parse_u8(stream, pos);
    miss.ltb_dsi       = parse_u8(stream, pos);
    miss.ltb_j         = parse_u8(stream, pos);
    miss.ltb_ch        = parse_u8(stream, pos);
    miss.rb_id         = parse_u8(stream, pos);
    miss.rb_ch         = parse_u8(stream, pos);
    *pos += 2; // account for header in verify_fixed
    Ok(miss)
  }

  fn to_bytestream(&self) -> Vec<u8> {
    let mut stream = Vec::<u8>::with_capacity(Self::SIZE);
    stream.extend_from_slice(&Self::HEAD.to_le_bytes());
    stream.extend_from_slice(&self.event_id.to_le_bytes());
    stream.extend_from_slice(&self.ltb_hit_index.to_le_bytes());
    stream.extend_from_slice(&self.ltb_id.to_le_bytes());
    stream.extend_from_slice(&self.ltb_dsi.to_le_bytes());
    stream.extend_from_slice(&self.ltb_j.to_le_bytes());
    stream.extend_from_slice(&self.ltb_ch.to_le_bytes());
    stream.extend_from_slice(&self.rb_id.to_le_bytes());
    stream.extend_from_slice(&self.rb_ch.to_le_bytes());
    stream.extend_from_slice(&Self::TAIL.to_le_bytes());
    stream
  }
}

impl RBMissingHit {

  pub fn new() -> Self {
    RBMissingHit {
      event_id      : 0,
      ltb_hit_index : 0,
      ltb_id        : 0,
      ltb_dsi       : 0,
      ltb_j         : 0,
      ltb_ch        : 0,
      rb_id         : 0,
      rb_ch         : 0,
    }
  }
}

impl fmt::Display for RBMissingHit {
  fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
    write!(f, "<RBMissingHit:
           \t event ID    {},
           \t LTB hit idx {}, 
           \t LTB ID      {}, 
           \t LTB DSI     {}, 
           \t LTB J       {}, 
           \t LTB CHN     {},   
           \t RB ID       {}, 
           \t RB CH {}>", 
           self.event_id      ,
           self.ltb_hit_index ,
           self.ltb_id        ,
           self.ltb_dsi       ,
           self.ltb_j         ,
           self.ltb_ch        ,
           self.rb_id         ,
           self.rb_ch         )
  }
}

impl Default for RBMissingHit {

  fn default() -> Self {
    Self::new()
  }
}

#[cfg(feature = "random")]
impl FromRandom for RBMissingHit {
    
  fn from_random() -> Self {
    let mut miss = Self::new();
    let mut rng = rand::thread_rng();
    miss.event_id      = rng.gen::<u32>();
    miss.ltb_hit_index = rng.gen::<u8>();
    miss.ltb_id        = rng.gen::<u8>();
    miss.ltb_dsi       = rng.gen::<u8>();
    miss.ltb_j         = rng.gen::<u8>();
    miss.ltb_ch        = rng.gen::<u8>();
    miss.rb_id         = rng.gen::<u8>();
    miss.rb_ch         = rng.gen::<u8>();
    miss
  }
}

/// A wrapper class for raw binary RB data exposing the event id
///
/// This is useful when we need the event id, but don't want to 
/// spend the CPU resources to decode the whole event.
///
#[deprecated(since="0.7.1", note="There are too many different event types. This is not special enough")]
#[derive(Debug, Clone)]
pub struct RBEventPayload {
  pub event_id : u32,
  pub payload  : Vec<u8>
}

impl RBEventPayload {

  pub fn new(event_id : u32, payload : Vec<u8>) -> Self {
    Self {
      event_id,
      payload
    }
  }
  
  /// Only decode the event id from a bytestream
  /// 
  /// The bytestream has to be starting with 
  /// HEAD
  pub fn decode_event_id(bytestream : &[u8]) -> u32 {
    let mut evid_pos = 22; // the eventid is 22 bytes from the 
                       // start including HEAD
    parse_u32_for_16bit_words(&bytestream.to_vec(), &mut evid_pos) 
  }

  pub fn from_bytestream(bytestream  : &Vec<u8>,
                         start_pos   : usize,
                         no_fragment : bool)
      -> Result<Self, SerializationError> {
    let head_pos = search_for_u16(RBEventMemoryView::HEAD, bytestream, start_pos)?; 
    // heuristic guess, jump ahead
    let tail_pos = search_for_u16(RBEventMemoryView::TAIL, bytestream, head_pos + RBEventMemoryView::SIZE - 4)?;
    // At this state, this can be a header or a full event. Check here and
    // proceed depending on the options
    if head_pos - tail_pos != RBEventMemoryView::SIZE
        && no_fragment { 
      return Err(SerializationError::EventFragment);
    }

    // we have to find and decode the event id.
    // FIXME - if we do this smarter, we can 
    //         most likely save a clone operation
    let slice          = &bytestream[head_pos..=tail_pos+2];
    let event_id       = RBEventPayload::decode_event_id(slice); 
    let mut payload    = Vec::<u8>::with_capacity(RBEventMemoryView::SIZE);
    payload.extend_from_slice(slice);
    let ev_payload     = RBEventPayload::new(event_id, payload.clone());
    Ok(ev_payload)
  }
   
  ///!  
  ///
  ///
  pub fn from_slice(slice       : &[u8],
                    do_checks   : bool)
      -> Result<RBEventPayload, SerializationError> {
    let payload        = Vec::<u8>::with_capacity(RBEventMemoryView::SIZE);
    if do_checks {
      let head_pos = search_for_u16(RBEventMemoryView::HEAD, &payload, 000000000)?; 
      let tail_pos = search_for_u16(RBEventMemoryView::TAIL, &payload, head_pos)?;
      // At this state, this can be a header or a full event. Check here and
      // proceed depending on the options
      if head_pos - tail_pos != RBEventMemoryView::SIZE { 
        return Err(SerializationError::EventFragment);
      }
    }
    //payload.extend_from_slice(slice);
    let event_id       = RBEventPayload::decode_event_id(slice);
    let ev_payload     = RBEventPayload::new(event_id, payload.clone()); 
    Ok(ev_payload)
  }
}



/// RBEventMemoryView is the closest representation of actual 
/// RB binary data in memory, with a fixed number of 
/// channels at compile time, optimized for speed by 
/// using fixed (at compile time) sizes for channels 
/// and sample size
///
/// FIXME - the channel mask is only one byte, 
///         and we can get rid of 3 bytes for 
///         the DNA
#[derive(Debug, Clone, PartialEq)]
pub struct RBEventMemoryView {
  pub head            : u16, // Head of event marker
  pub status          : u16,
  pub len             : u16,
  pub roi             : u16,
  pub dna             : u64, 
  pub fw_hash         : u16,
  pub id              : u16,   
  pub ch_mask         : u16,
  pub event_id        : u32,
  pub dtap0           : u16,
  pub dtap1           : u16,
  pub timestamp_32    : u32,
  pub timestamp_16    : u16,
  pub ch_head         : [ u16; NCHN],
  pub ch_adc          : [[u16; NWORDS];NCHN], 
  pub ch_trail        : [ u32; NCHN],
  pub stop_cell       : u16,
  pub crc32           : u32,
  pub tail            : u16, // End of event marker
}

impl RBEventMemoryView {

  // the size is fixed, assuming fixed
  // nchannel and sample size
  
  pub fn new() -> Self {
    Self {
      head            : 0, // Head of event marker
      status          : 0,
      len             : 0,
      roi             : 0,
      dna             : 0, 
      fw_hash         : 0,
      id              : 0,   
      ch_mask         : 0,
      event_id        : 0,
      dtap0           : 0,
      dtap1           : 0,
      timestamp_32    : 0,
      timestamp_16    : 0,
      ch_head         : [ 0; NCHN],
      ch_adc          : [[0; NWORDS];NCHN], 
      ch_trail        : [ 0; NCHN],
      stop_cell       : 0,
      crc32           : 0,
      tail            : 0, // End of event marker
    }
  }

  // FIXME
  pub fn decode_event_id(bytestream : &[u8]) -> Result<u32, SerializationError> {
    let stream = bytestream.to_vec();
    let mut pos = 0usize;
    let head_pos = search_for_u16(Self::HEAD, &stream, pos)?; 
    let tail_pos = search_for_u16(Self::TAIL, &stream, pos + Self::SIZE-2)?;
    // At this state, this can be a header or a full event. Check here and
    // proceed depending on the options
    if tail_pos + 2 - pos != Self::SIZE {
      error!("Event seems incomplete. Seing {} bytes, but expecting {}", tail_pos + 2 - head_pos, RBEventMemoryView::SIZE);
      //error!("{:?}", &stream[head_pos + 18526..head_pos + 18540]);
      pos = pos + 2; //start_pos += RBEventMemoryView::SIZE;
      return Err(SerializationError::EventFragment);
    }
    pos = pos + 2 + 2 + 2 + 2 + 8 + 2 + 2 + 2;
    let event_id = parse_u32_for_16bit_words(&stream, &mut pos); 
    Ok(event_id)
  }

  pub fn get_active_data_channels(&self) -> Vec<u8> {
    let mut active_channels = Vec::<u8>::with_capacity(8);
    for ch in 1..9 {
      if self.ch_mask as u8 & (ch as u8 -1).pow(2) == (ch as u8 -1).pow(2) {
        active_channels.push(ch);
      }
    }
    active_channels
  }

  
  pub fn get_n_datachan(&self) -> u8 {
    self.get_active_data_channels().len() as u8
  }
}

impl Default for RBEventMemoryView {
  fn default() -> Self {
    Self::new()
  }
}


impl fmt::Display for RBEventMemoryView {
  fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
    write!(f, "<RBEventMemoryView:\n
           \t RB {},\n
           \t len {}, \n
           \t roi {}, \n
           \t dna {}, \n
           \t hash {},   \n
           \t chmask {}, \n
           \t dtap0 {}, \n
           \t dtap1 {}, \n
           \t event id {}, \n
           \t timestamp32 {}, \n
           \t timestamp16 {}, \n 
           \t crc32 {},\n",
           self.id, self.len, self.roi, self.dna, self.fw_hash,
           self.ch_mask, self.dtap0, self.dtap1, self.event_id,
           self.timestamp_32, self.timestamp_16, self.crc32)
  }
}

impl Serialization for RBEventMemoryView {
  const SIZE : usize = 18530;
  const HEAD : u16   = 0xAAAA;
  const TAIL : u16   = 0x5555;

  fn to_bytestream(&self) -> Vec<u8> {
    let mut stream = Vec::<u8>::with_capacity(Self::SIZE);
    stream.extend_from_slice(&Self::HEAD.to_le_bytes());
    stream.extend_from_slice(&self.status  .to_le_bytes());
    stream.extend_from_slice(&self.len     .to_le_bytes());
    stream.extend_from_slice(&self.roi     .to_le_bytes());
    stream.extend_from_slice(&self.dna     .to_le_bytes());
    stream.extend_from_slice(&self.fw_hash .to_le_bytes());
    stream.extend_from_slice(&self.id      .to_le_bytes());  
    stream.extend_from_slice(&self.ch_mask .to_le_bytes());
    let four_bytes = self.event_id.to_be_bytes();
    let four_bytes_shuffle = [four_bytes[1],
                              four_bytes[0],
                              four_bytes[3],
                              four_bytes[2]];
    stream.extend_from_slice(&four_bytes_shuffle); 
    

    //stream.extend_from_slice(&self.event_id.to_le_bytes());
    stream.extend_from_slice(&self.dtap0   .to_le_bytes());
    stream.extend_from_slice(&self.dtap1   .to_le_bytes());
    stream.extend_from_slice(&self.timestamp_32.to_le_bytes());
    stream.extend_from_slice(&self.timestamp_16.to_le_bytes());
    for n in 0..NCHN {
      stream.extend_from_slice(&self.ch_head[n].to_le_bytes());
      for k in 0..NWORDS {
        stream.extend_from_slice(&self.ch_adc[n][k].to_le_bytes());  
      }
      stream.extend_from_slice(&self.ch_trail[n].to_le_bytes());
    }

    stream.extend_from_slice(&self.stop_cell.to_le_bytes());
   // four_bytes = self.crc32.to_be_bytes();
   // four_bytes_shuffle = [four_bytes[1],
   //                       four_bytes[0],
   //                       four_bytes[3],
   //                       four_bytes[2]];
   // stream.extend_from_slice(&four_bytes_shuffle); 
    stream.extend_from_slice(&self.crc32.to_le_bytes());
    stream.extend_from_slice(&Self::TAIL.to_le_bytes());
    stream
  }

  fn from_bytestream(stream : &Vec<u8>, pos : &mut usize)
    -> Result<Self, SerializationError> {
    let mut bin_data = Self::new();
    let head_pos = search_for_u16(Self::HEAD, stream, *pos)?; 
    let tail_pos = search_for_u16(Self::TAIL, stream, head_pos + Self::SIZE-2)?;
    // At this state, this can be a header or a full event. Check here and
    // proceed depending on the options
    if tail_pos + 2 - head_pos != Self::SIZE {
      error!("Event seems incomplete. Seing {} bytes, but expecting {}", tail_pos + 2 - head_pos, RBEventMemoryView::SIZE);
      //error!("{:?}", &stream[head_pos + 18526..head_pos + 18540]);
      *pos = head_pos + 2; //start_pos += RBEventMemoryView::SIZE;
      return Err(SerializationError::EventFragment);
    }
    *pos = head_pos + 2; 
    bin_data.status         = parse_u16(&stream, pos);
    bin_data.len            = parse_u16(&stream, pos);
    bin_data.roi            = parse_u16(&stream, pos);
    bin_data.dna            = parse_u64(&stream, pos); 
    bin_data.fw_hash        = parse_u16(&stream, pos);
    bin_data.id             = parse_u16(&stream, pos);   
    bin_data.ch_mask        = parse_u8 (&stream, pos) as u16;
    *pos += 1;
    bin_data.event_id       = parse_u32_for_16bit_words(&stream, pos);
    bin_data.dtap0          = parse_u16(&stream, pos);
    bin_data.dtap1          = parse_u16(&stream, pos);
    bin_data.timestamp_32   = parse_u32(&stream, pos);
    bin_data.timestamp_16   = parse_u16(&stream, pos);
    //let nch = bin_data.get_n_datachan();
    for n in 0..NCHN as usize {
      bin_data.ch_head[n]   = parse_u16(&stream, pos);
      for k in 0..NWORDS {
        bin_data.ch_adc[n][k] = 0x3FFF & parse_u16(&stream, pos);  
      }
      bin_data.ch_trail[n]  =  parse_u32(&stream, pos);
    }

    bin_data.stop_cell      =  parse_u16(&stream, pos);
    bin_data.crc32          =  parse_u32(&stream, pos);
    bin_data.head           =  Self::HEAD;
    bin_data.tail           =  Self::TAIL;
    *pos += 2; // since we deserialized the tail earlier and 
              // didn't account for it
    Ok(bin_data)
  }
}

#[cfg(feature = "random")]
impl FromRandom for RBEventMemoryView {
    
  fn from_random() -> Self {
    let mut bin_data = Self::new();
    let mut rng = rand::thread_rng();
    bin_data.head           =  0xAAAA; // Head of event marker
    bin_data.status         =  rng.gen::<u16>();
    bin_data.len            =  rng.gen::<u16>();
    bin_data.roi            =  rng.gen::<u16>();
    bin_data.dna            =  rng.gen::<u64>(); 
    bin_data.fw_hash        =  rng.gen::<u16>();
    let rb_id               =  rng.gen::<u8>() as u16;   
    bin_data.id             = rb_id;
    bin_data.id             =  rb_id << 8;   
    bin_data.ch_mask        =  rng.gen::<u8>() as u16;
    bin_data.event_id       =  rng.gen::<u32>();
    bin_data.dtap0          =  rng.gen::<u16>();
    bin_data.dtap1          =  rng.gen::<u16>();
    bin_data.timestamp_32   =  rng.gen::<u32>();
    bin_data.timestamp_16   =  rng.gen::<u16>();
    //let nch = bin_data.get_n_datachan();
    for n in 0..NCHN as usize {
      bin_data.ch_head[n]   =  rng.gen::<u16>();
      bin_data.ch_trail[n]  =  rng.gen::<u32>();
      for k in 0..NWORDS {
        bin_data.ch_adc[n][k] = 0x3FFF & rng.gen::<u16>();  
      }
    }

    bin_data.stop_cell      =  rng.gen::<u16>();
    bin_data.crc32          =  rng.gen::<u32>();
    bin_data.tail           =  0x5555; // End of event marker
    bin_data
  }
}

impl From<&RBEventPayload> for RBEventMemoryView {
  fn from(event : &RBEventPayload) -> Self {
    match RBEventMemoryView::from_bytestream(&event.payload, &mut 0) {
      Ok(event) => {
        return event;
      }
      Err(err) => { 
        error!("Can not get RBEventMemoryView from RBEventPayload! Error {err}!");
        error!("Returning empty event!");
        return RBEventMemoryView::new();
      }
    }
  }
}

#[derive(Debug, Clone)]
pub struct RBChannelData {
  pub header : u16, // that should be the channel id
  pub footer : u32, // crc32
  pub nwords : u32, // 1024                   
  pub data   : Vec<u8>,
}

impl RBChannelData {

  pub fn get_adc(&self) -> Vec<i16> {
    let mut adc = Vec::<i16>::with_capacity(self.nwords as usize);
    let mut pos = 0;
    for _ in 0..self.nwords {
      adc.push( 0x3FFF & i16::from_le_bytes([self.data[pos],self.data[pos+1]]));
      pos += 2;
    }
    adc
  }
}

///// Get traces in a conscise form from a 
///// number of RBEvents
/////
///// This will create a clone of all the 
///// traces, so they can be manipulated
///// without regrets
//pub fn unpack_traces_u16(events : &Vec<RBEvent>) -> Vec<Vec<Vec<u16>>> {
//  let nevents    = events.len();
//  let mut nchan  = 0;
//  let mut nwords = 0;
//  if nevents > 0 {
//    nchan  = events[0].header.get_nchan();
//    nwords = events[0].adc[0].len();
//  }
//  let mut traces: Vec<Vec<Vec<u16>>> = vec![vec![vec![0u16; nwords]; nevents]; nchan];
//  for ch in 0..nchan {
//    for ev in 0..nevents { 
//      for n in 0..nwords {
//        traces[ch][ev][n] = events[ev].adc[ch][n];
//      }
//    }
//  }
//  traces
//}

/// Get traces in a conscise form from a 
/// number of RBEvents
///
/// This will create a clone of all the 
/// traces, so they can be manipulated
/// without regrets
pub fn unpack_traces_f32(events : &Vec<RBEvent>) -> Vec<Vec<Vec<f32>>> {
  let nevents    = events.len();
  let mut nchan  = 0usize;
  let mut nwords = 0usize;
  // get a sane event
  for ev in events {
    if ev.adc[0].len() > 0 {
      nwords = ev.adc[0].len();   
    }
    nchan = ev.nchan as usize;
  }
  
  info!("Will construct traces cube with nchan {}, nevents {}, nwords {}", nchan, nevents, nwords);
  let mut traces: Vec<Vec<Vec<f32>>> = vec![vec![vec![0.0f32; nwords]; nevents]; nchan];
  if nevents == 0 {
    return traces;
  }
  let mut nevents_skipped = 0u32;
  for ch in 0..nchan {
    for ev in 0..nevents { 
      if events[ev].adc[ch].len() != nwords {
        // ignore corrupt events
        //println!("{}", events[ev]);
        nevents_skipped += 1;
        continue;
      }
      for n in 0..nwords {
        //println!("{}", events[ev].adc.len());
        //println!("{}", events[ev].adc[ch].len());
        //println!("{}", traces[ch][ev].len());
        //println!("{}", traces[ch].len());
        //println!("{}", traces.len());

        traces[ch][ev][n] = events[ev].adc[ch][n] as f32;
      }
    }
  }
  if nevents_skipped > 0 {
    warn!("Skipping {nevents_skipped} events due to malformed traces!");
  }
  traces
}

/// Default RB event data. 
///
/// This contains a channel adc values
/// for each active channel as 
/// well as a general header.
///
/// The order of the values in 
/// the adc vector is defined 
/// by header.get_active_channels()
///
///
#[derive(Debug, Clone)]
pub struct RBEvent {
  pub data_type : DataType,
  pub nchan     : u8,
  pub n_paddles : u8, // number of entries in paddles vector
  pub header    : RBEventHeader,
  pub adc       : Vec<Vec<u16>>,
  pub paddles   : Vec<PaddlePacket>
}

impl RBEvent {

  pub fn new() -> Self {
    let mut adc = Vec::<Vec<u16>>::with_capacity(NCHN);
    for _ in 0..NCHN {
      adc.push(Vec::<u16>::new());
    }
    Self {
      data_type  : DataType::Unknown,
      nchan      : 0,
      n_paddles  : 0,
      header     : RBEventHeader::new(),
      adc        : adc,
      paddles    : Vec::<PaddlePacket>::new(),
    }
  }

  /// Get the datatype from a bytestream when we know
  /// that it is an RBEvent
  ///
  /// The data type is encoded in byte 3
  pub fn extract_datatype(stream : &Vec<u8>) -> Result<DataType, SerializationError> {
    if stream.len() < 3 {
      return Err(SerializationError::StreamTooShort);
    }
    Ok(DataType::from_u8(&stream[2]))
  }

  pub fn extract_eventid(stream : &Vec<u8>) -> Result<u32, SerializationError> {
    if stream.len() < 28 {
      return Err(SerializationError::StreamTooShort);
    }
    // event header starts at position 5
    // in the header, it is as positon 19
    let event_id = parse_u32(stream, &mut 24);
    Ok(event_id)
  }

  pub fn is_over_adc_threshold(&self, ch : u8, threshold : u16) -> bool {
    match self.get_adc_ch(ch).iter().max() {
      None => {
        return false;
      }
      Some(max) => {
        return max > &threshold
      }
    }
  }

  /// Channels are always from 1-9
  ///
  /// This explicitly returns a clone. 
  /// FIXME - we should also return a constant
  /// refernece, however I have the feeling the
  /// public attribute is enough.
  pub fn get_adc_ch(&self, ch : u8) -> Vec::<u16> {
    if ch < 1 {
      panic!("Remember, channels go from 1-9!");
    }
    let channel : usize = ch as usize - 1;
    self.adc[channel as usize].clone()
  }

  /// If we know that the stream contains an RBEventMemeoryView, 
  /// we can convert the stream directly to a RBEvent.
  pub fn extract_from_rbeventmemoryview(stream : &Vec<u8>,
                                        pos    : &mut usize) 
    -> Result<RBEvent, SerializationError> {
    let mut event  = RBEvent::new();
    let header     = RBEventHeader::extract_from_rbeventmemoryview(stream, pos)?;
    event.header   = header;
  
    if header.lost_trigger || header.broken || header.event_fragment {
      warn!("Will not extract channel data for event {} because header indicates it is incomplete!", header.event_id);
      return Ok(event);
    }
    let mut active_channels = header.get_active_data_channels();
    let mut nchan = active_channels.len();
    // we know the header finishes with stop cell, crc and tail
    // then there is the crc32 for the channel, then the channel data as well as 
    // the header for the channel. Let's figure out if there is a channel9
    let mut skip_bytes = 2 + 4 + 2 + 4 + NWORDS*2 + 2;
    *pos -= skip_bytes; //here should be the id of the last channel
    // check if the channel id is ch9 (==8)
    let ch_id = parse_u16(stream, pos);
    *pos -= 2;
    //trace!("Last channel id! {ch_id}");    
    if ch_id == 8 {
      nchan += 1;
      // FIXME - we really should get rid 
      // of this convention!
      active_channels.push(8 + 1);
    }
    event.nchan = nchan as u8;
    // we already rewound one channel, so we have 
    // to subtrackt it here
    skip_bytes = (nchan - 1) * (NWORDS * 2 + 6);
    //if (nchan != 0) && !header.lost_trigger {
    //  skip_bytes = (nchan as usize + 1) * (NWORDS * 2 + 6);
    //}
    *pos -= skip_bytes;
    //let active_channels = header.get_active_data_channels();
    //println!("{:?}", active_channels);
    // we ignore the channel header field (which is just 
    // the channel id itself) and the trailer field which 
    // is a crc32 checksum
    // FIXME - in the future, allow for an option to calculate
    // these checksums!
    // channel data is always there, however, it 
    // actually might not be part of the trigger.
    for n in 0..nchan {
      // two bytes for the header
      let ch_head = parse_u16(stream, pos);
      //trace!("This is ch {ch_head}");
      if !active_channels.contains(&(n as u8 + 1)) {
        continue;
      }
      
      //trace!("This is ch {ch_head}");
      for _ in 0..NWORDS {  
        event.adc[n].push(0x3FFF & parse_u16(stream, pos));  
      }
      *pos += 4;
    } // end nchn loop
    *pos += 6; // skip to the tail position
    let tail = parse_u16(stream, pos);
    if tail != Self::TAIL {
      error!("After parsing the event, we found an invalid tail signature {}", tail);
      return Err(SerializationError::TailInvalid);
    }
    // per definition, an RBEvent coming from a bytestream can't have 
    // any paddle packets
    event.n_paddles = 0;
    Ok(event)
  }
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
    event.data_type = DataType::from_u8(&parse_u8(stream, pos));
    event.nchan     = parse_u8(stream, pos);
    event.n_paddles = parse_u8(stream, pos);
    event.header    = RBEventHeader::from_bytestream(stream, pos)?;
    let ch_ids      = event.header.get_active_data_channels();
    let stream_len  = stream.len();
    if event.header.event_fragment {
      error!("Fragmented event {} found! Disregarding channel data..", event.header.event_id);
      return Ok(event);
    }
    if event.header.lost_trigger {
      error!("Event {} has lost trigger! Disregarding channel data..", event.header.event_id);
      return Ok(event);
    }
    for k in 0..event.nchan {
      trace!("Found active data channel {}!", k);
      if *pos + 2*NWORDS >= stream_len {
        error!("The channel data for ch {} seems corrupt!", k);
        return Err(SerializationError::WrongByteSize {})
      }
      // 2*NWORDS because stream is Vec::<u8> and it is 16 bit words.
      let data = &stream[*pos..*pos+2*NWORDS];
      // remember, that ch ids are 1..8
      event.adc[k as usize] = u8_to_u16(data);
      *pos += 2*NWORDS;
    }
    if event.n_paddles > 0 {
      for _ in 0..event.n_paddles {
        match PaddlePacket::from_bytestream(stream, pos) {
          Err(err) => {
            error!("Can't read PaddlePacket! Err {err}");
            let mut pp = PaddlePacket::new();
            pp.valid = false;
            event.paddles.push(pp);
          },
          Ok(pp) => {
            event.paddles.push(pp);
          }
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
    let mut stream = Vec::<u8>::new();
    stream.extend_from_slice(&Self::HEAD.to_le_bytes());
    stream.push(self.data_type.to_u8());
    stream.push(self.nchan);
    stream.push(self.n_paddles);
    stream.extend_from_slice(&self.header.to_bytestream());
    // for an empty channel, we will add an empty vector
    for k in 0..self.adc.len() {
      stream.extend_from_slice(&u16_to_u8(&self.adc[k])); 
    }
    if self.n_paddles > 0 {
      for k in 0..self.n_paddles {
        stream.extend_from_slice(&self.paddles[k as usize].to_bytestream());
      }
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
    let active_channels = self.header.get_active_data_channels();
    let mut adc = Vec::<usize>::new();
    for k in 0..self.adc.len() {
      adc.push(self.adc[k].len());
    }
    write!(f, "<RBEvent 
    header        : {}
    data channels : {:?},
    adc nwords    : {:?} >",
    self.header,
    active_channels,
    adc)
  }
}

#[cfg(feature = "random")]
impl FromRandom for RBEvent {
    
  fn from_random() -> Self {
    let mut event   = RBEvent::new();
    let header      = RBEventHeader::from_random();
    let mut rng     = rand::thread_rng();
    event.data_type = DataType::Physics; 
    event.n_paddles = 0;
    event.header    = header;
    event.nchan     = 0u8; 
    let ch_ids      = event.header.get_active_data_channels();
    if !event.header.event_fragment && !event.header.lost_trigger {
      for k in ch_ids.iter() {
        debug!("Found active data channel {}!", k);
        let random_numbers: Vec<u16> = (0..NWORDS).map(|_| rng.gen()).collect();
        event.adc[(k-1) as usize] = random_numbers;
        event.nchan += 1;
      }
    }
    event
  }
}

impl From<&RBEventPayload> for RBEvent {
  fn from(event : &RBEventPayload) -> Self {
    match RBEvent::from_bytestream(&event.payload, &mut 0) {
      Ok(event) => {
        return event;
      }
      Err(err) => { 
        error!("Can not get RBEventMemoryView from RBEventPayload! Error {err}!");
        error!("Returning empty event!");
        return RBEvent::new();
      }
    }
  }
}

impl From<&TofPacket> for RBEvent {
  fn from(pk : &TofPacket) -> Self {
    if pk.packet_type == PacketType::RBEventPayload {
      match RBEvent::extract_from_rbeventmemoryview(&pk.payload, &mut 0) {
        Ok(event) => {
          return event;
        }
        Err(err) => { 
          error!("Can not get RBEventMemoryView from RBEventPayload! Error {err}!");
          error!("Returning empty event!");
          return RBEvent::new();
        }
      }
    } else {
      error!("Other packet types than RBEventPayload are not implmented yet!");
      return RBEvent::new();
    }
    //match RBEvent::from_bytestream(&pk.payload, &mut 0) {
    //  Ok(event) => {
    //    return event;
    //  }
    //  Err(err) => { 
    //    error!("Can not get RBEventMemoryView from RBEventPayload! Error {err}!");
    //    error!("Returning empty event!");
    //    return RBEvent::new();
    //  }
    //}
  }
}


/// The RBEvent header gets generated once per event
/// per RB. 
/// Contains information about event id, timestamps,
/// etc.
#[derive(Debug, Copy, Clone)]
pub struct RBEventHeader {
  pub channel_mask         : u8   , 
  pub stop_cell            : u16  , 
  pub crc32                : u32  , 
  pub dtap0                : u16  , 
  pub drs4_temp            : u16  , 
  pub is_locked            : bool , 
  pub is_locked_last_sec   : bool , 
  pub lost_trigger         : bool , 
  pub event_fragment       : bool ,
  pub fpga_temp            : u16  , 
  pub event_id             : u32  , 
  pub rb_id                : u8   , 
  pub timestamp_48         : u64  , 
  pub broken               : bool , 
}

impl RBEventHeader {

  pub fn new() -> Self {
    Self {
      channel_mask        : 0 ,  
      stop_cell           : 0 ,  
      crc32               : 0 ,  
      dtap0               : 0 ,  
      drs4_temp           : 0 ,  
      is_locked           : false,  
      is_locked_last_sec  : false,  
      lost_trigger        : false,  
      event_fragment      : false,
      fpga_temp           : 0,  
      event_id            : 0,  
      rb_id               : 0,  
      timestamp_48        : 0,  
      broken              : false,  
    }
  }

  /// Only get the eventid from a binary stream
  pub fn extract_eventid_from_rbheader(stream :&Vec<u8>) -> u32 {
    // event id is 18 bytes in (including HEAD bytes)
    let event_id = parse_u32(stream, &mut 19);
    event_id
  }

  /// Get the entire header from a full binary representation of
  /// the raw RBEventMemoryView encoded in a binary stream
  pub fn extract_from_rbeventmemoryview(stream : &Vec<u8>, pos : &mut usize) 
    -> Result<Self, SerializationError> {
    let start = *pos;
    let mut header = RBEventHeader::new();
    // we look for headers/tails from RBEventMemoryView, not header!
    let head_pos   = search_for_u16(RBEventMemoryView::HEAD, stream, *pos)?; 
    let tail_pos   = search_for_u16(RBEventMemoryView::TAIL, stream, head_pos + RBEventMemoryView::SIZE -2)?;
    // At this state, this can be a header or a full event. Check here and
    // proceed depending on the options
    *pos = head_pos + 2;    
    let status                = parse_u16(stream, pos);
    header.event_fragment     = (status & 1) == 1;
    header.lost_trigger       = (status & 2) == 2;
    header.is_locked          = (status & 4) == 4;
    header.is_locked_last_sec = (status & 8) == 8;
    header.fpga_temp    = status >> 4;
    if !header.lost_trigger {
      // in case there is no trigger, that means the DRS was busy so 
      // we won't get channel data or a stop cell
      if tail_pos + 2 - head_pos != RBEventMemoryView::SIZE {
        error!("Size of {} not expected for RBEvenHeader!", tail_pos + 2 - head_pos);
        //error!("LOST {} FRAGMENT {}" , header.lost_trigger, event_fragment);
        //let event_len = parse_u16(stream, pos);
        //error!("LEN IN WORDS {}", event_len);
        return Err(SerializationError::EventFragment);
      }
    }  
    //let event_len = parse_u16(stream, pos);
    //pos -= 2;
    //println!("Got LEN {}", event_len);
    *pos += 2 + 2 + 8 + 2 + 1; // skip len, roi, dna, fw hash and reserved part of rb_id
    header.rb_id        = parse_u8(stream, pos);
    header.channel_mask = parse_u8(stream, pos);
    *pos += 1;
    header.event_id     = parse_u32_for_16bit_words(stream, pos);
    header.dtap0        = parse_u16(stream, pos);
    header.drs4_temp    = parse_u16(stream, pos); 
    header.timestamp_48 = parse_u48_for_16bit_words(stream,pos);
    //let nchan = header.get_n_datachan();
    //let nchan = NCHN - 1;
    let nchan = 8;
    let mut skip_bytes = 0usize;
    if (nchan != 0) && !header.lost_trigger {
      skip_bytes = (nchan as usize + 1) * (NWORDS * 2 + 6);
    }
    *pos += skip_bytes;
    //println!("SKIP BYTES {} NCHAN {}", skip_bytes, nchan);
    if !header.lost_trigger {
      header.stop_cell = parse_u16(stream, pos);
    } else {
      error!("LOST TRIGGER FOUND [DRS WAS BUSY] - Event ID {}", header.event_id); 
    }
    header.crc32     = parse_u32_for_16bit_words(stream, pos);
    let tail         = parse_u16(stream, pos);
    if tail != RBEventHeader::TAIL {
      error!("No tail signature found {} bytes from the start! Found {} instead", *pos - start - 2, tail );  
    } else {
      header.broken = false;
    }
    Ok(header)
  }

  /// Again, remember, channel numbers are in 1-9
  ///
  /// FIXME - maybe we should change that, I think 
  /// we are shooting ourselves in the foot too many
  /// times now.
  pub fn get_active_data_channels(&self) -> Vec<u8> {
    let mut active_channels = Vec::<u8>::with_capacity(8);
    for ch in 1..9 {
      if self.channel_mask & (ch as u8 -1).pow(2) == (ch as u8 -1).pow(2) {
        active_channels.push(ch);
      }
    }
    active_channels
  }
 
  /// Get the number of data channels + 1 for ch9
  pub fn get_nchan(&self) -> usize {
    self.get_active_data_channels().len() + 1
  }

  pub fn get_clock_cycles_48bit(&self) -> u64 {
    self.timestamp_48
  }
  
  pub fn get_n_datachan(&self) -> u8 {
    self.get_active_data_channels().len() as u8
  }
  
  /// Returns the fpga temperature in Celsius
  pub fn get_fpga_temp(&self) -> f32 {
    todo!("Needs adc to celsius conversion!");
    #[allow(unreachable_code)]{
      let conversion : f32 = 1.0;
      let temp = conversion * self.drs4_temp as f32;
      temp
    }
  }  
}

impl Default for RBEventHeader {

  fn default() -> Self {
    Self::new()
  }
}

impl From<&Path> for RBEventHeader {
  fn from(path : &Path) -> Self {
    info!("Will read {}", path.display());
    todo!("This is not implemented yet!");
    #[allow(unreachable_code)]{
      //let file   = std::io::BufReader::new(std::fs::File::open(path).expect("Unable to open file {}"));    
      let header = Self::new();
      header
    }
  }
}

impl fmt::Display for RBEventHeader {
  fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
    write!(f, "<RBEventHeader:
           \t RB {},
           \t ch mask {}, 
           \t event id {}, 
           \t timestamp (48bit) {},
           \t locked {}, 
           \t locked last sec. {}, 
           \t lost trigger. {}, 
           \t is event fragment. {}, 
           \t drs4 Temp [C] {}, 
           \t FPGA Temp [C] {}, 
           \t stop cell     {}, 
           \t dtap0         {},
           \t crc32         {},
           \t broken        {}>",
           self.rb_id,
           self.channel_mask,
           self.event_id,
           self.timestamp_48,
           self.is_locked,
           self.is_locked_last_sec,
           self.lost_trigger,
           self.event_fragment,
           self.drs4_temp,
           -99999,
           //self.get_fpga_temp(),
           self.stop_cell,
           self.dtap0,
           self.crc32,
           self.broken)
  }
}

impl Serialization for RBEventHeader {
  
  const HEAD : u16 = 0xAAAA;
  const TAIL : u16 = 0x5555;
  const SIZE : usize = 35; // size in bytes with HEAD and TAIL

  fn from_bytestream(stream : &Vec<u8>, pos : &mut usize)
    -> Result<Self, SerializationError> {
    let mut header  = Self::new();
    Self::verify_fixed(stream, pos)?;
    header.channel_mask        = parse_u8(stream  , pos);   
    header.stop_cell           = parse_u16(stream , pos);  
    header.crc32               = parse_u32(stream , pos);  
    header.dtap0               = parse_u16(stream , pos);  
    header.drs4_temp           = parse_u16(stream , pos);  
    // FIX - we should pack these 4 bits in a byte!
    header.is_locked           = parse_bool(stream, pos);
    header.is_locked_last_sec  = parse_bool(stream, pos);
    header.lost_trigger        = parse_bool(stream, pos);
    header.event_fragment      = parse_bool(stream, pos);
    header.fpga_temp           = parse_u16(stream , pos);  
    header.event_id            = parse_u32(stream , pos);  
    header.rb_id               = parse_u8(stream  , pos);  
    header.timestamp_48        = parse_u64(stream , pos);  
    header.broken              = parse_bool(stream, pos);  
    *pos += 2; // account for tail earlier 
    Ok(header) 
  }

  fn to_bytestream(&self) -> Vec<u8> {
    let mut stream = Vec::<u8>::with_capacity(Self::SIZE);
    stream.extend_from_slice(&Self::HEAD.to_le_bytes());
    stream.extend_from_slice(&self.channel_mask      .to_le_bytes());
    stream.extend_from_slice(&self.stop_cell         .to_le_bytes());
    stream.extend_from_slice(&self.crc32             .to_le_bytes());
    stream.extend_from_slice(&self.dtap0             .to_le_bytes());
    stream.extend_from_slice(&self.drs4_temp         .to_le_bytes());
    stream.extend_from_slice(&(u8::from(self.is_locked)  .to_le_bytes()));
    stream.extend_from_slice(&(u8::from(self.is_locked_last_sec).to_le_bytes()));
    stream.extend_from_slice(&(u8::from(self.lost_trigger)      .to_le_bytes()));
    stream.extend_from_slice(&(u8::from(self.event_fragment)    .to_le_bytes()));
    stream.extend_from_slice(&self.fpga_temp         .to_le_bytes());
    stream.extend_from_slice(&self.event_id          .to_le_bytes());
    stream.extend_from_slice(&self.rb_id             .to_le_bytes());
    stream.extend_from_slice(&self.timestamp_48      .to_le_bytes());
    stream.extend_from_slice(&(u8::from(self.broken)      .to_le_bytes()));
    stream.extend_from_slice(&RBEventHeader::TAIL.to_le_bytes());
    stream
  }
}

impl PartialEq for RBEventHeader {
  fn eq(&self, other: &Self) -> bool {
      self.event_id == other.event_id
  }
}

impl PartialEq for RBEvent {
  fn eq(&self, other: &Self) -> bool {
      self.header.event_id == other.header.event_id
  }
}

#[cfg(feature = "random")]
impl FromRandom for RBEventHeader {
    
  fn from_random() -> Self {
    let mut header = RBEventHeader::new();
    let mut rng = rand::thread_rng();

    header.channel_mask         = rng.gen::<u8>();    
    header.stop_cell            = rng.gen::<u16>();   
    header.crc32                = rng.gen::<u32>();   
    header.dtap0                = rng.gen::<u16>();   
    header.drs4_temp            = rng.gen::<u16>();   
    header.is_locked            = rng.gen::<bool>();  
    header.is_locked_last_sec   = rng.gen::<bool>();  
    header.lost_trigger         = rng.gen::<bool>();  
    header.event_fragment       = rng.gen::<bool>();  
    header.fpga_temp            = rng.gen::<u16>();   
    header.event_id             = rng.gen::<u32>();   
    header.rb_id                = rng.gen::<u8>();    
    header.timestamp_48         = rng.gen::<u64>();   
    header.broken               = rng.gen::<bool>();  
    header
  }
}

#[cfg(test)]
mod test_rbevents {
  use crate::serialization::Serialization;
  use crate::FromRandom;
  use crate::events::{RBEvent,
                      RBMissingHit,
                      RBEventMemoryView,
                      RBEventHeader};
  #[test]
  fn serialization_rbeventheader() {
    let head = RBEventHeader::from_random();
    let test = RBEventHeader::from_bytestream(&head.to_bytestream(), &mut 0).unwrap();
    assert_eq!(head, test);
  }
  
  #[test]
  fn serialization_rbevent() {
    let head = RBEvent::from_random();
    let test = RBEvent::from_bytestream(&head.to_bytestream(), &mut 0).unwrap();
    assert_eq!(head.header, test.header);
    assert_eq!(head.header.get_active_data_channels(), test.header.get_active_data_channels());
    assert_eq!(head, test);
    //if head.header.event_fragment == test.header.event_fragment {
    //  println!("Event fragment found, no channel data available!");
    //} else {
    //  assert_eq!(head, test);
    //}
  }
  
  #[test]
  fn serialization_rbmissinghit() {
    let head = RBMissingHit::from_random();
    let test = RBMissingHit::from_bytestream(&head.to_bytestream(), &mut 0).unwrap();
    assert_eq!(head, test);
  }
  
  #[test]
  fn serialization_rbmemoryview() {
    let head = RBEventMemoryView::from_random();
    let test = RBEventMemoryView::from_bytestream(&head.to_bytestream(), &mut 0).unwrap();
    assert_eq!(head, test);
  }
}
