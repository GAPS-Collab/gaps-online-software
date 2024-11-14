//! Atomic units to get data from bytestreams

/// Get a u8 from a bytestream and advance a position marker
///
/// # Arguments 
///
/// * bs     : Serialized data, stream of bytes
/// * pos    : Position marker - start postion of 
///            the deserialization
pub fn parse_u8<T: AsRef<[u8]>>(stream : &T, pos : &mut usize) -> u8 {
  let bs    = stream.as_ref();
  let value = u8::from_le_bytes([bs[*pos]]);
  *pos += 1;
  value
}

/// Get a u16 from a bytestream and advance a position marker
///
/// # Arguments 
///
/// * bs     : Serialized data, stream of bytes
/// * pos    : Position marker - start postion of 
///            the deserialization
pub fn parse_u16<T: AsRef<[u8]>>(stream : &T, pos : &mut usize) -> u16 {
  let bs    = stream.as_ref();
  let value = u16::from_le_bytes([bs[*pos], bs[*pos+1]]);
  *pos += 2;
  value
}

/// Get a u32 from a bytestream and advance a position marker
///
/// # Arguments 
///
/// * bs     : Serialized data, stream of bytes
/// * pos    : Position marker - start postion of 
///            the deserialization
pub fn parse_u32<T: AsRef<[u8]>>(stream : &T, pos : &mut usize) -> u32 {
  let bs    = stream.as_ref();
  let value = u32::from_le_bytes([bs[*pos], bs[*pos+1], bs[*pos+2], bs[*pos+3]]);
  *pos += 4;
  value
}

/// Get a u64 from a bytestream and advance a position marker
///
/// # Arguments 
///
/// * bs     : Serialized data, stream of bytes
/// * pos    : Position marker - start postion of 
///            the deserialization
pub fn parse_u64<T: AsRef<[u8]>>(stream : &T, pos : &mut usize) -> u64 {
  let bs    = stream.as_ref();
  let value = u64::from_le_bytes([bs[*pos],   bs[*pos+1], bs[*pos+2], bs[*pos+3],
                                  bs[*pos+4], bs[*pos+5], bs[*pos+6], bs[*pos+7]]);
  *pos += 8;
  value
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

//pub fn parse_u8_deque(bs : &VecDeque::<u8>, pos : &mut usize) -> u8 {
//  let value = u8::from_le_bytes([bs[*pos]]);
//  *pos += 1;
//  value
//}
//
//
///// Get a u16 from a bytestream and advance a position marker
/////
///// # Arguments 
/////
///// * bs     : Serialized data, stream of bytes
///// * pos    : Position marker - start postion of 
/////            the deserialization
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
//
//
