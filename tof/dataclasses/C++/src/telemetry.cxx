#include "telemetry.hpp"
#include "parsers.h"
#include "logging.hpp"

f64 Gaps::TelemetryHeader::get_gcutime(){
  return timestamp * 0.064 + 1631030675.0;
};
  
Gaps::TelemetryHeader Gaps::TelemetryHeader::from_bytestream(Vec<u8> const &stream,
                                                             usize &pos) {
  TelemetryHeader thead;
  if (stream.size() < pos + Gaps::TelemetryHeader::SIZE) {
    log_error("The telemetry header is too short! (" << stream.size() << " bytes when " << TelemetryHeader::SIZE << " are expected");
    return thead;
  }
  if (parse_u16(stream, pos) != Gaps::TelemetryHeader::HEAD) {
    log_error("The given position " << pos << " does not point to a valid header signature of " << Gaps::TelemetryHeader::HEAD);
    return thead;
  }
  thead.sync      = 0x90eb;
  thead.ptype     = parse_u8 (stream, pos);
  thead.timestamp = parse_u32(stream, pos);
  thead.counter   = parse_u16(stream, pos);
  thead.length    = parse_u16(stream, pos);
  thead.checksum  = parse_u16(stream, pos);
  return thead;
}

std::string Gaps::TelemetryHeader::to_string() {
  std::string repr = "<TelemetryHeader:";
  repr += std::format("\n  Header      : {}" ,sync);
  repr += std::format("\n  Packet Type : {}" ,ptype);
  repr += std::format("\n  Timestamp   : {}" ,timestamp);
  repr += std::format("\n  Counter     : {}" ,counter);
  repr += std::format("\n  Length      : {}" ,length);
  repr += std::format("\n  Checksum    : {}>",checksum);
  return repr;
}

//----------------------------------------

Gaps::TelemetryPacket Gaps::TelemetryPacket::from_bytestream(Vec<u8> const &stream,
                                                             usize &pos) {
  Gaps::TelemetryPacket tpacket;
  Gaps::TelemetryHeader header  = Gaps::TelemetryHeader::from_bytestream(stream, pos);
  tpacket.header = header;
  auto payload   = Gaps::slice(stream, pos, pos + header.length - Gaps::TelemetryHeader::SIZE);
  tpacket.payload = std::move(payload);
  return tpacket;
}

std::string Gaps::TelemetryPacket::to_string() {
  std::string repr = "<TelemetryPacket:";
  repr += std::format("{}", header.to_string());
  repr += "\n --------";
  repr += std::format("\n  Payload len : {}>",payload.size());
  return repr;
}


