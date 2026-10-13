/// This file is part of gaps-online-software and published 
/// under the GPLv3 license

#pragma once

#include <array>
#include <cstddef>
#include <cstdint>
#include <vector>

#include "spdlog/spdlog.h"
#include "spdlog/cfg/env.h"

#include "result/result.h"

#include "gondola_typedefs.hpp"
#include "errors.hpp"

namespace gondola { 

  /// A TOF pulse template. Positions are measured from side A [mm],
  /// charge is in pC, time in ns, and voltage in mV.
  struct TofPulseTemplate {
    static constexpr usize N_BINS = 108;
    static constexpr u8    SIDE_A = 0;
    static constexpr u8    SIDE_B = 1;
    static constexpr u16   HEAD = 0xAAAA;
    static constexpr u16   TAIL = 0x5555;
    static constexpr usize SIZE =
        2 + 1 + 1 + 4 + 6 * 4 + 2 * N_BINS * 4 + 2; // 898 bytes
  
    u8  paddle_id           = 0;
    u8  side                = SIDE_A;
    u32 n_waveforms         = 0;
    f32 position_low        = 0.0f;
    f32 position_high       = 0.0f;
    f32 charge_low          = 0.0f;
    f32 charge_high         = 0.0f;
    f32 time_start          = 0.0f;
    f32 time_step           = 1.0f;
    std::array<f32, N_BINS> mean {};
    std::array<f32, N_BINS> rms {};
  
    auto is_side_a() const noexcept -> bool;
    auto is_side_b() const noexcept -> bool;
  
    /// Throws std::out_of_range if index >= N_BINS.
    auto time_at(std::size_t index) const -> f32;
    auto time_axis() const noexcept -> std::array<f32, N_BINS>;
  
    static auto from_bytestream(const Vec<u8> &bytestream, u64 &pos)
      -> result::Result<TofPulseTemplate, IOError>;
  
    auto to_bytestream() const -> Vec<u8>;
  
    auto to_string() const -> std::string;
  };  

  typedef std::array<std::vector<TofPulseTemplate>, 2> PulseTemplatesBySide;
}
