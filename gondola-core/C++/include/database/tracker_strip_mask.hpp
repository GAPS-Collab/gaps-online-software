// This file is part of gaps-online-software and published 
// under the GPLv3 license
#pragma once

#include "gondola_typedefs.hpp"
  
namespace gondola {
  /// Each module can have a mask, which allows to disable
  /// trcker strips. The mask is typically a 32bit number
  struct TrackerStripMask {
    u32         strip_id ;
    u64         volume_id;
    u64         utc_timestamp;
    std::string mask_name; 
    bool        active     ; 
  
    auto to_string() const -> std::string;
    friend std::ostream& operator<<(std::ostream& os, const gondola::TrackerStripMask& strip);

  };

  typedef std::map<u32, bool> TrkStripMaskMap;

  auto get_trackerstripmasks(std::string mask_name = "") -> TrkStripMaskMap;
}
