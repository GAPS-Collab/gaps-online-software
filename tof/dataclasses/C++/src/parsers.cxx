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
  return Gaps::u16_from_le_bytes(bytestream, pos);
}

u32 Gaps::parse_u32(const Vec<u8> &bytestream,
                    u64 &pos) {
  return Gaps::u32_from_le_bytes(bytestream, pos);
}

u64 Gaps::parse_u64(const Vec<u8> &bytestream,
                    usize &pos) {

  return Gaps::u64_from_le_bytes(bytestream, pos);
}

i32 Gaps::parse_i32(const Vec<u8> &bytestream,
                    usize &pos) {
  i32 result = 0;
  // Assuming little-endian byte order (LSB first)
  for (int i = 0; i < 4; ++i) {
    result |= (static_cast<int32_t>(bytestream[i]) << (i * 8));
  }  
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

u16 Gaps::u16_from_le_bytes(const Vec<u8> &bytestream,
                            u64 &pos) {
  u16 value = (u16)(
        ((bytestream[pos+1] & 0xFF) << 8)
      |  (bytestream[pos]));
  pos += 2;
  return value;
}

void Gaps::u16_to_le_bytes(const u16 value, 
                     Vec<u8> &bytestream,
                     usize &pos) {
  bytestream[pos + 1] = (value >> 8)  & 0xFF;
  bytestream[pos] = value & 0xFF;
  pos += 2;
}

u32 Gaps::u32_from_le_bytes(const Vec<u8> &bytestream,
                            usize &pos) {

  u32 value = (u32)(
         ((bytestream[pos+3] & 0xFF) << 24)
      |  ((bytestream[pos+2] & 0xFF) << 16)
      |  ((bytestream[pos+1] & 0xFF) << 8)
      |   (bytestream[pos+0]));
  pos += 4;
  return value;
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

u64 Gaps::u64_from_le_bytes(const Vec<u8> &bytestream,
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


void Gaps::u32_to_le_bytes(const u32 value, 
                           Vec<u8> &bytestream,
                           usize &pos) {

  bytestream[pos + 3] = (value >> 24) & 0xFF;
  bytestream[pos + 2] = (value >> 16) & 0xFF;
  bytestream[pos + 1] = (value >> 8)  & 0xFF;
  bytestream[pos] = value & 0xFF;
  pos += 4;
}
