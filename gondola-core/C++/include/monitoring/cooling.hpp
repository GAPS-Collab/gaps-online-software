//! This file is part of gaps-online-software and published 
//! under the GPLv3 license

#pragma once 

#include <memory>
#include "tof_typedefs.h"
#include "result/result.h"
#include "errors.hpp"
#include "packets/telemetry_packet.hpp"

namespace r = result;


namespace gondola {
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

