#include <iostream>
#include <filesystem>
#include <vector>
#include <string>
#include <algorithm>
#include<map> 

#include "spdlog/spdlog.h"
#include "spdlog/cfg/env.h"

#include "io/parsers.h"
#include "caraspace.hpp"
#include "io.hpp"
#include "io/telemetry_reader.hpp"
#include "io/parsers.h" 

namespace g    = gondola;
namespace fs   = std::filesystem;

using namespace result;

//--------------------------------------------------

auto g::string_to_bytes(std::string value) -> Vec<u8> {
  Vec<u8> stream;
  u16 string_size = static_cast<u16>(value.length());
  u8 size_bytes[2];
  size_bytes[0] = static_cast<u8>(string_size & 0xFF);         // Lower byte
  size_bytes[1] = static_cast<u8>((string_size >> 8) & 0xFF);  // Upper byte
  stream.reserve(sizeof(u16) + value.length());
  stream.push_back(size_bytes[0]);
  stream.push_back(size_bytes[1]);
  stream.insert(stream.end(), value.begin(), value.end());
  return stream;
}

//--------------------------------------------------
  
auto g::get_runfilename(u32 run, u32 subrun, bool is_sim, Option<std::string> timestamp = None) -> std::string {
  std::string ts  = "";
  auto fname = std::string();
  if (timestamp.is_some()) {
    ts = timestamp.unwrap();
  }
  if (ts != "") { 
    fname = std::format("Run{}_{}.{}.",run,subrun,ts);
  } else { 
    fname = std::format("Run{}_{}.", run,subrun);
  }
  if (is_sim) {
    fname += "sim.gaps";
  } else {
    fname += ".gaps";
  }
  return fname;
}

//--------------------------------------------------

auto g::CRFrameObject::to_bytestream() const -> Vec<u8> { 
  Vec<u8> stream;
  // remember to be compatible with rust!
  stream.push_back(0xAA);
  stream.push_back(0xAA);
  stream.push_back((u8)version);
  stream.push_back((u8)ftype);
  auto size = g::to_le_bytes((u32)payload.size());
  stream.insert(stream.end(), size.begin(), size.end());
  stream.insert(stream.end(),payload.begin(), payload.end());
  stream.push_back(0x55);
  stream.push_back(0x55);
  return stream;
}

//--------------------------------------------------

g::CRFrameObject g::CRFrameObject::from_bytestream(Vec<u8> stream, usize &pos) {
  auto f_obj = g::CRFrameObject();
  if (stream.size() < 2) {
    spdlog::error("CRFrame::HeadInvalid");
    return f_obj;
    //return Err(CRSerializationError::HeadInvalid {});
  }
  auto head = g::parse_u16(stream, pos);
  if (head != HEAD) {
    //FIXME - throws linker error - but why?
    //spdlog::error("CRFrameObject doesn't start with HEAD signature of {}!", HEAD);
    return f_obj;
  }
  
  f_obj.version     = g::parse_u8(stream, pos);
  f_obj.ftype       = static_cast<g::CRFrameObjectType>(g::parse_u8(stream, pos));
  auto payload_size = g::parse_u32(stream, pos);
  pos += payload_size; 
  auto tail = g::parse_u16(stream, pos);
  if (tail != CRFrameObject::TAIL) {
    spdlog::error("Packet does not end with CRTAIL signature");
    return f_obj;
  }
  pos -= 2; // for tail parsing
  pos -= payload_size;
  auto buffer   = g::slice(stream, pos, pos + payload_size ); 
  f_obj.payload = buffer; 
  return f_obj;
}

//--------------------------------------------------

auto g::CRFrameObject::to_string() -> std::string {
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
    repr += std::format("\n payload ({} bytes)>", p_len);
  }
  return repr;
}

// ---------------------------------------------------------------

std::map<std::string, std::tuple<u64, g::CRFrameObjectType>> g::CRFrame::parse_index(Vec<u8> stream, usize &pos) {
  std::map<std::string, std::tuple<u64, g::CRFrameObjectType>> index;
  u8 idx_size        = g::parse_u8(stream, pos);
  for (u8 k=0; k<idx_size; k++) {
    std::string name = g::parse_string(stream, pos);
    u64 obj_pos      = g::parse_u64(stream, pos);
    g::CRFrameObjectType obj_t = static_cast<g::CRFrameObjectType>(g::parse_u8(stream, pos));
    auto value = std::tuple<u64, g::CRFrameObjectType>(obj_pos, obj_t);
    index.insert(std::make_pair(name, value));
  }
  return index;
}

//--------------------------------------------------

auto g::CRFrame::to_string() const -> std::string {
  std::string repr = "<CRFrame : ";
  repr += std::format("\n  size  : {}", bytestorage.size() ); 
  repr += "\n  --- index ---";
  for (const auto& pair : index) {
    repr += std::format("\n  {} :  {}@{}", static_cast<u8>(std::get<1>(pair.second)) , pair.first, std::get<0>(pair.second));
  }
  repr += "\n>";
  return repr;
};

//--------------------------------------------------

auto g::CRFrame::put_fobject(g::CRFrameObject const &fobj, std::string name) -> void {
  u64 pos = bytestorage.size();
  //std::cout << "Have bytestorage size of " << pos << std::endl;
  index[name] = std::tuple<u64, CRFrameObjectType>(pos, fobj.ftype);
  auto bytes = fobj.to_bytestream();
  bytestorage.insert(bytestorage.end(), bytes.begin(), bytes.end());
}

//--------------------------------------------------

auto g::CRFrame::from_bytestream(Vec<u8> stream, usize &pos)
   -> g::CRFrame {
  CRFrame frame;
  // FIXME - error checking
  u16 head  = g::parse_u16(stream, pos);
  if (head != CRFrame::HEAD) {
    spdlog::error("CRFrame doesn't start with HEAD signature of {}!", CRFrame::HEAD);
    return frame;
  }
  u64 fr_size = g::parse_u64(stream, pos); 
  pos += fr_size - 2; // count from the beginning
  //std::cout << "fr size : " << fr_size << std::endl;
  u16 tail  = g::parse_u16(stream, pos);
  if (tail != CRFrame::TAIL) {
    spdlog::error("CRFrame doesn't conclude with TAIL signature of {}!", CRFrame::TAIL);
    return frame;
  }
  // now go back and get the content
  pos -= fr_size - 2; // wind back, accounting for tail
  u64 size = g::parse_u64(stream, pos); // account for size
  //std::cout << "size : " << size << std::endl;
  frame.index       = parse_index(stream, pos);
  Vec<u8> packet_bytestream(stream.begin()+ pos,
                            stream.begin()+ pos + size)  ;
  frame.bytestorage = packet_bytestream;
  return frame;
}

//--------------------------------------------------

auto g::CRFrame::serialize_index() const -> Vec<u8> {
  Vec<u8> s_index = {};
  // we do not support frames with more than 
  // 255 objects (mostly for the reason of 
  // keeping things not too busy?)
  u8 idx_size  = index.size();
  s_index.push_back(idx_size);
  for (const auto& pair : index) {
    auto k      = pair.first; 
    auto s_name = g::string_to_bytes(k);
    auto s_pos  = g::to_le_bytes((u64)std::get<0>(pair.second));
    s_index.insert(std::end(s_index), s_name.cbegin(), s_name.cend());
    s_index.insert(std::end(s_index), s_pos.cbegin(), s_pos.cend());
    s_index.push_back(static_cast<u8>(std::get<1>(pair.second)));
  }
  return s_index; 
}

//--------------------------------------------------

auto g::CRFrame::to_bytestream() const -> Vec<u8> {
  Vec<u8> stream = {}; 
  auto s_index = serialize_index();
  auto head    = g::to_le_bytes(HEAD); 
  stream.insert(std::end(stream), head.cbegin(), head.cend());
  auto size = g::to_le_bytes((u64) (bytestorage.size() + s_index.size()));

  stream.insert(std::end(stream), size.cbegin(), size.cend()); 
  stream.insert(std::end(stream), s_index.cbegin(), s_index.cend()); 
  stream.insert(std::end(stream), bytestorage.cbegin(), bytestorage.cend());
  auto tail    = g::to_le_bytes(TAIL);
  stream.insert(std::end(stream), tail.cbegin(), tail.cend());
  return stream;
}

//--------------------------------------------------

auto g::CRFrame::get_tofpacket(std::string name)
  -> Result<TofPacket,g::IOError> {
  TofPacket tp;
  //let mut lookup : (usize, CRFrameObjectType);
  usize pos = 0;
  g::CRFrameObjectType dtype = g::CRFrameObjectType::Unknown;
  if (index.contains(name)) {
    pos   = std::get<0>(index.at(name));
    dtype = static_cast<g::CRFrameObjectType>(std::get<1>(index.at(name)));
  } else {
     spdlog::debug("Unable to find TofPacket {} in frame!", name);
     std::string msg = std::format("Can't find TofPacket {} in frame!", name);
     auto err = g::IOError(g::IOError::ErrorKind::PacketNotFound, msg);
     return Err(err);
  }
  if (dtype == g::CRFrameObjectType::TofPacket) {
    auto f_obj = g::CRFrameObject::from_bytestream(bytestorage, pos);
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

g::TelemetryPacket g::CRFrame::get_telemetrypacket(std::string name) {
  g::TelemetryPacket tp;
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
    tp         = g::TelemetryPacket::from_bytestream(f_obj.payload, pos); 
    //std::cout << tp.to_string() << std::endl;
  } else {
    spdlog::error("Trying to get TofPacket {}  however, that is of type {}", name, static_cast<u8>(dtype)); 
    return tp;
  }
  return tp;
}

//----------------------------------------------------------

g::CRReader::CRReader() : 
  exhausted_         (0),
  n_packets_read_    (0),
  filenames_         (Vec<std::string>()),
  fileindex_         (0),
  is_from_telemetry_ (false) {
  telly_reader_ = std::unique_ptr<TelemetryPacketReader>(new TelemetryPacketReader());
  #ifdef BUILD_CXX_DB
  spdlog::info("Will load tofpaddles from DB for this reader!");
  paddles_ = g::get_tofpaddles();
  #endif 
};

//--------------------------------------------------

g::CRReader::CRReader(String pathname) : CRReader::CRReader() {
  set_path(pathname);
}

//--------------------------------------------------

Vec<std::string> g::CRReader::get_filenames() const {
  return filenames_;
}

//--------------------------------------------------
    
auto g::CRReader::get_rbcalibrations(u8 n_rb) -> g::RBCalibrationMap {
  g::RBCalibrationMap cali_map;
  auto frame = g::CRFrame();
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
        auto rb_cali   = g::RBCalibration::from_bytestream(cali_pack.unwrap().payload, pos);
        cali_map.insert(std::make_pair(rb_cali.rb_id, rb_cali));
        ++n_rb;
      } // FIXME error check!
    } 
  }
  return cali_map; 
};     

//--------------------------------------------------

void g::CRReader::set_path(std::string pathname) {
  auto files = list_path_contents_sorted(pathname);
  if (files.size() == 0) {
    spdlog::warn("We did not see any files matching the filenames for L0!");
    spdlog::warn("Trying to look for telemetry ('.bin') files instead...");
    files = list_path_contents_sorted(pathname, true);
    if (files.size() > 0) {
      spdlog::info("Found {} telemetry files at {}!", files.size(), pathname);
      is_from_telemetry_ = true;
      telly_reader_ = std::unique_ptr<TelemetryPacketReader>(new TelemetryPacketReader(pathname));
      filenames_ = files;
    } 
  }
  if (files.size() > 0 && !is_from_telemetry_) {
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

//--------------------------------------------------

auto g::CRReader::is_exhausted() const -> bool {
  if (is_from_telemetry_) {
    return telly_reader_->is_exhausted();
  }
  return exhausted_;
}

//--------------------------------------------------

auto g::CRReader::is_from_telemetry() const -> bool {
  return is_from_telemetry_;
}

//--------------------------------------------------
    
auto g::CRReader::get_telemetry_packet_index() const -> const HashMap<TelemetryPacketType, u64> & {
  return telly_reader_->get_packet_index(); 
}

//--------------------------------------------------

auto g::CRReader::cache_telemetry_first() -> void {
  if (is_from_telemetry_) {
    telly_reader_->cache_all_packets();
  }
}

//--------------------------------------------------

auto g::CRReader::n_packets_read() const -> bool {
  return n_packets_read_;
}

//--------------------------------------------------

auto g::CRReader::prime_next_file_() -> void {
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

//--------------------------------------------------

auto g::CRReader::get_next_frame() -> g::CRFrame {
  // there are 2 "modes" - this is either from 
  // telemetry files or not. If it is from 
  // telemetry, we do the following
  // 1) unpack the telemetry 
  // 2) create a new frame with a TelemetryPacket  
  //    (and an upsampled TofEvent in it in case 
  //     it is a MergedEvent) 
  if (is_from_telemetry_) {
    auto packet = telly_reader_->get_next_packet(); 
    auto frame  = CRFrame();
    auto f_obj  = CRFrameObject();
    f_obj.version = 0;
    f_obj.ftype   = CRFrameObjectType::TelemetryPacket; 
    auto payload  = packet.header.to_bytestream();
    payload.insert(payload.end(), packet.payload.begin(), packet.payload.end());
    f_obj.payload = payload;
    std::string obj_name = "TelemetryPacketType::Unknown";
    switch (packet.header.ptype) {
      case g::TelemetryPacketType::BoringEvent : {
        obj_name = "TelemetryPacketType.BoringEvent";
        break;
      } 
      case g::TelemetryPacketType::InterestingEvent : {
        obj_name = "TelemetryPacketType.InterestingEvent";
        break;
      } 
      case g::TelemetryPacketType::NoGapsTriggerEvent : {
        obj_name = "TelemetryPacketType.NoGapsTriggerEvent";
        break;
      } 
      case g::TelemetryPacketType::NoTofDataEvent : {
        obj_name = "TelemetryPacketType.NoTofDataEvent";
        break;
      }
      case g::TelemetryPacketType::Tracker : {
        obj_name = "TelemetryPacketType.Tracker";
      }
      default : {
        // deal with monitoring etc
      } 
    }
    frame.put_fobject(f_obj, obj_name);
    return frame;
  } else {
    while (true) { // the infite loop gets broken by the 
                   // throw in prima_next_file 
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
          u64 p_size       = g::parse_u64(buffer, pos);
          payload.insert(payload.end(), buffer.begin(), buffer.end());
          buffer = bytestream(p_size);
          stream_file_.read(reinterpret_cast<char*>(buffer.data()), p_size);
          payload.insert(payload.end(), buffer.begin(), buffer.end());
          u64 pos_in_frame = 0;
          // from_bytestream is broken
          //auto frame = g::CRFrame::from_bytestream(payload, pos_in_frame);
          auto frame = CRFrame();
          frame.index = CRFrame::parse_index(buffer, pos_in_frame);
          buffer = g::slice(buffer, pos_in_frame, p_size); 
          frame.bytestorage = std::move(buffer);
          n_packets_read_++;
          return frame;
        }
      } 
    }
  }
}

//--------------------------------------------------

g::CRWriter::CRWriter(String fpath, String filename, u32 run_id, Option<u32> subrun_id, Option<String> timestamp)  { 
  file_path = fpath;
  file_name = filename;
  timestamp = timestamp;
  auto fname = std::format("{}/{}", file_path, file_name);
  file = std::ofstream(fname, std::ios::app | std::ios::out);
  if (!file.is_open()) {
    throw std::runtime_error("Unable to open file: " + fname);
  }
}

//--------------------------------------------------

auto g::CRWriter::new_file(Option<String> timestamp, bool is_sim) -> void { 
  auto fname = std::format("{}{}", file_path, get_runfilename(run_id, file_id, is_sim, timestamp));
  //let path     = Path::new(&filename); 
  file = std::ofstream(fname, std::ios::app | std::ios::out);
}

//--------------------------------------------------

auto g::CRWriter::add_frame(const CRFrame& frame) -> void { 
  bool newfile = false;
  auto buffer = frame.to_bytestream();    
  if (file.is_open() && !buffer.empty()) {
    file.write(reinterpret_cast<const char*>(buffer.data()),buffer.size());
  } 
  file_nbytes_wr += buffer.size();  
  n_frames += 1;
  if (frames_per_file != 0) { 
    if (n_frames == frames_per_file) {
      newfile = true;
      n_frames = 0;
    } else { 
      if (mbytes_per_file != 0) { 
        if (file_nbytes_wr >= mbytes_per_file * 1048576) {
          newfile = true;
          file_nbytes_wr = 0;
        }
      }
    }
  } 
  if (newfile) { 
    file.flush();
    n_frames = 0;
    file_id += 1;
    // FIXME - will always write sim files 
    new_file(file_timestamp, true);
  } 

  //  if newfile {
  //      //let filename = self.file_prefix.clone() + "_" + &self.file_id.to_string() + ".tof.gaps";
  //      match self.file.sync_all() {
  //        Err(err) => {
  //          error!("Unable to sync file to disc! {err}");
  //        },
  //        Ok(_) => ()
  //      }
  //      self.file = self.get_file(self.file_timestamp.clone());
  //      self.file_id += 1;
  //      //let path  = Path::new(&filename);
  //      //println!("==> [TOFPACKETWRITER] Will start a new file {}", path.display());
  //      //self.file = OpenOptions::new().create(true).append(true).open(path).expect("Unable to open file {filename}");
  //      //self.n_packets = 0;
  //      //self.file_id += 1;
  //    }
  //debug!("CRFrame written!");
}

