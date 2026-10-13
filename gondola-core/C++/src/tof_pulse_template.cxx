/// This file is part of gaps-online-software and published 
/// under the GPLv3 license
#include <format>
#include "tof_pulse_template.hpp"
#include "io/parsers.h"

namespace g = gondola; 

using namespace result;

//----------------------------------------------------------

auto g::TofPulseTemplate::is_side_a() const noexcept -> bool { return side == SIDE_A; }

//----------------------------------------------------------

auto g::TofPulseTemplate::is_side_b() const noexcept -> bool { return side == SIDE_B; }

//----------------------------------------------------------

auto g::TofPulseTemplate::time_at(std::size_t index) const -> f32 { 
  if (index >= g::TofPulseTemplate::N_BINS)
    throw std::out_of_range("TofPulseTemplate::time_at: invalid index");
  return time_start + static_cast<f32>(index) * time_step;
}

//----------------------------------------------------------

auto g::TofPulseTemplate::time_axis() const noexcept -> std::array<f32, N_BINS> {
 std::array<f32, N_BINS> axis{};
 for (std::size_t i = 0; i < N_BINS; ++i) {
   axis[i] = time_start + static_cast<f32>(i) * time_step;
 }
 return axis;
}

//----------------------------------------------------------

auto g::TofPulseTemplate::from_bytestream(const Vec<u8> &stream, u64 &pos)
  -> Result<g::TofPulseTemplate, g::IOError> {
  u16 head = g::parse_u16(stream, pos);
  if (head != g::TofPulseTemplate::HEAD) {
    auto message = std::format("Decoding of HEAD failed! Got {} instead!", head);
    auto err = g::IOError(g::IOError::ErrorKind::WrongHeaderBytes, message);
    return Err(err);
  }
  g::TofPulseTemplate tpt;
  tpt.paddle_id     = g::parse_u8(stream, pos);
  tpt.side          = g::parse_u8(stream, pos);
  tpt.n_waveforms   = g::parse_u32(stream, pos);
  tpt.position_low  = g::parse_f32(stream, pos);
  tpt.position_high = g::parse_f32(stream, pos);
  tpt.charge_low    = g::parse_f32(stream, pos);
  tpt.charge_high   = g::parse_f32(stream, pos);
  tpt.time_start    = g::parse_f32(stream, pos);
  tpt.time_step     = g::parse_f32(stream, pos);
  for (f32& value : tpt.mean) {
    value = parse_f32(stream, pos); 
  }
  for (f32& value : tpt.rms) {
    value = parse_f32(stream, pos);
  }
  u16 tail = g::parse_u16(stream, pos);
  if (tail != g::TofPulseTemplate::TAIL) {
    auto message = std::format("Decoding of TAIL failed! Got {} instead!", tail);
    auto err = g::IOError(g::IOError::ErrorKind::WrongTailBytes, message);
    return Err(err);
  }
  return Ok(tpt);
}

//----------------------------------------------------------

auto g::TofPulseTemplate::to_bytestream() const -> Vec<u8> {
  Vec<u8> stream;
  stream.reserve(g::TofPulseTemplate::SIZE);
  g::bytestream_extend(stream, (u16)g::TofPulseTemplate::HEAD);
  stream.push_back(paddle_id);
  stream.push_back(side);
  g::bytestream_extend(stream, (u32)n_waveforms);
  g::bytestream_extend(stream, (f32)position_low);
  g::bytestream_extend(stream, (f32)position_high);
  g::bytestream_extend(stream, (f32)charge_low);
  g::bytestream_extend(stream, (f32)charge_high);
  g::bytestream_extend(stream, (f32)time_start);
  g::bytestream_extend(stream, (f32)time_step);
  for (auto const &value : mean) {
    g::bytestream_extend(stream, value);
  }
  for (auto const & value : rms) {
    g::bytestream_extend(stream, value);
  }
  g::bytestream_extend(stream, (u32)g::TofPulseTemplate::TAIL);
  return stream;
}

//----------------------------------------------------------

auto g::TofPulseTemplate::to_string() const -> std::string {
  std::string repr = "<TofPulseTemplate:";
  repr += std::format("\n paddle_id     : {}", paddle_id    );
  repr += std::format("\n side          : {}", side         );
  repr += std::format("\n n_waveforms   : {}", n_waveforms  );
  repr += std::format("\n position_low  : {}", position_low );
  repr += std::format("\n position_high : {}", position_high);
  repr += std::format("\n charge_low    : {}", charge_low   );
  repr += std::format("\n charge_high   : {}", charge_high  );
  repr += std::format("\n time_start    : {}", time_start   );
  repr += std::format("\n time_step     : {}", time_step    );
  repr += "\n ----- waveform [bins 30 - 35] -----\b mean: ";
  for (usize k=30;k<36;k++) {
    repr += std::format(" {} ", mean[k]); 
  } 
  repr += "\n rms : ";
  for (usize k=30;k<36;k++) {
    repr += std::format(" {} ", rms[k]); 
  } 
  repr += ">";
  return repr;
}

