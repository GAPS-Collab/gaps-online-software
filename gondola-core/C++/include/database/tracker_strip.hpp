// This file is part of gaps-online-software and published 
// under the GPLv3 license
#pragma once

#include "gondola_typedefs.hpp"

namespace gondola {
  struct TrackerStrip {
    u32 strip_id           ;
    i32 layer              ; 
    i32 row                ; 
    i32 module             ; 
    i32 channel            ;  
    f32 global_pos_x_l0    ;
    f32 global_pos_y_l0    ;
    f32 global_pos_z_l0    ;
    f32 global_pos_x_det_l0;
    f32 global_pos_y_det_l0;
    f32 global_pos_z_det_l0;
    f32 principal_x        ;
    f32 principal_y        ;
    f32 principal_z        ;
    u64 volume_id          ;
  
    auto to_string() const -> std::string;
    auto create_id() const -> u32; 
    static auto create_id(u32 layer, u32 row, u32 module, u32 channel) -> u32;
    
    // FIXME - needs implementation
    ///// Vector along the longest axis
    //auto get_principal() const -> Vec<f32>;
  
    friend std::ostream& operator<<(std::ostream& os, const gondola::TrackerStrip& strip);
  };

  /// A map of strip identifier (layer-row-module-channel -> Tracker strip
  typedef std::map<u32, TrackerStrip> TrkStripMap;
  
  /// Retrieve all tracker strips from the database
  auto get_trackerstrips() -> TrkStripMap;        
  
  /// Get the position of a module - returns in cm
  auto get_module_position(u8 layer, u8 row, u8 mod, const TrkStripMap&) -> Vec<f32>;
  
}

