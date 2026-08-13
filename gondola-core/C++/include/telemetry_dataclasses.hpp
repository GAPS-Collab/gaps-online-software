#ifndef GO_TELEMETRY_DATACLASSES_H_INLCUDED
#define GO_TELEMETRY_DATACLASSES_H_INLCUDED

//! Bascially a re-write of some bfsw stuff to 
//! avoid pulling in the dependency
//!

#include <memory>
#include "tof_typedefs.h"
#include "result/result.h"
#include "errors.hpp"
#include "events/telemetry_event.hpp"
#include "packets/telemetry_packet.hpp"
#include "events/tracker_event.hpp"


namespace r = result;

namespace gondola {
  
  struct TrkHeader {
     static constexpr u16 SIZE = 17; 
     static constexpr u16 HEAD = 0x90eb;
     
     u16   sync;
     u16   crc;
     u8    sys_id;
     u8    packet_id;
     u16   length;
     u16   daq_count;
     u64   sys_time;
     u8    version;
    
     auto to_string() const -> std::string;
     
     static auto from_bytestream(Vec<u8> const &stream, usize &pos)
       -> r::Result<TrkHeader, IOError>;
  };
  
  
  struct TrkEventPacket {
    TelemetryPacketHeader  header;
    TrkHeader              daq_header;
    Vec<TrkEvent>          events;
    u16                    run_id;
    u8                     run_id_old;
    
    auto to_string() const -> std::string;
    static auto from_bytestream(Vec<u8> const &stream, usize &pos)
      -> r::Result<TrkEventPacket, IOError>;
  };
}

#endif
