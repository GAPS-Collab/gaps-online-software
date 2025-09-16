#ifndef VERSION_H_INCLUDED
#define VERSION_H_INCLUDED

#include "tof_typedefs.h"

namespace Gaps {

  static const u8 PROTOCOLVERSION_UNKNOWN = 0;
  static const u8 PROTOCOLVERSION_V1      = 64;
  static const u8 PROTOCOLVERSION_V2      = 128;
  static const u8 PROTOCOLVERSION_V3      = 192;
  
  enum class ProtocolVersion : u8 {
    Unknown = PROTOCOLVERSION_UNKNOWN,
    V1      = PROTOCOLVERSION_V1,
    V2      = PROTOCOLVERSION_V2,
    V3      = PROTOCOLVERSION_V3
  }; 

  std::string pversion_to_string(ProtocolVersion version);
}


std::ostream& operator<<(std::ostream& os, const Gaps::ProtocolVersion& version);

#endif
