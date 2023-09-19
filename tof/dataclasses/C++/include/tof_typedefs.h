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
#endif
