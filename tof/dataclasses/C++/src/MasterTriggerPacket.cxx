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


Vec<u8> MasterTriggerPacket::serialize() const {
  return to_bytestream();
}

u64 MasterTriggerPacket::deserialize(Vec<u8> &payload,
                                     u64 start_pos) {
  return from_bytestream(payload, start_pos);
}

Vec<u8> MasterTriggerPacket::to_bytestream() const {
  Vec<u8> bytestream;
  return bytestream;
}

u64 MasterTriggerPacket::from_bytestream(Vec<u8> &payload, 
                                         u64 start_pos) {
  u64 pos          = start_pos; 
  u16 header       = Gaps::u16_from_le_bytes(payload, pos);
  if (header != head) {
    spdlog::error("No header signature found!");  
  }
  event_id         = Gaps::parse_u32(payload, pos); 
  timestamp        = Gaps::parse_u32(payload, pos); 
  tiu_timestamp    = Gaps::parse_u32(payload, pos); 
  gps_timestamp_32 = Gaps::parse_u32(payload, pos); 
  gps_timestamp_16 = Gaps::parse_u32(payload, pos); 
  n_paddles        = Gaps::parse_u8(payload, pos);
  board_mask       = Gaps::parse_u32(payload, pos); 
  //idecoded_board_mask = [false;32];
  //hits         = [[false;32];32];

  //u32 crc;
  //u64 end_pos;
  return pos;
}

Vec<u8> MasterTriggerPacket::get_hit_board_ids() const {
  Vec<u8> board_ids;
  spdlog::error("Not implemented yet!");
  return board_ids;
}

Vec<u8> MasterTriggerPacket::get_hit_paddle_ids() const {
  Vec<u8> paddle_ids;
  spdlog::error("Not implemented yet!");
  return paddle_ids;
}
