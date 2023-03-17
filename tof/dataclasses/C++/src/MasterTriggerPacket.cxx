#include "packets/MasterTriggerPacket.h"
#include "serialization.h"
#include "parsers.h"
#include "spdlog/spdlog.h"

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
  u64 pos          = start_pos; 
  u16 header       = Gaps::u16_from_le_bytes(payload, pos);
  if (header != head)  {
    spdlog::error("No header signature found!");  
  }
  event_id         = Gaps::u32_from_le_bytes(payload, pos); 
  timestamp        = Gaps::u32_from_le_bytes(payload, pos); 
  tiu_timestamp    = Gaps::u32_from_le_bytes(payload, pos); 
  gps_timestamp_32 = Gaps::u32_from_le_bytes(payload, pos); 
  gps_timestamp_16 = Gaps::u32_from_le_bytes(payload, pos); 
  n_paddles        = payload[pos]; pos+= 1;
  board_mask       = Gaps::u32_from_le_bytes(payload, pos); 
  //idecoded_board_mask = [false;32];
  //hits         = [[false;32];32];

  //u32 crc;
  //u64 end_pos;
  return pos;
}

vec_u8 MasterTriggerPacket::get_hit_board_ids() const {
  vec_u8 board_ids;
  spdlog::error("Not implemented yet!");
  return board_ids;
}

vec_u8 MasterTriggerPacket::get_hit_paddle_ids() const {
  vec_u8 paddle_ids;
  spdlog::error("Not implemented yet!");
  return paddle_ids;
}
