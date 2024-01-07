#ifndef GAPSPARSERS_H_INCLUDED
#define GAPSPARSERS_H_INCLUDED

#include "tof_typedefs.h"

namespace Gaps {

template<typename T>
Vec<T> slice(const Vec<T>& vec, usize start, usize end) {
  if (start >= vec.size()) {
    return Vec<T>();  // Return an empty vector if start is out of range
  }
  end = std::min(end, vec.size());  // Clamp the end index to the vector size
  return Vec<T>(vec.begin() + start, vec.begin() + end);
}


u16 u16_from_le_bytes(const Vec<u8> &bytestream,
                      u64 &pos);

void u16_to_le_bytes(const u16 value, 
                     Vec<u8> &bytestream,
                     usize &pos);



/**
 * Get an u32 from a vector of bytes. 
 *
 * The byteorder is compatible with the 
 * rust from_le_bytes, where
 * 
 * let value = u32::from_le_bytes([0x78, 0x56, 0x34, 0x12]);
 * assert_eq!(value, 0x12345678);
 *
 * @params: 
 *
 * @pos : this gets advanced by 4 bytes
 *
 *
 */
bool parse_bool(const Vec<u8> &bytestream,
                usize &pos);

u8 parse_u8(const Vec<u8> &bytestream,
            usize &pos);

u16 parse_u16(const Vec<u8> &bytestream,
              usize &pos);

u32 parse_u32(const Vec<u8> &bytestream,
              usize &pos);

u64 parse_u64(const Vec<u8> &bytestream,
              usize &pos);

i32 parse_i32(const Vec<u8> &bytestream,
              usize &pos);

f32 parse_f32(const Vec<u8> &bytestream,
              usize &pos);

f64 parse_f64(const Vec<u8> &bytestream,
              usize &pos);

u32 u32_from_le_bytes(const Vec<u8> &bytestream,
                      usize &pos);

u64 u64_from_le_bytes(const Vec<u8> &bytestream,
                      usize &pos);

u32 u32_from_be_bytes(const Vec<u8> &bytestream,
                      usize &pos);

u32 parse_u32_for_16bit_words(const Vec<u8> &bytestream,
                              usize &pos);

u64 parse_u48_for_16bit_words(const Vec<u8> &bytestream,
                              usize &pos);


void u32_to_le_bytes(const u32 value, 
                     Vec<u8> &bytestream,
                     usize &pos);

}


#endif
