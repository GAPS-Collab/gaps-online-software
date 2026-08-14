/// This file is part of gaps-online-software and published 
/// under the GPLv3 license

#include "packets/telemetry_packet.hpp"
#include "io/parsers.h"

#include "spdlog/spdlog.h"
#include "spdlog/cfg/env.h"

namespace g   = gondola;

// make it more look like rust
using namespace result;

/// FIXME - this is obviously incomplete
u8 g::bfsw_ptype_to_u8(g::TelemetryPacketType pt) {
  switch (pt) {
    case g::TelemetryPacketType::Unknown:
      return 0;
    case g::TelemetryPacketType::Tracker:
      return 80;
    case g::TelemetryPacketType::BoringEvent:
      return 90;
    case g::TelemetryPacketType::RBWaveform:
      return 91;
    case g::TelemetryPacketType::InterestingEvent:
      return 190;
    case g::TelemetryPacketType::NoGapsTriggerEvent:
      return 191;
    case g::TelemetryPacketType::NoTofDataEvent:
      return 192;
    default:
      return 0;
  }
}

std::string g::bfsw_ptype_to_str(g::TelemetryPacketType pt) {
  switch (pt) {
      case g::TelemetryPacketType::Unknown:
      return "Unknown";
    case g::TelemetryPacketType::Tracker:
      return "Tracker";
    case g::TelemetryPacketType::BoringEvent:
      return "BoringEvent";
    case g::TelemetryPacketType::RBWaveform:
      return "RBWaveform";
    case g::TelemetryPacketType::InterestingEvent:
      return "InterestingEvent";
    case g::TelemetryPacketType::NoGapsTriggerEvent:
      return "NoGapsTriggerEvent";
    case g::TelemetryPacketType::NoTofDataEvent:
      return "NoTofDataEvent";
    case g::TelemetryPacketType::CoolingHK:
      return "CoolingHK";
    case g::TelemetryPacketType::CardHKP:
      return "CardHKP";
    case g::TelemetryPacketType::PDUHK:
      return "PDUHK";
    case g::TelemetryPacketType::AnyTofHK:
      return "AnyTofHK";
    case g::TelemetryPacketType::LabJackHK:
      return "LabJackHK";
    case g::TelemetryPacketType::TrackerDAQCntr:
      return "TrackerDAQCntr";
    default:
      return std::format("Unknown/NotImplemented ({})", (int)pt) ;
  }
}

//----------------------------------------

f64 g::TelemetryPacketHeader::get_gcutime() const {
  return timestamp * 0.064 + 1631030675.0;
};

//----------------------------------------

auto g::TelemetryPacketHeader::to_bytestream() const -> Vec<u8> {
  Vec<u8> bytes{0xeb, 0x90};
  bytes.push_back((u8)ptype);
  auto ts_byt = gondola::to_le_bytes(timestamp);
  auto co_byt = gondola::to_le_bytes(counter);
  auto le_byt = gondola::to_le_bytes(length);
  auto ch_byt = gondola::to_le_bytes(checksum);
  bytes.insert(bytes.end(),ts_byt.begin(), ts_byt.end());
  bytes.insert(bytes.end(),co_byt.begin(), co_byt.end());
  bytes.insert(bytes.end(),le_byt.begin(), le_byt.end());
  bytes.insert(bytes.end(),ch_byt.begin(), ch_byt.end());  
  return bytes;
}

auto  g::TelemetryPacketHeader::from_bytestream(Vec<u8> const &stream, usize &pos)
  -> Result<g::TelemetryPacketHeader, g::IOError> {
  g::TelemetryPacketHeader ph;
  if (stream.size() < pos + g::TelemetryPacketHeader::SIZE) {
    spdlog::error("The telemetry header is too short! ({} bytes when {} are expected!", stream.size(), g::TelemetryPacketHeader::SIZE);
    pos += g::TelemetryPacketHeader::SIZE;
    return Ok(ph);
  }
  if (g::parse_u16(stream, pos) != g::TelemetryPacketHeader::HEAD) {
    spdlog::error("The given position {} does not point to a valid header signature of {}", pos-2 ,g::TelemetryPacketHeader::HEAD);
    pos += g::TelemetryPacketHeader::SIZE - 2;
    return Ok(ph);
  }
  ph.sync      = g::TelemetryPacketHeader::HEAD;
  ph.ptype     = static_cast<g::TelemetryPacketType>(g::parse_u8 (stream, pos));
  ph.timestamp = g::parse_u32(stream, pos);
  ph.counter   = g::parse_u16(stream, pos);
  ph.length    = g::parse_u16(stream, pos);
  ph.checksum  = g::parse_u16(stream, pos);
  return Ok(ph);
}

auto g::TelemetryPacketHeader::to_string() const -> std::string {
  std::string repr = "<TelemetryPacketHeader:";
  repr += std::format("\n  Header      : {}" ,sync);
  repr += std::format("\n  Packet Type : {}" ,bfsw_ptype_to_str(ptype));
  repr += std::format("\n  Timestamp   : {}" ,timestamp);
  repr += std::format("\n  Counter     : {}" ,counter);
  repr += std::format("\n  Length      : {}" ,length);
  repr += std::format("\n  Checksum    : {}>",checksum);
  return repr;
}

//----------------------------------------

auto  g::TelemetryPacket::from_bytestream(Vec<u8> const &stream,
                                 usize &pos) -> g::TelemetryPacket {
  g::TelemetryPacket packet;
  auto header    = g::TelemetryPacketHeader::from_bytestream(stream, pos).unwrap();
  // FIXME
  packet.header  = header;
  auto payload   = g::slice(stream, pos, pos + header.length);
  packet.payload = std::move(payload);
  return packet;
}

auto g::TelemetryPacket::is_event_packet() const -> bool {
  return header.ptype == g::TelemetryPacketType::InterestingEvent   || 
         header.ptype == g::TelemetryPacketType::BoringEvent        || 
         header.ptype == g::TelemetryPacketType::NoGapsTriggerEvent || 
         header.ptype == g::TelemetryPacketType::NoTofDataEvent;      
}


auto g::TelemetryPacket::to_string() const -> std::string {
  std::string repr = "<TelemetryPacket:";
  repr += std::format("{}", header.to_string());
  repr += "\n --------";
  repr += std::format("\n  Payload len : {}>",payload.size());
  return repr;
}

