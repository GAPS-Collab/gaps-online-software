#ifndef GO_TELEMETRY_DATACLASSES_H_INLCUDED
#define GO_TELEMETRY_DATACLASSES_H_INLCUDED

//! Bascially a re-write of some bfsw stuff to 
//! avoid pulling in the dependency
//!

#include "tof_typedefs.h"

namespace Gaps {
  namespace Telemetry {
    
    enum class BfswPacketType : u8 {
      Unknown            = 0,
      CardHKP            = 30,
      CoolingHK          = 40,
      PDUHK              = 50,
      Tracker            = 80,
      TrackerDAQCntr     = 81,
      GPS                = 82,
      TrkTempLeak        = 83,
      BoringEvent        = 90,
      RBWaveform         = 91,
      AnyTofHK           = 92,
      GcuEvtBldSettings  = 93,
      LabJackHK          = 100,
      MagHK              = 108,
      GcuMon             = 110,
      InterestingEvent   = 190,
      NoGapsTriggerEvent = 191,
      NoTofDataEvent     = 192,
      Ack                = 200,     
      AnyTrackerHK       = 255,
      // unknown/unused stuff
      TmP33              = 33,
      TmP34              = 34,
      TmP37              = 37,
      TmP38              = 38,
      TmP55              = 55,
      TmP64              = 64,
      //TmP92            = 92,
      TmP96              = 96,
      TmP214             = 214,
    };

    u8 bfsw_ptype_to_u8(BfswPacketType pt);
    std::string bfsw_ptype_to_str(BfswPacketType pt);

    struct PacketHeader {
      static const u16 SIZE = 13; 
      static const u16 HEAD = 0x90eb;

      u16             sync     {0};
      BfswPacketType  ptype    {BfswPacketType::Unknown};
      u32             timestamp{0};
      u16             counter  {0};
      u16             length   {0};
      u16             checksum {0};
    
      f64 get_gcutime();
      std::string to_string();
      static PacketHeader from_bytestream(Vec<u8> const &stream, usize &pos);
    };

    struct Packet {
      PacketHeader header;
      Vec<u8> payload;
      std::string to_string();
      static Packet from_bytestream(Vec<u8> const &stream, usize &pos);
    };

    struct TrkHit {
      // this is stupid
      i32 layer          {-1};
      i32 row            {-1};
      i32 module         {-1};
      i32 channel        {-1};
      i32 adc            {-1};
      i64 oscillator     {-1};
      f64 energy         {0};
    
      std::string to_string();
    };
   
   struct TrkEvent {
      u8          layer;
      u8          flags1;
      u32         event_id; 
      u32         event_time;
      Vec<TrkHit> hits;

      std::string to_string();
   };
     
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
      
      static TofMetaData from_bytestream(Vec<u8> const &stream, usize &pos);
      std::string to_string();
    };
     
    struct TrkCalibratedHit {
      uint16_t strip_id;
      uint16_t adc;
      //calibrated_hit(uint16_t strip_id, uint16_t adc) : strip_id(strip_id), adc(adc) {}
     };
     
    struct TrkMetaData {
      u64 num_hits {0};
      u64 row_flags {0};
      f64 total_energy {0};
      Vec<TrkCalibratedHit> calibrated_hits;
      //std::array<uint64_t,8> oscillators = {0,0,0,0,0,0,0,0};
     };

    /// The actual merged event sent over telemetry 
    struct MergedEvent {
     // bfsw::header header;
     // uint8_t flags0 {0};
     // uint64_t row_flags {0};
     // uint32_t event_id {0xffffffff};
     // uint8_t num_tof_hits {0};
     // std::vector<uint8_t> tof_data;
     // std::vector<tracker_hit> tracker_hits;
     // std::vector<uint64_t> tracker_oscillators;
      
      PacketHeader  header;
      u8            version;
      u8            flags0;
      u8            flags1;
      Vec<u8>       row_flags;
      u64           creation_time;
      u32           event_id;
      u8            n_tof_hits;
      u16           n_trk_hits;
      Vec<TrkEvent> tracker_events;  
      Vec<TrkHit>   trk_hits;
      Vec<u8>       tof_data;
      Vec<u8>       raw_data;
      TofMetaData   tof_meta;
      TrkMetaData   tracker_meta;
      Vec<u64>      tracker_oscillators {10,0};
    
      std::string to_string();
      static MergedEvent from_bytestream(Vec<u8> const &stream, usize &pos);
    };
  }
}

#endif
