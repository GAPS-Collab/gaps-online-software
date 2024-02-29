#include "logging.hpp"
#include "packets/tof_packet.h"
#include "serialization.h"
#include "parsers.h"

// this is just stupid imo
//https://stackoverflow.com/questions/4891067/weird-undefined-symbols-of-static-constants-inside-a-struct-class
const u16 TofPacket::HEAD;
const u16 TofPacket::TAIL;

std::string packet_type_to_string(const PacketType pt) {
  switch (pt) { 
      case PacketType::Unknown : {
      return "Unknown";
    }
      case PacketType::Command : {
      return "Command";
    }      
      case PacketType::RBEvent : {
      return "RBEvent";
    }      
      case PacketType::TofEvent : {
      return "TofEvent";
    }      
      case PacketType::RBWaveform : {
      return "RBWaveform";
    }      
      case PacketType::HeartBeat : {
      return "Heartbeat";
    }      
      case PacketType::Scalar : {
      return "Scalar";
    }      
      case PacketType::MasterTrigger : {
      return "MasterTriggerEvent";
    }      
      case PacketType::RBHeader : {
      return "RBEventHeader";
    }
      case PacketType::CPUMoniData : {
      return "CPUMoniData";
    }
      case PacketType::MTBMoni : {
      return "MtbMoni";
    }
      case PacketType::RBMoni : {
      return "RBMoni";
    }
      case PacketType::PBMoniData : {
      return "PBMoniData";
    }
      case PacketType::LTBMoniData : {
      return "LTBMoniData";
    }
      case PacketType::PAMoniData : {
      return "PAMoniData";
    }
      case PacketType::RBCalibration : {
      return "RBCalibration";
    }
      case PacketType::RBEventMemoryView : {
      return "RBEventMemoryView";
    }
      case PacketType::RBEventPayload : {
      return "RBEventMemoryView";
    }
  }
  return "Unknown";
}

/**************************************************/

std::ostream& operator<<(std::ostream& os, const PacketType& pck)
{
  os << packet_type_to_string(pck);
  return os;
}

/**************************************************/

TofPacket::TofPacket() {
  packet_type = PacketType::Unknown;
  payload_size = 0;
  payload = {};
}

/**************************************************/

TofPacket TofPacket::from_bytestream(const Vec<u8> &bytestream,
                                     u64           &pos){ 
  TofPacket packet = TofPacket();
  if (bytestream.size() <= pos) {
    log_debug("Bytestream exhausted, returning empty packet!");
    return packet;
  }
  if (bytestream.size() <= pos + 1) {
    log_debug("Bytestream exhausted, returning empty packet!");
    return packet;
  }
  if (bytestream.size() <= pos + 2) {
    log_debug("Bytestream exhausted, returning empty packet!");
    return packet;
  }
  u16 value = Gaps::parse_u16(bytestream, pos);
  if (value != TofPacket::HEAD) {
    log_warn("No header found for position " << pos << "! Bytes are " << bytestream[pos] << " " << bytestream[pos+1] << ". Decoded to " << value << " Returning EMPTY packet!");
    /// print out the next/pre 5 bytes
    //spdlog::error("Byte! {}",bytestream[pos -5]);
    //spdlog::error("Byte! {}",bytestream[pos -4]);
    //spdlog::error("Byte! {}",bytestream[pos -3]);
    //spdlog::error("Byte! {}",bytestream[pos -2]);
    //spdlog::error("Byte! {}",bytestream[pos -1]);
    //spdlog::error("Byte! {}",bytestream[pos ]);
    //spdlog::error("Byte! {}",bytestream[pos +1]);
    //spdlog::error("Byte! {}",bytestream[pos +2]);
    //spdlog::error("Byte! {}",bytestream[pos +3]);
    //spdlog::error("Byte! {}",bytestream[pos +4]);
    pos -= 2; // rewind position so that client knows we did not 
              // parse anything
    return packet;
  }
  packet.head = value;
  packet.packet_type  = static_cast<PacketType>(bytestream[pos]); pos+=1;
  packet.payload_size = Gaps::parse_u32(bytestream, pos);
  log_debug("Found TofPacket of type " << packet_type_to_stirng(packet.packet_type) << " with " << packet.payload_size << " bytes payload!");
  usize payload_end = pos + packet.payload_size;
  Vec<u8> packet_bytestream(bytestream.begin()+ pos,
                            bytestream.begin()+ payload_end)  ;
  packet.payload = packet_bytestream;
  pos += packet.payload_size;
  u16 tail = Gaps::parse_u16(bytestream, pos);
  if (tail != TofPacket::TAIL)
    {log_error("TofPacket doesn't conclude with TAIL signature of " << TofPacket::TAIL);}
  return packet;
}

/**************************************************/

std::string TofPacket::to_string() const
{
   std::string repr = "<TofPacket - type : ";
   repr += packet_type_to_string(static_cast<PacketType>(packet_type)) + " - payload size : " + std::to_string(payload_size) + ">";
   return repr;

}

/**************************************************/

std::ostream& operator<<(std::ostream& os, const TofPacket& pck)
{ 
  os << pck.to_string();
  return os;
}

/**************************************************/

