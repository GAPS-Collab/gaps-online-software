// This file is part of gaps-online-software and published 
// under the GPLv3 license

#include "io/telemetry_reader.hpp"
#include "spdlog/spdlog.h"
#include "spdlog/cfg/env.h"

namespace g = gondola;

//FIXME 
#include "caraspace.hpp"

auto g::TelemetryPacketReader::set_path(std::string pathname) -> void {
  auto files = g::list_path_contents_sorted(pathname);
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

Vec<std::string> g::TelemetryPacketReader::get_filenames() const {
  return filenames_;
}


