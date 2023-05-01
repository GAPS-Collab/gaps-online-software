//! Readoutboard binary event formats, former denoted as BLOB (binary large object)
//! 
//! The structure is the following
//!
//! - RBBinaryDump  - the raw "orignal" blob, written to the memory of the 
//!   RB's
//!
//! - RBEvent       - still "raw" event, however, with modified fields 
//!                   (removed superflous ones, changed meaning of some others)
//!                   Each RBEvent has a header and a body
//!
//! - RBEventHeader - timestamp, status, len of event
//! - RBEventBody   - the raw channel adc data
//!
//!

use std::fmt;
use std::path::Path;
use std::io;
use std::io::BufRead;
use std::io::BufReader;
use std::fs::File;

use crate::constants::{NWORDS, NCHN, MAX_NUM_PEAKS};
use crate::serialization::Serialization;
use crate::serialization::SerializationError;
use crate::serialization::search_for_u16;
use crate::serialization::{parse_bool,
                           parse_u8,
                           parse_u16,
                           parse_u32,
                           parse_u32_for_16bit_words,
                           parse_u48_for_16bit_words,
                           parse_u64};
#[cfg(feature = "random")]
use crate::FromRandom;
#[cfg(feature = "random")]
extern crate rand;
#[cfg(feature = "random")]
use rand::Rng;
// helper
fn read_lines<P>(filename: P) -> io::Result<io::Lines<io::BufReader<File>>>
where P: AsRef<Path>, {
  let file = File::open(filename)?;
    Ok(io::BufReader::new(file).lines())
  }



/// RBBinaryDump is the closest representation of actual 
/// RB binary data in memory, with a fixed number of 
/// channels at compile time, optimized for speed by 
/// using fixed (at compile time) sizes for channels 
/// and sample size
///
/// FIXME - the channel mask is only one byte, 
///         and we can get rid of 3 bytes for 
///         the DNA
#[derive(Debug, Clone, PartialEq)]
pub struct RBBinaryDump {
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

impl RBBinaryDump {

  // the size is fixed, assuming fixed
  // nchannel and sample size
  const SIZE : usize = 18530;
  const HEAD : u16   = 0xAAAA;
  const TAIL : u16   = 0x5555;
  pub fn new() -> RBBinaryDump {
    RBBinaryDump {
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

  pub fn get_active_data_channels(&self) -> Vec<u8> {
    let mut active_channels = Vec::<u8>::with_capacity(8);
    for ch in 1..9 {
      if ((self.ch_mask as u8 & (ch as u8 -1).pow(2)) == (ch as u8 -1).pow(2)) {
        active_channels.push(ch);
      }
    }
    active_channels
  }

  
  pub fn get_n_datachan(&self) -> u8 {
    self.get_active_data_channels().len() as u8
  }


}

impl Default for RBBinaryDump {
  fn default() -> RBBinaryDump {
    RBBinaryDump::new()
  }
}


impl fmt::Display for RBBinaryDump {
  fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
    write!(f, "<RBBinaryDump:\n
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

impl Serialization for RBBinaryDump {

  fn to_bytestream(&self) -> Vec<u8> {
    let mut stream = Vec::<u8>::with_capacity(RBBinaryDump::SIZE);
    stream.extend_from_slice(&RBBinaryDump::HEAD.to_le_bytes());
    stream.extend_from_slice(&self.status  .to_le_bytes());
    stream.extend_from_slice(&self.len     .to_le_bytes());
    stream.extend_from_slice(&self.roi     .to_le_bytes());
    stream.extend_from_slice(&self.dna     .to_le_bytes());
    stream.extend_from_slice(&self.fw_hash .to_le_bytes());
    stream.extend_from_slice(&self.id      .to_le_bytes());  
    stream.extend_from_slice(&self.ch_mask .to_le_bytes());
    let mut four_bytes = self.event_id.to_be_bytes();
    let mut four_bytes_shuffle = [four_bytes[1],
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
    stream.extend_from_slice(&RBBinaryDump::TAIL.to_le_bytes());
    stream
  }

  fn from_bytestream(stream : &Vec<u8>, pos : &mut usize)
    -> Result<RBBinaryDump, SerializationError> {
    let mut bin_data = RBBinaryDump::new();
    let head_pos = search_for_u16(RBBinaryDump::HEAD, stream, *pos)?; 
    let tail_pos = search_for_u16(RBBinaryDump::TAIL, stream, head_pos + RBBinaryDump::SIZE-2)?;
    // At this state, this can be a header or a full event. Check here and
    // proceed depending on the options
    if tail_pos + 2 - head_pos != RBBinaryDump::SIZE {
      error!("Event seems incomplete. Seing {} bytes, but expecting {}", tail_pos + 2 - head_pos, RBBinaryDump::SIZE);
      //error!("{:?}", &stream[head_pos + 18526..head_pos + 18540]);
      *pos = head_pos + 2; //start_pos += RBBinaryDump::SIZE;
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
    bin_data.head           =  RBBinaryDump::HEAD;
    bin_data.tail           =  RBBinaryDump::TAIL;
    *pos += 2; // since we deserialized the tail earlier and 
              // didn't account for it
    Ok(bin_data)
  }
}

#[cfg(feature = "random")]
impl FromRandom for RBBinaryDump {
    
  fn from_random() -> RBBinaryDump {
    let mut bin_data = RBBinaryDump::new();
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
    for n in 0..self.nwords {
      adc.push( 0x3FFF & i16::from_le_bytes([self.data[pos],self.data[pos+1]]));
      pos += 2;
    }
    adc
  }

}

#[derive(Debug,Clone)]
pub struct RBEventBody {
  pub nwords   : u16,
  pub nchannel : u8,
  pub data     : Vec<u8>,
}

impl RBEventBody {

  pub fn get_channel(&self, id : u8) {
    //usize pos = 0 ;

  }
}

pub struct RBEvent {}

#[derive(Debug, Copy, Clone, PartialEq)]
pub struct RBEventHeader {
  pub channel_mask         : u8   , 
  pub stop_cell            : u16  , 
  pub crc32                : u32  , 
  pub dtap0                : u16  , 
  pub drs4_temp            : u16  , 
  pub is_locked            : bool , 
  pub is_locked_last_sec   : bool , 
  pub lost_trigger         : bool , 
  pub fpga_temp            : u16  , 
  pub event_id             : u32  , 
  pub rb_id                : u8   , 
  pub timestamp_48         : u64  , 
  pub broken               : bool , 
}

impl RBEventHeader {
  const HEAD : u16 = 0xAAAA;
  const TAIL : u16 = 0x5555;
  const SIZE : usize = 34; // size in bytes with HEAD and TAIL

  pub fn new() -> RBEventHeader {
    RBEventHeader {
      channel_mask        : 0 ,  
      stop_cell           : 0 ,  
      crc32               : 0 ,  
      dtap0               : 0 ,  
      drs4_temp           : 0 ,  
      is_locked           : false,  
      is_locked_last_sec  : false,  
      lost_trigger        : false,  
      fpga_temp           : 0,  
      event_id            : 0,  
      rb_id               : 0,  
      timestamp_48        : 0,  
      broken              : false,  
    }
  }

  pub fn extract_eventid_from_rbheader(stream :&Vec<u8>) -> u32 {
    // event id is 18 bytes in (including HEAD bytes)
    let event_id = parse_u32(stream, &mut 18);
    event_id
  }

  pub fn extract_from_rbbinarydump(stream : &Vec<u8>, pos : &mut usize) 
    -> Result<RBEventHeader, SerializationError> {
    let start = *pos;
    let mut header = RBEventHeader::new();
    let head_pos   = search_for_u16(RBBinaryDump::HEAD, stream, *pos)?; 
    let tail_pos   = search_for_u16(RBBinaryDump::TAIL, stream, head_pos + RBBinaryDump::SIZE -2)?;
    // At this state, this can be a header or a full event. Check here and
    // proceed depending on the options
    *pos = head_pos + 2;    
    let status          = parse_u16(stream, pos);
    let event_fragment  = (status & 1) == 1;
    header.lost_trigger = (status & 2) == 2;
    header.is_locked    = (status & 4) == 4;
    header.is_locked_last_sec = (status & 8) == 8;
    header.fpga_temp    = (status >> 4);
    if !header.lost_trigger {
      // in case there is no trigger, that means the DRS was busy so 
      // we won't get channel data or a stop cell
      if tail_pos + 2 - head_pos != RBBinaryDump::SIZE {
        error!("Size of {} not expected for RBBinaryDump!", tail_pos + 2 - head_pos);
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
    header.rb_id        = stream[*pos];
    *pos += 1;
    header.channel_mask = stream[*pos];
    *pos += 2;
    header.event_id  = parse_u32_for_16bit_words(stream, pos);
    header.dtap0     = parse_u16(stream, pos);
    header.drs4_temp = parse_u16(stream, pos); 
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

  pub fn get_active_data_channels(&self) -> Vec<u8> {
    let mut active_channels = Vec::<u8>::with_capacity(8);
    for ch in 1..9 {
      if ((self.channel_mask & (ch as u8 -1).pow(2)) == (ch as u8 -1).pow(2)) {
        active_channels.push(ch);
      }
    }
    active_channels
  }
  
  pub fn get_clock_cycles_48bit(&self) -> u64 {
    self.timestamp_48
  }
  
  pub fn get_n_datachan(&self) -> u8 {
    self.get_active_data_channels().len() as u8
  }
  
  pub fn get_fpga_temp(&self) -> f32 {
    self.drs_adc_to_celsius(self.fpga_temp)
  }
  
  pub fn get_drs_temp(&self) -> f32 {
    let drs_temp : f32 = 0.0;
    drs_temp
  }
  
  fn drs_adc_to_celsius(&self,adc : u16) -> f32 {
    let temp : f32 = 0.0;
    temp
  }
}

impl Default for RBEventHeader {

  fn default() -> RBEventHeader {
    RBEventHeader::new()
  }
}

impl From<&Path> for RBEventHeader {
  fn from(path : &Path) -> RBEventHeader {
    let mut header =  RBEventHeader::new();
    let file = BufReader::new(File::open(path).expect("Unable to open file {}"));
    
    header
  }
}

impl fmt::Display for RBEventHeader {
  fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
    write!(f, "<RBEventHeader:\n
           \t RB {},\n
           \t ch mask {}, \n
           \t event id {}, \n
           \t timestamp (48bit) {}, \n,
           \t locked {}, \n
           \t locked last sec. {}, \n
           \t drs4 Temp [C] {}, \n
           \t FPGA Temp [C] {}, \n
           \t stop cell {}, \n,
           \t dtap0 {},\n,
           \t crc32 {},\n,
           \t broken {}>",
           self.rb_id,
           self.channel_mask,
           self.event_id,
           self.timestamp_48,
           self.is_locked,
           self.is_locked_last_sec,
           self.drs4_temp,
           self.fpga_temp,
           self.stop_cell,
           self.dtap0,
           self.crc32,
           self.broken)
  }
}

impl Serialization for RBEventHeader {

  fn from_bytestream(stream : &Vec<u8>, pos : &mut usize)
    -> Result<RBEventHeader, SerializationError> {
    let mut header  = RBEventHeader::new();
    let head_pos    = search_for_u16(RBBinaryDump::HEAD, stream, *pos)?; 
    let tail_pos    = search_for_u16(RBBinaryDump::TAIL, stream, head_pos + RBEventHeader::SIZE-2)?;
    // At this state, this can be a header or a full event. Check here and
    // proceed depending on the options
    if tail_pos + 2 - head_pos != RBEventHeader::SIZE {
      return Err(SerializationError::EventFragment);
    }
    *pos = head_pos + 2;  
    header.channel_mask        = parse_u8(stream  , pos);   
    header.stop_cell           = parse_u16(stream , pos);  
    header.crc32               = parse_u32(stream , pos);  
    header.dtap0               = parse_u16(stream , pos);  
    header.drs4_temp           = parse_u16(stream , pos);  
    header.is_locked           = parse_bool(stream, pos);
    header.is_locked_last_sec  = parse_bool(stream, pos);
    header.lost_trigger        = parse_bool(stream, pos);
    header.fpga_temp           = parse_u16(stream , pos);  
    header.event_id            = parse_u32(stream , pos);  
    header.rb_id               = parse_u8(stream  , pos);  
    header.timestamp_48        = parse_u64(stream , pos);  
    header.broken              = parse_bool(stream, pos);  
    Ok(header) 
  }

  fn to_bytestream(&self) -> Vec<u8> {
    let mut stream = Vec::<u8>::with_capacity(RBEventHeader::SIZE);
    stream.extend_from_slice(&RBEventHeader::HEAD.to_le_bytes());
    stream.extend_from_slice(&self.channel_mask      .to_le_bytes());
    stream.extend_from_slice(&self.stop_cell         .to_le_bytes());
    stream.extend_from_slice(&self.crc32             .to_le_bytes());
    stream.extend_from_slice(&self.dtap0             .to_le_bytes());
    stream.extend_from_slice(&self.drs4_temp         .to_le_bytes());
    stream.extend_from_slice(&(u8::from(self.is_locked)  .to_le_bytes()));
    stream.extend_from_slice(&(u8::from(self.is_locked_last_sec).to_le_bytes()));
    stream.extend_from_slice(&(u8::from(self.lost_trigger)      .to_le_bytes()));
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
    
  fn from_random() -> RBEventHeader {
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
    header.fpga_temp            = rng.gen::<u16>();   
    header.event_id             = rng.gen::<u32>();   
    header.rb_id                = rng.gen::<u8>();    
    header.timestamp_48         = rng.gen::<u64>();   
    header.broken               = rng.gen::<bool>();  
    header
  }
}

