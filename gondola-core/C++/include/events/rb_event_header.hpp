/// This file is part of gaps-online-software and published 
/// under the GPLv3 license
#pragma once 

#include "result/result.h"

#include "errors.hpp"

namespace r = result;

namespace gondola {

  /// RB binary data header information
  /// 
  /// This does not include the channel data!
  /// The header contains rb id, event id,
  /// event status and timestamps.
  ///  
  struct RBEventHeader {
    static constexpr u16 HEAD = 0xAAAA;
    static constexpr u16 TAIL = 0x5555;
    static constexpr u16 SIZE = 30; // size in bytes with HEAD and TAIL
  
    u8   rb_id                 = 0;
    u32  event_id              = 0;
    u8   status_byte           = 0;
    u16  channel_mask          = 0;
    u16  stop_cell             = 0;
    u16  ch9_amp               = 0;
    u16  ch9_freq              = 0;
    u16  ch9_phase             = 0; 
    u16  fpga_temp             = 0;
    u32  timestamp32           = 0;
    u16  timestamp16           = 0;
    
    RBEventHeader();
   
    static auto from_bytestream(const Vec<u8> &bytestream, u64 &pos)
      -> r::Result<RBEventHeader, IOError>;
  
    auto get_channels()             const -> Vec<u8>;
    auto get_nchan()                const -> u8;
    auto get_active_data_channels() const -> Vec<u8>;
    auto has_ch9()                  const -> bool;
    auto get_n_datachan()           const -> u8;
    auto get_fpga_temp()            const -> f32;
    auto is_event_fragment()        const -> bool;
    auto drs_lost_trigger()         const -> bool;
    auto lost_lock()                const -> bool;
    auto lost_lock_last_sec()       const -> bool;
    auto is_locked()                const -> bool;
    auto is_locked_last_sec()       const -> bool;
    auto get_sine_fit()             const -> std::array<f32,3>;
    /// the combined timestamp 
    auto get_timestamp48()          const -> u64;
    /// string representation for printing
    auto to_string()                const -> std::string;
  };
  
  std::ostream& operator<<(std::ostream& os, const gondola::RBEventHeader& rh);
}
