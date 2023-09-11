#include <fstream>
#include "spdlog/spdlog.h"



#include "TOFCommon.h"
#include "tof_typedefs.h"
#include "parsers.h"

#include "serialization.h"

u16 decode_ushort(const Vec<u8>& bytestream,
                  u32 start_pos) {
  u16 value= (u16)(((bytestream[start_pos+0] & 0xFF) << 8) | bytestream[start_pos+1]);
  return value;
}

/***********************************************/

short decode_short(const Vec<u8>& bytestream,
                   u32 start_pos)
{
  //short value = (short)(((bytestream[start_pos+0] & 0xFF) << 8) | bytestream[start_pos+1]);
  short value = (short)(((bytestream[start_pos+0]) << 8) | bytestream[start_pos+1]);
  
  return value;
}

/***********************************************/

short decode_short_rev(const Vec<u8>& bytestream,
                   u32 start_pos)
{
  short value= (short)(((bytestream[start_pos+1] & 0xFF) << 8) | bytestream[start_pos+0]);
  return value;
}

/***********************************************/

u16 decode_ushort_rev(const Vec<u8>& bytestream,
                             u32 start_pos)
{
  u16 value= (u16)(((bytestream[start_pos+1] & 0xFF) << 8) | bytestream[start_pos+0]);
  return value;
}

/***********************************************/

void encode_ushort(u16 value,
                   Vec<u8>& bytestream,
                   u32 start_pos)
{
  //Vec<u8> buffer(2);
  bytestream[start_pos + 0] = (value >> 8) & 0xFF;
  bytestream[start_pos + 1] = value & 0xFF;
}

/***********************************************/

int16_t decode_14bit(const Vec<u8>& bytestream,
                     u32 start_pos)
{
   int16_t value =  decode_short_rev(bytestream, start_pos);
   return  value & 0x3FFF;

}

/***********************************************/

void encode_ushort_rev(u16 value,
                   Vec<u8>& bytestream,
                   u32 start_pos)
{
  //Vec<u8> buffer(2);
  bytestream[start_pos + 1] = (value >> 8) & 0xFF;
  bytestream[start_pos + 0] = value & 0xFF;
}

/***********************************************/

void encode_short_rev(short value,
                      Vec<u8>& bytestream,
                      u32 start_pos)
{
  //Vec<u8> buffer(2);
  bytestream[start_pos + 1] = (value >> 8) & 0xFF;
  bytestream[start_pos + 0] = value & 0xFF;
}

/***********************************************/

u32 decode_uint32(const Vec<u8>& bytestream,
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

u32 decode_uint32_rev(const Vec<u8> &bytestream,
                           u32 start_pos)
{
  u32 value = (u32)(
         ((bytestream[start_pos+1] & 0xFF) << 24)
      |  ((bytestream[start_pos+0] & 0xFF) << 16)
      |  ((bytestream[start_pos+3] & 0xFF) << 8)
      |  ( bytestream[start_pos+2]));
  return value;
}

/***********************************************/

u32 u32_from_le_bytes(const Vec<u8> &bytestream,
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
                     Vec<u8> &bytestream,
                     u8 start_pos) {
  bytestream[start_pos + 3] = (value >> 24) & 0xFF;
  bytestream[start_pos + 2] = (value >> 16) & 0xFF;
  bytestream[start_pos + 1] = (value >> 8) & 0xFF;
  bytestream[start_pos] = value & 0xFF;
}

/***********************************************/

void encode_uint32(u32 value, 
                   Vec<u8>& bytestream, 
                   u32 start_pos)
{
  bytestream[start_pos + 0] = (value >> 24) & 0xFF;
  bytestream[start_pos + 1] = (value >> 16) & 0xFF;
  bytestream[start_pos + 2] = (value >> 8) & 0xFF;
  bytestream[start_pos + 3] = value & 0xFF;
}

/***********************************************/

void encode_uint32_rev(u32 value,
                       Vec<u8>& bytestream,
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

u64 u64_from_le_bytes(const Vec<u8> &bytestream,
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

uint64_t decode_uint64(const Vec<u8>& bytestream,
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

uint64_t decode_uint64_rev(const Vec<u8>& bytestream,
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
                     Vec<u8> &bytestream,
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
                   Vec<u8>& bytestream, 
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
                       Vec<u8>& bytestream,
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
                       Vec<u8>& bytestream,
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

uint64_t decode_timestamp(const Vec<u8>& bytestream,
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
               Vec<u8>& bytestream, 
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
                   Vec<u8>& bytestream,
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

// file i/o

bytestream get_bytestream_from_file(const String &filename) {
  // bytestream stream;
  // Not going to explicitly check these.
  // // The use of gcount() below will compensate for a failure here.
  std::ifstream is(filename, std::ios::binary);

  is.seekg (0, is.end);
  u64 length = is.tellg();
  is.seekg (0, is.beg);
  spdlog::debug("Read {} bytes from file!", length);
  bytestream stream = bytestream(length);
  is.read(reinterpret_cast<char*>(stream.data()), length);
  return stream;
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

void encode_blobevent(const BlobEvt_t* evt, Vec<u8> &bytestream, u32 start_pos)
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

BlobEvt_t decode_blobevent(const Vec<u8> &bytestream,
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
  // the first byte of the event id short is RESERVED
  event.id        = bytestream[dec_pos + 1]; dec_pos += 2;
  //event.id        = decode_ushort_rev( bytestream, dec_pos); dec_pos += 2;
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

u64 search_for_2byte_marker(const Vec<u8> &bytestream,
                            u8 marker,
                            bool &has_ended,
                            u64 start_pos,
                            u64 end_pos)
{
  has_ended = false;
  if ((end_pos == bytestream.size()) || (end_pos == 0)) 
    { end_pos = bytestream.size() - 1;} 
  if (start_pos >= end_pos) {
    has_ended = true;
    return 0;
  }
  for (u64 k=start_pos; k<end_pos; k++)
    { 
      if ((bytestream[k] == marker) && (bytestream[k+1] == marker))  {
        //std::cout << "endpos " << end_pos << std::endl;
        //std::cout << "Found marker at pos " << k << " " << bytestream[k] << std::endl;
        return k;
      }
    }
  has_ended = true;
  return 0;
}

/***********************************************/

Vec<u32> get_2byte_markers_indices(const Vec<u8> &bytestream, uint8_t marker)
{
  Vec<u32> indices;
  for (size_t k=0; k<bytestream.size() -1; k++)
    { 
      if ((bytestream[k] == marker) && (bytestream[k+1] == marker)) 
        { indices.push_back(k);}
    }
  return indices;
}

/***********************************************/


