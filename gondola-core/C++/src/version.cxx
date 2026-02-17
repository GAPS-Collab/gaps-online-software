#include <iostream>

#include "version.h"

namespace g = gondola;

std::string g::pversion_to_string(g::ProtocolVersion version) {
  std::string repr = "<ProtocolVersion: ";
  switch (version) {
    case g::ProtocolVersion::Unknown : { 
      repr += "Unknown>";
      break;
    }
    case g::ProtocolVersion::V1 : { 
      repr += "V1>";
      break;
    }
    case g::ProtocolVersion::V2 : { 
      repr += "V2>";
      break;
    }
    case g::ProtocolVersion::V3 : { 
      repr += "V3>";
      break;
    }
  }
  return repr;
}

std::ostream& operator<<(std::ostream& os, const g::ProtocolVersion& version) {
  os << g::pversion_to_string(version);
  return os;
}

