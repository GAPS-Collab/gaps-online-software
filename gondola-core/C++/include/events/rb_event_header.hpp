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
    /// The status byte contains information about lsos of lock
    /// and event fragments and needs to be decoded
    u8   status_byte           = 0;
    /// The channel mask is 9bit for the 9 channels.
    /// This leaves 7 bits of space so we actually 
    /// hijack that for the version information 
    /// 
    /// Bit 15 will be set 1 in case we are sending
    /// the DRS_DEADTIME instead of FPGA TEMP
    u16  channel_mask          = 0;
    u16  stop_cell             = 0;
    /// RBPaddleID - component 
    u8   pid_ch12              = 0;
    /// RBPaddleID - component
    u8   pid_ch34              = 0;
    /// RBPaddleID - component
    u8   pid_ch56              = 0;
    /// RBPaddleID - component
    u8   pid_ch78              = 0;
    /// RBPaddleID - component
    u8   pid_ch_order          = 0;
    /// Reserved
    u8   rsvd1                 = 0;
    /// Reserved
    u8   rsvd2                 = 0;
    /// Reserved
    u8   rsvd3                 = 0;
    u16  fpga_temp             = 0;
    u32  timestamp32           = 0;
    u16  timestamp16           = 0;
    /// Store the drs_deadtime instead 
    /// of the fpga temperature
    bool deadtime_instead_temp = false;
    u16  drs_deadtime          = 0;

    RBEventHeader();
   
    static auto from_bytestream(const Vec<u8> &bytestream, u64 &pos)
      -> r::Result<RBEventHeader, IOError>;
    static auto parse_channel_mask(u16 ch_mask) -> std::tuple<bool, u16>;
  
    auto get_channels()             const -> Vec<u8>;
    auto set_channel_mask(u16 ch_mask)    -> void;
    auto get_channel_mask()         const -> u16;
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
    /// the combined timestamp 
    auto get_timestamp48()          const -> u64;
    /// string representation for printing
    auto to_string()                const -> std::string;
    auto to_bytestream()            const -> Vec<u8>;
  };
  
  std::ostream& operator<<(std::ostream& os, const gondola::RBEventHeader& rh);
}
