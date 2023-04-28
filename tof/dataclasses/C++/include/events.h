#ifndef TOFEVENTS_H_INCLUDED
#define TOFEVENTS_H_INCLUDED

#include "tof_typedefs.h"

class RBEventHeader;

/**
 * Extract only event ids from a bytestream with raw readoutboard binary data
 *
 * @param bytestream : Readoutboard binary (.robin) data.
 * @param start_pos  : Byte position to start searching from in bytestream
 */
vec_u32 get_event_ids_from_raw_stream(const vec_u8 &bytestream, u64 &start_pos);

/**
 * Read event headers from a RB binary file
 *
 */
Vec<RBEventHeader> get_headers(const String &filename);

/**
 * RB binary data header information
 *
 * This does not include the channel data!
 *
 */ 
struct RBEventHeader {
  static const u16 HEAD = 0xAAAA;
  static const u16 TAIL = 0x5555;
  static const u16 SIZE = 30; // size in bytes with HEAD and TAIL

  u8   channel_mask          ;
  u16  stop_cell             ;
  u32  crc32                 ;
  u16  dtap0                 ;
  u16  drs4_temp             ;
  bool is_locked             ;
  bool is_locked_last_sec    ;
  bool lost_trigger          ;
  u16  fpga_temp             ;
  u32  event_id              ;
  u8   rb_id                 ;
  //u32  timestamp_32          ;
  //u16  timestamp_16          ;
  u64  timestamp_48          ;
  bool broken                ;
  
  
  RBEventHeader();
  
  /**
   * Take a "regular" ("blob") data stream from the RB and 
   * process only the header part.
   *
   */
  static RBEventHeader from_bytestream(const Vec<u8> bytestream,
                                       u64 &pos);
  Vec<u8> get_active_data_channels()const;
  //u64 get_timestamp_16_corrected();
  u64 get_clock_cycles_48bit() const;
  u8  get_n_datachan() const;
  f32 get_fpga_temp() const;
  f32 get_drs_temp() const;

  private:
    f32 drs_adc_to_celsius(u16 adc) const; 
};

#endif 
