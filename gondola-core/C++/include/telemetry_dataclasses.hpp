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
   
   
   
   
   struct Cooling {
     /// size with packet header
     static constexpr u16 SIZE = 105; 
  
     TelemetryPacketHeader header;
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
      -> r::Result<Cooling, IOError>;   
  };
}

#endif
