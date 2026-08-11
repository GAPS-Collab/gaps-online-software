/// This file is part of gaps-online-software and published 
/// under the GPLv3 license
#pragma once 

#include <format>
  
namespace gondola {
  
  enum class LTBThreshold : u8 {
    NoHit   = 0,
    /// First threshold, 40mV, about 0.75 minI
    Hit     = 1,
    /// Second threshold, 32mV (? error in doc ?, about 2.5 minI
    Beta    = 2,
    /// Third threshold, 375mV about 30 minI
    Veto    = 3,
    /// Use u8::MAX for Unknown, since 0 is pre-determined for 
    /// "NoHit, 
    Unknown = 255,
  };
  
  std::ostream& operator<<(std::ostream& os, const gondola::LTBThreshold& thresh);

  
  /// GAPS Trigger types/sources. Description
  /// can be found elsewhere. More than oen
  /// of them can be active at the same time
  enum class TriggerType : u8 {
    Unknown      = 0,
    /// -> 1-10 "pysics" triggers
    Any          = 1,
    Track        = 2,
    TrackCentral = 3,
    Gaps         = 4,
    /// > 100 -> Debug triggers
    Poisson      = 100,
    Forced       = 101, 
  };

  std::ostream& operator<<(std::ostream& os, const gondola::TriggerType& t_type);
}
