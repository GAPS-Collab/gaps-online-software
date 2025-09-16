#include <iostream>

#include "version.h"

std::string Gaps::pversion_to_string(Gaps::ProtocolVersion version) {
  std::string repr = "<ProtocolVersion: ";
  switch (version) {
    case Gaps::ProtocolVersion::Unknown : { 
      repr += "Unknown>";
      break;
    }
    case Gaps::ProtocolVersion::V1 : { 
      repr += "V1>";
      break;
    }
    case Gaps::ProtocolVersion::V2 : { 
      repr += "V2>";
      break;
    }
    case Gaps::ProtocolVersion::V3 : { 
      repr += "V3>";
      break;
    }
  }
  return repr;
}

std::ostream& operator<<(std::ostream& os, const Gaps::ProtocolVersion& version) {
  os << Gaps::pversion_to_string(version);
  return os;
}

