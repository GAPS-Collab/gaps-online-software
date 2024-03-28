#include <cstring>
#include "parsers.h"

bool Gaps::parse_bool(const Vec<u8> &bytestream,
                      usize &pos) {
  u8 value = bytestream[pos];
  pos += 1;
  return value > 0;
}

u8 Gaps::parse_u8(const Vec<u8> &bytestream,
                  u64 &pos) {
  u8 value = bytestream[pos];
  pos += 1;
  return value;
}

u16 Gaps::parse_u16(const Vec<u8> &bytestream,
                    u64 &pos) {
  u16 value = (u16)(
        ((bytestream[pos+1] & 0xFF) << 8)
      |  (bytestream[pos]));
  pos += 2;
  return value;
}

u32 Gaps::parse_u32(const Vec<u8> &bytestream,
                    u64 &pos) {
  u32 value = (u32)(
         ((bytestream[pos+3] & 0xFF) << 24)
      |  ((bytestream[pos+2] & 0xFF) << 16)
      |  ((bytestream[pos+1] & 0xFF) << 8)
      |   (bytestream[pos+0]));
  pos += 4;
  return value;
}

u64 Gaps::parse_u64(const Vec<u8> &bytestream,
                    usize &pos) {
  u64 value = (u64)(
         ((bytestream[pos+7] & 0xFF) << 56)
      |  ((bytestream[pos+6] & 0xFF) << 48)
      |  ((bytestream[pos+5] & 0xFF) << 40)
      |  ((bytestream[pos+4] & 0xFF) << 32)
      |  ((bytestream[pos+3] & 0xFF) << 24)
      |  ((bytestream[pos+2] & 0xFF) << 16)
      |  ((bytestream[pos+1] & 0xFF) << 8)
      |   (bytestream[pos+0]));
  pos += 8;
  return value;
}

i32 Gaps::parse_i32(const Vec<u8> &bytestream,
                    usize &pos) {
  i32 result = 0;
  // Assuming little-endian byte order (LSB first)
  for (int i = 0; i < 4; ++i) {
    result |= (static_cast<int32_t>(bytestream[i]) << (i * 8));
  } 
  pos += 4; 
  return result;
}

f32 Gaps::parse_f32(const Vec<u8> &bytestream,
                    usize &pos) {
  f32 result;
  Vec<u8> bytes = Gaps::slice(bytestream,pos,pos+4); 
  // Copy the bytes into a float variable using type punning
  std::memcpy(&result, bytes.data(), sizeof(f32));
  pos += 4;
  return result;
}

f64 Gaps::parse_f64(const Vec<u8> &bytestream,
                    usize &pos) {
  f64 result;
  Vec<u8> bytes = Gaps::slice(bytestream,pos,pos+8); 
  // Copy the bytes into a float variable using type punning
  std::memcpy(&result, bytes.data(), sizeof(f64));
  pos += 8;
  return result;
}


u32 Gaps::u32_from_be_bytes(const Vec<u8> &bytestream,
                            usize &pos) {

  u32 value = (u32)(
         ((bytestream[pos+0] & 0xFF) << 24)
      |  ((bytestream[pos+1] & 0xFF) << 16)
      |  ((bytestream[pos+2] & 0xFF) << 8)
      |   (bytestream[pos+3]));
  pos += 4;
  return value;
}

u32 Gaps::parse_u32_for_16bit_words(const Vec<u8> &bs,
                                    usize &pos) {
  u32 value = (u32)(
         ((bs[pos+1] & 0xFF) << 24)
      |  ((bs[pos+0] & 0xFF) << 16)
      |  ((bs[pos+3] & 0xFF) << 8)
      |  ( bs[pos+2]));
  pos += 4;
  return value;
}
 
u64 Gaps::parse_u48_for_16bit_words(const Vec<u8> &bytestream,
                                    usize &pos) {
  u64 buffer64 = 0x0000000000000000;
  u64 buffer =  
         (((bytestream[pos+1] & 0xFF) | buffer64) << 40)
      |  (((bytestream[pos+0] & 0xFF) | buffer64) << 32)
      |  (((bytestream[pos+3] & 0xFF) | buffer64) << 24)
      |  (((bytestream[pos+2] & 0xFF) | buffer64) << 16)
      |  (((bytestream[pos+5] & 0xFF) | buffer64) << 8)
      |  (((bytestream[pos+4] & 0xFF) | buffer64));

  pos += 6;
  return buffer;
}


