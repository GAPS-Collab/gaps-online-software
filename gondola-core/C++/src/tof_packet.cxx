#include <format>
#include "spdlog/spdlog.h"
#include "spdlog/cfg/env.h"

#include "packets/tof_packet.h"
#include "serialization.h"
#include "io/parsers.h"

namespace g = gondola;
using namespace result;

auto packet_type_to_string(const PacketType pt) -> std::string {
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
      case PacketType::TofEventDeprecated : {
      return "TofEventDeprecated";
    }      
      case PacketType::RBWaveform : {
      return "RBWaveform";
    }      
      case PacketType::TofEvent : {
      return "TofEvent";
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

auto TofPacket::from_bytestream(const Vec<u8> &bytestream, u64 &pos) 
  -> Result<TofPacket, g::IOError> { 
  TofPacket packet = TofPacket();
  if (bytestream.size() <= pos + 2) {
    auto message = std::format("Bytestream is too short!");
    auto err = g::IOError(g::IOError::ErrorKind::StreamTooShort, message);
    return Err(err);
  }
  u16 head = g::parse_u16(bytestream, pos);
  if (head != TofPacket::HEAD) {
    auto message = std::format("Decoding of HEAD failed! Got {} instead!", head);
    auto err = g::IOError(g::IOError::ErrorKind::WrongHeaderBytes, message);
    pos -= 2; // rewind position so that client knows we did not 
              // parse anything
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
    return Err(err);
  }
  packet.head = head;
  packet.packet_type  = static_cast<PacketType>(bytestream[pos]); pos+=1;
  packet.payload_size = g::parse_u32(bytestream, pos);
  spdlog::debug("Found TofPacket of type {} with {} bytes payload!", packet_type_to_string(packet.packet_type), packet.payload_size);
  usize payload_end = pos + packet.payload_size;
  Vec<u8> packet_bytestream(bytestream.begin()+ pos,
                            bytestream.begin()+ payload_end)  ;
  packet.payload = packet_bytestream;
  pos += packet.payload_size;
  u16 tail = g::parse_u16(bytestream, pos);
  if (tail != TofPacket::TAIL) {
    auto message = std::format("Decoding of TAIL failed! Got {} instead!", tail);
    auto err = g::IOError(g::IOError::ErrorKind::WrongTailBytes, message);
    return Err(err);
  }
  return Ok(packet);
}

/**************************************************/

auto TofPacket::to_string() const -> std::string {
  std::string repr = "<TofPacket - type : ";
  repr += packet_type_to_string(static_cast<PacketType>(packet_type)) + " - payload size : " + std::to_string(payload_size) + " - actual payload " + std::to_string(payload.size()) + ">";
  return repr;
}

/**************************************************/

auto TofPacket::to_bytestream() const -> Vec<u8> {
  Vec<u8> stream = Vec<u8>();
  stream.reserve(6 + payload.size());
  g::bytestream_extend(stream, TofPacket::HEAD);
  u8 p_type = (u8)packet_type;
  stream.push_back(p_type);
  // payload size of 32 bit accomodates up to 4 GB packet
  // a 16 bit size would only hold 65k, which might be not
  // good enough if we sent multiple events in a batch in 
  // the same TofPacket (in case we do that)
  u32 payload_len = (u32)payload.size();
  g::bytestream_extend(stream, payload_len);
  stream.insert(stream.end(), payload.begin(), payload.end());
  g::bytestream_extend(stream, TofPacket::TAIL);
  return stream;
}

std::ostream& operator<<(std::ostream& os, const TofPacket& pck) { 
  os << pck.to_string();
  return os;
}
