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

Vec<u8> CommandPacket::to_bytestream() {
  Vec<u8> buffer = std::vector<u8>(p_length_fixed );
  usize pos    = 0;
  encode_ushort(head, buffer, pos); pos+=2;
  u8 cmd_class = (u8)command; 
  buffer[2] = cmd_class; pos += 1; 
  u32_to_le_bytes(value, buffer, pos); pos+=4;
  encode_ushort(tail, buffer, pos);  pos+=2;
  return buffer;
}


usize CommandPacket::from_bytestream(Vec<u8> &payload,
                                     usize start_pos) {

  auto pos = start_pos;
  u16 header = decode_ushort(payload, pos);
  if (header != head) {
    std::cerr << "[WARN] no header found!" << std::endl; 
  } 
  pos += 2;
  command        = static_cast<TofCommand>(payload[2])       ; pos += 1;
  value          = u32_from_le_bytes(payload, pos); pos += 4;
  u16 tail_flag  = decode_ushort(payload, pos); pos += 2;
  if (tail_flag != tail) {
    std::cerr << "[WARN] no tail found!" << std::endl; 
  } 
  return pos;
}


//  The package layout in binary is like this
//  HEAD         : u16 = 0xAAAA
//  CommnadClass : u8
//  DATA         : u32
//  TAIL         : u16 = 0x5555
ResponsePacket::ResponsePacket(const TofResponse &resp,
                               const u32 val) {
  response = resp;
  value   = val;
}

Vec<u8> ResponsePacket::to_bytestream() const {
  Vec<u8> buffer = std::vector<u8>(p_length_fixed );
  usize pos    = 0;
  encode_ushort(head, buffer, pos); pos+=2;
  u8 resp_class = (u8)response; 
  buffer[2] = resp_class; pos += 1; 
  u32_to_le_bytes(value, buffer, pos); pos+=4;
  encode_ushort(tail, buffer, pos);  pos+=2;
  return buffer;
}

std::string ResponsePacket::translate_response_code(u32 code) const {
  switch (code) {
    case ResponsePacket::RESP_ERR_LEVEL_NOPROBLEM        : { return "RESP_ERR_LEVEL_NOPROBLEM";};
    case ResponsePacket::RESP_ERR_LEVEL_MEDIUM           : { return "RESP_ERR_LEVEL_MEDIUM";};
    case ResponsePacket::RESP_ERR_LEVEL_CRITICAL         : { return "RESP_ERR_LEVEL_CRITICAL";};
    case ResponsePacket::RESP_ERR_LEVEL_MISSION_CRITICAL : { return "RESP_ERR_LEVEL_MISSION_CRITICAL";};
    case ResponsePacket::RESP_ERR_LEVEL_RUN_FOOL_RUN     : { return "RESP_ERR_LEVEL_RUN_FOOL_RUN";};
    case ResponsePacket::RESP_ERR_LEVEL_SEVERE           : { return "RESP_ERR_LEVEL_SEVERE";};
    case ResponsePacket::RESP_ERR_NORUNACTIVE            : { return "RESP_ERR_NORUNACTIVE";};
    case ResponsePacket::RESP_ERR_NOTIMPLEMENTED         : { return "RESP_ERR_NOTIMPLEMENTED";};
    case ResponsePacket::RESP_ERR_RUNACTIVE              : { return "RESP_ERR_RUNACTIVE";};
    case ResponsePacket::RESP_ERR_UNEXECUTABLE           : { return "RESP_ERR_UNEXECUTABLE";};
    case ResponsePacket::RESP_SUCC_FINGERS_CROSSED       : { return "RESP_SUCC_FINGERS_CROSSED";};
  }
  return "UNKNOWN RESPONSE CODE " + std::to_string(code);
}

usize ResponsePacket::from_bytestream(Vec<u8> &payload,
                                      usize start_pos) {

  auto pos = start_pos;
  u16 header = decode_ushort(payload, pos);
  if (header != head) {
    std::cerr << "[WARN] no header found!" << std::endl; 
  } 
  pos += 2;
  response       = static_cast<TofResponse>(payload[2])       ; pos += 1;
  value          = decode_uint32(payload, pos); pos += 4;
  u16 tail_flag  = decode_ushort(payload, pos); pos += 2;
  if (tail_flag != tail) {
    std::cerr << "[WARN] no tail found!" << std::endl; 
  } 
  return pos;
}


