#ifndef RBMONIPACKET_H_INCLUDED
#define RBMONIPACKET_H_INCLUDED

#include "TofTypeDefs.h"

/****************************************
 * Housekeeping data for the individual 
 * readout board
 *
 *
 */

struct RBMoniPacket {
  static const u16 HEAD = 0xAAAA;
  static const u16 TAIL = 0x5555;
  static const u8  SIZE = 6;

  u32 rate;

  vec_u8 to_bytestream() const;

  usize from_bytestream(vec_u8& payload,
                        usize start_pos=0);

};

#endif
