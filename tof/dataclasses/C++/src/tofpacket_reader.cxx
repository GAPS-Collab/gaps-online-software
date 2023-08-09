#include <spdlog/spdlog.h>
#include <iostream>
#include <filesystem>

#include "tofpacket_reader.hpp"
#include "serialization.h"

namespace fs = std::filesystem;

// has to be larger than max packet size
const usize CHUNK_SIZE = 20000;

Vec<u8> read_chunk(const String& filename, usize offset) {

  Vec<u8> buffer;
  buffer.reserve(CHUNK_SIZE);
  

  char chunk[CHUNK_SIZE];
  std::ifstream file(filename, std::ios::binary);
  file.seekg(offset);
  while (file.read(chunk, CHUNK_SIZE)) {
    buffer.insert(buffer.end(), chunk, chunk + file.gcount());
  }

  if (file.eof()) {
    // Reached the end of the file
    buffer.insert(buffer.end(), chunk, chunk + file.gcount());
  } else if (!file) {
    // Error occurred while reading the file
    spdlog::error("Failed to read file: {}", filename);
    buffer.clear();
  }
  return buffer;
}


Gaps::TofPacketReader::TofPacketReader(String filename) {
  if (fs::exists(filename)) {
    spdlog::info("Will read packets from {}", filename);
    filename_ = filename;
  } else {
    spdlog::error("File {} does not exist!", filename); 
    filename_ = "";
    return;
  }
  stream_file_ = std::ifstream(filename_);   
  std::streampos file_s = stream_file_.tellg();
  file_size_ = static_cast<usize>(file_s);
  nchunks_ = file_size_ / CHUNK_SIZE;
  current_pos_ = 0;
  last_packet_ = TofPacket();
}

void Gaps::TofPacketReader::process_chunk() {
  auto stream = read_chunk(filename_, current_pos_);
  bool has_ended = false;
  u64 head_pos = search_for_2byte_marker(stream, 0xAA, has_ended);
  if (!(has_ended)) {
    stream = read_chunk(filename_, head_pos);
    last_packet_ = TofPacket::from_bytestream(stream, head_pos);  
  } else {
    last_packet_ = TofPacket();
  }
}

String Gaps::TofPacketReader::get_filename() const {
  return filename_;
}

TofPacket Gaps::TofPacketReader::get_next_packet() {
    process_chunk();
    return last_packet_;
}
