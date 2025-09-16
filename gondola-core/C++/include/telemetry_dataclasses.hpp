#ifndef GO_TELEMETRY_DATACLASSES_H_INLCUDED
#define GO_TELEMETRY_DATACLASSES_H_INLCUDED

//! Bascially a re-write of some bfsw stuff to 
//! avoid pulling in the dependency
//!

#include "tof_typedefs.h"
#include "result/result.h"
#include "events.h"
#include "errors.hpp"

namespace g = Gaps;
namespace r = result;

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

    auto bfsw_ptype_to_u8(BfswPacketType pt) -> u8;
    auto bfsw_ptype_to_str(BfswPacketType pt) -> std::string;

    struct PacketHeader {
      static constexpr u16 SIZE = 13; 
      static constexpr u16 HEAD = 0x90eb;

      u16             sync     {0};
      BfswPacketType  ptype    {BfswPacketType::Unknown};
      u32             timestamp{0};
      u16             counter  {0};
      u16             length   {0};
      u16             checksum {0};
    
      auto get_gcutime() -> f64;
      auto to_string() const -> std::string;
      static auto from_bytestream(Vec<u8> const &stream, usize &pos)
        -> r::Result<PacketHeader, g::IOError>;
    };

    struct Packet {
      PacketHeader header;
      Vec<u8> payload;
      auto to_string() const -> std::string;
      static auto from_bytestream(Vec<u8> const &stream, usize &pos) -> Packet;
    };

    struct TrkHit {
      // using i32 here makes no sense in my eyes, but I defer to 
      // bfsw
      i32 layer          {-1};
      i32 row            {-1};
      i32 module         {-1};
      i32 channel        {-1};
      i32 adc            {-1};
      i64 oscillator     {-1};
      f64 energy         {0};
    
      auto to_string() const -> std::string;
    };
   
   struct TrkEvent {
      u8          layer;
      u8          flags1;
      u32         event_id; 
      u32         event_time;
      Vec<TrkHit> hits;

      auto to_string() const -> std::string;
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
      
      static auto from_bytestream(Vec<u8> const &stream, usize &pos) -> TofMetaData;
      auto to_string() const -> std::string;
    };
     
    struct TrkCalibratedHit {
      u16 strip_id;
      u16 adc;
      //calibrated_hit(uint16_t strip_id, uint16_t adc) : strip_id(strip_id), adc(adc) {}
     };
     
    struct TrkMetaData {
      u64 num_hits {0};
      u64 row_flags {0};
      f64 total_energy {0};
      Vec<TrkCalibratedHit> calibrated_hits;
      //std::array<uint64_t,8> oscillators = {0,0,0,0,0,0,0,0};
     };
     
     struct Cooling {
       /// size with packet header
       static constexpr u16 SIZE = 105; 
  
       PacketHeader header;
       u32 frame_counter   {0xffffffff};
       u8  status_1        {0xff};
       u8  status_2        {0xff};
       u8  rx_byte_num     {0xff};
       u8  rx_cmd_num      {0xff};
       u64 last_cmd        {0xffffffffffffffff};
       u16 rsv_t           {0xffff};
       u16 rh_on           {0xffff};
       u16 rh_off          {0xffff};
       u16 fpga_board_v_in {0xffff};
       u16 fpga_board_i_in {0xffff};
       u16 fpga_board_t    {0xffff};
       u16 fpga_board_p    {0xffff};
       std::array<u16, 64> rtd;
       u16 sh_current      {0xffff};
       u16 rh_current      {0xffff};
       u16 pw_board1_t     {0xffff};
       u16 pw_board2_t     {0xffff};
       u16 sh1_time_left   {0xffff};
       u16 sh2_time_left   {0xffff};
       u16 sh3_time_left   {0xffff};
       
       auto to_string() const -> std::string;
       
       static auto from_bytestream(Vec<u8> const &stream, usize &pos)
        -> r::Result<Cooling, g::IOError>;
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
      
      PacketHeader    header;
      u8              version = 0;
      u8              flags0  = 0;
      u8              flags1  = 0;
      Vec<u8>         row_flags;
      u64             creation_time = 0;
      u32             event_id      = 0;
      u8              n_tof_hits    = 0;
      u16             n_trk_hits    = 0;
      Vec<TrkEvent>   tracker_events;  
      Vec<TrkHit>     trk_hits;
      TofEventSummary tof_event;
      Vec<u8>         raw_data;
      TofMetaData     tof_meta;
      TrkMetaData     tracker_meta;
      Vec<u64>        tracker_oscillators = Vec<u64>(10,0) ;
    
      auto to_string() const -> std::string;
      static auto from_bytestream(Vec<u8> const &stream, usize &pos)
        -> r::Result<MergedEvent, g::IOError>;
    };
  }
}

#endif
