// This file is part of gaps-online-software and published 
// under the GPLv3 license

#include "events/mc_event.hpp"
#include "io/parsers.h"

#include <format>

namespace go = gondola;
using namespace result;

#ifdef BUILD_SIM

auto go::McEvent::to_bytestream() const -> Vec<u8> {
  auto stream = Vec<u8>();
  auto bytes = go::to_le_bytes((u16)0xAAAA);
  stream.insert(stream.end(), bytes.begin(), bytes.end());
  bytes = go::to_le_bytes((u32)run_id);
  stream.insert(stream.end(), bytes.begin(), bytes.end());
  bytes = go::to_le_bytes((u32)event_id);
  stream.insert(stream.end(), bytes.begin(), bytes.end());
  bytes = primary.to_bytestream();
  stream.insert(stream.end(), bytes.begin(), bytes.end());
  u16 nhits  = (u16)hits.size();
  bytes = go::to_le_bytes((u16)nhits);
  stream.insert(stream.end(), bytes.begin(), bytes.end());
  for (auto const &h : hits) {
    auto h_bytes = h.to_bytestream();
    stream.insert(stream.end(), h_bytes.begin(), h_bytes.end());
  }
  bytes = go::to_le_bytes((u16)0x5555);
  stream.insert(stream.end(), bytes.begin(), bytes.end());
  return stream;
}

#endif

//-------------------------------------------

auto go::McEvent::from_bytestream(const Vec<u8> &stream, usize &pos)
  -> result::Result<go::McEvent, go::IOError> {
  // head, run id, event id, (tracklet), nhits, (hits), tail
  if (stream.size() < pos + 2 + 4 + 4 + Tracklet::SIZE + 2 + 2) {
    auto message = std::format("Stream too short for McEvent! {} bytes left", stream.size() - pos);
    return Err(go::IOError(go::IOError::ErrorKind::StreamTooShort, message));
  }
  u16 head = go::parse_u16(stream, pos);
  if (head != McEvent::HEAD) {
    auto message = std::format("Decoding of HEAD failed! Got {} instead!", head);
    return Err(go::IOError(go::IOError::ErrorKind::WrongHeaderBytes, message));
  }
  auto event     = McEvent();
  event.run_id   = go::parse_u32(stream, pos);
  event.event_id = go::parse_u32(stream, pos);
  auto primary   = Tracklet::from_bytestream(stream, pos);
  if (primary.is_err()) {
    return Err(primary.unwrap_err());
  }
  event.primary  = primary.unwrap();
  u16 nhits      = go::parse_u16(stream, pos);
  event.hits.reserve(nhits);
  for (u16 k=0; k<nhits; k++) {
    auto hit = McHit::from_bytestream(stream, pos);
    if (hit.is_err()) {
      return Err(hit.unwrap_err());
    }
    event.hits.push_back(hit.unwrap());
  }
  if (stream.size() < pos + 2) {
    auto message = std::format("Stream too short for McEvent TAIL!");
    return Err(go::IOError(go::IOError::ErrorKind::StreamTooShort, message));
  }
  u16 tail = go::parse_u16(stream, pos);
  if (tail != McEvent::TAIL) {
    auto message = std::format("Decoding of TAIL failed! Got {} instead!", tail);
    return Err(go::IOError(go::IOError::ErrorKind::WrongTailBytes, message));
  }
  return Ok(event);
}
