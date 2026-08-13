/// This file is part of gaps-online-software and published 
/// under the GPLv3 license

#include <format>
#include "events/tracker_hit.hpp"

namespace g = gondola;

auto g::TrkHit::to_string() const -> std::string {
  std::string repr = "<TrackerHit:";
  repr += std::format("\n  Layer      : {}", layer);
  repr += std::format("\n  Row        : {}", row);
  repr += std::format("\n  Module     : {}", module);
  repr += std::format("\n  Channel    : {}", channel);
  repr += std::format("\n  ADC        : {}", adc);
  repr += std::format("\n  Oscillator : {}", oscillator);
  repr += std::format("\n  Energy     : {}", energy);
  return repr;
}

