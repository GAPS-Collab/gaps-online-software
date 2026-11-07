#pragma once 

#include "gondola.hpp"

#include "mc_hit.hpp"
#include "tracklet.hpp"


namespace gondola {
  struct McEvent {
    static constexpr u16 HEAD = 0xAAAA;
    static constexpr u16 TAIL = 0x5555;

    u32 run_id   = 0;
    u32 event_id = 0;
    Tracklet primary;
    Vec<McHit> hits;
   
    auto to_bytestream() const -> Vec<u8>;
    /// Deserialization, mirrors the rust implementation. 
    /// Available without BUILD_SIM, e.g. for converting 
    /// simulated data to SimpleDet root files.
    static auto from_bytestream(const Vec<u8> &stream, usize &pos)
      -> result::Result<McEvent, IOError>;
  };
}
