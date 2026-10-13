// This file is part of gaps-online-software and published 
// under the GPLv3 license

#include <cmath>

#include "spdlog/spdlog.h"

#include "events/tof_hit.hpp"
#include "io/parsers.h" 

namespace g = gondola;
using namespace result;

//-----------------------------------

void g::TofHit::set_paddle_len(f32 length) {
  paddle_len = length;
}

//-----------------------------------

#ifdef BUILD_CXXDB
auto g::TofHit::set_paddle(const g::TofPaddle& paddle) -> void {
  paddle_len    = paddle.length;
  coax_cbl_time = paddle.coax_cable_time; 
  hart_cbl_time = paddle.harting_cable_time;
}

//-----------------------------------

auto g::TofHit::get_phase_delay() const -> f32 {
  f32 freq = 20e6;
  f32 phase_fixed = phase;
  auto PI = std::numbers::pi_v<f32>;
  while (phase_fixed < -PI/2.0) {
    phase_fixed += PI/2.0;
  }
  while (phase_fixed > PI/2.0) {
    phase_fixed -= PI/2.0;
  }
  auto phase_delay = (phase_fixed/(2.0*PI*freq))*1.0e9;
  return phase_delay;
}

//-----------------------------------

auto g::TofHit::get_cable_delay() const -> f32 {
  return hart_cbl_time - coax_cbl_time;
}

//-----------------------------------

auto g::TofHit::get_t0()          const -> f32 {
  return get_t0_relative() + get_cable_delay() + get_phase_delay();
}
#endif

//-----------------------------------

f32 g::TofHit::get_time_a() const {
  return time_a_f32;
}

//-----------------------------------

f32 g::TofHit::get_time_b() const {
  return time_b_f32;
}

//-----------------------------------

f32 g::TofHit::get_peak_a() const {
  return peak_a_f32;
}

//-----------------------------------

f32 g::TofHit::get_peak_b() const {
  return peak_b_f32;
}

//-----------------------------------

f32 g::TofHit::get_charge_a() const {
  return charge_a_f32;
}

//-----------------------------------

f32 g::TofHit::get_charge_b() const {
  return charge_b_f32;
}

//-----------------------------------

// time-over-threshold variables 
auto g::TofHit::get_tot_low_a() const -> f32 {
  return tot_low_a; 
}

//-----------------------------------

auto g::TofHit::get_tot_low_b() const -> f32 {
  return tot_low_b; 
}

//-----------------------------------

auto g::TofHit::get_tot_high_a() const -> f32 {
  return tot_high_a;
}

//-----------------------------------

auto g::TofHit::get_tot_high_b() const -> f32 {
  return tot_high_b; 
}

//-----------------------------------

auto g::TofHit::get_tot_slp_low_a() const -> f32 {
  return tot_slp_low_a;
}

//-----------------------------------

auto g::TofHit::get_tot_slp_low_b() const -> f32 {
  return tot_slp_low_b;
}

//-----------------------------------

auto g::TofHit::get_tot_slp_high_a() const -> f32 {
  return tot_slp_high_a;
}

//-----------------------------------

auto g::TofHit::get_tot_slp_high_b() const -> f32 {
  return tot_slp_high_b;
}

//-----------------------------------

f32 g::TofHit::get_x_pos() const {
  return (time_a_f32 - get_t0_relative())*C_LIGHT_PADDLE*10.0; // 10 for cm->mm 
}

//-----------------------------------

auto g::TofHit::obeys_causality() const -> bool {
  return 10.0*paddle_len/(10.0*C_LIGHT_PADDLE) - std::abs(time_a_f32 - time_b_f32) > 0.0
  && get_t0_relative() > 0.0;
}

//-----------------------------------

f32 g::TofHit::get_t0_relative() const {
  // FIXME - we should just consistently make the paddle len in mm!
  return 0.5*(time_a_f32 + time_b_f32 - (10.0*paddle_len/(10.0*C_LIGHT_PADDLE)));
}

//-----------------------------------

/// combine the slow timestamp with 
/// the fast to get the full
auto g::TofHit::get_timestamp48() const -> f64 {
  f64 ts48 = timestamp16 << 16 | timestamp32;
  return ts48;
}

//-----------------------------------

auto g::TofHit::get_edep_noatt() const -> f32 {
  return (1.29/34.3)*(get_peak_a() + get_peak_b()) / 2.0;
}

//-----------------------------------

#ifdef BUILD_CXXDB
/// simple calculation based on Philip's 
/// attenuation function as used in pgaps
auto g::TofHit::get_edep() const -> f32 {
  f32 x0    = get_x_pos();
  f32 att_a = (std::exp(3.9-0.00126*( x0            ))+22.15)/ (std::exp(3.9)+22.15);
  f32 att_b = (std::exp(3.9-0.00126*(10.0*paddle_len - x0))+22.15)/ (std::exp(3.9)+22.15);
  f32 edep  = 0.0159 * (get_peak_a()/att_a + get_peak_b()/att_b) / 2.; // vertical muon peak @ 0.97 MeV
  return edep; 
}

//-----------------------------------

auto g::TofHit::get_edep_birk() const -> f32 {
  f32 x0    = get_x_pos();
  //f32 edep  = 0.0159 * (get_peak_a()/att_a + get_peak_b()/att_b) / 2.; // vertical muon peak @ 0.97 MeV
  f32 att_a = (std::exp(3.9-0.00126*( x0            ))+22.15)/ (std::exp(3.9)+22.15);
  f32 att_b = (std::exp(3.9-0.00126*(10.0*paddle_len - x0))+22.15)/ (std::exp(3.9)+22.15);
  f32 v  = (get_peak_a()/att_a + get_peak_b()/att_b) / 2.0; 
  f32 edep  = -1000.0*v/(21.0*v - 35260.0);
  return edep; 
}
#endif

//-----------------------------------

auto g::TofHit::to_bytestream() const -> Vec<u8> {
  Vec<u8> stream;
  stream.reserve(SIZE);
  bytestream_extend(stream, HEAD);
  stream.push_back(paddle_id); 
  bytestream_extend_f32tof16(stream, time_a_f32);
  bytestream_extend_f32tof16(stream, time_b_f32);
  bytestream_extend_f32tof16(stream, peak_a_f32);
  bytestream_extend_f32tof16(stream, peak_b_f32);
  bytestream_extend_f32tof16(stream, charge_a_f32);
  bytestream_extend_f32tof16(stream, charge_b_f32);
  bytestream_extend_f32tof16(stream, tot_low_a);
  bytestream_extend_f32tof16(stream, baseline_a);
  bytestream_extend_f32tof16(stream, baseline_a_rms);
  bytestream_extend_f32tof16(stream, phase);
  u8 quality_version = (u8)quality  | (u8)version;
  stream.push_back(quality_version);
  bytestream_extend_f32tof16(stream, baseline_b);
  bytestream_extend_f32tof16(stream, baseline_b_rms);
  bytestream_extend_f32tof16(stream, tot_low_b);
  bytestream_extend_f32tof16(stream, tot_high_a);
  bytestream_extend_f32tof16(stream, tot_high_b);
  bytestream_extend_f32tof16(stream, tot_slp_low_a);
  bytestream_extend_f32tof16(stream, tot_slp_low_b);
  bytestream_extend_f32tof16(stream, tot_slp_high_a);
  bytestream_extend_f32tof16(stream, tot_slp_high_b);
  bytestream_extend(stream, g::TofHit::TAIL);
  return stream;
}

//-----------------------------------

auto g::TofHit::from_bytestream(const Vec<u8> &bytestream, u64 &pos) 
 -> r::Result<g::TofHit,g::IOError> {
 auto hit = g::TofHit();
 u16 maybe_header = g::parse_u16(bytestream, pos);
 if (maybe_header != TofHit::HEAD) {
   auto message = std::format("Decoding of HEAD failed! Got {} instead!", maybe_header);
   auto err = g::IOError(g::IOError::ErrorKind::WrongHeaderBytes, message);
   return Err(err);
 }
 // UPDATE - get version byte first!
 u64 ver_pos            = pos + 21; // version byte is at position 23
 if (bytestream.size() <= ver_pos) {
   auto message = std::format("Currently, reading older data without the hit time-over-threshold variables is not supported!");
   auto err = g::IOError(g::IOError::ErrorKind::UnsupportedProtocolVersion, message);
   return Err(err);
 }
 u8 quality_version_u8     = g::parse_u8(bytestream, ver_pos);
 hit.quality               = static_cast<HitQuality>(quality_version_u8 & 0x3f);
 hit.version               = (g::ProtocolVersion)(quality_version_u8 & 0xc0);
 //u8  version        = g::parse_u8(bytestream, ver_pos);
 //hit.version        = (g::ProtocolVersion) version;
 hit.paddle_id      = bytestream[pos]; pos+=1;
 hit.time_a_f32     = g::parse_f16(bytestream, pos); 
 hit.time_b_f32     = g::parse_f16(bytestream, pos); 
 hit.peak_a_f32     = g::parse_f16(bytestream, pos); 
 hit.peak_b_f32     = g::parse_f16(bytestream, pos); 
 hit.charge_a_f32   = g::parse_f16(bytestream, pos); 
 hit.charge_b_f32   = g::parse_f16(bytestream, pos); 
 // this is one of the new saturation variables 
 hit.tot_low_a      = g::parse_f16(bytestream, pos);
 //hit.charge_min_i   = g::parse_u16(bytestream, pos); 
 hit.baseline_a     = g::parse_f16(bytestream, pos);
 hit.baseline_a_rms = g::parse_f16(bytestream, pos);
 hit.phase          = g::parse_f16(bytestream, pos);
 pos += 1; // skip version - it has already been read
 hit.baseline_b     = g::parse_f16(bytestream, pos);
 hit.baseline_b_rms = g::parse_f16(bytestream, pos);
 // new saturation variables 
 hit.tot_low_b      = g::parse_f16(bytestream, pos);
 hit.tot_high_a     = g::parse_f16(bytestream, pos);
 hit.tot_high_b     = g::parse_f16(bytestream, pos);
 hit.tot_slp_low_a  = g::parse_f16(bytestream, pos);
 hit.tot_slp_low_b  = g::parse_f16(bytestream, pos);
 hit.tot_slp_high_a = g::parse_f16(bytestream, pos);
 hit.tot_slp_high_b = g::parse_f16(bytestream, pos);
 //------------------------------
 u16 tail = g::parse_u16(bytestream, pos);
 if (tail != TAIL) {
   std::cout << hit << std::endl;
   SPDLOG_ERROR("VERSION {}", quality_version_u8 & 0xc0); 
   SPDLOG_ERROR("TofHit TAIL signature {} is incorrect!", tail);
 }
 return Ok(hit); 
}

//-----------------------------------

auto g::TofHit::to_string() const -> std::string {
  std::string vstr = g::pversion_to_string(version);
  std::string repr = std::format("<TofHit ({})", vstr);
  //repr += std::format("\n -- format test {:.2f}", get_time_a() );
  repr += "\n  paddle ID         : "     + std::to_string(paddle_id         );
  repr += "\n  paddle len        : "     + std::to_string(paddle_len         );
  if (version == g::ProtocolVersion::Unknown) {
    repr += "\n  timestamp32       : "     + std::to_string(timestamp32       );
    repr += "\n  timestamp16       : "     + std::to_string(timestamp16       );
    repr += "\n   |-> timestamp48  : "     + std::to_string(get_timestamp48() ); 
  }
  repr += "\n  _________";
  repr += "\n  ##  Peak:";
  repr += std::format("\n  >> time       A | B  : {:.2f} {:.2f}", get_time_a(), get_time_b());
  //repr += "\n  >>  time   A | B  : "     + std::to_string(get_time_a()      )
  //     +  " " + std::to_string(get_time_b());
  repr += std::format("\n  >> height     A | B  : {:.2f} {:.2f}",get_peak_a(), get_peak_b());
  repr += std::format("\n  >> charge     A | B  : {:.2f} {:.2f}",get_charge_a(), get_charge_b());
  repr += std::format("\n  >> baseline   A | B  : {:.2f} {:.2f}",baseline_a, baseline_b);
  repr += std::format("\n  >> base. rms  A | B  : {:.2f} {:.2f}",baseline_a_rms, baseline_b_rms);
  // saturation variabes 
  repr += "\n  ** time over threshold information";
  repr += std::format("\n  >> Lo TOT     A | B  : {:.2f} {:.2f}",get_tot_low_a(), get_tot_low_b());  
  repr += std::format("\n  >> Hi TOT     A | B  : {:.2f} {:.2f}",get_tot_high_a(), get_tot_high_b());  
  repr += std::format("\n  >> Lo Slope   A | B  : {:.2f} {:.2f}",get_tot_slp_low_a(), get_tot_slp_low_b()); 
  repr += std::format("\n  >> Hi Slope   A | B  : {:.2f} {:.2f}",get_tot_slp_high_a(), get_tot_slp_high_b());  
  // -------------------
  
  #ifdef BUILD_CXXDB
  repr += "\n  >> t0            : "     + std::to_string(get_t0()       );
  #endif
  repr += ">";
  return repr;
}


