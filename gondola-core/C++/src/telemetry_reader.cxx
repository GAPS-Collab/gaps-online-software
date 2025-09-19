// This file is part of gaps-online-software and published 
// under the GPLv3 license

#include "io/telemetry_reader.hpp"
#include "spdlog/spdlog.h"
#include "spdlog/cfg/env.h"

namespace g = gondola;

//FIXME 
#include "caraspace.hpp"
#include "io/parsers.h"


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

g::TelemetryPacketReader::TelemetryPacketReader() : 
  exhausted_      (0),
  //n_packets_read_ (0),
  filenames_      (Vec<std::string>()),
  file_idx_       (0) {
  //#ifdef BUILD_CXXDB
  //spdlog::info("Will load tofpaddles from DB for this reader!");
  //paddles_ = Gaps::get_tofpaddles();
  //#endif 
};

g::TelemetryPacketReader::TelemetryPacketReader(String pathname) : TelemetryPacketReader::TelemetryPacketReader() {
  set_path(pathname);
}

auto g::TelemetryPacketReader::get_filenames() const -> Vec<std::string> {
  return filenames_;
}

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

auto g::TelemetryPacketReader::get_next_packet() -> Gaps::Telemetry::Packet {
  auto packet = Gaps::Telemetry::Packet();
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
        // because bytestream does not contain header, size is at 5 
        usize pos = 5;
        //u64 p_size;
        u16 p_size       = Gaps::parse_u16(buffer, pos);
        payload.insert(payload.end(), buffer.begin(), buffer.end());
        // now we just need to append p_size bytes
        Vec<u8> buffer_data = bytestream(p_size);
        stream_file_.read(reinterpret_cast<char*>(buffer_data.data()), p_size);
        payload.insert(payload.end(), buffer_data.begin(), buffer_data.end());
        pos = 0;
        auto packet = Gaps::Telemetry::Packet::from_bytestream(payload, pos);
        ++n_packs_read_;
        return packet;
      }
    } 
  }
  return packet;
}

bool g::TelemetryPacketReader::is_exhausted() const {
  return exhausted_;
}


