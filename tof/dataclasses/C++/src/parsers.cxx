#include <cstring>
#include <iostream>
#include "parsers.h"

bool Gaps::parse_bool(const Vec<u8> &bytestream,
                      usize &pos) {
  u8 value = bytestream[pos];
  pos += 1;
  return value > 0;
}

/***********************************************/

u8 Gaps::parse_u8(const Vec<u8> &bytestream,
                  u64 &pos) {
  u8 value = bytestream[pos];
  pos += 1;
  return value;
}

/***********************************************/

u16 Gaps::parse_u16(const Vec<u8> &bytestream,
                    u64 &pos) {
  u16 value = (u16)(
        ((bytestream[pos+1] & 0xFF) << 8)
      |  (bytestream[pos]));
  pos += 2;
  return value;
}

/***********************************************/

u32 leading_zeros(u16 x) {
  u32 c = 0;
  u16 msb = 1 << 15;
  for (u8 k=0;k<16;k++) {
    if ((x & msb) == 0) {
      c += 1;
    } else {
      return c;
    }
    if (k < 15) {
      x <<= 1;
    }
  }
  return c;
}

//lil' helpa
f32 u32tof32(u32 val) {
  f32 result;
  Vec<u8> bytes = Vec<u8>();
  bytes.push_back(0);
  bytes.push_back(0);
  bytes.push_back(0);
  bytes.push_back(0);
  bytes[3] = (val >> 24) & 0xFF;
  bytes[2] = (val >> 16) & 0xFF;
  bytes[1] = (val >> 8)  & 0xFF;
  bytes[0] =  val & 0xFF;
  std::memcpy(&result, bytes.data(), sizeof(f32));
  return result;
}

/***********************************************/


f32 Gaps::parse_f16(const Vec<u8> &bytestream,
                    usize &pos) {
  u16 bits = Gaps::parse_u16(bytestream, pos);
  //  // Check for signed zero
  //  // TODO: Replace mem::transmute with from_bits() once from_bits is const-stabilized
  f32 result;
  //Vec<u8> bytes = Gaps::slice(bytestream,pos,pos+2); 
  // Copy the bytes into a float variable using type punning
  if ((bits & 0x7FFF) == 0) {
    u32 bits_u32 = (u32)bits << 16;
    return u32tof32(bits_u32);
    return result;
  }

  u32 half_sign = (u32)(bits & 0x8000);
  u32 half_exp  = (u32)(bits & 0x7C00);
  u32 half_man  = (u32)(bits & 0x03FF);
  //  // Check for an infinity or NaN when all exponent bits set
  if (half_exp == 0x7C00) {
    // Check for signed infinity if mantissa is zero
    if (half_man == 0) {
      u32 bits_u32 = half_sign << 16 | 0x7f800000;
      return u32tof32(bits_u32);
        //return unsafe { mem::transmute::<u32, f32>((half_sign << 16) | 0x7F80_0000u32) };
    } else {
        // NaN, keep current mantissa but also set most significiant mantissa bit
        //return unsafe {
        //    mem::transmute::<u32, f32>((half_sign << 16) | 0x7FC0_0000u32 | (half_man << 13))
        //};
      u32 bits_u32 = (half_sign << 16) | 0x7fc00000 | (half_man << 13);
      return u32tof32(bits_u32);
    }
  }

  //  // Calculate single-precision components with adjusted exponent
  u32 sign = half_sign << 16;
  //  // Unbias exponent
  i32 unbiased_exp = ((i32)half_exp >> 10) - 15;

  // Check for subnormals, which will be normalized by adjusting exponent
  if (half_exp == 0) {
    // Calculate how much to adjust the exponent by
    //let e = leading_zeros_u16(half_man as u16) - 6;
    u16 e = leading_zeros((u16)half_man) - 6;
    //// Rebias and adjust exponent
    u32 exp = (127 - 15 - e) << 23;
    u32 man = (half_man << (14 + e)) & (u32)0x7FFFFF;
    u32 bits_u32 = sign | exp | man;
    return u32tof32(bits_u32);
    //return unsafe { mem::transmute::<u32, f32>(sign | exp | man) };
  }

  // Rebias exponent for a normalized normal
  u32 exp = (u32)(unbiased_exp + 127) << 23;
  u32 man = (half_man & 0x03FF) << 13;
  //  unsafe { mem::transmute::<u32, f32>(sign | exp | man) }
  u32 bits_u32 = sign | exp | man;
  return u32tof32(bits_u32);
}

/***********************************************/

u32 Gaps::parse_u32(const Vec<u8> &bytestream,
                    u64 &pos) {
  u32 value = (u32)(
         ((u32)(bytestream[pos+3]) << 24)
      |  ((u32)(bytestream[pos+2]) << 16)
      |  ((u32)(bytestream[pos+1]) << 8)
      |   (u32)(bytestream[pos+0]));
  pos += 4;
  return value;
}

/***********************************************/

u64 Gaps::parse_u64(const Vec<u8> &bytestream,
                    usize &pos) {
  u64 value = (u64)(
         ((u64)(bytestream[pos+7]) << 56)
      |  ((u64)(bytestream[pos+6]) << 48)
      |  ((u64)(bytestream[pos+5]) << 40)
      |  ((u64)(bytestream[pos+4]) << 32)
      |  ((u64)(bytestream[pos+3]) << 24)
      |  ((u64)(bytestream[pos+2]) << 16)
      |  ((u64)(bytestream[pos+1]) << 8)
      |   (u64)(bytestream[pos+0]));
  pos += 8;
  return value;
}

/***********************************************/

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

/***********************************************/

f32 Gaps::parse_f32(const Vec<u8> &bytestream,
                    usize &pos) {
  f32 result;
  Vec<u8> bytes = Gaps::slice(bytestream,pos,pos+4); 
  // Copy the bytes into a float variable using type punning
  std::memcpy(&result, bytes.data(), sizeof(f32));
  pos += 4;
  return result;
}

/***********************************************/

f64 Gaps::parse_f64(const Vec<u8> &bytestream,
                    usize &pos) {
  f64 result;
  Vec<u8> bytes = Gaps::slice(bytestream,pos,pos+8); 
  // Copy the bytes into a float variable using type punning
  std::memcpy(&result, bytes.data(), sizeof(f64));
  pos += 8;
  return result;
}

/***********************************************/
