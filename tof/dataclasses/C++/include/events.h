#ifndef TOFEVENTS_H_INCLUDED
#define TOFEVENTS_H_INCLUDED

#include "tof_typedefs.h"
#include "packets/monitoring.h"

struct RBEventHeader;
struct RBEvent;

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
Vec<RBEventHeader> get_headers(const String &filename, bool is_header=false);

//Vec<RBBinaryDump> get_level0_events(const String &filename);


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
 
  static RBEventHeader from_bytestream(const Vec<u8> &bytestream,
                                       u64 &pos);

  /**
   * Take a "regular" ("blob") data stream from the RB and 
   * process only the header part.
   *
   */
  static RBEventHeader extract_from_rbbinarydump(const Vec<u8> &bytestream,
                                                 u64 &pos);
  Vec<u8> get_active_data_channels() const;
  u64 get_clock_cycles_48bit() const;
  u8  get_n_datachan() const;
  f32 get_fpga_temp() const;
  f32 get_drs_temp() const;

  private:
    f32 drs_adc_to_celsius(u16 adc) const; 
};

/**
 * A complete event for a single readout board 
 * with header and channel data
 *
 *
 */ 
struct RBEvent {
  static const u16 HEAD = 0xAAAA;
  static const u16 TAIL = 0x5555;
  
  RBEventHeader header;
  Vec<Vec<u16>> adc; 
  
  static RBEvent from_bytestream(const Vec<u8> &bytestream,
                                 u64 &pos);
};

struct RBMissingHit {
  
  static const u16 HEAD = 0xAAAA;
  static const u16 TAIL = 0x5555;
  static const usize SIZE = 15; // bytes
  
  u32 event_id     ;  
  u8  ltb_hit_index;  
  u8  ltb_id       ;  
  u8  ltb_dsi      ;  
  u8  ltb_j        ;  
  u8  ltb_ch       ;  
  u8  rb_id        ;  
  u8  rb_ch        ;  

  static RBMissingHit from_bytestream(const Vec<u8> &bytestream,
                                      u64 &pos);
};

static const u8 EVENT_QUALITY_UNKNOWN         =  0;
static const u8 EVENT_QUALITY_SILVER          =  10;
static const u8 EVENT_QUALITY_GOLD            =  20;
static const u8 EVENT_QUALITY_DIAMOND         =  30;
static const u8 EVENT_QUALITY_FOURLEAFCLOVER  =  40;

enum class EventQuality : u8 {
  Unknown        = EVENT_QUALITY_UNKNOWN,
  Silver         = EVENT_QUALITY_SILVER,
  Gold           = EVENT_QUALITY_GOLD,
  Diamond        = EVENT_QUALITY_DIAMOND,
  FourLeafClover = EVENT_QUALITY_FOURLEAFCLOVER
};

std::ostream& operator<<(std::ostream& os, const EventQuality& qual);

static const u8 COMPRESSION_LEVEL_UNKNOWN         =  0;
static const u8 COMPRESSION_LEVEL_NONE            =  10;

enum class CompressionLevel : u8 {
  Unknown        = COMPRESSION_LEVEL_UNKNOWN,
  None           = COMPRESSION_LEVEL_NONE,
};

std::ostream& operator<<(std::ostream& os, const CompressionLevel& level);

struct TofEvent {
  static const u16 HEAD = 0xAAAA;
  static const u16 TAIL = 0x5555;

  //MasterTriggerEvent
  //Vec<RBEvent>      rb_events;
  Vec<RBMissingHit> missing_hits;
  Vec<RBMoniData>   rb_moni_data;

  static TofEvent from_bytestream(const Vec<u8> &bytestream,
                                  u64 &pos);
};


#endif 
