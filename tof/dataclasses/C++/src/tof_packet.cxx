#include "spdlog/spdlog.h"

#include "packets/tof_packet.h"
#include "serialization.h"
#include "parsers.h"

std::string packet_type_to_string(PacketType pt) {
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
      case PacketType::MonitorTofCmp : {
      return "TofCmpMoni";
    }
      case PacketType::MonitorMtb : {
      return "MtbMoni";
    }
      case PacketType::MonitorRb : {
      return "RBMoni";
    }
  }
  return "Unknown";
}

/**************************************************/

Vec<u8> TofPacket::to_bytestream() const
{

  // first we need to hold only 5 bytes, then 
  // the payload will grow the vector with "insert"
  Vec<u8> buffer(p_size_fixed);
  //buffer.reserve(p_size_fixed + payload.size());
  std::cout << "PAYLOAD SIZE " << payload.size() << std::endl;
  usize pos = 0; // position in bytestream
  encode_ushort(head, buffer, pos); pos+=2;
  buffer[pos] = packet_type; pos += 1;
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

u16 TofPacket::from_bytestream(const Vec<u8> &bytestream,
                               usize          start_pos){ 
  usize pos = start_pos;
  u16 value = Gaps::parse_u16(bytestream, pos);
  if (!(value == head)) {
    spdlog::error("No header found!");
    return start_pos;
  }
  head = value;
  //pos += 2; // position in bytestream, 2 since we 
  packet_type = bytestream[pos]; pos+=1;
  //std::cout << "found packet type : " << packet_type << std::endl;
  //payload_size = u32_from_le_bytes(bytestream, pos); pos+=4;
  payload_size = Gaps::parse_u32(bytestream, pos);
  spdlog::info("Found TofPacket with {} bytes payload!", payload_size);
  //std::cout << "found payload size " << payload_size << std::endl;
  //size_t payload_end = pos + bytestream.size() - 2;
  usize payload_end = pos + payload_size;
  //std::cout << " found payload end " << payload_end << std::endl;
  Vec<u8> packet_bytestream(bytestream.begin()+ pos,
                            bytestream.begin()+ payload_end)  ;
  payload = packet_bytestream;
  pos += payload_size;
  tail = Gaps::parse_u16(bytestream, pos);
  if (tail != 0x5555)
    {spdlog::error("Tail wrong! ");}
  return pos;
}

/**************************************************/

std::string TofPacket::to_string() const
{
   std::string repr = "TOFPACKET - type : ";
   repr += packet_type_to_string(static_cast<PacketType>(packet_type)) + " - payload size " + std::to_string(payload_size);
   return repr;

}

/**************************************************/

std::ostream& operator<<(std::ostream& os, const TofPacket& pck)
{
  os << "--TOFPACKET--\n";
  os << "-- type " << pck.packet_type << "\n";
  return os;
}

/**************************************************/

