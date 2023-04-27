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
use crate::serialization::{parse_u16, parse_u32, parse_u64};
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
    stream.extend_from_slice(&self.event_id.to_le_bytes());
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
      return Err(SerializationError::EventFragment);
    }
    *pos = head_pos + 2; 
    bin_data.status         = parse_u16(&stream, pos);
    bin_data.len            = parse_u16(&stream, pos);
    bin_data.roi            = parse_u16(&stream, pos);
    bin_data.dna            = parse_u64(&stream, pos); 
    bin_data.fw_hash        = parse_u16(&stream, pos);
    bin_data.id             = parse_u16(&stream, pos);   
    bin_data.ch_mask        = parse_u16(&stream, pos);
    bin_data.event_id       = parse_u32(&stream, pos);
    bin_data.dtap0          = parse_u16(&stream, pos);
    bin_data.dtap1          = parse_u16(&stream, pos);
    bin_data.timestamp_32   = parse_u32(&stream, pos);
    bin_data.timestamp_16   = parse_u16(&stream, pos);
    for n in 0..NCHN {
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
    bin_data.id             =  rng.gen::<u16>();   
    bin_data.ch_mask        =  rng.gen::<u16>();
    bin_data.event_id       =  rng.gen::<u32>();
    bin_data.dtap0          =  rng.gen::<u16>();
    bin_data.dtap1          =  rng.gen::<u16>();
    bin_data.timestamp_32   =  rng.gen::<u32>();
    bin_data.timestamp_16   =  rng.gen::<u16>();
    for n in 0..NCHN {
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

pub struct RBEventHeader {
  pub nchannel     : u8,
  pub stop_cell    : u16,
  pub drs4_temp    : u16,
  pub is_locked    : bool,
  pub event_id     : u32,
  pub rb_id        : u8,
  pub timestamp_32 : u32,
  pub timestamp_16 : u16,
}

impl RBEventHeader {

  pub fn new() -> RBEventHeader {
    RBEventHeader {
      nchannel     : 0,
      stop_cell    : 0,
      drs4_temp    : 0,
      is_locked    : false,
      event_id     : 0,
      rb_id        : 0,
      timestamp_32 : 0,
      timestamp_16 : 0,
    }
  }

  pub fn get_timestamp48(&self) -> u64 {
    let mut ts_48 = 0u64;
    todo!();
    ts_48
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
           \t nchan {}, \n
           \t drs4 T {}, \n
           \t stop cell {}, \n
           \t locked {}, \n
           \t event id {}, \n
           \t timestamp (48bit) {}, \n",
           self.rb_id, self.nchannel, self.drs4_temp, self.stop_cell,
           self.is_locked,
           self.event_id, self.get_timestamp48())
  }
}
