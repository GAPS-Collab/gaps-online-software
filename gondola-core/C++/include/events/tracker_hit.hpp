#pragma once 

#include "tof_typedefs.h"

namespace gondola {
  struct TrkHit {
    // using i32 here makes no sense in my eyes, but I defer to 
    // bfsw ( I assume it is because of sqlite which does not know
    // unsigned) 
    // FIXME - change this
    i32 layer           {-1};
    i32 row             {-1};
    i32 module          {-1};
    i32 channel         {-1};
    i32 adc             {-1};
    i64 oscillator      {-1};
    f64 energy          {0};
    auto get_strip_id() const -> u32;
    /// In BFSW, there are two versions of the tracker hit, 
    /// tracker_hit and tracker::hit. The latter has 
    /// an extra ASIC event code field. Let's unify those here
    u8  asic_event_code {0};
    auto to_string() const -> std::string;
    
    /// Decode layer, row, module, channel from the strip id 
    static auto decode_id(u32 hw_id) -> Vec<u32>;
  
  };

}
