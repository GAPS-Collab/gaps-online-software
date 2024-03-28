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
  
  bool parse_bool(const Vec<u8> &bytestream,
                  usize &pos);
  
  /// get an unsigned char from a vector of bytes, advancing pos by 1
  u8 parse_u8(const Vec<u8> &bytestream,
              usize &pos);
  
  /// get an unsigned short from a vector of bytes, advancing pos by 2
  u16 parse_u16(const Vec<u8> &bytestream,
                usize &pos);
  
  /// get an unsigned 32bit int from a vector of bytes, advancing pos by 4
  u32 parse_u32(const Vec<u8> &bytestream,
                usize &pos);
  
  /// get an unsigned long64 from a vector of bytes, advancing pos by 8
  u64 parse_u64(const Vec<u8> &bytestream,
                usize &pos);
  
  /// get a signed 32bit int from a vector of bytes, advancing pos by 4
  i32 parse_i32(const Vec<u8> &bytestream,
                usize &pos);
  
  /// get a signed float32 from a vector of bytes, advancning pos by 4
  f32 parse_f32(const Vec<u8> &bytestream,
                usize &pos);
  
  /// get a signed long float (64) from a vector of bytes, advancing pos by 8
  f64 parse_f64(const Vec<u8> &bytestream,
                usize &pos);
  
}
#endif
