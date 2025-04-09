#include "events/tof_event_header.hpp"
#include "parsers.h"
#include "logging.hpp"
//#include "serialization.h"

//#include "spdlog/spdlog.h"
//#include "spdlog/cfg/env.h"

using namespace result;
namespace g = Gaps;

auto TofEventHeader::from_bytestream(const Vec<u8> &stream, u64 &pos) 
  -> Result<TofEventHeader, g::IOError> {
  SPDLOG_TRACE("Start decoding at pos {}", pos);
  u16 head = Gaps::parse_u16(stream, pos);
  if (head != TofEventHeader::HEAD)  {
    auto msg = std::format("No header signature found!");
    spdlog::error(msg);
    return Err(g::IOError(g::IOError::ErrorKind::WrongHeaderBytes, msg));
  }
  TofEventHeader header      = TofEventHeader();
  header.run_id              = Gaps::parse_u32(stream, pos);
  header.event_id            = Gaps::parse_u32(stream, pos);
  header.timestamp32         = Gaps::parse_u32(stream, pos);
  header.timestamp16         = Gaps::parse_u16(stream, pos);
  header.primary_beta        = Gaps::parse_u16(stream, pos);
  header.primary_beta_unc    = Gaps::parse_u16(stream, pos);
  header.primary_charge      = Gaps::parse_u16(stream, pos);
  header.primary_charge_unc  = Gaps::parse_u16(stream, pos);
  header.primary_outer_tof_x = Gaps::parse_u16(stream, pos);
  header.primary_outer_tof_y = Gaps::parse_u16(stream, pos);
  header.primary_outer_tof_z = Gaps::parse_u16(stream, pos);
  header.primary_inner_tof_x = Gaps::parse_u16(stream, pos);
  header.primary_inner_tof_y = Gaps::parse_u16(stream, pos);
  header.primary_inner_tof_z = Gaps::parse_u16(stream, pos); 
  header.nhit_outer_tof      = Gaps::parse_u8(stream, pos);
  header.nhit_inner_tof      = Gaps::parse_u8(stream, pos);
  header.trigger_info        = Gaps::parse_u8(stream, pos);
  header.ctr_etx             = Gaps::parse_u8(stream, pos);
  header.n_paddles           = Gaps::parse_u8(stream, pos); 
  u16 tail                   = Gaps::parse_u16(stream, pos);
  if (tail != TAIL) {
    auto msg = std::format("No tail signature found! Got {} instead.", tail);
    spdlog::error(msg);
    return Err(g::IOError(g::IOError::ErrorKind::WrongTailBytes, msg));
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

