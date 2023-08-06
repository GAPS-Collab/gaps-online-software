#include "events.h"
#include "parsers.h"
#include "serialization.h"

#include "spdlog/spdlog.h"
#include "spdlog/cfg/env.h"


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




//Vec<RBBinaryDump> get_level0_events(const String &filename) {
//  spdlog::cfg::load_env_levels();
//  Vec<RBBinaryDump> events;
//  u64 n_good = 0;  
//  u64 n_bad  = 0; 
//  bytestream stream = get_bytestream_from_file(filename); 
//  bool has_ended = false;
//  auto pos = search_for_2byte_marker(stream,0xAA, has_ended );
//  spdlog::info("Read {} bytes from {}", stream.size(), filename);
//  spdlog::info("For 8+1 channels and RB compression level 0, this mean a max number of events of {}", stream.size()/18530.0);
//  while (!has_ended) {
//    //RBBinaryDump data = RBBinaryDump::from_bytestrea(stream, pos);
//    //header.broken ? n_bad++ : n_good++ ;
//    BlobEvt_t event;
//    event
//    events.push_back(data);
//    pos -= 2;
//    pos = search_for_2byte_marker(stream, 0xAA, has_ended, pos);
//    //if (header.broken) {
//    //  std::cout << pos << std::endl;
//    //  std::cout << (u32)header.channel_mask << std::endl;
//    //}
//  }
//  spdlog::info("Retrieved {} good events, but {} of which we had to set the `broken` flag", n_good, n_bad);
//  return events;
//}



RBEventHeader::RBEventHeader() {
  channel_mask       = 0; 
  stop_cell          = 0; 
  crc32              = 0;
  drs4_temp          = 0; 
  dtap0              = 0;
  is_locked          = false; 
  is_locked_last_sec = false;
  lost_trigger       = false;
  fpga_temp          = 0;
  event_id           = 0; 
  rb_id              = 0; 
  timestamp_48       = 0; 
  broken             = true;
}

RBEventHeader RBEventHeader::from_bytestream(const Vec<u8> &stream,
                                             u64 &pos){

  RBEventHeader header;
  pos += 2;
  header.channel_mask        = Gaps::parse_u8(stream  , pos);   
  header.stop_cell           = Gaps::parse_u16(stream , pos);  
  header.crc32               = Gaps::parse_u32(stream , pos);  
  header.dtap0               = Gaps::parse_u16(stream , pos);  
  header.drs4_temp           = Gaps::parse_u16(stream , pos);  
  header.is_locked           = Gaps::parse_bool(stream, pos);
  header.is_locked_last_sec  = Gaps::parse_bool(stream, pos);
  header.lost_trigger        = Gaps::parse_bool(stream, pos);
  header.fpga_temp           = Gaps::parse_u16(stream , pos);  
  header.event_id            = Gaps::parse_u32(stream , pos);  
  header.rb_id               = Gaps::parse_u8(stream  , pos);  
  header.timestamp_48        = Gaps::parse_u64(stream , pos);  
  header.broken              = Gaps::parse_bool(stream, pos);  
  pos += 2; //FIXME TAIL & HEADER check
  return header; 
}



f32 RBEventHeader::get_fpga_temp() const {
  f32 temp = (fpga_temp * 503.975/4096) - 273.15;
  return temp;
}

u64 RBEventHeader::get_clock_cycles_48bit() const {
  return timestamp_48;
}

Vec<u8> RBEventHeader::get_active_data_channels() const {
  Vec<u8> active_channels;
  for (auto const &ch : {1,2,3,4,5,6,7,8} ) {
    if ((channel_mask & (u8)pow(2, ch - 1)) == (u8)pow(2,ch - 1)) active_channels.push_back(ch);
  } 
  //if ((channel_mask & 1)   == 1)   active_channels.push_back(1);
  return active_channels;
}

u8 RBEventHeader::get_n_datachan() const {
  Vec<u8> active_channels = get_active_data_channels();
  return (u8)active_channels.size();
}

f32 RBEventHeader::get_drs_temp() const {
  f32 temp = drs_adc_to_celsius(drs4_temp);
  return temp;
}

f32 RBEventHeader::drs_adc_to_celsius(u16 adc) const {
  f32 sign = 1.0;
  if (adc >= 0x800) {
    sign = -1.0;
    adc = 0xFFF - adc;
  }
  return sign * (f32)adc * 0.0625;
}                                             


RBEventHeader RBEventHeader::extract_from_rbbinarydump(const Vec<u8> &bs,
                                                       u64 &pos) {
  RBEventHeader header = RBEventHeader();
  u64 start = pos;
  spdlog::debug("Start decoding at pos {}", pos);
  u16 head = Gaps::u16_from_le_bytes(bs, pos);
  if (head != RBEventHeader::HEAD)  {
    spdlog::error("No header signature found!");  
  }
  // status field is at bytes 3,4;
  u16 status = Gaps::u16_from_le_bytes(bs, pos);
  header.lost_trigger = (status & 2 ) == 2;
  header.is_locked    = (status & 4 ) == 4;
  header.is_locked_last_sec = (status & 8) == 8;
  header.fpga_temp    = (status >> 4); 
  
  pos += 2 + 2 + 8 + 2 + 1; // skip len, roi, dna, fw hash and reserved part of rb_id
  header.rb_id = bs[pos];
  pos += 1;
  header.channel_mask = bs[pos];
  pos += 2;
  //header.event_id  = Gaps::u32_from_be_bytes(bs, pos);
  header.event_id     = Gaps::parse_u32_for_16bit_words(bs, pos); 
  header.dtap0        = Gaps::u16_from_le_bytes(bs, pos);
  header.drs4_temp    = Gaps::u16_from_le_bytes(bs, pos);
  header.timestamp_48 = Gaps::parse_u48_for_16bit_words(bs, pos);
  // FIXME - currently, the number of samples is still fixed
  u8 nchan = header.get_n_datachan();
  // we have to skip for each ch:
  // 1) head -> 2bytes
  // 2) NWORDS * 2bytes 
  // 3) trail -> 4bytes
  // if no channel is active, ch9 won't be active,
  // otherwise ch9 is ALWAYS active
  
  // THe header up to this point consists of 36 bytes
  u32 skip_bytes = 0;
  if ( nchan != 0) {
    skip_bytes = (nchan + 1) * (NWORDS * 2 + 6);
  }
  spdlog::debug("Skipping {} bytes of channel data!", skip_bytes);
  pos += skip_bytes;
  header.stop_cell = Gaps::parse_u16(bs, pos);
  header.crc32     = Gaps::parse_u32_for_16bit_words(bs, pos);
  spdlog::debug("Looking for TAIL at pos {}", pos);
  u16 tail         = Gaps::u16_from_le_bytes(bs, pos);
  if (tail != RBEventHeader::TAIL)  {
    spdlog::error("No tail signature found {} bytes from the start! Found {} instead", pos -start - 2, tail );  
  } else {
  header.broken = false;
  }
  return header;
} 

/**********************************************************/

RBEvent RBEvent::from_bytestream(const Vec<u8> &stream,
                                 u64 &pos) {
  RBEvent event = RBEvent();
  spdlog::debug("Start decoding at pos {}", pos);
  u16 head = Gaps::parse_u16(stream, pos);
  if (head != RBEvent::HEAD)  {
    spdlog::error("No header signature found!");  
  }
  event.header   = RBEventHeader::from_bytestream(stream, pos);
  Vec<u8> ch_ids = event.header.get_active_data_channels();
  for (auto k : ch_ids) {
    spdlog::debug("Found active data channel {}!", k);
    Vec<u8>::const_iterator start = stream.begin() + pos;
    Vec<u8>::const_iterator end   = stream.begin() + pos + 2*NWORDS;    // 2*NWORDS because stream is Vec::<u8> and it is 16 bit words.
    Vec<u8> data(start, end);
    event.adc[(k-1)] = u8_to_u16(data);
    pos += 2*NWORDS;
  }
  u16 tail = Gaps::parse_u16(stream, pos);
  if (tail != RBEvent::TAIL) {
    spdlog::error("After parsing the event, we found an invalid tail signature {}", tail);
  }
  return event;
}

/**********************************************************/

RBMissingHit RBMissingHit::from_bytestream(const Vec<u8> &stream,
                                           u64 &pos) {
  spdlog::debug("Start decoding at pos {}", pos);
  u16 head = Gaps::parse_u16(stream, pos);
  if (head != RBMissingHit::HEAD)  {
    spdlog::error("No header signature found!");  
  }
  // verify_fixed already advances pos by 2
  RBMissingHit miss  = RBMissingHit();
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
    spdlog::error("After parsing the event, we found an invalid tail signature {}", tail);
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
  
TofEvent TofEvent::from_bytestream(const Vec<u8> &stream,
                                   u64 &pos) {
  spdlog::debug("Start decoding at pos {}", pos);
  u16 head = Gaps::parse_u16(stream, pos);
  if (head != TofEvent::HEAD)  {
    spdlog::error("No header signature found!");  
  }
  TofEvent event = TofEvent();
  //  event.compression_level = CompressionLevel::from_u8(&parse_u8(stream, pos));
  //  event.quality           = EventQuality::from_u8(&parse_u8(stream, pos));
  //  event.mt_event          = MasterTriggerEvent::from_bytestream(stream, pos)?;
  //  let v_sizes = Self::decode_size_header(&parse_u32(stream, pos));
  //  for k in 0..v_sizes.0 {
  //    match RBEvent::from_bytestream(stream, pos) {
  //      Err(err) => error!("Expected RBEvent {} of {}, but got serialization error {}!", k,  v_sizes.0, err),
  //      Ok(ev) => {
  //        event.rb_events.push(ev);
  //      }
  //    }
  //  }
  //  for k in 0..v_sizes.1 {
  //    match RBMissingHit::from_bytestream(stream, pos) {
  //      Err(err) => error!("Expected RBMissingHit {} of {}, but got serialization error {}!", k,  v_sizes.1, err),
  //      Ok(miss) => {
  //        event.missing_hits.push(miss);
  //      }
  //    }
  //  }
  //  for k in 0..v_sizes.2 {
  //    //match PaddlePacket::from_bytestream(stream, pos) {
  //    //  Err(err) => error!("Expected PaddlePacket {} of {}, but got serialization error {}!", k,  v_sizes.2, err),
  //    //  Ok(pp) => {
  //    //    event.paddle_packets.push(pp);
  //    //  }
  //    //}
  //    //event.paddle_packets(PaddlePacket::from_bytestream(stream, pos));
  //  }
  //  for k in 0..v_sizes.3 {
  //    match RBMoniData::from_bytestream(stream, pos) {
  //      Err(err) => error!("Expected RBMoniPacket {} of {}, but got serialization error {}!", k,  v_sizes.3, err),
  //      Ok(moni) => {
  //        event.rb_moni.push(moni);
  //      }
  //    }
  //  }
  //  Ok(event)
  //}
  return event;
}

