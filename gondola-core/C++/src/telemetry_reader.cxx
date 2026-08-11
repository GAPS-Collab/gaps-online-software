// This file is part of gaps-online-software and published 
// under the GPLv3 license

#include <algorithm>

#include "spdlog/spdlog.h"
#include "spdlog/cfg/env.h"

//FIXME 
#include "caraspace.hpp"
#include "io/parsers.h"
#include "io/telemetry_reader.hpp"
#ifdef BUILD_CXX_DB
#include "database.h"
#endif 

namespace g = gondola;

//--------------------------------------------------------------------------

auto g::TelemetryPacketReader::set_path(std::string pathname) -> void {
  auto files = g::list_path_contents_sorted(pathname, true);
  if (files.size() > 0) {
    filenames_   = files;
    exhausted_   = false;
    file_idx_    = 0;
    stream_file_ = std::ifstream(files[0], std::ios::binary);   
    stream_file_.seekg (0, stream_file_.end);
    auto file_size = stream_file_.tellg();
    stream_file_.seekg (0, stream_file_.beg);
    auto fs_string = std::format("{:4.2f}", (f64)file_size/1e6);
    spdlog::info("Will read packets from {} [{} MB]", files[0], fs_string);
  }
}

//--------------------------------------------------------------------------

g::TelemetryPacketReader::TelemetryPacketReader() : 
                                                    exhausted_      (0),
                                                    //n_packets_read_ (0),
                                                    filenames_      (Vec<std::string>()),
                                                    file_idx_       (0) {
  #ifdef BUILD_CXX_DB
  spdlog::info("Will load tofpaddles from DB for this reader!");
  paddles = std::make_shared<g::TofPaddleMap>(g::get_tofpaddles());
  #endif 
};

//--------------------------------------------------------------------------

g::TelemetryPacketReader::TelemetryPacketReader(String pathname) : TelemetryPacketReader::TelemetryPacketReader() {
  set_path(pathname);
}

//--------------------------------------------------------------------------

auto g::TelemetryPacketReader::get_filenames() const -> Vec<std::string> {
  return filenames_;
}

//--------------------------------------------------------------------------

auto g::TelemetryPacketReader::prime_next_file_() -> void {
  if (file_idx_ > filenames_.size() - 2) { // -2 because -1 is the last index
    file_idx_ += 1;
    // we simply open the next file
    stream_file_ = std::ifstream(filenames_[file_idx_], std::ios::binary);   
    stream_file_.seekg (0, stream_file_.beg);
  } else {
    exhausted_ = true;
    spdlog::info("TelemetryPacketReader exhausted the packets in the current file!");
    throw std::runtime_error("TelemetryPacketReader exhausted the packets in the current file!");
  }
}

//--------------------------------------------------------------------------

auto g::TelemetryPacketReader::count_packets() -> u64 {
  u64 npacks = 0;
  while (!is_exhausted()) {
    get_next_packet();
    ++npacks;
  }
  return npacks;
}

//--------------------------------------------------------------------------

auto g::TelemetryPacketReader::register_packet_type_(g::TelemetryPacketType const &ptype) -> void {
   if (packet_index_.contains(ptype)) {
     ++packet_index_[ptype];
   } else {
     packet_index_.insert(std::make_pair(ptype,1)); 
   }
} 

//--------------------------------------------------------------------------

auto g::TelemetryPacketReader::cache_all_packets() -> void {
  in_caching_ = true;
  while (!exhausted_) {
    auto packet = get_next_packet();
    packet_cache_.push_back(packet);
  }

  // Sort packet cache by timestamp and packet counter
  std::sort(packet_cache_.begin(), packet_cache_.end(),
    [](const g::TelemetryPacket& a, const g::TelemetryPacket& b) {
    if (a.header.timestamp != b.header.timestamp) {
        return a.header.get_gcutime() > b.header.get_gcutime();
    }
    return a.header.counter < b.header.counter;
  });

  in_caching_ = false;
}

//--------------------------------------------------------------------------

auto g::TelemetryPacketReader::get_packet_index() const -> const HashMap<g::TelemetryPacketType, u64>& {
  return packet_index_;
}

//--------------------------------------------------------------------------

auto g::TelemetryPacketReader::get_next_packet() -> g::TelemetryPacket {
  auto packet = g::TelemetryPacket();
  if (packet_cache_.size() > 0 && !in_caching_) { // packets have been cached, return those
    packet = packet_cache_.back();
    packet_cache_.pop_back();
    #ifdef BUILD_CXX_DB 
    packet.paddles = paddles;
    #endif 
    return packet;
  }
  while (true) { 
    if (stream_file_.eof()) {
      prime_next_file_();
      return get_next_packet();
    } 
    u8 byte = stream_file_.get();
    if (byte == 0xeb) {
      // first byte of the header found
      byte = stream_file_.get();
      if (stream_file_.eof()) {
        //std::cout << "ex 2" << std::endl;
        exhausted_ = true;
        prime_next_file_();
        return get_next_packet();
      } 
      if (byte == 0x90) {
        // we need to skip 7 bytes and then read 2 bytes for 
        // the size, then we can that combine with the 13 bytes
        // for the header and get all at once
        // 
        // so that we don't have to jump back and forth, let's
        // first get the 13 bytes
        Vec<u8> payload = {0xeb, 0x90};
        //u8 packet_type = stream_file_.get();
        Vec<u8> buffer = bytestream(11);
        stream_file_.read(reinterpret_cast<char*>(buffer.data()), 11);
        // because bytestream does not contain header, size is at 7 instead 
        // of 9 
        usize pos   = 7;
        // reminder! The size is the size including the 13bytes header, so we 
        // need to subtrackt that 
        u16 p_size  = g::parse_u16(buffer, pos) - 13;
        payload.insert(payload.end(), buffer.begin(), buffer.end());
        // now we just need to append p_size bytes
        Vec<u8> buffer_data = bytestream(p_size);
        stream_file_.read(reinterpret_cast<char*>(buffer_data.data()), p_size);
        payload.insert(payload.end(), buffer_data.begin(), buffer_data.end());
        pos = 0;
        auto packet = g::TelemetryPacket::from_bytestream(payload, pos);
        ++n_packs_read_;
        register_packet_type_(packet.header.ptype);
        #ifdef BUILD_CXX_DB 
        packet.paddles = paddles;
        #endif 
        return packet;
      }
    } 
  }
  #ifdef BUILD_CXX_DB 
  packet.paddles = paddles;
  #endif 
  return packet;
}

//--------------------------------------------------------------------------

auto g::TelemetryPacketReader::is_exhausted() const -> bool {
  return exhausted_;
}

//--------------------------------------------------------------------------

auto g::TelemetryPacketReader::print_packet_index() const -> void {
  for (auto const &pair : packet_index_) {
    std::cout << " -- " << g::bfsw_ptype_to_str(pair.first) << " -> " << pair.second << std::endl;
  }
}

//--------------------------------------------------------------------------

auto g::TelemetryPacketReader::rewind() -> void {
  exhausted_   = false;
  file_idx_    = 0;
  stream_file_ = std::ifstream(filenames_[0], std::ios::binary);   
  stream_file_.seekg (0, stream_file_.end);
  auto file_size = stream_file_.tellg();
}


