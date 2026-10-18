#include "mc_hit.hpp"
#include "io.hpp"
#include "io/parsers.h"
namespace go = gondola;

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

