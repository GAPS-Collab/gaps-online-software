/**
 * Typedefs for tof relevant code.
 *
 * Rationale: Make sure that numeric types
 *            have the same size on different
 *            systems, which ir relevant for 
 *            (de)serialization.
 *
 * Bonus: Match rust syntax a bit more closely,
 *        so it is easier to compater to its
 *        Rust counterpart
 *
 */
#ifndef GAPSTOFTYPEDEFS_H_INCLUDED
#define GAPSTOFTYPEDEFS_H_INCLUDED

#include <vector>
#include <map>
#include <cstddef>
#include <string>
#include <cstdint>

typedef uint8_t   u8;
typedef uint16_t  u16;
typedef uint32_t  u32;
typedef uint64_t  u64;
typedef int8_t    i8;
typedef int16_t   i16;
typedef int32_t   i32;
typedef int64_t   i64;
typedef size_t    usize;
///FIXME - get the correct type for float
typedef float     f32;
///FIXME - get the correct type for double
typedef double    f64;
typedef std::string String;

/// Define vectors the same as in Rust
template <typename T>
using Vec = std::vector<T>;

/// Define std::map the same as HashMap in Rust
template <typename T, typename U>
using HashMap = std::map<T,U>;

typedef HashMap<u8,HashMap<u8,HashMap<u8, std::pair<u8,u8>>>> LtbRBMap;

typedef Vec<u8> bytestream;
#endif
