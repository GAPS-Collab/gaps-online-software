#ifndef TOFPACKET_H_INCLUDED
#define TOFPACKET_H_INCLUDED

#include <cstdint>
#include <vector>

#include "tof_typedefs.h"

static const u8 UNKNOWN            =  0;
static const u8 COMMAND            = 10;
static const u8 RBEVENT            = 20;
static const u8 TOFEVENT           = 21;
static const u8 MONITOR            = 30;
static const u8 HEARTBEAT          = 40;
static const u8 SCALAR             = 50;
static const u8 MT                 = 60;
static const u8 RBHEADER           = 70;
static const u8 TOFCMP_MONI        = 80;
static const u8 MTB_MONI           = 90;
static const u8 RB_MONI            = 100;
static const u8 PBMONIDATA         = 101;
static const u8 LTBMONIDATA        = 102;
static const u8 PAMONIDATA         = 103;
static const u8 RBEVENTPAYLOAD     = 110;
static const u8 RBEVENTMEMORYVIEW  = 120;
static const u8 RBCALIBRATION      = 130;


enum class PacketType : u8 {
  Unknown           = UNKNOWN            ,
  Command           = COMMAND            ,
  RBEvent           = RBEVENT            ,
  TofEvent          = TOFEVENT           ,
  Monitor           = MONITOR            ,
  HeartBeat         = HEARTBEAT          ,
  Scalar            = SCALAR             ,
  MasterTrigger     = MT                 ,
  RBHeader          = RBHEADER           ,
  TOFCmpMoni        = TOFCMP_MONI        ,
  MTBMoni           = MTB_MONI           ,
  RBMoni            = RB_MONI            ,
  PBMoniData        = PBMONIDATA         , 
  LTBMoniData       = LTBMONIDATA        ,
  PAMoniData        = PAMONIDATA         , 
  RBEventPayload    = RBEVENTPAYLOAD     ,
  RBEventMemoryView = RBEVENTMEMORYVIEW  ,
  RBCalibration     = RBCALIBRATION      ,
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

  /**
   * Transcode from bytestream
   *
   */
  static TofPacket from_bytestream(const Vec<u8> &bytestream,
                                   u64 &pos);

  //! Just to be used for debugging - NO SERIALIZATION. 
  std::string to_string() const;
  
  //! A generic unpacking method - unpack everything which
  //! is stored within the payload 
  template <typename T>
  T unpack() {
    // Check if T has a 'from_bytestream' method.
    static_assert(
        std::is_member_function_pointer<decltype(&T::from_bytestream)>::value,
        "T must have a 'from_bytestream' method.");
    return T::from_bytestream(payload, 0);
  }
}; // end TofPacket

std::ostream& operator<<(std::ostream& os, const TofPacket& pck);

#endif
