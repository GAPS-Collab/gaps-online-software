/*! \file 
 * \brief Tof event classes
 *
 * An event is basically anything with an 
 * event id.
 * - events for individual readoutboards
 *   - RBEventHeader     : header information of event
 *   - RBEvent           : contains header + active channels
 *   - RBWaveform        : A single waveform - this is for the 
 *                         telemetry stream, since larger packets
 *                         would be too big
 * - events for the MasterTriggerBoard
 *
 *  For actual flight code, see the rust project 
 *
 *  This file is part of gaps-online-software and published 
 *  under the GPLv3 license
 *  
 */
#ifndef TOFEVENTS_H_INCLUDED
#define TOFEVENTS_H_INCLUDED

#include <tuple>
#include <array>
#include <format>

#include "result/result.h"

#include "tof_typedefs.h"
#include "packets/monitoring.h"
#include "packets/tof_packet.h"
#include "events/tof_event_header.hpp"
#include "calibration.h"
#include "version.h"
#include "errors.hpp"
#ifdef BUILD_CXX_DB
#include "database.h"
#endif

namespace r = result;
namespace gon = gondola;

class RBCalibration;

#define NCHN 9
#define NWORDS 1024
#define N_LTBS 25
#define N_CHN_PER_LTB 16

namespace gondola {
  
  /*********************************************************/
  
  /// Speed of light in the paddle in cm/ns
  static const f32 C_LIGHT_PADDLE = 15.4; 
  
  /*********************************************************/
    
  static const u8 EVENTSTATUS_UNKNOWN                 =   0;
  static const u8 EVENTSTATUS_CRC32WRONG              =  10;
  static const u8 EVENTSTATUS_TAILWRONG               =  11;
  static const u8 EVENTSTATUS_CHIDWRONG               =  12;
  static const u8 EVENTSTATUS_CELLSYNCERR             =  13;
  static const u8 EVENTSTATUS_CHNSYNCERR              =  14;
  static const u8 EVENTSTATUS_CELLANDCHNSYNCERR       =  15;
  static const u8 EVENTSTATUS_ANYDATAMANGLING         =  16;
  static const u8 EVENTSTATUS_INCOMPLETEREADOUT       =  21;
  static const u8 EVENTSTATUS_INCOMPATIBLEDATA        =  22;
  static const u8 EVENTSTATUS_EVENTTIMEOUT            =  23;
  static const u8 EVENTSTATUS_GOODNOCRCORERRBITCHECK  =  39;
  static const u8 EVENTSTATUS_GOODNOCRCCHECK          =  40;
  static const u8 EVENTSTATUS_GOODNOERRBITCHECK       =  41;
  static const u8 EVENTSTATUS_PERFECT                 =  42;
  
  /// The event status indicates if there are technical 
  /// issues with the retrieval of the event.
  /// If there are no problems, events should have status
  /// EventStatus::EVENTSTATUS_PERFECT (42)
  enum class EventStatus : u8 {
    Unknown                = EVENTSTATUS_UNKNOWN,
    Crc32Wrong             = EVENTSTATUS_CRC32WRONG,
    TailWrong              = EVENTSTATUS_TAILWRONG,
    ChannelIDWrong         = EVENTSTATUS_CHIDWRONG, 
    CellSyncErrors         = EVENTSTATUS_CELLSYNCERR,
    ChnSyncErrors          = EVENTSTATUS_CHNSYNCERR,
    CellAndChnSyncErrors   = EVENTSTATUS_CELLANDCHNSYNCERR,
    AnyDataMangling        = EVENTSTATUS_ANYDATAMANGLING,
    IncompatibleData       = EVENTSTATUS_INCOMPATIBLEDATA,
    EventTimeOut           = EVENTSTATUS_EVENTTIMEOUT,
    GoodNoCRCOrErrBitCheck = EVENTSTATUS_GOODNOCRCORERRBITCHECK,
    /// The event status is good, but we did not 
    /// perform any CRC32 check
    GoodNoCRCCheck         = EVENTSTATUS_GOODNOCRCCHECK,
    /// The event is good, but we did not perform
    /// error checks
    GoodNoErrBitCheck      = EVENTSTATUS_GOODNOERRBITCHECK,
    IncompleteReadout      = EVENTSTATUS_INCOMPLETEREADOUT,
    Perfect                = EVENTSTATUS_PERFECT,
    
  };

  std::ostream& operator<<(std::ostream& os, const gondola::EventStatus& status);

} 



template <>
struct std::formatter<gondola::EventStatus> : std::formatter<std::string> {
  // Parse format specifiers (default implementation)
  constexpr auto parse(std::format_parse_context& ctx) {
      return ctx.begin();
  }
  
  auto format(const gondola::EventStatus& status, auto& ctx) const {
      std::ostringstream oss;
      oss << status;  // Use the << operator to convert enum to string
      return std::format_to(ctx.out(), "{}", oss.str());
  }
};
  
/*********************************************************/

namespace gondola {

  static const u8 TRIGGERTYPE_UNKNOWN      = 0;
  static const u8 TRIGGERTYPE_ANY          = 1;
  static const u8 TRIGGERTYPE_TRACK        = 2;
  static const u8 TRIGGERTYPE_TRACKCENTRAL = 3;
  static const u8 TRIGGERTYPE_GAPS         = 4;
  static const u8 TRIGGERTYPE_POISSON      = 100;
  static const u8 TRIGGERTYPE_FORCED       = 101;
  
  
  /// GAPS Trigger types/sources. Description
  /// can be found elsewhere. More than oen
  /// of them can be active at the same time
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

  std::ostream& operator<<(std::ostream& os, const gondola::TriggerType& t_type);
} 


/*********************************************************/
  
namespace gondola {

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
  
  std::ostream& operator<<(std::ostream& os, const gondola::LTBThreshold& thresh);
  
  /*********************************************************/
  
  /// RB binary data header information
  /// 
  /// This does not include the channel data!
  /// The header contains rb id, event id,
  /// event status and timestamps.
  ///  
  struct RBEventHeader {
    static constexpr u16 HEAD = 0xAAAA;
    static constexpr u16 TAIL = 0x5555;
    static constexpr u16 SIZE = 30; // size in bytes with HEAD and TAIL
  
    u8   rb_id                 = 0;
    u32  event_id              = 0;
    u8   status_byte           = 0;
    u16  channel_mask          = 0;
    u16  stop_cell             = 0;
    u16  ch9_amp               = 0;
    u16  ch9_freq              = 0;
    u16  ch9_phase             = 0; 
    u16  fpga_temp             = 0;
    u32  timestamp32           = 0;
    u16  timestamp16           = 0;
    
    RBEventHeader();
   
    static auto from_bytestream(const Vec<u8> &bytestream, u64 &pos)
      -> r::Result<RBEventHeader, Gaps::IOError>;
  
    auto get_channels()             const -> Vec<u8>;
    auto get_nchan()                const -> u8;
    auto get_active_data_channels() const -> Vec<u8>;
    auto has_ch9()                  const -> bool;
    auto get_n_datachan()           const -> u8;
    auto get_fpga_temp()            const -> f32;
    auto is_event_fragment()        const -> bool;
    auto drs_lost_trigger()         const -> bool;
    auto lost_lock()                const -> bool;
    auto lost_lock_last_sec()       const -> bool;
    auto is_locked()                const -> bool;
    auto is_locked_last_sec()       const -> bool;
    auto get_sine_fit()             const -> std::array<f32,3>;
    /// the combined timestamp 
    auto get_timestamp48()          const -> u64;
    /// string representation for printing
    auto to_string()                const -> std::string;
  };
  
  ///Reconstructed waveform peak information
  ///
  ///There should be one TofHit per reconstructed
  ///peak
  struct TofHit  {
    static constexpr u16 HEAD = 0xF0F0;
    static constexpr u16 TAIL = 0xF0F;
  
    u8   paddle_id;
    // deprecated
    bool broken;
  
    // new variables for V1
    Gaps::ProtocolVersion version;
    f32 baseline_a;
    f32 baseline_a_rms;
    f32 baseline_b;
    f32 baseline_b_rms;
    f32 phase;
  
    // event wide calculated time
    f32 event_t0     = 0;
  
    u32 timestamp32;
    u16 timestamp16;
    

    // don't serialize
    f32 paddle_len    = 0;  
    f32 coax_cbl_time = 0;
    f32 hart_cbl_time = 0;
 
    u8 ctr_etx;
    u16 tail = 0xF0F; 
  
    auto get_time_a()       const -> f32;
    auto get_time_b()       const -> f32;
    auto get_peak_a()       const -> f32;
    auto get_peak_b()       const -> f32;
    auto get_charge_a()     const -> f32;
    auto get_charge_b()     const -> f32;
    auto get_charge_min_i() const -> f32;
    auto get_x_pos()        const -> f32;
    auto get_t_avg()        const -> f32;
    /// If the two reconstructed pulse times are not related to each other by the paddle length,
    /// meaning that they can't be caused by the same event, we dub this hit as "not following
    /// causality"
    auto obeys_causality()  const -> bool; 
    /// get the interaction time of the particle,
    /// not accounting for cable len and global phase
    auto get_t0_relative()  const -> f32;
    auto get_timestamp48()  const -> f64;
    
    /// time-over-threshold for paddle end A for the 
    /// lower threshold (see config file for value)
    auto get_tot_low_a()    const -> f32;
    /// time-over-threshold for paddle end B for the 
    /// lower threshold (see config file for value)
    auto get_tot_low_b()    const -> f32;
    /// time-over-threshold for paddle end A for the 
    /// higher threshold (see config file for value)
    auto get_tot_high_a()    const -> f32;
    /// time-over-threshold for paddle end B for the 
    /// higher threshold (see config file for value)
    auto get_tot_high_b()     const -> f32;
    /// the slope of the waveform at the point of the 
    /// intersection of the lower threshold and the 
    /// waveform for side A
    auto get_tot_slp_low_a()  const -> f32;
    /// the slope of the waveform at the point of the 
    /// intersection of the lower threshold and the 
    /// waveform for side B
    auto get_tot_slp_low_b()  const -> f32;
    /// the slope of the waveform at the point of the 
    /// intersection of the higher threshold and the 
    /// waveform for side A
    auto get_tot_slp_high_a() const -> f32;
    /// the slope of the waveform at the point of the 
    /// intersection of the higher threshold and the 
    /// waveform for side B
    auto get_tot_slp_high_b() const -> f32;


    /// The paddle length will not be in the packet,
    /// but has to be added after the fact
    void set_paddle_len(f32 paddle_len);
  
    #if BUILD_CXX_DB
    auto set_paddle(const Gaps::TofPaddle& paddle) -> void;
    auto get_phase_delay() const -> f32;
    auto get_cable_delay() const -> f32;
    auto get_t0()          const -> f32;
    auto get_edep()        const -> f32;
    #endif
  
    static auto from_bytestream(const Vec<u8> &bytestream, u64 &pos)
      -> r::Result<TofHit,Gaps::IOError>;
   
    // String representation for printing
    auto to_string() const -> std::string;
    
    private:
      // we keep this private, since 
      // the user should use the getters
      // to get the values converted 
      // back to f32
      // deprecated, but kept for compatibility
      u16 time_a;
      u16 time_b;
      u16 peak_a;
      u16 peak_b;
      u16 charge_a;
      u16 charge_b;
      u16 charge_min_i;
      u16 x_pos;
      u16 t_average;
      
      f32 time_a_f32   = 0;
      f32 time_b_f32   = 0;
      f32 peak_a_f32   = 0;
      f32 peak_b_f32   = 0;
      f32 charge_a_f32 = 0;
      f32 charge_b_f32 = 0;

      // new (2025/26) variables to deal with 
      // pulse saturation.
      // These are variables for time-over-threshold
      f32 tot_low_a      = 0;
      f32 tot_low_b      = 0;
      f32 tot_high_a     = 0;
      f32 tot_high_b     = 0;
      f32 tot_slp_low_a  = 0;
      f32 tot_slp_low_b  = 0;
      f32 tot_slp_high_a = 0;
      f32 tot_slp_high_b = 0;

  };
  
  /// A complete event for a single readout board 
  /// with header and channel data.
  /// The size is flexible, only active datachannels
  /// are recorded.
  struct RBEvent {
    static constexpr u16 HEAD = 0xAAAA;
    static constexpr u16 TAIL = 0x5555;
  
    // data type will be an enum
    u8            data_type = 0;
    EventStatus   status    = EventStatus::Unknown;
    RBEventHeader header    = RBEventHeader();
    Vec<Vec<u16>> adc       = Vec<Vec<u16>>(); 
    Vec<TofHit>   hits      = Vec<TofHit>();  
   
    RBEvent();
  
    auto get_channel_by_label(u8 channel) const -> const Vec<u16>&;
    auto get_channel_by_id(u8 channel)    const -> const Vec<u16>&;
  
    auto get_channel_adc(u8 channel) const -> const Vec<u16>&; 
   
    /// Get the baseline for a single channel
    static auto calc_baseline(const Vec<f32> &volts, usize min_bin, usize max_bin) -> f32; 
  
    static auto from_bytestream(const Vec<u8> &bytestream, u64 &pos)
      -> RBEvent;
  
    auto to_string() const -> std::string;
  
    private:
  
      /**
       * Check if the channel follows the convention 1-9
       *
       */
      auto channel_check(u8 channel) const -> bool;
      Vec<u16> _empty_channel = Vec<u16>();
  };
  
  /*********************************************************/
  
  static const u8 EVENT_QUALITY_UNKNOWN         =  0;
  static const u8 EVENT_QUALITY_SILVER          =  10;
  static const u8 EVENT_QUALITY_GOLD            =  20;
  static const u8 EVENT_QUALITY_DIAMOND         =  30;
  static const u8 EVENT_QUALITY_FOURLEAFCLOVER  =  40;
  
  
  /// EventQuality will get assigned by online reconstructions
  /// or the flight computer. This contains information about
  /// physics and might pre-select "golden" candidate events.
  /// The default event quelity is EventQuality::UNKNOWN
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

  std::ostream& operator<<(std::ostream& os, const gondola::EventQuality& qual);
  
  /*********************************************************/

  static const u8 COMPRESSION_LEVEL_UNKNOWN         =  0;
  static const u8 COMPRESSION_LEVEL_NONE            =  10;
  
  enum class CompressionLevel : u8 {
    Unknown        = COMPRESSION_LEVEL_UNKNOWN,
    None           = COMPRESSION_LEVEL_NONE,
  };
  
  std::ostream& operator<<(std::ostream& os, const gondola::CompressionLevel& level);
  
  /// The MasterTriggerEvent represesnts the information
  /// provided by the MTB for this one specific event.
  /// Most notably, it includes a board mask,
  /// which is the DSI/J connections which triggered, and 
  /// a hit mask. The hit mask gives hit channels per DSI/J,
  /// which correspond to hit channels on a LTB.
  /// 
  /// FIXME -compatibiltiy. Reading older data, we have 
  /// 2 scenarios - either 113 bytes of fixed size for 
  /// 20 LTBs, or 133 bytes for 25 LTBs. 
  /// We can modify from_bytestream to at least not 
  /// throw an error when reading older data, but 
  /// this would currently be a #todo of lower 
  /// priority
  struct MasterTriggerEvent {
    /// begin struct marker
    static constexpr u16 HEAD = 0xAAAA;
    /// end struct marker
    static constexpr u16 TAIL = 0x5555;
    /// Variable size for MasterTriggerEvent
    static constexpr usize SIZE = 0; // size in bytes
    /// 
    EventStatus event_status = EventStatus::Unknown;
    /// event_id as assigned by the MasterTriggerBoard
    u32 event_id             = 0; 
    /// MTB timestamp
    u32 timestamp            = 0; 
    /// Tracker (?) timestamp
    u32 tiu_timestamp        = 0; 
    /// GAPS GPS clock value (slow)
    u32 tiu_gps32            = 0; 
    /// GAPS GPS clock value (fast)
    u32 tiu_gps16            = 0; 
    /// triggered paddles as seen by the MTB
    u32 crc                  = 0;
    u16 trigger_source       = 0;
    u32 dsi_j_mask           = 0;
    Vec<u16> channel_mask    = Vec<u16>();
    u64 mtb_link_mask        = 0;
    
    MasterTriggerEvent();
    
    /// The combined GPS 48bit timestamp
    /// into a 48bit timestamp
    [[deprecated("The format of the gps timestamp changed and it is only 32 bits as of now")]]
    auto get_timestamp_gps48() const -> u64;
    /// Get the timestamp as sent by the GPS
    auto get_timestamp_gps() const -> u32;
    /// Get absolute timestamp which is calculated 
    /// with the help of the 1pps pulse from the GPS
    auto get_timestamp_abs48() const -> u64;
    auto get_rb_link_ids()     const -> Vec<u8>;
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
    auto get_trigger_hits() const 
      -> Vec<std::tuple<u8, u8, u8, LTBThreshold>>;
  
    /// Get the trigger sources from trigger source byte
    auto get_trigger_sources() const -> Vec<TriggerType>; 
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
    static auto from_bytestream(const Vec<u8> &bytestream, u64 &pos)
      -> MasterTriggerEvent;
    /// String representation of the struct
    auto to_string() const -> std::string;
  };
  
  /// A container accounting for a "complete" event of the Tof
  /// including:
  /// - A MasterTriggerEvent
  /// - Possible monitoring data for readoutboards
  /// - A number of Readoutboardevents (each with 
  ///   header and the number of active channels) 
  /// - A number of MissingHits. These are such 
  ///   where the MTB claims we should see data 
  ///   in one of the RBs, but we do not have 
  ///   any.
  struct TofEvent {
    static constexpr u16 HEAD = 0xAAAA;
    static constexpr u16 TAIL = 0x5555;
 
    Gaps::ProtocolVersion version ;
    EventStatus status            ;
    EventQuality          quality ;
    u16         trigger_sources   ;
    /// the number of triggered paddles coming
    /// from the MTB directly. This might NOT be
    /// the same as the number of hits!
    u8          n_trigger_paddles ;
    u32         event_id          ;
    u16         run_id            ;
    u32         timestamp32       ;
    u16         timestamp16       ;
    // deprecated, won't get serialized
    //u16         primary_beta      ;
    //u16         primary_charge    ;

    u16         drs_dead_lost_hits;
    u32         dsi_j_mask        ;
    Vec<u16>    channel_mask      ;
    u64         mtb_link_mask     ;

    // flight computer event variable packet
    u8          n_hits_umb        ;
    u8          n_hits_cbe        ;
    u8          n_hits_cor        ;
    f32         tot_edep_umb      ;
    f32         tot_edep_cbe      ;
    f32         tot_edep_cor      ;
    TofEventHeader header;
    MasterTriggerEvent mt_event;

    /// A container holding the individual events from all RBs with 
    /// triggers in this event  
    Vec<RBEvent>      rb_events = {};
    Vec<TofHit>       hits      = {};


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
    static auto from_bytestream(const Vec<u8> &bytestream, u64 &pos)
      -> r::Result<TofEvent, Gaps::IOError>;
  
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
    static auto from_tofpacket(const TofPacket &packet) -> TofEvent;
  
    #ifdef BUILD_CXX_DB
    /// set a TofPaddle, that is enrich every tofhit with information
    /// about the corresponding paddle
    auto set_paddlemap(const Gaps::TofPaddleMap&) -> void;
    #endif
      
    static auto get_n_rbevents(u32 mask) -> u32;
    /// Get all hits from all rb_events
    auto get_hits() const -> Vec<TofHit>;
    #ifdef BUILD_CXX_DB
    /// normalize all the hit times, taking the global ch9 
    /// phase into account
    auto normalize_hit_times(const TofPaddleTimingConstantMap &offsets) -> void;
    #endif 
    /// string representation for printing
    auto to_string() const -> std::string;
  
    /**
     * Get the rb event for a specific board id.
     */
    auto get_rbevent(u8 board_id) const -> const RBEvent&; 
 
    /// Get the ids of the readoutboards participating in 
    /// the event
    auto get_rbids() const -> Vec<u8>;

    private:
      /**
       * Check if there are more than one RBEvent per board
       * and if the eventids are matching up.
       */
      auto passed_consistency_check() -> bool;
  
      /// an empty event, which can be returned 
      /// in case of a null result.
      RBEvent _empty_event = RBEvent();
  };
  
  
  /// A part of a TofEvent 
  /// - a single waveform 
  /// 
  /// That is a waveform for 
  /// a specific channel for a 
  /// specific id.
  /// 
  /// Each paddle has 2 waveforms
  struct RBWaveform {
    static constexpr u16 HEAD = 0xAAAA;
    static constexpr u16 TAIL = 0x5555;
  
    u32       event_id     ; 
    u8        rb_id        ; 
    u8        rb_channel_a ; 
    u8        rb_channel_b ;
    u16       stop_cell    ;
    Vec<u16>  adc_a        ; 
    Vec<u16>  adc_b        ;
    
    static auto from_bytestream(const Vec<u8> &bytestream, u64 &pos) -> RBWaveform;
    auto to_string() const -> std::string;
  };
  
  
  /// Concise summary for the flight computer and 
  /// telemtry stream
  struct TofEventSummary {
    static constexpr u16 HEAD = 0xAAAA;
    static constexpr u16 TAIL = 0x5555;
  
    Gaps::ProtocolVersion version ;
    EventStatus status            ; 
    u8          quality           ; 
    u16         trigger_sources   ; 
    /// the number of triggered paddles coming
    /// from the MTB directly. This might NOT be
    /// the same as the number of hits!
    u8          n_trigger_paddles ; 
    u32         event_id          ; 
    u16         run_id            ;
    u32         timestamp32       ; 
    u16         timestamp16       ; 
    // deprecated, won't get serialized
    //u16         primary_beta      ; 
    //u16         primary_charge    ;
    
    u16         drs_dead_lost_hits; 
    u32         dsi_j_mask        ;
    Vec<u16>    channel_mask      ;
    u64         mtb_link_mask     ;
    Vec<TofHit> hits              ;
    
    // flight computer event variable packet
    u8          n_hits_umb        ;
    u8          n_hits_cbe        ;
    u8          n_hits_cor        ;
    f32         tot_edep_umb      ;
    f32         tot_edep_cbe      ;
    f32         tot_edep_cor      ;
    
    static auto from_tofpacket(const TofPacket &packet)          
      -> r::Result<TofEventSummary, Gaps::IOError>;
    static auto from_bytestream(const Vec<u8> &stream, u64 &pos) -> r::Result<TofEventSummary, Gaps::IOError> ;
    
    #ifdef BUILD_CXX_DB
    /// set a TofPaddle, that is enrich every tofhit with information
    /// about the corresponding paddle
    auto set_paddlemap(const Gaps::TofPaddleMap&) -> void;
    /// normalize all the hit times, taking the global ch9 
    /// phase into account
    auto normalize_hit_times(const TofPaddleTimingConstantMap &offsets) -> void;
    #endif
  
    // combined timestamp
    auto get_timestamp48() const -> u64;
   
    auto get_rb_link_ids() const -> Vec<u8>;
    
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
    auto get_trigger_hits() const -> Vec<std::tuple<u8, u8, u8, LTBThreshold>>;
    /// Get the trigger sources from trigger source byte
    auto get_trigger_sources() const -> Vec<TriggerType>; 
    
    auto to_string() const -> std::string;
  };

  std::ostream& operator<<(std::ostream& os, const gondola::TofHit& pad);
  
  std::ostream& operator<<(std::ostream& os, const gondola::MasterTriggerEvent& mt);
  
  std::ostream& operator<<(std::ostream& os, const gondola::TofEvent& et);
  
  std::ostream& operator<<(std::ostream& os, const gondola::RBEvent& re);
  
  std::ostream& operator<<(std::ostream& os, const gondola::RBEventHeader& rh);
  
  std::ostream& operator<<(std::ostream& os, const gondola::RBWaveform& rh);
  
  std::ostream& operator<<(std::ostream& os, const gondola::TofEventSummary& tes);
} 


#endif 
