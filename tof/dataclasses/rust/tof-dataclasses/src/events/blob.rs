/***********************************
 * Readoutboard data, calibration and 
 * waveform analysis.
 *
 * Basically a translation of the 
 * tof library written by 
 * J. Zweerink
 *
 *
 ***********************************/

use std::fmt;

use crate::constants::{NWORDS, NCHN, MAX_NUM_PEAKS};
use crate::errors::{WaveformError, 
                    SerializationError};
use crate::serialization::search_for_u16;
use crate::calibrations::Calibrations;

pub fn get_constant_blobeventsize() -> usize {
  let size = 36 + (NCHN*2) + (NCHN*NWORDS*2) + (NCHN*4) + 8;
  size
}

// for diagnostics, we use hdf5 files
#[cfg(feature = "diagnostics")]
#[cfg(feature = "blosc")]
use hdf5::filters::blosc_set_nthreads;

#[cfg(feature = "diagnostics")]
use hdf5;

#[derive(Debug, Clone)]
pub struct RBEventPayload {
  pub event_id : u32,
  pub payload  : Vec<u8>
}

impl RBEventPayload {

  pub fn new(event_id : u32, payload : Vec<u8>) -> RBEventPayload {
    RBEventPayload {
      event_id,
      payload
    }
  }

  pub fn from_bytestream(bytestream  : &Vec<u8>,
                         start_pos   : usize,
                         no_fragment : bool)
      -> Result<RBEventPayload, SerializationError> {
    let head_pos = search_for_u16(BlobData::HEAD, bytestream, start_pos)?; 
    let tail_pos = search_for_u16(BlobData::TAIL, bytestream, head_pos)?;
    // At this state, this can be a header or a full event. Check here and
    // proceed depending on the options
    if head_pos - tail_pos != BlobData::SERIALIZED_SIZE
        && no_fragment { 
      return Err(SerializationError::EventFragment);
    }

    // we have to find and decode the event id.
    // FIXME - if we do this smarter, we can 
    //         most likely save a clone operation
    let slice          = &bytestream[head_pos..=tail_pos+2];
    let event_id       = BlobData::decode_event_id(slice); 
    let mut payload    = Vec::<u8>::with_capacity(BlobData::SERIALIZED_SIZE);
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
    let payload        = Vec::<u8>::with_capacity(BlobData::SERIALIZED_SIZE);
    if do_checks {
      let head_pos = search_for_u16(BlobData::HEAD, &payload, 000000000)?; 
      let tail_pos = search_for_u16(BlobData::TAIL, &payload, head_pos)?;
      // At this state, this can be a header or a full event. Check here and
      // proceed depending on the options
      if head_pos - tail_pos != BlobData::SERIALIZED_SIZE { 
        return Err(SerializationError::EventFragment);
      }
    }
    //payload.extend_from_slice(slice);
    let event_id       = BlobData::decode_event_id(slice);
    let ev_payload     = RBEventPayload::new(event_id, payload.clone()); 
    Ok(ev_payload)
  }
}

/***********************************/

#[derive(Debug, Clone)]
pub struct ReducedRBEvent {
  pub len             : u16,
  pub roi             : u16,
  pub event_id        : u32,
  pub timestamp       : u64,

  // these are NOT in the official blob format
  // these will NOT be able to be deserialized from
  // a standard readoutboard blob file
  pub voltages           : [[f64;NWORDS];NCHN],
  pub nanoseconds        : [[f64;NWORDS];NCHN],
  
  // these values are for baseline 
  // subtraction, cfd calculation etc.
  pub threshold      : [f64;NCHN],
  pub cfds_fraction  : [f64;NCHN],
  pub ped_begin_bin  : [usize;NCHN],
  pub ped_bin_range  : [usize;NCHN],    
  pub pedestal       : [f64;NCHN],
  pub pedestal_sigma : [f64;NCHN],

  // fields used for internal calculations
  pub peaks      : [[usize;MAX_NUM_PEAKS];NCHN],
  pub tdcs       : [[f64;MAX_NUM_PEAKS];NCHN],
  pub charge     : [[f64;MAX_NUM_PEAKS];NCHN],
  pub width      : [[f64;MAX_NUM_PEAKS];NCHN], 
  pub height     : [[f64;MAX_NUM_PEAKS];NCHN],    
  pub num_peaks  : [usize;NCHN],
  //pub stop_cell  : [u16;NCHN],
  pub begin_peak : [[usize;MAX_NUM_PEAKS];NCHN],
  pub end_peak   : [[usize;MAX_NUM_PEAKS];NCHN],
  pub spikes     : [[usize;MAX_NUM_PEAKS];NCHN],
  
  pub impedance  : f64,
}

/***********************************/

#[derive(Debug, Clone, Copy, PartialEq)]
#[cfg_attr(feature = "diagnostics", derive(hdf5::H5Type))]
#[cfg_attr(feature = "diagnostics", repr(C))] 
pub struct BlobData
{
  pub head            : u16, // Head of event marker
  pub status          : u16,
  pub len             : u16,
  pub roi             : u16,
  pub dna             : u64, 
  pub fw_hash         : u16,
  pub id              : u16,   
  pub ch_mask         : u16,
  pub event_id       : u32,
  pub dtap0           : u16,
  pub dtap1           : u16,
  pub timestamp_32    : u32,
  pub timestamp_16    : u16,
  pub ch_head         : [u16; NCHN],
  pub ch_adc          : [[i16; NWORDS];NCHN], 
  pub ch_trail        : [u32; NCHN],
  pub stop_cell       : u16,
  pub crc32           : u32,
  pub tail            : u16, // End of event marker

  // these are NOT in the official blob format
  // these will NOT be able to be deserialized from
  // a standard readoutboard blob file
  pub voltages           : [[f64;NWORDS];NCHN],
  pub nanoseconds        : [[f64;NWORDS];NCHN],
  
  // these values are for baseline 
  // subtraction, cfd calculation etc.
  pub threshold      : [f64;NCHN],
  pub cfds_fraction  : [f64;NCHN],
  pub ped_begin_bin  : [usize;NCHN],
  pub ped_bin_range  : [usize;NCHN],    
  pub pedestal       : [f64;NCHN],
  pub pedestal_sigma : [f64;NCHN],

  // fields used for internal calculations
  pub peaks      : [[usize;MAX_NUM_PEAKS];NCHN],
  pub tdcs       : [[f64;MAX_NUM_PEAKS];NCHN],
  pub charge     : [[f64;MAX_NUM_PEAKS];NCHN],
  pub width      : [[f64;MAX_NUM_PEAKS];NCHN], 
  pub height     : [[f64;MAX_NUM_PEAKS];NCHN],    
  pub num_peaks  : [usize;NCHN],
  //pub stop_cell  : [u16;NCHN],
  pub begin_peak : [[usize;MAX_NUM_PEAKS];NCHN],
  pub end_peak   : [[usize;MAX_NUM_PEAKS];NCHN],
  pub spikes     : [[usize;MAX_NUM_PEAKS];NCHN],
  
  pub impedance  : f64,

} 

impl fmt::Display for BlobData {
  fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
    write!(f, "<BlobData: RB {}, event id {}>", self.id, self.event_id)
  }
}

impl BlobData {

  ///
  ///FIXME - more correctly, this should be 
  ///"deserialize from readoutboard_blob
  ///
  // the size currently a blob occupies
  // in memory or disk in serialized form
  pub const SERIALIZED_SIZE : usize = 18530; 
  pub const HEAD            : u16   = 0xAAAA; // 43690 
  pub const TAIL            : u16   = 0x5555; // 21845 
  pub fn new() -> BlobData {
    BlobData {
      head            : 0, // Head of event marker
      status          : 0,
      len             : 0,
      roi             : 0,
      dna             : 0,
      fw_hash         : 0,
      id              : 0,
      ch_mask         : 0,
      event_id       : 0,
      dtap0           : 0,
      dtap1           : 0,
      timestamp_32    : 0,
      timestamp_16    : 0,
      ch_head         : [0; NCHN],
      ch_adc          : [[0; NWORDS]; NCHN],
      ch_trail        : [0; NCHN],
      stop_cell       : 0,
      crc32           : 0,
      tail            : 0, // End of event marker

      voltages        : [[0.0; NWORDS]; NCHN],
      nanoseconds     : [[0.0; NWORDS]; NCHN],

      threshold      : [0.0;NCHN],
      cfds_fraction  : [0.0;NCHN],
      ped_begin_bin  : [0;NCHN],
      ped_bin_range  : [0;NCHN],
      pedestal       : [0.0;NCHN],
      pedestal_sigma : [0.0;NCHN],

      peaks      : [[0;MAX_NUM_PEAKS];NCHN],
      tdcs       : [[0.0;MAX_NUM_PEAKS];NCHN],
      charge     : [[0.0;MAX_NUM_PEAKS];NCHN],
      width      : [[0.0;MAX_NUM_PEAKS];NCHN],
      height     : [[0.0;MAX_NUM_PEAKS];NCHN],
      num_peaks  : [0;NCHN],
      //stop_cell  : [u16;NCHN],
      begin_peak : [[0;MAX_NUM_PEAKS];NCHN],
      end_peak   : [[0;MAX_NUM_PEAKS];NCHN],
      spikes     : [[0;MAX_NUM_PEAKS];NCHN],
      impedance  : 50.0,
    }
  }

  ///! Only decode the event id from a bytestream
  ///  
  ///  The bytestream has to be starting with 
  ///  HEAD
  pub fn decode_event_id(bytestream : &[u8]) -> u32 {
    let evid_pos = 22; // the eventid is 22 bytes from the 
                       // start including HEAD
    let raw_bytes_4  = [bytestream[evid_pos + 1],
                        bytestream[evid_pos    ],
                        bytestream[evid_pos + 3],
                        bytestream[evid_pos + 2]];
    
    u32::from_be_bytes(raw_bytes_4) 
  }

  ///! EXPERIMENTAL Initialize the blob from a bytestream. 
  ///
  ///  This is a member here, so this can be done
  ///  repeatedly without re-allocation of the 
  ///  blob arrays.
  ///
  ///  THIS VERSION IS DEDICATED TO OPERATE ON THE RB!
  ///
  ///  # Arguments
  ///
  ///  * search_start : automatically look ahead 
  ///                   from start_pos unitl HEAD
  ///                   is found
  ///
  ///FIXME - this should return Result!                   
  pub fn from_bytestream_experimental(&mut self,
                                      bytestream   : &Vec<u8>,
                                      start_pos    : usize,
                                      search_start : bool) -> usize {
    let mut pos = start_pos;
    
    if search_start {
      match search_for_u16(BlobData::HEAD, bytestream, start_pos) {
        Err(err) => warn!("Can not find blob in selected range from {start_pos} to end! Error {:?}", err),
        Ok(n)    => {pos = n;}
      }
    }

    let mut raw_bytes_2  = [bytestream[pos],bytestream[pos + 1]];
    pos   += 2;
    self.head    = u16::from_le_bytes(raw_bytes_2);
    
    raw_bytes_2  = [bytestream[pos],bytestream[pos + 1]];
    pos   += 2;
    self.status  = u16::from_le_bytes(raw_bytes_2); 
    
    raw_bytes_2  = [bytestream[pos],bytestream[pos + 1]];
    pos   += 2;
    self.len     = u16::from_le_bytes(raw_bytes_2); 
    
    raw_bytes_2  = [bytestream[pos],bytestream[pos + 1]];
    pos   += 2;
    self.roi     = u16::from_le_bytes(raw_bytes_2); 

    let mut raw_bytes_8  = [bytestream[pos    ],
                            bytestream[pos + 1],
                            bytestream[pos + 2],
                            bytestream[pos + 3],
                            bytestream[pos + 4],
                            bytestream[pos + 5],
                            bytestream[pos + 6],
                            bytestream[pos + 7]];
    pos   += 8;
    self.dna     = u64::from_le_bytes(raw_bytes_8);

    raw_bytes_2  = [bytestream[pos],bytestream[pos + 1]];
    pos   += 2;
    self.fw_hash = u16::from_le_bytes(raw_bytes_2); 
    
    raw_bytes_2  = [bytestream[pos],bytestream[pos + 1]];
    pos   += 2;
    self.id      = u16::from_le_bytes(raw_bytes_2);    
    
    raw_bytes_2  = [bytestream[pos],bytestream[pos + 1]];
    pos   += 2;
    self.ch_mask = u16::from_le_bytes(raw_bytes_2); 
   
    let mut raw_bytes_4  = [bytestream[pos    ],
                            bytestream[pos + 1],
                            bytestream[pos + 2],
                            bytestream[pos + 3]];
    pos   += 4; 
    self.event_id = u32::from_le_bytes(raw_bytes_4); 


    raw_bytes_2  = [bytestream[pos],bytestream[pos + 1]];
    pos   += 2;
    self.dtap0   = u16::from_le_bytes(raw_bytes_2); 

    raw_bytes_2  = [bytestream[pos],bytestream[pos + 1]];
    pos   += 2;
    self.dtap1   = u16::from_le_bytes(raw_bytes_2); 
    
    //raw_bytes_8  = [bytestream[pos    ],
    //                bytestream[pos + 1],
    //                bytestream[pos + 2],
    //                bytestream[pos + 3],
    //                bytestream[pos + 4],
    //                bytestream[pos + 5],
    //                bytestream[pos + 6],
    //                bytestream[pos + 7]];
    //pos += 8;
    //self.timestamp  = u64::from_le_bytes(raw_bytes_8); 
    raw_bytes_4 = [bytestream[pos],
                   bytestream[pos + 1],
                   bytestream[pos + 2],
                   bytestream[pos + 3]];
    self.timestamp_32 = u32::from_le_bytes(raw_bytes_4);
    pos += 4;
    raw_bytes_2 = [bytestream[pos],
                   bytestream[pos + 1]];
    self.timestamp_16 = u16::from_le_bytes(raw_bytes_2);
    pos += 2;


    for n in 0..NCHN {
      raw_bytes_2  = [bytestream[pos],bytestream[pos + 1]];
      self.ch_head[n] = u16::from_le_bytes(raw_bytes_2);
      pos   += 2;
      for k in 0..NWORDS {
        raw_bytes_2  = [bytestream[pos],bytestream[pos + 1]];
        self.ch_adc[n][k] = i16::from_le_bytes(raw_bytes_2);
        pos += 2;
      }
      raw_bytes_4  = [bytestream[pos    ],
                      bytestream[pos + 1],
                      bytestream[pos + 2],
                      bytestream[pos + 3]];
      pos   += 4; 
      self.ch_trail[n] = u32::from_le_bytes(raw_bytes_4); 
    } // end nchn loop

    raw_bytes_2  = [bytestream[pos+0],bytestream[pos + 1]];
    pos   += 2;
    self.stop_cell       = u16::from_le_bytes(raw_bytes_2); 
    raw_bytes_4  = [bytestream[pos    ],
                    bytestream[pos + 1],
                    bytestream[pos + 2],
                    bytestream[pos + 3]];
    pos   += 4; 
    self.crc32   = u32::from_le_bytes(raw_bytes_4); 

    raw_bytes_2  = [bytestream[pos],bytestream[pos + 1]];
    pos   += 2;
    self.tail    = u16::from_le_bytes(raw_bytes_2);  // End of event marker
    pos
  }

  ///! Initialize the blob from a bytestream. 
  ///
  ///  This is a member here, so this can be done
  ///  repeatedly without re-allocation of the 
  ///  blob arrays
  ///
  ///  # Arguments
  ///
  ///  * search_start : automatically look ahead 
  ///                   from start_pos unitl HEAD
  ///                   is found
  ///
  ///FIXME - this should return Result!                   
  pub fn from_bytestream(&mut self,
                         bytestream   : &Vec<u8>,
                         start_pos    : usize,
                         search_start : bool) -> usize {
    let mut pos = start_pos;
    
    if search_start {
      match search_for_u16(BlobData::HEAD, bytestream, start_pos) {
        Err(err) => warn!("Can not find blob in selected range from {start_pos} to end! Error {:?}", err),
        Ok(n)    => {pos = n;}
      }
    }

    let mut raw_bytes_2  = [bytestream[pos],bytestream[pos + 1]];
    pos   += 2;
    self.head    = u16::from_le_bytes(raw_bytes_2);
    
    raw_bytes_2  = [bytestream[pos],bytestream[pos + 1]];
    pos   += 2;
    self.status  = u16::from_le_bytes(raw_bytes_2); 
    
    raw_bytes_2  = [bytestream[pos],bytestream[pos + 1]];
    pos   += 2;
    self.len     = u16::from_le_bytes(raw_bytes_2); 
    
    raw_bytes_2  = [bytestream[pos],bytestream[pos + 1]];
    pos   += 2;
    self.roi     = u16::from_le_bytes(raw_bytes_2); 

    // these two should ultimatly be the same
    let mut raw_bytes_8  = [bytestream[pos + 6],
                            bytestream[pos + 7],
                            bytestream[pos + 4],
                            bytestream[pos + 5],
                            bytestream[pos + 2],
                            bytestream[pos + 3],
                            bytestream[pos    ],
                            bytestream[pos + 1]];
    pos   += 8;
    self.dna     = u64::from_le_bytes(raw_bytes_8);
    
    //let mut raw_bytes_8  = [bytestream[pos + 1],
    //                        bytestream[pos + 0],
    //                        bytestream[pos + 3],
    //                        bytestream[pos + 2],
    //                        bytestream[pos + 5],
    //                        bytestream[pos + 4],
    //                        bytestream[pos + 7],
    //                        bytestream[pos + 6]];
    //pos   += 8;
    //self.dna     = u64::from_be_bytes(raw_bytes_8);

    raw_bytes_2  = [bytestream[pos],bytestream[pos + 1]];
    pos   += 2;
    self.fw_hash = u16::from_le_bytes(raw_bytes_2); 
    
    raw_bytes_2  = [bytestream[pos],bytestream[pos + 1]];
    pos   += 2;
    self.id      = u16::from_le_bytes(raw_bytes_2);    
    
    raw_bytes_2  = [bytestream[pos],bytestream[pos + 1]];
    pos   += 2;
    self.ch_mask = u16::from_le_bytes(raw_bytes_2); 
   
    let mut raw_bytes_4  = [bytestream[pos + 1],
                            bytestream[pos    ],
                            bytestream[pos + 3],
                            bytestream[pos + 2]];
    pos   += 4; 
    self.event_id = u32::from_be_bytes(raw_bytes_4); 


    raw_bytes_2  = [bytestream[pos],bytestream[pos + 1]];
    pos   += 2;
    self.dtap0   = u16::from_le_bytes(raw_bytes_2); 

    raw_bytes_2  = [bytestream[pos],bytestream[pos + 1]];
    pos   += 2;
    self.dtap1   = u16::from_le_bytes(raw_bytes_2); 
    
    //raw_bytes_8  = [0,0,bytestream[pos+1],
    //                    bytestream[pos    ],
    //                    bytestream[pos + 3],
    //                    bytestream[pos + 2],
    //                    bytestream[pos + 5],
    //                    bytestream[pos + 4]];
    //pos += 6;
    //self.timestamp  = u64::from_be_bytes(raw_bytes_8); 
    raw_bytes_4 =  [bytestream[pos],
                    bytestream[pos + 1],
                    bytestream[pos + 2],
                    bytestream[pos + 3]];
    self.timestamp_32 = u32::from_le_bytes(raw_bytes_4);
    pos += 4;
    raw_bytes_2 =  [bytestream[pos],
                    bytestream[pos + 1]];
    self.timestamp_16 = u16::from_le_bytes(raw_bytes_2);
    pos += 2;
    for n in 0..NCHN {
      raw_bytes_2  = [bytestream[pos],bytestream[pos + 1]];
      self.ch_head[n] = u16::from_le_bytes(raw_bytes_2);
      pos   += 2;
      for k in 0..NWORDS {
        raw_bytes_2  = [bytestream[pos],bytestream[pos + 1]];
        self.ch_adc[n][k] = i16::from_le_bytes(raw_bytes_2);
        pos += 2;
      }
      raw_bytes_4  = [bytestream[pos + 1],
                      bytestream[pos    ],
                      bytestream[pos + 3],
                      bytestream[pos + 2]];
      pos   += 4; 
      self.ch_trail[n] = u32::from_be_bytes(raw_bytes_4); 
    }

    raw_bytes_2  = [bytestream[pos],bytestream[pos + 1]];
    pos   += 2;
    self.stop_cell       = u16::from_le_bytes(raw_bytes_2); 
    raw_bytes_4  = [bytestream[pos + 1],
                    bytestream[pos    ],
                    bytestream[pos + 3],
                    bytestream[pos + 2]];
    pos   += 4; 
    self.crc32   = u32::from_be_bytes(raw_bytes_4); 

    raw_bytes_2  = [bytestream[pos],bytestream[pos + 1]];
    pos   += 2;
    self.tail    = u16::from_le_bytes(raw_bytes_2);  // End of event marker
    
    pos
  }


  ///! Serialize the event to a bytestream
  pub fn to_bytestream(&self) ->Vec<u8> {
    let mut bytestream : Vec<u8> = Vec::<u8>::new();
    
    // containers for the individual byte words
    let mut two_bytes           : [u8;2];
    let mut four_bytes          : [u8;4];
    let mut four_bytes_shuffle  : [u8;4]; 
    let mut eight_bytes         : [u8;8];
    
    // begin serialization
    two_bytes = self.head.to_le_bytes();
    bytestream.extend_from_slice(&two_bytes);
    
    two_bytes = self.status.to_le_bytes();
    bytestream.extend_from_slice(&two_bytes);
    
    two_bytes = self.len.to_le_bytes();
    bytestream.extend_from_slice(&two_bytes);
    
    two_bytes = self.roi.to_le_bytes();
    bytestream.extend_from_slice(&two_bytes);

    eight_bytes = self.dna.to_be_bytes();
    let eight_bytes_shuffle  = [eight_bytes[1],
                                eight_bytes[0],
                                eight_bytes[3],
                                eight_bytes[2],
                                eight_bytes[5],
                                eight_bytes[4],
                                eight_bytes[7],
                                eight_bytes[6]];
    bytestream.extend_from_slice(&eight_bytes_shuffle);

    two_bytes = self.fw_hash.to_le_bytes();
    bytestream.extend_from_slice(&two_bytes);
    
    two_bytes = self.id.to_le_bytes();   
    bytestream.extend_from_slice(&two_bytes);
    
    two_bytes = self.ch_mask.to_le_bytes();
    bytestream.extend_from_slice(&two_bytes);

    four_bytes = self.event_id.to_be_bytes();
    four_bytes_shuffle = [four_bytes[1],
                          four_bytes[0],
                          four_bytes[3],
                          four_bytes[2]];
    bytestream.extend_from_slice(&four_bytes_shuffle); 
 
    two_bytes = self.dtap0.to_le_bytes();
    bytestream.extend_from_slice(&two_bytes);

    two_bytes = self.dtap1.to_le_bytes();
    bytestream.extend_from_slice(&two_bytes);
   
    four_bytes = self.timestamp_32.to_le_bytes();
    bytestream.extend_from_slice(&four_bytes);

    two_bytes = self.timestamp_16.to_le_bytes();
    bytestream.extend_from_slice(&two_bytes);

    //eight_bytes          = self.timestamp.to_be_bytes();
    //let six_bytes_shuffle    = [eight_bytes[1 + 2],
    //                            eight_bytes[    2],
    //                            eight_bytes[3 + 2],
    //                            eight_bytes[2 + 2],
    //                            eight_bytes[5 + 2],
    //                            eight_bytes[4 + 2]];
    //bytestream.extend_from_slice(&six_bytes_shuffle);
    
    for n in 0..NCHN {
      two_bytes = self.ch_head[n].to_le_bytes();
      bytestream.extend_from_slice(&two_bytes);
      for k in 0..NWORDS {
        two_bytes = self.ch_adc[n][k].to_le_bytes();
        bytestream.extend_from_slice(&two_bytes);
      }
      four_bytes  = self.ch_trail[n].to_be_bytes();
      four_bytes_shuffle = [four_bytes[1],
                            four_bytes[0],
                            four_bytes[3],
                            four_bytes[2]];
      bytestream.extend_from_slice(&four_bytes_shuffle);
    }
    two_bytes = self.stop_cell.to_le_bytes();
    bytestream.extend_from_slice(&two_bytes);

    four_bytes = self.crc32.to_be_bytes();
    four_bytes_shuffle  = [four_bytes[1],
                           four_bytes[0],
                           four_bytes[3],
                           four_bytes[2]];
    
    bytestream.extend_from_slice(&four_bytes_shuffle);
    
    two_bytes = self.tail.to_le_bytes();
    bytestream.extend_from_slice(&two_bytes);

    bytestream
  }

  /// Apply the calibration for time
  /// and voltage.
  pub fn calibrate (&mut self, cal : &[Calibrations;NCHN]) {
    self.voltage_calibration(cal);
    self.timing_calibration (cal);
  }

  fn voltage_calibration(&mut self, cal : &[Calibrations;NCHN]) {
    let mut value : f64; 
    for n in 0..NCHN {
      for m in 0..NWORDS {
        value  = self.ch_adc[n][m] as f64;
        value -= cal[n].v_offsets[(m + (self.stop_cell as usize)) %NWORDS];
        value -= cal[n].v_dips[m];
        value *= cal[n].v_inc[(m + (self.stop_cell as usize)) %NWORDS];
        self.voltages[n][m] = value;
        }
      }
    }

  fn timing_calibration( &mut self, cal : &[Calibrations;NCHN]){
    for n in 0..NCHN {
      self.nanoseconds[n][0] = 0.0;
      for m in 1..NWORDS { 
        self.nanoseconds[n][m] = self.nanoseconds[n][m-1] + cal[n].tbin[(m-1+(self.stop_cell as usize))%NWORDS];
      }
    }
  }

  pub fn remove_spikes (&mut self,
                        spikes : &mut [i32;10]) {

  //let mut spikes  : [i32;10] = [0;10];
  let mut filter  : f64;
  let mut dfilter : f64;
  //let mut n_symmetric : usize;
  let mut n_neighbor  : usize;

  let mut n_rsp      = 0usize;

  let mut rsp : [i32;10]    = [-1;10];
  //let mut spikes : [i32;10] = [-1;10
  // to me, this seems that should be u32
  // the 10 is for a maximum of 10 spikes (Jeff)
  let mut sp   : [[usize;10];NCHN] = [[0;10];NCHN];
  let mut n_sp : [usize;10]      = [0;10];

  for j in 0..NWORDS as usize {
    for i in 0..NCHN as usize {
      filter = -self.voltages[i][j] + self.voltages[i][(j + 1) % NWORDS] + self.voltages[i][(j + 2) % NWORDS] - self.voltages[i][(j + 3) % NWORDS];
      dfilter = filter + 2.0 * self.voltages[i][(j + 3) % NWORDS] + self.voltages[i][(j + 4) % NWORDS] - self.voltages[i][(j + 5) % NWORDS];
      if filter > 20.0  && filter < 100.0 {
        if n_sp[i] < 10 {   // record maximum of 10 spikes
          sp[i][n_sp[i] as usize] = (j + 1) % NWORDS ;
          n_sp[i] += 1;
        // FIXME - error checking
        } else {return;}            // too many spikes -> something wrong
      }// end of if
      else if dfilter > 40.0 && dfilter < 100.0 && filter > 10.0 {
        if n_sp[i] < 9 {  // record maximum of 10 spikes
          sp[i][n_sp[i] as usize] = (j + 1) % NWORDS ;
          sp[i][(n_sp[i] + 1) as usize] = (j + 3) % NWORDS ;
          n_sp[i] += 2;
        } else { return;} // too many spikes -> something wrong
      } // end of else if

    }// end loop over NCHN
  } // end loop over NWORDS

  // go through all spikes and look for neighbors */
  for i in 0..NCHN {
    for j in 0..n_sp[i] as usize {
      //n_symmetric = 0;
      n_neighbor = 0;
      for k in 0..NCHN {
        for l in 0..n_sp[k] as usize {
        //check if this spike has a symmetric partner in any channel
          if (sp[i][j] as i32 + sp[k][l] as i32 - 2 * self.stop_cell as i32) as i32 % NWORDS as i32 == 1022 {
            //n_symmetric += 1;
            break;
          }
        }
      } // end loop over k
      // check if this spike has same spike is in any other channels */
      //for (k = 0; k < nChn; k++) {
      for k in 0..NCHN {
        if i != k {
          for l in 0..n_sp[k] {
            if sp[i][j] == sp[k][l] {
            n_neighbor += 1;
            break;
            }
          } // end loop over l   
        } // end if
      } // end loop over k

      if n_neighbor >= 2 {
        for k in 0..n_rsp {
          if rsp[k] == sp[i][j] as i32 {break;} // ignore repeats
          if n_rsp < 10 && k == n_rsp {
            rsp[n_rsp] = sp[i][j] as i32;
            n_rsp += 1;
          }
        }  
      }

    } // end loop over j
  } // end loop over i

  // recognize spikes if at least one channel has it */
  //for (k = 0; k < n_rsp; k++)
  let magic_value : f64 = 14.8;
  let mut x : f64;
  let mut y : f64;

  let mut skip_next : bool = false;
  for k in 0..n_rsp {
    if skip_next {
      skip_next = false;
      continue;
    }
    spikes[k] = rsp[k];
    //for (i = 0; i < nChn; i++)
    for i in 0..NCHN {
      if k < n_rsp && i32::abs(rsp[k] as i32 - rsp[k + 1] as i32 % NWORDS as i32) == 2
      {
        // remove double spike 
        let j = if rsp[k] > rsp[k + 1] {rsp[k + 1] as usize}  else {rsp[k] as usize};
        x = self.voltages[i][(j - 1) % NWORDS];
        y = self.voltages[i][(j + 4) % NWORDS];
        if f64::abs(x - y) < 15.0
        {
          self.voltages[i][j % NWORDS] = x + 1.0 * (y - x) / 5.0;
          self.voltages[i][(j + 1) % NWORDS] = x + 2.0 * (y - x) / 5.0;
          self.voltages[i][(j + 2) % NWORDS] = x + 3.0 * (y - x) / 5.0;
          self.voltages[i][(j + 3) % NWORDS] = x + 4.0 * (y - x) / 5.0;
        }
        else
        {
          self.voltages[i][j % NWORDS] -= magic_value;
          self.voltages[i][(j + 1) % NWORDS] -= magic_value;
          self.voltages[i][(j + 2) % NWORDS] -= magic_value;
          self.voltages[i][(j + 3) % NWORDS] -= magic_value;
        }
      }
      else
      {
        // remove single spike 
        x = self.voltages[i][((rsp[k] - 1) % NWORDS as i32) as usize];
        y = self.voltages[i][(rsp[k] + 2) as usize % NWORDS];
        if f64::abs(x - y) < 15.0 {
          self.voltages[i][rsp[k] as usize] = x + 1.0 * (y - x) / 3.0;
          self.voltages[i][(rsp[k] + 1) as usize % NWORDS] = x + 2.0 * (y - x) / 3.0;
        }
        else
        {
          self.voltages[i][rsp[k] as usize] -= magic_value;
          self.voltages[i][(rsp[k] + 1) as usize % NWORDS] -= magic_value;
        }
      } // end loop over nchn
    } // end loop over n_rsp
    if k < n_rsp && i32::abs(rsp[k] - rsp[k + 1] % NWORDS as i32) == 2
      {skip_next = true;} // skip second half of double spike
    } // end loop over k
  }
  
  ///! Set the threshold and check if the waveform got over threahshold
  pub fn set_threshold(&mut self, thr : f64, ch : usize) -> bool {
    self.threshold[ch] = thr;
    for n in 0..NWORDS {
      if self.voltages[ch][n] > thr {
        return true;
      }
    }
    false
  }

  pub fn set_cfds_fraction(&mut self, fraction : f64, ch : usize) {
      self.cfds_fraction[ch] = fraction;
  }
  
  pub fn set_ped_begin(&mut self, time : f64, ch : usize) {
      match self.time_2_bin(time, ch) {
          Err(err) => println!("Can not find bin for time {}, ch {}, err {:?}", time, ch, err),
          Ok(begin) => {self.ped_begin_bin[ch] = begin;}
      }
  }

  pub fn set_ped_range(&mut self, range : f64, ch : usize) {
    // This is a little convoluted, but we must convert the range (in
    // ns) into bins
    match self.time_2_bin(self.nanoseconds[ch][self.ped_begin_bin[ch]] + range, ch) {
        Err(err)      => println!("Can not set pedestal range for range {} for ch {}, err {:?}", range, ch, err),
        Ok(bin_range) => {self.ped_bin_range[ch] = bin_range;}
    }
  }

  pub fn subtract_pedestal(&mut self, ch : usize) {
    for n in 0..NWORDS {
      self.voltages[ch][n] -= self.pedestal[ch];
    }
  }

  pub fn calc_ped_range(&mut self, ch : usize) {
    let mut sum  = 0f64;
    let mut sum2 = 0f64;

    for n in self.ped_begin_bin[ch]..self.ped_begin_bin[ch] + self.ped_bin_range[ch] {
      if f64::abs(self.voltages[ch][n]) < 10.0 {
        sum  += self.voltages[ch][n];
        sum2 += self.voltages[ch][n]*self.voltages[ch][n];
      }
    }
    let average = sum/(self.ped_bin_range[ch] as f64);
    self.pedestal[ch] = average;
    self.pedestal_sigma[ch] = f64::sqrt(sum2/(self.ped_bin_range[ch] as f64 - (average*average)))

  }

  fn time_2_bin(&self, t_ns : f64, ch : usize) -> Result<usize, WaveformError> {
    // Given a time in ns, find the bin most closely corresponding to that time
    for n in 0..NWORDS {
      if self.nanoseconds[ch][n] > t_ns {
        return Ok(n-1);
      }
    }
    println!("Did not find a bin corresponding to the given time {} for ch {}", t_ns, ch);
    return Err(WaveformError::TimesTooSmall);
  }


  // 
  // Return the bin with the maximum DC value
  //
  fn get_max_bin(&self,
                 lower_bound : usize,
                 window : usize,
                 ch : usize ) -> Result<usize, WaveformError> {
    if lower_bound + window > NWORDS {
      return Err(WaveformError::OutOfRangeUpperBound);
    }
    let mut maxval = self.voltages[ch][lower_bound];
    let mut maxbin = lower_bound;
    for n in lower_bound..lower_bound + window {
      if self.voltages[ch][n] > maxval {
        maxval  = self.voltages[ch][n];
        maxbin  = n;
      }
    } // end for
    trace!("Got maxbin {} with a value of {}", maxbin, maxval);
    Ok(maxbin)
  } // end fn

  pub fn find_cfd_simple(&mut self, peak_num : usize, ch : usize) -> f64 {
    if self.num_peaks[ch] == 0 {
      trace!("No peaks for ch {}!", ch);
      return 0.0;
    }
    if peak_num > self.num_peaks[ch] {
      warn!("Requested peak {} is larger than detected peaks (={})", peak_num, self.num_peaks[ch]); 
      return self.nanoseconds[ch][NWORDS];
    }
    // FIXME
    // FIXME - this needs some serious error checking
    if self.end_peak[ch][peak_num] < self.begin_peak[ch][peak_num] {
        debug!("cfd simple method failed!! Peak begin {}, peak end {}", self.begin_peak[ch][peak_num], self.end_peak[ch][peak_num]);
        return 0.0;
    }
    let mut idx : usize;
    match self.get_max_bin(self.begin_peak[ch][peak_num],
                           self.end_peak[ch][peak_num]-self.begin_peak[ch][peak_num],
                           ch) {
      Err(err) => {warn!("Can not find cfd due to err {:?}",err);
                   return 0.0;
      }
      Ok(maxbin)  => {trace!("Got bin {} for max val", maxbin);
                      idx = maxbin;
      }

    }
    trace!("Got max bin {} for peak {} .. {}", idx, self.begin_peak[ch][peak_num], self.end_peak[ch][peak_num]);
    trace!("Voltage at max {}", self.voltages[ch][idx]);
    if idx < 1 {idx = 1;}
    let mut sum : f64 = 0.0;
    for n in idx-1..idx+1 {sum += self.voltages[ch][n];}
    let cfds_frac  : f64 = 0.2;
    let tmp_thresh : f64 = f64::abs(cfds_frac * (sum / 3.0));
    trace!("Calculated tmp threshold of {}", tmp_thresh);
    // Now scan through the waveform around the peak to find the bin
    // crossing the calculated threshold. Bin idx is the peak so it is
    // definitely above threshold. So let's walk backwards through the
    // trace until we find a bin value less than the threshold.
    let mut lo_bin : usize = NWORDS;
    let mut n = idx;
    assert!(idx >= self.begin_peak[ch][peak_num]);
    if self.begin_peak[ch][peak_num] >= 10 {
      while n > self.begin_peak[ch][peak_num] - 10 {
      //for n in (idx..self.begin_peak[peak_num] - 10).rev() {
        if f64::abs(self.voltages[ch][n]) < tmp_thresh {
          lo_bin = n;
          break;
        }
        n -= 1;
      }  
    }
    trace!("Lo bin {} , begin peak {}", lo_bin, self.begin_peak[ch][peak_num]);
    let cfd_time : f64;
    if lo_bin < NWORDS -1 {
      cfd_time = self.find_interpolated_time(tmp_thresh, lo_bin, 1, ch).unwrap();  
    }
    else {cfd_time = self.nanoseconds[ch][NWORDS - 1];} 

    // save it in member variable
    self.tdcs[ch][peak_num] = cfd_time;
    return cfd_time;
  }

  pub fn find_interpolated_time (&self,
                                 //adc       : [f64;NWORDS],
                                 //times     : [f64;NWORDS], 
                                 mut threshold : f64,
                                 mut idx       : usize,
                                 size          : usize, 
                                 ch            : usize) -> Result<f64, WaveformError> {
    if idx + 1 > NWORDS {
      return Err(WaveformError::OutOfRangeUpperBound);
    }
    
    threshold = threshold.abs();
    let mut lval  = (self.voltages[ch][idx]).abs();
    let mut hval : f64 = 0.0; 
    if size == 1 {
      hval = (self.voltages[ch][idx+1]).abs();
    } else {
    for n in idx+1..idx+size {
      hval = self.voltages[ch][n].abs();
      if (hval>=threshold) && (threshold<=lval) { // Threshold crossing?
        idx = n-1; // Reset idx to point before crossing
        break;
        }
      lval = hval;
      }
    }
    if ( lval > threshold) && (size != 1) {
      return Ok(self.nanoseconds[ch][idx]);
    } else if lval == hval {
      return Ok(self.nanoseconds[ch][idx]);
    } else {
      return Ok(self.nanoseconds[ch][idx] 
            + (threshold-lval)/(hval-lval) * (self.nanoseconds[ch][idx+1]
            - self.nanoseconds[ch][idx]));
  //float time = WaveTime[idx] +  
  //  (thresh-lval)/(hval-lval) * (WaveTime[idx+1]-WaveTime[idx]) ;
    }
  }


  ///
  /// Find peaks in a given time window (in ns) by 
  /// comparing the waveform voltages with the 
  /// given threshold. 
  /// Minimum peak width is currently hardcoded to 
  /// be 3 bins in time.
  ///
  ///
  pub fn find_peaks(&mut self,
                    start_time  : f64,
                    window_size : f64,
                    ch          : usize) {
    // FIXME - replace unwrap calls
    let start_bin  = self.time_2_bin(start_time, ch).unwrap();
    let window_bin = self.time_2_bin(start_time + window_size, ch).unwrap() - start_bin;
    // minimum number of bins a peak must have
    // over threshold so that we consider it 
    // a peak
    let min_peak_width       = 3usize; 
    let mut pos              = 0usize;
    let mut peak_bins        = 0usize;
    let mut peak_ctr         = 0usize;
    while self.voltages[ch][pos] < self.threshold[ch]  {
      pos += 1;
      if pos == NWORDS {
        pos = NWORDS -1;
        break;
      }
    }
    //narn!("{} {}", pos, self.voltages[ch][pos]);
    for n in pos..(start_bin + window_bin) {
      if self.voltages[ch][n] > self.threshold[ch] {
        //warn!("{} {}", peak_bins, window_bin);
        peak_bins += 1;
        if peak_bins == min_peak_width {
          // we have a new peak
          if peak_ctr == MAX_NUM_PEAKS -1 {
            debug!("Max number of peaks reached in this waveform");
            break;
          }
          self.begin_peak[ch][peak_ctr] = n - (min_peak_width - 1); 
          self.spikes    [ch][peak_ctr] = 0;
          self.end_peak  [ch][peak_ctr] = 0;
          peak_ctr += 1;
        } else if peak_bins > min_peak_width {
          let mut grad = 1;
          for k in 0..3 {
            if self.voltages[ch][n-k] > self.voltages[ch][n-(k+1)]
              {grad = 0;}
          }
          if grad == 0 {continue;}
          if self.end_peak[ch][peak_ctr-1] == 0 {
            self.end_peak[ch][peak_ctr-1] = n; // Set last bin included in peak
          }
        }
      } else {
          // this is for the case when the 
          // voltage is NOT over threshold
          peak_bins = 0;
      } 
    } // end for loop

    
    //for pos in start_bin..NWORDS {
    //  if (self.wave > threshold) {
    //  }
        
    //}

    //((self.wave[pos] < WF_VOLTAGE_THRESHOLD) && (pos < wf_size))
    self.num_peaks[ch] = peak_ctr;
    // some debugging information
    trace!("{} peaks found for ch {} -- ", peak_ctr, ch);
    for n in 0..peak_ctr {
      trace!("Found peak {} : {}.. {} ",n,  self.begin_peak[ch][n], self.end_peak[ch][n]);
    }
    if peak_ctr > 0 && self.end_peak[ch][peak_ctr-1] < self.begin_peak[ch][peak_ctr] {
          self.end_peak[ch][peak_ctr] = NWORDS; // Need this to measure last peak correctly
          trace!("Reset the last peak (={}) to  {}..{}", peak_ctr, self.begin_peak[ch][peak_ctr], self.end_peak[ch][peak_ctr] );
    }
    //peaks_found = 1;
  }

  pub fn integrate(&mut self, lower_bound : f64, size : f64, channel : usize) ->Result<f64, WaveformError>  {
    if lower_bound < 0.0 { 
        return Err(WaveformError::NegativeLowerBound);
    }


    let lo_bin   = self.time_2_bin(lower_bound, channel)?;
    let mut size_bin = self.time_2_bin(lower_bound + size, channel)?;
    size_bin = size_bin - lo_bin;
    if lo_bin + size_bin > NWORDS {
        warn!("Limiting integration range to waveform size!");
        size_bin = NWORDS - lo_bin;
    }
    let mut sum = 0f64;
    let upper_bin = lo_bin + size_bin;
    for n in lo_bin..upper_bin {
        sum += self.voltages[channel][n] * (self.nanoseconds[channel][n] - self.nanoseconds[channel][n-1]) ;
    }
    sum /= self.impedance;

    // FIXME - this is not how it is intended
    self.charge[channel][0] = sum;
    Ok(sum)
  }

  pub fn reset(&mut self) {
    self.head            =  0; // Head of event marker
    self.status          =  0;
    self.len             =  0;
    self.roi             =  0;
    self.dna             =  0; 
    self.fw_hash         =  0;
    self.id              =  0;   
    self.ch_mask         =  0;
    self.event_id       =  0;
    self.dtap0           =  0;
    self.dtap1           =  0;
    self.timestamp_32    =  0;
    self.timestamp_16    =  0;
    self.ch_head         =  [0; NCHN];
    self.ch_adc          =  [[0; NWORDS]; NCHN];
    self.ch_trail        =  [0; NCHN];
    self.stop_cell       =  0;
    self.crc32           =  0;
    self.tail            =  0; // End of event marker

    self.voltages        = [[0.0; NWORDS]; NCHN];
    self.nanoseconds     = [[0.0; NWORDS]; NCHN];
    
    self.threshold       = [0.0;NCHN];
    self.cfds_fraction   = [0.0;NCHN];
    self.ped_begin_bin   = [0;NCHN];
    self.ped_bin_range   = [0;NCHN];    
    self.pedestal        = [0.0;NCHN];
    self.pedestal_sigma  = [0.0;NCHN];
    
    self.peaks           = [[0;MAX_NUM_PEAKS];NCHN];
    self.tdcs            = [[0.0;MAX_NUM_PEAKS];NCHN];
    self.charge          = [[0.0;MAX_NUM_PEAKS];NCHN];
    self.width           = [[0.0;MAX_NUM_PEAKS];NCHN]; 
    self.height          = [[0.0;MAX_NUM_PEAKS];NCHN];    
    self.num_peaks       = [0;NCHN];
    self.begin_peak      = [[0;MAX_NUM_PEAKS];NCHN];
    self.end_peak        = [[0;MAX_NUM_PEAKS];NCHN];
    self.spikes          = [[0;MAX_NUM_PEAKS];NCHN];
    self.impedance       = 50.0;
  }


  pub fn print (&self) {
    println!("======");
    println!("==> HEAD       {} ", self.head);
    println!("==> STATUS     {} ", self.status);
    println!("==> LEN        {} ", self.len);
    println!("==> ROI        {} ", self.roi);
    println!("==> DNA        {} ", self.dna);
    println!("==> FW_HASH    {} ", self.fw_hash);
    println!("==> ID         {} ", self.id);
    println!("==> CH_MASK    {} ", self.ch_mask);
    println!("==> EVT_CTR    {} ", self.event_id);
    println!("==> DTAP0      {} ", self.dtap0);
    println!("==> DTAP1      {} ", self.dtap1);
    println!("==> TIMESTAMP32{} ", self.timestamp_32);
    println!("==> TIMESTAMP16{} ", self.timestamp_16);
    println!("==> STOP_CELL  {} ", self.stop_cell);
    println!("==> CRC32      {} ", self.crc32);
    println!("==> TAIL       {} ", self.tail);
    println!("======");
  }
}    

/***********************************/

impl Default for BlobData {
  fn default() -> BlobData {
    BlobData::new()
  }    
}

/***********************************/


#[cfg(test)]
mod test_readoutboard_blob {
  use crate::events::blob::BlobData;
  #[test]
  fn serialize_deserialize_roundabout () {
    let mut blob = BlobData {..Default::default()};
    blob.head = BlobData::HEAD;
    blob.status = 212;
    blob.len  = 42;
    blob.roi = 100;
    blob.dna = 4294967298;
    blob.fw_hash = 42;
    blob.id = 5;
    blob.ch_mask = 111;
    blob.event_id = 9800001;
    blob.dtap0 = 10000;
    blob.dtap1 = 11000;
    blob.timestamp_32 = 1123456;
    blob.stop_cell = 4;
    blob.crc32  = 88888;
    blob.tail   = 0x5555;
    blob.print();
    let bytestream = blob.to_bytestream();
    //for _n in 0..get_constant_blobeventsize() {
    //    bytestream.push(0);
    //}
    blob.from_bytestream(&bytestream, 0, true);
    let read_back_bytes = blob.to_bytestream();
    blob.print();

    assert_eq!(bytestream,read_back_bytes);
  }
}




