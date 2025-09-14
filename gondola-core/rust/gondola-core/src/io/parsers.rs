//! This file contains generic parsers to read data from a stream of 
//! bytes and interpret them as various types.
//!
// This file is part of gaps-online-software and published 
// under the GPLv3 license

use crate::prelude::*;

//// Luma's generic version - needs to be checked and benchmarked
//pub fn parse_num<T, S>(stream: S, pos: &mut usize) -> T
//where
//    T: Copy + Default + Sized + FromBytes,
//    S: AsRef<[u8]> {
//    let bs = stream.as_ref();
//    let mut buf = [0u8; size_of::<T>()];
//    buf.copy_from_slice(&bs[*pos..*pos + size_of::<T>()]);
//    *pos += size_of::<T>();
//    T::from_le_bytes(buf)
//}

/// Get a u8 from a vector of bytes and advance 
/// a position marker by 1
pub fn parse_bool<T: AsRef<[u8]>>(stream : &T, pos : &mut usize) -> bool {
  let bs = stream.as_ref();
  let value = u8::from_le_bytes([bs[*pos]]); 
  *pos += 1;
  value > 0
}


/// Get a u8 from a vector of bytes and advance 
/// a position marker by 1
pub fn parse_u8<T: AsRef<[u8]>>(stream : &T, pos : &mut usize) -> u8 {
  let bs = stream.as_ref();
  let value = u8::from_le_bytes([bs[*pos]]);
  *pos += 1;
  value
}

/// Get a u16 from a vector of bytes and advance 
/// a position marker by 2
///
/// Note: written out as a generic here, TODO benchmark
pub fn parse_u16<T: AsRef<[u8]>>(stream : &T, pos : &mut usize) -> u16 {
  let bs = stream.as_ref();
  let value = u16::from_le_bytes([bs[*pos], bs[*pos+1]]);
  *pos += 2;
  value
}

/// Get a u16 from a vector of bytes in big-endian and advance 
/// a position marker by 2
pub fn parse_u16_be<T: AsRef<[u8]>>(stream : &T, pos : &mut usize) -> u16 {
  let bs = stream.as_ref();
  let value = u16::from_be_bytes([bs[*pos], bs[*pos+1]]);
  *pos += 2;
  value
}

/// Get a u32 from a vector of bytes in big-endian and advance 
/// a position marker by 4
pub fn parse_u32_be<T: AsRef<[u8]>>(stream : &T, pos : &mut usize) -> u32 {
  let bs = stream.as_ref();
  let value = u32::from_be_bytes([bs[*pos], bs[*pos+1], bs[*pos+2], bs[*pos+3]]);
  *pos += 4;
  value
}

/// Get a u32 from a vector of bytes and advance 
/// a position marker by 4
pub fn parse_u32<T: AsRef<[u8]>>(stream : &T, pos : &mut usize) -> u32 {
  let bs = stream.as_ref();
  let value = u32::from_le_bytes([bs[*pos], bs[*pos+1], bs[*pos+2], bs[*pos+3]]);
  *pos += 4;
  value
}

/// Get a u64 from a vector of bytes and advance 
/// a position marker by 8
pub fn parse_u64<T: AsRef<[u8]>>(stream : &T, pos : &mut usize) -> u64 {
  let bs = stream.as_ref();
  let value = u64::from_le_bytes([bs[*pos],   bs[*pos+1], bs[*pos+2], bs[*pos+3],
                                  bs[*pos+4], bs[*pos+5], bs[*pos+6], bs[*pos+7]]);
  *pos += 8;
  value
}


/// Get a u64 from a vector of bytes and advance 
/// a position marker by 8
pub fn parse_u64_old_for_test(bs : &Vec::<u8>, pos : &mut usize) -> u64 {
  let value = u64::from_le_bytes([bs[*pos],   bs[*pos+1], bs[*pos+2], bs[*pos+3],
                                  bs[*pos+4], bs[*pos+5], bs[*pos+6], bs[*pos+7]]);
  *pos += 8;
  value
}

#[deprecated(note = "Please use parse_u32 or parse_u64 explicitly, since the decoding of the bytestream is architecture independent!")]
#[cfg(not(target_arch="arm"))]
pub fn parse_usize(bs: &Vec::<u8>, pos: &mut usize) -> usize {
  let value: usize = usize::from_le_bytes([bs[*pos],bs[*pos + 1], bs[*pos + 2], bs[*pos + 3], 
    bs[*pos + 4], bs[*pos + 5], bs[*pos + 6], bs[*pos + 7],]);
  *pos += std::mem::size_of::<usize>();
  value
}

#[deprecated(note = "Please use parse_u32 or parse_u64 explicitly, since the decoding of the bytestream is architecture independent!")]
#[cfg(target_arch="arm")]
pub fn parse_usize(bs: &Vec::<u8>, pos: &mut usize) -> usize {
  parse_u32(bs, pos) as usize
}

/// Get a string from a bytestream and advance a position marker
/// 
/// Warning, this is unsafe and might fail. It also expects that the 
/// string is perfixed with a u16 containing its size.
///
/// # Arguments 
///
/// * bs     : Serialized data, stream of bytes
/// * pos    : Position marker - start postion of 
///            the deserialization
pub fn parse_string<T: AsRef<[u8]>>(stream : &T, pos : &mut usize) -> String {
  let bs    = stream.as_ref();
  let size  = parse_u16(stream, pos) as usize;
  let s_string : Vec<u8> = bs[*pos..*pos + size].to_vec();
  let value = String::from_utf8(s_string).unwrap();
  *pos += size;
  value
}

/// Get a u32 from a vector of bytes and advance
/// a position marker by 4 for a non-standard 
/// representation of u32 (neither le or be, but 
/// shuffled)
/// 
/// <div class="warning>
/// This assumes an underlying representation of 
/// an atomic unit of 16bit instead of 8.
/// This is a non-convetional byte respresentation
/// for a u32 and needs to be used with care
/// </div>
pub fn parse_u32_for_16bit_words<T: AsRef<[u8]>>(stream : &T, pos : &mut usize) -> u32 {
  let bs = stream.as_ref();
  let raw_bytes_4  = [bs[*pos + 2],
                      bs[*pos + 3],
                      bs[*pos    ],
                      bs[*pos + 1]];
  *pos += 4;
  u32::from_le_bytes(raw_bytes_4)
}

/// Get a 48bit number from a vector of bytes 
///
/// <div class="warning"> 48bit unsigned integer is a custom "type"!
/// </div>
///
/// <div class="warning"> 
/// This assumes an underlying representation of 
/// an atomic unit of 16bit instead of 8.
/// </div>
pub fn parse_u48_for_16bit_words<T: AsRef<[u8]>>(stream : &T, pos : &mut usize) -> u64 {
  let bs = stream.as_ref();
  let raw_bytes_8  = [0u8,
                      0u8,
                      bs[*pos + 4],
                      bs[*pos + 5],
                      bs[*pos + 2],
                      bs[*pos + 3],
                      bs[*pos    ],
                      bs[*pos + 1]];
  *pos += 6;
  u64::from_le_bytes(raw_bytes_8)
}

/// Get a f16 from a vector of bytes and advance 
/// a position marker by 2
///
/// <div class="warning"> f16 is called "half" and a non-common
/// datatype which can dependent on the implementaion.
/// The implementation used here is from the rust "half" crate
/// </div>
pub fn parse_f16<T: AsRef<[u8]>>(stream : &T, pos : &mut usize) -> f16 {
  let bs = stream.as_ref();
  let value = f16::from_le_bytes([bs[*pos], bs[*pos+1]]);
  *pos += 2;
  value
}

/// Get a f32 from a vector of bytes and advance 
/// a position marker by 4
pub fn parse_f32<T: AsRef<[u8]>>(stream : &T, pos : &mut usize) -> f32 {
  let bs = stream.as_ref();
  let value = f32::from_le_bytes([bs[*pos],   bs[*pos+1],  
                                  bs[*pos+2], bs[*pos+3]]);
  *pos += 4;
  value
}

/// Get a f64 from a vector of bytes and advance 
/// a position marker by 8
pub fn parse_f64<T: AsRef<[u8]>>(stream : &T, pos : &mut usize) -> f64 {
  let bs = stream.as_ref();
  let value = f64::from_le_bytes([bs[*pos],   bs[*pos+1],  
                                  bs[*pos+2], bs[*pos+3],
                                  bs[*pos+4], bs[*pos+5],
                                  bs[*pos+6], bs[*pos+7]]);
  *pos += 8;
  value
}

/// Restore a vector of u16 from a vector of u8
///
/// This interpretes two following u8 as an u16
/// Useful for deserialization of waveforms.
pub fn u8_to_u16(vec_u8: &[u8]) -> Vec<u16> {
    vec_u8.chunks_exact(2)
        .map(|chunk| u16::from_le_bytes([chunk[0], chunk[1]]))
        .collect()
}

/// This interpretes two following u8 as an u16
/// Useful for deserialization of waveforms.
/// Additionally it masks the first 2 bits 
/// binary adding 0x3ff to each u16.
pub fn u8_to_u16_14bit(vec_u8: &[u8]) -> Vec<u16> {
    vec_u8.chunks_exact(2)
        .map(|chunk| 0x3fff & u16::from_le_bytes([chunk[0], chunk[1]]))
        .collect()
}

/// Restore a vector of u16 from a vector of u8, using the first 2 bits of each u16 
/// to get channel/cell error bit information
///
/// This interpretes two following u8 as an u16
/// Useful for deserialization of waveforms.
/// Additioanlly, it preserves the error bits
///
/// # Arguments:
///
/// # Returns:
///
///   `Vec<u16>`, ch_sync_err, cell_sync_err : if one of the error bits is
///                                            set, ch_sync_err or cell_sync_err
///                                            will be set to true
pub fn u8_to_u16_err_check(vec_u8: &[u8]) -> (Vec<u16>, bool, bool) {
    let mut ch_sync_err   = true;
    let mut cell_sync_err = true;
    let vec_u16 = vec_u8.chunks_exact(2)
        .map(|chunk| {
          let value     =  u16::from_le_bytes([chunk[0], chunk[1]]);
          ch_sync_err   = ch_sync_err   && (((0x8000 & value) >> 15) == 0x1); 
          cell_sync_err = cell_sync_err && (((0x4000 & value) >> 14) == 0x1) ;
          return 0x3fff & value;
        })
        .collect();
    (vec_u16, ch_sync_err, cell_sync_err)
}

/// The resulting vector has twice the number
/// of entries of the original vector.
/// This is useful, when serializing data 
/// represented as u16, e.g. the waveforms.
pub fn u16_to_u8(vec_u16: &[u16]) -> Vec<u8> {
    vec_u16.iter()
        .flat_map(|&n| n.to_le_bytes().to_vec())
        .collect()
}

//====================================================

#[cfg(feature = "random")]
#[test]
fn test_parse_bool() {
  #[cfg(feature="random")]
  use rand::Rng;

  let mut rng    = rand::rng();
  let mut stream = Vec::<u8>::new();
  let mut data   = Vec::<bool>::new();
  for _ in 0..100 {
    let test_byte  = rng.random::<bool>();
    stream.push(test_byte as u8);
    data.push(test_byte);
  }
  let mut pos = 0usize;
  for k in 0..stream.len() {
    assert_eq!(parse_bool(&stream, &mut pos), data[k]);
  }
}

#[cfg(feature = "random")]
#[test]
fn test_parse_u8() {
  #[cfg(feature="random")]
  use rand::Rng;

  let mut rng    = rand::rng();
  let mut stream = Vec::<u8>::new();
  let mut data   = Vec::<u8>::new();
  for _ in 0..100 {
    let test_byte  = rng.random::<u8>();
    stream.push(test_byte);
    data.push(test_byte);
  }
  let mut pos = 0usize;
  for k in 0..stream.len() {
    assert_eq!(parse_u8(&stream, &mut pos), data[k]);
  }
}

#[cfg(feature = "random")]
#[test]
fn test_parse_u16() {
  #[cfg(feature="random")]
  use rand::Rng;

  let mut rng    = rand::rng();
  let mut stream = Vec::<u8>::new();
  let mut data   = Vec::<u16>::new();
  for _ in 0..100 {
    let test_data  = rng.random::<u16>();
    for k in test_data.to_le_bytes() {
      stream.push(k);
    }
    data.push(test_data);
  }
  let mut pos = 0usize;
  for k in data {
    assert_eq!(parse_u16(&stream, &mut pos), k);
  }
}

#[test]
fn test_parse_u16_be() {
  #[cfg(feature="random")]
  use rand::Rng;

  let mut rng    = rand::rng();
  let mut stream = Vec::<u8>::new();
  let mut data   = Vec::<u16>::new();
  for _ in 0..100 {
    let test_data  = rng.random::<u16>();
    for k in test_data.to_be_bytes() {
      stream.push(k);
    }
    data.push(test_data);
  }
  let mut pos = 0usize;
  for k in data {
    assert_eq!(parse_u16_be(&stream, &mut pos), k);
  }
}

#[cfg(feature = "random")]
#[test]
fn test_parse_u32() {
  #[cfg(feature="random")]
  use rand::Rng;

  let mut rng    = rand::rng();
  let mut stream = Vec::<u8>::new();
  let mut data   = Vec::<u32>::new();
  for _ in 0..100 {
    let test_data  = rng.random::<u32>();
    for k in test_data.to_le_bytes() {
      stream.push(k);
    }
    data.push(test_data);
  }
  let mut pos = 0usize;
  for k in data {
    assert_eq!(parse_u32(&stream, &mut pos), k);
  }
}

#[test]
fn test_parse_u32_be() {
  #[cfg(feature="random")]
  use rand::Rng;

  let mut rng    = rand::rng();
  let mut stream = Vec::<u8>::new();
  let mut data   = Vec::<u32>::new();
  for _ in 0..100 {
    let test_data  = rng.random::<u32>();
    for k in test_data.to_be_bytes() {
      stream.push(k);
    }
    data.push(test_data);
  }
  let mut pos = 0usize;
  for k in data {
    assert_eq!(parse_u32_be(&stream, &mut pos), k);
  }
}

#[cfg(feature = "random")]
#[test]
fn test_parse_u64() {
  #[cfg(feature="random")]
  use rand::Rng;

  let mut rng    = rand::rng();
  let mut stream = Vec::<u8>::new();
  let mut data   = Vec::<u64>::new();
  for _ in 0..100 {
    let test_data  = rng.random::<u64>();
    for k in test_data.to_le_bytes() {
      stream.push(k);
    }
    data.push(test_data);
  }
  let mut pos = 0usize;
  for k in data {
    assert_eq!(parse_u64(&stream, &mut pos), k);
  }
}

#[cfg(feature = "random")]
#[test]
fn test_parse_f32() {
  #[cfg(feature="random")]
  use rand::Rng;

  let mut rng    = rand::rng();
  let mut stream = Vec::<u8>::new();
  let mut data   = Vec::<f32>::new();
  for _ in 0..100 {
    let test_data  = rng.random::<f32>();
    for k in test_data.to_le_bytes() {
      stream.push(k);
    }
    data.push(test_data);
  }
  let mut pos = 0usize;
  for k in data {
    assert_eq!(parse_f32(&stream, &mut pos), k);
  }
}

#[cfg(feature = "random")]
#[test]
fn test_parse_f64() {
  #[cfg(feature="random")]
  use rand::Rng;

  let mut rng    = rand::rng();
  let mut stream = Vec::<u8>::new();
  let mut data   = Vec::<f64>::new();
  for _ in 0..100 {
    let test_data  = rng.random::<f64>();
    for k in test_data.to_le_bytes() {
      stream.push(k);
    }
    data.push(test_data);
  }
  let mut pos = 0usize;
  for k in data {
    assert_eq!(parse_f64(&stream, &mut pos), k);
  }
}

//--------------------------------------------

#[test]
fn prop_u8_u16_back_and_forth() {
  use quickcheck::quickcheck;
  quickcheck! {
    fn prop_roundtrip(vec: Vec<u8>) -> bool {
      let converted = u8_to_u16(&vec);
      let roundtrip = u16_to_u8(&converted);

      // Only the portion that forms full pairs
      let expected: Vec<u8> = vec.chunks_exact(2)
          .flat_map(|chunk| chunk.to_vec())
          .collect();

      roundtrip == expected
    }
  }
}



