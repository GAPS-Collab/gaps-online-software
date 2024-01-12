#include "events/tof_event_header.hpp"
#include "parsers.h"
#include "logging.hpp"
//#include "serialization.h"

//#include "spdlog/spdlog.h"
//#include "spdlog/cfg/env.h"

TofEventHeader TofEventHeader::from_bytestream(const Vec<u8> &stream, 
                                               u64 &pos) {
  log_debug("Start decoding at pos " <<  pos);
  u16 head = Gaps::parse_u16(stream, pos);
  if (head != TofEventHeader::HEAD)  {
    log_error("No header signature found!");  
  }
  TofEventHeader header      = TofEventHeader();
  header.run_id              = Gaps::parse_u32(stream, pos);
  header.event_id            = Gaps::parse_u32(stream, pos);
  header.timestamp_32        = Gaps::parse_u32(stream, pos);
  header.timestamp_16        = Gaps::parse_u16(stream, pos);
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
    log_error("TAIL signature not found! Got " << tail << " instead.");
  }
  return header;
} 
  
std::string TofEventHeader::to_string() const {
  std::string repr = "<TofEventHeader";
  repr += "\n  Run   ID          : " + std::to_string(run_id              );
  repr += "\n  Event ID          : " + std::to_string(event_id            );
  repr += "\n  Timestamp 32      : " + std::to_string(timestamp_32        );
  repr += "\n  Timestamp 16      : " + std::to_string(timestamp_16        );
  repr += "\n  Prim. Beta        : " + std::to_string(primary_beta        );
  repr += "\n  Prim. Beta Unc    : " + std::to_string(primary_beta_unc    );
  repr += "\n  Prim. Charge      : " + std::to_string(primary_charge      );
  repr += "\n  Prim. Charge unc  : " + std::to_string(primary_charge_unc  );
  repr += "\n  Prim. Outer Tof X : " + std::to_string(primary_outer_tof_x );
  repr += "\n  Prim. Outer Tof Y : " + std::to_string(primary_outer_tof_y );
  repr += "\n  Prim. Outer Tof Z : " + std::to_string(primary_outer_tof_z );
  repr += "\n  Prim. Inner Tof X : " + std::to_string(primary_inner_tof_x );
  repr += "\n  Prim. Inner Tof Y : " + std::to_string(primary_inner_tof_y );
  repr += "\n  Prim. Inner Tof Z : " + std::to_string(primary_inner_tof_z );
  repr += "\n  NHit  Outer Tof   : " + std::to_string(nhit_outer_tof      );
  repr += "\n  NHit  Inner Tof   : " + std::to_string(nhit_inner_tof      );
  repr += "\n  TriggerInfo       : " + std::to_string(trigger_info        );
  repr += "\n  Ctr ETX           : " + std::to_string(ctr_etx             );
  repr += "\n  NPaddles          : " + std::to_string(n_paddles           ) + ">";
  return repr;
}

std::ostream& operator<<(std::ostream& os, const TofEventHeader& h) {
  os<<h.to_string();
  return os;
}

