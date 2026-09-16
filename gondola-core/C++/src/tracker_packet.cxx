/// This file is part of gaps-online-software and published 
/// under the GPLv3 license

#include <format>
#include <ranges>

#include "spdlog/spdlog.h"
#include "spdlog/cfg/env.h"

#include "packets/tracker_packet.hpp"
#include "io/parsers.h"
#include "events/tracker_hit.hpp"

namespace g = gondola; 

using namespace result;

auto g::TrackerHeader::to_string() const -> std::string {
  std::string repr = "<TrackerPacket:";
  repr    += std::format("\n  Sync     : {}", sync);
  repr    += std::format("\n  Crc      : {}", crc);
  repr    += std::format("\n  PacketID : {}", packet_id);
  repr    += std::format("\n  Sys ID   : {}", sys_id);
  repr    += std::format("\n  Length   : {}", length);
  repr    += std::format("\n  DAQ Cnt  : {}", daq_count);
  repr    += std::format("\n  Sys Time : {}", sys_time);
  repr    += std::format("\n  Version  : {}>",version);
  return repr;
}

auto g::TrackerHeader::from_bytestream(const Vec<u8> &stream, u64 &pos) 
  -> Result<g::TrackerHeader, g::IOError> { 
  auto h = g::TrackerHeader();
  if (stream.size() < SIZE) {
    spdlog::error("Unable to decode TrackerHeader, the provided byte stream is too short!"); 
    auto message = std::format("Bytestream is too short!");
    auto err = g::IOError(g::IOError::ErrorKind::StreamTooShort, message);
    return Err(err);
  }
  h.sync        = g::parse_u16(stream, pos);
  h.crc         = g::parse_u16(stream, pos); 
  h.sys_id      = g::parse_u8 (stream, pos);
  h.packet_id   = g::parse_u8 (stream, pos);
  h.length      = g::parse_u16(stream, pos);
  h.daq_count   = g::parse_u16(stream, pos);
  auto lower    = g::parse_u32(stream, pos);
  auto upper    = g::parse_u16(stream, pos);
  //h.sys_time    = g::make_systime(lower, upper);
  h.sys_time    = ((u64)upper << 32) | (u64)lower;
  h.version     = g::parse_u8 (stream, pos);
  return Ok(h);
}

// ------------------------------------------------------

auto g::TrackerDAQEventPacket::to_string() const -> std::string {
  std::string repr = "<TrackerDAQEventPacket:";
  repr += std::format("\n  TrackerHeader       : {}", daq_header.to_string());
  repr += std::format("\n  Run ID/Run ID (old) : {} {}", run_id, run_id_old);
  repr += std::format("\n  - N DAQ ev., N Hits : {} {}", events.size(), n_hits);
  for (auto const& daq : events) {
    repr += std::format("\n  {}", daq.to_string());
  }
  repr += "\n";
  return repr;
}

auto g::TrackerDAQEventPacket::from_bytestream(const Vec<u8> &stream, u64 &pos) 
  -> Result<g::TrackerDAQEventPacket, g::IOError> { 
  auto pck = g::TrackerDAQEventPacket();
  auto daq_header = g::TrackerHeader::from_bytestream(stream, pos); 
  if (daq_header.is_ok()) {
    pck.daq_header = std::move(daq_header.unwrap());
  } else {
    auto err = daq_header.unwrap_err();
    return Err(err);
  }
  if (pck.daq_header.version >= 5) {
    pck.run_id     = g::parse_u16(stream, pos);
  } else {
    pck.run_id_old = g::parse_u8(stream, pos);
  }
  usize event_header_size = 12;  
  while (true) {
    if (pck.events.size() > 170) {
      spdlog::error("There seem to be more than 170 events (!) in the tracker. This is nonsense!");
      auto message = std::format("More than 170 tracker events!");
      auto err = g::IOError(g::IOError::ErrorKind::TooManyTrkEvents, message);
      return Err(err); 
    }
    if (pck.daq_header.version >= 4) {
      if (pos + 1 == stream.size()) {
        if (stream[pos] == 0xff) {
          return Ok(pck);
        }
      }
      if (pos == stream.size()) {
        return Ok(pck);
      }
    } 
    if (pos + event_header_size > stream.size()) { 
      spdlog::error("Unable to read more TrackerEvents! Stream is too short!");
      auto message = std::format("Unable to read more TrackerEvents! Stream is too short!");
      auto err = g::IOError(g::IOError::ErrorKind::StreamTooShort, message);
      return Err(err);
    }
    
    auto daq_event     = g::TrkEvent();
    daq_event.layer    = pck.daq_header.sys_id;
    auto n_hits        = g::parse_u8(stream, pos);
    daq_event.flags1   = g::parse_u8(stream, pos);
    daq_event.event_id = g::parse_u32(stream, pos);
    auto lower         = g::parse_u32(stream, pos); 
    auto upper         = g::parse_u16(stream, pos);
    daq_event.event_time = ((u64)upper << 32) | (u64)lower;
    if (n_hits > 192) {
      spdlog::error("We see more than 192 hits in the event! This seems to be an issue.");
      auto message = std::format("We see more than 192 hits in the event! This seems to be an issue!");
      auto err = g::IOError(g::IOError::ErrorKind::TooManyTrkHits, message);
      return Err(err);
    } 
    if ((pos + (3*(n_hits))) > stream.size()) {
      spdlog::error("Unable to read all {} tracker hits! Stream is too short!", n_hits);
      auto message = std::format("Unable to read all {} tracker hits! Stream is too short!", n_hits);
      auto err = g::IOError(g::IOError::ErrorKind::StreamTooShort, message);
      return Err(err);
    }
    for (u8 _ : std::views::iota(0U, n_hits)) {
    //for _ in 0..n_hits {
      auto h0 = g::parse_u8(stream, pos);
      auto h1 = g::parse_u8(stream, pos);
      auto h2 = g::parse_u8(stream, pos);
      auto asic_event_code = h2 >> 6;
      auto channel = h0 & 0b11111;
      auto mod = h0 >> 5;
      auto row = h1 & 0b111;
      u16 adc  = (((u16)h2  & 0b00111111) << 5) | ((u16)h1 >> 3);
      auto hit = g::TrkHit();
      hit.layer   = daq_event.layer - 128;
      hit.row     = row    ;
      hit.module  = mod    ;
      hit.channel = channel;
      hit.adc     = adc    ;
      hit.asic_event_code   = asic_event_code;
      daq_event.hits.push_back(hit);
      pck.n_hits += 1; 
    }
    pck.events.push_back(daq_event);
  }
  return Ok(pck);
}

// ------------------------------------------------------

namespace gondola {
  std::ostream& operator<<(std::ostream& os, const g::TrackerHeader& trk_head) {
    os << trk_head.to_string();
    return os;
  }
  
  std::ostream& operator<<(std::ostream& os, const g::TrackerDAQEventPacket& pck) {
    os << pck.to_string();
    return os;
  }
}
