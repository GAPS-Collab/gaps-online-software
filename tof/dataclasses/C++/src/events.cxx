#include<numeric>
#include<sstream>
#include<format>
#include<limits>
#include<bitset>
#include<cmath>

#include "events.h"
#include "parsers.h"
#include "serialization.h"
#include "logging.hpp"

#include "spdlog/cfg/env.h"


/// masks to decode LTB hit masks
const u16 LTB_CH0 = 0x3   ;
const u16 LTB_CH1 = 0xc   ;
const u16 LTB_CH2 = 0x30  ; 
const u16 LTB_CH3 = 0xc0  ;
const u16 LTB_CH4 = 0x300 ;
const u16 LTB_CH5 = 0xc00 ;
const u16 LTB_CH6 = 0x3000;
const u16 LTB_CH7 = 0xc000;
const u16 LTB_CHANNELS[8] = {
    LTB_CH0,
    LTB_CH1,
    LTB_CH2,
    LTB_CH3,
    LTB_CH4,
    LTB_CH5,
    LTB_CH6,
    LTB_CH7
};


/**
 * Helper to get adc data from Vec<u8>
 *
 */
Vec<u16> u8_to_u16(const Vec<u8> &vec_u8) {
  Vec<u16> vec_u16;
  vec_u16.reserve(vec_u8.size() / sizeof(u16));
  for (size_t i = 0; i < vec_u8.size(); i += sizeof(u16)) {
    u16 value;
    std::memcpy(&value, &vec_u8[i], sizeof(u16));
    vec_u16.push_back(value);
  }
  return vec_u16;
}

/*************************************/

RBEventHeader::RBEventHeader() {
  rb_id              = 0; 
  event_id           = 0; 
  channel_mask       = 0; 
  status_byte        = 0;
  stop_cell          = 0; 
  ch9_amp            = 0;
  ch9_freq           = 0;
  ch9_phase          = 0;
  fpga_temp          = 0;
  timestamp16        = 0; 
  timestamp32        = 0; 
}

/*************************************/

std::string RBEventHeader::to_string() const {
  auto sfit = get_sine_fit();
  std::string repr = "<RBEventHeader";
  repr += "\n  rb id          " + std::to_string(rb_id)                 ;
  repr += "\n  event id       " + std::to_string(event_id)              ;
  repr += "\n  is locked      " + std::to_string(is_locked())           ;
  repr += "\n  is locked (1s) " + std::to_string(is_locked_last_sec())  ;
  repr += "\n  lost trigger   " + std::to_string(drs_lost_trigger())    ;
  repr += "\n  event fragment " + std::to_string(is_event_fragment())   ;
  repr += "\n  channel mask   " + std::to_string(channel_mask)          ;
  repr += "\n  |-> channels   ";
  for (auto ch : get_channels()) {
    repr += " " + std::to_string(ch) + " ";
  }
  repr += "\n  stop cell      " + std::to_string(stop_cell)             ;
  repr += "\n  ** online ch9 fit amp, freq, phase";
  repr += "\n    AMP " + std::to_string(sfit[0]);
  repr += "  FREQ " + std::to_string(sfit[1]);
  repr += "  PHASE " + std::to_string(sfit[2]); 
  repr += "\n  timestamp32    " + std::to_string(timestamp32)           ;
  repr += "\n  timestamp16    " + std::to_string(timestamp16)           ;
  repr += "\n  |->timestamp48 " + std::to_string(get_timestamp48())     ;
  repr += "\n  FPGA temp [C]  " + std::to_string(get_fpga_temp())       ;
  repr += ">";
  return repr;
}

/*************************************/

Vec<u8> RBEventHeader::get_channels() const {
  Vec<u8>  channels = Vec<u8>();
  for (u8 k=0;k<9;k++) {
    if ((channel_mask & (1 << k)) > 0) {
      channels.push_back(k);
    }
  }
  return channels; 
}

/*************************************/

u8 RBEventHeader::get_nchan() const {
  return get_channels().size(); 
}

/*************************************/

RBEventHeader RBEventHeader::from_bytestream(const Vec<u8> &stream,
                                             u64 &pos){
  Gaps::set_loglevel(Gaps::LOGLEVEL::info);
  RBEventHeader header;
  u16 head                  = Gaps::parse_u16(stream, pos);
  if (head != RBEventHeader::HEAD) {
    //log_error("[RBEventHeader::from_bytestream] Header signature " << head << " invalid!");
  }
  header.rb_id               = Gaps::parse_u8(stream , pos);  
  header.event_id            = Gaps::parse_u32(stream, pos);  
  header.channel_mask        = Gaps::parse_u16(stream, pos);   
  header.status_byte         = Gaps::parse_u8(stream , pos); 
  header.stop_cell           = Gaps::parse_u16(stream, pos);  
  header.ch9_amp             = Gaps::parse_u16(stream, pos);  
  header.ch9_freq            = Gaps::parse_u16(stream, pos);  
  header.ch9_phase           = Gaps::parse_u32(stream, pos);  
  header.fpga_temp           = Gaps::parse_u16(stream, pos);  
  header.timestamp32         = Gaps::parse_u32(stream, pos);
  header.timestamp16         = Gaps::parse_u16(stream, pos);
  u16 tail                   = Gaps::parse_u16(stream, pos);
  if (tail != RBEventHeader::TAIL) {
    log_error("Tail signature incorrect! Got tail " << tail);
  }
  return header; 
}

/*************************************/

bool RBEventHeader::has_ch9() const {
  return (channel_mask & 512) > 0;
}

/*************************************/
  
f32 RBEventHeader::get_fpga_temp() const {
  f32 zynq_temp = (((fpga_temp & 4095) * 503.975) / 4096.0) - 273.15;
  //f32 temp = (fpga_temp * 503.975/4096) - 273.15;
  return zynq_temp;
}

/*************************************/

bool RBEventHeader::is_event_fragment() const {
  return (status_byte & 1) > 0;
}

/*************************************/

bool RBEventHeader::drs_lost_trigger() const {
  return ((status_byte >> 1) & 1) > 0;
}

/*************************************/

bool RBEventHeader::lost_lock() const {
  return ((status_byte >> 2) & 1) > 0;
}

/*************************************/

bool RBEventHeader::lost_lock_last_sec() const {
  return ((status_byte >> 3) & 1) > 0;
}

/*************************************/

bool RBEventHeader::is_locked() const {
  return !(lost_lock());
}

/*************************************/

bool RBEventHeader::is_locked_last_sec() const {
  return !(lost_lock_last_sec());
}

/*************************************/

u64 RBEventHeader::get_timestamp48() const {
  return ((u64)timestamp16 << 32) | (u64)timestamp32;
}

/*************************************/

Vec<u8> RBEventHeader::get_active_data_channels() const {
  Vec<u8> active_channels;
  for (auto const &ch : {1,2,3,4,5,6,7,8} ) {
    if ((channel_mask & (u8)pow(2, ch - 1)) == (u8)pow(2,ch - 1)) active_channels.push_back(ch);
  } 
  //if ((channel_mask & 1)   == 1)   active_channels.push_back(1);
  return active_channels;
}

/*************************************/

u8 RBEventHeader::get_n_datachan() const {
  Vec<u8> active_channels = get_active_data_channels();
  return (u8)active_channels.size();
}

/*************************************/

std::array<f32, 3> RBEventHeader::get_sine_fit() const {
  f32 u16_MAX = 65535;
  f32 amp    = (20.0 * ch9_amp   /u16_MAX) - 10.0;
  f32 freq   = (20.0 * ch9_freq  /u16_MAX) - 10.0;
  f32 phase  = (20.0 * ch9_phase /u16_MAX) - 10.0;
  std::array<f32, 3> result = {amp,freq,phase};
  return result;
}

/*************************************/

RBEvent::RBEvent() {  
  data_type = 0;
  status    = EventStatus::Unknown;
  header = RBEventHeader();
  adc    = Vec<Vec<u16>>(); 
  for (usize k=0; k<NCHN; k++) {
    adc.push_back(Vec<u16>(NWORDS));
  }
  hits  = Vec<TofHit>();
}

/**********************************************************/

std::string RBEvent::to_string() const {
  std::string repr = "<RBEvent\n";
  std::stringstream ss;
  ss << status;
  //repr += "  data type : " + ss.str(); 
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

bool RBEvent::channel_check(u8 channel) const {
  if (channel == 0) {
    log_error("Remember, channels start at 1. 0 does not exist!");
    return false;
  }
  if (channel > 9) {
    log_error("Thera are no channels > 9!");
    return false;
  }
  return true;
}

/**********************************************************/
  
const Vec<u16>& RBEvent::get_channel_adc(u8 channel) const {
  if (!(channel_check(channel))) {
    return _empty_channel;
  }
  return adc[channel -1]; 
}
  
const Vec<u16>& RBEvent::get_channel_by_label(u8 channel) const {
  return adc[channel - 1];
}

const Vec<u16>& RBEvent::get_channel_by_id(u8 channel) const {
  return adc[channel];
}

/**********************************************************/

f32 RBEvent::calc_baseline(const Vec<f32> &volts,
                           usize min_bin,
                           usize max_bin) {
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

RBEvent RBEvent::from_bytestream(const Vec<u8> &stream,
                                 u64 &pos) {
  RBEvent event = RBEvent();
  log_debug("Start decoding at pos " << pos);
  u16 head = Gaps::parse_u16(stream, pos);
  if (head != RBEvent::HEAD)  {
    //log_error("[RBEvent::from_bytestream] Header signature invalid!");  
    event.status = EventStatus::IncompleteReadout;
    return event;
  }
  event.data_type = Gaps::parse_u8(stream, pos);
  //event.status    = Gaps::parse_u8(stream, pos);
  // FIXME - this can fail. Write a custom casting method that doesn't
  event.status    = static_cast<EventStatus>(stream[pos]); pos+=1; 
  // hits are below when readking out hit vector
  // FIXME
  u8 nhits        = Gaps::parse_u8(stream, pos);
  //spdlog::info("{}", event.data_type);
  //spdlog::info("{}", event.status);
  event.header    = RBEventHeader::from_bytestream(stream, pos);
  log_debug("Decoded RBEventHeader!");
  if (event.header.is_event_fragment() || event.header.drs_lost_trigger()) {
    return event;
  }
  for (auto const &ch : event.header.get_channels()) {
    log_debug("Found active data channel " <<  ch);
    Vec<u8>::const_iterator start = stream.begin() + pos;
    Vec<u8>::const_iterator end   = stream.begin() + pos + 2*NWORDS;    // 2*NWORDS because stream is Vec::<u8> and it is 16 bit words.
    Vec<u8> data(start, end);
    event.adc[ch] = u8_to_u16(data);
    pos += 2*NWORDS;
  }
  // Decode the hits
  for (u8 k=0;k<nhits;k++) {
    auto hit = TofHit::from_bytestream(stream, pos);
    event.hits.push_back(hit);
  }

  u16 tail = Gaps::parse_u16(stream, pos);
  if (tail != RBEvent::TAIL) {
    log_error("After parsing the event, we found an invalid tail signature " << tail);
  }
  return event;
}

/**********************************************************/

RBMissingHit RBMissingHit::from_bytestream(const Vec<u8> &stream,
                                           u64 &pos) {
  log_debug("Start decoding at pos " << pos);
  u16 head = Gaps::parse_u16(stream, pos);
  RBMissingHit miss  = RBMissingHit();
  if (head != RBMissingHit::HEAD)  {
    //log_error("No header signature found!");  
    return miss;  
  }
  // verify_fixed already advances pos by 2
  miss.event_id      = Gaps::parse_u32(stream, pos);
  miss.ltb_hit_index = Gaps::parse_u8(stream, pos);
  miss.ltb_id        = Gaps::parse_u8(stream, pos);
  miss.ltb_dsi       = Gaps::parse_u8(stream, pos);
  miss.ltb_j         = Gaps::parse_u8(stream, pos);
  miss.ltb_ch        = Gaps::parse_u8(stream, pos);
  miss.rb_id         = Gaps::parse_u8(stream, pos);
  miss.rb_ch         = Gaps::parse_u8(stream, pos);
  u16 tail = Gaps::parse_u16(stream, pos);
  if (tail != RBMissingHit::TAIL) {
    log_error("After parsing the event, we found an invalid tail signature " << tail);
  }
  return miss;
}

/**********************************************************/

std::ostream& operator<<(std::ostream& os, const EventQuality& qual) {
   os << "<EventQuality: " ;
   switch (qual) {
     case EventQuality::Unknown : { 
       os << "Unknown>";
       break;
     }
     case EventQuality::Silver : { 
       os << "Silver>";
       break;
     }
     case EventQuality::Gold : { 
       os << "Gold>";
       break;
     }
     case EventQuality::Diamond : { 
       os << "Diamond>";
       break;
     }
     case EventQuality::FourLeafClover : { 
       os << "FourLeafClover>";
       break;
     }
   }
   return os;
}

/**********************************************************/

std::ostream& operator<<(std::ostream& os, const CompressionLevel& level) {
   os << "<CompressionLevel: " ;
   switch (level) {
     case CompressionLevel::Unknown : { 
       os << "Unknown>";
       break;
     }
     case CompressionLevel::None : { 
       os << "None>";
       break;
     }
   }
   return os;
}

/**********************************************************/

std::ostream& operator<<(std::ostream& os, const EventStatus& qual) {
   os << "<EventStatus: " ;
   switch (qual) {
     case EventStatus::Unknown : { 
       os << "Unknown>";
       break;
     }
     case EventStatus::Crc32Wrong : { 
       os << "Crc32Wrong>";
       break;
     }
     case EventStatus::TailWrong : { 
       os << "TailWrong>";
       break;
     }
     case EventStatus::IncompleteReadout : { 
       os << "IncompleteReadout>";
       break;
     }
     case EventStatus::Perfect : { 
       os << "Perfect>";
       break;
     }
   }
   return os;
}

/**********************************************************/

std::ostream& operator<<(std::ostream& os, const TriggerType& t_type) {
   os << "<TriggerType: " ;
   switch (t_type) {
     case TriggerType::Unknown : { 
       os << "Unknown>";
       break;
     }
     case TriggerType::Gaps : { 
       os << "Gaps>";
       break;
     }
     case TriggerType::Any : { 
       os << "Any>";
       break;
     }
     case TriggerType::Track : { 
       os << "Track>";
       break;
     }
     case TriggerType::TrackCentral : { 
       os << "TrackCentral>";
       break;
     }
     case TriggerType::Poisson : { 
       os << "Poisson>";
       break;
     }
     case TriggerType::Forced : { 
       os << "Forced>";
       break;
     }
   }
   return os;
}

/**********************************************************/

std::ostream& operator<<(std::ostream& os, const LTBThreshold& thresh) {
   os << "<LTBThresholde: " ;
   switch (thresh) {
     case LTBThreshold::Unknown : { 
       os << "Unknown>";
       break;
     }
     case LTBThreshold::NoHit : { 
       os << "NoHit>";
       break;
     }
     case LTBThreshold::Hit : { 
       os << "Hit>";
       break;
     }
     case LTBThreshold::Beta : { 
       os << "Beta>";
       break;
     }
     case LTBThreshold::Veto : { 
       os << "Veto>";
       break;
     }
   }
   return os;
}

/**********************************************************/

u32 TofEvent::get_n_rbmissinghits(u32 mask){
  return ((mask & 0xFF00)     >> 8);
}

/**********************************************************/

u32 TofEvent::get_n_rbevents(u32 mask){
  return (mask & 0xFF);
}

/**********************************************************/

TofEvent::TofEvent() {
  status       = EventStatus::IncompleteReadout;
  header       = TofEventHeader();
  mt_event     = MasterTriggerEvent();
  rb_events    = Vec<RBEvent>();
  missing_hits = Vec<RBMissingHit>();
}

/**********************************************************/

TofEvent TofEvent::from_bytestream(const Vec<u8> &stream,
                                   u64 &pos) {
  spdlog::cfg::load_env_levels();
  TofEvent event = TofEvent();
  // FIXME - we need more of these checks
  // update expected_size as we go
  usize expected_size = 4
      + 2
      + TofEventHeader::SIZE
      + MasterTriggerEvent::SIZE
      + 12;
  if (stream.size() - pos < expected_size) {
    event.status = EventStatus::IncompleteReadout;
    return event;
  }
  log_debug("Start decoding at pos " << pos);
  u16 head = Gaps::parse_u16(stream, pos);
  if (head != TofEvent::HEAD)  {
    log_error("No header signature found!");  
  }
  // for now skip quality and compression level
  pos += 2;
  event.header      = TofEventHeader::from_bytestream(stream, pos);
  event.mt_event    = MasterTriggerEvent::from_bytestream(stream, pos);
  //pos += 45; // for now skip master trigger event
  u32 mask          = Gaps::parse_u32(stream, pos);
  u32 n_rbevents    = get_n_rbevents(mask);
  u32 n_missing     = get_n_rbmissinghits(mask);
  log_debug("Expecting " << n_rbevents << " RBEvents, " << n_missing << " RBMissingHits");
  for (u32 k=0; k< n_rbevents; k++) {
    RBEvent rb_event = RBEvent::from_bytestream(stream, pos);
    event.rb_events.push_back(rb_event);
    if (rb_event.status == EventStatus::IncompleteReadout) {
      event.status = EventStatus::IncompleteReadout;
    }
  }

  for (u32 k=0; k< n_missing; k++) {
    RBMissingHit missy = RBMissingHit::from_bytestream(stream, pos);
    event.missing_hits.push_back(missy);
  }
  
  //  event.compression_level = CompressionLevel::from_u8(&parse_u8(stream, pos));
  //  event.quality           = EventQuality::from_u8(&parse_u8(stream, pos));
  return event;
}
  
/**********************************************************/

TofEvent TofEvent::from_tofpacket(const TofPacket &packet) {
  TofEvent event;
  if (packet.packet_type != PacketType::TofEvent) {
    log_error("Wrong packet type! " << packet_type_to_string(packet.packet_type));
    return event;
  } 
  u64 _pos = 0;
  event = TofEvent::from_bytestream(packet.payload, _pos);
  return event;
}

/**********************************************************/
  
const RBEvent& TofEvent::get_rbevent(u8 board_id) const {
  for (const auto &ev : rb_events) {
    if (ev.header.rb_id == board_id) {
      return ev;
    }
  }
  log_error("No RBEvent for board " << board_id);
  return _empty_event;
}

/**********************************************************/
  
Vec<u8> TofEvent::get_rbids() const {
  Vec<u8> rb_ids = Vec<u8>();
  for (const auto &ev : rb_events) {
    rb_ids.push_back(ev.header.rb_id);
  }
  return rb_ids;
}

/**********************************************************/
    
bool TofEvent::passed_consistency_check() {
  log_error("This is not implmented yet!");
  return false;
}

/**********************************************************/

  
u64 MasterTriggerEvent::get_timestamp_gps48() const {
  return (((u64)tiu_gps16 << 32) | (u64) tiu_gps32); 
}

/*************************************/

u64 MasterTriggerEvent::get_timestamp_abs48() const {
  u64 gps = get_timestamp_gps48();
  u32 ts  = timestamp;
  // FIXME - I guess we need to cast to u64
  // This might be a bug
  if (ts < tiu_timestamp) {
    // counter rollover
    ts += (u64)std::numeric_limits<u32>::max();
  }
  u64 ts_abs  = 1e9 * gps + (u64)(ts - tiu_timestamp);
  return ts_abs;
}

/*************************************/

Vec<TriggerType> MasterTriggerEvent::get_trigger_sources() const {
  auto t_types = Vec<TriggerType>();
  u16 gaps_trigger = (trigger_source >> 5 & 0x1) == 1;
  if (gaps_trigger) {
    t_types.push_back(TriggerType::Gaps);
  }
  u16 any_trigger    = (trigger_source >> 6 & 0x1) == 1;
  if (any_trigger) {
    t_types.push_back(TriggerType::Any);
  }
  u16 forced_trigger = (trigger_source >> 7 & 0x1) == 1;
  if (forced_trigger) {
    t_types.push_back(TriggerType::Forced);
  }
  u16 track_trigger  = (trigger_source >> 8 & 0x1) == 1;
  if (track_trigger) {
    t_types.push_back(TriggerType::Track);
  }
  u16 central_track_trigger
                     = (trigger_source >> 9 & 0x1) == 1;
  if (central_track_trigger) {
    t_types.push_back(TriggerType::TrackCentral);
  }
  return t_types;
} 

/**********************************************************/

MasterTriggerEvent::MasterTriggerEvent() {
  event_id       = 0; 
  timestamp      = 0; 
  tiu_timestamp  = 0; 
  tiu_gps32      = 0; 
  tiu_gps16      = 0; 
  crc            = 0;
  trigger_source = 0;
  dsi_j_mask     = 0;
  channel_mask   = Vec<u16>();
  mtb_link_mask  = 0;
}  

/**********************************************************/
  
Vec<u8> MasterTriggerEvent::get_rb_link_ids() const {
  auto links = Vec<u8>();
  for (u8 k=0;k<64;k++) {
    if (((u64)(mtb_link_mask >> k) & (u64)0x1) == 1) {
      links.push_back(k);
    }
  }
  return links;
}
    
Vec<std::tuple<u8, u8, u8, LTBThreshold>> MasterTriggerEvent::get_trigger_hits() const {

  auto hits = Vec<std::tuple<u8,u8,u8,LTBThreshold>>(); 
  //let n_masks_needed = self.dsi_j_mask.count_ones() / 2 + self.dsi_j_mask.count_ones() % 2;
  auto dsi_j_mask_bits = std::bitset<32>(dsi_j_mask);
  u32 n_masks_needed   = dsi_j_mask_bits.count();
  if (channel_mask.size() < n_masks_needed) {
    log_error("We need " << n_masks_needed << " hit masks, but only have " << channel_mask.size() << "! This is bad!");
    return hits;
  }
  u8 n_mask = 0;
  for (u8 k=0;k<32;k++) {
    if ((u32)((dsi_j_mask >> k) & 0x1) == 1) {
      u8 dsi = 0;
      u8 j   = 0;
      if (k < 5) {
        dsi = 1;
        j   = k  + 1;
      } else if (k < 10) {
        dsi = 2;
        j   = k  - 5 + 1;
      } else if (k < 15) {
        dsi = 3;
        j   = k - 10 + 1;
      } else if (k < 20) {
        dsi = 4;
        j   = k - 15 + 1;
      } else if (k < 25) {
        dsi = 5;
        j   = k - 20 + 1;
      } 
      u32 channels = channel_mask[n_mask]; 
      for (u8 i=0;i<8; i++) {
        u32 ch  = LTB_CHANNELS[i];
        u32 chn = i + 1; 
        //for (i,ch) in LTB_CHANNELS.iter().enumerate() {
        //let chn = ch + 1;
        //println!("i,ch {}, {}", i, ch);
        u32 thresh_bits = (u8)(channels & (ch) >> (i*2));
        //println!("thresh_bits {}", thresh_bits);
        if (thresh_bits > 0) { // hit over threshold
          hits.push_back(std::make_tuple(dsi, j, chn, (LTBThreshold)(thresh_bits)));
        }
      }
      n_mask += 1;
    }
  }
  return hits;
}


/*************************************/

MasterTriggerEvent MasterTriggerEvent::from_bytestream(const Vec<u8> &bytestream,
                                                       u64 &pos) {

  MasterTriggerEvent event;
  
  // HACK - we make this compatible with the old data, 
  // but old data won't be useful
  //usize n_ltbs = 20;
  //// now we have to figure out if we have 20 or 25 
  //// LTBS
  //usize packet_size = MasterTriggerEvent::get_packet_size(bytestream,
  //                                                        pos);
  //if (packet_size == MasterTriggerEvent::SIZE_LTB20) {
  //  n_ltbs = 20;
  //  event = MasterTriggerEvent(n_ltbs);
  //} else if (packet_size == MasterTriggerEvent::SIZE_LTB25) {
  //  n_ltbs = 25;
  //  event = MasterTriggerEvent(n_ltbs);
  //} else {
  //  log_error("Size matches neither 20 nor 25 LTBs!");
  //  return event;
  //}
  
  u16 header = Gaps::parse_u16(bytestream, pos);
  if (header != MasterTriggerEvent::HEAD) {
    log_error("Wrong header signature!");
    return event;
  }
  event.event_status   = (EventStatus)Gaps::parse_u8 (bytestream, pos);
  event.event_id       = Gaps::parse_u32(bytestream, pos);
  event.timestamp      = Gaps::parse_u32(bytestream, pos);
  event.tiu_timestamp  = Gaps::parse_u32(bytestream, pos);
  event.tiu_gps32      = Gaps::parse_u32(bytestream, pos);
  event.tiu_gps16      = Gaps::parse_u16(bytestream, pos);
  event.crc            = Gaps::parse_u32(bytestream, pos);
  event.trigger_source = Gaps::parse_u16(bytestream, pos);
  event.dsi_j_mask     = Gaps::parse_u32(bytestream, pos);
  u8 n_channel_masks   = Gaps::parse_u8 (bytestream, pos);
  for (u8 k=0;k<n_channel_masks;k++) {
  //for _ in 0..n_channel_masks {
    event.channel_mask.push_back(Gaps::parse_u16(bytestream, pos));
  }
  event.mtb_link_mask  = Gaps::parse_u64(bytestream, pos);

  // just search the next footer and don't fill the deprecated fields
  //bool has_ended = false;
  //u64 tail_pos = search_for_2byte_marker(bytestream,0x55,has_ended,pos);   
  u16 tail = Gaps::parse_u16(bytestream, pos);
  if (tail != MasterTriggerEvent::TAIL) {
    log_error("Invalid tail signature!");
  }
  //event.n_paddles          = Gaps::parse_u8 (bytestream, pos);

  //event.set_board_mask(Gaps::parse_u3h2(bytestream, pos));
  //// FIXME
  //for (usize k=0;k<n_ltbs;k++) {
  //  u32 hitmask = Gaps::parse_u32(bytestream, pos);
  //  event.set_hit_mask(k, hitmask);
  //}
  //event.crc = Gaps::parse_u32(bytestream, pos);
  //u8 tail_a = Gaps::parse_u8 (bytestream, pos);
  //u8 tail_b = Gaps::parse_u8 (bytestream, pos);
  //if (tail_a == 85 && tail_b == 85) {
  //  log_debug("Correct tail found!");
  //}
  //else if (tail_a == 85 && tail_b == 5) {
  //  log_warn("Tail for version 0.6.0/0.6.1 found");
  //} else {
  //  log_error("Tail is messed up. See comment for version 0.6.0/0.6.1 in CHANGELOG! We got " << tail_a << " " << tail_b << " but were expecting 85 5");
  //}
  return event;
}

std::string MasterTriggerEvent::to_string() const {
  std::string repr = "<MasterTriggerEvent";
  repr += std::format("\n  event_status  : {}",(u8)event_status ); 
  repr += std::format("\n  event_id      : {}",event_id     ); 
  repr += std::format("\n  timestamp     : {}",timestamp    ); 
  repr += std::format("\n  tiu_timestamp : {}",tiu_timestamp); 
  repr += std::format("\n  tiu_gps32     : {}",tiu_gps32    ); 
  repr += std::format("\n  tiu_gps16     : {}",tiu_gps16    ); 
  repr += std::format("\n  crc           : {}",crc          );
  repr += "\n** Trigger Sources **";
  for (const auto &ts : get_trigger_sources()) {
    repr += std::format("\n -- {}", (u8)ts);
  }
  repr += "\n** Trigger Hits **";
  for (const auto &h : get_trigger_hits()) {
    repr += std::format("\n -- {} {} {} {}", std::get<0>(h), std::get<1>(h), std::get<2>(h), (u8)std::get<3>(h));
  }
  repr += "\n** MTB Link IDs **";
  for (const u8 &lid : get_rb_link_ids()) {
    repr += std::format("\n -- {}", lid);
  }
  repr += ">";
  return repr;
}

std::string TofEvent::to_string() const {
  std::string repr = "<TofEvent\n";
  repr += "  " + header.to_string()   + "\n";
  repr += "  " + mt_event.to_string() + "\n";
  repr += ".. .. ..\n";
  repr += "  n RBEvents    : " + std::to_string(rb_events.size() )     ;
  repr += "  missing hits  : " + std::to_string(missing_hits.size() ) + ">" ;
  return repr;
}

/*************************************/

f32 TofHit::get_time_a() const {
 f32 prec = 0.004;
 return prec*time_a;
}

f32 TofHit::get_time_b() const {
  f32 prec = 0.004;//ns
  return prec*time_b;
}

f32 TofHit::get_peak_a() const {
  f32 prec = 0.2;
  return prec*peak_a;
}

f32 TofHit::get_peak_b() const {
  f32 prec = 0.2;
  return prec*peak_b;
}

f32 TofHit::get_charge_a() const {
  f32 prec = 0.01; //pC
  return prec*charge_a - 50;
}

f32 TofHit::get_charge_b() const {
  f32 prec = 0.01;
  return prec*charge_b - 50;
}

f32 TofHit::get_charge_min_i() const {
  f32 prec = 0.002;// minI
  return prec*charge_min_i - 10;
}

f32 TofHit::get_x_pos() const {
  // FIXME - check if it is really in the middle
  f32 prec = 0.005; //cm
  return prec*x_pos - 163.8;
}

f32 TofHit::get_t_avg() const {
  f32 prec = 0.004;//ps
  return prec*t_average;
}

/// combine the slow timestamp with 
/// the fast to get the full
f64 TofHit::get_timestamp48() const {
  f64 ts48 = timestamp16 << 16 | timestamp32;
  return ts48;
}

TofHit TofHit::from_bytestream(const Vec<u8> &bytestream,
                               u64 &pos) {
 TofHit hit = TofHit();
 u16 maybe_header = Gaps::parse_u16(bytestream, pos);
 if (maybe_header != hit.HEAD) {
   //log_error("Can not find HEADER at presumed position. Maybe give a different value for start_pos?");
   return hit;
 }
 hit.paddle_id     = bytestream[pos]; pos+=1;
 hit.time_a        = Gaps::parse_u16(bytestream, pos); 
 hit.time_b        = Gaps::parse_u16(bytestream, pos); 
 //std::cout << " " << time_a << " " << time_b << " " << charge_a << " " << charge_b << std::endl;
 hit.peak_a        = Gaps::parse_u16(bytestream, pos); 
 hit.peak_b        = Gaps::parse_u16(bytestream, pos); 
 hit.charge_a      = Gaps::parse_u16(bytestream, pos); 
 hit.charge_b      = Gaps::parse_u16(bytestream, pos); 
 hit.charge_min_i  = Gaps::parse_u16(bytestream, pos); 
 hit.x_pos         = Gaps::parse_u16(bytestream, pos); 
 hit.t_average     = Gaps::parse_u16(bytestream, pos); 

 hit.ctr_etx = bytestream[pos]; pos+=1;

 hit.timestamp32 = Gaps::parse_u32(bytestream, pos);
 hit.timestamp16 = Gaps::parse_u16(bytestream, pos);

 // FIXME checks - packetlength, checksum ?
 u16 tail = Gaps::parse_u16(bytestream, pos);
 if (tail != TAIL) {
   log_error("TofHit TAIL signature " << tail << " is incorrect!");
 }
 //if (tail != 0xF0F) {
 //  broken = true;
 //}
 return hit; 
}

std::string TofHit::to_string() const {
  std::string repr = "<TofHit";
  //repr += std::format("\n -- format test {:.2f}", get_time_a() );
  repr += "\n  paddle ID         : "     + std::to_string(paddle_id         );
  repr += "\n  timestamp32       : "     + std::to_string(timestamp32       );
  repr += "\n  timestamp16       : "     + std::to_string(timestamp16       );
  repr += "\n   |-> timestamp48  : "     + std::to_string(get_timestamp48() ); 
  repr += "\n  _________";
  repr += "\n  ##  Peak:";
  repr += std::format("\n  >> time   A | B  : {} {}", get_time_a(), get_time_b());
  //repr += "\n  >>  time   A | B  : "     + std::to_string(get_time_a()      )
  //     +  " " + std::to_string(get_time_b());
  repr += std::format("\n  >>  height A | B  : {} {}",get_peak_a(), get_peak_b());
  repr += std::format("\n  >>  charge A | B  : {} {}",get_charge_a(), get_charge_b());
  //repr += "\n  >>  height A | B  : "     + std::to_string(get_peak_a()      )
  //     +  " " + std::to_string(get_time_a());
  //repr += "\n  >>  charge A | B  : "     + std::to_string(get_charge_a()    )
  //     +  " " + std::to_string(get_time_b());
  repr += "\n  >>  charge min_I  : "     + std::to_string(get_charge_min_i());
  repr += "\n  >>  in pad. pos   : "     + std::to_string(get_x_pos()       );
  repr += "\n  >>  t_avg         : "     + std::to_string(get_t_avg()       );
  repr += "\n  cntr ETX          : "     + std::to_string(ctr_etx           );
  repr += "\n  broken (?depr)    : "     + std::to_string(broken            );
  repr += ">";
  return repr;
}

RBWaveform RBWaveform::from_bytestream(const Vec<u8> &stream,
                                       u64 &pos) {
  RBWaveform wf = RBWaveform();
  u16 head = Gaps::parse_u16(stream, pos);
  if (head != RBWaveform::HEAD)  {
    //log_error("[RBEvent::from_bytestream] Header signature invalid!");  
    return wf;
  }
  wf.event_id   = Gaps::parse_u32(stream, pos);
  wf.rb_id      = Gaps::parse_u8(stream, pos);
  wf.rb_channel = Gaps::parse_u8(stream, pos); 
  wf.stop_cell  = Gaps::parse_u16(stream, pos);
  Vec<u8>::const_iterator start = stream.begin() + pos;
  Vec<u8>::const_iterator end   = stream.begin() + pos + 2*NWORDS;    // 2*NWORDS because stream is Vec::<u8> and it is 16 bit words.
  Vec<u8> data(start, end);
  wf.adc = u8_to_u16(data);
  pos += 2*NWORDS;
  u16 tail   = Gaps::parse_u16(stream, pos);
  if (tail != RBWaveform::TAIL) {
    log_error("After parsing, we found an invalid tail signature " << tail);
  }
  return wf;
} 

std::string RBWaveform::to_string() const {
  std::string repr = "<RBWaveform";
  //repr += std::format("\n  format test {:.2f}", get_time_a() );
  repr += std::format("\n  Event ID  : {}", event_id);
  repr += std::format("\n  RB        : {}", rb_id);
  repr += std::format("\n  Channel   : {}", rb_channel);
  repr += std::format("\n  Stop cell : {}", stop_cell);
  if (adc.size() >= 273) {
    repr += std::format("\n  adc[{}]    : .. {} {} {} ..", adc.size(), adc[270], adc[271], adc[272]);
  } else {
    repr += std::format("\n  adc [{}/corrupt?]", adc.size());
  }
  repr += ">";
  return repr;
}

Vec<TriggerType> TofEventSummary::get_trigger_sources() const {
  auto t_types = Vec<TriggerType>();
  u16 gaps_trigger = (trigger_sources >> 5 & 0x1) == 1;
  if (gaps_trigger) {
    t_types.push_back(TriggerType::Gaps);
  }
  u16 any_trigger    = (trigger_sources >> 6 & 0x1) == 1;
  if (any_trigger) {
    t_types.push_back(TriggerType::Any);
  }
  u16 forced_trigger = (trigger_sources >> 7 & 0x1) == 1;
  if (forced_trigger) {
    t_types.push_back(TriggerType::Forced);
  }
  u16 track_trigger  = (trigger_sources >> 8 & 0x1) == 1;
  if (track_trigger) {
    t_types.push_back(TriggerType::Track);
  }
  u16 central_track_trigger
                     = (trigger_sources >> 9 & 0x1) == 1;
  if (central_track_trigger) {
    t_types.push_back(TriggerType::TrackCentral);
  }
  return t_types;
} 

Vec<std::tuple<u8, u8, u8, LTBThreshold>> TofEventSummary::get_trigger_hits() const {
  auto hits = Vec<std::tuple<u8,u8,u8,LTBThreshold>>(); 
  //let n_masks_needed = self.dsi_j_mask.count_ones() / 2 + self.dsi_j_mask.count_ones() % 2;
  auto dsi_j_mask_bits = std::bitset<32>(dsi_j_mask);
  u32 n_masks_needed   = dsi_j_mask_bits.count();
  if (channel_mask.size() < n_masks_needed) {
    log_error("We need " << n_masks_needed << " hit masks, but only have " << channel_mask.size() << "! This is bad!");
    return hits;
  }
  u8 n_mask = 0;
  for (u8 k=0;k<32;k++) {
    if ((u32)((dsi_j_mask >> k) & 0x1) == 1) {
      u8 dsi = 0;
      u8 j   = 0;
      if (k < 5) {
        dsi = 1;
        j   = k  + 1;
      } else if (k < 10) {
        dsi = 2;
        j   = k  - 5 + 1;
      } else if (k < 15) {
        dsi = 3;
        j   = k - 10 + 1;
      } else if (k < 20) {
        dsi = 4;
        j   = k - 15 + 1;
      } else if (k < 25) {
        dsi = 5;
        j   = k - 20 + 1;
      } 
      //println!("n_mask {n_mask}");
      u32 channels = channel_mask[n_mask]; 
      for (u8 i=0;i<8; i++) {
        u32 ch  = LTB_CHANNELS[i];
        u32 chn = i + 1; 
        //for (i,ch) in LTB_CHANNELS.iter().enumerate() {
        //let chn = ch + 1;
        //println!("i,ch {}, {}", i, ch);
        u32 thresh_bits = (u8)(channels & (ch) >> (i*2));
        //println!("thresh_bits {}", thresh_bits);
        if (thresh_bits > 0) { // hit over threshold
          hits.push_back(std::make_tuple(dsi, j, chn, (LTBThreshold)(thresh_bits)));
        }
      }
      n_mask += 1;
    }
  }
  return hits;
}

Vec<u8> TofEventSummary::get_rb_link_ids() const {
  auto links = Vec<u8>();
  for (u8 k=0;k<64;k++) {
    if (((u64)(mtb_link_mask >> k) & (u64)0x1) == 1) {
      links.push_back(k);
    }
  }
  return links;
}


TofEventSummary TofEventSummary::from_bytestream(const Vec<u8> &stream, 
                                                 u64 &pos) {
  TofEventSummary tes;
  u16 head = Gaps::parse_u16(stream, pos);
  if (head != TofEventSummary::HEAD) {
    log_error("Decoding of HEAD failed! Got " << head << "instead!");
    //return Err(SerializationError::HeadInvalid);
  }
  tes.status            = Gaps::parse_u8(stream, pos);
  tes.trigger_sources   = Gaps::parse_u16(stream, pos);
  tes.n_trigger_paddles = Gaps::parse_u8(stream, pos);
  tes.event_id          = Gaps::parse_u32(stream, pos);
  tes.quality           = Gaps::parse_u8(stream, pos);
  tes.timestamp32       = Gaps::parse_u32(stream, pos);
  tes.timestamp16       = Gaps::parse_u16(stream, pos);
  tes.primary_beta      = Gaps::parse_u16(stream, pos); 
  tes.primary_charge    = Gaps::parse_u16(stream, pos); 
  tes.dsi_j_mask        = Gaps::parse_u32(stream, pos);
  u8 n_channel_masks    = Gaps::parse_u8(stream, pos);
  for (u8 k=0;k<n_channel_masks;k++) {
    tes.channel_mask.push_back(Gaps::parse_u16(stream, pos));
  }
  tes.mtb_link_mask     = Gaps::parse_u64(stream, pos);
  u16 nhits             = Gaps::parse_u16(stream, pos);
  for (u16 k=0; k<nhits; k++) {
    TofHit h = TofHit::from_bytestream(stream, pos);
    tes.hits.push_back(h);
  }
  u16 tail = Gaps::parse_u16(stream, pos);
  if (tail != TofEventSummary::TAIL) {
    log_error("Decoding of TAIL failed! Got " << tail << " instead!");
  }
  return tes;
}

u64 TofEventSummary::get_timestamp48() const {
  return ((u64)timestamp16 << 32) | (u64)timestamp32;
}

std::string TofEventSummary::to_string() const {
  std::string repr = "<TofEventSummary";
  //repr += std::format("\n  format test {:.2f}", get_time_a() );
  repr += std::format("\n  Status             : {}", status);
  repr += std::format("\n  Quality            : {}", quality);
  repr += std::format("\n  Trigger Sources    : {}", trigger_sources);
  repr += std::format("\n  N trig paddles     : {}", n_trigger_paddles);
  repr += std::format("\n  Event ID           : {}", event_id);
  repr += std::format("\n  timestamp32        : {}", timestamp32)      ;
  repr += std::format("\n  timestamp16        : {}", timestamp16)      ;
  repr += std::format("\n  |->timestamp48     : {}", get_timestamp48());
  repr += std::format("\n  NHits       (reco) : {}", hits.size());
  repr += std::format("\n  Prim Beta   (reco) : {}", primary_beta);
  repr += std::format("\n  Prim Charge (reco) : {}", primary_beta);
  repr += "\n** Trigger Sources **";
  for (const auto &ts : get_trigger_sources()) {
    repr += std::format("\n -- {}", (u8)ts);
  }
  repr += "\n** Trigger Hits **";
  for (const auto &h : get_trigger_hits()) {
    repr += std::format("\n -- {} {} {} {}", std::get<0>(h), std::get<1>(h), std::get<2>(h), (u8)std::get<3>(h));
  }
  repr += "\n** MTB Link IDs **";
  for (const u8 &lid : get_rb_link_ids()) {
    repr += std::format("\n -- {}", lid);
  }
  repr += ">";
  repr += "\n  **** **** ****";
  for (auto const &h : hits) {
    repr += std::format("\n  {}",h.to_string()); 
  }
  repr += ">";
  return repr;
}

std::ostream& operator<<(std::ostream& os, const TofHit& th) {
  os << th.to_string();
  return os;
}

std::ostream& operator<<(std::ostream& os, const MasterTriggerEvent& mt) {
  os << mt.to_string();
  return os;
}

std::ostream& operator<<(std::ostream& os, const TofEvent& te) {
  os << te.to_string();
  return os;
}

std::ostream& operator<<(std::ostream& os, const RBEvent& re) {
  os << re.to_string();
  return os;
}

std::ostream& operator<<(std::ostream& os, const RBEventHeader& rh) {
  os << rh.to_string();
  return os;
}

std::ostream& operator<<(std::ostream& os, const RBWaveform& wf) {
  os << wf.to_string();
  return os;
}

std::ostream& operator<<(std::ostream& os, const TofEventSummary& tes) {
  os << tes.to_string();
  return os;
}

