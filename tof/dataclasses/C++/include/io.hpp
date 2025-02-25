#ifndef TOFIO_H_INCLUDED
#define TOFIO_H_INCLUDED

#include <fstream>
#include <functional>

#include "events.h"
#include "packets/tof_packet.h"
#include "serialization.h"


//template<typename T>
//requires HasFromByteStream<T>
//Vec<T> unpack<T>(String filename) {
//  usize pos = 0;
//  auto packets = get_tofpackets(filename);
//  for (const auto &p : packets) {
//    T data = T::from_bytestream(p.payload, 0);
// 
//
//  Vec<T> data;
//  return data;
//}

/**
 * Extract tof dataclasses from files
 */


/**
 * Read event headers from a RB binary file
 *
 */
Vec<RBEventHeader> get_rbeventheaders(const String &filename, bool is_header=false);

/**
 * Generic extractor for all types of deserializable dataclasse
 */ 
//typedef


/**
 * Extract only event ids from a bytestream with raw readoutboard binary data
 *
 * @param bytestream : Readoutboard binary (.robin) data.
 * @param start_pos  : Byte position to start searching from in bytestream
 */
Vec<u32> get_event_ids_from_raw_stream(const Vec<u8> &bytestream, u64 &start_pos);

/**
 * Extract TofPackets from a stream of binary data 
 *
 * @param bytestream : Binary TofPacket data.
 * @param start_pos  : Byte position to start searching from in bytestream
 * @param filter     : Only get TofPackets of this type. If set to 
 *                     PacketType::Unknown, get all packets
 */
Vec<TofPacket> get_tofpackets(const Vec<u8> &bytestream, u64 start_pos, PacketType filter=PacketType::Unknown);

/**
 * Extract TofPackets from a file on disk
 *
 * @param bytestream : Binary TofPacket data.
 * @param start_pos  : Byte position to start searching from in bytestream
 * @param filter     : Only get TofPackets of this type. If set to 
 *                     PacketType::Unknown, get all packets
 */
Vec<TofPacket> get_tofpackets(const String filename, PacketType filter = PacketType::Unknown);

/**
 * Directly gets TofEvents from a stream with tofpackets, assuming all
 * packets are actually TofEvents. Other packets will be discarded.
 *
 * @param bytestream : Binary TofPacket data.
 * @param start_pos  : Byte position to start searching from in bytestream
 */
Vec<TofEvent> unpack_tofevents_from_tofpackets(const Vec<u8> &bytestream, u64 start_pos);

/**
 * Directly gets TofEvents from a stream with tofpackets, assuming all
 * packets are actually TofEvents. Other packets will be discarded
 *
 * @param filename : Binary file with TofPacket data.
 */
Vec<TofEvent> unpack_tofevents_from_tofpackets(const String filename);

///**
// * Extract TofEvents from a stream of binary data 
// *
// * @param bytestream : Binary TofEvent data.
// * @param start_pos  : Byte position to start searching from in bytestream
// */
//Vec<TofPacket> get_tofpackets(const Vec<u8> &bytestream, u64 start_pos);
//
///**
// * Extract TofEvents from a file on disk
// *
// * @param filename : Full path to binary file with TofEvents.
// */
//Vec<TofPacket> get_tofpackets(const String filename);

namespace Gaps {

  /// Read serialized TofPackets from 
  /// a file and emit them as packets
  class TofPacketReader {
    public: 
      TofPacketReader();
      TofPacketReader(String filename);
      TofPacketReader(const TofPacketReader&) = delete;
      //TofPacketReader& operator=(const TofPacketReader&) = delete;
      /// Set a filename where to read packets from. This is a binary file format,
      /// typically ending in ".tof.gaps"
      void      set_filename(String filename);
      /// Walk over the file and return the next packet
      TofPacket get_next_packet();
      /// Return the filename we assigned
      String    get_filename() const;
      /// All packets have been read from the file. 
      /// If they should be read again, the reader 
      /// has to be created again
      bool      is_exhausted() const;
      /// The number of files this reader has read
      /// from the file
      usize     n_packets_read() const;

    private:
      std::ifstream  stream_file_;
      bool           exhausted_;
      usize          n_packets_read_;
      String         filename_;
  };
}


#endif
