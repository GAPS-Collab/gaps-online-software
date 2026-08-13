//! This file is part of gaps-online-software and published 
//! under the GPLv3 license

#pragma once 

#include "result/result.h"
#include "tof_typedefs.h"
#include "errors.hpp"
#include "version.h"
#include "events/event_status.hpp"
#include "events/event_quality.hpp"
#include "events/trigger.hpp"
#include "events/tracker_hit.hpp"
#include "events/tracker_event.hpp"
#include "events/tof_event_summary.hpp"
#include "packets/tof_packet.h"
#include "packets/telemetry_packet.hpp"
#include "tracker_meta.hpp"
#include "tof_meta.hpp"
#include "database.h"

namespace gondola {
  /// The actual merged event sent over telemetry 
  struct TelemetryEvent {
    TelemetryPacketHeader header   = TelemetryPacketHeader();
    u8              version        = 0;
    u8              flags0         = 0;
    u8              flags1         = 0;
    Vec<u8>         row_flags      = {};
    u64             creation_time  = 0;
    u32             event_id       = 0;
    u8              n_tof_hits     = 0;
    u16             n_trk_hits     = 0;
    Vec<TrkEvent>   tracker_events = {};  
    Vec<TrkHit>     trk_hits       = {};
    TofEventSummary tof_event      = TofEventSummary();
    Vec<u8>         raw_data       = {};
    TofMetaData     tof_meta;
    TrkMetaData     tracker_meta;
    Vec<u64>        tracker_oscillators = Vec<u64>(10,0) ;
  
    auto to_string() const -> std::string;

    static auto from_bytestream(Vec<u8> const &stream, usize &pos)
      -> r::Result<TelemetryEvent, IOError>;
    
    static auto from_telemetrypacket(TelemetryPacket const &packet) 
      -> r::Result<TelemetryEvent, IOError>;
  };  
}



