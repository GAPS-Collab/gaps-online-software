// This file is part of gaps-online-software and published 
// under the GPLv3 license
#pragma once 

#include "gondola_typedefs.hpp"

namespace gondola {
  struct TrackerStripGain {   
    i32          data_id            ;
    i32          strip_id           ;  
    i64          volume_id          ;  
    i64          utc_timestamp_start;  
    i64          utc_timestamp_stop ;  
    std::string  name               ; 
    f32          gain               ;
    bool         gain_is_mean       ;
    
    auto to_string() const -> std::string;
    
    friend std::ostream& operator<<(std::ostream& os, const TrackerStripGain& paddle);
  }; 
  
  typedef std::map<u32, f32> TrkStripGainMap;
  
  /// Get all gain (values) for a specific name
  /// (the name will be usually related to the filename
  auto get_trackerstripgains(std::string name) -> TrkStripGainMap;
}


