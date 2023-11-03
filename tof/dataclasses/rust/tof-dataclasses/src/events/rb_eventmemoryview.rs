use std::fmt;
use crate::serialization::{parse_u8,
                           parse_u16,
                           parse_u32,
                           parse_u32_for_16bit_words,
                           parse_u64,
                           search_for_u16,
                           Serialization,
                           SerializationError};

use crate::constants::{NCHN,
                       NWORDS};

cfg_if::cfg_if! {
  if #[cfg(feature = "random")]  {
    use crate::FromRandom;
    extern crate rand;
    use rand::Rng;
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

#[test]
fn serialization_rbmemoryview() {
  let head = RBEventMemoryView::from_random();
  let test = RBEventMemoryView::from_bytestream(&head.to_bytestream(), &mut 0).unwrap();
  assert_eq!(head, test);
}

