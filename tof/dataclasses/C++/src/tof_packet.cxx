#include "spdlog/spdlog.h"

#include "packets/tof_packet.h"
#include "serialization.h"


std::string packet_type_to_string(PacketType pt) {
  switch (pt) { 
    case PACKET_TYPE_UNKNOWN : {
      return "Unknown";
    }
    case PACKET_TYPE_COMMAND : {
      return "Command";
    }      
    case PACKET_TYPE_RBEVENT : {
      return "RBEvent";
    }      
    case PACKET_TYPE_TOFEVENT : {
      return "TofEvent";
    }      
    case PACKET_TYPE_MONITOR : {
      return "Monitor";
    }      
    case PACKET_TYPE_HEARTBEAT : {
      return "Heartbeat";
    }      
    case PACKET_TYPE_SCALAR : {
      return "Scalar";
    }      
    case PACKET_TYPE_MT : {
      return "MasterTriggerEvent";
    }      
    case PACKET_TYPE_RBHEADER : {
      return "RBEventHeader";
    }
    case PACKET_TYPE_TOFCMP_MONI : {
      return "TofCmpMoni";
    }
    case PACKET_TYPE_MTB_MONI : {
      return "MtbMoni";
    }
    case PACKET_TYPE_RB_MONI : {
      return "RBMoni";
    }
  }
  return "Unknown";
}

/**************************************************/

vec_u8 TofPacket::to_bytestream() const
{

  // first we need to hold only 5 bytes, then 
  // the payload will grow the vector with "insert"
  vec_u8 buffer(p_size_fixed);
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

u16 TofPacket::from_bytestream(vec_u8 &bytestream,
                               usize   start_pos){ 
  usize pos = start_pos;
  u16 value = decode_ushort(bytestream, start_pos);
  if (!(value == head)) {
    spdlog::error("No header found!");
    return start_pos;
  }
  head = value;
  pos += 2; // position in bytestream, 2 since we 
  packet_type = bytestream[pos]; pos+=1;
  //std::cout << "found packet type : " << packet_type << std::endl;
  payload_size = u32_from_le_bytes(bytestream, pos); pos+=4;
  //std::cout << "found payload size " << payload_size << std::endl;
  //size_t payload_end = pos + bytestream.size() - 2;
  usize payload_end = pos + payload_size;
  //std::cout << " found payload end " << payload_end << std::endl;
  vec_u8 packet_bytestream(bytestream.begin()+ pos,
                           bytestream.begin()+ payload_end)  ;
  payload = packet_bytestream;
  pos += payload_size;
  tail = decode_ushort(bytestream, pos); pos +=2;
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

