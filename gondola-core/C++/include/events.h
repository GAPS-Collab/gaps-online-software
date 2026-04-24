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
#include "events/rb_event_header.hpp"
#include "events/tof_hit.hpp"
#include "events/rb_event.hpp"
#include "events/event_status.hpp"
#include "events/event_quality.hpp"
#include "events/rb_waveform.hpp"
#include "events/tof_event.hpp"
#include "calibration.h"
#include "version.h"
#include "errors.hpp"
#ifdef BUILD_CXX_DB
#include "database.h"
#endif

namespace r = result;
//namespace g = gondola;

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

  enum class CompressionLevel : u8 {
    Unknown        =  0,
    None           = 10,
  };
  
  std::ostream& operator<<(std::ostream& os, const gondola::CompressionLevel& level);
   
  /// Concise summary for the flight computer and 
  /// telemtry stream
  struct TofEventSummary {
    static constexpr u16 HEAD = 0xAAAA;
    static constexpr u16 TAIL = 0x5555;
  
    gondola::ProtocolVersion    version ;
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
      -> r::Result<TofEventSummary, gondola::IOError>;
    static auto from_bytestream(const Vec<u8> &stream, u64 &pos) -> r::Result<TofEventSummary, gondola::IOError> ;
    
    #ifdef BUILD_CXX_DB
    /// set a TofPaddle, that is enrich every tofhit with information
    /// about the corresponding paddle
    auto set_paddlemap(const gondola::TofPaddleMap&) -> void;
    /// normalize all the hit times, taking the global ch9 
    /// phase into account
    auto normalize_hit_times(const TofPaddleTimingConstantMap &offsets) -> void;
    /// get the trigger hits directly if we have a paddle map) 
    auto get_trigger_pids(const gondola::DsiJChnPaddleIdMap& lgmap) const -> Vec<u8>;
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
 
   
  
  std::ostream& operator<<(std::ostream& os, const gondola::TofEventSummary& tes);
} 


#endif 
