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
  ch9_amp            = 0;
  ch9_freq           = 0;
  ch9_phase          = 0;
  fpga_temp          = 0;
  timestamp16        = 0; 
  timestamp32        = 0; 
}

/*************************************/

auto g::RBEventHeader::to_string() const -> std::string {
  auto sfit = get_sine_fit();
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
  repr += "\n  ** online ch9 fit amp, freq, phase";
  repr += "\n    AMP " + std::to_string(sfit[0]);
  repr += "  FREQ " + std::to_string(sfit[1]);
  repr += "  PHASE " + std::to_string(sfit[2]); 
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
  header.channel_mask        = g::parse_u16(stream, pos);   
  header.status_byte         = g::parse_u8(stream , pos); 
  header.stop_cell           = g::parse_u16(stream, pos);  
  header.ch9_amp             = g::parse_u16(stream, pos);  
  header.ch9_freq            = g::parse_u16(stream, pos);  
  header.ch9_phase           = g::parse_u32(stream, pos);  
  header.fpga_temp           = g::parse_u16(stream, pos);  
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

/*************************************/

auto g::RBEventHeader::get_sine_fit() const -> std::array<f32, 3> {
  f32 u16_MAX = 65535;
  f32 amp    = (20.0 * ch9_amp   /u16_MAX) - 10.0;
  f32 freq   = (20.0 * ch9_freq  /u16_MAX) - 10.0;
  f32 phase  = (20.0 * ch9_phase /u16_MAX) - 10.0;
  std::array<f32, 3> result = {amp,freq,phase};
  return result;
}

