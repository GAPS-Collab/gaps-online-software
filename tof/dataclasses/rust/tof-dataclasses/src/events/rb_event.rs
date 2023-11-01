//! Readoutboard binary event formats, former denoted as BLOB (binary large object)
//! 
//! The structure is the following
//! FIXME - come up with more descriptive names, e.g. RoBinDataL0
//!
//! - RBEventMemoryView   - the raw "orignal" blob, written to the memory of the 
//!                    RB's. This corresponds to compression level 0.
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

extern crate crc;
use crc::Crc;
//extern crate libdeflater;
//use libdeflater::Crc;

use crate::packets::{TofPacket, PacketType};
use crate::events::TofHit;
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
use crate::errors::UserError;

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

/// RBEventMemoryView is the closest representation of actual 
/// RB binary data in memory, with a fixed number of 
/// channels at compile time, optimized for speed by 
/// using fixed (at compile time) sizes for channels 
/// and sample size
///
/// FIXME - the channel mask is only one byte, 
///         and we can get rid of 3 bytes for 
///         the DNA
#[deprecated(since="0.7.2", note="RBEvent is sufficient to fulfill all our needs!")]
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
      //pos = pos + 2; //start_pos += RBEventMemoryView::SIZE;
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
    let mut nchan = rng.gen::<u8>();
    while nchan > 9 {
      nchan = rng.gen::<u8>();
    }
    let roi = nchan as usize * (2 + 4 + 2 * NWORDS);  
    bin_data.head           =  0xAAAA; // Head of event marker
    bin_data.status         =  rng.gen::<u16>();
    bin_data.len            =  rng.gen::<u16>();
    bin_data.roi            =  roi as u16;
    bin_data.dna            =  rng.gen::<u64>(); 
    bin_data.fw_hash        =  rng.gen::<u16>();
    let rb_id               =  rng.gen::<u8>() as u16;   
    bin_data.id             =  rb_id;
    bin_data.id             =  rb_id << 8;   
    bin_data.ch_mask        =  rng.gen::<u8>() as u16;
    bin_data.event_id       =  rng.gen::<u32>();
    bin_data.dtap0          =  rng.gen::<u16>();
    bin_data.dtap1          =  rng.gen::<u16>();
    bin_data.timestamp_32   =  rng.gen::<u32>();
    bin_data.timestamp_16   =  rng.gen::<u16>();
    //let nch = bin_data.get_n_datachan();
    for n in 0..nchan as usize {
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



// FIXME - do we want this? OOP overkill?
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


/// Get traces in a conscise form from a 
/// number of RBEvents
///
/// This will create a clone of all the 
/// traces including ch9,
/// so they can be manipulated
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
    nchan = ev.header.get_ndatachan() as usize;
  }
  
  info!("Will construct traces cube with nchan {}, nevents {}, nwords {}", nchan, nevents, nwords);
  let mut traces: Vec<Vec<Vec<f32>>> = vec![vec![vec![0.0f32; nwords]; nevents]; nchan + 1];
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
  // ch9
  for ev in 0..nevents { 
    if !events[ev].header.has_ch9 {
      nevents_skipped += 1;
      continue
    }
    if events[ev].ch9_adc.len() != nwords {
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
      traces[8][ev][n] = events[ev].ch9_adc[n] as f32;
    }
  }
  if nevents_skipped > 0 {
    error!("Skipping {nevents_skipped} events due to malformed traces!");
  }
  traces
}

/// Event data for each individual ReadoutBoard (RB)
///
/// 
///
#[derive(Debug, Clone, PartialEq)]
pub struct RBEvent {
  pub data_type : DataType,
  pub header    : RBEventHeader,
  pub adc       : Vec<Vec<u16>>,
  pub ch9_adc   : Vec<u16>,
  pub hits      : Vec<TofHit>,
}

impl RBEvent {

  pub fn new() -> Self {
    let mut adc = Vec::<Vec<u16>>::with_capacity(NCHN);
    //for _ in 0..NCHN {
    //  adc.push(Vec::<u16>::new());
    //}
    Self {
      data_type  : DataType::Unknown,
      header     : RBEventHeader::new(),
      adc        : adc,
      ch9_adc    : Vec::<u16>::new(),
      hits       : Vec::<TofHit>::new(),
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
    // TODO This might panic! Is it ok?
    Ok(DataType::try_from(stream[2]).unwrap())
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
    if stream.len() < 28 {
      return Err(SerializationError::StreamTooShort);
    }
    // event header starts at position 5
    // in the header, it is as positon 19
    let event_id = parse_u32(stream, &mut 24);
    Ok(event_id)
  }

  pub fn get_nchan(&self) -> usize {
    let mut nchan = 0usize;
    if self.ch9_adc.len() > 0 {
      nchan += 1;
    }
    nchan += self.adc.len();
    nchan
  }
  
  pub fn get_ndatachan(&self) -> usize {
    self.adc.len()
  }

  pub fn get_channel_by_id(&self, ch : usize) -> Result<&Vec::<u16>, UserError> {
    if ch >= 9 {
      error!("channel_by_id expects numbers from 0-8!");
      return Err(UserError::IneligibleChannelLabel)
    }
    if ch < 8 {
      return Ok(&self.adc[ch]);
    } else {
      if self.header.has_ch9 {
        return Ok(&self.ch9_adc);
      } else {
        error!("No channel 9 data for this event!");
        return Err(UserError::NoChannel9Data);
      }
    }
  }

  pub fn get_channel_by_label(&self, ch : u8) -> Result<&Vec::<u16>, UserError>  {
    //let mut ch_adc = Vec::<u16>::new();
    if ch == 0 || ch > 9 {
      error!("channel_by_label expects numbers from 1-9!");
      return Err(UserError::IneligibleChannelLabel)
    }
    if ch == 9 {
      if self.header.has_ch9 {
        return Ok(&self.ch9_adc);
      } else {
        error!("No channel 9 data for this event!");
        return Err(UserError::NoChannel9Data);
      }
    }
    Ok(&self.adc[ch as usize -1])
  }

  pub fn get_adcs(&self) -> Vec<&Vec<u16>> {
    let mut adcs = Vec::<&Vec<u16>>::new();
    for v in self.adc.iter() {
      adcs.push(&v);
    }
    if self.header.has_ch9 {
      adcs.push(&self.ch9_adc);
    }
    adcs
  }

  /// If we know that the stream contains an RBEventMemeoryView, 
  /// we can convert the stream directly to a RBEvent.
  pub fn extract_from_rbeventmemoryview(stream : &Vec<u8>,
                                        pos    : &mut usize) 
    -> Result<Self, SerializationError> {
    let mut event  = Self::new();
    let header     = RBEventHeader::extract_from_rbeventmemoryview(stream, pos)?;
 
    /// calculate crc32 sums
    let crc        = crc::Crc::<u32>::new(&crc::CRC_32_ISO_HDLC);
    //let crc        = crc::Crc::<u32>::new(&crc::CRC_32_BZIP2);
    let mut ch_crc       : u32;
    let mut ch_crc_check : u32;
    if header.broken {
      error!("Broken event {}! This won't have any channel data, since the event end markes is not at the expected position. Treat the header values with caution!", &header.event_id);
      event.header   = header;
      return Ok(event);
    }
    //let mut active_channels = header.get_active_data_channels();
    //let mut nchan = active_channels.len();
    event.header = header;
    // set the position marker to the start of the channel
    // adc data field
    *pos = event.header.channel_packet_start; 
    if stream.len() < event.header.channel_packet_len + 44 {
      error!("The header says there is channel data, but the event is corrupt!");
      return Err(SerializationError::StreamTooShort);
    }

    for ch in event.header.channel_packet_ids.iter() {
      let mut dig    = crc.digest_with_initial(0);
      *pos += 2; // ch id
      if ch > &9 {
        error!("Channel ID is messed up. Not sure if this event can be saved!");
        *pos += 2*event.header.nwords;
        *pos += 4;
      } else {
        //ch_crc_check = crc.checksum(&stream[*pos..*pos+2*event.header.nwords]); 
        //let mut crc_test = Crc::new();
        //crc_test.update(&stream[*pos..*pos+2*event.header.nwords]);
        //let ch_crc_check = crc_test.sum();
        let mut this_ch_adc = Vec::<u16>::with_capacity(event.header.nwords);
        for _ in 0..event.header.nwords {  
          //let this_bytes = [stream[*pos ], stream[*pos + 1]]; 
          let this_word  = [stream[*pos], stream[*pos+1]];
          
          this_ch_adc.push(0x3FFF & parse_u16(stream, pos));
          dig.update(&this_word);
        }
        if ch < &8 {
          event.adc.push(this_ch_adc);  
        } else {
          event.ch9_adc = this_ch_adc;
        }
        ch_crc = parse_u32_for_16bit_words(stream, pos);
        ch_crc_check = dig.finalize();
        //println!("==> Calculated crc32 {}, expected crc32 {}", ch_crc_check, ch_crc);
        //*pos += 4; // trailer
      }
    }
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
    event.data_type = DataType::try_from(parse_u8(stream, pos)).unwrap();
    let nchan_data  = parse_u8(stream, pos);
    let n_hits      = parse_u8(stream, pos);
    event.header    = RBEventHeader::from_bytestream(stream, pos)?;
    //let ch_ids      = event.header.get_active_data_channels();
    let stream_len  = stream.len();
    if event.header.event_fragment {
      error!("Fragmented event {} found! Disregarding channel data..", event.header.event_id);
      return Ok(event);
    }
    if event.header.lost_trigger {
      error!("Event {} has lost trigger! Disregarding channel data..", event.header.event_id);
      return Ok(event);
    }
    for k in 0..nchan_data {
      if *pos + 2*NWORDS >= stream_len {
        error!("The channel data for ch {} seems corrupt!", k);
        return Err(SerializationError::WrongByteSize {})
      }
      // 2*NWORDS because stream is Vec::<u8> and it is 16 bit words.
      let data = &stream[*pos..*pos+2*NWORDS];
      //event.adc[k as usize] = u8_to_u16(data);
      event.adc.push(u8_to_u16(data));
      *pos += 2*NWORDS;
    }
    if event.header.has_ch9 {
      if *pos + 2*NWORDS >= stream_len {
        error!("The channel data for ch 9 (calibration channel) seems corrupt!");
        return Err(SerializationError::WrongByteSize {})
      }
      let data = &stream[*pos..*pos+2*NWORDS];
      event.ch9_adc = u8_to_u16(data);
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
    let mut stream = Vec::<u8>::new();
    stream.extend_from_slice(&Self::HEAD.to_le_bytes());
    stream.push(self.data_type as u8);
    let nchan_data  = self.adc.len() as u8;
    stream.push(nchan_data);
    let n_hits      = self.hits.len() as u8;
    stream.push(n_hits);
    stream.extend_from_slice(&self.header.to_bytestream());
    // for an empty channel, we will add an empty vector
    for channel_adc in self.adc.iter() {
      stream.extend_from_slice(&u16_to_u8(&channel_adc)); 
    }
    if self.ch9_adc.len() > 0 {
      stream.extend_from_slice(&u16_to_u8(&self.ch9_adc));
    }
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
    for k in 0..self.adc.len() {
      adc.push(self.adc[k].len());
    }
    let mut ch9_str = String::from("[");
    for k in self.ch9_adc.iter().take(5) {
      ch9_str += &k.to_string();
      ch9_str += ","
    }
    ch9_str += " .. :";
    ch9_str += &self.ch9_adc.len().to_string();
    ch9_str += "]";
    let mut ch_field = String::from("[\n");
    for (ch, vals) in self.adc.iter().enumerate() {
      let label = (ch + 1).to_string();
      ch_field += "[ch ";
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
    {}
    .. .. 
    has ch9       : {},
      -> ch9      : {},
    data channels : 
      -> {},
    n hits        : {},
    .. .. .. .. .. .. .. >",
    self.header,
    self.header.has_ch9,
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
    event.data_type = DataType::Physics; 
    event.header    = header;
    //if !event.header.event_fragment && !event.header.lost_trigger {
    for k in 0..event.header.get_nchan() {
      let random_numbers: Vec<u16> = (0..NWORDS).map(|_| rng.gen()).collect();
      event.adc.push(random_numbers);
    }
    //}
    event
  }
}


impl From<&TofPacket> for RBEvent {
  fn from(pk : &TofPacket) -> Self {
    match pk.packet_type {
      PacketType::RBEventMemoryView => {
        match RBEvent::extract_from_rbeventmemoryview(&pk.payload, &mut 0) {
          Ok(event) => {
            return event;
          }
          Err(err) => { 
            error!("Can not get RBEvent from RBEventMemoryView! Error {err}!");
            error!("Returning empty event!");
            return RBEvent::new();
          }
        }
      },
      PacketType::RBEvent => {
        match RBEvent::from_bytestream(&pk.payload, &mut 0) {
          Ok(event) => {
            return event;
          }
          Err(err) => { 
            error!("Can not decode RBEvent! Error {err}!");
            error!("Returning empty event!");
            return RBEvent::new();
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
  pub channel_mask         : u8   , 
  pub has_ch9              : bool ,
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

  // fields which don't get serialized
  pub nwords               : usize,
  pub channel_packet_len   : usize,
  pub channel_packet_start : usize,
  pub channel_packet_ids   : Vec<u8>,
}

impl RBEventHeader {

  pub fn new() -> Self {
    Self {
      channel_mask         : 0 ,  
      has_ch9              : false ,
      stop_cell            : 0 ,  
      crc32                : 0 ,  
      dtap0                : 0 ,  
      drs4_temp            : 0 ,  
      is_locked            : false,  
      is_locked_last_sec   : false,  
      lost_trigger         : false,  
      event_fragment       : false,
      fpga_temp            : 0,  
      event_id             : 0,  
      rb_id                : 0,  
      timestamp_48         : 0,  
      broken               : false,  
      // fields that won't get serialized
      nwords               : 0,
      channel_packet_len   : 0,
      channel_packet_start : 0,
      channel_packet_ids   : Vec::<u8>::with_capacity(9),
    }
  }

  /// Only get the eventid from a binary stream
  pub fn extract_eventid_from_rbheader(stream :&Vec<u8>) -> u32 {
    // event id is 18 bytes in (including HEAD bytes)
    let event_id = parse_u32(stream, &mut 19);
    event_id
  }


  /// Get the temperature value (Celsius) from the fpga_temp adc.
  pub fn get_fpga_temp(&self) -> f32 {
    let zynq_temp = (((self.fpga_temp & 4095) as f32 * 503.975) / 4096.0) - 273.15;
    zynq_temp
  }

  /// Get the entire header from a full binary representation of
  /// the raw RBEventMemoryView encoded in a binary stream
  pub fn extract_from_rbeventmemoryview(stream : &Vec<u8>, pos : &mut usize) 
    -> Result<Self, SerializationError> {
    let mut header = Self::new();
    let start = *pos;
    // we look for headers/tails from RBEventMemoryView, not header!
    let head_pos   = search_for_u16(RBEventMemoryView::HEAD, stream, *pos)?; 
    // At this state, this can be a header or a full event. Check here and
    // proceed depending on the options
    *pos = head_pos + 2;   
    // parsing the 2 bytes which contain
    // fpga_temp and status
    let mut status = parse_u16(stream, pos);

    header.event_fragment = status & 1 > 0;
    status = status >> 1;
    header.lost_trigger = status & 1 > 0;
    status = status >> 1;
    header.is_locked = status & 1 > 0;
    status = status >> 1;
    header.is_locked_last_sec = status & 1 > 0;
    status = status >> 1;
    header.fpga_temp = status;

    header.has_ch9 = false; // we check for that later
    // don't write packet len and roi to struct
    let packet_len = parse_u16(stream, pos) as usize * 2;
    let nwords     = parse_u16(stream, pos) as usize + 1; // the field will tell you the 
                                                 // max index instead of len
    debug!("Got packet len of {} bytes, roi of {}", packet_len, nwords);
    *pos += 8 + 2 + 1; // skip dna, fw hash and reserved part of rb_id
    header.rb_id        = parse_u8(stream, pos);
    header.channel_mask = parse_u8(stream, pos);
    *pos += 1;
    header.event_id     = parse_u32_for_16bit_words(stream, pos);
    header.dtap0        = parse_u16(stream, pos);
    header.drs4_temp    = parse_u16(stream, pos); 
    header.timestamp_48 = parse_u48_for_16bit_words(stream,pos);
    //let nchan = 8;
    // 36 bytes before event payload
    // 8 bytes after
    let channel_packet_start = head_pos + 36;
    let nchan_data = packet_len - 44;
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
    *pos = channel_packet_start;
    for _ in 0..nchan {
      let this_ch_id = parse_u16(stream, pos) as u8;
      if this_ch_id == 8 {
        header.has_ch9 = true;
      }
      ch_ids.push(this_ch_id);
      *pos += (nwords*2) as usize;
      *pos += 4; // trailer
    }
    debug!("Got channel ids {:?}", ch_ids);
    header.nwords               = nwords;
    header.channel_packet_len   = nchan_data;
    header.channel_packet_start = channel_packet_start as usize;
    header.channel_packet_ids   = ch_ids;

    header.stop_cell = parse_u16(stream, pos);
    header.crc32     = parse_u32_for_16bit_words(stream, pos);
    let tail         = parse_u16(stream, pos);
    if tail != RBEventHeader::TAIL {
      error!("No tail signature found {} bytes from the start! Found {} instead Will set broken flag in header!", *pos - start - 2, tail );  
    } else {
      header.broken = false;
    }
    Ok(header)
  }

  /// Decode the channel mask into channel ids.
  ///
  /// The channel ids inside the memory representation
  /// of the RB Event data ("blob") are from 0-7
  ///
  /// We keep ch9 seperate.
  pub fn decode_channel_mask(&self) -> Vec<u8> {
    let mut channels = Vec::<u8>::with_capacity(8);
    for k in 0..8 {
      if self.channel_mask & 1 << k > 0 {
        channels.push(k);
      }
    }
    channels
  }

  /// Get the number of data channels + 1 for ch9
  pub fn get_nchan(&self) -> usize {
    let mut nchan = self.decode_channel_mask().len();
    if self.has_ch9 {
      nchan += 1;
    }
    nchan
  }
  
  pub fn get_ndatachan(&self) -> usize {
    self.decode_channel_mask().len()
  }

  pub fn get_clock_cycles_48bit(&self) -> u64 {
    self.timestamp_48
  }
}

impl Default for RBEventHeader {
  fn default() -> Self {
    Self::new()
  }
}


impl fmt::Display for RBEventHeader {
  fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
    write!(f, "<RBEventHeader:
           \t RB                 {}
           \t ch mask            {} 
           \t has ch9            {}
           \t event id           {} 
           \t timestamp (48bit)  {}
           \t locked             {} 
           \t locked last sec.   {} 
           \t lost trigger.      {} 
           \t is event fragment. {} 
           \t drs4 Temp [C]      {} 
           \t FPGA Temp [C]      {} 
           \t stop cell          {} 
           \t dtap0              {}
           \t crc32              {}
           \t broken             {}>",
           self.rb_id,
           self.channel_mask,
           self.has_ch9,
           self.event_id,
           self.timestamp_48,
           self.is_locked,
           self.is_locked_last_sec,
           self.lost_trigger,
           self.event_fragment,
           self.drs4_temp,
           self.get_fpga_temp(),
           self.stop_cell,
           self.dtap0,
           self.crc32,
           self.broken)
  }
}

impl Serialization for RBEventHeader {
  
  const HEAD : u16   = 0xAAAA;
  const TAIL : u16   = 0x5555;
  const SIZE : usize = 36; // size in bytes with HEAD and TAIL

  fn from_bytestream(stream : &Vec<u8>, pos : &mut usize)
    -> Result<Self, SerializationError> {
    let mut header  = Self::new();
    Self::verify_fixed(stream, pos)?;
    header.channel_mask        = parse_u8(stream  , pos);   
    header.has_ch9             = parse_bool(stream, pos);
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
    stream.push(self.has_ch9 as u8);
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

#[cfg(feature = "random")]
impl FromRandom for RBEventHeader {
    
  fn from_random() -> Self {
    let mut header = RBEventHeader::new();
    let mut rng = rand::thread_rng();

    header.channel_mask         = rng.gen::<u8>();    
    header.has_ch9              = rng.gen::<bool>();
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

#[cfg(all(test,feature = "random"))]
mod test_rbevents {
  use crate::serialization::Serialization;
  use crate::FromRandom;
  use crate::events::{RBEvent,
                      RBMissingHit,
                      RBEventMemoryView,
                      RBEventHeader};
  #[test]
  fn serialization_rbeventheader() {
    let mut pos = 0usize;
    let head = RBEventHeader::from_random();
    let test = RBEventHeader::from_bytestream(&head.to_bytestream(), &mut pos).unwrap();
    assert_eq!(pos, RBEventHeader::SIZE);
    assert_eq!(head, test);
  }
  
  #[test]
  fn serialization_rbevent() {
    let head = RBEvent::from_random();
    let test = RBEvent::from_bytestream(&head.to_bytestream(), &mut 0).unwrap();
    assert_eq!(head.header, test.header);
    assert_eq!(head.header.get_nchan(), test.header.get_nchan());
    assert_eq!(head, test);
    //if head.header.event_fragment == test.header.event_fragment {
    //  println!("Event fragment found, no channel data available!");
    //} else {
    //  assert_eq!(head, test);
    //}
  }
  
  #[test]
  fn serialization_rbmissinghit() {
    let mut pos = 0usize;
    let head = RBMissingHit::from_random();
    let test = RBMissingHit::from_bytestream(&head.to_bytestream(), &mut pos).unwrap();
    assert_eq!(head, test);
    assert_eq!(pos, RBMissingHit::SIZE);
  }
  
  #[test]
  fn serialization_rbmemoryview() {
    let head = RBEventMemoryView::from_random();
    let test = RBEventMemoryView::from_bytestream(&head.to_bytestream(), &mut 0).unwrap();
    assert_eq!(head, test);
  }
}
