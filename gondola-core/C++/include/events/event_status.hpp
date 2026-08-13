/// This file is part of gaps-online-software and published 
/// under the GPLv3 license
#pragma once 

#include <format>
#include <sstream>
#include "tof_typedefs.h"

namespace gondola { 
  
  /// The event status indicates if there are technical 
  /// issues with the retrieval of the event.
  /// If there are no problems, events should have status
  /// EventStatus::EVENTSTATUS_PERFECT (42)
  enum class EventStatus : u8 {
    Unknown                =  0,
    Crc32Wrong             = 10,
    TailWrong              = 11,
    ChannelIDWrong         = 12, 
    CellSyncErrors         = 13,
    ChnSyncErrors          = 14,
    CellAndChnSyncErrors   = 15,
    AnyDataMangling        = 16,
    IncompleteReadout      = 21,
    IncompatibleData       = 22,
    EventTimeOut           = 23,
    GoodNoCRCOrErrBitCheck = 39,
    /// The event status is good, but we did not 
    /// perform any CRC32 check
    GoodNoCRCCheck         = 40,
    /// The event is good, but we did not perform
    /// error checks
    GoodNoErrBitCheck      = 41,
    Perfect                = 42, 
  };

  std::ostream& operator<<(std::ostream& os, const EventStatus& status);

} 

template <>
struct std::formatter<gondola::EventStatus> : std::formatter<std::string> {
  // Parse format specifiers (default implementation)
  constexpr auto parse(std::format_parse_context& ctx) {
      return ctx.begin();
  }
  
  auto format(const gondola::EventStatus& status, auto& ctx) const {
      std::ostringstream oss;
      oss << status;  // Use the << operator to convert enum to string
      return std::format_to(ctx.out(), "{}", oss.str());
  }
};

