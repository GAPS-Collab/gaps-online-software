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

/*************************************/

RBEventMemoryView::RBEventMemoryView() {
  status   = 0;
  len      = 0;
  roi      = 0;
  dna      = 0;
  fw_hash  = 0;
  id       = 0;
  ch_mask  = 0;
  event_ctr = 0;
  dtap0     = 0;
  dtap1     = 0;
  timestamp = 0 ;
  for (usize k=0; k<NCHN; k++) {
    ch_head[k]  = 0;
    ch_trail[k] = 0;
    for (usize n=0; n<NWORDS;n++) {
      ch_adc[k][n] = 0;
    }
  }
  stop_cell  = 0;
  crc32      = 0;
}

/*************************************/

Vec<u16> RBEventMemoryView::get_channel_adc(u8 channel) const {
  Vec<u16> adc = Vec<u16>(NWORDS, 0);
  if (channel == 0) {
    spdlog::error("Please remembmer channel ids are ranged from 1-9! Ch0 does not exist");
    return adc;
  }
  //  adc[k] = ch_adc[channel -1][k];
  //}
  return adc;
}

/*************************************/

RBEventMemoryView RBEventMemoryView::from_bytestream(const Vec<u8> &stream,
                                                     u64 &pos) {
  RBEventMemoryView event;
  event.head      = Gaps::parse_u16( stream, pos); 
  event.status    = Gaps::parse_u16( stream, pos); 
  event.len       = Gaps::parse_u16( stream, pos); 
  event.roi       = Gaps::parse_u16( stream, pos); 
  event.dna       = decode_uint64_rev( stream, pos); pos += 8;
  event.fw_hash   = Gaps::parse_u16( stream, pos);
  // the first byte of the event id short is RESERVED
  event.id        = stream[pos + 1]; pos += 2;
  event.ch_mask   = Gaps::parse_u16( stream, pos);
  event.event_ctr = Gaps::parse_u32_for_16bit_words( stream, pos);
  event.dtap0     = Gaps::parse_u16( stream, pos); 
  event.dtap1     = Gaps::parse_u16( stream, pos); 
  event.timestamp = Gaps::parse_u48_for_16bit_words( stream, pos);
  for (int i=0; i<NCHN; i++) {
    event.ch_head[i] = Gaps::parse_u16(stream, pos);
    // Read the channel data
    for (int j=0; j<NWORDS; j++) {
      //event.ch_adc[i][j] = decode_14bit(bytestream, dec_pos); dec_pos += 2;
      event.ch_adc[i][j] = Gaps::parse_u16(stream, pos) & 0x3FFF; 
    }
    event.ch_trail[i] = Gaps::parse_u32_for_16bit_words(stream, pos); 
  }    
  
  event.stop_cell = Gaps::parse_u16(stream, pos); 
  event.crc32     = Gaps::parse_u32_for_16bit_words(stream, pos);
  event.tail      = Gaps::parse_u16(stream, pos);
  return event;
}

/*************************************/

RBEventHeader::RBEventHeader() {
  channel_mask       = 0; 
  stop_cell          = 0; 
  crc32              = 0;
  drs4_temp          = 0; 
  dtap0              = 0;
  is_locked          = false; 
  is_locked_last_sec = false;
  lost_trigger       = false;
  event_fragment     = false;
  fpga_temp          = 0;
  event_id           = 0; 
  rb_id              = 0; 
  timestamp_48       = 0; 
  broken             = true;
}

/*************************************/

std::string RBEventHeader::to_string() const {
  std::string repr = "<RBEventHeader";
  repr += "\n\t rb id          " + std::to_string(rb_id)                 ;
  repr += "\n\t has ch9        " + std::to_string(has_ch9);
  repr += "\n\t event id       " + std::to_string(event_id)              ;
  repr += "\n\t is locked      " + std::to_string(is_locked)             ;
  repr += "\n\t is locked (1s) " + std::to_string(is_locked_last_sec)    ;
  repr += "\n\t lost trigger   " + std::to_string(lost_trigger)          ;
  repr += "\n\t event fragment " + std::to_string(event_fragment)        ;
  repr += "\n\t channel mask   " + std::to_string(channel_mask)          ;
  repr += "\n\t stop cell      " + std::to_string(stop_cell)             ;
  repr += "\n\t crc32          " + std::to_string(crc32)                 ;
  repr += "\n\t dtap0          " + std::to_string(dtap0)                 ;
  repr += "\n\t timestamp (48bit) " + std::to_string(timestamp_48)        ;
  repr += "\n\t FPGA temp [C]  " + std::to_string(get_fpga_temp())     ;
  repr += "\n\t DRS4 temp [C]  " + std::to_string(get_drs_temp())      ;
  repr += ">";
  return repr;
}

/*************************************/

RBEventHeader RBEventHeader::from_bytestream(const Vec<u8> &stream,
                                             u64 &pos){

  RBEventHeader header;
  pos += 2;
  header.channel_mask        = Gaps::parse_u8(stream  , pos);   
  header.has_ch9             = Gaps::parse_bool(stream, pos);
  header.stop_cell           = Gaps::parse_u16(stream , pos);  
  header.crc32               = Gaps::parse_u32(stream , pos);  
  header.dtap0               = Gaps::parse_u16(stream , pos);  
  header.drs4_temp           = Gaps::parse_u16(stream , pos);  
  header.is_locked           = Gaps::parse_bool(stream, pos);
  header.is_locked_last_sec  = Gaps::parse_bool(stream, pos);
  header.lost_trigger        = Gaps::parse_bool(stream, pos);
  header.event_fragment      = Gaps::parse_bool(stream, pos);
  header.fpga_temp           = Gaps::parse_u16(stream , pos);  
  header.event_id            = Gaps::parse_u32(stream , pos);  
  header.rb_id               = Gaps::parse_u8(stream  , pos);  
  header.timestamp_48        = Gaps::parse_u64(stream , pos);  
  header.broken              = Gaps::parse_bool(stream, pos);  
  u16 tail                   = Gaps::parse_u16(stream, pos);
  if (tail != RBEventHeader::TAIL) {
    spdlog::error("Tail signature incorrect! Got tail {}", tail);
  }
  return header; 
}

/*************************************/

f32 RBEventHeader::get_fpga_temp() const {
  f32 zynq_temp = (((fpga_temp & 4095) * 503.975) / 4096.0) - 273.15;
  //f32 temp = (fpga_temp * 503.975/4096) - 273.15;
  return zynq_temp;
}

/*************************************/

u64 RBEventHeader::get_clock_cycles_48bit() const {
  return timestamp_48;
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

f32 RBEventHeader::get_drs_temp() const {
  f32 temp = drs_adc_to_celsius(drs4_temp);
  return temp;
}

/*************************************/

f32 RBEventHeader::drs_adc_to_celsius(u16 adc) const {
  f32 sign = 1.0;
  if (adc >= 0x800) {
    sign = -1.0;
    adc = 0xFFF - adc;
  }
  return sign * (f32)adc * 0.0625;
}                                             

/*************************************/

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
  header.event_fragment = (status & 1 ) == 1;
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

RBEvent::RBEvent() {  
  header = RBEventHeader();
  data_type = 0;
  nchan     = 0;
  npaddles  = 0;
  adc    = Vec<Vec<u16>>(); 
  for (usize k=0; k<NCHN; k++) {
    adc.push_back(Vec<u16>(NWORDS/2,0));
  }
}

/**********************************************************/

std::string RBEvent::to_string() const {
  std::string repr = "<RBEvent\n";
  repr += header.to_string();
  repr += "\n";
  if (adc.size() > 0) {
    repr += "ADC CHANNELS : " + std::to_string(adc.size());
    repr += "\n-- --  Ch 0 -- --\n";
    repr += std::to_string(adc[0][0]);
    repr += " "; 
    repr += std::to_string(adc[0][1]);
    repr += " .. .. \n"; 
  }
  repr += ">";
  return repr;
}


/**********************************************************/

bool RBEvent::channel_check(u8 channel) const {
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
  
const Vec<u16>& RBEvent::get_channel_adc(u8 channel) const {
  if (!(channel_check(channel))) {
    return _empty_channel;
  }
  if (channel == 9) {
    return ch9_adc;
  }
  return adc[channel -1]; 
}
  
const Vec<u16>& RBEvent::get_channel_by_label(u8 channel) const {
  if (channel == 9) {
    return ch9_adc;
  }
  return adc[channel];
}

const Vec<u16>& RBEvent::get_channel_by_id(u8 channel) const {
  if (channel == 8) {
    return ch9_adc;
  }
  return adc[channel - 1];
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
  event.data_type = Gaps::parse_u8(stream, pos);
  event.nchan     = Gaps::parse_u8(stream, pos);
  event.npaddles  = Gaps::parse_u8(stream, pos); 
  event.header    = RBEventHeader::from_bytestream(stream, pos);
  spdlog::debug("Decoded RBEventHeader!");
  for (usize ch=0; ch<event.nchan; ch++) {
    spdlog::debug("Found active data channel {}!", ch);
    Vec<u8>::const_iterator start = stream.begin() + pos;
    Vec<u8>::const_iterator end   = stream.begin() + pos + 2*NWORDS;    // 2*NWORDS because stream is Vec::<u8> and it is 16 bit words.
    Vec<u8> data(start, end);
    event.adc[ch] = u8_to_u16(data);
    pos += 2*NWORDS;
  }
  if (event.header.has_ch9) {
    spdlog::debug("Trying to parse ch9 data!");
    Vec<u8>::const_iterator start = stream.begin() + pos;
    Vec<u8>::const_iterator end   = stream.begin() + pos + 2*NWORDS;    // 2*NWORDS because stream is Vec::<u8> and it is 16 bit words.
    Vec<u8> data(start, end);
    event.ch9_adc = u8_to_u16(data);
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

u32 TofEvent::get_n_rbmissinghits(u32 mask){
  return ((mask & 0xFF00)     >> 8);
}

/**********************************************************/

u32 TofEvent::get_n_rbevents(u32 mask){
  return (mask & 0xFF);
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
  // for now skip quality and compression level
  pos += 2;
  event.header   = TofEventHeader::from_bytestream(stream, pos);
  event.mt_event = MasterTriggerEvent::from_bytestream(stream, pos);
  //pos += 45; // for now skip master trigger event
  u32 mask          = Gaps::parse_u32(stream, pos);
  u32 n_rbevents    = get_n_rbevents(mask);
  u32 n_missing     = get_n_rbmissinghits(mask);
  spdlog::debug("Expecting {} RBEvents, {} RBMissingHits",
                n_rbevents, n_missing);

  for (u32 k=0; k< n_rbevents; k++) {
    RBEvent rb_event = RBEvent::from_bytestream(stream, pos);
    event.rb_events.push_back(rb_event);
  }
  for (u32 k=0; k< n_missing; k++) {
    RBMissingHit missy = RBMissingHit::from_bytestream(stream, pos);
    event.missing_hits.push_back(missy);
  }
  
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
  
/**********************************************************/

TofEvent TofEvent::from_tofpacket(const TofPacket &packet) {
  TofEvent event;
  if (packet.packet_type != PacketType::TofEvent) {
    spdlog::error("Wrong packet type! {}", packet_type_to_string(packet.packet_type));
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
  spdlog::error("No RBEvent for board {}", board_id);
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
  spdlog::error("This is not implmented yet!");
  return false;
}

MasterTriggerEvent::MasterTriggerEvent() {
  event_id      = 0; 
  timestamp     = 0; 
  tiu_timestamp = 0; 
  tiu_gps_32    = 0; 
  tiu_gps_16    = 0; 
  n_paddles     = 0; 
  std::fill(board_mask, board_mask + N_LTBS, 0);
  for (usize k=0;k<N_LTBS;k++) {
    std::fill(hits[k], hits[k] + N_CHN_PER_LTB, 0);
  }
  crc = 0;
  broken = true;
  valid  = false;
}  

/**********************************************************/

/// Helper to get the number of the triggered LTB from the bitmask
void MasterTriggerEvent::decode_board_mask(u32 mask_number, bool (&decoded_mask)[N_LTBS]) {
  //bool decoded_mask[N_LTBS];
  std::fill(decoded_mask, decoded_mask + N_LTBS, false);
  //std::fill(board_mask, board_mask + N_LTBS, false);
  // FIXME this implicitly asserts that the fields for non available LTBs 
  // will be 0 and all the fields will be in order 
  usize index = N_LTBS - 1;
  //for n in 0..N_LTBS {
  for (usize n=0;n<N_LTBS; n++) {
    u32 mask = 1 << n;
    bool bit_is_set = (mask & mask_number) > 0;
    //decoded_mask[index] = bit_is_set;
    decoded_mask[index] = bit_is_set;
    if (index != 0) {
      index -= 1;
    }
  }
  //board_mask = decoded_mask;
}

/*************************************/

void MasterTriggerEvent::decode_hit_mask(u32 mask_number, bool (&hitmask_1)[N_CHN_PER_LTB], bool (&hitmask_2)[N_CHN_PER_LTB]) {
  //let mut decoded_mask_0 = [false;N_CHN_PER_LTB];
  //let mut decoded_mask_1 = [false;N_CHN_PER_LTB];
  // FIXME this implicitly asserts that the fields for non available LTBs 
  // will be 0 and all the fields will be in order
  u32 index = N_CHN_PER_LTB - 1;
  //for n in 0..N_CHN_PER_LTB {
  for (usize n=0; n<N_CHN_PER_LTB; n++) {
    u32 mask = 1 << n;
    //println!("MASK {:?}", mask);
    bool bit_is_set = (mask & mask_number) > 0;
    hitmask_1[index] = bit_is_set;
    if (index != 0) {
      index -= 1;
    }
  }
  index = N_CHN_PER_LTB -1;
  for (usize n=N_CHN_PER_LTB; n<2*N_CHN_PER_LTB; n++) {
  //for n in N_CHN_PER_LTB..2*N_CHN_PER_LTB {
    u32 mask = 1 << n;
    bool bit_is_set = (mask & mask_number) > 0;
    hitmask_2[index] = bit_is_set;
    if (index != 0) {
      index -= 1;
    }
  }
}

/*************************************/
  
void MasterTriggerEvent::set_board_mask(u32 mask) {
  // FIXME -> This basically inverses the order of the LTBs
  // so bit 0 (rightmost in the mask is the leftmost in the 
  // array 
  for (usize i=0;i<N_LTBS;i++) {
     board_mask[i] = (mask & (1 << i)) != 0;
  } 
}   

/*************************************/
        
void MasterTriggerEvent::set_hit_mask(usize ltb_idx, u32 mask) {
  for (usize i=0;i<N_CHN_PER_LTB;i++) {
    hits[ltb_idx][i] = (mask & (1 << i)) != 0;
  } 
}

/*************************************/

MasterTriggerEvent MasterTriggerEvent::from_bytestream(const Vec<u8> &bytestream,
                                                       u64 &pos) {

  MasterTriggerEvent event;
  u16 header = Gaps::parse_u16(bytestream, pos);
  if (header != MasterTriggerEvent::HEAD) {
    spdlog::error("Wrong header signature!");
    return event;
  }
  event.event_id           = Gaps::parse_u32(bytestream, pos);
  event.timestamp          = Gaps::parse_u32(bytestream, pos);
  event.tiu_timestamp      = Gaps::parse_u32(bytestream, pos);
  event.tiu_gps_32         = Gaps::parse_u32(bytestream, pos);
  event.tiu_gps_16         = Gaps::parse_u32(bytestream, pos);
  event.n_paddles          = Gaps::parse_u8 (bytestream, pos);

  event.set_board_mask(Gaps::parse_u32(bytestream, pos));
  //decode_board_mask(Gaps::parse_u32(bytestream, pos), event.board_mask);

  // FIXME
  for (usize k=0;k<N_LTBS;k++) {
    u32 hitmask = Gaps::parse_u32(bytestream, pos);
    event.set_hit_mask(k, hitmask);
  }
  event.crc = Gaps::parse_u32(bytestream, pos);
  u8 tail_a = Gaps::parse_u8 (bytestream, pos);
  u8 tail_b = Gaps::parse_u8 (bytestream, pos);
  if (tail_a == 85 && tail_b == 85) {
    spdlog::debug("Correct tail found!");
  }
  else if (tail_a == 85 && tail_b == 5) {
     spdlog::warn("Tail for version 0.6.0/0.6.1 found");
  } else {
    spdlog::error("Tail is messed up. See comment for version 0.6.0/0.6.1 in CHANGELOG! We got {} {} but were expecting 85 5", tail_a, tail_b);
  }
  return event;
}

std::string MasterTriggerEvent::to_string() const {
  std::string repr = "<MasterTriggerEvent :";
  repr += "\n  event_id      : " + std::to_string(event_id                    ); 
  repr += "\n  timestamp     : " + std::to_string(timestamp                   ); 
  repr += "\n  tiu_timestamp : " + std::to_string(tiu_timestamp               ); 
  repr += "\n  tiu_gps_32    : " + std::to_string(tiu_gps_32                  ); 
  repr += "\n  tiu_gps_16    : " + std::to_string(tiu_gps_16                  ); 
  repr += "\n  n_paddles     : " + std::to_string(n_paddles                   ); 
  repr += "\n  crc           : " + std::to_string(crc                         );
  repr += "\n  broken        : " + std::to_string(broken                      );
  repr += "\n  valid         : " + std::to_string(valid                       );
  repr += "\n  -- hit mask --";
  repr += "\n [DSI/J]";
  repr += "\n 1/1 - 1/2 - 1/3 - 1/4 - 1/5 - 2/1 - 2/2 - 2/3 - 2/4 - 2/5 - 3/1 - 3/2 - 3/3 - 3/4 - 3/5 - 4/1 - 4/2 - 4/3 - 4/4 - 4/5 \n";
  Vec<u8> hit_boards = Vec<u8>();
  HashMap<u8, String> dsi_j = HashMap<u8, String>();
  dsi_j[0] = "1/1";
  dsi_j[1] = "1/2";
  dsi_j[2] = "1/3";
  dsi_j[3] = "1/4";
  dsi_j[4] = "1/5";
  dsi_j[5] = "2/1";
  dsi_j[6] = "2/2";
  dsi_j[7] = "2/3";
  dsi_j[8] = "2/4";
  dsi_j[9] = "2/5";
  dsi_j[10] = "3/1";
  dsi_j[11] = "3/2";
  dsi_j[12] = "3/3";
  dsi_j[13] = "3/4";
  dsi_j[14] = "3/5";
  dsi_j[15] = "4/1";
  dsi_j[16] = "4/2";
  dsi_j[16] = "4/3";
  dsi_j[17] = "4/4";
  dsi_j[19] = "4/5";
  repr += " ";
  for (usize k=0;k<N_LTBS;k++) {
    if (board_mask[k]) {
      repr += "-X-   ";
      hit_boards.push_back(k);
    } else {
      repr += "-0-   ";
    }
  }
  repr += "\n\t == == LTB HITS [BRD CH] == ==\n";
  for (auto k : hit_boards) {
    repr += "\t DSI/J " + dsi_j[k] + "\t=> ";
    for (usize j=0;j<N_CHN_PER_LTB;j++) {
      if (hits[k][j]) {
        repr += " " + std::to_string(j + 1) + " ";
      } else {
        continue;
        //repr += " N.A. ";
      } 
    }
    repr += "\n";
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

