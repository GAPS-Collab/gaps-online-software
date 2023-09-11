#ifndef TOFIO_H_INCLUDED
#define TOFIO_H_INCLUDED

#include <fstream>

#include "events.h"
#include "packets/tof_packet.h"
#include "TOFCommon.h"

/**
 * Extract tof dataclasses from files
 */

/**
 * Extract "BlobEvents" from a bytestream
 *
 */
[[deprecated("Use RBEventMemoryView instead of BlobEvt_t!")]]
Vec<BlobEvt_t> get_events_from_stream(const Vec<u8> &bytestream, u64 start_pos);

/**
 * Get RBEventMemoryViews from a raw data ("*.robin") file
 *
 * This file must have the raw memory data from the readoutboards 
 * directly written to the file, without it being packed in 
 * TofPackets
 * 
 * @param filename : Full path to file with RB binary data
 */
Vec<RBEventMemoryView> get_rbeventmemoryviews(const String &filename, bool omit_duplicates = false);


/**
 * Get RBEventMemoryViews from a vector of bytes
 *
 * @param stream : 
 * @param pos    : 
 */
Vec<RBEventMemoryView> get_rbeventmemoryviews(const Vec<u8> &stream, u64 start_pos, bool omit_duplicates = false);


/**
 * Read event headers from a RB binary file
 *
 */
Vec<RBEventHeader> get_rbeventheaders(const String &filename, bool is_header=false);

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
 */
Vec<TofPacket> get_tofpackets(const Vec<u8> &bytestream, u64 start_pos);

/**
 * Extract TofPackets from a file on disk
 *
 * @param bytestream : Binary TofPacket data.
 * @param start_pos  : Byte position to start searching from in bytestream
 */
Vec<TofPacket> get_tofpackets(const String filename);

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
      TofPacketReader(String filename);
      void process_chunk();
      TofPacket get_next_packet();
      String get_filename() const;
    private:
      String filename_;
      usize  file_size_;
      usize  nchunks_;
      usize  current_pos_;
      TofPacket last_packet_;
      std::ifstream stream_file_;
  };
}


#endif
