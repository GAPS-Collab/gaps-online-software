#include "mc_event.hpp"
#include "io/parsers.h"

namespace go = gondola;

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

