#ifndef TOFPACKET_H_INCLUDED
#define TOFPACKET_H_INCLUDED

#include <cstdint>
#include <vector>

#include "RBEnvPacket.h"

enum TofPacketType : uint8_t {
    UNKNOWN     = 0,
    EVENT       = 10,
    ENVIRONMENT = 20
};

struct TofPacket {
  
   uint16_t head = 0xAAAA;
   uint16_t tail = 0x5555;

   uint8_t  packet_type; 
   // just the size of the payload, 
   // not iuncluding type, header or tail
   uint16_t payload_size;

   std::vector<uint8_t> payload;

   std::vector<uint8_t> serialize() const;

   /**
    * Transcode from bytestream
    *
    * Returns:
    *    position where the event is found in the bytestream
    *    (tail position +=1, so that bytestream can be iterated
    *    over easily)
    */
   uint16_t deserialize(std::vector<uint8_t>& payload,
                            uint16_t start_pos=0);


   //! Just to be used for debugging - NO SERIALIZATION. 
   std::string to_string() const;

}; // end TofPacket

std::ostream& operator<<(std::ostream& os, const TofPacket& pck);

#endif
