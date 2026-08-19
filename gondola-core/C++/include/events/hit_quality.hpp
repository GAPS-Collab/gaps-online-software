/// This file is part of gaps-online-software and published
/// under the GPLv3 license
// only ever include this header once per compilation unit,
// even if multiple files #include it
#pragma once

// std::format / std::formatter - needed for the formatter
// specialization at the bottom of this file
#include <format>
// std::ostringstream - used by the formatter to capture the
// output of operator<< into a string
#include <sstream>

// basic typedefs (u8, f32, Vec, ...) via tof_typedefs.h
#include "gondola.hpp"

// everything in gaps-online-software C++ lives in the
// gondola namespace
namespace gondola {

  struct TofHit;

  enum class HitQuality : u8 {
    // hit has not been classified (yet)
    Unknown           =  0,
    //$$$$$ Reserved for future use, nothing sets these yet.
    Mangled           =  1,
    MissingHit        =  2,
    //$$$$$
    LowPedPlusPedRMS  =  3, // abs(ped) > 1 and pedRMS > 2 (jeff's cut)
    LowRMSgrThree     =  4, // rms > 3 (box2 cut)
    LowElenaCuts      =  5, // cuts where really the analysis failed.
    Low               = 10, // general Low status

    MidPedRMS         = 11, // middle triangular region in peak vs rms for coincidence pulses (box1 cut)
    MidPos            = 12, // hit position too close to a paddle end
    MidPedRMSandPos   = 13, // MidPedRMS and MidPos both apply
    MidPosSat         = 14, // position issue on an otherwise-saturated pulse; ped cuts don't apply to saturated pulses
    Mid               = 20, // general Mid status

    HighSat           = 21, // High, but the pulse is saturated (> 600 mV)
    High              = 30, // general High status
    // between mid and high, really just techincally high hits
    // but could use some sort of recalibration...
    // like a noisy saturated pulse for example
    ReprocessHighSat  = 31,
    ReprocessHigh     = 40,
    // same as ReprocessHigh, but the pulse is also saturated (tot725 filled)
  };

  // Implemented in src/events.cxx
  auto classify_hit(const TofHit& hit) -> HitQuality;

  // Implemented in src/events.cxx
  std::ostream& operator<<(std::ostream& os, const gondola::HitQuality& quality);
}


template <>

struct std::formatter<gondola::HitQuality> : std::formatter<std::string> {
  constexpr auto parse(std::format_parse_context& ctx) {
      return ctx.begin();
  }

  // format() does the actual conversion of a HitQuality value
  // into characters written to the output (ctx.out())
  auto format(const gondola::HitQuality& quality, auto& ctx) const {
      // an in-memory output stream to write into
      std::ostringstream oss;
      // reuse operator<< (see above) so the printed text is the
      // same for std::cout and std::format
      oss << quality;
      // copy the accumulated string into the formatter's output
      return std::format_to(ctx.out(), "{}", oss.str());
  }
};
