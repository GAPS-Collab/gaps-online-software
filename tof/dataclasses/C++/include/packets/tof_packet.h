#ifndef TOFPACKET_H_INCLUDED
#define TOFPACKET_H_INCLUDED

#include <cstdint>
#include <vector>

#include "tof_typedefs.h"

static const u8 PACKET_TYPE_UNKNOWN     =  0;
static const u8 PACKET_TYPE_COMMAND     = 10;
static const u8 PACKET_TYPE_RBEVENT     = 20;
static const u8 PACKET_TYPE_TOFEVENT    = 21;
static const u8 PACKET_TYPE_MONITOR     = 30;
static const u8 PACKET_TYPE_HEARTBEAT   = 40;
static const u8 PACKET_TYPE_SCALAR      = 50;
static const u8 PACKET_TYPE_MT          = 60;
static const u8 PACKET_TYPE_RBHEADER    = 70;
static const u8 PACKET_TYPE_TOFCMP_MONI = 80;
static const u8 PACKET_TYPE_MTB_MONI    = 90;
static const u8 PACKET_TYPE_RB_MONI     = 100;

enum class PacketType : u8 {
  Unknown       = PACKET_TYPE_UNKNOWN,
  Command       = PACKET_TYPE_COMMAND,
  RBEvent       = PACKET_TYPE_RBEVENT,
  TofEvent      = PACKET_TYPE_TOFEVENT,
  Monitor       = PACKET_TYPE_MONITOR,
  HeartBeat     = PACKET_TYPE_HEARTBEAT,
  Scalar        = PACKET_TYPE_SCALAR,
  MasterTrigger = PACKET_TYPE_MT,
  RBHeader      = PACKET_TYPE_RBHEADER,
  MonitorRb     = PACKET_TYPE_RB_MONI,
  MonitorTofCmp = PACKET_TYPE_TOFCMP_MONI,
  MonitorMtb    = PACKET_TYPE_MTB_MONI 
};


/**
 * String representation of enum "PacketType"
 *
 */ 
std::string packet_type_to_string(PacketType pt);

std::ostream& operator<<(std::ostream& os, const PacketType& pck);


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
  static const u16 HEAD = 0xAAAA;
  static const u16 TAIL = 0x5555;
  
  u16 head = 0xAAAA;
  u16 tail = 0x5555;

  // head (2) + tail (2) + type (1) + payload size (4)
  u8  p_size_fixed = 9;
  PacketType  packet_type; 
  // just the size of the payload, 
  // not including type, header or tail
  u32 payload_size;

  Vec<u8> payload;

  Vec<u8> to_bytestream() const;

  //[[deprecated("Use static from_bytestream instead!")]]
  //u64 from_bytestream(const Vec<u8> &payload,
  //                    usize start_pos=0);


  /**
   * Transcode from bytestream
   *
   */
  static TofPacket from_bytestream(const Vec<u8> &bytestream,
                                   u64 &pos);

  //! Just to be used for debugging - NO SERIALIZATION. 
  std::string to_string() const;

}; // end TofPacket

std::ostream& operator<<(std::ostream& os, const TofPacket& pck);

#endif
