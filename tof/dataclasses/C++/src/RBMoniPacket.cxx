#include "packets/RBMoniPacket.h"

#include "serialization.h"

vec_u8 RBMoniPacket::to_bytestream() const {
  vec_u8 buffer = std::vector<u8>(SIZE);
  usize pos    = 0;
  encode_ushort(HEAD, buffer, pos); pos+=2;
  u32_to_le_bytes(rate, buffer, pos); pos+=4;
  encode_ushort(TAIL, buffer, pos);  pos+=2;
  return buffer;
}


usize RBMoniPacket::from_bytestream(vec_u8 &payload,
                                     usize start_pos) {

  auto pos = start_pos;
  u16 header = decode_ushort(payload, pos);
  if (header != HEAD) {
    std::cerr << "[WARN] no header found!" << std::endl; 
  } 
  pos += 2;
  rate          = u32_from_le_bytes(payload, pos); pos += 4;
  u16 tail_flag  = decode_ushort(payload, pos); pos += 2;
  if (tail_flag != TAIL) {
    std::cerr << "[WARN] no tail found!" << std::endl; 
  } 
  return pos;
}

