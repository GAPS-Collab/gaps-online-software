#include <fstream>
#include "spdlog/spdlog.h"

#include "tof_typedefs.h"
#include "parsers.h"

#include "serialization.h"

uint64_t decode_uint64_rev(const Vec<u8>& bytestream,
                           u32 start_pos)
{
  uint64_t buffer64 = 0x0000000000000000;

  //unsigned long long value = (unsigned long long)(
  uint64_t buffer =  
         ((bytestream[start_pos+1] & 0xFF | buffer64) << 56)
      |  ((bytestream[start_pos+0] & 0xFF | buffer64) << 48)
      |  ((bytestream[start_pos+3] & 0xFF | buffer64) << 40)
      |  ((bytestream[start_pos+2] & 0xFF | buffer64) << 32)
      |  ((bytestream[start_pos+5] & 0xFF | buffer64) << 24)
      |  ((bytestream[start_pos+4] & 0xFF | buffer64) << 16)
      |  ((bytestream[start_pos+7] & 0xFF | buffer64) << 8)
      |  (bytestream[start_pos+6]);

  return buffer;
  
}

// file i/o

bytestream get_bytestream_from_file(const String &filename) {
  // bytestream stream;
  // Not going to explicitly check these.
  // // The use of gcount() below will compensate for a failure here.
  std::ifstream is(filename, std::ios::binary);

  is.seekg (0, is.end);
  u64 length = is.tellg();
  is.seekg (0, is.beg);
  spdlog::debug("Read {} bytes from file!", length);
  bytestream stream = bytestream(length);
  is.read(reinterpret_cast<char*>(stream.data()), length);
  return stream;
}

u64 search_for_2byte_marker(const Vec<u8> &bytestream,
                            u8 marker,
                            bool &has_ended,
                            u64 start_pos,
                            u64 end_pos)
{
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


