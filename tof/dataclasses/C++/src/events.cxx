#include "events.h"
#include "parsers.h"
#include "serialization.h"

#include "spdlog/spdlog.h"
#include "spdlog/cfg/env.h"

/*****
 * Get the event id from the first event following an offset in a bytestream
 *
 * Will set the pos variable to the location of the next possible 
 * event
 *
 * @param bytestream : A vector or bytes representing one or more 
 *                     raw readoutboard events ("blob")
 * @param pos        : The position from which to start searching 
 *                     for the next event in the bytestream                    
 */ 
vec_u32 get_event_ids_from_raw_stream(const vec_u8 &bytestream, u64 &pos) {
  vec_u32 event_ids;

  u32 event_id = 0;
  // first, we need to find the first header in the 
  // stream starting from the given position
  bool has_ended = false;
  while (!has_ended) { 
    pos = search_for_2byte_marker(bytestream, 0xAA, has_ended, pos);  
    pos += 22;
    event_id = Gaps::u32_from_le_bytes(bytestream, pos);
    event_ids.push_back(event_id);
    pos += 18530 - 22 - 4;
  }
  return event_ids; 
}

Vec<RBEventHeader> get_headers(const String &filename, bool is_headers) {
  spdlog::cfg::load_env_levels();
  u64 n_good = 0;  
  u64 n_bad  = 0; 
  Vec<RBEventHeader> headers;
  bytestream stream = get_bytestream_from_file(filename); 
  bool has_ended = false;
  auto pos = search_for_2byte_marker(stream,0xAA, has_ended );
  spdlog::info("Read {} bytes from {}", stream.size(), filename);
  spdlog::info("For 8+1 channels and RB compression level 0, this mean a max number of events of {}", stream.size()/18530.0);
  while (!has_ended) {
    RBEventHeader header;
    if (is_headers) {
      header = RBEventHeader::from_bytestream(stream, pos);
    } else {
      header = RBEventHeader::extract_from_rbbinarydump(stream, pos);
    }
    header.broken ? n_bad++ : n_good++ ;
    headers.push_back(header);
    pos -= 2;
    pos = search_for_2byte_marker(stream, 0xAA, has_ended, pos);
    if (header.broken) {
      std::cout << pos << std::endl;
      std::cout << (u32)header.channel_mask << std::endl;
    }
  }
  spdlog::info("Retrieved {} headers, {} of which we had to set the `broken` flag", n_good, n_bad);
  return headers;
}

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
  return header; 
  
  
  
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
  header.stop_cell = Gaps::u16_from_le_bytes(bs, pos);
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

