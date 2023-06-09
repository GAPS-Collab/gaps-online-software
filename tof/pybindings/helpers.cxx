#include "tof_typedefs.h"
#include "serialization.h"

bytestream wrap_encode_ushort(u16 value, u32 start_pos) {
  bytestream stream;
  for (size_t foo=0; foo<2; foo++) stream.push_back(0);
  encode_ushort(value, stream, start_pos);
  return stream;
}

/***********************************************/

bytestream wrap_encode_ushort_rev(u16 value, size_t start_pos) {
  bytestream stream;
  for (size_t foo=0; foo<2; foo++) stream.push_back(0);
  encode_ushort_rev(value, stream, start_pos);
  return stream;
}


/***********************************************/

bytestream wrap_u32_to_le_bytes(u32 value) {
  bytestream stream;
  for (size_t foo=0; foo<4; foo++) stream.push_back(0);
  u32_to_le_bytes(value, stream, 0);
  return stream;
}

/***********************************************/

bytestream wrap_encode_uint32(u32 value, size_t start_pos) {
  bytestream stream;
  for (size_t foo=0; foo<4; foo++) stream.push_back(0);
  encode_uint32(value, stream, start_pos);
  return stream;
}

/***********************************************/

bytestream wrap_encode_uint32_rev(u32 value, size_t start_pos) {
  bytestream stream;
  for (size_t foo=0; foo<4; foo++) stream.push_back(0);
  encode_uint32_rev(value, stream, start_pos);
  return stream;
}

/***********************************************/

bytestream wrap_encode_uint64_rev(u64 value, size_t start_pos) {
  bytestream stream;
  for (size_t foo=0; foo<8; foo++) stream.push_back(0);
  encode_uint64_rev(value, stream, start_pos);
  return stream;
}

