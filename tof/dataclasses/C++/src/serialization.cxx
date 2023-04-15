#include "serialization.h"

#include "TOFCommon.h"
#include "tof_typedefs.h"

u16 decode_ushort(const vec_u8& bytestream,
                             u32 start_pos)
{
  u16 value= (u16)(((bytestream[start_pos+0] & 0xFF) << 8) | bytestream[start_pos+1]);
  return value;
}

/***********************************************/

short decode_short(const vec_u8& bytestream,
                   u32 start_pos)
{
  //short value = (short)(((bytestream[start_pos+0] & 0xFF) << 8) | bytestream[start_pos+1]);
  short value = (short)(((bytestream[start_pos+0]) << 8) | bytestream[start_pos+1]);
  
  return value;
}

/***********************************************/

short decode_short_rev(const vec_u8& bytestream,
                   u32 start_pos)
{
  short value= (short)(((bytestream[start_pos+1] & 0xFF) << 8) | bytestream[start_pos+0]);
  return value;
}

/***********************************************/

u16 decode_ushort_rev(const vec_u8& bytestream,
                             u32 start_pos)
{
  u16 value= (u16)(((bytestream[start_pos+1] & 0xFF) << 8) | bytestream[start_pos+0]);
  return value;
}

/***********************************************/

void encode_ushort(u16 value,
                   vec_u8& bytestream,
                   u32 start_pos)
{
  //vec_u8 buffer(2);
  bytestream[start_pos + 0] = (value >> 8) & 0xFF;
  bytestream[start_pos + 1] = value & 0xFF;
}

/***********************************************/

int16_t decode_14bit(const vec_u8& bytestream,
                     u32 start_pos)
{
   int16_t value =  decode_short_rev(bytestream, start_pos);
   return  value & 0x3FFF;

}

/***********************************************/

void encode_ushort_rev(u16 value,
                   vec_u8& bytestream,
                   u32 start_pos)
{
  //vec_u8 buffer(2);
  bytestream[start_pos + 1] = (value >> 8) & 0xFF;
  bytestream[start_pos + 0] = value & 0xFF;
}

/***********************************************/

void encode_short_rev(short value,
                      vec_u8& bytestream,
                      u32 start_pos)
{
  //vec_u8 buffer(2);
  bytestream[start_pos + 1] = (value >> 8) & 0xFF;
  bytestream[start_pos + 0] = value & 0xFF;
}

/***********************************************/

u32 decode_uint32(const vec_u8& bytestream,
                       u32 start_pos)
{
  u32 value = (u32)(
         ((bytestream[start_pos+0] & 0xFF) << 24)
      |  ((bytestream[start_pos+1] & 0xFF) << 16)
      |  ((bytestream[start_pos+2] & 0xFF) << 8)
      |  (bytestream[start_pos+3]));
  return value;
}

/***********************************************/

u32 decode_uint32_rev(const vec_u8 &bytestream,
                           u32 start_pos)
{
  u32 value = (u32)(
         ((bytestream[start_pos+1] & 0xFF) << 24)
      |  ((bytestream[start_pos+0] & 0xFF) << 16)
      |  ((bytestream[start_pos+3] & 0xFF) << 8)
      |  (bytestream[start_pos+2]));
  return value;
}

/***********************************************/

u32 u32_from_le_bytes(const vec_u8 &bytestream,
                      u64 start_pos)
{
  u32 value = (u32)(
         ((bytestream[start_pos+3] & 0xFF) << 24)
      |  ((bytestream[start_pos+2] & 0xFF) << 16)
      |  ((bytestream[start_pos+1] & 0xFF) << 8)
      |   (bytestream[start_pos+0]));
  return value;
}

/***********************************************/

void u32_to_le_bytes(u32 value,
                     vec_u8 &bytestream,
                     u8 start_pos) {
  bytestream[start_pos + 3] = (value >> 24) & 0xFF;
  bytestream[start_pos + 2] = (value >> 16) & 0xFF;
  bytestream[start_pos + 1] = (value >> 8) & 0xFF;
  bytestream[start_pos] = value & 0xFF;
}

/***********************************************/

void encode_uint32(u32 value, 
                   vec_u8& bytestream, 
                   u32 start_pos)
{
  bytestream[start_pos + 0] = (value >> 24) & 0xFF;
  bytestream[start_pos + 1] = (value >> 16) & 0xFF;
  bytestream[start_pos + 2] = (value >> 8) & 0xFF;
  bytestream[start_pos + 3] = value & 0xFF;
}

/***********************************************/

void encode_uint32_rev(u32 value,
                       vec_u8& bytestream,
                       u32 start_pos)
{

  bytestream[start_pos + 1] = (value >> 24) & 0xFF;
  bytestream[start_pos + 0] = (value >> 16) & 0xFF;
  bytestream[start_pos + 3] = (value >> 8) & 0xFF;
  bytestream[start_pos + 2] = value & 0xFF;
  //bytestream[start_pos + 3] = (value >> 24) & 0xFF;
  //bytestream[start_pos + 2] = (value >> 16) & 0xFF;
  //bytestream[start_pos + 1] = (value >> 8) & 0xFF;
  //bytestream[start_pos + 0] = value & 0xFF;
}

/***********************************************/

u64 u64_from_le_bytes(const vec_u8 &bytestream,
                      usize start_pos) {
  u64 buffer64 = 0x0000000000000000;

  //unsigned long long value = (unsigned long long)(
  uint64_t buffer =  
         (((bytestream[start_pos+7] & 0xFF) | buffer64) << 56)
      |  (((bytestream[start_pos+6] & 0xFF) | buffer64) << 48)
      |  (((bytestream[start_pos+5] & 0xFF) | buffer64) << 40)
      |  (((bytestream[start_pos+4] & 0xFF) | buffer64) << 32)
      |  (((bytestream[start_pos+3] & 0xFF) | buffer64) << 24)
      |  (((bytestream[start_pos+2] & 0xFF) | buffer64) << 16)
      |  (((bytestream[start_pos+1] & 0xFF) | buffer64) << 8)
      |    (bytestream[start_pos+0]);

  return buffer;
}

uint64_t decode_uint64(const vec_u8& bytestream,
                       u32 start_pos)
{
  uint64_t buffer64 = 0x0000000000000000;

  //unsigned long long value = (unsigned long long)(
  uint64_t buffer =  
         (((bytestream[start_pos+0] & 0xFF) | buffer64) << 56)
      |  (((bytestream[start_pos+1] & 0xFF) | buffer64) << 48)
      |  (((bytestream[start_pos+2] & 0xFF) | buffer64) << 40)
      |  (((bytestream[start_pos+3] & 0xFF) | buffer64) << 32)
      |  (((bytestream[start_pos+4] & 0xFF) | buffer64) << 24)
      |  (((bytestream[start_pos+5] & 0xFF) | buffer64) << 16)
      |  (((bytestream[start_pos+6] & 0xFF) | buffer64) << 8)
      |  (bytestream[start_pos+7]);

  return buffer;
}

/***********************************************/

uint64_t decode_uint64_rev(const vec_u8& bytestream,
                           u32 start_pos)
{
  uint64_t buffer64 = 0x0000000000000000;

  //unsigned long long value = (unsigned long long)(
  uint64_t buffer =  
         ((bytestream[start_pos+1] & 0xFF | buffer64) << 56)
      |  ((bytestream[start_pos+0] & 0xFF | buffer64) << 48)
      |  ((bytestream[start_pos+3] & 0xFF | buffer64) << 40)
      |  ((bytestream[start_pos+2] & 0xFF | buffer64) << 32)
      |  ((bytestream[start_pos+5] & 0xFF | buffer64) << 24)
      |  ((bytestream[start_pos+4] & 0xFF | buffer64) << 16)
      |  ((bytestream[start_pos+7] & 0xFF | buffer64) << 8)
      |  (bytestream[start_pos+6]);

  return buffer;
  
}

/***********************************************/

void u64_to_le_bytes(u64 value,
                     vec_u8 &bytestream,
                     u64 start_pos) {
  bytestream[start_pos + 7] = (value >> 56) & 0xFF;
  bytestream[start_pos + 6] = (value >> 48) & 0xFF;
  bytestream[start_pos + 5] = (value >> 40) & 0xFF;
  bytestream[start_pos + 4] = (value >> 32) & 0xFF;
  bytestream[start_pos + 3] = (value >> 24) & 0xFF;
  bytestream[start_pos + 2] = (value >> 16) & 0xFF;
  bytestream[start_pos + 1] = (value >> 8) & 0xFF;
  bytestream[start_pos + 0] = value & 0xFF; 
}

/***********************************************/

void encode_uint64(uint64_t value, 
                   vec_u8& bytestream, 
                   u32 start_pos)
{
  bytestream[start_pos + 0] = (value >> 56) & 0xFF;
  bytestream[start_pos + 1] = (value >> 48) & 0xFF;
  bytestream[start_pos + 2] = (value >> 40) & 0xFF;
  bytestream[start_pos + 3] = (value >> 32) & 0xFF;
  bytestream[start_pos + 4] = (value >> 24) & 0xFF;
  bytestream[start_pos + 5] = (value >> 16) & 0xFF;
  bytestream[start_pos + 6] = (value >> 8) & 0xFF;
  bytestream[start_pos + 7] = value & 0xFF;
}

/***********************************************/

void encode_uint64_rev(uint64_t value,
                       vec_u8& bytestream,
                       u32 start_pos)
{
  bytestream[start_pos + 1] = (value >> 56) & 0xFF;
  bytestream[start_pos + 0] = (value >> 48) & 0xFF;
  bytestream[start_pos + 3] = (value >> 40) & 0xFF;
  bytestream[start_pos + 2] = (value >> 32) & 0xFF;
  bytestream[start_pos + 5] = (value >> 24) & 0xFF;
  bytestream[start_pos + 4] = (value >> 16) & 0xFF;
  bytestream[start_pos + 7] = (value >> 8) & 0xFF;
  bytestream[start_pos + 6] = value & 0xFF;
  //bytestream[start_pos + 7] = (value >> 56) & 0xFF;
  //bytestream[start_pos + 6] = (value >> 48) & 0xFF;
  //bytestream[start_pos + 5] = (value >> 40) & 0xFF;
  //bytestream[start_pos + 4] = (value >> 32) & 0xFF;
  //bytestream[start_pos + 3] = (value >> 24) & 0xFF;
  //bytestream[start_pos + 2] = (value >> 16) & 0xFF;
  //bytestream[start_pos + 1] = (value >> 8) & 0xFF;
  //bytestream[start_pos + 0] = value & 0xFF;
}

/***********************************************/

void encode_timestamp(uint64_t value,
                       vec_u8& bytestream,
                       u32 start_pos)
{
  //bytestream[start_pos + 1] = (value >> 56) & 0xFF;
  //bytestream[start_pos + 0] = (value >> 48) & 0xFF;
  bytestream[start_pos + 1] = (value >> 40) & 0xFF;
  bytestream[start_pos + 0] = (value >> 32) & 0xFF;
  bytestream[start_pos + 3] = (value >> 24) & 0xFF;
  bytestream[start_pos + 2] = (value >> 16) & 0xFF;
  bytestream[start_pos + 5] = (value >> 8) & 0xFF;
  bytestream[start_pos + 4] = value & 0xFF;
}

/***********************************************/

uint64_t decode_timestamp(const vec_u8& bytestream,
                          u32 start_pos)
{
  uint64_t buffer64 = 0x0000000000000000;

  //unsigned long long value = (unsigned long long)(
  uint64_t buffer =  
         (((bytestream[start_pos+1] & 0xFF) | buffer64) << 40)
      |  (((bytestream[start_pos+0] & 0xFF) | buffer64) << 32)
      |  (((bytestream[start_pos+3] & 0xFF) | buffer64) << 24)
      |  (((bytestream[start_pos+2] & 0xFF) | buffer64) << 16)
      |  (((bytestream[start_pos+5] & 0xFF) | buffer64) << 8)
      |  (((bytestream[start_pos+4] & 0xFF) | buffer64));

  return buffer;
  
}

/***********************************************/

void encode_48(uint64_t value, 
               vec_u8& bytestream, 
               u32 start_pos)
{
  bytestream[start_pos + 0] = (value >> 40) & 0xFF;
  bytestream[start_pos + 1] = (value >> 32) & 0xFF;
  bytestream[start_pos + 2] = (value >> 24) & 0xFF;
  bytestream[start_pos + 3] = (value >> 16) & 0xFF;
  bytestream[start_pos + 4] = (value >> 8) & 0xFF;
  bytestream[start_pos + 5] = value & 0xFF;
}

/***********************************************/

void encode_48_rev(uint64_t value, 
                   vec_u8& bytestream,
                   u32 start_pos)
{
  bytestream[start_pos + 5] = (value >> 40) & 0xFF;
  bytestream[start_pos + 4] = (value >> 32) & 0xFF;
  bytestream[start_pos + 3] = (value >> 24) & 0xFF;
  bytestream[start_pos + 2] = (value >> 16) & 0xFF;
  bytestream[start_pos + 1] = (value >> 8) & 0xFF;
  bytestream[start_pos + 0] = value & 0xFF;
}

/***********************************************/

uint16_t encode_12bitsensor(float value, float minrange, float maxrange)
{

   float width = maxrange - minrange;
   float increment = width/4096.0; //12bit
   // encoded value is always positive,
   // offset information gets added when decoding
   uint16_t encoded = (uint16_t)(value/increment);
   return encoded;
}

/***********************************************/

float decode_12bitsensor(uint16_t value, float minrange, float maxrange)
{
   float increment = (maxrange - minrange)/4096.0; //12bit
   float decoded = minrange + ((float)value*increment);
   return decoded;
}

/***********************************************/
//
//void convert_envdata_to_packet(RBEnvData* env_data, RBEnvPacket* env_packet)
//{
//  env_packet->temperature    = encode_12bitsensor(env_data->temperature   ,0,100);
//  env_packet->voltage        = encode_12bitsensor(env_data->voltage       ,0,100);
//  env_packet->current        = encode_12bitsensor(env_data->current       ,0,100);
//  env_packet->power          = encode_12bitsensor(env_data->power         ,0,100);
//  env_packet->preamp_temp    = encode_12bitsensor(env_data->preamp_temp   ,0,100);
//  env_packet->preamp_bias    = encode_12bitsensor(env_data->preamp_bias   ,0,100);
//  env_packet->temperature_rb = encode_12bitsensor(env_data->temperature_rb,0,100);
//  env_packet->voltage_rb     = encode_12bitsensor(env_data->voltage_rb    ,0,100);
//  env_packet->current_rb     = encode_12bitsensor(env_data->current_rb    ,0,100);
//  env_packet->power_rb       = encode_12bitsensor(env_data->power_rb      ,0,100);
//  env_packet->lol_status     = encode_12bitsensor(env_data->lol_status    ,0,100);
//}
//
///***********************************************/
//
//void convert_envpacket_to_data(RBEnvPacket* env_packet, RBEnvData* env_data)
//{
//  env_data->temperature    = decode_12bitsensor(env_packet->temperature   ,0,100);
//  env_data->voltage        = decode_12bitsensor(env_packet->voltage       ,0,100);
//  env_data->current        = decode_12bitsensor(env_packet->current       ,0,100);
//  env_data->power          = decode_12bitsensor(env_packet->power         ,0,100);
//  env_data->preamp_temp    = decode_12bitsensor(env_packet->preamp_temp   ,0,100);
//  env_data->preamp_bias    = decode_12bitsensor(env_packet->preamp_bias   ,0,100);
//  env_data->temperature_rb = decode_12bitsensor(env_packet->temperature_rb,0,100);
//  env_data->voltage_rb     = decode_12bitsensor(env_packet->voltage_rb    ,0,100);
//  env_data->current_rb     = decode_12bitsensor(env_packet->current_rb    ,0,100);
//  env_data->power_rb       = decode_12bitsensor(env_packet->power_rb      ,0,100);
//  env_data->lol_status     = decode_12bitsensor(env_packet->lol_status    ,0,100);
//
//
//}
//
///***********************************************/
//

void encode_blobevent(const BlobEvt_t* evt, std::vector<uint8_t> &bytestream, u32 start_pos)
{
  u32 enc_pos = start_pos;
  encode_ushort_rev(evt->head,        bytestream, enc_pos); enc_pos += 2;
  encode_ushort_rev(evt->status,      bytestream, enc_pos); enc_pos += 2;
  encode_ushort_rev(evt->len,         bytestream, enc_pos); enc_pos += 2;
  encode_ushort_rev(evt->roi,         bytestream, enc_pos); enc_pos += 2;
  encode_uint64_rev(evt->dna,         bytestream, enc_pos); enc_pos += 8;
  encode_ushort_rev(evt->fw_hash,     bytestream, enc_pos); enc_pos += 2;
  encode_ushort_rev(evt->id,          bytestream, enc_pos); enc_pos += 2;
  encode_ushort_rev(evt->ch_mask,     bytestream, enc_pos); enc_pos += 2;
  encode_uint32_rev(evt->event_ctr,   bytestream, enc_pos); enc_pos += 4;
  encode_ushort_rev(evt->dtap0,       bytestream, enc_pos); enc_pos += 2;
  encode_ushort_rev(evt->dtap1,       bytestream, enc_pos); enc_pos += 2;
  encode_timestamp (evt->timestamp,   bytestream, enc_pos); enc_pos += 6;
  
  //encode_uint64(evt->timestamp, buffer, enc_pos); enc_pos += 8;
  // NOW WE READ IN THE ADC DATA
  for (int i=0; i<NCHN; i++) {
    encode_ushort_rev(evt->ch_head[i],     bytestream, enc_pos); enc_pos += 2;
    // Read the channel data
    for (int j=0; j<NWORDS; j++) {
      encode_short_rev(evt->ch_adc[i][j], bytestream, enc_pos); enc_pos += 2;
    }
    encode_uint32_rev(evt->ch_trail[i],    bytestream, enc_pos); enc_pos += 4;
  }    
  
  // FIXME - the data has bad event stati
  //event_[k].tail = 21845;
  encode_ushort_rev(evt->stop_cell,        bytestream, enc_pos); enc_pos += 2;
  encode_uint32_rev(evt->crc32,            bytestream, enc_pos); enc_pos += 4;
  encode_ushort_rev(evt->tail,             bytestream, enc_pos); enc_pos += 2;
}

/***********************************************/

BlobEvt_t decode_blobevent(const std::vector<uint8_t> &bytestream,
                           u32 start_pos)
{
  BlobEvt_t event;
  u32 dec_pos     = start_pos;
  event.head      = decode_ushort_rev( bytestream, 0); dec_pos += 2;
  event.status    = decode_ushort_rev( bytestream, dec_pos); dec_pos += 2;
  event.len       = decode_ushort_rev( bytestream, dec_pos); dec_pos += 2;
  event.roi       = decode_ushort_rev( bytestream, dec_pos); dec_pos += 2;
  event.dna       = decode_uint64_rev( bytestream, dec_pos); dec_pos += 8;
  event.fw_hash   = decode_ushort_rev( bytestream, dec_pos); dec_pos += 2;
  event.id        = decode_ushort_rev( bytestream, dec_pos); dec_pos += 2;
  event.ch_mask   = decode_ushort_rev( bytestream, dec_pos); dec_pos += 2;
  event.event_ctr = decode_uint32_rev( bytestream, dec_pos); dec_pos += 4;
  event.dtap0     = decode_ushort_rev( bytestream, dec_pos); dec_pos += 2;
  event.dtap1     = decode_ushort_rev( bytestream, dec_pos); dec_pos += 2;
  event.timestamp = decode_timestamp ( bytestream, dec_pos); dec_pos += 6;
  for (int i=0; i<NCHN; i++) {
    event.ch_head[i] = decode_ushort_rev(bytestream, dec_pos); dec_pos += 2;
    // Read the channel data
    for (int j=0; j<NWORDS; j++) {
      //event.ch_adc[i][j] = decode_14bit(bytestream, dec_pos); dec_pos += 2;
      event.ch_adc[i][j] = decode_short_rev(bytestream, dec_pos) & 0x3FFF; dec_pos += 2;
    }
    event.ch_trail[i] = decode_uint32_rev(bytestream, dec_pos); dec_pos += 4;
  }    
  
  // FIXME - the data has bad event stati
  //event_[k].tail = 21845;
  event.stop_cell = decode_ushort_rev(bytestream, dec_pos); dec_pos += 2;
  event.crc32 = decode_uint32_rev(bytestream, dec_pos); dec_pos += 4;
  event.tail  = decode_ushort_rev(bytestream, dec_pos); dec_pos += 2;
  return event;
} 

/***********************************************/

u64 search_for_2byte_marker(const vec_u8 &bytestream,
                            u8 marker,
                            bool &has_ended,
                            u64 start_pos,
                            u64 end_pos)
{
  has_ended = false;
  if ((end_pos == bytestream.size()) || (end_pos == 0)) 
    { end_pos = bytestream.size() - 1;} 
  for (u64 k=start_pos; k<end_pos; k++)
    { 
      if ((bytestream[k] == marker) && (bytestream[k+1] == marker)) 
        { return k;}
    }
  has_ended = true;
  return 0;
}

/***********************************************/

std::vector<u32> get_2byte_markers_indices(const std::vector<uint8_t> &bytestream, uint8_t marker)
{
  std::vector<u32> indices;
  for (size_t k=0; k<bytestream.size() -1; k++)
    { 
      if ((bytestream[k] == marker) && (bytestream[k+1] == marker)) 
        { indices.push_back(k);}
    }
  return indices;
   
}

/***********************************************/

std::vector<BlobEvt_t> get_events_from_stream(const vec_u8 &bytestream,
	       			                 	      u64 start_pos) {
  u64 nevents_in_stream = (float)bytestream.size()/BLOBEVENTSIZE;
  std::cout << "[INFO] There might be at max " << nevents_in_stream<< " events in the stream" << std::endl;
  std::vector<BlobEvt_t> events; 
  BlobEvt_t event;

  bool finished = false;
  u64 head_index, tail_index;
  usize current_index = 0;
  i64 event_size; // can be negative if things go wrong

  size_t n_events_found = 0;
  u32 n_iter_debug = 0;
  u32 n_iter_stuck_debug = 0;
  bool has_ended = false;
  //unsigned long head_start = 0;
  uint nheaders = 0;
  uint ntails   = 0;
  usize pos = 0;
  usize nblobs = 0;
  usize ncorrupt_blobs = 0;
  bool header_found_start= false;
  while (true) { 
    // FIXME - this needs care. If there is only one event in the stream
    // this can't fail. To bypass this, we omit this if a header has been 
    // found. Not sure if that is good.
    if ((pos + BLOBEVENTSIZE > bytestream.size()) && !(header_found_start)) {
      break;
    }
    auto byte = bytestream[pos];
    if (!header_found_start) {
      if (byte == 0xaa) {
        header_found_start = true;
      }  
      pos++;
      continue;
    }   
    if (header_found_start) {
      pos++;
      if (byte == 0xaa) {
        header_found_start = false;
        event = decode_blobevent(bytestream,
                                 pos -2);
        nblobs++;
	    //std::cout << "NBLOBS" << nblobs << std::endl;
        //std::cout << event.head << std::endl;
        //std::cout << event.event_ctr << std::endl;
        //std::cout << event.timestamp << std::endl;
        //std::cout << event.stop_cell << std::endl;
        //std::cout << event.crc32 << std::endl;
        //std::cout << event.tail << std::endl;
        //std::cout << NCHN << std::endl;
        if (event.tail == 0x5555) {
            pos += BLOBEVENTSIZE - 2;  
            events.push_back(event);
        } else {
            // the event is corrupt
            //println!("{}", blob_data.head);
            ncorrupt_blobs += 1;
        }
      } else {
          // it wasn't an actual header
          header_found_start = false;
      }   
    }   
  }// end loop
  std::cout << "==> Deserialized " <<  nblobs << " blobs! " << ncorrupt_blobs << " blobs were corrupt" << std::endl;
  


  //while (!has_ended)
  //  { n_iter_debug++;
  //    //std::cout << "current_index " << current_index << std::endl;
  //    //if (current_index > 20) {
  //    //    head_start -= 20;
  //    //} else {
  //    //    head_start = current_index;
  //    //}
  //    head_index = search_for_2byte_marker(bytestream, 0xaa,
  //                                         has_ended,
  //                                         current_index,
  //                                         bytestream.size());
  //    // there is 94 padding bytes
  //    if ( has_ended) break;
  //    tail_index = search_for_2byte_marker(bytestream, 0x55,
  //                                         has_ended,
  //                                         head_index,
  //                                         bytestream.size()  );
  //    // the tail might still be somewhere in the waveform
  //    // let's check if the event is too small if we can find a better one
  //    event_size = tail_index - head_index + 2;
  //    std::cout << "event size " << event_size << std::endl;
  //    current_index = tail_index + 2;
  //    while (event_size < BLOBEVENTSIZE)
  //    { 
  //      tail_index = search_for_2byte_marker(bytestream, 0x55,
  //                                           has_ended,
  //                                           current_index,
  //                                           bytestream.size()  );
  //      event_size = tail_index - head_index + 2;
  //      current_index++;
  //      std::cout << current_index << " " << event_size << std::endl;
  //      if (event_size == BLOBEVENTSIZE) break; 
  //      if (has_ended) break;
  //    }

  //    std::cout << head_index << " " << tail_index << std::endl;
  //    if (has_ended) break;
  //    //if (n_iter_debug > 10 ) break;
  //    
  //    // this includes negative event sizes
  //    if (event_size != BLOBEVENTSIZE)
  //      {
  //        if (event_size > BLOBEVENTSIZE) {std::cout << "event is "  << event_size - BLOBEVENTSIZE << " bytes too big "  << std::endl;}
  //        else {std::cout << " event is " << BLOBEVENTSIZE - event_size << " bytes too small " << std::endl;}
  //        current_index++;
  //        n_iter_stuck_debug += 1;
  //        continue;
  //      }
  //    //head_index = current_index + head_index;
  //    //tail_index = current_index + tail_index;
  //    events[n_events_found] = decode_blobevent(bytestream,
  //                                              head_index,
  //                                              tail_index);
  //    //events.push_back(decode_blobevent(bytestream, head_index, tail_index));
  //    //current_index += BLOBEVENTSIZE;
  //    current_index = tail_index;
  //    n_events_found++;
  //    //std::cout << "-----------" << std::endl;
  //  }
  //if (events.size() > n_events_found)
  //   {events.resize(n_events_found);}
  //events.shrink_to_fit();
  std::cout << "We ran through " << n_iter_debug << " iterations" << std::endl;
  std::cout << "We ran through " << n_iter_stuck_debug << " continues" << std::endl;
  return events;
}

