#include "parsers.h"

u16 Gaps::u16_from_le_bytes(const vec_u8 &bytestream,
                            u64 &pos) {
  u16 value = (u16)(
        ((bytestream[pos+1] & 0xFF) << 8)
      |  (bytestream[pos]));
  pos += 2;
  return value;
}

void Gaps::u16_to_le_bytes(const u16 value, 
                     vec_u8 &bytestream,
                     usize &pos) {
  bytestream[pos + 1] = (value >> 8)  & 0xFF;
  bytestream[pos] = value & 0xFF;
  pos += 2;
}

u32 Gaps::u32_from_le_bytes(const vec_u8 &bytestream,
                            usize &pos) {

  u32 value = (u32)(
         ((bytestream[pos+3] & 0xFF) << 24)
      |  ((bytestream[pos+2] & 0xFF) << 16)
      |  ((bytestream[pos+1] & 0xFF) << 8)
      |   (bytestream[pos+0]));
  pos += 4;
  return value;
}

u32 Gaps::u32_from_be_bytes(const vec_u8 &bytestream,
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


void Gaps::u32_to_le_bytes(const u32 value, 
                           vec_u8 &bytestream,
                           usize &pos) {

  bytestream[pos + 3] = (value >> 24) & 0xFF;
  bytestream[pos + 2] = (value >> 16) & 0xFF;
  bytestream[pos + 1] = (value >> 8)  & 0xFF;
  bytestream[pos] = value & 0xFF;
  pos += 4;
}
