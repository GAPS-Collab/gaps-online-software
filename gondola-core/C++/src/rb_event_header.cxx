// This file is part of gaps-online-software and published 
// under the GPLv3 license

#include<numeric>
#include<sstream>
#include<format>
#include<limits>
#include<bitset>
#include<cmath>
#include <sstream>
#include <numbers>

#include "io/parsers.h"
#include "events/rb_event_header.hpp"
#include "serialization.h"
#include "version.h"

#include "spdlog/spdlog.h"
#include "spdlog/cfg/env.h"

namespace g = gondola;
using namespace result;
g::RBEventHeader::RBEventHeader() {
  rb_id              = 0; 
  event_id           = 0; 
  channel_mask       = 0; 
  status_byte        = 0;
  stop_cell          = 0; 
  fpga_temp          = 0;
  timestamp16        = 0; 
  timestamp32        = 0; 
}

/*************************************/

auto g::RBEventHeader::to_string() const -> std::string {
  std::string repr = "<RBEventHeader";
  repr += "\n  rb id          " + std::to_string(rb_id)                 ;
  repr += "\n  event id       " + std::to_string(event_id)              ;
  repr += "\n  is locked      " + std::to_string(is_locked())           ;
  repr += "\n  is locked (1s) " + std::to_string(is_locked_last_sec())  ;
  repr += "\n  lost trigger   " + std::to_string(drs_lost_trigger())    ;
  repr += "\n  event fragment " + std::to_string(is_event_fragment())   ;
  repr += "\n  channel mask   " + std::to_string(channel_mask)          ;
  repr += "\n  |-> channels   ";
  for (auto ch : get_channels()) {
    repr += " " + std::to_string(ch) + " ";
  }
  repr += "\n  stop cell      " + std::to_string(stop_cell)             ;
  repr += "\n  timestamp32    " + std::to_string(timestamp32)           ;
  repr += "\n  timestamp16    " + std::to_string(timestamp16)           ;
  repr += "\n  |->timestamp48 " + std::to_string(get_timestamp48())     ;
  repr += "\n  FPGA temp [C]  " + std::to_string(get_fpga_temp())       ;
  repr += ">";
  return repr;
}

/*************************************/

auto g::RBEventHeader::get_channels() const -> Vec<u8> {
  Vec<u8>  channels = Vec<u8>();
  for (u8 k=0;k<9;k++) {
    if ((channel_mask & (1 << k)) > 0) {
      channels.push_back(k);
    }
  }
  return channels; 
}

/*************************************/

auto g::RBEventHeader::get_nchan() const -> u8 {
  return get_channels().size(); 
}

/*************************************/

auto g::RBEventHeader::from_bytestream(const Vec<u8> &stream, u64 &pos)\
  -> Result<RBEventHeader, g::IOError> {
  //g::set_loglevel(g::LOGLEVEL::info);
  if (stream.size() < RBEventHeader::SIZE) {
    auto message = std::format("RBEventHeader can not be parsed from a string with size {}, when {} bytes are expected!", stream.size(), RBEventHeader::SIZE);
    auto err = g::IOError(g::IOError::ErrorKind::StreamTooShort, message);
    return Err(err);
  }
  RBEventHeader header;
  u16 head                  = g::parse_u16(stream, pos);
  if (head != RBEventHeader::HEAD) {
    spdlog::error("[RBEventHeader::from_bytestream] Header signature {} invalid!", head);
  }
  header.rb_id               = g::parse_u8(stream , pos);  
  header.event_id            = g::parse_u32(stream, pos);  
  u16 ch_mask                = g::parse_u16(stream, pos);   
  auto ch_mask_deadtime      = g::RBEventHeader::parse_channel_mask(ch_mask);
  header.deadtime_instead_temp = std::get<0>(ch_mask_deadtime);
  header.set_channel_mask(std::get<1>(ch_mask_deadtime));
  header.status_byte         = g::parse_u8(stream , pos); 
  header.stop_cell           = g::parse_u16(stream, pos);  
  header.pid_ch12            = g::parse_u8(stream, pos);
  header.pid_ch34            = g::parse_u8(stream, pos);
  header.pid_ch56            = g::parse_u8(stream, pos);
  header.pid_ch78            = g::parse_u8(stream, pos);
  header.pid_ch_order        = g::parse_u8(stream, pos);
  header.rsvd1               = g::parse_u8(stream, pos);
  header.rsvd2               = g::parse_u8(stream, pos);
  header.rsvd3               = g::parse_u8(stream, pos);
  if (header.deadtime_instead_temp) {
    header.drs_deadtime      = g::parse_u16(stream, pos);
  } else {
    header.fpga_temp         = g::parse_u16(stream, pos);
  }
  header.timestamp32         = g::parse_u32(stream, pos);
  header.timestamp16         = g::parse_u16(stream, pos);
  u16 tail                   = g::parse_u16(stream, pos);
  if (tail != RBEventHeader::TAIL) {
    spdlog::error("Tail signature incorrect! Got tail {}", tail);
  }
  return Ok(header); 
}

/*************************************/

auto g::RBEventHeader::has_ch9() const -> bool {
  return (channel_mask & 512) > 0;
}

/*************************************/
  
auto g::RBEventHeader::get_fpga_temp() const -> f32 {
  f32 zynq_temp = (((fpga_temp & 4095) * 503.975) / 4096.0) - 273.15;
  //f32 temp = (fpga_temp * 503.975/4096) - 273.15;
  return zynq_temp;
}

/*************************************/

auto g::RBEventHeader::is_event_fragment() const -> bool {
  return (status_byte & 1) > 0;
}

/*************************************/

auto g::RBEventHeader::drs_lost_trigger() const -> bool {
  return ((status_byte >> 1) & 1) > 0;
}

/*************************************/

auto g::RBEventHeader::lost_lock() const -> bool {
  return ((status_byte >> 2) & 1) > 0;
}

/*************************************/

auto g::RBEventHeader::lost_lock_last_sec() const -> bool {
  return ((status_byte >> 3) & 1) > 0;
}

/*************************************/

auto g::RBEventHeader::is_locked() const -> bool {
  return !(lost_lock());
}

/*************************************/

auto g::RBEventHeader::is_locked_last_sec() const -> bool {
  return !(lost_lock_last_sec());
}

/*************************************/

auto g::RBEventHeader::get_timestamp48() const -> u64 {
  return ((u64)timestamp16 << 32) | (u64)timestamp32;
}

/*************************************/

auto g::RBEventHeader::get_active_data_channels() const -> Vec<u8> {
  Vec<u8> active_channels;
  for (auto const &ch : {1,2,3,4,5,6,7,8} ) {
    if ((channel_mask & (u8)pow(2, ch - 1)) == (u8)pow(2,ch - 1)) active_channels.push_back(ch);
  } 
  //if ((channel_mask & 1)   == 1)   active_channels.push_back(1);
  return active_channels;
}

/*************************************/

auto g::RBEventHeader::get_n_datachan() const -> u8 {
  Vec<u8> active_channels = get_active_data_channels();
  return (u8)active_channels.size();
}

//---------------------------------------------------

auto g::RBEventHeader::to_bytestream() const -> Vec<u8> {
  Vec<u8> stream = Vec<u8>(SIZE);
  bytestream_extend(stream, HEAD);
  bytestream_extend(stream, rb_id);
  bytestream_extend(stream, event_id);
  u16 ch_mask = (((u16)deadtime_instead_temp) << 15) | get_channel_mask();
  bytestream_extend(stream, ch_mask);
  bytestream_extend(stream, status_byte);
  bytestream_extend(stream, stop_cell);
  stream.push_back(pid_ch12    );
  stream.push_back(pid_ch34    );
  stream.push_back(pid_ch56    );
  stream.push_back(pid_ch78    );
  stream.push_back(pid_ch_order);
  stream.push_back(rsvd1       );
  stream.push_back(rsvd2       );
  stream.push_back(rsvd3       );
  if (deadtime_instead_temp) {
    bytestream_extend(stream, drs_deadtime);
  } else {
    bytestream_extend(stream, fpga_temp );
  }
  bytestream_extend(stream, timestamp32 );
  bytestream_extend(stream, timestamp16 );
  bytestream_extend(stream, TAIL);
  return stream;
}

//---------------------------------------------------
  
auto g::RBEventHeader::parse_channel_mask(u16 ch_mask) -> std::tuple<bool, u16> {
  u16 channel_mask;;
  bool deadtime_instead_temp = ch_mask >> 15 == 1;
  channel_mask = ch_mask & 0x1ff;
  return std::make_tuple(deadtime_instead_temp, channel_mask);
}

//---------------------------------------------------

auto g::RBEventHeader::set_channel_mask(u16 ch_mask) -> void {
  if (deadtime_instead_temp) {
    channel_mask = 0x8000 | ch_mask; // 2**15 = 0x8000
  } else {
    channel_mask = ch_mask;
  }
}

//---------------------------------------------------

auto g::RBEventHeader::get_channel_mask() const -> u16 {
  return channel_mask & 0x1ff; 
}


