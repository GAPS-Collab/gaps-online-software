#include "packets/MasterTriggerPacket.h"
#include "serialization.h"

void MasterTriggerPacket::reset() {
}

std::string MasterTriggerPacket::to_string() const {

  std::string repr = "";
  return repr;
}


vec_u8 MasterTriggerPacket::serialize() const {
  return to_bytestream();
}

u64 MasterTriggerPacket::deserialize(vec_u8 &payload,
                                     u64 start_pos) {
  return from_bytestream(payload, start_pos);
}

vec_u8 MasterTriggerPacket::to_bytestream() const {
  vec_u8 bytestream;
  return bytestream;
}

u64 MasterTriggerPacket::from_bytestream(vec_u8 &payload, 
                                         u64 start_pos) {
  u64 pos = start_pos; 
  event_id         = u32_from_le_bytes(payload, pos); pos += 4 ; 
  timestamp        = u32_from_le_bytes(payload, pos); pos += 4 ; 
  tiu_timestamp    = u32_from_le_bytes(payload, pos); pos += 4 ; 
  gps_timestamp_32 = u32_from_le_bytes(payload, pos); pos += 4 ; 
  gps_timestamp_16 = u32_from_le_bytes(payload, pos); pos += 4 ; 
  board_mask       = u32_from_le_bytes(payload, pos); pos += 4 ;
  //idecoded_board_mask = [false;32];
  //hits         = [[false;32];32];

  //u32 crc;
  //u64 end_pos;
  return pos;
}

