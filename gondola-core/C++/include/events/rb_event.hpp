/// This file is part of gaps-online-software and published 
/// under the GPLv3 license
#pragma once 

#include "result/result.h"
#include "tof_typedefs.h"
#include "errors.hpp"
#include "database.h"
#include "version.h"
#include "events/rb_event_header.hpp"
#include "events/tof_hit.hpp"
#include "events/event_status.hpp" 

namespace gondola {
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
  
  std::ostream& operator<<(std::ostream& os, const gondola::RBEvent& re);
}
