#include <fstream>
#include "spdlog/spdlog.h"
#include "spdlog/cfg/env.h"

#include "tof_typedefs.h"
#include "parsers.h"
#include "logging.hpp"
#include "serialization.h"

// file i/o
bytestream get_bytestream_from_file(const String &filename) {
  spdlog::cfg::load_env_levels();
  // bytestream stream;
  // Not going to explicitly check these.
  // // The use of gcount() below will compensate for a failure here.
  std::ifstream is(filename, std::ios::binary);

  is.seekg (0, is.end);
  u64 length = is.tellg();
  is.seekg (0, is.beg);
  log_debug("Read " << length << " bytes from " << filename << "!");
  bytestream stream = bytestream(length);
  is.read(reinterpret_cast<char*>(stream.data()), length);
  return stream;
}

u64 search_for_2byte_marker(const Vec<u8> &bytestream,
                            u8 marker,
                            bool &has_ended,
                            u64 start_pos,
                            u64 end_pos) {
  has_ended = false;
  if ((end_pos == bytestream.size()) || (end_pos == 0)) 
    { end_pos = bytestream.size() - 1;} 
  if (start_pos >= end_pos) {
    has_ended = true;
    spdlog::warn("Start and end positions are invalid! Start pos {}, end pos {}", start_pos, end_pos);
    return 0;
  }
  for (u64 k=start_pos; k<end_pos; k++) { 
      if ((bytestream[k] == marker) && (bytestream[k+1] == marker))  {
        //std::cout << "endpos " << end_pos << std::endl;
        //std::cout << "Found marker at pos " << k << " " << bytestream[k] << std::endl;
        return k;
      }
    }
  has_ended = true;
  return 0;
}

/***********************************************/

Vec<u32> get_2byte_markers_indices(const Vec<u8> &bytestream, uint8_t marker)
{
  Vec<u32> indices;
  for (size_t k=0; k<bytestream.size() -1; k++)
    { 
      if ((bytestream[k] == marker) && (bytestream[k+1] == marker)) 
        { indices.push_back(k);}
    }
  return indices;
}

/***********************************************/


