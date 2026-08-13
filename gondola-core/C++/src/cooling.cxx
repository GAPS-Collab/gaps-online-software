//! This file is part of gaps-online-software and published 
//! under the GPLv3 license

#include <format>
#include "result/result.h"
#include "errors.hpp"
#include "io/parsers.h"

#include "monitoring/cooling.hpp"

namespace g = gondola;

using namespace result;

auto g::Cooling::to_string() const -> std::string {
  std::string repr = "<Cooling";
  repr += std::format("\n  frame_counter : {}", frame_counter);
  repr += std::format("\n  status_1      : {}", status_1);
  repr += std::format("\n  status_2      : {}", status_2);
  repr += std::format("\n  rx_byte_num   : {}", rx_byte_num);
  repr += std::format("\n  rx_cmd_num    : {}", rx_cmd_num);
  repr += std::format("\n  last_cmd      : {}", last_cmd);
  repr += std::format("\n  rsv_t         : {}", rsv_t);
  repr += std::format("\n  rh_don        : {}", rh_on);
  repr += std::format("\n  rh_off        : {}", rh_off);
  repr += std::format("\n  fpga_board_v_i: {}", fpga_board_v_in);
  repr += std::format("\n  fpga_board_i_i: {}", fpga_board_i_in);
  repr += std::format("\n  fpga_board_t  : {}", fpga_board_t);
  repr += std::format("\n  fpga_board_p  : {}", fpga_board_p);
  //repr += std::format(:array<u16, 64);
  repr += std::format("\n  sh_current    : {}", sh_current);
  repr += std::format("\n  rh_current    : {}", rh_current);
  repr += std::format("\n  pw_board1_t   : {}", pw_board1_t);
  repr += std::format("\n  pw_board2_t   : {}", pw_board2_t);
  repr += std::format("\n  sh1_time_left : {}", sh1_time_left);
  repr += std::format("\n  sh2_time_left : {}", sh2_time_left);
  repr += std::format("\n  sh3_time_left : {}", sh3_time_left);
  return repr;
}

auto g::Cooling::from_bytestream(Vec<u8> const &stream, usize &pos) -> Result<Cooling, g::IOError> {
  auto cln = Cooling();
  if (stream.size() - pos < g::Cooling::SIZE) {
    std::string message = std::format("Stream is too short for a cooling packet. We got a streamof size {} when expectinog {} bytes!", stream.size() - pos, g::Cooling::SIZE);
    auto err = g::IOError(g::IOError::ErrorKind::WrongDelimiter, message);
    return Err(err);
  }
  auto start_byte = g::parse_u8(stream, pos);
  if (start_byte != 0x1e) {
    std::string message = std::format("Start byte for cooling packet incorrect! Got {} instead of 0x1e", start_byte);
    auto err = g::IOError(g::IOError::ErrorKind::WrongDelimiter, message);
    return Err(err);
  } 
  cln.rtd.fill(0xffff);
  cln.frame_counter   = 0xffffff & g::parse_u32(stream, pos);
  pos -= 1;
  cln.status_1        = g::parse_u8(stream, pos);  
  cln.status_2        = g::parse_u8(stream, pos);  
  cln.rx_byte_num     = g::parse_u8(stream, pos);  
  cln.rx_cmd_num      = g::parse_u8(stream, pos);  
  cln.last_cmd        = 0xffffffffffffff & g::parse_u64(stream, pos); 
  pos -= 1;
  cln.rsv_t           = g::parse_u16(stream, pos);
  cln.rh_on           = g::parse_u16(stream, pos);
  cln.rh_off          = g::parse_u16(stream, pos);
  cln.fpga_board_v_in = g::parse_u16(stream, pos);
  cln.fpga_board_i_in = g::parse_u16(stream, pos);
  pos += 2;
  cln.fpga_board_t    = g::parse_u16(stream, pos);
  cln.fpga_board_p    = g::parse_u16(stream, pos);
  pos += 6;
  //std::cout << cln.rtd.size() << std::endl;
  for(usize k=0; k < cln.rtd.size(); k++) {
    cln.rtd[k] = g::parse_u16(stream, pos); 
  }
  cln.sh_current    = g::parse_u16(stream, pos);
  cln.rh_current    = g::parse_u16(stream, pos);
  cln.pw_board1_t   = g::parse_u16(stream, pos);
  cln.pw_board2_t   = g::parse_u16(stream, pos);
  cln.sh1_time_left = g::parse_u16(stream, pos);
  cln.sh2_time_left = g::parse_u16(stream, pos);
  cln.sh3_time_left = g::parse_u16(stream, pos);
  pos += 2; //spare
  auto stop_byte = g::parse_u8(stream,pos);
  if (stop_byte != 0x0a) {
    std::string message = std::format("Stop byte for cooling packet incorrect! Got {} instead of 0x0a", stop_byte);
    auto err = g::IOError(g::IOError::ErrorKind::WrongDelimiter, message);
    return Err(err);
  } 
  return Ok(cln);
}

