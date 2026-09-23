// This file is part of gaps-online-software and published 
// under the GPLv3 license

#include "io/parsers.h"
#include "serialization.h"
#include "version.h"

#include "spdlog/spdlog.h"
#include "spdlog/cfg/env.h"

#include "events/tof_event_summary.hpp"

const u16 LTB_CH0 = 0x3   ;
const u16 LTB_CH1 = 0xc   ;
const u16 LTB_CH2 = 0x30  ; 
const u16 LTB_CH3 = 0xc0  ;
const u16 LTB_CH4 = 0x300 ;
const u16 LTB_CH5 = 0xc00 ;
const u16 LTB_CH6 = 0x3000;
const u16 LTB_CH7 = 0xc000;
const u16 LTB_CHANNELS[8] = {
    LTB_CH0,
    LTB_CH1,
    LTB_CH2,
    LTB_CH3,
    LTB_CH4,
    LTB_CH5,
    LTB_CH6,
    LTB_CH7
};

namespace g = gondola;
using namespace result;

auto g::TofEventSummary::set_event_status(u8 int_status) -> void {
  status = static_cast<EventStatus>(int_status);
}

auto g::TofEventSummary::get_event_status() const -> u8 {
  return static_cast<u8>(status);
}

auto g::TofEventSummary::get_trigger_sources() const -> Vec<g::TriggerType> {
  auto t_types = Vec<g::TriggerType>();
  u16 gaps_trigger = (trigger_sources >> 5 & 0x1) == 1;
  if (gaps_trigger) {
    t_types.push_back(g::TriggerType::Gaps);
  }
  u16 any_trigger    = (trigger_sources >> 6 & 0x1) == 1;
  if (any_trigger) {
    t_types.push_back(g::TriggerType::Any);
  }
  u16 forced_trigger = (trigger_sources >> 7 & 0x1) == 1;
  if (forced_trigger) {
    t_types.push_back(g::TriggerType::Forced);
  }
  u16 track_trigger  = (trigger_sources >> 8 & 0x1) == 1;
  if (track_trigger) {
    t_types.push_back(g::TriggerType::Track);
  }
  u16 central_track_trigger
                     = (trigger_sources >> 9 & 0x1) == 1;
  if (central_track_trigger) {
    t_types.push_back(g::TriggerType::TrackCentral);
  }
  return t_types;
} 

#ifdef BUILD_CXX_DB
auto g::TofEventSummary::normalize_hit_times(const g::TofPaddleTimingConstantMap &offsets) -> void {
  if (hits.size() == 0) {
    return;
  }
  auto PI = std::numbers::pi_v<f32>;
  bool first_phase = false;
  f32 phase0 = 0;
  for (auto &h : hits) {
    if (!first_phase) {
      phase0 = h.phase;
      first_phase = true;
    }
    auto t0 = h.get_t0_relative() + h.get_cable_delay();
    auto phase_diff = h.phase - phase0;
    while (phase_diff < - PI/2.0) {
      phase_diff += 2.0*PI;
    }
    while (phase_diff > PI/2.0) {
      phase_diff -= 2.0*PI;
    }
    auto t_shift = 50.0*phase_diff/(2.0*PI);
    h.event_t0 = t0 + t_shift;
    if (!offsets.empty()) {
      h.event_t0 -= offsets.at(h.paddle_id);
    }
  }
}

auto g::TofEventSummary::get_trigger_pids(const gondola::DsiJChnPaddleIdMap& lgmap) const -> Vec<u8> {
  auto trigger_pids = Vec<u8>();
  for (auto const &hit : get_trigger_hits()) {
    u8 dsi = std::get<0>(hit);
    u8 j   = std::get<1>(hit);
    u8 ch  = std::get<2>(hit);
    // don't care about the threshold here
    //u8 thr = (u8)std::get<3>(hit);
    if (!lgmap.contains(dsi)) {
      spdlog::error("Can not find DSI {} in LG map!", dsi);
      continue;
    } 
    if (!lgmap.at(dsi).contains(j)) {
      spdlog::error("Can not find J {} for DSI {} in LG map!", j, dsi);
      continue;
    }
    if (!lgmap.at(dsi).at(j).contains(ch)) {
      spdlog::error("Can not find Ch {} for DSI {} J {} in LG map!", ch, dsi, j);
      continue;
    }
    trigger_pids.push_back(lgmap.at(dsi).at(j).at(ch));
  }
  return trigger_pids;
}
#endif


auto g::TofEventSummary::get_trigger_hits() const -> Vec<std::tuple<u8,u8,u8, g::LTBThreshold>> {
  auto hits = Vec<std::tuple<u8,u8,u8, g::LTBThreshold>>(); 
  auto dsi_j_mask_bits = std::bitset<32>(dsi_j_mask);
  u32 n_masks_needed   = dsi_j_mask_bits.count();
  if (channel_mask.size() < n_masks_needed) {
    spdlog::error("We need {} hit masks, but only have {}! This is bad!", n_masks_needed,  channel_mask.size());
    return hits;
  }
  u8 n_mask = 0;
  for (u8 k=0;k<32;k++) {
    if ((u32)((dsi_j_mask >> k) & 0x1) == 1) {
      u8 dsi = 0;
      u8 j   = 0;
      if (k < 5) {
        dsi = 1;
        j   = k  + 1;
      } else if (k < 10) {
        dsi = 2;
        j   = k  - 5 + 1;
      } else if (k < 15) {
        dsi = 3;
        j   = k - 10 + 1;
      } else if (k < 20) {
        dsi = 4;
        j   = k - 15 + 1;
      } else if (k < 25) {
        dsi = 5;
        j   = k - 20 + 1;
      } 
      u32 channels = channel_mask[n_mask]; 
      for (u8 i=0;i<8; i++) {
        u32 ch  = LTB_CHANNELS[i];
        u32 chn = 2*i + 1; 
        int thresh_bits = (int)((channels & ch) >> (i*2));
        if (thresh_bits > 0 && thresh_bits < 255 ) { // hit over threshold
          hits.push_back(std::make_tuple(dsi, j, chn, (LTBThreshold)(thresh_bits)));
        }
      }
      n_mask += 1;
    }
  }
  return hits;
}

auto g::TofEventSummary::get_rb_link_ids() const -> Vec<u8> {
  auto links = Vec<u8>();
  for (u8 k=0;k<64;k++) {
    if (((u64)(mtb_link_mask >> k) & (u64)0x1) == 1) {
      links.push_back(k);
    }
  }
  return links;
}


auto g::TofEventSummary::from_bytestream(const Vec<u8> &stream, u64 &pos) 
  -> Result<g::TofEventSummary, g::IOError> {
  u16 head = g::parse_u16(stream, pos);
  if (head != TofEventSummary::HEAD) {
    auto message = std::format("Decoding of HEAD failed! Got {} instead!", head);
    auto err = g::IOError(g::IOError::ErrorKind::WrongHeaderBytes, message);
    return Err(err);
  }
  TofEventSummary tes;
  u8 status_version_u8  = g::parse_u8(stream, pos);
  tes.status            = static_cast<EventStatus>(status_version_u8 & 0x3f);
  tes.version           = (g::ProtocolVersion)(status_version_u8 & 0xc0);
  tes.trigger_sources   = g::parse_u16(stream, pos);
  tes.n_trigger_paddles = g::parse_u8(stream, pos);
  tes.event_id          = g::parse_u32(stream, pos);
  if (tes.version == g::ProtocolVersion::V1) {
    tes.n_hits_umb      = g::parse_u8(stream, pos);
    tes.n_hits_cbe      = g::parse_u8(stream, pos);
    tes.n_hits_cor      = g::parse_u8(stream, pos);
    tes.tot_edep_umb    = g::parse_f32(stream, pos);
    tes.tot_edep_cbe    = g::parse_f32(stream, pos);
    tes.tot_edep_cor    = g::parse_f32(stream, pos);
  } 
  tes.quality           = g::parse_u8(stream, pos);
  tes.timestamp32       = g::parse_u32(stream, pos);
  tes.timestamp16       = g::parse_u16(stream, pos);
  tes.run_id            = g::parse_u16(stream, pos);
  //tes.primary_beta      = g::parse_u16(stream, pos); 
  //tes.primary_charge    = g::parse_u16(stream, pos); 
  tes.drs_dead_lost_hits = g::parse_u16(stream, pos);
  tes.dsi_j_mask        = g::parse_u32(stream, pos);
  u8 n_channel_masks    = g::parse_u8(stream, pos);
  for (u8 k=0;k<n_channel_masks;k++) {
    tes.channel_mask.push_back(g::parse_u16(stream, pos));
  }
  tes.mtb_link_mask     = g::parse_u64(stream, pos);
  u16 nhits             = g::parse_u16(stream, pos);
  for (u16 k=0; k<nhits; k++) {
    auto maybe_h = g::TofHit::from_bytestream(stream, pos);
    if (maybe_h.is_ok()) {
      auto h = maybe_h.unwrap();
      tes.hits.push_back(h);
    }
  }
  u16 tail = g::parse_u16(stream, pos);
  if (tail != g::TofEventSummary::TAIL) {
    auto message = std::format("Decoding of TAIL failed! Got {} instead!", tail);
    auto err = g::IOError(g::IOError::ErrorKind::WrongTailBytes, message);
    return Err(err);
  }
  return Ok(tes);
}

auto g::TofEventSummary::from_tofpacket(const TofPacket &packet) 
  -> Result<TofEventSummary, g::IOError> {
  TofEventSummary event;
  // this is not a bug. We are removing TofEventSummary, and there are some 
  // changes to the packet type. TofEvent -> TofEventDeprecated TofEventSummary -> TofEvent
  if (packet.packet_type != PacketType::TofEvent) {
    auto message = std::format("Wrong packet type! {}", packet_type_to_string(packet.packet_type));
    spdlog::error(message);
    auto err = g::IOError(g::IOError::ErrorKind::WrongPacketType, message);
    return Err(err);
  } 
  u64 _pos = 0;
  return TofEventSummary::from_bytestream(packet.payload, _pos);
}

u64 g::TofEventSummary::get_timestamp48() const {
  return ((u64)timestamp16 << 32) | (u64)timestamp32;
}

auto g::TofEventSummary::set_timestamp48(u64 timestamp48) -> void {
  timestamp16 = static_cast<u16>((timestamp48 >> 32) & 0xFFFF);
  timestamp32 = static_cast<u32>(timestamp48 & 0xFFFFFFFF);
}

#ifdef BUILD_CXXDB
auto g::TofEventSummary::set_paddlemap(const g::TofPaddleMap& paddlemap) -> void {
  for (auto &h : hits) {
    h.set_paddle(paddlemap.at(h.paddle_id));
  }
};
#endif

auto g::TofEventSummary::to_string() const -> std::string {
  std::ostringstream oss;
  oss << status;  // Use the << operator to stream the enum
  std::string repr = "<TofEventSummary";
  //repr += std::format("\n  format test {:.2f}", get_time_a() );
  repr += std::format("\n  Run ID             : {}", run_id);
  repr += std::format("\n  Event ID           : {}", event_id);
  repr += std::format("\n  Status             : {}", oss.str());
  repr += std::format("\n  Trigger Sources    : {}", trigger_sources);
  repr += std::format("\n  N trig paddles     : {}", n_trigger_paddles);
  repr += std::format("\n  timestamp32        : {}", timestamp32)      ;
  repr += std::format("\n  timestamp16        : {}", timestamp16)      ;
  repr += std::format("\n  |->timestamp48     : {}", get_timestamp48());
  repr += std::format("\n  NHits       (reco) : {}", hits.size());
  repr += "\n** Trigger Sources **";
  for (const auto &ts : get_trigger_sources()) {
    repr += std::format("\n -- {}", (u8)ts);
  }
  repr += "\n** Trigger Hits **";
  for (const auto &h : get_trigger_hits()) {
    repr += std::format("\n -- {} {} {} {}", std::get<0>(h), std::get<1>(h), std::get<2>(h), (u8)std::get<3>(h));
  }
  repr += "\n** MTB Link IDs **";
  for (const u8 &lid : get_rb_link_ids()) {
    repr += std::format("\n -- {}", lid);
  }
  repr += ">";
  repr += "\n  **** **** ****";
  for (auto const &h : hits) {
    repr += std::format("\n  {}",h.to_string()); 
  }
  repr += ">";
  return repr;
}
  
namespace gondola {

  std::ostream& operator<<(std::ostream& os, const g::TofEventSummary& tes) {
    os << tes.to_string();
    return os;
  }
}
