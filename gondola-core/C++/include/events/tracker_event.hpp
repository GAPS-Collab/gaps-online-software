#pragma once

#include "events/tracker_hit.hpp"

namespace gondola {
  struct TrkEvent {
    u8          layer;
    u8          flags1;
    u32         event_id; 
    u64         event_time;
    Vec<TrkHit> hits;

    auto to_string() const -> std::string;
  };
}

