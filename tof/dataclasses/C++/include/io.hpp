#ifndef TOFIO_H_INCLUDED
#define TOFIO_H_INCLUDED

#include "events.h"
#include "packets/tof_packet.h"

/**
 * Extract tof dataclasses from files
 */

/**
 * Read event headers from a RB binary file
 *
 */
Vec<RBEventHeader> get_headers(const String &filename, bool is_header=false);

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

#endif
