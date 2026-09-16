/// This file is part of gaps-online-software and published 
/// under the GPLv3 license
#pragma once 

#include <cstdint>
#include <vector>

#include "result/result.h"
#include "tof_typedefs.h"
#include "errors.hpp"
#include "packets/telemetry_packet.hpp"
#include "events/tracker_event.hpp"

namespace r = result;

namespace gondola {
  struct TrackerHeader {
    static constexpr u8  SIZE = 17;
    
    u16 sync      ;    
    u16 crc       ;
    u8  sys_id    ;
    u8  packet_id ;
    u16 length    ;
    u16 daq_count ;
    u64 sys_time  ;
    u8  version   ;

    auto to_string() const -> std::string;

    static auto from_bytestream(const Vec<u8> &bytestream, u64 &pos) -> r::Result<TrackerHeader, IOError>; 
  };

  struct TrackerDAQEventPacket {
    TelemetryPacketHeader header    ;
    TrackerHeader         daq_header;
    Vec<TrkEvent>         events    ;
    u16 run_id     ;
    u8  run_id_old ;
    // not serialized 
    /// internal counter for number of tracker hits in 
    /// this event 
    u16 n_hits     ;
    
    auto to_string() const -> std::string;
    
    static auto from_bytestream(const Vec<u8> &stream, u64 &pos) -> r::Result<TrackerDAQEventPacket, IOError>; 
  };
  
  std::ostream& operator<<(std::ostream& os, const TrackerHeader& pck);
  
  std::ostream& operator<<(std::ostream& os, const TrackerDAQEventPacket& pck);
}

