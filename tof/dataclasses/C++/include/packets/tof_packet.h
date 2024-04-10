#ifndef TOFPACKET_H_INCLUDED
#define TOFPACKET_H_INCLUDED

#include <cstdint>
#include <vector>

#include "tof_typedefs.h"

static const u8 UNKNOWN            =  0;
static const u8 COMMAND            = 10;
static const u8 RBEVENT            = 20;
static const u8 TOFEVENT           = 21;
static const u8 RBWAVEFORM         = 22;
static const u8 TOFEVENTSUMMARY    = 23;
static const u8 HEARTBEAT          = 40;
static const u8 SCALAR             = 50;
static const u8 MT                 = 60;
static const u8 RBHEADER           = 70;
static const u8 CPUMONIDATA        = 80;
static const u8 MTB_MONI           = 90;
static const u8 RB_MONI            = 100;
static const u8 PBMONIDATA         = 101;
static const u8 LTBMONIDATA        = 102;
static const u8 PAMONIDATA         = 103;
static const u8 RBEVENTPAYLOAD     = 110;
static const u8 RBEVENTMEMORYVIEW  = 120;
static const u8 RBCALIBRATION      = 130;

/// The PacketType is essential to identify 
/// individual TofPackets. This has to 
/// resemble the Rust API
enum class PacketType : u8 {
  Unknown           = UNKNOWN            ,
  Command           = COMMAND            ,
  RBEvent           = RBEVENT            ,
  TofEvent          = TOFEVENT           ,
  RBWaveform        = RBWAVEFORM         ,
  TofEventSummary   = TOFEVENTSUMMARY    ,
  HeartBeat         = HEARTBEAT          ,
  Scalar            = SCALAR             ,
  MasterTrigger     = MT                 ,
  RBHeader          = RBHEADER           ,
  CPUMoniData       = CPUMONIDATA        ,
  MTBMoni           = MTB_MONI           ,
  RBMoni            = RB_MONI            ,
  PBMoniData        = PBMONIDATA         , 
  LTBMoniData       = LTBMONIDATA        ,
  PAMoniData        = PAMONIDATA         , 
  RBEventPayload    = RBEVENTPAYLOAD     ,
  RBEventMemoryView = RBEVENTMEMORYVIEW  ,
  RBCalibration     = RBCALIBRATION      ,
};



/// String representation of enum "PacketType"
std::string packet_type_to_string(PacketType pt);

std::ostream& operator<<(std::ostream& os, const PacketType& pck);

/// Ensures that <T> has a method ::from_bytestream
template<typename T>
concept HasFromByteStream = requires(const Vec<u8>& stream, usize &pos) {
  { T::from_bytestream(stream, pos) } -> std::same_as<T>;
};

/// The most basic of all packets
/// 
/// A wrapper packet for an arbitrary bytestream
/// 
/// It looks like the following
/// 
/// HEAD    : u16 = 0xAAAA
/// TYPE    : u8  = PacketType
/// SIZE    : u64
/// PAYLOAD : [u8;6-SIZE]
/// TAIL    : u16 = 0x5555
/// => The packet has a fixed size of 9 bytes
/// => The packet has a size of 9 + PAYLOAD.size()
/// 
struct TofPacket {
  static const u16 HEAD = 0xAAAA;
  static const u16 TAIL = 0x5555;
  
  u16 head = 0xAAAA;
  u16 tail = 0x5555;

  // head (2) + tail (2) + type (1) + payload size (4)
  PacketType  packet_type; 
  // just the size of the payload, 
  // not including type, header or tail
  u32 payload_size;

  Vec<u8> payload;

  TofPacket();
  
  /// Transcode the bytestream into the respective 
  /// TofPacket
  static TofPacket from_bytestream(const Vec<u8> &bytestream,
                                   u64 &pos);

  /// A representative representation of the TofPacket 
  /// very usefule for debugging
  std::string to_string() const;
  
  /// A generic unpacking method - unpack everything which
  /// is stored within the payload 
  /// This requires that T has a ::from_bytestream method,
  /// which is enforced at compile time by 
  /// the concept HasFromByteStream
  template <HasFromByteStream T>
  T unpack() {
    usize pos = 0;
    return T::from_bytestream(payload, pos);
  }
}; // end TofPacket

std::ostream& operator<<(std::ostream& os, const TofPacket& pck);

#endif
