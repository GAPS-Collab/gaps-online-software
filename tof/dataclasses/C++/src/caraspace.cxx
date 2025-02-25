#ifdef BUILD_CARASPACE

#include <filesystem>

#include "spdlog/spdlog.h"
#include "spdlog/cfg/env.h"

#include "logging.hpp"
#include "parsers.h"
#include "caraspace.hpp"


namespace fs = std::filesystem;

//--------------------------------------------------

Gaps::CRFrameObject Gaps::CRFrameObject::from_bytestream(Vec<u8> stream, usize &pos) {
  auto f_obj = CRFrameObject();
  if (stream.size() < 2) {
    log_error("CRFrame::HeadInvalid");
    return f_obj;
    //return Err(CRSerializationError::HeadInvalid {});
  }
  auto head = parse_u16(stream, pos);
  if (head != CRFrameObject::HEAD) {
    log_error("CRFrame doesn't start with HEAD signature of " << CRFrame::HEAD);
    return f_obj;
  }
  
  f_obj.version     = parse_u8(stream, pos);
  f_obj.ftype      = static_cast<CRFrameObjectType>(parse_u8(stream, pos));
  auto payload_size = parse_u32(stream, pos);
  pos += payload_size; 
  auto tail = parse_u16(stream, pos);
  if (tail != CRFrameObject::TAIL) {
    log_error("Packet does not end with CRTAIL signature");
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
  repr += std::format("\n  size  : {}", static_cast<u8>(ftype) ); 
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
  pos += fr_size - 2; // count from the beginning
  std::cout << "fr size : " << fr_size << std::endl;
  u16 tail    = parse_u16(stream, pos);
  if (tail != CRFrame::TAIL) {
    log_error("CRFrame doesn't conclude with TAIL signature of " << CRFrame::TAIL);
    return frame;
  }
  // now go back and get the content
  pos -= fr_size - 2; // wind back, accounting for tail
  u64 size = parse_u64(stream, pos); // account for size
  std::cout << "size : " << size << std::endl;
  frame.index       = parse_index(stream, pos);
  Vec<u8> packet_bytestream(stream.begin()+ pos,
                            stream.begin()+ pos + size)  ;
  frame.bytestorage = packet_bytestream;
  return frame;
}

TofPacket Gaps::CRFrame::get_tofpacket(std::string name) {
  TofPacket tp;
  //let mut lookup : (usize, CRFrameObjectType);
  usize pos = 0;
  CRFrameObjectType dtype = CRFrameObjectType::Unknown;
  if (index.contains(name)) {
    pos   = std::get<0>(index.at(name));
    dtype = static_cast<CRFrameObjectType>(std::get<1>(index.at(name)));
  } else {
    log_error("Unable to find TofPacket " << name << " in frame!");
  }
  if (dtype == CRFrameObjectType::TofPacket) {
    auto f_obj = CRFrameObject::from_bytestream(bytestorage, pos);
    std::cout << f_obj.to_string() << std::endl;
    pos        = 0;
    tp         = TofPacket::from_bytestream(f_obj.payload, pos); 
    std::cout << tp << std::endl;
  } else {
    log_error("Trying to get TofPacket " << name << " however, that is of type " << static_cast<u8>(dtype)); 
    return tp;
  }
  return tp;
}

Gaps::TelemetryPacket Gaps::CRFrame::get_telemetrypacket(std::string name) {
  Gaps::TelemetryPacket tp;
  usize pos = 0;
  CRFrameObjectType dtype = CRFrameObjectType::Unknown;
  if (index.contains(name)) {
    pos   = std::get<0>(index.at(name));
    dtype = static_cast<CRFrameObjectType>(std::get<1>(index.at(name)));
  } else {
    log_error("Unable to find TelemetryPacket " << name << " in frame!");
  }
  if (dtype == CRFrameObjectType::TelemetryPacket) {
    auto f_obj = CRFrameObject::from_bytestream(bytestorage, pos);
    std::cout << f_obj.to_string() << std::endl;
    pos        = 0;
    tp         = TelemetryPacket::from_bytestream(f_obj.payload, pos); 
    std::cout << tp.to_string() << std::endl;
  } else {
    log_error("Trying to get TofPacket " << name << " however, that is of type " << static_cast<u8>(dtype)); 
    return tp;
  }
  return tp;
}

//------------------------------------------------------------

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
