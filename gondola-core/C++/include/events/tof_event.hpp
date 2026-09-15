/// This file is part of gaps-online-software and published 
/// under the GPLv3 license
#pragma once 

#include "result/result.h"
#include "gondola.hpp"
#include "version.h"
#include "events/event_status.hpp"
#include "events/event_quality.hpp"
#include "events/trigger.hpp"
#include "events/rb_event.hpp"
#include "packets/tof_packet.h"
#include "database.h"

namespace gondola {

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
 
    ProtocolVersion       version ;
    EventStatus           status  ;
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
    //MasterTriggerEvent mt_event;

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
      -> r::Result<TofEvent, IOError>;
  
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
    auto set_paddlemap(const TofPaddleMap&) -> void;
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
  
  std::ostream& operator<<(std::ostream& os, const gondola::TofEvent& et);

}
