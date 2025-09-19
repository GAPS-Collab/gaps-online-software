#ifndef TOFIO_H_INCLUDED
#define TOFIO_H_INCLUDED

#include <fstream>
#include <functional>

#include "result/result.h"

#include "events.h"
#include "packets/tof_packet.h"
#include "serialization.h"
#include "errors.hpp"

namespace r = result;

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

namespace gondola {
  /// Get all files in a certain directory in case it is a directory, for 
  /// a single file just get the file <3 ChatGPT
  ///
  /// # Arguments 
  ///   * input            : path or filename 
  ///   * use_telemetry_re : use the regex for telemetry files to find the files.
  ///                        Default is set to false, which will find L0 or TOF files
  auto list_path_contents_sorted(const std::string& input, bool use_telemetry_re = false) -> Vec<std::string>;
} 

/**
 * Extract only event ids from a bytestream with raw readoutboard binary data
 *
 * @param bytestream : Readoutboard binary (.robin) data.
 * @param start_pos  : Byte position to start searching from in bytestream
 */
[[deprecated("This might not even be correct!")]]
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
      void set_filename(String filename);
      /// Walk over the file and return the next packet
      auto get_next_packet() -> r::Result<TofPacket, Gaps::IOError>;
      /// Return the filename we assigned
      auto get_filename() const -> std::string;
      /// All packets have been read from the file. 
      /// If they should be read again, the reader 
      /// has to be created again
      auto is_exhausted() const -> bool;
      /// The number of files this reader has read
      /// from the file
      auto n_packets_read() const -> usize;

    private:
      std::ifstream  stream_file_;
      bool           exhausted_;
      usize          n_packets_read_;
      String         filename_;
  };
}


#endif
