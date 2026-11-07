// This file is part of gaps-online-software and published 
// under the GPLv3 license


#include "events/mc_hit.hpp"
#include "io.hpp"
#include "io/parsers.h"
#include <format>

namespace go = gondola;
using namespace result;

#ifdef BUILD_SIM

auto go::McHit::to_bytestream() const -> Vec<u8> {
  Vec<u8> stream = {};
  //stream.insert()
  auto bytes = go::to_le_bytes((u16)0xAAAA);
  stream.insert(stream.end(), bytes.begin(), bytes.end());
  bytes = go::to_le_bytes((u32)volume_id);
  stream.insert(stream.end(), bytes.begin(), bytes.end());
  bytes = go::to_le_bytes((u32)hw_id);
  stream.insert(stream.end(), bytes.begin(), bytes.end());
  bytes = go::to_le_bytes((u32)parent_id);
  stream.insert(stream.end(), bytes.begin(), bytes.end());
  bytes = go::to_le_bytes((u32)track_id);
  stream.insert(stream.end(), bytes.begin(), bytes.end());
  bytes = go::to_le_bytes((f32)kin_E);
  stream.insert(stream.end(), bytes.begin(), bytes.end());
  bytes = go::to_le_bytes((f32)glob_time);
  stream.insert(stream.end(), bytes.begin(), bytes.end());
  bytes = go::to_le_bytes((f32)pos_x);
  stream.insert(stream.end(), bytes.begin(), bytes.end());
  bytes = go::to_le_bytes((f32)pos_y);
  stream.insert(stream.end(), bytes.begin(), bytes.end());
  bytes = go::to_le_bytes((f32)pos_z);
  stream.insert(stream.end(), bytes.begin(), bytes.end());
  bytes = go::to_le_bytes((f32)vertex_pos_x);
  stream.insert(stream.end(), bytes.begin(), bytes.end());
  bytes = go::to_le_bytes((f32)vertex_pos_y);
  stream.insert(stream.end(), bytes.begin(), bytes.end());
  bytes = go::to_le_bytes((f32)vertex_pos_z);
  stream.insert(stream.end(), bytes.begin(), bytes.end());
  bytes = go::to_le_bytes((f32)vertex_kin_E);
  stream.insert(stream.end(), bytes.begin(), bytes.end());
  bytes = go::to_le_bytes((f32)mom_x);
  stream.insert(stream.end(), bytes.begin(), bytes.end());
  bytes = go::to_le_bytes((f32)mom_y);
  stream.insert(stream.end(), bytes.begin(), bytes.end());
  bytes = go::to_le_bytes((f32)mom_z);
  stream.insert(stream.end(), bytes.begin(), bytes.end());
  bytes = go::to_le_bytes((f32)vertex_mom_x);
  stream.insert(stream.end(), bytes.begin(), bytes.end());
  bytes = go::to_le_bytes((f32)vertex_mom_y);
  stream.insert(stream.end(), bytes.begin(), bytes.end());
  bytes = go::to_le_bytes((f32)vertex_mom_z);
  stream.insert(stream.end(), bytes.begin(), bytes.end());
  bytes = go::to_le_bytes((f32)step_len);
  stream.insert(stream.end(), bytes.begin(), bytes.end());
  bytes = go::to_le_bytes((f32)step_edep);
  stream.insert(stream.end(), bytes.begin(), bytes.end());
  bytes = go::to_le_bytes((f32)pre_mom_x);
  stream.insert(stream.end(), bytes.begin(), bytes.end());
  bytes = go::to_le_bytes((f32)pre_mom_y);
  stream.insert(stream.end(), bytes.begin(), bytes.end());
  bytes = go::to_le_bytes((f32)pre_mom_z);
  stream.insert(stream.end(), bytes.begin(), bytes.end());
  bytes = go::to_le_bytes((f32)pre_kin_E);
  stream.insert(stream.end(), bytes.begin(), bytes.end());
  // new stuff 
  bytes = go::to_le_bytes((i32)pdg);  
  stream.insert(stream.end(), bytes.begin(), bytes.end());
  bytes = go::to_le_bytes((i32)pre_step_status);  
  stream.insert(stream.end(), bytes.begin(), bytes.end());
  bytes = go::to_le_bytes((i32)post_step_status);  
  stream.insert(stream.end(), bytes.begin(), bytes.end());
  bytes = go::to_le_bytes((u32)vertex_vol_id);  
  stream.insert(stream.end(), bytes.begin(), bytes.end());
  bytes = go::to_le_bytes((u32)vertex_hw_id);  
  stream.insert(stream.end(), bytes.begin(), bytes.end());
  stream.push_back((u8)is_first_step_in_vol);
  stream.push_back((u8)is_last_step_in_vol);
  stream.push_back((u8)process_type);
  bytes = go::to_le_bytes((u16)0x5555);
  stream.insert(stream.end(), bytes.begin(), bytes.end());
  return stream;
}
#endif

//-------------------------------------------

auto go::McHit::from_bytestream(const Vec<u8> &stream, usize &pos)
  -> result::Result<go::McHit, go::IOError> {
  if (stream.size() < pos + McHit::SIZE) {
    auto message = std::format("Stream too short for McHit! {} bytes left", stream.size() - pos);
    return Err(go::IOError(go::IOError::ErrorKind::StreamTooShort, message));
  }
  u16 head = go::parse_u16(stream, pos);
  if (head != McHit::HEAD) {
    auto message = std::format("Decoding of HEAD failed! Got {} instead!", head);
    return Err(go::IOError(go::IOError::ErrorKind::WrongHeaderBytes, message));
  }
  auto hit = McHit();
  hit.volume_id            = go::parse_u32(stream, pos);
  hit.hw_id                = go::parse_u32(stream, pos);
  hit.parent_id            = go::parse_u32(stream, pos);
  hit.track_id             = go::parse_u32(stream, pos);
  hit.kin_E                = go::parse_f32(stream, pos);
  hit.glob_time            = go::parse_f32(stream, pos);
  hit.pos_x                = go::parse_f32(stream, pos);
  hit.pos_y                = go::parse_f32(stream, pos);
  hit.pos_z                = go::parse_f32(stream, pos);
  hit.vertex_pos_x         = go::parse_f32(stream, pos);
  hit.vertex_pos_y         = go::parse_f32(stream, pos);
  hit.vertex_pos_z         = go::parse_f32(stream, pos);
  hit.vertex_kin_E         = go::parse_f32(stream, pos);
  hit.mom_x                = go::parse_f32(stream, pos);
  hit.mom_y                = go::parse_f32(stream, pos);
  hit.mom_z                = go::parse_f32(stream, pos);
  hit.vertex_mom_x         = go::parse_f32(stream, pos);
  hit.vertex_mom_y         = go::parse_f32(stream, pos);
  hit.vertex_mom_z         = go::parse_f32(stream, pos);
  hit.step_len             = go::parse_f32(stream, pos);
  hit.step_edep            = go::parse_f32(stream, pos);
  hit.pre_mom_x            = go::parse_f32(stream, pos);
  hit.pre_mom_y            = go::parse_f32(stream, pos);
  hit.pre_mom_z            = go::parse_f32(stream, pos);
  hit.pre_kin_E            = go::parse_f32(stream, pos);
  hit.pdg                  = go::parse_i32(stream, pos);
  hit.pre_step_status      = go::parse_i32(stream, pos);
  hit.post_step_status     = go::parse_i32(stream, pos);
  hit.vertex_vol_id        = go::parse_u32(stream, pos);
  hit.vertex_hw_id         = go::parse_u32(stream, pos);
  hit.is_first_step_in_vol = go::parse_bool(stream, pos);
  hit.is_last_step_in_vol  = go::parse_bool(stream, pos);
  hit.process_type         = go::parse_u8(stream, pos);
  u16 tail = go::parse_u16(stream, pos);
  if (tail != McHit::TAIL) {
    auto message = std::format("Decoding of TAIL failed! Got {} instead!", tail);
    return Err(go::IOError(go::IOError::ErrorKind::WrongTailBytes, message));
  }
  return Ok(hit);
}
