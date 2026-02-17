#include "spdlog/spdlog.h"
#include "spdlog/cfg/env.h"

#include "events/tof_event_header.hpp"
#include "io/parsers.h"

namespace g = gondola;

using namespace result;

auto TofEventHeader::from_bytestream(const Vec<u8> &stream, u64 &pos) 
  -> r::Result<TofEventHeader, Gaps::IOError> {
  SPDLOG_TRACE("Start decoding at pos {}", pos);
  u16 head = g::parse_u16(stream, pos);
  if (head != TofEventHeader::HEAD)  {
    auto msg = std::format("No header signature found!");
    spdlog::error(msg);
    return Err(Gaps::IOError(Gaps::IOError::ErrorKind::WrongHeaderBytes, msg));
  }
  TofEventHeader header      = TofEventHeader();
  header.run_id              = g::parse_u32(stream, pos);
  header.event_id            = g::parse_u32(stream, pos);
  header.timestamp32         = g::parse_u32(stream, pos);
  header.timestamp16         = g::parse_u16(stream, pos);
  header.primary_beta        = g::parse_u16(stream, pos);
  header.primary_beta_unc    = g::parse_u16(stream, pos);
  header.primary_charge      = g::parse_u16(stream, pos);
  header.primary_charge_unc  = g::parse_u16(stream, pos);
  header.primary_outer_tof_x = g::parse_u16(stream, pos);
  header.primary_outer_tof_y = g::parse_u16(stream, pos);
  header.primary_outer_tof_z = g::parse_u16(stream, pos);
  header.primary_inner_tof_x = g::parse_u16(stream, pos);
  header.primary_inner_tof_y = g::parse_u16(stream, pos);
  header.primary_inner_tof_z = g::parse_u16(stream, pos); 
  header.nhit_outer_tof      = g::parse_u8(stream, pos);
  header.nhit_inner_tof      = g::parse_u8(stream, pos);
  header.trigger_info        = g::parse_u8(stream, pos);
  header.ctr_etx             = g::parse_u8(stream, pos);
  header.n_paddles           = g::parse_u8(stream, pos); 
  u16 tail                   = g::parse_u16(stream, pos);
  if (tail != TAIL) {
    auto msg = std::format("No tail signature found! Got {} instead.", tail);
    spdlog::error(msg);
    return Err(Gaps::IOError(Gaps::IOError::ErrorKind::WrongTailBytes, msg));
  }
  return Ok(header);
} 
  
std::string TofEventHeader::to_string() const {
  std::string repr = "<TofEventHeader";
  repr += std::format("\n  Run   ID          : {}",run_id              );
  repr += std::format("\n  Event ID          : {}",event_id            );
  repr += std::format("\n  Timestamp 32      : {}",timestamp32         );
  repr += std::format("\n  Timestamp 16      : {}",timestamp16         );
  repr += std::format("\n  Prim. Beta        : {}",primary_beta        );
  repr += std::format("\n  Prim. Beta Unc    : {}",primary_beta_unc    );
  repr += std::format("\n  Prim. Charge      : {}",primary_charge      );
  repr += std::format("\n  Prim. Charge unc  : {}",primary_charge_unc  );
  repr += std::format("\n  Prim. Outer Tof X : {}",primary_outer_tof_x );
  repr += std::format("\n  Prim. Outer Tof Y : {}",primary_outer_tof_y );
  repr += std::format("\n  Prim. Outer Tof Z : {}",primary_outer_tof_z );
  repr += std::format("\n  Prim. Inner Tof X : {}",primary_inner_tof_x );
  repr += std::format("\n  Prim. Inner Tof Y : {}",primary_inner_tof_y );
  repr += std::format("\n  Prim. Inner Tof Z : {}",primary_inner_tof_z );
  repr += std::format("\n  NHit  Outer Tof   : {}",nhit_outer_tof      );
  repr += std::format("\n  NHit  Inner Tof   : {}",nhit_inner_tof      );
  repr += std::format("\n  TriggerInfo       : {}",trigger_info        );
  repr += std::format("\n  Ctr ETX           : {}",ctr_etx             );
  repr += std::format("\n  NPaddles          : {}>",n_paddles          );
  return repr;
}

/// combine the slow timestamp with 
/// the fast to get the full
f64 TofEventHeader::get_timestamp48() const {
  f64 ts48 = timestamp16 << 16 | timestamp32;
  return ts48;
}

std::ostream& operator<<(std::ostream& os, const TofEventHeader& h) {
  os<<h.to_string();
  return os;
}

