#include "packets/TofPacket.h"
#include "serialization.h"

/**************************************************/

std::vector<uint8_t> TofPacket::serialize() const
{

  // first we need to hold only 5 bytes, then 
  // the payload will grow the vector with "insert"
  std::vector<uint8_t> buffer(5);
  uint16_t pos = 0; // position in bytestream
  encode_ushort(head, buffer, pos); pos+=2;
  buffer[pos] = packet_type; pos += 1;
  //buffer.push_back(packet_type);    pos+=1;
  encode_ushort(payload_size, buffer, pos);  pos+=2;
  buffer.insert(buffer.end(), payload.begin(), payload.end()); pos += payload.size();
  std::cout << "buffer size " << buffer.size() << std::endl;
  encode_ushort(tail, buffer, pos); pos+=2;
  return buffer;
}

/**************************************************/

uint16_t TofPacket::deserialize(std::vector<uint8_t>& bytestream,
                                uint16_t start_pos)
{
    uint16_t value = decode_ushort(bytestream, start_pos);
    if (!(value == head))
        {std::cerr << "[ERROR] no header found!" << std::endl;}
    head = value;
    uint16_t pos = 2 + start_pos; // position in bytestream, 2 since we 
    packet_type = bytestream[pos]; pos+=1;
    //std::cout << "found packet type : " << packet_type << std::endl;
    payload_size = decode_ushort(bytestream, pos); pos+=2;
    //std::cout << "found payload size " << payload_size << std::endl;
    //size_t payload_end = pos + bytestream.size() - 2;
    size_t payload_end = pos + bytestream.size();
    //std::cout << " found payload end " << payload_end << std::endl;
    std::vector<uint8_t> packet_bytestream(bytestream.begin()+ pos,
                                           bytestream.begin()+ payload_end)  ;
    payload = packet_bytestream;
    tail = decode_ushort(bytestream, pos); pos +=2;
    return pos;
}

/**************************************************/

std::string TofPacket::to_string() const
{
   std::string repr = "TOFPACKET - type : ";
   repr += std::to_string (packet_type) + " - payload size " + std::to_string(payload_size);
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

