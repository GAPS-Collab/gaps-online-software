#include "spdlog/spdlog.h"

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
      case PacketType::Monitor : {
      return "Monitor";
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
      case PacketType::TOFCmpMoni : {
      return "TofCmpMoni";
    }
      case PacketType::MTBMoni : {
      return "MtbMoni";
    }
      case PacketType::RBMoni : {
      return "RBMoni";
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

Vec<u8> TofPacket::to_bytestream() const
{

  // first we need to hold only 5 bytes, then 
  // the payload will grow the vector with "insert"
  Vec<u8> buffer(p_size_fixed);
  //buffer.reserve(p_size_fixed + payload.size());
  spdlog::debug("Will add payload of size {}", payload.size());
  usize pos = 0; // position in bytestream
  encode_ushort(head, buffer, pos); pos+=2;
  buffer[pos] = static_cast<u8>(packet_type); pos += 1;
  u32_to_le_bytes(payload_size, buffer, pos);  pos+=4;

  //std::cout << "buffer size " << buffer.size() << std::endl;
  //std::cout << "payload size " << payload.size() << std::endl;
  buffer.insert(buffer.begin() + 7, payload.begin(), payload.end()); pos += payload.size();
  //std::cout << "buffer size " << buffer.size() << std::endl;
  encode_ushort(tail, buffer, pos); pos+=2;
  //std::cout << "buffer size " << buffer.size() << std::endl;
  spdlog::info("TofPacket of size {}", buffer.size());
  return buffer;
}

/**************************************************/

//u64 TofPacket::from_bytestream(const Vec<u8> &bytestream,
//                               usize          start_pos){ 
//  usize pos = start_pos;
//  u16 value = Gaps::parse_u16(bytestream, pos);
//  if (!(value == head)) {
//    spdlog::error("No header found!");
//    /// print out the next/pre 5 bytes
//    spdlog::error("Byte! {}",bytestream[pos -5]);
//    spdlog::error("Byte! {}",bytestream[pos -4]);
//    spdlog::error("Byte! {}",bytestream[pos -3]);
//    spdlog::error("Byte! {}",bytestream[pos -2]);
//    spdlog::error("Byte! {}",bytestream[pos -1]);
//    spdlog::error("Byte! {}",bytestream[pos ]);
//    spdlog::error("Byte! {}",bytestream[pos +1]);
//    spdlog::error("Byte! {}",bytestream[pos +2]);
//    spdlog::error("Byte! {}",bytestream[pos +3]);
//    spdlog::error("Byte! {}",bytestream[pos +4]);
//
//    
//    return start_pos;
//  }
//  head = value;
//  //pos += 2; // position in bytestream, 2 since we 
//  packet_type = bytestream[pos]; pos+=1;
//  //std::cout << "found packet type : " << packet_type << std::endl;
//  //payload_size = u32_from_le_bytes(bytestream, pos); pos+=4;
//  payload_size = Gaps::parse_u32(bytestream, pos);
//  spdlog::info("Found TofPacket with {} bytes payload!", payload_size);
//  //std::cout << "found payload size " << payload_size << std::endl;
//  //size_t payload_end = pos + bytestream.size() - 2;
//  usize payload_end = pos + payload_size;
//  //std::cout << " found payload end " << payload_end << std::endl;
//  Vec<u8> packet_bytestream(bytestream.begin()+ pos,
//                            bytestream.begin()+ payload_end)  ;
//  payload = packet_bytestream;
//  pos += payload_size;
//  tail = Gaps::parse_u16(bytestream, pos);
//  if (tail != 0x5555)
//    {spdlog::error("Tail wrong! ");}
//  return pos;
//}

/**************************************************/

TofPacket TofPacket::from_bytestream(const Vec<u8> &bytestream,
                                     u64           &pos){ 
  TofPacket packet = TofPacket();
  u16 value = Gaps::parse_u16(bytestream, pos);
  if (value != TofPacket::HEAD) {
    spdlog::error("No header found for position {}! Bytes are {} {}. Decoded to {}.", pos, bytestream[pos], bytestream[pos+1], value);
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
  spdlog::debug("Found TofPacket of type {} with {} bytes payload!", packet_type_to_string(packet.packet_type), packet.payload_size);
  usize payload_end = pos + packet.payload_size;
  Vec<u8> packet_bytestream(bytestream.begin()+ pos,
                            bytestream.begin()+ payload_end)  ;
  packet.payload = packet_bytestream;
  pos += packet.payload_size;
  u16 tail = Gaps::parse_u16(bytestream, pos);
  if (tail != TofPacket::TAIL)
    {spdlog::error("Tail wrong! ");}
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

