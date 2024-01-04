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


MasterTriggerPacket MasterTriggerPacket::from_bytestream(Vec<u8> &payload, 
                                                         u64 &pos) {
  auto mte         = MasterTriggerPacket();
  u16 header       = Gaps::u16_from_le_bytes(payload, pos);
  if (header != MasterTriggerPacket::HEAD) {
    spdlog::error("No header signature found!");  
  }
  mte.event_id         = Gaps::parse_u32(payload, pos); 
  mte.timestamp        = Gaps::parse_u32(payload, pos); 
  mte.tiu_timestamp    = Gaps::parse_u32(payload, pos); 
  mte.gps_timestamp_32 = Gaps::parse_u32(payload, pos); 
  mte.gps_timestamp_16 = Gaps::parse_u32(payload, pos); 
  mte.n_paddles        = Gaps::parse_u8(payload, pos);
  mte.board_mask       = Gaps::parse_u32(payload, pos); 
  return mte;
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
