/// This file is part of gaps-online-software and published 
/// under the GPLv3 license
#pragma once 

#include <format>

namespace gondola {
  
  /// EventQuality will get assigned by online reconstructions
  /// or the flight computer. This contains information about
  /// physics and might pre-select "golden" candidate events.
  /// The default event quelity is EventQuality::UNKNOWN
  enum class EventQuality : u8 {
    Unknown        =  0,
    Silver         = 10,
    Gold           = 20,
    Diamond        = 30,
    /// FourLeavClover events are events with exactly
    /// 4 hits in overlapping pannels. 2 overlapping 
    /// in the Umbrella/Cortina, 2 overlapping in the 
    /// TOF cube
    FourLeafClover = 40
  };

  std::ostream& operator<<(std::ostream& os, const gondola::EventQuality& qual);
}
