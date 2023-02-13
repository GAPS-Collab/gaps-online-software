#ifndef GAPSPARSERS_H_INCLUDED
#define GAPSPARSERS_H_INCLUDED

#include "TofTypeDefs.h"


namespace Gaps {


u16 u16_from_le_bytes(const vec_u8 &bytestream,
                      u64 pos);

void u16_to_le_bytes(const u16 value, 
                     vec_u8 &bytestream,
                     usize &pos);



/**
 * Get an u32 from a vector of bytes. 
 *
 * The byteorder is compatible with the 
 * rust from_le_bytes, where
 * 
 * let value = u32::from_le_bytes([0x78, 0x56, 0x34, 0x12]);
 * assert_eq!(value, 0x12345678);
 *
 * @params: 
 *
 * @pos : this gets advanced by 4 bytes
 *
 *
 */
u32 u32_from_le_bytes(const vec_u8 &bytestream,
                      usize &pos);


void u32_to_le_bytes(const u32 value, 
                     vec_u8 &bytestream,
                     usize &pos);

}


#endif
