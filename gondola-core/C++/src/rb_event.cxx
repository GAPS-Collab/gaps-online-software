// This file is part of gaps-online-software and published 
// under the GPLv3 license

#include "events/rb_event.hpp"
#include "io.hpp"
#include "io/parsers.h"
#include "serialization.h" 

#include "spdlog/spdlog.h"
#include "spdlog/cfg/env.h"

namespace g = gondola;
using namespace result;



/*************************************/

g::RBEvent::RBEvent() {  
  data_type = 0;
  status    = EventStatus::Unknown;
  header = RBEventHeader();
  adc    = Vec<Vec<u16>>(); 
  for (usize k=0; k<NCHN; k++) {
    adc.push_back(Vec<u16>());
  }
  hits  = Vec<g::TofHit>();
}

/**********************************************************/

auto g::RBEvent::to_string() const -> std::string {
  std::string repr = "<RBEvent\n";
  std::stringstream ss;
  ss << status;
  repr += "  status    : " + ss.str() + "\n";
  repr += header.to_string();
  repr += "\n";
  repr += " -- -- adc -- --";
  for (auto ch : header.get_channels()) {
    repr += "\n " + std::to_string(ch)  + ": ..";
    repr += std::to_string(adc[ch][0]);
    repr += " "; 
    repr += std::to_string(adc[ch][1]);
    repr += " .. .."; 
  }
  if ( hits.size() > 0 ) {
    repr += "\n\n ** ** hits ** **\n";
    for (auto const &h : hits) {
      repr += h.to_string();
      repr += "\n";
    } 
  } else {
    repr += "\n -- no hits!";
  }
  repr += ">";
  return repr;
}

/**********************************************************/

bool g::RBEvent::channel_check(u8 channel) const {
  if (channel == 0) {
    spdlog::error("Remember, channels start at 1. 0 does not exist!");
    return false;
  }
  if (channel > 9) {
    spdlog::error("Thera are no channels > 9!");
    return false;
  }
  return true;
}

/**********************************************************/
  
const Vec<u16>& g::RBEvent::get_channel_adc(u8 channel) const {
  if (!(channel_check(channel))) {
    return _empty_channel;
  }
  return adc[channel -1]; 
}

/*************************************/
  
const Vec<u16>& g::RBEvent::get_channel_by_label(u8 channel) const {
  return adc[channel - 1];
}

const Vec<u16>& g::RBEvent::get_channel_by_id(u8 channel) const {
  return adc[channel];
}

/**********************************************************/

auto g::RBEvent::calc_baseline(const Vec<f32> &volts, usize min_bin, usize max_bin) 
  -> f32 {
  f32 bl     = 0;
  for (usize idx = 0; idx<volts.size(); idx++) {
    //f32 bl     = std::accumulate(ch_bl[ch].begin() + min_bin, ch_bl[ch].begin() + max_bin,0);
    if (idx <= min_bin) {
      continue;
    } else if ((idx > min_bin) && (idx <=max_bin)) {
      bl += volts[idx];
    } else {
      break;
    }
  }
  bl        /= (f32)(max_bin - min_bin);
    //baselines.push_back(bl);
  return bl;
}

/**********************************************************/

auto g::RBEvent::from_bytestream(const Vec<u8> &stream, u64 &pos) 
  -> RBEvent {
  RBEvent event = RBEvent();
  spdlog::debug("Start decoding at pos {}", pos);
  u16 head = g::parse_u16(stream, pos);
  if (head != RBEvent::HEAD)  {
    spdlog::error("[RBEvent::from_bytestream] Header signature invalid!");  
    event.status = EventStatus::IncompleteReadout;
    return event;
  }
  event.data_type = g::parse_u8(stream, pos);
  //event.status    = g::parse_u8(stream, pos);
  // FIXME - this can fail. Write a custom casting method that doesn't
  event.status    = static_cast<EventStatus>(stream[pos]); pos+=1; 
  // hits are below when readking out hit vector
  // FIXME
  u8 nhits        = g::parse_u8(stream, pos);
  //spdlog::info("{}", event.data_type);
  //spdlog::info("{}", event.status);
  auto header     = RBEventHeader::from_bytestream(stream, pos);
  if (header.is_err()) {
    // FIXME
    return event;
  }
  event.header    = header.unwrap();
  spdlog::debug("Decoded RBEventHeader!");
  if (event.header.is_event_fragment() || event.header.drs_lost_trigger()) {
    return event;
  }
  for (auto ch : event.header.get_channels()) {
    if (stream.size() < pos + 2*NWORDS) {
      event.status = EventStatus::IncompleteReadout;
      return event;
    }
    Vec<u8>::const_iterator start = stream.begin() + pos;
    Vec<u8>::const_iterator end   = stream.begin() + pos + 2*NWORDS;    // 2*NWORDS because stream is Vec::<u8> and it is 16 bit words.
    Vec<u8> data(start, end);
    event.adc[ch] = u8_to_u16(data);
    pos += 2*NWORDS;
  }
  // Decode the hits
  for (u8 k=0;k<nhits;k++) {
    auto maybe_hit = g::TofHit::from_bytestream(stream, pos);
    if (maybe_hit.is_ok()) {
      auto hit = maybe_hit.unwrap();
      event.hits.push_back(hit);
    }
  }
  u16 tail = g::parse_u16(stream, pos);
  if (tail != RBEvent::TAIL) {
    spdlog::error("After parsing the event, we found an invalid tail signature {}", tail);
  }
  return event;
}


