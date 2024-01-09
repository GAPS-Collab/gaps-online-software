#ifndef SERIALIZATION_H_INCLUDED
#define SERIALIZATION_H_INCLUDED

#include <iostream>
#include <sstream>
#include <bitset>
#include <vector>
#include <numeric>
#include <assert.h>

#include "tof_typedefs.h"

[[deprecated("The assumed byteorder in this function is unclear/confusing")]]
u64 decode_uint64_rev(const Vec<u8>& bytestream,
                           unsigned int start_pos=0);
/***********************************************/

/**
 * Idnentify the postion of a byte marker in a stream
 *
 * The bytemarker has to be the 2 same bytes 
 * (otherwise it would not be a good marker anyway)
 *
 * @param bytestrem : stream with raw binary data
 * @param marker    : 1 byte of the two byte pattern to
 *                    search for, eg. 0xAA 
 * @param has_ended : Indicate if the bytestream has been
 *                    traversed without finding the marker
 * @param start_pos : Start searching only after this position
 *                    in the bytestream
 * @param end_pos   : Restrict searching only until this position
 *                    in the bytestream                   
 */
u64 search_for_2byte_marker(
           const Vec<u8> &bytestream,
           u8 marker,
           bool &has_ended,
           u64 start_pos=0,
           u64 end_pos=0);

/***********************************************/

Vec<u32> get_2byte_markers_indices(const Vec<u8> &bytestream, u8 marker);

// file i/o
/***********************************************/
bytestream get_bytestream_from_file(const std::string &filename);


#endif
