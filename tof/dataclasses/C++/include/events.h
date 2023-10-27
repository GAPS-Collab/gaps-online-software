#ifndef TOFEVENTS_H_INCLUDED
#define TOFEVENTS_H_INCLUDED

/**
 * Tof event classes. An event is basically anything with an 
 * event id.
 * - events for individual readoutboards
 *   - RBEventMemoryView : representation of event in RB memory
 *   - RBEventHeader     : header information of event
 *   - RBEvent           : contains header + active channels
 *
 * - events for the MasterTriggerBoard
 * 
 *
 *
 */ 

#include "tof_typedefs.h"
#include "packets/monitoring.h"
#include "packets/tof_packet.h"
#include "events/tof_event_header.hpp"

#define NCHN 9
#define NWORDS 1024
#define N_LTBS 20
#define N_CHN_PER_LTB 16

struct RBEventHeader;
struct RBEvent;
struct RBEventMemoryView;
struct MasterTriggerEvent;


/**
 * The "purest" form of an event for a single RB. 
 * Formerly known as "blob". This represents the 
 * layout of the event for each readoutboard in 
 * its internal memory. 
 *
 *
 */ 
struct RBEventMemoryView {
  static const u16 HEAD = 0xAAAA;
  static const u16 TAIL = 0x5555;
  static const u16 SIZE = 18530; // size in bytes with HEAD and TAIL
  u16 head; // Head of event marker
  u16 status;
  u16 len;
  u16 roi;
  u64 dna;
  u16 fw_hash;
  u16 id;
  u16 ch_mask;
  u32 event_ctr;
  u16 dtap0;
  u16 dtap1;
  u64 timestamp;
  u16 ch_head[NCHN];
  u16 ch_adc[NCHN][NWORDS];
  u32 ch_trail[NCHN];
  u16 stop_cell;
  u32 crc32;
  u16 tail; // End of event marker

  RBEventMemoryView();
 
  static RBEventMemoryView from_bytestream(const Vec<u8> &bytestream,
                                           u64 &pos);


  /**
   * Return adc values for specific channel
   *
   * @param channel : Channel ID 1-9, channel 9 has calibration sinus data.
   */
  Vec<u16> get_channel_adc(u8 channel) const;

};

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
  bool event_fragment        ;
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

  std::string to_string() const;

  private:
    f32 drs_adc_to_celsius(u16 adc) const; 
};

/**
 * A complete event for a single readout board 
 * with header and channel data.
 * The size is flexible, only active datachannels
 * are recorded.
 *
 *
 */ 
struct RBEvent {
  static const u16 HEAD = 0xAAAA;
  static const u16 TAIL = 0x5555;

  // data type will be an enum
  u8 data_type;
  // number of channels in this event
  u8 nchan;
  // number of paddle packets in this event
  u8 npaddles; 
  RBEventHeader header;
  Vec<Vec<u16>> adc; 

  // FIXME - needs paddle packet extension
 
  RBEvent();

  const Vec<u16>& get_channel_adc(u8 channel) const; 

  static RBEvent from_bytestream(const Vec<u8> &bytestream,
                                 u64 &pos);

  std::string to_string() const;

  private:

    /**
     * Check if the channel follows the convention 1-9
     *
     */
    bool channel_check(u8 channel) const;
    Vec<u16> _empty_channel = Vec<u16>();
};

/**
 * RBMissingHit represents missing data from a readoutboard.
 * This can occur when the MTB has a hit registered, but for
 * some reason we do not get the corresponding data for the 
 * RB. RBMissingHit might help with debugging.
 */
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

/**
 * The MasterTriggerEvent represesnts the information
 * provided by the MTB for this one specific event.
 * Most notably, it includes a board mask,
 * which is the DSI/J connections which triggered, and 
 * a hit mask. The hit mask gives hit channels per DSI/J,
 * which correspond to hit channels on a LTB.
 *
 */
struct MasterTriggerEvent {
  static const u16 HEAD = 0xAAAA;
  static const u16 TAIL = 0x5555;
  static const usize SIZE = 45; // size in bytes

  u32 event_id      ; 
  u32 timestamp     ; 
  u32 tiu_timestamp ; 
  u32 tiu_gps_32    ; 
  u32 tiu_gps_16    ; 
  u8  n_paddles     ; 
  bool board_mask[N_LTBS];
  //ne 16 bit value per LTB
  bool hits[N_LTBS][N_CHN_PER_LTB];
  //hits          : [[false;N_CHN_PER_LTB]; N_LTBS],
  u32 crc           ;
  // these fields won't get serialized
  bool broken       ;
  bool valid        ;

  MasterTriggerEvent();

  // FIXME - this has to   
  static MasterTriggerEvent from_bytestream(const Vec<u8> &bytestream,
                                            u64 &pos);
  static void decode_board_mask(u32 mask_number, bool (&decoded_mask)[N_LTBS]); 

  static void decode_hit_mask(u32 mask_number, bool (&hitmask_1)[N_CHN_PER_LTB], bool (&hitmask_2)[N_CHN_PER_LTB]);

  void set_board_mask(u32 mask);
  
  void set_hit_mask(usize ltb_index,u32 mask);

  std::string to_string() const;
};


/**
 * A container accounting for a "complete" event of the Tof
 * including:
 * - A MasterTriggerEvent
 * - Possible monitoring data for readoutboards
 * - A number of Readoutboardevents (each with 
 *   header and the number of active channels) 
 * - A number of MissingHits. These are such 
 *   where the MTB claims we should see data 
 *   in one of the RBs, but we do not have 
 *   any.
 *
 */ 
struct TofEvent {
  static const u16 HEAD = 0xAAAA;
  static const u16 TAIL = 0x5555;

  TofEventHeader header;
  MasterTriggerEvent mt_event;
  
  Vec<RBEvent>      rb_events;
  Vec<RBMissingHit> missing_hits;

  static TofEvent from_bytestream(const Vec<u8> &bytestream,
                                  u64 &pos);

  static TofEvent from_tofpacket(const TofPacket &packet);

  static u32 get_n_rbmissinghits(u32 mask);
  static u32 get_n_rbevents(u32 mask);

  std::string to_string() const;

  /**
   * Get the rb event for a specific board id.
   */
  const RBEvent& get_rbevent(u8 board_id) const; 

  /**
   * Get the rb event for a specific board id.
   */
  Vec<u8> get_rbids() const;

  private:
    /**
     * Check if there are more than one RBEvent per board
     * and if the eventids are matching up.
     */
    bool passed_consistency_check();

    // an empty event, which can be returned 
    // in case of a null result.
    RBEvent _empty_event = RBEvent();
};

std::ostream& operator<<(std::ostream& os, const MasterTriggerEvent& mt);

std::ostream& operator<<(std::ostream& os, const TofEvent& et);

std::ostream& operator<<(std::ostream& os, const RBEvent& re);

std::ostream& operator<<(std::ostream& os, const RBEventHeader& rh);


#endif 
