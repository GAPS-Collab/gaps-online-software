#ifndef TOFPACKET_H_INCLUDED
#define TOFPACKET_H_INCLUDED

#include <cstdint>
#include <vector>

#include "TofTypeDefs.h"


enum PacketType : u8 {
  Unknown   =  0,
  Command   = 10,
  RBEvent   = 20,
  Monitor   = 30,
  HeartBeat = 40 
};

/*********************************************************
 * The most basic of all packets
 *
 * A wrapper packet for an arbitrary bytestream
 * 
 * It looks like the following
 * 
 * HEAD    : u16 = 0xAAAA
 * TYPE    : u8  = PacketType
 * SIZE    : u64
 * PAYLOAD : [u8;6-SIZE]
 * TAIL    : u16 = 0x5555
 * => The packet has a fixed size of 9 bytes
 * => The packet has a size of 9 + PAYLOAD.size()
 */
struct TofPacket {
  
  u16 head = 0xAAAA;
  u16 tail = 0x5555;

  // head (2) + tail (2) + type (1) + payload size (4)
  u8  p_size_fixed = 9;
  u8  packet_type; 
  // just the size of the payload, 
  // not including type, header or tail
  u32 payload_size;

  vec_u8 payload;

  vec_u8 to_bytestream() const;

  /**
   * Transcode from bytestream
   *
   * Returns:
   *    position where the event is found in the bytestream
   *    (tail position +=1, so that bytestream can be iterated
   *    over easily)
   */
  u16 from_bytestream(vec_u8& payload,
                      usize start_pos=0);


  //! Just to be used for debugging - NO SERIALIZATION. 
  std::string to_string() const;

}; // end TofPacket

std::ostream& operator<<(std::ostream& os, const TofPacket& pck);

#endif
