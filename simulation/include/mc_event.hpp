#include "gondola.hpp"

#include "mc_hit.hpp"
#include "tracklet.hpp"


namespace gondola {
  struct McEvent {
    u32 run_id;
    u32 event_id;
    Tracklet primary;
    Vec<McHit> hits;
   
    auto to_bytestream() const -> Vec<u8>;
  };
}
