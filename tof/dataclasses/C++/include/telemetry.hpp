//! Bascially a re-write of some bfsw stuff to 
//! avoid pulling in the dependency
//!

#include "tof_typedefs.h"

namespace Gaps {
  struct TelemetryHeader {
    static const u16 SIZE = 13; 
    static const u16 HEAD = 0x90eb;

    u16 sync     ;
    u8  ptype    ;
    u32 timestamp;
    u16 counter  ;
    u16 length   ;
    u16 checksum ;
  
    f64 get_gcutime();
    std::string to_string();
    static TelemetryHeader from_bytestream(Vec<u8> const &stream, usize &pos);
  };

  struct TelemetryPacket {
    TelemetryHeader header;
    Vec<u8> payload;
    std::string to_string();
    static TelemetryPacket from_bytestream(Vec<u8> const &stream, usize &pos);
  };
}

  //pub struct MergedEvent {
  //  pub header              : TelemetryHeader,
  //  pub creation_time       : u64,
  //  pub event_id            : u32,
  //  pub tracker_events      : Vec<TrackerEvent>,
  //  /// in case this is version 2, we don't have
  //  /// tracker_events, but new-style tracker hits
  //  /// (TrackerHitV2)
  //  pub tracker_hitsv2      : Vec<TrackerHitV2>,
  //  pub tracker_oscillators : Vec<u64>,
  //  pub tof_data            : Vec<u8>,
  //  pub raw_data            : Vec<u8>,
  //  pub flags0              : u8,
  //  pub flags1              : u8,
  //  pub version             : u8
  //}

