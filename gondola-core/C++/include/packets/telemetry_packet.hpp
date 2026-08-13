/// This file is part of gaps-online-software and published 
/// under the GPLv3 license

#pragma once

#include "result/result.h"
#include "tof_typedefs.h"
#include "errors.hpp"
#include "database.h"

namespace r = result;

namespace gondola {
  
  enum class TelemetryPacketType : u8 {
    Unknown            = 0,
    SipGpsPosition     = 20,
    SipGpsTime         = 21,
    SipPressure        = 22,
    CardHKP            = 30,
    CoolingHK          = 40,
    PDUHK              = 50,
    Tracker            = 80,
    TrackerDAQCntr     = 81,
    GPS                = 82,
    TrkTempLeak        = 83,
    RPiHKP             = 89,
    BoringEvent        = 90,
    RBWaveform         = 91,
    AnyTofHK           = 92,
    GcuEvtBldSettings  = 93,
    GcuEvtBuilderStats = 94,
    TmP96              = 96,
    LabJackHK          = 100,
    LabjackSettings    = 101,
    HeatHVLVSettings   = 102,
    MagHK              = 108,
    GcuMon             = 110,
    PacketStats        = 111,
    TeleMainSettings   = 112,
    DecimationSettings = 113,
    SurvivalPacket     = 114,
    GcuMonHKAddendum   = 120,
    InterestingEvent   = 190,
    NoGapsTriggerEvent = 191,
    NoTofDataEvent     = 192,
    Ack                = 200,     
    RatePacket         = 219,
    AnyTrackerHK       = 255,
    // unknown/unused stuff
    TmP33              = 33,
    TmP34              = 34,
    TmP37              = 37,
    TmP38              = 38,
    TmP55              = 55,
    TmP64              = 64,
    //TmP92            = 92,
    TmP214             = 214,
  };

  auto bfsw_ptype_to_u8(TelemetryPacketType pt) -> u8;
  auto bfsw_ptype_to_str(TelemetryPacketType pt) -> std::string;

  struct TelemetryPacketHeader {
    static constexpr u16 SIZE = 13; 
    static constexpr u16 HEAD = 0x90eb;

    u16                  sync     {0};
    TelemetryPacketType  ptype    {TelemetryPacketType::Unknown};
    u32                  timestamp{0};
    u16                  counter  {0};
    u16                  length   {0};
    u16                  checksum {0};
  
    auto get_gcutime()   const -> f64;
    auto to_string()     const -> std::string;
    auto to_bytestream() const -> Vec<u8>;
    static auto from_bytestream(Vec<u8> const &stream, usize &pos)
      -> r::Result<TelemetryPacketHeader, IOError>;
  };
  
  struct TelemetryPacket {
    auto to_string() const -> std::string;
    auto is_event_packet() const -> bool;
    
    static auto from_bytestream(Vec<u8> const &stream, usize &pos) -> TelemetryPacket;
     
    TelemetryPacketHeader header;
    Vec<u8> payload;
    #ifdef BUILD_CXX_DB
    /// The map of all paddles. This is needed later on to look up properties 
    /// of the TOF paddles when we are unpacking events 
    TofPaddleMapPtr paddles;
    #endif 
  };
}

