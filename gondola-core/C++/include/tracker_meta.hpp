/// This file is part of gaps-online-software and published 
/// under the GPLv3 license
  
#pragma once

#include "events/tracker_calibrated_hit.hpp"

namespace gondola {
  struct TrkMetaData {
    u64 num_hits {0};
    u64 row_flags {0};
    f64 total_energy {0};
    Vec<TrkCalibratedHit> calibrated_hits;
    //std::array<uint64_t,8> oscillators = {0,0,0,0,0,0,0,0};
  };
}
