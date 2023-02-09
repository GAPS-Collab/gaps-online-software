#ifndef MASTERTRIGGERPACKET_H_INCLUDED
#define MASTERTRIGGERPACKET_H_INCLUDED

/**************************************
 *
 * MasterTrigger event format
 * Feb 2023
 * 
 ***********************/

#include "TofTypeDefs.h"

struct MasterTriggerPacket {
  u16 head = 0xAAAA;
  u16 tail = 0x5555;
  u32 event_id        ; 
  u32 timestamp       ; 
  u32 tiu_timestamp   ; 
  u32 gps_timestamp_32; 
  u32 gps_timestamp_16; 
  u32 board_mask      ;
  //idecoded_board_mask = [false;32];
  //hits         = [[false;32];32];

  u32 crc;
  //n_paddles = 0u8;
 
  /**
   * String representation
   *
   */
  std::string to_string() const;

  /**
   * Reset all fields to 0 values
   * FIXME - nan would be better
   */
  void reset();
  
  /**
   * Transcode to bytestream
   *
   *
   */
  [[deprecated("Use ::to_bytestream instead!")]]
  vec_u8 serialize() const;

  /**
   * Transcode from bytestream
   *
   * Returns:
   *    position where the event is found in the bytestream
   *    (tail position +=1, so that bytestream can be iterated
   *    over easily)
   */
  [[deprecated("Use ::from_bytestream instead!")]]
  u64 deserialize(vec_u8 &payload,
                  u64 start_pos=0);

  //! Byte representation of the packet
  vec_u8 to_bytestream() const;

  //! Decode packet from byte representation
  u64 from_bytestream(vec_u8 &payload, 
                      u64 start_pos=0);


};


#endif
