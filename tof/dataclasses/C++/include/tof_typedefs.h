#ifndef GAPSTOFTYPEDEFS_H_INCLUDED
#define GAPSTOFTYPEDEFS_H_INCLUDED

#include <vector>
#include <map>
#include <cstddef>
#include <string>
#include <cstdint>

/******************************************
 * Basic typedefs to be used with all tof
 * related code
 *
 *
 */

typedef uint8_t  u8;
typedef uint16_t u16;
typedef uint32_t u32;
typedef uint64_t u64;
typedef int8_t   i8;
typedef int16_t  i16;
typedef int32_t  i32;
typedef int64_t  i64;
typedef size_t   usize;
typedef float    f32;
typedef double   f64;
typedef std::string String;

template <typename T>
using Vec = std::vector<T>;

template <typename T, typename U>
using HashMap = std::map<T,U>;

typedef Vec<u8> bytestream;

// vectors
typedef std::vector<u8>  vec_u8; // this is used for (de)serialization
typedef vec_u8 bytestream;
typedef std::vector<u16> vec_u16;
typedef std::vector<i16> vec_i16;
typedef std::vector<u32> vec_u32;
typedef std::vector<u64> vec_u64;
typedef std::vector<f32> vec_f32;
typedef std::vector<f64> vec_f64;
typedef std::vector<vec_f32> vec_vec_f32;
typedef std::vector<vec_f64> vec_vec_f64;
typedef std::vector<vec_i16> vec_vec_i16;
typedef std::vector<vec_u16> vec_vec_u16;

#endif
