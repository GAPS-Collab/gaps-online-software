#include "packets/CommandPacket.h"
#include "serialization.h"
 
//  The package layout in binary is like this
//  HEAD         : u16 = 0xAAAA
//  CommnadClass : u8
//  DATA         : u32
//  TAIL         : u16 = 0x5555
CommandPacket::CommandPacket(const TofCommand &cmd,
                             const u32 val) {
  command = cmd;
  value   = val;
}

vec_u8 CommandPacket::to_bytestream() {
  vec_u8 buffer = std::vector<u8>(p_length_fixed );
  usize pos    = 0;
  encode_ushort(head, buffer, pos); pos+=2;
  u8 cmd_class = (u8)command; 
  buffer[2] = cmd_class; pos += 1; 
  encode_uint32(value, buffer, pos); pos+=4;
  encode_ushort(tail, buffer, pos);  pos+=2;
  return buffer;
}


usize CommandPacket::from_bytestream(vec_u8 &payload,
                                     usize start_pos) {

  auto pos = start_pos;
  u16 header = decode_ushort(payload, pos);
  if (header != head) {
    std::cerr << "[WARN] no header found!" << std::endl; 
  } 
  pos += 2;
  command        = static_cast<TofCommand>(payload[2])       ; pos += 1;
  value          = decode_uint32(payload, pos); pos += 4;
  u16 tail_flag  = decode_ushort(payload, pos); pos += 2;
  if (tail_flag != tail) {
    std::cerr << "[WARN] no tail found!" << std::endl; 
  } 
  return pos;
}



