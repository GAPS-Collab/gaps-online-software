#ifndef TOFEVENTS_H_INCLUDED
#define TOFEVENTS_H_INCLUDED

#include "tof_typedefs.h"

/**
 * Extract only event ids from a bytestream with raw readoutboard binary data
 *
 * @param bytestream : Readoutboard binary (.robin) data.
 * @param start_pos  : Byte position to start searching from in bytestream
 */
vec_u32 get_event_ids_from_raw_stream(const vec_u8 &bytestream, u64 &start_pos);

#endif 
