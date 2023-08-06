#include "spdlog/spdlog.h"
#include "spdlog/cfg/env.h"

#include "serialization.h"
#include "parsers.h"

#include "io.hpp"

/***************************************************/

Vec<RBEventHeader> get_headers(const String &filename, bool is_headers) {
  spdlog::cfg::load_env_levels();
  u64 n_good = 0;  
  u64 n_bad  = 0; 
  Vec<RBEventHeader> headers;
  bytestream stream = get_bytestream_from_file(filename); 
  bool has_ended = false;
  auto pos = search_for_2byte_marker(stream,0xAA, has_ended );
  spdlog::info("Read {} bytes from {}", stream.size(), filename);
  spdlog::info("For 8+1 channels and RB compression level 0, this mean a max number of events of {}", stream.size()/18530.0);
  while (!has_ended) {
    RBEventHeader header;
    if (is_headers) {
      header = RBEventHeader::from_bytestream(stream, pos);
    } else {
      header = RBEventHeader::extract_from_rbbinarydump(stream, pos);
    }
    header.broken ? n_bad++ : n_good++ ;
    headers.push_back(header);
    pos -= 2;
    pos = search_for_2byte_marker(stream, 0xAA, has_ended, pos);
    //if (header.broken) {
    //  std::cout << pos << std::endl;
    //  std::cout << (u32)header.channel_mask << std::endl;
    //}
  }
  spdlog::info("Retrieved {} good headers, but {} of which we had to set the `broken` flag", n_good, n_bad);
  return headers;
}

/***************************************************/

Vec<u32> get_event_ids_from_raw_stream(const Vec<u8> &bytestream, u64 &pos) {
  Vec<u32> event_ids;
  u32 event_id = 0;
  // first, we need to find the first header in the 
  // stream starting from the given position
  bool has_ended = false;
  while (!has_ended) { 
    pos = search_for_2byte_marker(bytestream, 0xAA, has_ended, pos);  
    pos += 22;
    event_id = Gaps::u32_from_le_bytes(bytestream, pos);
    event_ids.push_back(event_id);
    pos += 18530 - 22 - 4;
  }
  return event_ids; 
}

/***************************************************/

Vec<TofPacket> get_tofpackets(const Vec<u8> &bytestream, u64 start_pos) {
  Vec<TofPacket> packets;
  u64 pos  = start_pos;
  // just make sure in the beginning they
  // are not the same
  u64 last_pos = start_pos += 1;
  TofPacket packet;
  while (true) {
    last_pos = pos;
    pos = packet.from_bytestream(bytestream, pos);
    if (pos != last_pos) {
      packets.push_back(packet);
    } else {
      break;
    }
  }
  return packets;
}

/***************************************************/

Vec<TofPacket> get_tofpackets(const String filename) {
  spdlog::cfg::load_env_levels();
  auto stream = get_bytestream_from_file(filename); 
  bool has_ended = false;
  auto pos = search_for_2byte_marker(stream,0xAA, has_ended );
  spdlog::info("Read {} bytes from {}", stream.size(), filename);
  return get_tofpackets(stream, pos);
}

