#include <filesystem>
#include <algorithm>
#include "spdlog/spdlog.h"
#include "spdlog/cfg/env.h"

#include "serialization.h"
#include "parsers.h"
#include "logging.hpp"
#include "io.hpp"

namespace fs = std::filesystem;

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
    event_id = Gaps::parse_u32(bytestream, pos);
    event_ids.push_back(event_id);
    pos += 18530 - 22 - 4;
  }
  return event_ids; 
}

/***************************************************/

Vec<TofPacket> get_tofpackets(const Vec<u8> &bytestream, u64 start_pos, PacketType filter) {
  Vec<TofPacket> packets;
  u64 pos  = start_pos;
  // just make sure in the beginning they
  // are not the same
  u64 last_pos = start_pos += 1;
  TofPacket packet;
  while (true) {
    packet = TofPacket::from_bytestream(bytestream, pos);
    if (pos != last_pos) {
      if (filter != PacketType::Unknown) {
        if (packet.packet_type != filter) {
          last_pos = pos;
          continue;
        }
      }
      packets.push_back(packet);
    } else {
      break;
    }
    last_pos = pos;
  }
  spdlog::debug("Read out {} packets from bytestream!", packets.size());
  return packets;
}

/***************************************************/

Vec<TofPacket> get_tofpackets(const String filename, PacketType filter) {
  spdlog::cfg::load_env_levels();
  if (!fs::exists(filename)) {
    spdlog::critical("Can't open {}! (it does not exist)");
  }

  auto stream = get_bytestream_from_file(filename); 
  bool has_ended = false;
  auto pos = search_for_2byte_marker(stream,0xAA, has_ended );
  if (has_ended) {
    spdlog::error("The stream ended before we found any header marker!");
  } else {
    spdlog::debug("Found the first header at pos {}", pos);
  }
  spdlog::debug("Read {} bytes from {}", stream.size(), filename);
  return get_tofpackets(stream, pos, filter);
}

/***************************************************/

Vec<TofEvent> unpack_tofevents_from_tofpackets(const Vec<u8> &bytestream, u64 start_pos) {
  Vec<TofEvent> events = Vec<TofEvent>();
  u64 pos  = start_pos;
  // just make sure in the beginning they
  // are not the same
  u64 last_pos = start_pos += 1;
  TofPacket packet;
  TofEvent event;
  while (true) {
    last_pos = pos;
    packet = TofPacket::from_bytestream(bytestream, pos);
    //if (n_packets == 100) {break;}
    if (pos != last_pos) {
      if (packet.packet_type == PacketType::TofEvent) {
        event = TofEvent::from_tofpacket(packet);
        events.push_back(event);
      }
    } else {
      break;
    }
  }
  spdlog::info("Read {} TofEvents!", events.size());
  return events;
}

/***************************************************/

Vec<TofEvent> unpack_tofevents_from_tofpackets(const String filename) {
  Vec<TofEvent> events = Vec<TofEvent>();
  auto stream = get_bytestream_from_file(filename); 
  spdlog::debug("Read {} bytes from {}", stream.size(), filename);
  bool has_ended = false;
  auto pos = search_for_2byte_marker(stream, 0xAA, has_ended );
  if (has_ended) {
    spdlog::error("Opened file {} but no start marker {} could be found indicating that this file is no good!", filename, TofPacket::HEAD);
    return events;
  }
  return unpack_tofevents_from_tofpackets(stream, pos);
}

/***************************************************/

Gaps::TofPacketReader::TofPacketReader() {
  // here it is exhausted because we did not 
  // set a file yet
  exhausted_  = true;
  n_packets_read_ = 0;
}

/***************************************************/

void Gaps::TofPacketReader::set_filename(String filename) {
  if (fs::exists(filename)) {
    filename_  = filename;
    exhausted_ = false;
    stream_file_ = std::ifstream(filename, std::ios::binary);   
    stream_file_.seekg (0, stream_file_.end);
    auto file_size = stream_file_.tellg();
    stream_file_.seekg (0, stream_file_.beg);
    auto fs_string = std::format("{:4.2f}", (f64)file_size/1e6);
    spdlog::info("Will read packets from {} [{} MB]", filename, fs_string);
  } else {
    auto msg = std::format("File {} does not exist!", filename);
    spdlog::critical(msg); 
    throw std::runtime_error(msg);
  }
}

/***************************************************/

Gaps::TofPacketReader::TofPacketReader(String filename) : Gaps::TofPacketReader() {
  set_filename(filename);
}

/***************************************************/

bool Gaps::TofPacketReader::is_exhausted() const {
  return exhausted_;
}

/***************************************************/

usize Gaps::TofPacketReader::n_packets_read() const {
  return n_packets_read_;
}

/***************************************************/

TofPacket Gaps::TofPacketReader::get_next_packet() {
  while (true) {
    if (stream_file_.eof()) {
      exhausted_ = true;
      throw std::runtime_error("No more packets in file!");
    } 
    u8 byte = stream_file_.get();
    if (byte == 0xAA) {
      byte = stream_file_.get();
      if (stream_file_.eof()) {
        exhausted_ = true;
        throw std::runtime_error("No more packets in file!");
      } 
      if (byte == 0xAA) {
        u8 packet_type = stream_file_.get();
        bytestream buffer = bytestream(4);
        stream_file_.read(reinterpret_cast<char*>(buffer.data()), 4);
        usize pos = 0;
        u32 p_size       = Gaps::parse_u32(buffer, pos);
        TofPacket packet;
        packet.packet_type  = static_cast<PacketType>(packet_type);
        packet.payload_size = p_size;
        buffer = bytestream(p_size);
        stream_file_.read(reinterpret_cast<char*>(buffer.data()), p_size);
        buffer.resize(stream_file_.gcount());
        packet.payload = std::move(buffer);
        n_packets_read_++;
        return packet;
      }
    } 
  }
}

/***************************************************/

String Gaps::TofPacketReader::get_filename() const {
  return filename_;
}

