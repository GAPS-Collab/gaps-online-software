#include "spdlog/spdlog.h"
#include "spdlog/cfg/env.h"

#include "telemetry_dataclasses.hpp"
#include "events/tracker_event.hpp"
#include "events/tracker_hit.hpp"
#include "packets/telemetry_packet.hpp"
#include "io/parsers.h"

namespace g   = gondola;

// make it more look like rust
using namespace result;


//----------------------------------------

auto g::TrkHeader::to_string() const -> std::string {
  std::string repr = "<TrkHeader:";
  repr += std::format("\n  sync     : {}", sync);
  repr += std::format("\n  crc      : {}", crc);
  repr += std::format("\n  sys_id   : {}", sys_id);
  repr += std::format("\n  pkt_id   : {}", packet_id);
  repr += std::format("\n  length   : {}", length);
  repr += std::format("\n  daq_cnt  : {}", daq_count);
  repr += std::format("\n  sys_time : {}", sys_time);
  repr += std::format("\n  version  : {}", version);
  return repr;
}

auto g::TrkHeader::from_bytestream(Vec<u8> const &stream, usize &pos) 
  -> r::Result<TrkHeader, g::IOError> {
  auto header = TrkHeader();
  if (stream.size() - pos < g::TrkHeader::SIZE) {
    std::string message = std::format("Stream is too short for a trkheader packet. We got a stream of size {} when expectinog {} bytes!", stream.size() - pos, g::TrkHeader::SIZE);
    auto err = g::IOError(g::IOError::ErrorKind::StreamTooShort, message);
    return Err(err);
  } 
  header.sync   = g::parse_u16(stream, pos);
  header.crc    = g::parse_u16(stream, pos);
  header.sys_id = g::parse_u8(stream, pos);
  header.packet_id = g::parse_u8(stream, pos); 
  header.length    = g::parse_u16(stream, pos);
  header.daq_count = g::parse_u16(stream, pos);
  u32 lower        = g::parse_u32(stream, pos);
  u16 upper        = g::parse_u16(stream, pos);
  u64 sys_time     = (static_cast<u64>(upper) << 32) | static_cast<u64>(lower);
  header.sys_time  = sys_time;
  header.version   = g::parse_u8(stream, pos);
  return Ok(header);
} 

//----------------------------------------

auto g::TrkHit::decode_id(u32 hw_id) -> Vec<u32> {
  Vec<u32> result;
  u32 remaining_number = hw_id;
  u32 channel          = remaining_number % 100;
  remaining_number    /= 100; // Removes channel contribution (e.g., 2050307 -> 20503)
  u32 mod              = remaining_number % 100;
  remaining_number    /= 100; // Removes module contribution (e.g., 20503 -> 205)
  u32 row              = remaining_number % 10;
  remaining_number    /= 10; // Removes row contribution (e.g., 205 -> 20)
  u32 layer            = remaining_number;
  return {layer, row, mod, channel};
}

auto g::TrkHit::get_strip_id() const -> u32 {
  return (u32)channel + (u32)module*100 + (u32)row*10000 + (u32)layer*100000;
}

//----------------------------------------

auto g::TrkEvent::to_string() const -> std::string {
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

auto g::TrkEventPacket::to_string() const -> std::string {
  auto repr = std::string("<TrkEventPacket:");
  repr     += std::format("\n  pkt header {}", header.to_string());
  repr     += std::format("\n  trk header {}", daq_header.to_string());
  repr     += "\n ----- TRK EVENTS -----";
  for (auto const &ev : events) {
    repr     += std::format("\n {}", ev.to_string());
  }
  repr     += std::format("\n run id     {}", run_id);
  repr     += std::format("\n run id old {}", run_id_old);
  return repr;
}

auto g::TrkEventPacket::from_bytestream(Vec<u8> const &stream, usize &pos)
  -> r::Result<TrkEventPacket, g::IOError> {
  TrkEventPacket packet;
  //auto packet_header = PacketHeader::from_bytestream(stream, pos);
  //if (packet_header.is_ok()) {
  //  packet.header = packet_header.unwrap();
  //} else {
  //  spdlog::error("Unpacking of the telemetry header failed!");
  //  return Err(packet_header.unwrap_err());
  //}
  auto trk_header = TrkHeader::from_bytestream(stream, pos);
  if (trk_header.is_ok()) {
    packet.daq_header = trk_header.unwrap();
  } else {
    return Err(trk_header.unwrap_err());
  }
  if (packet.daq_header.version >= 5) {
    packet.run_id = g::parse_u16(stream, pos);
  } else {
    packet.run_id_old = g::parse_u8(stream, pos);
  }
  // now read the events
  const size_t event_header_size = 12;
  while (true) {
    // FIXME - Alex seemingly has a bug here
    if ((packet.daq_header.version >= 4) && ((pos == stream.size()) || ((pos + 1 == stream.size()) && (stream.at(pos) == 0xff))       )) {
      return Ok(packet);
    } 
    if (pos + event_header_size > stream.size()) { 
      std::string message("Unable to read more TrackerEvents! Stream is too short!");
      //spdlog::error("{}",packet.to_string());
      auto err = g::IOError(g::IOError::ErrorKind::StreamTooShort, message);
      return Err(err);
    }
    g::TrkEvent trk_event;
    trk_event.layer = packet.daq_header.sys_id;
    u8 n_hits          = g::parse_u8(stream, pos);
    trk_event.flags1   = g::parse_u8(stream, pos);
    trk_event.event_id = g::parse_u32(stream, pos);
    u32 lower          = g::parse_u32(stream, pos);
    u16 upper          = g::parse_u16(stream, pos);
    u64 systime        = (static_cast<uint64_t>(upper) << 32) | lower;
    trk_event.event_time = systime;
    if (n_hits > 192) {
      // should that return error instead?
      return Ok(packet);
    } 
    if ((pos + (3*n_hits)) > stream.size()) {
      auto message =  std::format("Unable to read all {} tracker hits! Stream is too short!", n_hits);
      //spdlog::error("{}",packet.to_string());
      auto err = g::IOError(g::IOError::ErrorKind::StreamTooShort, message);
      return Err(err);
    }
    for (u8 j = 0; j<n_hits; j++) {
      u8 h0 = g::parse_u8(stream, pos);
      u8 h1 = g::parse_u8(stream, pos);
      u8 h2 = g::parse_u8(stream, pos);
      u8  asic_event_code = h2 >> 6;
      u8  channel = h0 & 0b11111;
      u8  module = h0 >> 5;
      u8  row = h1 & 0b111;
      u16 adc = ((h2 & 0b00111111) << 5) |        (h1 >> 3);

      auto hit = TrkHit();
      hit.channel = channel;
      hit.module  = module;
      hit.row     = row;
      hit.adc     = adc;
      hit.asic_event_code = asic_event_code;
      trk_event.hits.push_back(std::move(hit));
    }
    packet.events.push_back(std::move(trk_event));
  }
  if (packet.events.size() > 170) {
    std::string message = std::format("There seem to be more than 170 events (!) in the tracker. This is nonsense!");
    spdlog::error("{}",message);
    auto err = g::IOError(g::IOError::ErrorKind::TooManyTrkEvents, message);
    return Err(err); 
  }
  return Ok(packet);
}

//----------------------------------------

/// FIXME - direct copy from bfsw, don't like it. 
/// Replace with parse_u16 etc methods. 
g::TofMetaData g::TofMetaData::from_bytestream(Vec<u8> const &bytes, usize &pos) {
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

auto g::TofMetaData::to_string() const -> std::string {
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

auto g::TelemetryEvent::to_string() const -> std::string {
  std::string repr = "<TelemetryEvent";
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

auto g::TelemetryEvent::from_bytestream(Vec<u8> const &stream, usize &pos) 
    -> r::Result<TelemetryEvent, g::IOError> {
  // check if it has at least the fix part
  if (stream.size() < pos + 18) {
    std::string message = std::format("Stream does not contain enough bytes to parse TelemetryEvent event id and basic information! Packet might be broken(?)");
    spdlog::error("{}",message);
    auto err = g::IOError(g::IOError::ErrorKind::StreamTooShort, message);
    return Err(err);
  }
  
  auto evt     = TelemetryEvent();
  //evt.header   = PacketHeader::from_bytestream(stream, pos);
  evt.version  = g::parse_u8(stream, pos);
  evt.flags0   = g::parse_u8(stream, pos);
  if (evt.version == 1) {
    for (u8 k=0; k<8; k++) {
      evt.row_flags.push_back(g::parse_u8(stream, pos)); 
    }
  } else {
    evt.flags1   = g::parse_u8(stream, pos);
  }

  evt.event_id   = g::parse_u32(stream, pos);
  evt.n_tof_hits = g::parse_u8(stream, pos);
  // FIXME - only for version 1, version 0
  // does still have a tof delimiter
  //if (tof_delim != 0xAA) {
  //  log_error("Got incorrect Tof delimiter flag of " << (int)tof_delim);
  //  return evt;
  //}
  u16 num_tof_bytes = g::parse_u16(stream, pos);
  if (stream.size() < pos + num_tof_bytes) {
    //spdlog::error("{}", evt.to_string());
    std::string message = std::format("Stream does not contain enough TOF bytes! We expect {} when the remaing size is only {}", num_tof_bytes, stream.size() - pos);
    spdlog::error("{}",message);
    auto err = g::IOError(g::IOError::ErrorKind::StreamTooShort, message);
    return Err(err);
  }
  auto tof_data = g::slice(stream, pos, pos + num_tof_bytes);
  if (tof_data.size() > 0) {
    usize tpos = 0;
    auto tof_packet = TofPacket::from_bytestream(tof_data, tpos);
    if (tof_packet.is_ok()) {
      auto tof_event = gondola::TofEventSummary::from_tofpacket(tof_packet.unwrap());
      if (tof_event.is_ok()) {
        evt.tof_event = tof_event.unwrap();
      }
    }
  }
  pos += num_tof_bytes;
  u8 tracker_delim = g::parse_u8(stream, pos);

  if(tracker_delim != 0xbb) {
    std::string message = std::format("Incorrect tracker delmiter flag ({})!", tracker_delim);
    spdlog::error("{}",message);
    auto err = g::IOError(g::IOError::ErrorKind::WrongDelimiter, message);
    return Err(err);
  }
  evt.n_trk_hits = g::parse_u16(stream, pos); 
  for(u16 j = 0; j < evt.n_trk_hits; ++j) {
    u16 strip_id = g::parse_u16(stream, pos);
    u16 adc      = g::parse_u16(stream, pos);
    TrkHit hit;
    hit.channel = strip_id & 0b11111;
    hit.module  = (strip_id >> 5) & 0b111;
    hit.row     = (strip_id >> 8) & 0b111;
    hit.layer   = (strip_id >> 11) & 0b1111;
    hit.adc     = adc;
    evt.trk_hits.push_back(hit);
  }
  u8 osci_delim = g::parse_u8(stream, pos);
  if(osci_delim != 0xcc) {
    std::string message = std::format("Incorrect osci delmiter flag ({})!", osci_delim);
    spdlog::error("{}",message);
    auto err = g::IOError(g::IOError::ErrorKind::WrongDelimiter, message);
    return Err(err);
  }
  u8 osc_flags = g::parse_u8(stream, pos);
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
    u32 lower = g::parse_u32(stream, pos);
    u16 upper = g::parse_u16(stream, pos);
    //std::cout << (int)idx << " pos " << pos << " size " << stream.size() << std::endl;
    u64 osc = (static_cast<uint64_t>(upper) << 32) | lower;
    evt.tracker_oscillators[idx] = osc;
  }
  return Ok(evt);
}  
      
auto g::TelemetryEvent::from_telemetrypacket(TelemetryPacket const &packet) 
  -> r::Result<TelemetryEvent, g::IOError> {
  usize pos = 0;
  auto  res = g::TelemetryEvent::from_bytestream(packet.payload, pos);
  if (res.is_err()) {
    return res;
  }  
  auto ev = res.unwrap();
  ev.tof_event.set_paddlemap(*packet.paddles);
  return Ok(ev); 
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

