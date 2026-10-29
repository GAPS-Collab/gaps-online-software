#pragma once 

namespace gondola {
  
    struct TrackerStripPedestal {
      i32         data_id; // primary key
      u32         strip_id;
      u64         volume_id;
      u64         utc_timestamp_start;
      u64         utc_timestamp_stop;
      std::string name;
      f32         pedestal_mean;
      f32         pedestal_sigma;
      bool        is_mean_value;
    
      auto to_string() const -> std::string; 
    };
    
    typedef std::map<u32, TrackerStripPedestal> TrkStripPedMap;
    
    auto get_trackerstrippedestals(std::string name) -> TrkStripPedMap;
    
    auto get_trackerstrippedestals_by_volumeid(std::string name) -> TrkStripPedMap;
}  
