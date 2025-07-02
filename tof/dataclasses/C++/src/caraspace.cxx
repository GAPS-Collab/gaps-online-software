#ifdef BUILD_CARASPACE

#include <iostream>
#include <filesystem>
#include <vector>
#include <regex>
#include <string>
#include <algorithm>

#include "spdlog/spdlog.h"
#include "spdlog/cfg/env.h"

#include "logging.hpp"
#include "parsers.h"
#include "caraspace.hpp"

namespace g    = Gaps;
namespace fs   = std::filesystem;
namespace gtel = Gaps::Telemetry;

using namespace result;

std::vector<std::string> Gaps::list_path_contents_sorted(const std::string& input) {
  fs::path path(input);
  std::vector<std::string> result;

  if (!fs::exists(path)) {
    std::cerr << "Error: Path does not exist." << std::endl;
    return result;
  }

  if (fs::is_regular_file(path)) {
    result.push_back(path.string());
    return result;
  }

  if (fs::is_directory(path)) {
    std::regex re(R"(Run\d+_\d+\.(\d{6})_(\d{6})UTC\.gaps$)");
    std::vector<std::tuple<uint32_t, uint32_t, std::string>> entries;

    for (const auto& entry : fs::directory_iterator(path)) {
      if (entry.is_regular_file()) {
        std::string filename = entry.path().string();
        std::smatch match;
        if (std::regex_search(filename, match, re) && match.size() > 2) {
          try {
            u32 date = std::stoul(match[1].str());
            u32 time = std::stoul(match[2].str());
            entries.emplace_back(date, time, filename);
          } catch (const std::exception&) {
            continue;
          }
        }
      }
    }

    std::sort(entries.begin(), entries.end(), [](const auto& a, const auto& b) {
      return std::tie(std::get<0>(a), std::get<1>(a)) < std::tie(std::get<0>(b), std::get<1>(b));
    });

    for (const auto& entry : entries) {
      result.push_back(std::get<2>(entry));
    }
  } else {
    std::cerr << "Error: Path is neither a file nor a directory." << std::endl;
  }
  return result;
}

//--------------------------------------------------

Gaps::CRFrameObject Gaps::CRFrameObject::from_bytestream(Vec<u8> stream, usize &pos) {
  auto f_obj = CRFrameObject();
  if (stream.size() < 2) {
    spdlog::error("CRFrame::HeadInvalid");
    return f_obj;
    //return Err(CRSerializationError::HeadInvalid {});
  }
  auto head = parse_u16(stream, pos);
  if (head != HEAD) {
    //FIXME - throws linker error - but why?
    //spdlog::error("CRFrameObject doesn't start with HEAD signature of {}!", HEAD);
    return f_obj;
  }
  
  f_obj.version     = parse_u8(stream, pos);
  f_obj.ftype       = static_cast<CRFrameObjectType>(parse_u8(stream, pos));
  auto payload_size = parse_u32(stream, pos);
  pos += payload_size; 
  auto tail = parse_u16(stream, pos);
  if (tail != CRFrameObject::TAIL) {
    spdlog::error("Packet does not end with CRTAIL signature");
    return f_obj;
  }
  pos -= 2; // for tail parsing
  pos -= payload_size;
  auto buffer    = Gaps::slice(stream, pos, pos + payload_size ); 
  f_obj.payload = buffer; 
  return f_obj;
}

std::string Gaps::CRFrameObject::to_string() {
  std::string repr = "<CRFrameObject";
  usize p_len = payload.size();
  // FIXME - implement the string representation for ftype
  repr += std::format("\n  type  : {}", static_cast<u8>(ftype) ); 
  if (p_len >= 8) {
    repr += std::format("\n  payload ({} bytes) : [{} {} {} {} .. {} {} {} {}]", 
       p_len,
       payload[0],
       payload[1],
       payload[2],
       payload[3],
       payload[p_len - 4],
       payload[p_len - 3],
       payload[p_len - 2],
       payload[p_len - 1]);     
  } else {
    repr += std::format("\n payload ({} bytes)", p_len);
  }
  return repr;
}

// ---------------------------------------------------------------

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

auto Gaps::CRFrame::to_string() const -> std::string {
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
    spdlog::error("CRFrame doesn't start with HEAD signature of {}!", CRFrame::HEAD);
    return frame;
  }
  u64 fr_size = parse_u64(stream, pos); 
  pos += fr_size - 2; // count from the beginning
  //std::cout << "fr size : " << fr_size << std::endl;
  u16 tail    = parse_u16(stream, pos);
  if (tail != CRFrame::TAIL) {
    spdlog::error("CRFrame doesn't conclude with TAIL signature of {}!", CRFrame::TAIL);
    return frame;
  }
  // now go back and get the content
  pos -= fr_size - 2; // wind back, accounting for tail
  u64 size = parse_u64(stream, pos); // account for size
  //std::cout << "size : " << size << std::endl;
  frame.index       = parse_index(stream, pos);
  Vec<u8> packet_bytestream(stream.begin()+ pos,
                            stream.begin()+ pos + size)  ;
  frame.bytestorage = packet_bytestream;
  return frame;
}

auto Gaps::CRFrame::get_tofpacket(std::string name)
  -> Result<TofPacket,g::IOError> {
  TofPacket tp;
  //let mut lookup : (usize, CRFrameObjectType);
  usize pos = 0;
  CRFrameObjectType dtype = CRFrameObjectType::Unknown;
  if (index.contains(name)) {
    pos   = std::get<0>(index.at(name));
    dtype = static_cast<CRFrameObjectType>(std::get<1>(index.at(name)));
  } else {
     spdlog::debug("Unable to find TofPacket {} in frame!", name);
     std::string msg = std::format("Can't find TofPacket {} in frame!", name);
     auto err = g::IOError(g::IOError::ErrorKind::PacketNotFound, msg);
     return Err(err);
  }
  if (dtype == CRFrameObjectType::TofPacket) {
    auto f_obj = CRFrameObject::from_bytestream(bytestorage, pos);
    //std::cout << f_obj.to_string() << std::endl;
    pos        = 0;
    auto tdata = TofPacket::from_bytestream(f_obj.payload, pos); 
    if (tdata.is_err()) {
      return tdata;
    }
    tp = tdata.unwrap();
    //std::cout << tp << std::endl;
  } else {
    std::string msg = std::format("Trying to get TofPacket {}, but it is of type {}", name, (int)static_cast<u8>(dtype));
    SPDLOG_DEBUG(msg);
    auto err = g::IOError(g::IOError::ErrorKind::WrongPacketType, msg);
    return Err(err);
  }
  return Ok(tp);
}

gtel::Packet Gaps::CRFrame::get_telemetrypacket(std::string name) {
  gtel::Packet tp;
  usize pos = 0;
  CRFrameObjectType dtype = CRFrameObjectType::Unknown;
  if (index.contains(name)) {
    pos   = std::get<0>(index.at(name));
    dtype = static_cast<CRFrameObjectType>(std::get<1>(index.at(name)));
  } else {
    spdlog::error("Unable to find TelemetryPacket {} in frame!", name);
  }
  if (dtype == CRFrameObjectType::TelemetryPacket) {
    auto f_obj = CRFrameObject::from_bytestream(bytestorage, pos);
    //std::cout << f_obj.to_string() << std::endl;
    pos        = 0;
    tp         = gtel::Packet::from_bytestream(f_obj.payload, pos); 
    //std::cout << tp.to_string() << std::endl;
  } else {
    log_error("Trying to get TofPacket " << name << " however, that is of type " << static_cast<u8>(dtype)); 
    return tp;
  }
  return tp;
}

//------------------------------------------------------------

Gaps::CRReader::CRReader() : 
  exhausted_      (0),
  n_packets_read_ (0),
  filenames_      (Vec<std::string>()),
  fileindex_      (0) {
  #ifdef BUILD_CXXDB
  spdlog::info("Will load tofpaddles from DB for this reader!");
  paddles_ = Gaps::get_tofpaddles();
  #endif 
};

Gaps::CRReader::CRReader(String pathname) : CRReader::CRReader() {
  set_path(pathname);
}

Vec<std::string> Gaps::CRReader::get_filenames() const {
  return filenames_;
}
    
auto Gaps::CRReader::get_rbcalibrations(u8 n_rb) -> RBCalibrationMap {
  RBCalibrationMap cali_map;
  auto frame = Gaps::CRFrame();
  std::string calipackname = "PacketType.RBCalibration";
  while (!is_exhausted()) {
    try {
      frame = get_next_frame();
    } catch (const std::exception& e) {
      std::string emessage = std::format("--> Exception '{}' caught!", e.what());
      //std::string message = std::format("--> File {} with {} frames processed! In     total, we proceseed {} frames", l0file, n_frames_processed_file, n_frames_processe    d);
      std::cout << emessage << std::endl;
      //std::cout << message << std::endl;
      break;
    }
    if (cali_map.size() == (usize)n_rb) {
      break;
    }
    if (frame.index.contains(calipackname)) {
      u64 pos = 0;
      auto cali_pack = frame.get_tofpacket(calipackname);
      if (cali_pack.is_ok()) {
        auto rb_cali   = RBCalibration::from_bytestream(cali_pack.unwrap().payload, pos);
        cali_map.insert(std::make_pair(rb_cali.rb_id, rb_cali));
        ++n_rb;
      } // FIXME error check!
    } 
  }
  return cali_map; 
};     

void Gaps::CRReader::set_path(std::string pathname) {
  auto files = list_path_contents_sorted(pathname);
  if (files.size() > 0) {
    filenames_   = files;
    exhausted_   = false;
    fileindex_   = 0;
    stream_file_ = std::ifstream(files[0], std::ios::binary);   
    stream_file_.seekg (0, stream_file_.end);
    auto file_size = stream_file_.tellg();
    stream_file_.seekg (0, stream_file_.beg);
    auto fs_string = std::format("{:4.2f}", (f64)file_size/1e6);
    spdlog::info("Will read packets from {} [{} MB]", files[0], fs_string);
  }
}

bool Gaps::CRReader::is_exhausted() const {
  return exhausted_;
}

bool Gaps::CRReader::n_packets_read() const {
  return n_packets_read_;
}

void Gaps::CRReader::prime_next_file_() {
  if (fileindex_ > filenames_.size() - 2) { // -2 because -1 is the last index
    fileindex_ += 1;
    // we simply open the next file
    stream_file_ = std::ifstream(filenames_[fileindex_], std::ios::binary);   
    stream_file_.seekg (0, stream_file_.beg);
  } else {
    exhausted_ = true;
    spdlog::info("CRReader is exhausted!");
    throw std::runtime_error("CRReader is exhausted!");
  }
}

Gaps::CRFrame Gaps::CRReader::get_next_frame() {
  while (true) { 
    if (stream_file_.eof()) {
      //std::cout << "ex 1" << std::endl;
      prime_next_file_();
      return get_next_frame();
    } 
    u8 byte = stream_file_.get();
    if (byte == 0xAA) {
      byte = stream_file_.get();
      if (stream_file_.eof()) {
        //std::cout << "ex 2" << std::endl;
        exhausted_ = true;
        prime_next_file_();
        return get_next_frame();
      } 
      if (byte == 0xAA) {
        Vec<u8> payload = {0xAA, 0xAA};
        //u8 packet_type = stream_file_.get();
        Vec<u8> buffer = bytestream(8);
        stream_file_.read(reinterpret_cast<char*>(buffer.data()), 8);
        usize pos = 0;
        //u64 p_size;
        u64 p_size       = Gaps::parse_u64(buffer, pos);
        payload.insert(payload.end(), buffer.begin(), buffer.end());
        buffer = bytestream(p_size);
        stream_file_.read(reinterpret_cast<char*>(buffer.data()), p_size);
        payload.insert(payload.end(), buffer.begin(), buffer.end());
        u64 pos_in_frame = 0;
        // from_bytestream is broken
        //auto frame = Gaps::CRFrame::from_bytestream(payload, pos_in_frame);
        auto frame = CRFrame();
        frame.index = CRFrame::parse_index(buffer, pos_in_frame);
        buffer = Gaps::slice(buffer, pos_in_frame, p_size); 
        frame.bytestorage = std::move(buffer);
        n_packets_read_++;
        return frame;
      }
    } 
  }
}


#endif 
