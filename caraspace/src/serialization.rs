//! (De)serialization methods
//!
//!

use crate::errors::CRSerializationError;

/// Search for a u16 bytemarker in a stream.
///
/// E.g. This can be an 0xAAAA indicator as a packet delimiter
///
/// # Arguments:
///  
///  * marker     : The marker to search for. Currently, only 
///                 16bit markers are supported
///  * bytestream : The stream to search the number in
///  * start_pos  : Start searching from this position in 
///                 the stream
pub fn seek_marker<T: AsRef<[u8]>>(stream : &T, marker : u16, start_pos :usize) 
  -> Result<usize, CRSerializationError> {
  // -2 bc later on we are looking for 2 bytes!
  let bytestream = stream.as_ref();
  if bytestream.len() == 0 {
    error!("Stream empty!");
    return Err(CRSerializationError::StreamTooShort);
  }
  if start_pos  > bytestream.len() - 2 {
    error!("Start position {} beyond stream capacity {}!", start_pos, bytestream.len() -2);
    return Err(CRSerializationError::StreamTooShort);
  }
  let mut pos = start_pos;
  let mut two_bytes : [u8;2]; 
  // will find the next header
  two_bytes = [bytestream[pos], bytestream[pos + 1]];
  // FIXME - this should be little endian?
  if u16::from_le_bytes(two_bytes) == marker {
    return Ok(pos);
  }
  // if it is not at start pos, then traverse 
  // the stream
  pos += 1;
  let mut found = false;
  // we search for the next packet
  for n in pos..bytestream.len() - 1 {
    two_bytes = [bytestream[n], bytestream[n + 1]];
    if (u16::from_le_bytes(two_bytes)) == marker {
      pos = n;
      found = true;
      break;
    }
  }
  if !found {
    let delta = bytestream.len() - start_pos;
    warn!("Can not find {} in bytestream [-{}:{}]!", marker, delta ,bytestream.len());
    return Err(CRSerializationError::ValueNotFound);
  }
  trace!("Found {marker} at {pos}");
  Ok(pos)
}

///// Encode/decode structs to `Vec::<u8>` to write to a file or
///// send over the network
/////
pub trait CRSerializeable {

  const CRHEAD: u16 = 0xAAAA;
  const CRTAIL: u16 = 0x5555;
  /// The SIZE is the size of the serialized 
  /// bytestream INCLUDING 4 bytes for head
  /// and tail bytes. In case the struct does 
  /// NOT HAVE a fixed size, SIZE will be 0
  /// (so default value of the trait
  const CRSIZE: usize = 0;

  /// Verify that the serialized representation of the struct has the 
  /// correct size, including header + footer.
  ///
  /// Will panic for variable sized structs.
  fn verify_fixed(stream : &Vec<u8>, 
                  pos    : &mut usize) -> Result<(), CRSerializationError> {
    if !Self::CRSIZE == 0 {
      // we can panic here, since this is a conceptional logic error. If we
      // don't panic, monsters will arise downstream.
      panic!("Self::verify_fixed can be only used for structs with a fixed size! In case you are convinced, that your struct has indeed a fixed size, please implement trait Serialization::SIZE with the serialized size in bytes including 4 bytes for header and footer!");
    }
    let head_pos = seek_marker(stream, Self::CRHEAD, *pos)?; 
    let tail_pos = seek_marker(stream, Self::CRTAIL,  head_pos + Self::CRSIZE-2)?;
    // At this state, this can be a header or a full event. Check here and
    // proceed depending on the options
    if tail_pos + 2 - head_pos != Self::CRSIZE {
      error!("Seing {} bytes, but expecting {}", tail_pos + 2 - head_pos, Self::CRSIZE);
      *pos = head_pos + 2; 
      return Err(CRSerializationError::WrongByteSize);
    }
    *pos = head_pos + 2;
    Ok(())
  } 

  /// Decode a serializable from a bytestream  
  fn deserialize(bytestream : &Vec<u8>, 
                     pos        : &mut usize)
    -> Result<Self, CRSerializationError>
    where Self : Sized;
  
  /// Encode a serializable to a bytestream  
  fn serialize(&self) -> Vec<u8>;

  ///// Decode a serializable directly from a TofPacket
  //fn from_sclerite(sclerite : &Sclerite, name : String)
  //  -> Result<Self, CRSerializationError>
  //  where Self: Sized {
  //  let s_object = sclerite.get(name)?;
  //  let unpacked = Self::deserialize(&sclerite.payload, &mut 0)?;
  //  Ok(unpacked)
  //}
//
//
//  fn from_slice(_slice     : &[u8],
//                _start_pos : usize)
//    -> Result<Self, CRSerializationError>
//    where Self : Sized {
//    println!("There can't be a default implementation for this trait!");
//    todo!();
//    }
//
//  /// Construct byte slice out of self.
//  ///
//  /// Can not fail.
//  fn to_slice(&self) 
//    -> &[u8]
//    where Self : Sized {
//    println!("There can't be a default implementation for this trait!");
//    todo!();
//  }
}

#[test]
fn test_seek_marker() {
  // just test it two times - FIXME - use a better method
  let mut bytestream = vec![1,2,3,0xAA, 0xAA, 5, 7];
  let mut pos = seek_marker(&bytestream, 0xaaaa, 0).unwrap();
  assert_eq!(pos, 3);
  
  bytestream = vec![1,2,3,244, 16, 32, 0xaa, 0xff, 5, 7];
  // remember byte order in vectors
  pos = seek_marker(&bytestream, 0xffaa, 1).unwrap();
  assert_eq!(pos, 6);
  
  bytestream = vec![0xaa,0xaa,3,244, 16, 32, 0xAA, 0xFF, 5, 7];
  pos = seek_marker(&bytestream, 0xaaaa, 0).unwrap();
  assert_eq!(pos, 0);
}


//use half::f16;
//
//// re-exports
//pub use crate::errors::CRSerializationError;
//
//use std::error::Error;
//use std::path::Path;
//
//use std::collections::VecDeque;
//
//use crate::packets::{
//  TofPacket,
//  PacketType,
//};
//
//
//#[derive(Debug, Copy, Clone)]
//pub struct f8(u8);
//
//impl f8 {
//    // Convert from little-endian byte array to f8
//    pub fn from_le_bytes(bytes: [u8; 1]) -> Self {
//        f8(bytes[0])
//    }
//}
//
///// Convert a vector of u16 into a vector of u8
/////
///// The resulting vector has twice the number
///// of entries of the original vector.
///// This is useful, when serializing data 
///// represented as u16, e.g. the waveforms.
//pub fn u16_to_u8(vec_u16: &[u16]) -> Vec<u8> {
//    vec_u16.iter()
//        .flat_map(|&n| n.to_le_bytes().to_vec())
//        .collect()
//}
//
///// Restore a vector of u16 from a vector of u8
/////
///// This interpretes two following u8 as an u16
///// Useful for deserialization of waveforms.
//pub fn u8_to_u16(vec_u8: &[u8]) -> Vec<u16> {
//    vec_u8.chunks_exact(2)
//        .map(|chunk| u16::from_le_bytes([chunk[0], chunk[1]]))
//        .collect()
//}
//
///// Restore a vector of u16 from a vector of u8
/////
///// This interpretes two following u8 as an u16
///// Useful for deserialization of waveforms.
///// Additionally it masks the first 2 bits 
///// binary adding 0x3ff to each u16.
//pub fn u8_to_u16_14bit(vec_u8: &[u8]) -> Vec<u16> {
//    vec_u8.chunks_exact(2)
//        .map(|chunk| 0x3fff & u16::from_le_bytes([chunk[0], chunk[1]]))
//        .collect()
//}
//
///// Restore a vector of u16 from a vector of u8, using the first 2 bits of each u16 
///// to get channel/cell error bit information
/////
///// This interpretes two following u8 as an u16
///// Useful for deserialization of waveforms.
///// Additioanlly, it preserves the error bits
/////
///// # Arguments:
/////
///// # Returns:
/////
/////   `Vec<u16>`, ch_sync_err, cell_sync_err : if one of the error bits is
/////                                            set, ch_sync_err or cell_sync_err
/////                                            will be set to true
//pub fn u8_to_u16_err_check(vec_u8: &[u8]) -> (Vec<u16>, bool, bool) {
//    let mut ch_sync_err   = true;
//    let mut cell_sync_err = true;
//    let vec_u16 = vec_u8.chunks_exact(2)
//        .map(|chunk| {
//          let value     =  u16::from_le_bytes([chunk[0], chunk[1]]);
//          ch_sync_err   = ch_sync_err   && (((0x8000 & value) >> 15) == 0x1); 
//          cell_sync_err = cell_sync_err && (((0x4000 & value) >> 14) == 0x1) ;
//          return 0x3fff & value;
//        })
//        .collect();
//    (vec_u16, ch_sync_err, cell_sync_err)
//}
//
//pub fn parse_u8(bs : &Vec::<u8>, pos : &mut usize) -> u8 {
//  let value = u8::from_le_bytes([bs[*pos]]);
//  *pos += 1;
//  value
//}
//
//pub fn parse_u8_deque(bs : &VecDeque::<u8>, pos : &mut usize) -> u8 {
//  let value = u8::from_le_bytes([bs[*pos]]);
//  *pos += 1;
//  value
//}
//
//
///// Get u32 from a bytestream and move on the position marker
/////
///// # Arguments 
/////
///// * bs
///// * pos 
//pub fn parse_u16(bs : &Vec::<u8>, pos : &mut usize) -> u16 {
//  let value = u16::from_le_bytes([bs[*pos], bs[*pos+1]]);
//  *pos += 2;
//  value
//}
//
//// FIXME - make this a generic
//pub fn parse_u16_deque(bs : &VecDeque::<u8>, pos : &mut usize) -> u16 {
//  let value = u16::from_le_bytes([bs[*pos], bs[*pos+1]]);
//  *pos += 2;
//  value
//}
//
///// BIG Endian version of parse_u32. NOT for botched event id decoding!
///// Used for network communications
//pub fn parse_u32_be(bs : &Vec::<u8>, pos : &mut usize) -> u32 {
//  let value = u32::from_be_bytes([bs[*pos], bs[*pos+1], bs[*pos+2], bs[*pos+3]]);
//  *pos += 4;
//  value
//}
//
//pub fn parse_u32(bs : &Vec::<u8>, pos : &mut usize) -> u32 {
//  let value = u32::from_le_bytes([bs[*pos], bs[*pos+1], bs[*pos+2], bs[*pos+3]]);
//  *pos += 4;
//  value
//}
//
//pub fn parse_u64(bs : &Vec::<u8>, pos : &mut usize) -> u64 {
//  let value = u64::from_le_bytes([bs[*pos],   bs[*pos+1], bs[*pos+2], bs[*pos+3],
//                                  bs[*pos+4], bs[*pos+5], bs[*pos+6], bs[*pos+7]]);
//  *pos += 8;
//  value
//}
//
//#[cfg(not(target_arch="arm"))]
//pub fn parse_usize(bs: &Vec::<u8>, pos: &mut usize) -> usize {
//  let value: usize = usize::from_le_bytes([bs[*pos],bs[*pos + 1], bs[*pos + 2], bs[*pos + 3], 
//    bs[*pos + 4], bs[*pos + 5], bs[*pos + 6], bs[*pos + 7],]);
//  *pos += std::mem::size_of::<usize>();
//  value
//}
//
//#[cfg(target_arch="arm")]
//pub fn parse_usize(bs: &Vec::<u8>, pos: &mut usize) -> usize {
//  parse_u32(bs, pos) as usize
//}
//
///// Get an u32 from a bytestream 
/////
///// This assumes an underlying representation of 
///// an atomic unit of 16bit instead of 8.
///// This is realized for the raw data stream
///// from the readoutboards.
//pub fn parse_u32_for_16bit_words(bs  : &Vec::<u8>,
//                                 pos : &mut usize) -> u32 {
//  
//  let raw_bytes_4  = [bs[*pos + 2],
//                      bs[*pos + 3],
//                      bs[*pos    ],
//                      bs[*pos + 1]];
//  *pos += 4;
//  u32::from_le_bytes(raw_bytes_4)
//}
//
///// Get an 48bit number from a bytestream 
/////
///// This assumes an underlying representation of 
///// an atomic unit of 16bit instead of 8.
///// This is realized for the raw data stream
///// from the readoutboards.
//pub fn parse_u48_for_16bit_words(bs  : &Vec::<u8>,
//                                 pos : &mut usize) -> u64 {
//  
//  let raw_bytes_8  = [0u8,
//                      0u8,
//                      bs[*pos + 4],
//                      bs[*pos + 5],
//                      bs[*pos + 2],
//                      bs[*pos + 3],
//                      bs[*pos    ],
//                      bs[*pos + 1]];
//  *pos += 6;
//  u64::from_le_bytes(raw_bytes_8)
//}
//
//pub fn parse_f8(bs: &Vec<u8>, pos: &mut usize) -> f8 {
//  let value = f8::from_le_bytes([bs[*pos]]);
//  *pos += 1;
//  value
//}
//
//pub fn parse_f16(bs : &Vec::<u8>, pos : &mut usize) -> f16 {
//  let value = f16::from_le_bytes([bs[*pos], bs[*pos+1]]);
//  *pos += 2;
//  value
//}
//
//pub fn parse_f32(bs : &Vec::<u8>, pos : &mut usize) -> f32 {
//  let value = f32::from_le_bytes([bs[*pos],   bs[*pos+1],  
//                                  bs[*pos+2], bs[*pos+3]]);
//  *pos += 4;
//  value
//}
//
//pub fn parse_f64(bs : &Vec::<u8>, pos : &mut usize) -> f64 {
//  let value = f64::from_le_bytes([bs[*pos],   bs[*pos+1],  
//                                  bs[*pos+2], bs[*pos+3],
//                                  bs[*pos+4], bs[*pos+5],
//                                  bs[*pos+6], bs[*pos+7]]);
//  *pos += 8;
//  value
//}
//
//pub fn parse_bool(bs : &Vec::<u8>, pos : &mut usize) -> bool {
//  let value = u8::from_le_bytes([bs[*pos]]); 
//  *pos += 1;
//  value > 0
//}
//
//pub fn get_json_from_file(filename : &Path)
//    -> Result<String, Box<dyn Error>> {
//  let file_content = std::fs::read_to_string(filename)?;
//  let config = serde_json::from_str(&file_content)?;
//  Ok(config)
//}
//
//
//
//
//
//#[cfg(test)]
//mod test_serialization {
//  use crate::serialization::{search_for_u16,
//                             u16_to_u8};
//
//  #[test]
//  fn test_u16_to_u8_size_doubled() {
//    let size = 1000usize;
//    let data = vec![42u16;size];
//    let data_u8 = u16_to_u8(data.as_slice());
//    let data_u8_size = data_u8.len();
//    let double_size  = 2*size;
//    assert_eq!(data_u8_size, double_size);
//    
//  }
//
//  #[test]
//  fn test_search_for_2_bytemarker() {
//    // just test it two times - FIXME - use a better method
//    let mut bytestream = vec![1,2,3,0xAA, 0xAA, 5, 7];
//    let mut pos = search_for_u16(0xAAAA, &bytestream, 0).unwrap();
//    assert_eq!(pos, 3);
//    
//    bytestream = vec![1,2,3,244, 16, 32, 0xaa, 0xff, 5, 7];
//    pos = search_for_u16(65450, &bytestream, 1).unwrap();
//    assert_eq!(pos, 6);
//    
//    bytestream = vec![0xaa,0xaa,3,244, 16, 32, 0xAA, 0xFF, 5, 7];
//    pos = search_for_u16(0xaaaa, &bytestream, 0).unwrap();
//    assert_eq!(pos, 0);
//  }
//}
