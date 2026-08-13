/// This file is part of gaps-online-software and published 
/// under the GPLv3 license
  
#pragma once

namespace gondola {
  struct TofMetaData {
    u32  event_id {0xffffffff};
    u8   status_version {0xff};
    bool stats_valid {false};
    u16  trigger_sources {0};
    u8   n_hits_umb {0xff};
    u8   n_hits_cbe {0xff};
    u8   n_hits_cor {0xff};
    f32  tot_edep_umb {0};
    f32  tot_edep_cbe {0};
    f32  tot_edep_cor {0};
    
    static auto from_bytestream(Vec<u8> const &stream, usize &pos) -> TofMetaData;
    auto to_string() const -> std::string;
  };
}

