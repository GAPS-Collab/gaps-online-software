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

#include <tuple>

#include "tof_typedefs.h"
#include "packets/monitoring.h"
#include "packets/tof_packet.h"
#include "events/tof_event_header.hpp"
#include "calibration.h"
#include "packets/RPaddlePacket.h"

class RBCalibration;

#define NCHN 9
#define NWORDS 1024
#define N_LTBS 20
#define N_CHN_PER_LTB 16

struct RBEventHeader;
struct RBEvent;
struct RBEventMemoryView;
struct MasterTriggerEvent;

/*********************************************************/
  
static const u8 EVENTSTATUS_UNKNOWN           =   0;
static const u8 EVENTSTATUS_CRC32WRONG        =  10;
static const u8 EVENTSTATUS_TAILWRONG         =  11;
static const u8 EVENTSTATUS_INCOMPLETEREADOUT =  21;
static const u8 EVENTSTATUS_PERFECT           =  42;

/**
 * The event status indicates if there are technical 
 * issues with the retrieval of the event.
 * If there are no problems, events should have status
 * EventStatus::EVENTSTATUS_PERFECT (42)
 */
enum class EventStatus : u8 {
  Unknown           = EVENTSTATUS_UNKNOWN,
  Crc32Wrong        = EVENTSTATUS_CRC32WRONG,
  TailWrong         = EVENTSTATUS_TAILWRONG,
  IncompleteReadout = EVENTSTATUS_INCOMPLETEREADOUT,
  Perfect           = EVENTSTATUS_PERFECT,
};

std::ostream& operator<<(std::ostream& os, const EventStatus& status);

/*********************************************************/

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
 
  /**
   * Factory function for RBEVentMemeoryViews. 
   *
   * Create an instance by de-serializing it from a bytestream
   *
   * @param bytestream : (Byte) representation of RBEventMemoryView
   * @param pos        : Index in bytestream where to look for 
   *                     RBEventMemoryView::HEAD
   */ 
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
 * The header contains rb id, event id,
 * event status and timestamps.
 */ 
struct RBEventHeader {
  static const u16 HEAD = 0xAAAA;
  static const u16 TAIL = 0x5555;
  static const u16 SIZE = 30; // size in bytes with HEAD and TAIL

  u8   rb_id                 ;
  u32  event_id              ;
  u8   status_byte           ;
  u16  channel_mask          ;
  u16  stop_cell             ;
  u32  crc32                 ;
  u16  dtap0                 ;
  u16  fpga_temp             ;
  u16  drs4_temp             ;
  u32  timestamp32           ;
  u16  timestamp16           ;
  
  RBEventHeader();
 
  static RBEventHeader from_bytestream(const Vec<u8> &bytestream,
                                       u64 &pos);

  Vec<u8> get_channels()    const;
  u8      get_nchan()       const;
  Vec<u8> get_active_data_channels() const;
  bool has_ch9()            const;
  u8   get_n_datachan()     const;
  f32  get_fpga_temp()      const;
  f32  get_drs_temp()       const;
  bool is_event_fragment()  const;
  bool drs_lost_trigger()   const;
  bool lost_lock()          const;
  bool lost_lock_last_sec() const;
  bool is_locked()          const;
  bool is_locked_last_sec() const;
  u64  get_timestamp48()    const;

  /// string representation for printing
  std::string to_string() const;

  private:
    /// conversion method for drs temperature readout
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
  EventStatus status;
  RBEventHeader header;
  Vec<Vec<u16>> adc; 
  Vec<RPaddlePacket> hits;
 
  RBEvent();

  const Vec<u16>& get_channel_by_label(u8 channel) const;
  const Vec<u16>& get_channel_by_id(u8 channel) const;

  const Vec<u16>& get_channel_adc(u8 channel) const; 
 
  Vec<f32> get_baselines(const RBCalibration &cali, usize min_bin, usize max_bin); 

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

/*********************************************************/

static const u8 EVENT_QUALITY_UNKNOWN         =  0;
static const u8 EVENT_QUALITY_SILVER          =  10;
static const u8 EVENT_QUALITY_GOLD            =  20;
static const u8 EVENT_QUALITY_DIAMOND         =  30;
static const u8 EVENT_QUALITY_FOURLEAFCLOVER  =  40;


/**
 * EventQuality will get assigned by online reconstructions
 * or the flight computer. This contains information about
 * physics and might pre-select "golden" candidate events.
 * The default event quelity is EventQuality::UNKNOWN
 */
enum class EventQuality : u8 {
  Unknown        = EVENT_QUALITY_UNKNOWN,
  Silver         = EVENT_QUALITY_SILVER,
  Gold           = EVENT_QUALITY_GOLD,
  Diamond        = EVENT_QUALITY_DIAMOND,
  /// FourLeavClover events are events with exactly
  /// 4 hits in overlapping pannels. 2 overlapping 
  /// in the Umbrella/Cortina, 2 overlapping in the 
  /// TOF cube
  FourLeafClover = EVENT_QUALITY_FOURLEAFCLOVER
};

std::ostream& operator<<(std::ostream& os, const EventQuality& qual);

/*********************************************************/

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
  /// begin struct marker
  static const u16 HEAD = 0xAAAA;
  /// end struct marker
  static const u16 TAIL = 0x5555;
  /// the struct has a fixed size of SIZE
  static const usize SIZE = 45; // size in bytes

  /// event_id as assigned by the MasterTriggerBoard
  u32 event_id      ; 
  /// MTB timestamp
  u32 timestamp     ; 
  /// Tracker (?) timestamp
  u32 tiu_timestamp ; 
  /// GAPS GPS clock value (slow)
  u32 tiu_gps_32    ; 
  /// GAPS GPS clock value (fast)
  u32 tiu_gps_16    ; 
  /// triggered paddles as seen by the MTB
  u8  n_paddles     ; 
  /// bitmask indicating hit LTBs, identified by DSI/J
  bool board_mask[N_LTBS];
  /// bitmask per LTB to indicate hit channels. Each 
  /// channel maps to a RBID/RBCH
  bool hits[N_LTBS][N_CHN_PER_LTB];
  u32 crc           ;
  // these fields won't get serialized
  bool broken       ;
  bool valid        ;

  MasterTriggerEvent();

  /**
   * Factory function for MasterTriggerEvent
   *
   * Deserialize a MasterTriggerEvent from a vector of of bytes
   *
   * @param bytestream: Byte representation of a MasterTriggerEvent, or 
   *                    including one at pos
   * @param pos       : Expected position of MasterTriggerEvent::HEAD in 
   *                    the stream
   *
   */
  static MasterTriggerEvent from_bytestream(const Vec<u8> &bytestream,
                                            u64 &pos);
  static void decode_board_mask(u32 mask_number, bool (&decoded_mask)[N_LTBS]); 

  static void decode_hit_mask(u32 mask_number, bool (&hitmask_1)[N_CHN_PER_LTB], bool (&hitmask_2)[N_CHN_PER_LTB]);

  void set_board_mask(u32 mask);
  
  void set_hit_mask(usize ltb_index,u32 mask);

  /**
   * Get the hits in terms of DSI/J/CH
   * This can then be further used to 
   * calculate RB ID/CHANNEL
   */
  Vec<std::tuple<u8,u8,u8>>  get_dsi_j_ch();

  /// String representation of the struct
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

  /// A container holding the individual events from all RBs with 
  /// triggers in this event  
  Vec<RBEvent>      rb_events;
  /// A container holding information about missing rbevents. That 
  /// is events where we know the board triggered, but we did not
  /// get an associated RBEvent within a timeout
  Vec<RBMissingHit> missing_hits;


  /**
   * Factory function for TofEvents.
   *
   * Deserialize a TofEvetn from a vector of of bytes
   *
   * @param bytestream: Byte representation of a TofEvent, or 
   *                    including such a representation at pos
   * @param pos       : Expected position of TofEvent::HEAD in 
   *                    the stream
   *
   */
  static TofEvent from_bytestream(const Vec<u8> &bytestream,
                                  u64 &pos);

  /**
   * Factory function for TofEvents.
   *
   * Unpack the TofPacket, return an 
   * empty event in case the packet 
   * is not of PacketType::TofPacket
   *
   * @param packet: TofPacket with 
   *                PacketType::TofPacket 
   *                
   */
  static TofEvent from_tofpacket(const TofPacket &packet);

  static u32 get_n_rbmissinghits(u32 mask);
  static u32 get_n_rbevents(u32 mask);

  /// string representation for printing
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

    /// an empty event, which can be returned 
    /// in case of a null result.
    RBEvent _empty_event = RBEvent();
};

/***********************************************
 * Reconstructed waveform peak information
 * 
 * There should be one TofHit per reconstructed
 * peak
 * 
 *
 */
struct TofHit  {
  static const u16 HEAD = 0xF0F0;
  static const u16 TAIL = 0xF0F;

  u8   paddle_id;
  bool broken;

  u32 timestamp32;
  u16 timestamp16;

  u8 ctr_etx;
  u16 tail = 0xF0F; 

  f32 get_time_a()       const;
  f32 get_time_b()       const;
  f32 get_peak_a()       const;
  f32 get_peak_b()       const;
  f32 get_charge_a()     const;
  f32 get_charge_b()     const;
  f32 get_charge_min_i() const;
  f32 get_x_pos()        const;
  f32 get_t_avg()        const;
  f64 get_timestamp48()  const;

  static TofHit from_bytestream(const Vec<u8> &bytestream, 
                                       u64 &pos);
 
  // easier print out
  std::string to_string() const;
  
  private:
    // we keep this private, since 
    // the user should use the getters
    // to get the values converted 
    // back to f32
    u16 time_a;
    u16 time_b;
    u16 peak_a;
    u16 peak_b;
    u16 charge_a;
    u16 charge_b;
    u16 charge_min_i;
    u16 x_pos;
    u16 t_average;
    // don't serialize
};

std::ostream& operator<<(std::ostream& os, const TofHit& pad);

std::ostream& operator<<(std::ostream& os, const MasterTriggerEvent& mt);

std::ostream& operator<<(std::ostream& os, const TofEvent& et);

std::ostream& operator<<(std::ostream& os, const RBEvent& re);

std::ostream& operator<<(std::ostream& os, const RBEventHeader& rh);

#endif 
