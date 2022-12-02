#ifndef GAPSTOFTYPEDEFS_H_INCLUDED
#define GAPSTOFTYPEDEFS_H_INCLUDED

#include <vector>

namespace GAPS {

  typedef unsigned char  u8;
  typedef unsigned short u16;
  typedef uint32_t       u32;
  typedef unsigned long  u64;

  typedef short       i16;
  typedef int         i32;
  typedef long        i64;

  typedef float       f32;
  typedef double      f64;

  // vectors
  typedef std::vector<u8>  bytestream; // this is used for (de)serialization
  typedef std::vector<u16> vec_u16;
  typedef std::vector<u64> vec_u64;
  typedef std::vector<f64> vec_f64;
  typedef std::vector<std::vector<f64>> vec_vec_f64;
  typedef std::vector<std::vector<i16>> vec_vec_i16;


}

#endif
