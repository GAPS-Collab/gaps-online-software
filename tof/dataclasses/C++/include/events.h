#ifndef TOFEVENTS_H_INCLUDED
#define TOFEVENTS_H_INCLUDED

/**
 * Tof event classes. An event is basically anything with an 
 * event id.
 * - events for individual readoutboards
 *   - RBEventHeader     : header information of event
 *   - RBEvent           : contains header + active channels
 *   - RBWaveform        : A single waveform - this is for the 
 *                         telemetry stream, since larger packets
 *                         would be too big
 * - events for the MasterTriggerBoard
 * 
 *
 *
 */ 

#include <tuple>
#include <array>

#include "tof_typedefs.h"
#include "packets/monitoring.h"
#include "packets/tof_packet.h"
#include "events/tof_event_header.hpp"
#include "calibration.h"

class RBCalibration;

#define NCHN 9
#define NWORDS 1024
#define N_LTBS 25
#define N_CHN_PER_LTB 16

struct RBEventHeader;
struct RBEvent;
struct MasterTriggerEvent;
struct TofHit;

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

static const u8 TRIGGERTYPE_UNKNOWN      = 0;
static const u8 TRIGGERTYPE_GAPS         = 4;
static const u8 TRIGGERTYPE_ANY          = 1;
static const u8 TRIGGERTYPE_TRACK        = 2;
static const u8 TRIGGERTYPE_TRACKCENTRAL = 3;
static const u8 TRIGGERTYPE_POISSON      = 100;
static const u8 TRIGGERTYPE_FORCED       = 101;


/************************************
 *
 * GAPS Trigger types/sources. Description
 * can be found elsewhere. More than oen
 * of them can be active at the same time
 *
 */
enum class TriggerType : u8 {
  Unknown      = TRIGGERTYPE_UNKNOWN,
  /// -> 1-10 "pysics" triggers
  Gaps         = TRIGGERTYPE_GAPS,
  Any          = TRIGGERTYPE_ANY,
  Track        = TRIGGERTYPE_TRACK,
  TrackCentral = TRIGGERTYPE_TRACKCENTRAL,
  /// > 100 -> Debug triggers
  Poisson      = TRIGGERTYPE_POISSON,
  Forced       = TRIGGERTYPE_FORCED, 
};

std::ostream& operator<<(std::ostream& os, const TriggerType& t_type);

/*********************************************************/

static const u8 LTBTHRESHOLD_NOHIT   = 0;
static const u8 LTBTHRESHOLD_HIT     = 1;
static const u8 LTBTHRESHOLD_BETA    = 2;
static const u8 LTBTHRESHOLD_VETO    = 3;
static const u8 LTBTHRESHOLD_UNKNOWN = 255;

enum class LTBThreshold : u8 {
  NoHit   = LTBTHRESHOLD_NOHIT,
  /// First threshold, 40mV, about 0.75 minI
  Hit     = LTBTHRESHOLD_HIT,
  /// Second threshold, 32mV (? error in doc ?, about 2.5 minI
  Beta    = LTBTHRESHOLD_BETA,
  /// Third threshold, 375mV about 30 minI
  Veto    = LTBTHRESHOLD_VETO,
  /// Use u8::MAX for Unknown, since 0 is pre-determined for 
  /// "NoHit, 
  Unknown = LTBTHRESHOLD_UNKNOWN,
};

std::ostream& operator<<(std::ostream& os, const LTBThreshold& thresh);

/*********************************************************/

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
  u16  ch9_amp               ;
  u16  ch9_freq              ;
  u16  ch9_phase             ; 
  u16  fpga_temp             ;
  u32  timestamp32           ;
  u16  timestamp16           ;
  
  RBEventHeader();
 
  static RBEventHeader from_bytestream(const Vec<u8> &bytestream,
                                       u64 &pos);

  Vec<u8> get_channels()    const;
  u8      get_nchan()       const;
  Vec<u8> get_active_data_channels() const;
  bool    has_ch9()            const;
  u8      get_n_datachan()     const;
  f32     get_fpga_temp()      const;
  bool    is_event_fragment()  const;
  bool    drs_lost_trigger()   const;
  bool    lost_lock()          const;
  bool    lost_lock_last_sec() const;
  bool    is_locked()          const;
  bool    is_locked_last_sec() const;

  std::array<f32, 3> get_sine_fit() const;
    
  /// the combined timestamp 
  u64  get_timestamp48()    const;

  /// string representation for printing
  std::string to_string() const;
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
  Vec<TofHit> hits;
 
  RBEvent();

  const Vec<u16>& get_channel_by_label(u8 channel) const;
  const Vec<u16>& get_channel_by_id(u8 channel) const;

  const Vec<u16>& get_channel_adc(u8 channel) const; 
 
  /// Get the baseline for a single channel
  static f32 calc_baseline(const Vec<f32> &volts, usize min_bin, usize max_bin); 

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
 * FIXME -compatibiltiy. Reading older data, we have 
 * 2 scenarios - either 113 bytes of fixed size for 
 * 20 LTBs, or 133 bytes for 25 LTBs. 
 * We can modify from_bytestream to at least not 
 * throw an error when reading older data, but 
 * this would currently be a #todo of lower 
 * priority
 *
 *
 */
struct MasterTriggerEvent {
  /// begin struct marker
  static const u16 HEAD = 0xAAAA;
  /// end struct marker
  static const u16 TAIL = 0x5555;
  /// the struct has a fixed size of SIZE
  static const usize SIZE = 45; // size in bytes
  /// 
  EventStatus event_status;
  /// event_id as assigned by the MasterTriggerBoard
  u32 event_id            ; 
  /// MTB timestamp
  u32 timestamp           ; 
  /// Tracker (?) timestamp
  u32 tiu_timestamp       ; 
  /// GAPS GPS clock value (slow)
  u32 tiu_gps32           ; 
  /// GAPS GPS clock value (fast)
  u32 tiu_gps16           ; 
  /// triggered paddles as seen by the MTB
  u32 crc                 ;
  u16 trigger_source      ;
  u32 dsi_j_mask          ;
  Vec<u16> channel_mask   ;
  u64 mtb_link_mask       ;
  
  MasterTriggerEvent();
  
  Vec<u8> get_rb_link_ids() const;
  
  /// Get the combination of triggered DSI/J/CH on 
  /// the MTB which formed the trigger. This does 
  /// not include further hits which fall into the 
  /// integration window. For those, se rb_link_mask
  ///
  /// The returned values follow the TOF convention
  /// to start with 1, so that we can use them to 
  /// look up LTB ids in the db.
  ///
  /// # Returns
  ///
  ///   Vec<(hit)> where hit is (DSI, J, CH) 
  Vec<std::tuple<u8, u8, u8, LTBThreshold>> get_trigger_hits() const;

  /// The combined GPS 48bit timestamp
  /// into a 48bit timestamp
  u64 get_timestamp_gps48() const;

  /// Get absolute timestamp as sent by the GPS
  u64 get_timestamp_abs48() const;

  /// Get the trigger sources from trigger source byte
  Vec<TriggerType> get_trigger_sources() const; 
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

  EventStatus status;
  TofEventHeader header;
  MasterTriggerEvent mt_event;


  /// A container holding the individual events from all RBs with 
  /// triggers in this event  
  Vec<RBEvent>      rb_events;
  /// A container holding information about missing rbevents. That 
  /// is events where we know the board triggered, but we did not
  /// get an associated RBEvent within a timeout
  Vec<RBMissingHit> missing_hits;

  TofEvent();

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

/************************
 * A part of a TofEvent 
 * - a single waveform 
 *
 * That is a waveform for 
 * a specific channel for a 
 * specific id.
 *
 * Each paddle has 2 waveforms
 *
 *
 */ 
struct RBWaveform {
  static const u16 HEAD = 0xAAAA;
  static const u16 TAIL = 0x5555;

  u32       event_id  ; 
  u8        rb_id     ; 
  u8        rb_channel; 
  u16       stop_cell ;
  Vec<u16>  adc       ; 
  
  static RBWaveform from_bytestream(const Vec<u8> &bytestream, 
                                    u64 &pos);
  
  std::string to_string() const;
};


/**
 * Concise summary for the flight computer and 
 * telemtry stream
 *
 *
 */
struct TofEventSummary {
  static const u16 HEAD = 0xAAAA;
  static const u16 TAIL = 0x5555;

  u8          status            ; 
  u8          quality           ; 
  u8          trigger_setting   ; 
  /// the number of triggered paddles coming
  /// from the MTB directly. This might NOT be
  /// the same as the number of hits!
  u8          n_trigger_paddles ; 
  u32         event_id          ; 
  u32         timestamp32       ; 
  u16         timestamp16       ; 
  /// reconstructed primary beta
  u16         primary_beta      ; 
  /// reconstructed primary charge
  u16         primary_charge    ; 
  Vec<TofHit> hits              ;
  
  static TofEventSummary from_bytestream(const Vec<u8> &stream, 
                                         u64 &pos);
  // combined timestamp
  u64  get_timestamp48()    const;
  
  std::string to_string() const;
};

std::ostream& operator<<(std::ostream& os, const TofHit& pad);

std::ostream& operator<<(std::ostream& os, const MasterTriggerEvent& mt);

std::ostream& operator<<(std::ostream& os, const TofEvent& et);

std::ostream& operator<<(std::ostream& os, const RBEvent& re);

std::ostream& operator<<(std::ostream& os, const RBEventHeader& rh);

std::ostream& operator<<(std::ostream& os, const RBWaveform& rh);

std::ostream& operator<<(std::ostream& os, const TofEventSummary& tes);

#endif 
