#include "telemetry_dataclasses.hpp"
#include "parsers.h"
#include "logging.hpp"

namespace gtl = Gaps::Telemetry;
namespace g   = Gaps;

// make it more look like rust
using namespace result;

//----------------------------------------

/// FIXME - this is obviously incomplete
u8 gtl::bfsw_ptype_to_u8(gtl::BfswPacketType pt) {
  switch (pt) {
    case gtl::BfswPacketType::Unknown:
      return 0;
    case gtl::BfswPacketType::Tracker:
      return 80;
    case gtl::BfswPacketType::BoringEvent:
      return 90;
    case gtl::BfswPacketType::RBWaveform:
      return 91;
    case gtl::BfswPacketType::InterestingEvent:
      return 190;
    case gtl::BfswPacketType::NoGapsTriggerEvent:
      return 191;
    case gtl::BfswPacketType::NoTofDataEvent:
      return 192;
    default:
      return 0;
  }
}

std::string gtl::bfsw_ptype_to_str(gtl::BfswPacketType pt) {
  switch (pt) {
      case gtl::BfswPacketType::Unknown:
      return "Unknown";
    case gtl::BfswPacketType::Tracker:
      return "Tracker";
    case gtl::BfswPacketType::BoringEvent:
      return "BoringEvent";
    case gtl::BfswPacketType::RBWaveform:
      return "RBWaveform";
    case gtl::BfswPacketType::InterestingEvent:
      return "InterestingEvent";
    case gtl::BfswPacketType::NoGapsTriggerEvent:
      return "NoGapsTriggerEvent";
    case gtl::BfswPacketType::NoTofDataEvent:
      return "NoTofDataEvent";
    default:
      return std::format("Unknown/NotImplemented ({})", (int)pt) ;
  }
}

//----------------------------------------

f64 gtl::PacketHeader::get_gcutime(){
  return timestamp * 0.064 + 1631030675.0;
};
  
auto  gtl::PacketHeader::from_bytestream(Vec<u8> const &stream, usize &pos)
  -> Result<PacketHeader, g::IOError> {
  gtl::PacketHeader ph;
  if (stream.size() < pos + gtl::PacketHeader::SIZE) {
    spdlog::error("The telemetry header is too short! ({} bytes when {} are expected!", stream.size(), gtl::PacketHeader::SIZE);
    pos += gtl::PacketHeader::SIZE;
    return Ok(ph);
  }
  if (parse_u16(stream, pos) != gtl::PacketHeader::HEAD) {
    spdlog::error("The given position {} does not point to a valid header signature of {}", pos-2 ,gtl::PacketHeader::HEAD);
    pos += gtl::PacketHeader::SIZE - 2;
    return Ok(ph);
  }
  ph.sync      = PacketHeader::HEAD;
  ph.ptype     = static_cast<gtl::BfswPacketType>(parse_u8 (stream, pos));
  ph.timestamp = parse_u32(stream, pos);
  ph.counter   = parse_u16(stream, pos);
  ph.length    = parse_u16(stream, pos);
  ph.checksum  = parse_u16(stream, pos);
  return Ok(ph);
}

auto gtl::PacketHeader::to_string() const -> std::string {
  std::string repr = "<PacketHeader:";
  repr += std::format("\n  Header      : {}" ,sync);
  repr += std::format("\n  Packet Type : {}" ,bfsw_ptype_to_str(ptype));
  repr += std::format("\n  Timestamp   : {}" ,timestamp);
  repr += std::format("\n  Counter     : {}" ,counter);
  repr += std::format("\n  Length      : {}" ,length);
  repr += std::format("\n  Checksum    : {}>",checksum);
  return repr;
}

//----------------------------------------

auto  gtl::Packet::from_bytestream(Vec<u8> const &stream,
                                   usize &pos) -> gtl::Packet {
  gtl::Packet packet;
  auto header = gtl::PacketHeader::from_bytestream(stream, pos).unwrap();
  // FIXME
  packet.header  = header;
  auto payload   = Gaps::slice(stream, pos, pos + header.length);
  packet.payload = std::move(payload);
  return packet;
}

auto gtl::Packet::to_string() const -> std::string {
  std::string repr = "<TelemetryPacket:";
  repr += std::format("{}", header.to_string());
  repr += "\n --------";
  repr += std::format("\n  Payload len : {}>",payload.size());
  return repr;
}

//----------------------------------------

auto gtl::TrkHit::to_string() const -> std::string {
  std::string repr = "<TrackerHit:";
  repr += std::format("\n  Layer      : {}", layer);
  repr += std::format("\n  Row        : {}", row);
  repr += std::format("\n  Module     : {}", module);
  repr += std::format("\n  Channel    : {}", channel);
  repr += std::format("\n  ADC        : {}", adc);
  repr += std::format("\n  Oscillator : {}", oscillator);
  repr += std::format("\n  Energy     : {}", energy);
  return repr;
}

//----------------------------------------

auto gtl::TrkEvent::to_string() const -> std::string {
  std::string repr = "<TrackerEvent:";
  repr += std::format("\n  layer         : {}" ,layer);
  repr += std::format("\n  flags1        : {}" ,flags1);
  repr += std::format("\n  Event ID      : {}" ,event_id);
  repr += std::format("\n  Event Time    : {}" ,event_time);
  for (auto &h : hits) {
    repr += std::format("\n \t {}", h.to_string());
  }
  return repr;
}

//----------------------------------------

/// FIXME - direct copy from bfsw, don't like it. 
/// Replace with parse_u16 etc methods. 
gtl::TofMetaData gtl::TofMetaData::from_bytestream(Vec<u8> const &bytes, usize &pos) {
  auto tm = TofMetaData();
  if(bytes.size() < 17) {
     return tm;
  }
  std::memcpy(&tm.event_id, &bytes[13], sizeof(tm.event_id));
  tm.status_version     = bytes[9] & 0xc0;
  if(tm.status_version  == 64 && bytes.size() >= 32) {
     tm.stats_valid     = true;
     tm.trigger_sources = bytes[11];
     tm.trigger_sources = tm.trigger_sources << 8;
     tm.trigger_sources |= bytes[10];
     tm.n_hits_umb      = bytes[17];
     tm.n_hits_cbe      = bytes[18];
     tm.n_hits_cor      = bytes[19];
     std::memcpy(&tm.tot_edep_umb, &bytes[20], 4);
     std::memcpy(&tm.tot_edep_cbe, &bytes[24], 4);
     std::memcpy(&tm.tot_edep_cor, &bytes[28], 4);
  }
  return tm;
}

auto gtl::TofMetaData::to_string() const -> std::string {
  std::string repr = "<TofMetaData";
  repr += std::format("\n  EventID      : {}" ,event_id);
  repr += std::format("\n  Version      : {}" ,status_version);
  repr += std::format("\n  StatsValid   : {}" ,stats_valid);
  repr += std::format("\n  Trigger Src  : {}" ,trigger_sources);
  repr += std::format("\n  N Hit Umb    : {}" ,n_hits_umb);
  repr += std::format("\n  N Hit Cbe    : {}>",n_hits_cbe);
  repr += std::format("\n  N Hit Cor    : {}>",n_hits_cor);
  repr += std::format("\n  Tot Edep Umb : {}>",tot_edep_umb);
  repr += std::format("\n  Tot Edep Cbe : {}>",tot_edep_cbe);
  repr += std::format("\n  Tot Edep Cor : {}>",tot_edep_cor);
  return repr;
}

//----------------------------------------

auto gtl::MergedEvent::to_string() const -> std::string {
  std::string repr = "<MergedEvent";
  repr += std::format("\n  Header : {}", header.to_string());
  repr += std::format("\n  Creation Time : {}" , creation_time);
  repr += std::format("\n  EventID       : {}" , event_id);
  repr += std::format("\n  Flags0        : {}" , flags0);
  repr += std::format("\n  Flags1        : {}" , flags1);
  repr += std::format("\n  N Tof hits    : {}" , n_tof_hits);
  repr += std::format("\n  Tof hits      : {}" , tof_event.hits.size());
  repr += std::format("\n  Trk hits      : {}>" ,trk_hits.size());
  //    PacketHeader header;
  //    u64 creation_time;
  //    u32 event_id;
  //    Vec<TrkEvent> tracker_events;  
  //    Vec<u8> tof_data;
  //    Vec<u8> raw_data;
  //    TofMetaData tof_meta;
  //    TrkMetaData tracker_meta;
  //    u8 flags0;
  //    u8 flags1;
  return repr;
}

auto gtl::MergedEvent::from_bytestream(Vec<u8> const &stream, usize &pos) 
    -> r::Result<MergedEvent, g::IOError> {
  auto evt     = MergedEvent();
  //evt.header   = PacketHeader::from_bytestream(stream, pos);
  evt.version   = parse_u8(stream, pos);
  evt.flags0    = parse_u8(stream, pos);
  if (evt.version == 1) {
    for (u8 k=0; k<8; k++) {
      evt.row_flags.push_back(parse_u8(stream, pos)); 
    }
  } else {
    evt.flags1   = parse_u8(stream, pos);
  }

  evt.event_id   = parse_u32(stream, pos);
  evt.n_tof_hits = parse_u8(stream, pos);
  // FIXME - only for version 1, version 0
  // does still have a tof delimiter
  //if (tof_delim != 0xAA) {
  //  log_error("Got incorrect Tof delimiter flag of " << (int)tof_delim);
  //  return evt;
  //}
  u16 num_tof_bytes = parse_u16(stream, pos);
  if (stream.size() < pos + num_tof_bytes) {
    std::string message = std::format("Stream does not contain enough TOF bytes!");
    spdlog::error("{}",message);
    auto err = g::IOError(g::IOError::ErrorKind::StreamTooShort, message);
    return Err(err);
  }
  auto tof_data = Gaps::slice(stream, pos, pos + num_tof_bytes);
  if (tof_data.size() > 0) {
    usize tpos = 0;
    auto tof_packet = TofPacket::from_bytestream(tof_data, tpos);
    if (tof_packet.is_ok()) {
      auto tof_event = TofEventSummary::from_tofpacket(tof_packet.unwrap());
      if (tof_event.is_ok()) {
        evt.tof_event = tof_event.unwrap();
      }
    }
  }
  pos += num_tof_bytes;
  u8 tracker_delim = parse_u8(stream, pos);

  if(tracker_delim != 0xbb) {
    std::string message = std::format("Incorrect tracker delmiter flag ({})!", tracker_delim);
    spdlog::error("{}",message);
    auto err = g::IOError(g::IOError::ErrorKind::WrongDelimiter, message);
    return Err(err);
  }
  evt.n_trk_hits = parse_u16(stream, pos); 
  for(u16 j = 0; j < evt.n_trk_hits; ++j) {
    u16 strip_id = parse_u16(stream, pos);
    u16 adc      = parse_u16(stream, pos);
    TrkHit hit;
    hit.channel = strip_id & 0b11111;
    hit.module  = (strip_id >> 5) & 0b111;
    hit.row     = (strip_id >> 8) & 0b111;
    hit.layer   = (strip_id >> 11) & 0b1111;
    hit.adc     = adc;
    evt.trk_hits.push_back(hit);
  }
  u8 osci_delim = parse_u8(stream, pos);
  if(osci_delim != 0xcc) {
    std::string message = std::format("Incorrect osci delmiter flag ({})!", osci_delim);
    spdlog::error("{}",message);
    auto err = g::IOError(g::IOError::ErrorKind::WrongDelimiter, message);
    return Err(err);
  }
  u8 osc_flags = parse_u8(stream, pos);
  Vec<u8> oscillator_idx;
  for(u8 j = 0; j < 8; ++j) {
    if((osc_flags >> j) & 0b1) {
      oscillator_idx.push_back((u8)j);
    }
  }
  if (pos + 6*oscillator_idx.size() > stream.size()) {
    std::string message = std::format("Stream does not contain enough bytes for {} trk oscillators!! Only {} bytes left!", oscillator_idx.size(), stream.size() - pos);
    spdlog::error("{}",message);
    auto err = g::IOError(g::IOError::ErrorKind::StreamTooShort, message);
    return Err(err);
  }
  for(auto idx : oscillator_idx) {
    u32 lower = parse_u32(stream, pos);
    u16 upper = parse_u16(stream, pos);
    //std::cout << (int)idx << " pos " << pos << " size " << stream.size() << std::endl;
    u64 osc = (static_cast<uint64_t>(upper) << 32) | lower;
    evt.tracker_oscillators[idx] = osc;
  }
  return Ok(evt);
}  

//template <typename T> int unpack(const T& bytes, size_t i) {
//
//         size_t i_start {i};
//         int rc;
//         rc = header.unpack(bytes, i);
//         if(rc < 0)
//            return -100 + rc;
//         else
//            i += rc;
//
//         //check that we have enough bytes to parse up to num_tof_bytes
//         if((i + 10) > bytes.size())
//            return -2;
//
//         //sub header
//         uint8_t version; i += from_bytes(&bytes[i], version);
//         i += from_bytes(&bytes[i], flags0);
//         i += from_bytes(&bytes[i], flags1);
//         i += from_bytes(&bytes[i], event_id);
//
//         //tof delimiter
//         uint8_t tof_delimiter; i += from_bytes(&bytes[i], tof_delimiter);
//         if(tof_delimiter != 0xaa)
//            return -4;
//
//         //tof size
//         uint16_t num_tof_bytes; i += from_bytes(&bytes[i], num_tof_bytes);
//         if((i + num_tof_bytes) > bytes.size())
//            return -5;
//
//         //copy tof data
//         tof_data.clear();
//         for(int j = 0; j < num_tof_bytes; ++j)
//         {
//            tof_data.push_back(bytes[i]); i += 1;
//         }
//
//         //check for enough bytes to parse tracker delimiter and size
//         if((i + 3) > bytes.size())
//            return -6;
//
//         //tracker delimiter
//         uint8_t tracker_delimiter; i += from_bytes(&bytes[i], tracker_delimiter);
//         if(tracker_delimiter != 0xbb)
//            return -7;
//
//         //num tracker bytes
//         uint16_t num_tracker_bytes; i += from_bytes(&bytes[i], num_tracker_bytes); 
//         if((i + num_tracker_bytes - 2) > bytes.size()) //note this size check was being done differently before, and I think it was wrong. this size check was done before advancing the index
//            return -8;
//
//         //unpack tracker data
//         while(1)
//         {
//            if(i >= bytes.size())
//               break;
//            tracker::event event;
//            int rc = event.unpack(bytes,i);
//            if(rc < 0)
//            {
//               spdlog::info("DEBUG event.unpack rc = {}", rc);
//               return -9;
//            }
//            else
//            {
//                tracker_events.push_back(std::move(event));
//                i += rc;
//            }
//         }
//
//         if(i != bytes.size())
//            return -10;
//
//         return i - i_start;
//      }
//   };

