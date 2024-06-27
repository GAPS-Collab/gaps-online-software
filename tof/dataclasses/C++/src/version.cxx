#include <iostream>

#include "version.h"

std::ostream& operator<<(std::ostream& os, const Gaps::ProtocolVersion& version) {
  os << "<ProtocolVersion: " ;
  switch (version) {
    case Gaps::ProtocolVersion::Unknown : { 
      os << "Unknown>";
      break;
    }
    case Gaps::ProtocolVersion::V1 : { 
      os << "V1>";
      break;
    }
    case Gaps::ProtocolVersion::V2 : { 
      os << "V2>";
      break;
    }
    case Gaps::ProtocolVersion::V3 : { 
      os << "V3>";
      break;
    }
  }
  return os;
}

