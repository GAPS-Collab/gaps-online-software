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
//! - RBMissingHit   - a placeholder for debugging. If the MTB claims there is a hit,
//!                    but we do not see it, RBMissingHit accounts for the fact
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

use colored::Colorize;

use crate::packets::{TofPacket, PacketType};
use crate::events::TofHit;
use crate::constants::{NWORDS, NCHN};
use crate::serialization::{
    u8_to_u16,
    Serialization,
    SerializationError,
    Packable,
    parse_u8,
    parse_u16,
    parse_u32,
};

use crate::events::{
    DataType,
    EventStatus,
};
use crate::errors::UserError;
use crate::io::RBEventMemoryStreamer;

cfg_if::cfg_if! {
  if #[cfg(feature = "random")]  {
    use crate::FromRandom;
    extern crate rand;
    use rand::Rng;
  }
}


/// Debug information for missing hits. 
///
/// These hits have been seen by the MTB, but we are unable to determine where 
/// they are coming from, why they are there or we simply have lost the RB 
/// information for these hits.
#[deprecated(since = "0.10.0", note="feature was never really used")]
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

#[allow(deprecated)]
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

#[allow(deprecated)]
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

#[allow(deprecated)]
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

#[allow(deprecated)]
impl Default for RBMissingHit {

  fn default() -> Self {
    Self::new()
  }
}

#[cfg(feature = "random")]
#[allow(deprecated)]
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
    for ch in self.header.get_channels() {
      let mut wf     = RBWaveform::new();
      wf.rb_id       = self.header.rb_id;
      wf.rb_channel  = ch;
      wf.event_id    = self.header.event_id;
      wf.stop_cell   = self.header.stop_cell;
      // FIXME - can we move this somehow instead of 
      // cloning?
      wf.adc         = self.adc[ch as usize].clone();
      waveforms.push(wf);
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
      error!("Fragmented event {} found! Disregarding channel data..", event.header.event_id);
      return Ok(event);
    }
    if event.header.drs_lost_trigger() {
      error!("Event {} has lost trigger! Disregarding channel data..", event.header.event_id);
      return Ok(event);
    }
    let mut decoded_ch = Vec::<u8>::new();
    for ch in event.header.get_channels().iter() {
      if *pos + 2*NWORDS >= stream_len {
        error!("The channel data for event {} ch {} seems corrupt! We want to get channels {:?}, but have decoded only {:?}, because the stream ends {} bytes too early!",event.header.event_id, ch, event.header.get_channels(), decoded_ch, *pos + 2*NWORDS - stream_len);
        return Err(SerializationError::WrongByteSize {})
      }
      decoded_ch.push(*ch);
      // 2*NWORDS because stream is Vec::<u8> and it is 16 bit words.
      let data = &stream[*pos..*pos+2*NWORDS];
      //event.adc[k as usize] = u8_to_u16(data);
      event.adc[*ch as usize] = u8_to_u16(data);
      *pos += 2*NWORDS;
    }
    //if event.header.has_ch9() {
    //  if *pos + 2*NWORDS >= stream_len {
    //    error!("The channel data for ch 9 (calibration channel) seems corrupt!");
    //    return Err(SerializationError::WrongByteSize {})
    //  }
    //  //let data = &stream[*pos..*pos+2*NWORDS];
    //  //event.ch9_adc = u8_to_u16(data);
    //  //*pos += 2*NWORDS;
    //}
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
  pub rb_id                : u8   , 
  pub event_id             : u32  , 
  pub status_byte          : u8   ,
  // FIXME - channel mask still has space
  // for the status_bytes, since it only
  // uses 9bits
  pub channel_mask         : u16  , 
  pub stop_cell            : u16  , 
  // we change this by keeping the byte
  // order the same to accomodate the sine 
  // values
  pub ch9_amp                : u16,
  pub ch9_freq               : u16,
  pub ch9_phase              : u32,
  //pub crc32                : u32  , 
  //pub dtap0                : u16  , 
  //pub drs4_temp            : u16  , 
  pub fpga_temp            : u16  , 
  pub timestamp32          : u32  ,
  pub timestamp16          : u16  ,
  // channels (0-8)
  //pub channels             : Vec<u8>,
  //// fields which don't get serialized
  //pub nwords               : usize,
  //pub channel_packet_len   : usize,
  //pub channel_packet_start : usize,
  //pub channel_packet_ids   : Vec<u8>,
}

impl RBEventHeader {

  pub fn new() -> Self {
    Self {
      rb_id                : 0,  
      status_byte          : 0, 
      event_id             : 0,  
      channel_mask         : 0 ,  
      stop_cell            : 0 ,  
      ch9_amp              : 0 ,
      ch9_freq             : 0 ,
      ch9_phase            : 0 ,
      //crc32                : 0 ,  
      //dtap0                : 0 ,  
      //drs4_temp            : 0 ,  
      fpga_temp            : 0,  
      timestamp32          : 0,
      timestamp16          : 0,
      //channels             : Vec::<u8>::with_capacity(9),
      //// fields that won't get serialized
      //nwords               : 0,
      //channel_packet_len   : 0,
      //channel_packet_start : 0,
      //channel_packet_ids   : Vec::<u8>::with_capacity(9),
    }
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
    //let mut status          = status_bytes;
    //self.event_fragment     = status & 1 > 0;
    //status                  = status >> 1;
    //self.lost_trigger       = status & 1 > 0;
    //status                  = status >> 1;
    // FIXME - rename these fields
    //self.lost_lock          = status & 1 > 0;
    //status                  = status >> 1;
    //self.lost_lock_last_sec = status & 1 > 0;
    //status                  = status >> 1;
    //self.fpga_temp = status;
    self.fpga_temp = status_bytes >> 4;
  }

  /// Get the temperature value (Celsius) from the fpga_temp adc.
  pub fn get_fpga_temp(&self) -> f32 {
    let zynq_temp = (((self.fpga_temp & 4095) as f32 * 503.975) / 4096.0) - 273.15;
    zynq_temp
  }

  /// Get the entire header from a full binary representation of
  /// the raw RBEventMemoryView encoded in a binary stream
  //pub fn extract_from_rbeventmemoryview(stream : &Vec<u8>, pos : &mut usize) 
  //  -> Result<Self, SerializationError> {
  //  let mut header = Self::new();
  //  let start = *pos;
  //  // we look for headers/tails from RBEventMemoryView, not header!
  //  let head_pos   = search_for_u16(RBEvent::HEAD, stream, *pos)?; 
  //  // At this state, this can be a header or a full event. Check here and
  //  // proceed depending on the options
  //  *pos = head_pos + 2;   
  //  // parsing the 2 bytes which contain
  //  // fpga_temp and status
  //  let mut status = parse_u16(stream, pos);
  //  header.parse_status(status);

  //  header.has_ch9 = false; // we check for that later
  //  // don't write packet len and roi to struct
  //  let packet_len = parse_u16(stream, pos) as usize * 2;
  //  let nwords     = parse_u16(stream, pos) as usize + 1; // the field will tell you the 
  //                                               // max index instead of len
  //  debug!("Got packet len of {} bytes, roi of {}", packet_len, nwords);
  //  *pos += 8 + 2 + 1; // skip dna, fw hash and reserved part of rb_id
  //  header.rb_id        = parse_u8(stream, pos);
  //  header.channel_mask = parse_u8(stream, pos);
  //  *pos += 1;
  //  header.event_id     = parse_u32_for_16bit_words(stream, pos);
  //  header.dtap0        = parse_u16(stream, pos);
  //  header.drs4_temp    = parse_u16(stream, pos); 
  //  
  //  //header.timestamp_48 = parse_u48_for_16bit_words(stream,pos);
  //  //let nchan = 8;
  //  // 36 bytes before event payload
  //  // 8 bytes after
  //  let channel_packet_start = head_pos + 36;
  //  let nchan_data = packet_len - 44;
  //  let mut nchan = 0usize;
  //  //println!("========================================");
  //  //println!("{} {} {}", nchan, nwords, nchan_data);
  //  //println!("========================================");
  //  while nchan * (2*nwords + 6) < nchan_data {
  //    nchan += 1;
  //  }
  //  if nchan * (2*nwords + 6) != nchan_data {
  //    error!("NCHAN consistency check failed! nchan {} , nwords {}, packet_len {}", nchan, nwords, packet_len);
  //  }
  //  let mut ch_ids = Vec::<u8>::new();
  //  *pos = channel_packet_start;
  //  for _ in 0..nchan {
  //    let this_ch_id = parse_u16(stream, pos) as u8;
  //    if this_ch_id == 8 {
  //      header.has_ch9 = true;
  //    }
  //    ch_ids.push(this_ch_id);
  //    *pos += (nwords*2) as usize;
  //    *pos += 4; // trailer
  //  }
  //  debug!("Got channel ids {:?}", ch_ids);
  //  header.nwords               = nwords;
  //  header.channel_packet_len   = nchan_data;
  //  header.channel_packet_start = channel_packet_start as usize;
  //  header.channel_packet_ids   = ch_ids;
  //  header.stop_cell = parse_u16(stream, pos);
  //  header.crc32     = parse_u32_for_16bit_words(stream, pos);
  //  let tail         = parse_u16(stream, pos);
  //  if tail != RBEventHeader::TAIL {
  //    error!("No tail signature found {} bytes from the start! Found {} instead Will set broken flag in header!", *pos - start - 2, tail );  
  //  }
  //  Ok(header)
  //}

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
    repr += &("\n  RB ID            ".to_owned() + &self.rb_id.to_string()); 
    repr += &("\n  event id         ".to_owned() + &self.event_id.to_string());  
    repr += &("\n  ch mask          ".to_owned() + &self.channel_mask.to_string());  
    repr += &("\n  has ch9          ".to_owned() + &self.has_ch9().to_string()); 
    //repr += &("\n  DRS4 temp [C]    ".to_owned() + &self.drs4_temp.to_string());  
    repr += &sine_field;
    //repr += &("\n  FPGA temp [\u{00B0}C]    ".to_owned() + &self.get_fpga_temp().to_string()); 
    repr += &(format!("\n  FPGA T [\u{00B0}C]    : {:.2}", self.get_fpga_temp()));
    repr += &("\n  timestamp32      ".to_owned() + &self.timestamp32.to_string()); 
    repr += &("\n  timestamp16      ".to_owned() + &self.timestamp16.to_string()); 
    repr += &("\n   |-> timestamp48 ".to_owned() + &self.get_timestamp48().to_string()); 
    repr += &("\n  stop cell        ".to_owned() + &self.stop_cell.to_string()); 
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
    header.rb_id               = parse_u8(stream  , pos);  
    header.event_id            = parse_u32(stream , pos);  
    header.channel_mask        = parse_u16(stream  , pos);   
    header.status_byte         = parse_u8(stream, pos);
    header.stop_cell           = parse_u16(stream , pos);  
    header.ch9_amp             = parse_u16(stream, pos);
    header.ch9_freq            = parse_u16(stream, pos);
    header.ch9_phase           = parse_u32(stream, pos);
    //header.crc32               = parse_u32(stream , pos);  
    //header.dtap0               = parse_u16(stream , pos);  
    //header.drs4_temp           = parse_u16(stream , pos);  
    header.fpga_temp           = parse_u16(stream , pos);  
    header.timestamp32         = parse_u32(stream, pos);
    header.timestamp16         = parse_u16(stream, pos);
    *pos += 2; // account for tail earlier 
    Ok(header) 
  }
  

  fn to_bytestream(&self) -> Vec<u8> {
    let mut stream = Vec::<u8>::with_capacity(Self::SIZE);
    stream.extend_from_slice(&Self::HEAD.to_le_bytes());
    stream.extend_from_slice(&self.rb_id             .to_le_bytes());
    stream.extend_from_slice(&self.event_id          .to_le_bytes());
    stream.extend_from_slice(&self.channel_mask      .to_le_bytes());
    stream.extend_from_slice(&self.status_byte       .to_le_bytes());
    stream.extend_from_slice(&self.stop_cell         .to_le_bytes());
    stream.extend_from_slice(&self.ch9_amp           .to_le_bytes());
    stream.extend_from_slice(&self.ch9_freq          .to_le_bytes());
    stream.extend_from_slice(&self.ch9_phase         .to_le_bytes());
    //stream.extend_from_slice(&self.crc32             .to_le_bytes());
    //stream.extend_from_slice(&self.dtap0             .to_le_bytes());
    //stream.extend_from_slice(&self.drs4_temp         .to_le_bytes());
    stream.extend_from_slice(&self.fpga_temp         .to_le_bytes());
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

    header.rb_id                = rng.gen::<u8>();    
    header.event_id             = rng.gen::<u32>();   
    header.channel_mask         = rng.gen::<u16>();    
    header.status_byte          = rng.gen::<u8>();    
    header.stop_cell            = rng.gen::<u16>();   
    header.ch9_amp              = rng.gen::<u16>();
    header.ch9_freq             = rng.gen::<u16>();
    header.ch9_phase            = rng.gen::<u32>();
    //header.crc32                = rng.gen::<u32>();   
    //header.dtap0                = rng.gen::<u16>();   
    //header.drs4_temp            = rng.gen::<u16>();   
    header.fpga_temp            = rng.gen::<u16>();   
    header.timestamp32          = rng.gen::<u32>();
    header.timestamp16          = rng.gen::<u16>();
    header
  }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RBWaveform {
  pub event_id   : u32,
  pub rb_id      : u8,
  pub rb_channel : u8,
  /// DRS4 stop cell
  pub stop_cell  : u16,
  pub adc        : Vec<u16>,
}

impl RBWaveform {
  
  pub fn new() -> Self {
    Self {
      event_id   : 0,
      rb_id      : 0,
      rb_channel : 0,
      stop_cell  : 0,
      adc        : Vec::<u16>::new(),
    }
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
    wf.rb_channel        = parse_u8 (stream, pos);
    wf.stop_cell         = parse_u16(stream, pos);
    if stream.len() < *pos+2*NWORDS {
      return Err(SerializationError::StreamTooShort);
    }
    let data             = &stream[*pos..*pos+2*NWORDS];
    wf.adc               = u8_to_u16(data);
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
    stream.extend_from_slice(&self.rb_channel.to_le_bytes());
    stream.extend_from_slice(&self.stop_cell.to_le_bytes());
    if self.adc.len() != 0 {
      for k in 0..NWORDS {
        stream.extend_from_slice(&self.adc[k].to_le_bytes());  
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
    repr += &(format!("\n  Channel   : {}", self.rb_channel));
    repr += &(format!("\n  Stop cell : {}", self.stop_cell));
    if self.adc.len() >= 273 {
      repr += &(format!("\n  adc [{}]      : .. {} {} {} ..",self.adc.len(), self.adc[270], self.adc[271], self.adc[272]));
    } else {
      repr += &(String::from("\n  adc [EMPTY]"));
    }
    write!(f, "{}", repr)
  }
}

#[cfg(feature = "random")]
impl FromRandom for RBWaveform {
    
  fn from_random() -> Self {
    let mut wf    = Self::new();
    let mut rng   = rand::thread_rng();
    wf.event_id   = rng.gen::<u32>();
    wf.rb_id      = rng.gen::<u8>();
    wf.rb_channel = rng.gen::<u8>();
    wf.stop_cell  = rng.gen::<u16>();
    let random_numbers: Vec<u16> = (0..NWORDS).map(|_| rng.gen()).collect();
    wf.adc        = random_numbers;
    wf
  }
}
#[cfg(all(test,feature = "random"))]
mod test_rbevents {
  use crate::serialization::Serialization;
  use crate::FromRandom;
  use crate::events::{
      RBEvent,
      RBMissingHit,
      RBEventHeader,
      RBWaveform
  };
  #[test]
  fn serialization_rbeventheader() {
    let mut pos = 0usize;
    let head = RBEventHeader::from_random();
    let stream = head.to_bytestream();
    assert_eq!(stream.len(), RBEventHeader::SIZE);
    let test = RBEventHeader::from_bytestream(&stream, &mut pos).unwrap();
    assert_eq!(pos, RBEventHeader::SIZE);
    assert_eq!(head, test);
    assert_eq!(head.lost_lock()         , test.lost_lock());
    assert_eq!(head.lost_lock_last_sec(), test.lost_lock_last_sec());
    assert_eq!(head.drs_lost_trigger()  , test.drs_lost_trigger());

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
  
  #[test]
  fn serialization_rbwaveform() {
    for _ in 0..100 {
      let wf     = RBWaveform::from_random();
      let stream = wf.to_bytestream();
      let test   = RBWaveform::from_bytestream(&stream, &mut 0).unwrap();
      assert_eq!(wf, test);
    }
  }

  #[test]
  fn serialization_rbmissinghit() {
    let mut pos = 0usize;
    let head = RBMissingHit::from_random();
    let test = RBMissingHit::from_bytestream(&head.to_bytestream(), &mut pos).unwrap();
    assert_eq!(head, test);
    assert_eq!(pos, RBMissingHit::SIZE);
  }  
}
