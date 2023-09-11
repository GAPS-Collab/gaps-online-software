#ifndef SERIALIZATION_H_INCLUDED
#define SERIALIZATION_H_INCLUDED

/**
 * Serializer/Deserializers
 *
 *
 *
 */

#include <iostream>
#include <sstream>
#include <bitset>
#include <vector>
#include <numeric>
#include <assert.h>

#include "TOFCommon.h"
#include "tof_typedefs.h"

// this is the current size of a blobevent 
// in the serial representation
static const size_t BLOBEVENTSIZE=18530;

/************************************************/

/* TYPE DEFINITIONS */

typedef const Vec<u8> payload_t;
typedef Vec<u8> mutable_payload_t;

/***********************************************/

unsigned short decode_ushort(payload_t& bytestream,
                             unsigned int start_pos=0);

/***********************************************/

short decode_short(payload_t& bytestream,
                   unsigned int start_pos=0);

/***********************************************/

unsigned short decode_ushort_rev(payload_t& bytestream,
                             unsigned int start_pos=0);

/***********************************************/

short decode_short_rev(payload_t& bytestream,
                       unsigned int start_pos=0);

/***********************************************/

void encode_ushort(unsigned short value, mutable_payload_t& bytestream, unsigned int start_pos=0);
void encode_ushort_rev(unsigned short value, mutable_payload_t& bytestream, unsigned int start_pos=0);

/***********************************************/

uint32_t decode_uint32(payload_t& bytestream,
                       unsigned int start_pos=0);

/***********************************************/

[[deprecated("The assumed byteorder in this function is unclear/confusing")]]
uint32_t decode_uint32_rev(payload_t& bytestream,
                           unsigned int start_pos=0);

/***********************************************/

u32 u32_from_le_bytes(const Vec<u8> &bytestream,
                      u64 start_pos);

/***********************************************/

void u32_to_le_bytes(u32 value, Vec<u8> &bytestream, u8 start_pos);

void encode_uint32(uint32_t value, Vec<u8>& bytestream, unsigned int start_pos=0);
void encode_uint32_rev(uint32_t value, Vec<u8>& bytestream, unsigned int start_pos=0);

/***********************************************/

void encode_48(uint64_t value, Vec<u8>& bytestream, unsigned int start_pos=0);

/***********************************************/

void encode_48_rev(uint64_t value, Vec<u8>& bytestream, unsigned int start_pos=0);

/***********************************************/

uint64_t decode_uint64(payload_t& bytestream,
                       unsigned int start_pos=0);

/***********************************************/

[[deprecated("The assumed byteorder in this function is unclear/confusing")]]
uint64_t decode_uint64_rev(const Vec<u8>& bytestream,
                           unsigned int start_pos=0);

u64 u64_from_le_bytes(const Vec<u8> &bytestream,
		      usize start_pos=0);

/***********************************************/

void u64_to_le_bytes(u64 value, Vec<u8> &bytestream, u64 start_pos=0);

void encode_uint64(uint64_t value, Vec<u8>& bytestream, unsigned int start_pos=0);
void encode_uint64_rev(uint64_t value, Vec<u8>& bytestream, unsigned int start_pos=0);

//! encodes timestamp according to BlobEvent format - 48 bits instead of 64 and adds appropriate padding 
void encode_timestamp(uint64_t value, Vec<u8>& bytestream, unsigned int start_pos=0);

/***********************************************/

uint64_t decode_timestamp(Vec<u8>& bytestream, unsigned int start_pos=0);

/***********************************************/

uint16_t encode_12bitsensor(float value, float minrannge, float maxrange);

/***********************************************/

float decode_12bitsensor(uint16_t value, float minrange, float maxrange);

/***********************************************/

//int16_t encode_14bit(float value, float minrannge, float maxrange);

/***********************************************/

int16_t decode_14bit(const Vec<u8>& bytestream,
                     unsigned int start_pos=0);

/***********************************************/

void encode_blobevent(const BlobEvt_t* evt, std::vector<uint8_t> &bytestream, unsigned int start_pos);

/***********************************************/

[[deprecated("Use RBEventMemoryView::from_bytestream instead!")]]
BlobEvt_t decode_blobevent(const Vec<u8> &bytestream,
                           u32 start_pos);

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

std::vector<uint32_t> get_2byte_markers_indices(const std::vector<uint8_t> &bytestream, uint8_t marker);

// file i/o
/***********************************************/
bytestream get_bytestream_from_file(const std::string &filename);


#endif
