#ifdef BUILD_CARASPACE

#include <filesystem>

#include "spdlog/spdlog.h"
#include "spdlog/cfg/env.h"

#include "logging.hpp"
#include "parsers.h"
#include "caraspace.hpp"

namespace fs = std::filesystem;

Gaps::CRFrame::CRFrame() {
};

std::map<std::string, std::tuple<u64, Gaps::CRFrameObjectType>> Gaps::CRFrame::parse_index(Vec<u8> stream, usize &pos) {
  std::map<std::string, std::tuple<u64, Gaps::CRFrameObjectType>> index;
  u8 idx_size = parse_u8(stream, pos);
  for (u8 k=0; k<idx_size; k++) {
    std::string name        = parse_string(stream, pos);
    u64 obj_pos             = parse_u64(stream, pos);
    CRFrameObjectType obj_t = static_cast<CRFrameObjectType>(parse_u8(stream, pos));
    auto value = std::tuple<u64, Gaps::CRFrameObjectType>(obj_pos, obj_t);
    index.insert(std::make_pair(name, value));
  }
  return index;
}

std::string Gaps::CRFrame::to_string() const {
  std::string repr = "<CRFrame : ";
  repr += std::format("\n  size  : {}", bytestorage.size() ); 
  repr += "\n  --- index ---";
  for (const auto& pair : index) {
    repr += std::format("\n  {} :  {}@{}", static_cast<u8>(std::get<1>(pair.second)) , pair.first, std::get<0>(pair.second));
  }
  repr += "\n>";
  return repr;
};


Gaps::CRFrame Gaps::CRFrame::from_bytestream(Vec<u8> stream, 
                                             usize &pos) {
  CRFrame frame;
  // FIXME - error checking
  u16 head    = parse_u16(stream, pos);
  if (head != CRFrame::HEAD) {
    log_error("CRFrame doesn't start with HEAD signature of " << CRFrame::HEAD);
    return frame;
  }
  u64 fr_size = parse_u64(stream, pos); 
  pos += fr_size;
  u16 tail    = parse_u16(stream, pos);
  if (tail != CRFrame::TAIL) {
    log_error("CRFrame doesn't conclude with TAIL signature of " << CRFrame::TAIL);
    return frame;
  }
  // now go back and get the content
  pos -= fr_size - 2; // wind back
  u64 size          = parse_u64(stream, pos);
  frame.index       = parse_index(stream, pos);
  Vec<u8> packet_bytestream(stream.begin()+ pos,
                            stream.begin()+ pos + size)  ;
  frame.bytestorage = packet_bytestream;
  return frame;
    //if stream.len() < 2 {
    //  return Err(CRSerializationError::HeadInvalid {});
    //}
    //let head = parse_u16(stream, pos);
    //if Self::CRHEAD != head {
    //  error!("FrameObject does not start with HEAD signature");
    //  return Err(CRSerializationError::HeadInvalid {});
    //}
    //let fr_size   = parse_u64(stream, pos) as usize; 
    //*pos += fr_size as usize;
    //let tail = parse_u16(stream, pos);
    //if Self::CRTAIL != tail {
    //  error!("FrameObject does not end with TAIL signature");
    //  return Err(CRSerializationError::TailInvalid {});
    //}
    //*pos -= fr_size - 2; // wind back
    //let mut frame = CRFrame::new();
    //let size    = parse_u64(stream, pos) as usize;
    //frame.index = Self::parse_index(stream, pos);
    //frame.bytestorage = stream[*pos..*pos + size].to_vec();
    //Ok(frame)
}

Gaps::CRReader::CRReader() {
};

Gaps::CRReader::CRReader(String filename) : CRReader::CRReader() {
  set_filename(filename);
}

std::string Gaps::CRReader::get_filename() const {
  return filename_;
}

void Gaps::CRReader::set_filename(std::string filename) {
  if (fs::exists(filename)) {
    filename_  = filename;
    exhausted_ = false;
    stream_file_ = std::ifstream(filename, std::ios::binary);   
    stream_file_.seekg (0, stream_file_.end);
    auto file_size = stream_file_.tellg();
    stream_file_.seekg (0, stream_file_.beg);
    auto fs_string = std::format("{:4.2f}", (f64)file_size/1e6);
    log_info("Will read packets from " << filename  << " [" << fs_string << " MB]");
  } else {
    auto msg = std::format("File {} does not exist!", filename);
    log_error(msg); 
    throw std::runtime_error(msg);
  }
}


bool Gaps::CRReader::is_exhausted() const {
  return exhausted_;
}

bool Gaps::CRReader::n_packets_read() const {
  return n_packets_read_;
}

Gaps::CRFrame Gaps::CRReader::get_next_frame() {
  while (true) { 
    if (stream_file_.eof()) {
      exhausted_ = true;
      throw std::runtime_error("No more frames in file!");
    } 
    u8 byte = stream_file_.get();
    if (byte == 0xAA) {
      byte = stream_file_.get();
      if (stream_file_.eof()) {
        exhausted_ = true;
        throw std::runtime_error("No more frames in file!");
      } 
      if (byte == 0xAA) {
        u8 packet_type = stream_file_.get();
        bytestream buffer = bytestream(4);
        stream_file_.read(reinterpret_cast<char*>(buffer.data()), 4);
        usize pos = 0;
        u32 p_size       = Gaps::parse_u32(buffer, pos);
        Gaps::CRFrame frame;
        //packet.packet_type  = static_cast<PacketType>(packet_type);
        //packet.payload_size = p_size;
        //buffer = bytestream(p_size);
        //stream_file_.read(reinterpret_cast<char*>(buffer.data()), p_size);
        //buffer.resize(stream_file_.gcount());
        //packet.payload = std::move(buffer);
        n_packets_read_++;
        return frame;
      }
    } 
  }
}


#endif 
