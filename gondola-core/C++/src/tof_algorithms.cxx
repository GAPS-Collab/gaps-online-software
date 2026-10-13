/// This file is part of gaps-online-software and published 
/// under the GPLv3 license
#include "tof_algorithms.hpp"

#include "spdlog/spdlog.h"
#include "spdlog/cfg/env.h"

namespace g = gondola;

using namespace result;

[[nodiscard("CFD time extraction can fail")]]
auto g::find_cfd_simple_time(
    std::span<const f64> voltages,
    std::span<const f64> times_ns,
    f64 cfd_fraction,
    usize peak_begin,
    usize peak_end
) -> r::Result<f64, g::AnalysisError> {
  //using CfdResult = result::Result<f64, CfdError>;

  // Validate input arrays and CFD fraction.
  if (voltages.size() != times_ns.size() ||
    voltages.empty() ||
    !std::isfinite(cfd_fraction) ||
    cfd_fraction <= 0.0 ||
    cfd_fraction >= 1.0) {
    spdlog::error("Invalid Input. CFD fraction has to be in 0 <= fraction <= 1.0");
    auto message = std::format("Invalid Input. CFD fraction has to be in [0,1.0]");
    SPDLOG_ERROR(message);  
    auto err = g::AnalysisError(g::AnalysisError::ErrorKind::InvalidInput, message);
    return Err(err);
  }

  // The peak search uses the half-open interval
  // [peak_begin, peak_end).
  if (peak_begin >= peak_end ||
    peak_end > voltages.size()) {
    auto message = std::format("Invalid peak range!");
    SPDLOG_ERROR(message);  
    auto err = g::AnalysisError(g::AnalysisError::ErrorKind::InvalidPeakRange, message);
    return Err(err);
  }

  const auto peak_it = std::max_element(voltages.begin() + peak_begin, voltages.begin() + peak_end);
  const auto peak_idx = static_cast<usize>(peak_it - voltages.begin());

  // The peak amplitude estimate needs both neighbors.
  if (peak_idx == 0 || peak_idx + 1 >= voltages.size()) {
    auto message = std::format("Invalid peak range!");
    SPDLOG_ERROR(message);  
    auto err = g::AnalysisError(g::AnalysisError::ErrorKind::InvalidPeakRange, message);
    return Err(err);
  }

  // Original algorithm: average three samples around
  // the maximum and take the requested fraction.
  const f64 peak_average = (voltages[peak_idx - 1] + voltages[peak_idx] + voltages[peak_idx + 1]) / 3.0;
  const f64 threshold = std::abs(cfd_fraction * peak_average);

  // Search backward, up to 10 samples before peak_begin.
  // The original algorithm uses this same search boundary.
  const std::size_t search_begin = peak_begin > 10 ? peak_begin - 10 : 0;

  for (std::size_t i = peak_idx; i > search_begin; --i) {
    const f64 v0 = std::abs(voltages[i]);

    if (v0 >= threshold) {
      continue;
    }

    // i is below threshold; i+1 is the next sample.
    const f64 v1 = std::abs(voltages[i + 1]);

    if (v1 == v0) {
      return Ok(times_ns[i]);
    }

    // Linear interpolation between the two samples.
    const f64 fraction = (threshold - v0) / (v1 - v0);
    const f64 cfd_time = times_ns[i] + fraction * (times_ns[i + 1] - times_ns[i]);
    return Ok(cfd_time);
  }
  auto message = std::format("No Threshold crossing found!");
  SPDLOG_ERROR(message);  
  auto err = g::AnalysisError(g::AnalysisError::ErrorKind::NoThresholdCrossing, message);
  return Err(err);
}

